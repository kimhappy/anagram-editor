use std::sync::Arc;

use leptos::prelude::*;
use tw_merge::tw_merge;

use super::side_panel::{
    BackButton, EMPTY_CLASS, KEY_CLASS, SIDEBAR_ROW_HOVER_CLASS, SIDEBAR_ROW_LAYOUT_CLASS,
    SIDEBAR_ROW_SURFACE_CLASS, SearchField, SidePanelHeader, matches_needle, search_needle,
};
use crate::{
    components::{
        category_badge::CategoryBadge,
        icon::{Icon, IconKind},
        marquee::Marquee,
    },
    model::{catalog::BlockModel, category::Category, preset::Placement},
    session::Session,
};

const MODEL_ROW_HOVER_CLASS: &str = "[&:not(:has(>:first-child:disabled))]:hover:bg-sidebar-accent [&:not(:has(>:first-child:disabled))]:hover:text-sidebar-accent-foreground";
const BLOCK_ROW_CLASS: &str =
    "focus-visible:ring-2 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-40";

#[component]
pub fn BlockBrowser() -> impl IntoView {
    let session = Session::expect();
    let has_selection = Memo::new(move |_| session.selected.with(Option::is_some));
    let query = RwSignal::new(String::new());
    let needle = search_needle(query);
    let is_searching = move || needle.with(|needle| !needle.is_empty());
    let placement = Memo::new(move |_| {
        let cell = session.selected.get()?;
        Some(session.draft.with(|preset| {
            session
                .catalog
                .with(|catalog| preset.placement(cell, catalog.models()))
        }))
    });

    view! {
        <aside class="bg-sidebar text-sidebar-foreground flex min-h-0 flex-col border-l">
            <div class="shrink-0 border-b">
                <SearchField query label="Search blocks" />
            </div>
            {move || {
                if is_searching() {
                    return None;
                }
                let header = match session.ui.browsing.get() {
                    Some(category) => view! {
                        <SidePanelHeader title=category.label()>
                            <BackButton label="All categories" on_click=move |()| session.ui.browsing.set(None) />
                        </SidePanelHeader>
                    }
                    .into_any(),
                    None if has_selection.get() && !session.is_read_only() => {
                        view! { <SidePanelHeader title="Category" /> }.into_any()
                    }
                    None => return None,
                };
                Some(header)
            }}
            <div class="min-h-0 flex-1 overflow-x-hidden overflow-y-auto">
                {move || {
                    if session.is_read_only() {
                        view! {
                            <p class=EMPTY_CLASS>
                                "Presets of this area are read-only."
                            </p>
                        }
                            .into_any()
                    } else if !has_selection.get() {
                        ().into_any()
                    } else if is_searching() {
                        view! { <SearchResults needle placement /> }.into_any()
                    } else if let Some(category) = session.ui.browsing.get() {
                        view! { <ModelList category placement /> }.into_any()
                    } else {
                        view! { <CategoryList /> }.into_any()
                    }
                }}
            </div>
        </aside>
    }
}

#[component]
fn CategoryList() -> impl IntoView {
    let session = Session::expect();
    let current = Memo::new(move |_| {
        let cell = session.selected.get()?;
        session
            .draft
            .with(|preset| preset.block(cell).map(|block| block.model.category))
    });
    view! {
        <ul class="grid grid-cols-1">
            {Category::ALL
                .into_iter()
                .map(|category| {
                    let count = move || session.catalog.with(|catalog| catalog.in_category(category).count());
                    view! {
                        <li>
                            <button
                                type="button"
                                data-active=move || (current.get() == Some(category)).to_string()
                                class=tw_merge!(
                                    SIDEBAR_ROW_SURFACE_CLASS,
                                    SIDEBAR_ROW_HOVER_CLASS,
                                    SIDEBAR_ROW_LAYOUT_CLASS,
                                    BLOCK_ROW_CLASS
                                )
                                on:click=move |_| session.ui.browsing.set(Some(category))
                            >
                                <CategoryBadge category />
                                <Marquee text=category.label().to_owned() class="flex-1" />
                                <span class="text-muted-foreground text-xs tabular-nums">{count}</span>
                                <Icon kind=IconKind::ChevronRight class="text-muted-foreground" />
                            </button>
                        </li>
                    }
                })
                .collect_view()}
        </ul>
    }
}

