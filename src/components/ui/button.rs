use leptos::prelude::*;
use tw_merge::tw_merge;

use crate::components::icon::{Icon, IconKind};

const BASE_CLASS: &str = "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-normal transition-all disabled:opacity-50 [&_svg]:pointer-events-none [&_svg:not([class*='size-'])]:size-4 shrink-0 [&_svg]:shrink-0 outline-none focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-3 aria-invalid:ring-destructive/20 aria-invalid:border-destructive";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    #[default]
    Default,
    Outline,
    Ghost,
    Destructive,
}

impl ButtonVariant {
    const fn class(self) -> &'static str {
        match self {
            Self::Default => "bg-primary text-primary-foreground enabled:hover:bg-primary/90",
            Self::Outline => {
                "border bg-background enabled:hover:bg-accent enabled:hover:text-accent-foreground"
            }
            Self::Ghost => "enabled:hover:bg-accent enabled:hover:text-accent-foreground",
            Self::Destructive => "bg-destructive text-white enabled:hover:bg-destructive/90",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonSize {
    #[default]
    Default,
    Sm,
    Lg,
    Icon,
}

impl ButtonSize {
    const fn class(self) -> &'static str {
        match self {
            Self::Default => "h-9 px-4 py-2 has-[>svg]:px-3",
            Self::Sm => "h-8 rounded-md gap-1.5 px-3 has-[>svg]:px-2.5",
            Self::Lg => "h-10 rounded-md px-6 has-[>svg]:px-4",
            Self::Icon => "size-9",
        }
    }
}

#[component]
pub fn Button(
    #[prop(optional)] variant: ButtonVariant,
    #[prop(optional)] size: ButtonSize,
    #[prop(into, optional)] class: String,
    #[prop(into, optional)] disabled: Signal<bool>,
    children: Children,
) -> impl IntoView {
    view! {
        <button
            type="button"
            class=tw_merge!(BASE_CLASS, variant.class(), size.class(), class.as_str())
            disabled=disabled
        >
            {children()}
        </button>
    }
}

#[component]
pub fn IconButton(
    kind: IconKind,
    #[prop(into)] label: Signal<String>,
    #[prop(into, optional)] class: String,
    #[prop(into, optional)] icon_class: String,
    #[prop(into, optional)] disabled: Signal<bool>,
) -> impl IntoView {
    view! {
        <Button variant=ButtonVariant::Ghost size=ButtonSize::Icon class disabled attr:aria-label=label>
            <Icon kind class=icon_class />
        </Button>
    }
}
