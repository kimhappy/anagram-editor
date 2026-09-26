use leptos::prelude::*;
use tw_merge::tw_merge;

use crate::components::{
    icon::{Icon, IconKind},
    ui::{INPUT_CLASS, IconButton},
};

pub const SIDEBAR_ROW_SURFACE_CLASS: &str = "data-[active=true]:bg-background border-y border-transparent data-[active=true]:border-sidebar-border rounded-md transition-colors";
pub const SIDEBAR_ROW_HOVER_CLASS: &str =
    "hover:bg-sidebar-accent hover:text-sidebar-accent-foreground";
pub const SIDEBAR_ROW_LAYOUT_CLASS: &str =
    "flex h-9 w-full items-center gap-1.5 pr-2 pl-1 text-left text-sm outline-none ring-inset";
pub const KEY_CLASS: &str = "text-muted-foreground w-7 shrink-0 text-center text-xs tabular-nums";
pub const EMPTY_CLASS: &str = "text-muted-foreground px-2 py-6 text-center text-sm";

#[component]
pub fn SidePanelHeader(
    #[prop(into)] title: Signal<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class="box-content flex h-10 shrink-0 items-center gap-1 border-b px-1">
            <div class="size-8 shrink-0">{children.map(|children| children())}</div>
            <h2 class="truncate text-sm font-normal">{title}</h2>
        </div>
    }
}

#[component]
pub fn Placeholder(text: &'static str) -> impl IntoView {
    view! {
        <div class="text-muted-foreground flex h-full items-center justify-center p-6">{text}</div>
    }
}

pub fn search_needle(query: RwSignal<String>) -> Memo<String> {
    Memo::new(move |_| query.with(|query| query.trim().to_lowercase()))
}

pub fn matches_needle(text: &str, needle: &str) -> bool {
    text.to_lowercase().contains(needle)
}

#[component]
pub fn SearchField(query: RwSignal<String>, label: &'static str) -> impl IntoView {
    view! {
        <label class="relative block shrink-0">
            <Icon
                kind=IconKind::Search
                class="text-muted-foreground pointer-events-none absolute top-1/2 left-3 -translate-y-1/2"
            />
            <input
                type="search"
                placeholder=label
                aria-label=label
                class=tw_merge!(INPUT_CLASS, "bg-background h-10 border-0 pl-9")
                prop:value=move || query.get()
                on:input=move |event| query.set(event_target_value(&event))
            />
        </label>
    }
}

#[component]
pub fn BackButton(label: &'static str, #[prop(into)] on_click: Callback<()>) -> impl IntoView {
    view! {
        <IconButton
            kind=IconKind::ChevronLeft
            class="size-8"
            icon_class="-translate-x-[0.09375rem]"
            label
            on:click=move |_| on_click.run(())
        />
    }
}

#[cfg(test)]
mod tests {
    use super::matches_needle;

    #[test]
    fn matches_needle_lowercases_the_text_but_not_the_needle() {
        assert!(matches_needle("Ephemeris II", "ephem"));
        assert!(matches_needle("Ephemeris II", ""));
        assert!(matches_needle("B7K Ultra", "k u"));
        assert!(!matches_needle("Ephemeris", "Ephem"));
        assert!(!matches_needle("Chorus", "flanger"));
    }
}