#[component]
fn ModelList(category: Category, placement: Memo<Option<Placement>>) -> impl IntoView {
    let session = Session::expect();
    let models = Memo::new(move |_| {
        session.catalog.with(|catalog| {
            session
                .favourites
                .with(|uris| catalog.favourites_first(uris, catalog.in_category(category)))
        })
    });
    view! {
        <ul class="grid grid-cols-1">
            <For
                each=move || models.get()
                key=|model| model.uri.clone()
                children=move |model| view! { <ModelRow model placement /> }
            />
        </ul>
    }
}

#[component]
fn SearchResults(needle: Memo<String>, placement: Memo<Option<Placement>>) -> impl IntoView {
    let session = Session::expect();
    let models = Memo::new(move |_| {
        needle.with(|needle| {
            session.catalog.with(|catalog| {
                let found = Category::ALL
                    .into_iter()
                    .flat_map(|category| catalog.in_category(category))
                    .filter(|model| {
                        matches_needle(&model.name, needle)
                            || matches_needle(&model.short(), needle)
                    });
                session
                    .favourites
                    .with(|uris| catalog.favourites_first(uris, found))
            })
        })
    });
    view! {
        <ul class="grid grid-cols-1">
            <For
                each=move || models.get()
                key=|model| model.uri.clone()
                children=move |model| view! { <ModelRow model placement /> }
            />
            <Show when=move || models.with(Vec::is_empty)>
                <li class=EMPTY_CLASS>"No blocks found."</li>
            </Show>
        </ul>
    }
}

#[component]
fn ModelRow(model: Arc<BlockModel>, placement: Memo<Option<Placement>>) -> impl IntoView {
    let session = Session::expect();
    let uri = model.uri.clone();
    let is_current = {
        let uri = uri.clone();
        Memo::new(move |_| {
            session.selected.get().is_some_and(|cell| {
                session.draft.with(|preset| {
                    preset.block(cell).is_some_and(|block| {
                        session
                            .catalog
                            .with(|catalog| catalog.is_same_plugin(&block.model.uri, &uri))
                    })
                })
            })
        })
    };
    let is_favourite = {
        let uri = uri.clone();
        Memo::new(move |_| {
            session.favourites.with(|uris| {
                session
                    .catalog
                    .with(|catalog| catalog.is_favourite(uris, &uri))
            })
        })
    };
    let placeable = Arc::clone(&model);
    let placed = Arc::clone(&model);
    let can_place = move || {
        placement.with(|placement| {
            placement
                .as_ref()
                .is_some_and(|placement| placement.allows(&placeable))
        })
    };
    let star_label = format!("Favourite {}", model.name);

    view! {
        <li
            class=tw_merge!(SIDEBAR_ROW_SURFACE_CLASS, MODEL_ROW_HOVER_CLASS, "group flex items-center")
            data-active=move || is_current.get().to_string()
        >
            <button
                type="button"
                aria-current=move || is_current.get().then_some("true")
                class=tw_merge!(SIDEBAR_ROW_LAYOUT_CLASS, BLOCK_ROW_CLASS, "min-w-0 flex-1 rounded-md")
                disabled=move || !is_current.get() && !can_place()
                on:click=move |_| session.place(&placed)
            >
                <span class=KEY_CLASS aria-hidden="true">
                    {model.short()}
                </span>
                <Marquee text=model.name.clone() class="flex-1" />
            </button>
            <button
                type="button"
                class="text-muted-foreground hover:text-foreground data-[on=true]:text-favourite mr-2 rounded-sm p-1 opacity-0 transition-opacity group-hover:opacity-100 focus-visible:opacity-100 data-[on=true]:opacity-100 data-[on=true]:[&_svg]:fill-current"
                data-on=move || is_favourite.get().to_string()
                aria-label=star_label
                aria-pressed=move || is_favourite.get().to_string()
                disabled=move || !session.is_idle()
                on:click=move |_| session.toggle_favourite(&uri)
            >
                <Icon kind=IconKind::Star />
            </button>
        </li>
    }
}
