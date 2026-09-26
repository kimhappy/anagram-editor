use std::{rc::Rc, time::Duration};

use gloo_timers::future::sleep;
use leptos::prelude::*;
use send_wrapper::SendWrapper;
use wasm_bindgen_futures::spawn_local;

use super::{
    icon::{Icon, IconKind},
    ui::{Button, ButtonSize, ProgressBar},
};
use crate::{
    anagram::{Anagram, ConnectError, ConnectStep, DeviceChoice},
    backend::Backend,
    devices,
    lifecycle::Reconnect,
};

const RETRY_GAP: Duration = Duration::from_secs(3);

#[component]
pub fn ConnectScreen(
    #[prop(into)] on_connect: Callback<SendWrapper<Rc<dyn Backend>>>,
    awaiting: RwSignal<Option<Reconnect>>,
) -> impl IntoView {
    let step = RwSignal::new(None::<ConnectStep>);
    let error = RwSignal::new(None::<ConnectError>);
    let is_supported = devices::hid::is_supported() && devices::midi::is_supported();
    let is_connecting = move || step.with(Option::is_some);
    let is_mounted = StoredValue::new(true);
    on_cleanup(move || is_mounted.set_value(false));
    let retry = Retry {
        awaiting,
        is_mounted,
        progress: RwSignal::new(None),
        is_attempting: StoredValue::new(false),
        last_failure: RwSignal::new(None),
        error,
        on_connect,
    };
    let connect = move |_| {
        if retry.is_attempting.get_value() {
            return;
        }
        error.set(None);
        spawn_local(async move {
            if let Some(failure) = attempt(DeviceChoice::AskIfNeeded, step, on_connect).await {
                error.set(Some(failure));
            }
        });
    };
    if awaiting.with_untracked(Option::is_some) {
        spawn_local(keep_reconnecting(retry));
        let plugged = devices::hid::on_connect(move || spawn_local(attempt_again(retry))).ok();
        let plugged = StoredValue::new_local(plugged);
        on_cleanup(move || plugged.set_value(None));
    }

    view! {
        <main class="bg-muted/50 flex min-h-screen items-center justify-center p-6">
            <div class="grid w-full max-w-md justify-items-center gap-6 text-center">
                <ReconnectPanel retry />
                <div class="grid gap-2">
                    <h1 class="text-3xl font-medium tracking-tight">"Anagram Editor"</h1>
                    <p class="text-muted-foreground">
                        "Plug in your Anagram over USB, then connect. Firmware 1.18 or newer is required."
                    </p>
                </div>
                <div class="flex flex-wrap items-center justify-center gap-3">
                    <Button
                        size=ButtonSize::Lg
                        class="min-w-40"
                        disabled=Signal::derive(move || is_connecting() || !is_supported)
                        on:click=connect
                    >
                        {move || {
                            if is_connecting() {
                                view! {
                                    <Icon kind=IconKind::LoaderCircle class="animate-spin" />
                                    "Connecting…"
                                }
                                    .into_any()
                            } else {
                                view! {
                                    <Icon kind=IconKind::Usb />
                                    "Connect"
                                }
                                    .into_any()
                            }
                        }}
                    </Button>
                </div>
                <Show when=move || !is_supported>
                    <p class="text-destructive text-sm">
                        "This browser has no WebHID or Web MIDI. Use a Chromium-based browser such as Chrome or Edge."
                    </p>
                </Show>
                <div class="grid w-full gap-2" class:invisible=move || !is_connecting() aria-hidden=move || (!is_connecting()).to_string()>
                    <ProgressBar share=Signal::derive(move || step.get()?.share()) />
                    <p class="text-muted-foreground h-5 text-sm tabular-nums">
                        {move || step.get().map(|step| step.to_string())}
                    </p>
                </div>
                {move || error.get().map(|failure| view! { <ConnectFailure failure /> })}
                <Terms />
            </div>
        </main>
    }
}

async fn attempt(
    choice: DeviceChoice,
    step: RwSignal<Option<ConnectStep>>,
    on_connect: Callback<SendWrapper<Rc<dyn Backend>>>,
) -> Option<ConnectError> {
    step.set(Some(ConnectStep::RequestingHid));
    let outcome = Anagram::connect(choice, |progress| step.set(Some(progress))).await;
    step.set(None);
    match outcome {
        Ok(anagram) => {
            let backend: Rc<dyn Backend> = Rc::new(anagram);
            on_connect.run(SendWrapper::new(backend));
            None
        }
        Err(failure) => Some(failure),
    }
}

#[derive(Clone, Copy)]
struct Retry {
    awaiting: RwSignal<Option<Reconnect>>,
    is_mounted: StoredValue<bool>,
    progress: RwSignal<Option<ConnectStep>>,
    is_attempting: StoredValue<bool>,
    last_failure: RwSignal<Option<ConnectError>>,
    error: RwSignal<Option<ConnectError>>,
    on_connect: Callback<SendWrapper<Rc<dyn Backend>>>,
}

