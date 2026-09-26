use leptos::prelude::*;

use super::Switch;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FieldLook {
    #[default]
    Setting,
    Caption,
}

#[component]
pub fn Field(
    #[prop(into)] label: Signal<String>,
    #[prop(optional)] look: FieldLook,
    children: Children,
) -> impl IntoView {
    match look {
        FieldLook::Setting => view! {
            <label class="grid min-w-0 grid-cols-1 gap-1 text-sm">
                <span>{label}</span>
                {children()}
            </label>
        }
        .into_any(),
        FieldLook::Caption => view! {
            <div class="grid min-w-0 grid-cols-1 gap-1">
                <span class="text-muted-foreground text-xs">{label}</span>
                {children()}
            </div>
        }
        .into_any(),
    }
}

#[component]
pub fn SwitchField(
    #[prop(into)] label: String,
    #[prop(into)] checked: Signal<bool>,
    #[prop(into)] on_toggle: Callback<()>,
    #[prop(into, optional)] disabled: Signal<bool>,
) -> impl IntoView {
    view! {
        <div
            class="flex h-9 items-center justify-between gap-3 text-sm data-[disabled=true]:opacity-50"
            data-disabled=move || disabled.get().to_string()
        >
            <span aria-hidden="true">{label.clone()}</span>
            <Switch checked label disabled on:click=move |_| on_toggle.run(()) />
        </div>
    }
}
