use std::rc::Rc;

use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::{
    components::{
        editor::uploads,
        icon::{Icon, IconKind},
        ui::{Button, ButtonVariant, ProgressBar},
    },
    host,
    protocol::{
        firmware_package::{self, FirmwarePackage, PackageError},
        hid::version::Version,
    },
    session::Session,
};

const PACKAGE_EXTENSION: &str = ".tar";

#[derive(Clone, Debug, PartialEq, Eq)]
struct Chosen {
    file_name: String,
    size: usize,
    outcome: Result<FirmwarePackage, PackageError>,
}

#[component]
pub fn FirmwareTab() -> impl IntoView {
    let session = Session::expect();
    let installed = session.backend().firmware_version();
    let chosen = RwSignal::new(None::<Chosen>);
    let package = StoredValue::new_local(None::<Rc<Vec<u8>>>);
    let is_reading = RwSignal::new(false);
    let (picker, open_picker) = uploads::file_picker(PACKAGE_EXTENSION, move |file| {
        is_reading.set(true);
        chosen.set(None);
        package.set_value(None);
        spawn_local(async move {
            let read = wasm_bindgen_futures::JsFuture::from(file.array_buffer()).await;
            is_reading.set(false);
            let Ok(buffer) = read else {
                session.notify("The firmware file could not be read.", true);
                return;
            };
            let bytes = js_sys::Uint8Array::new(&buffer).to_vec();
            let outcome = firmware_package::inspect(&bytes);
            chosen.set(Some(Chosen {
                file_name: file.name(),
                size: bytes.len(),
                outcome,
            }));
            package.set_value(Some(Rc::new(bytes)));
        });
    });
    let installed_text = installed.to_string();
    let install = move |_| {
        if !session.backend().has_serial() {
            spawn_local(async move {
                match session.backend().ensure_serial().await {
                    Ok(()) => session.notify(
                        "The Anagram's file port is allowed. Choose Install again to continue.",
                        false,
                    ),
                    Err(error) => {
                        session.notify(format!("The file port was not allowed: {error}"), true);
                    }
                }
            });
            return;
        }
        let Some(selection) = chosen.get_untracked() else {
            return;
        };
        let Ok(inspected) = selection.outcome else {
            return;
        };
        let version = inspected
            .version
            .as_ref()
            .map_or_else(|| "from this file".to_owned(), Version::to_string);
        let question = format!(
            "Install firmware {version} from {}? The file is copied over USB, then the Anagram restarts into restore mode to install it. Do not unplug it or close this tab until it is back. Back up presets and user files with the Darkglass Suite first.",
            selection.file_name
        );
        if !host::confirm(&question) {
            return;
        }
        let Some(bytes) = package.get_value() else {
            return;
        };
        let owned = Rc::try_unwrap(bytes).unwrap_or_else(|shared| (*shared).clone());
        package.set_value(None);
        chosen.set(None);
        let _started = session.install_firmware(selection.file_name, owned, inspected.version);
    };
    let can_install = Signal::derive(move || {
        session.is_idle()
            && chosen.with(|picked| picked.as_ref().is_some_and(|picked| picked.outcome.is_ok()))
    });

    view! {
        <div class="grid gap-5 text-sm">
            <p>
                "Installed: " <span class="font-medium tabular-nums">{installed_text}</span>
            </p>
            {picker}
            <div class="flex items-center gap-3">
                <Button variant=ButtonVariant::Outline disabled=Signal::derive(move || is_reading.get() || !session.is_idle()) on:click=move |_| open_picker()>
                    <Icon kind=IconKind::Upload />
                    "Choose file…"
                </Button>
                <Show when=move || is_reading.get()>
                    <span class="text-muted-foreground">"Reading the file…"</span>
                </Show>
            </div>
            {move || chosen.get().map(|picked| view! { <PackageSummary picked installed=installed.clone() /> })}
            <Show when=move || session.files.upload_progress.with(Option::is_some)>
                <ProgressBar share=Signal::derive(move || session.files.upload_progress.get()) />
            </Show>
            <div class="flex justify-end">
                <Button variant=ButtonVariant::Destructive disabled=Signal::derive(move || !can_install.get()) on:click=install>
                    "Install…"
                </Button>
            </div>
        </div>
    }
}

#[component]
fn PackageSummary(picked: Chosen, installed: Version) -> impl IntoView {
    let size = format!("{:.1} MB", megabytes(picked.size));
    match picked.outcome {
        Ok(package) => {
            let is_upgrade = package
                .version
                .as_ref()
                .is_none_or(|version| version.is_newer_than(&installed));
            let version = package
                .version
                .as_ref()
                .map_or_else(|| "an unnamed version".to_owned(), Version::to_string);
            view! {
                <div class="grid gap-1 rounded-md border p-3">
                    <p>{picked.file_name} " · " {size}</p>
                    <p class="font-medium">{format!("Anagram firmware {version}")}</p>
                    {(!is_upgrade).then(|| view! {
                        <p class="text-destructive flex items-center gap-2">
                            <Icon kind=IconKind::TriangleAlert />
                            "This is not newer than the installed firmware."
                        </p>
                    })}
                </div>
            }
            .into_any()
        }
        Err(error) => view! {
            <p class="text-destructive flex items-center gap-2">
                <Icon kind=IconKind::TriangleAlert />
                {format!("{}: {error}", picked.file_name)}
            </p>
        }
        .into_any(),
    }
}

#[expect(
    clippy::cast_precision_loss,
    clippy::float_arithmetic,
    reason = "a byte count shown in megabytes"
)]
fn megabytes(bytes: usize) -> f64 {
    bytes as f64 / 1_000_000.0
}
