mod channels;
mod eq;
mod file_control;
mod marks;
mod pager;
mod param_controls;
mod quick_pot;

use std::sync::Arc;

use leptos::prelude::*;

use self::{
    channels::ChannelsControl,
    eq::BlockEq,
    file_control::FileControl,
    marks::{BoundMark, SwitchState, bound_to},
    pager::Pager,
    param_controls::ParamControl,
    quick_pot::QuickPotControl,
};
use crate::{
    components::{category_badge::CategoryBadge, ui::Switch},
    model::{binding::BYPASS_SYMBOL, catalog::BlockModel, eq::parametric_bands, preset::Cell},
    session::Session,
};

const ITEMS_PER_PAGE: usize = 12;

const PARAM_CLASS: &str = "grid min-w-0 grid-cols-1 grid-rows-[1.75rem_2.25rem] gap-2";
const LABEL_CLASS: &str = "flex min-w-0 items-center justify-between gap-1.5";
const LABEL_TEXT_CLASS: &str = "text-sm leading-normal font-normal select-none";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Item {
    Property(usize),
    Param(usize),
}

#[component]
pub fn BlockSettings() -> impl IntoView {
    let session = Session::expect();
    let target = Memo::new(move |_| {
        session.selected.get().map(|cell| {
            let placed = session.draft.with(|preset| {
                preset
                    .block(cell)
                    .map(|block| (block.id, Arc::clone(&block.model)))
            });
            (cell, placed)
        })
    });
    let block_id = Memo::new(move |_| {
        let cell = session.selected.get()?;
        session
            .draft
            .with(|preset| preset.block(cell).map(|block| block.id))
    });
    let page = RwSignal::new(0);
    Effect::watch(move || block_id.get(), move |_, _, _| page.set(0), false);

    view! {
        <section class="h-80 shrink-0 overflow-y-auto border-t">
            {move || {
                target
                    .get()
                    .map_or_else(
                        || ().into_any(),
                        |(cell, placed)| {
                            placed.map_or_else(
                                || ().into_any(),
                                |(_, model)| view! { <BlockEditor cell model page /> }.into_any(),
                            )
                        },
                    )
            }}
        </section>
    }
}

#[component]
fn BlockEditor(cell: Cell, model: Arc<BlockModel>, page: RwSignal<usize>) -> impl IntoView {
    let session = Session::expect();
    let enabled = Signal::derive(move || {
        let scene = session.viewed_scene();
        session
            .draft
            .with(|preset| preset.is_enabled(cell, scene).unwrap_or(false))
    });
    let bypass_bound = bound_to(cell, BYPASS_SYMBOL.to_owned());
    let items: Vec<Item> = (0..model.properties.len())
        .map(Item::Property)
        .chain((0..model.params.len()).map(Item::Param))
        .collect();
    let eq_bands = parametric_bands(&model);
    let page_count = if eq_bands.is_some() {
        1
    } else {
        items.len().div_ceil(ITEMS_PER_PAGE).max(1)
    };
    if page.get_untracked() >= page_count {
        page.set(page_count - 1);
    }
    let has_expression_param = model.expression_param().is_some();
    let is_unknown = model.is_unknown();
    let editor_model = Arc::clone(&model);
    let variants = session
        .catalog
        .with_untracked(|catalog| catalog.variants(&model.uri).cloned());

    view! {
        <div class="grid h-full grid-cols-1 grid-rows-[auto_minmax(0,1fr)] gap-4 py-4">
            <div class="flex items-start justify-between gap-3 px-4">
                <div class="flex min-w-0 items-center gap-3">
                    <CategoryBadge category=model.category class="size-10" />
                    <div class="grid min-w-0">
                        <h2 class="truncate leading-tight font-medium">{model.name.clone()}</h2>
                        <p class="text-muted-foreground truncate text-sm leading-tight">
                            {if is_unknown { "Unknown plugin · kept as read".to_owned() } else { model.category.label().to_owned() }}
                        </p>
                    </div>
                </div>
                <div class="flex shrink-0 items-center gap-5">
                    {variants.map(|pair| view! { <ChannelsControl cell model=Arc::clone(&model) pair /> })}
                    <QuickPotControl cell model=Arc::clone(&model) />
                    {has_expression_param.then(|| view! {
                        <Show when=move || !session.is_read_only()>
                            <button
                                type="button"
                                class="hover:bg-accent rounded-md border px-2 py-1 text-xs"
                                title="Bind this block's expression parameter to the expression pedal"
                                on:click=move |_| session.bind_expression_pedal(cell)
                            >
                                "Bind to expression pedal"
                            </button>
                        </Show>
                    })}
                    {(page_count > 1).then(|| view! { <Pager page page_count /> })}
                    <div class="flex items-center gap-2">
                        <BoundMark bound=bypass_bound />
                        <SwitchState checked=enabled class="w-7 text-right text-sm font-normal select-none" />
                        <Switch
                            checked=enabled
                            label=format!("{} enabled", model.name)
                            on:click=move |_| session.toggle(cell)
                        />
                    </div>
                </div>
            </div>
            {match eq_bands {
                Some(bands) => view! {
                    <div class=move || if enabled.get() { "min-h-0" } else { "min-h-0 opacity-50" }>
                        <BlockEq cell model=Arc::clone(&editor_model) bands />
                    </div>
                }
                .into_any(),
                None => view! { <ParamGrid cell model=editor_model items page enabled /> }.into_any(),
            }}
        </div>
    }
}

#[component]
fn ParamGrid(
    cell: Cell,
    model: Arc<BlockModel>,
    items: Vec<Item>,
    page: RwSignal<usize>,
    enabled: Signal<bool>,
) -> impl IntoView {
    view! {
        <div class="overflow-x-auto px-4">
            <div class=move || {
                format!(
                    "grid h-full min-w-[calc(6*6rem+5*0.75rem)] grid-cols-6 grid-rows-2 items-center gap-x-3 transition-opacity {}",
                    if enabled.get() { "" } else { "opacity-50" },
                )
            }>
            {
                move || {
                    items
                        .iter()
                        .skip(page.get() * ITEMS_PER_PAGE)
                        .take(ITEMS_PER_PAGE)
                        .map(|item| match *item {
                            Item::Param(index) => {
                                let spec = model.params.get(index).cloned();
                                spec.map(|spec| view! { <ParamControl cell index spec /> }.into_any())
                            }
                            Item::Property(index) => {
                                let spec = model.properties.get(index).cloned();
                                spec.map(|spec| view! { <FileControl cell index spec uri=model.uri.clone() /> }.into_any())
                            }
                        })
                        .collect_view()
                }
            }
            </div>
        </div>
    }
}