impl Retry {
    fn is_waiting(self) -> bool {
        self.is_mounted.get_value()
            && self
                .awaiting
                .try_with_untracked(|reconnect| {
                    reconnect.as_ref().is_some_and(|reconnect| {
                        reconnect
                            .deadline
                            .is_none_or(|deadline| js_sys::Date::now() < deadline)
                    })
                })
                .unwrap_or(false)
    }

    fn give_up(self, failure: ConnectError) {
        self.awaiting.set(None);
        self.error.set(Some(failure));
        devices::tab_lock::release();
    }
}

async fn attempt_again(retry: Retry) {
    if !retry.is_waiting() || retry.is_attempting.get_value() {
        return;
    }
    retry.is_attempting.set_value(true);
    let failure = attempt(DeviceChoice::GrantedOnly, retry.progress, retry.on_connect).await;
    if retry.is_mounted.get_value() {
        retry.is_attempting.set_value(false);
        match failure {
            Some(failure) if !failure.is_retryable() => retry.give_up(failure),
            Some(ConnectError::NoDevice) | None => {}
            Some(failure) => retry.last_failure.set(Some(failure)),
        }
    }
}

async fn keep_reconnecting(retry: Retry) {
    while retry.is_waiting() {
        sleep(RETRY_GAP).await;
        attempt_again(retry).await;
    }
    if retry.is_mounted.get_value() && retry.awaiting.with_untracked(Option::is_some) {
        retry.give_up(ConnectError::NoDevice);
    }
}

#[component]
fn ReconnectPanel(retry: Retry) -> impl IntoView {
    let stop = move |_| {
        retry.awaiting.set(None);
        devices::tab_lock::release();
    };
    move || {
        retry.awaiting.get().map(|reconnect| {
            view! {
                <div class="bg-background grid w-full gap-3 rounded-lg border p-4 text-left text-sm">
                    <p class="flex items-start gap-2">
                        <Icon kind=IconKind::LoaderCircle class="mt-0.5 shrink-0 animate-spin" />
                        <span>{reconnect.cause.waiting_text()}</span>
                    </p>
                    {move || retry.last_failure.get().map(|failure| view! { <ConnectFailure failure /> })}
                    <button
                        type="button"
                        class="text-muted-foreground hover:text-foreground justify-self-end text-xs underline"
                        on:click=stop
                    >
                        "Stop waiting"
                    </button>
                </div>
            }
        })
    }
}

#[component]
fn Terms() -> impl IntoView {
    view! {
        <section class="text-muted-foreground mt-4 grid gap-2 border-t pt-6 text-left text-xs leading-relaxed">
            <p>
                "This editor was built from the public MIDI specification and from observing how the Darkglass Suite talks to the device over HID. It is not made by Darkglass Electronics."
            </p>
            <p>
                "Everything runs in your browser. The only thing it talks to is the Anagram plugged into this computer; no data is sent anywhere else."
            </p>
            <p>
                "Use it at your own risk. The authors take no responsibility for the device, its presets or its files."
            </p>
        </section>
    }
}

const UDEV_RULE_PATH: &str = "/etc/udev/rules.d/70-anagram.rules";
const UDEV_RULE: &str =
    r#"SUBSYSTEM=="hidraw", ATTRS{idVendor}=="2fa6", ATTRS{idProduct}=="2500", TAG+="uaccess""#;
const UDEV_RELOAD_COMMAND: &str = "sudo udevadm control --reload";
const CODE_BLOCK_CLASS: &str = "bg-muted overflow-x-auto rounded-md px-3 py-2 font-mono text-xs";

#[component]
fn ConnectFailure(failure: ConnectError) -> impl IntoView {
    let needs_linux_access = matches!(failure, ConnectError::OpenFailed(_)) && devices::is_linux();
    view! {
        <div class="grid w-full gap-2 text-left text-sm">
            <p class="text-destructive flex items-start gap-2">
                <Icon kind=IconKind::TriangleAlert class="mt-0.5 shrink-0" />
                <span>{failure.to_string()}</span>
            </p>
            {needs_linux_access
                .then(|| {
                    view! {
                        <p class="text-muted-foreground">
                            "On Linux, save this rule as " <code class="font-mono">{UDEV_RULE_PATH}</code> ":"
                        </p>
                        <pre class=CODE_BLOCK_CLASS>{UDEV_RULE}</pre>
                        <p class="text-muted-foreground">"Then run this, replug the Anagram and connect again:"</p>
                        <pre class=CODE_BLOCK_CLASS>{UDEV_RELOAD_COMMAND}</pre>
                    }
                })}
        </div>
    }
}
