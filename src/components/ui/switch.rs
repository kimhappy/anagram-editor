use leptos::prelude::*;

const ROOT_CLASS: &str = "peer inline-flex h-[1.375rem] w-10 shrink-0 items-center rounded-full border border-transparent transition-all outline-none focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-3 disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-primary data-[state=unchecked]:bg-input";

const THUMB_CLASS: &str = "pointer-events-none block size-5 rounded-full bg-background ring-0 transition-transform data-[state=checked]:translate-x-[calc(100%-2px)] data-[state=unchecked]:translate-x-0";

#[component]
pub fn Switch(
    #[prop(into)] checked: Signal<bool>,
    #[prop(into)] label: String,
    #[prop(into, optional)] disabled: Signal<bool>,
) -> impl IntoView {
    let state = move || {
        if checked.get() {
            "checked"
        } else {
            "unchecked"
        }
    };
    view! {
        <button
            type="button"
            role="switch"
            aria-label=label
            aria-checked=move || checked.get().to_string()
            data-state=state
            disabled=disabled
            class=ROOT_CLASS
        >
            <span data-state=state class=THUMB_CLASS></span>
        </button>
    }
}
