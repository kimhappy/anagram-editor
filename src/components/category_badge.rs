use leptos::prelude::*;
use tw_merge::tw_merge;

use super::icon::{Icon, IconKind};
use crate::model::category::Category;

pub const fn icon(category: Category) -> IconKind {
    match category {
        Category::AmpCab => IconKind::BoomBox,
        Category::Drive => IconKind::Flame,
        Category::Dynamics => IconKind::SlidersVertical,
        Category::Modulation => IconKind::WavesHorizontal,
        Category::Ambience => IconKind::SquareStackFlipped,
        Category::Utility => IconKind::Activity,
    }
}

const TILE_CLASS: &str = "bg-[color-mix(in_oklab,var(--tint),white_50%)] [--edge:var(--tint)] hover:bg-[color-mix(in_oklab,var(--tint),white_30%)]";
const SELECTED_TILE_CLASS: &str = "bg-[color-mix(in_oklab,var(--tint),white_20%)] [--edge:color-mix(in_oklab,var(--tint),black_12%)]";

const fn tint(category: Category) -> &'static str {
    match category {
        Category::AmpCab => "[--tint:var(--color-amp-cab)] text-amber-700",
        Category::Drive => "[--tint:var(--color-drive)] text-red-700",
        Category::Dynamics => "[--tint:var(--color-dynamics)] text-teal-700",
        Category::Modulation => "[--tint:var(--color-modulation)] text-violet-700",
        Category::Ambience => "[--tint:var(--color-ambience)] text-sky-700",
        Category::Utility => "[--tint:var(--color-zinc-200)] text-zinc-600",
    }
}

pub fn tile_class(category: Category) -> String {
    format!("{TILE_CLASS} {}", tint(category))
}

pub fn selected_tile_class(category: Category) -> String {
    format!("{SELECTED_TILE_CLASS} {}", tint(category))
}

#[component]
pub fn CategoryBadge(category: Category, #[prop(into, optional)] class: String) -> impl IntoView {
    view! {
        <span class=tw_merge!(
            "inline-flex size-7 shrink-0 items-center justify-center rounded-md",
            "bg-[color-mix(in_oklab,var(--tint),white_30%)]",
            tint(category),
            class.as_str()
        )>
            <Icon kind=icon(category) class="size-1/2" />
        </span>
    }
}
