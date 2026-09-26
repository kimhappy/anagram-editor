use leptos::{ev, html, prelude::*};
use tw_merge::tw_merge;

use super::focus;
use crate::components::icon::{Icon, IconKind};

const PANEL_CLASS: &str = "bg-background text-foreground relative flex max-h-[90vh] w-full max-w-lg flex-col gap-4 overflow-y-auto rounded-lg border p-6";

#[component]
pub fn Dialog(
    #[prop(into)] is_open: Signal<bool>,
    #[prop(into)] on_close: Callback<()>,
    title: &'static str,
    #[prop(into, optional)] class: String,
    children: ChildrenFn,
) -> impl IntoView {
    let panel_class = tw_merge!(PANEL_CLASS, class.as_str());
    let panel = NodeRef::<html::Div>::new();
    let is_pressed_on_backdrop = RwSignal::new(false);
    let opener = StoredValue::new_local(None::<web_sys::HtmlElement>);
    Effect::new(move |was_open: Option<bool>| {
        let is_open = is_open.get();
        if is_open && was_open != Some(true) {
            opener.set_value(focus::focused_element());
        }
        if !is_open
            && was_open == Some(true)
            && let Some(element) = opener.get_value()
        {
            element.focus().unwrap_or_default();
        }
        is_open
    });
    Effect::new(move |_| {
        if let Some(panel) = panel.get() {
            panel.focus().unwrap_or_default();
        }
    });
    let title_id = format!("dialog-title-{}", title.replace(' ', "-"));
    let on_pointerdown = move |event: ev::PointerEvent| {
        let is_backdrop = event
            .target()
            .zip(event.current_target())
            .is_some_and(|(target, overlay)| target == overlay);
        is_pressed_on_backdrop.set(is_backdrop);
    };
    let on_backdrop_click = move |_| {
        if is_pressed_on_backdrop.get_untracked() {
            on_close.run(());
        }
    };
    view! {
        <Show when=move || is_open.get()>
            <div
                class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
                on:pointerdown=on_pointerdown
                on:click=on_backdrop_click
            >
                <CloseOnEscape on_close />
                <div
                    node_ref=panel
                    role="dialog"
                    aria-modal="true"
                    aria-labelledby=title_id.clone()
                    tabindex="-1"
                    class=panel_class.clone()
                    on:click=move |event| event.stop_propagation()
                >
                    <div class="flex items-center justify-between gap-4">
                        <h2 id=title_id.clone() class="text-lg font-medium">{title}</h2>
                        <button
                            type="button"
                            class="hover:bg-accent rounded-md p-1"
                            aria-label="Close"
                            on:click=move |_| on_close.run(())
                        >
                            <Icon kind=IconKind::Close />
                        </button>
                    </div>
                    {children()}
                </div>
            </div>
        </Show>
    }
}

#[component]
fn CloseOnEscape(on_close: Callback<()>) -> impl IntoView {
    let listener = window_event_listener(ev::keydown, move |event| {
        if event.key() == "Escape" && !event.default_prevented() {
            on_close.run(());
        }
    });
    on_cleanup(move || listener.remove());
}
