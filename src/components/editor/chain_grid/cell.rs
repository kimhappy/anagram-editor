use leptos::{
    ev::{KeyboardEvent, MouseEvent},
    prelude::*,
};
use tw_merge::tw_merge;

use super::{
    BYPASSED_CLASS, BYPASSED_SELECTED_CLASS, CELL_CLASS, DASH_PATH_LENGTH, DASH_PATTERN,
    EMPTY_CELL_CLASS, EMPTY_SELECTED_CLASS, SVG_CLASS,
    drag::{Drag, ShownPreset},
    geometry::{cell_style, group_mark_style, icon_style, outline_style},
};
use crate::{
    components::{
        category_badge,
        editor::{bound_marks, change_marks},
        icon::{Icon, IconKind},
        marquee::Marquee,
        ui::MenuState,
    },
    model::{category::Category, preset::Cell},
    session::Session,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tile {
    pub category: Category,
    pub short: String,
    pub enabled: bool,
    pub is_changed: bool,
    pub is_scene_specific: bool,
    pub bound: String,
}

impl Tile {
    pub fn class(&self, is_selected: bool) -> String {
        match (self.enabled, is_selected) {
            (true, false) => category_badge::tile_class(self.category),
            (true, true) => category_badge::selected_tile_class(self.category),
            (false, false) => BYPASSED_CLASS.to_owned(),
            (false, true) => BYPASSED_SELECTED_CLASS.to_owned(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct CellContext {
    pub shown: ShownPreset,
    pub shaking_row: Memo<Option<usize>>,
    pub drag: Drag,
    pub menu: MenuState<Cell>,
}

#[component]
pub fn ChainCell(cell: Cell, context: CellContext) -> impl IntoView {
    let session = Session::expect();
    let CellContext {
        shown,
        shaking_row,
        drag,
        menu,
    } = context;
    let tile = tile_of(&session, shown, cell);
    let is_dragging = move || drag.source.with(Option::is_some);
    let is_selected = move || session.selected.get() == Some(cell);
    let is_marked_in_group = move || session.marked.with(|marked| marked.contains(&cell));
    let is_landing = move || drag.is_landing_on(cell);
    let is_forbidden = Memo::new(move |_| drag.forbids(&session, cell));
    let is_shaking = move || shaking_row.get() == Some(cell.row) && tile.with(Option::is_some);

    let is_marked = move || {
        if is_dragging() {
            is_landing()
        } else {
            is_selected()
        }
    };
    let is_dashed = Signal::derive(move || tile.with(Option::is_none) && !is_marked());

    let class = move || {
        let is_marked = is_marked();
        let look = tile.with(|tile| tile_look(tile.as_ref(), is_marked));
        tw_merge!(
            CELL_CLASS,
            look,
            if is_dragging() && is_landing() {
                "opacity-60"
            } else {
                ""
            },
            if is_forbidden.get() { "opacity-30" } else { "" },
            if is_shaking() { "animate-shake" } else { "" }
        )
    };
    let on_click = move |event: MouseEvent| {
        event.stop_propagation();
        if event.ctrl_key() || event.meta_key() {
            session.toggle_marked(cell);
        } else {
            session.select(cell);
        }
    };
    let on_context_menu = move |event: MouseEvent| {
        event.stop_propagation();
        let is_in_group = session
            .marked
            .with_untracked(|marked| marked.contains(&cell));
        if !is_in_group {
            session.select(cell);
        }
        menu.open(&event, cell);
    };

    view! {
        <div
            role="button"
            tabindex="0"
            aria-label=move || tile.with(|tile| tile.as_ref().map_or_else(|| format!("{cell}: empty"), |tile| format!("{cell}: {}", tile.short)))
            aria-pressed=move || is_selected().to_string()
            draggable=move || tile.with(Option::is_some).to_string()
            data-block=move || tile.with(Option::is_some).to_string()
            data-row=cell.row.to_string()
            data-column=cell.column.to_string()
            class=class
            style=move || cell_style(tile.with(Option::is_some))
            on:click=on_click
            on:keydown=move |event: KeyboardEvent| {
                if matches!(event.key().as_str(), "Enter" | " ") {
                    event.prevent_default();
                    session.select(cell);
                }
            }
            on:contextmenu=on_context_menu
            on:dblclick=move |_| session.toggle(cell)
            on:dragstart=move |event| drag.start(&session, cell, &event)
            on:dragenter=move |event| drag.hover(cell, &event)
            on:dragover=move |event| drag.hover(cell, &event)
            on:dragend=move |_| drag.clear()
        >
            <Outline is_dashed is_grouped=Signal::derive(is_marked_in_group) />
            {move || {
                tile.get()
                    .map_or_else(
                        || view! { <Icon kind=IconKind::Plus attr:style=icon_style() /> }.into_any(),
                        |tile| view! { <TileFace tile /> }.into_any(),
                    )
            }}
        </div>
    }
}

fn tile_of(session_ref: &Session, shown: ShownPreset, cell: Cell) -> Memo<Option<Tile>> {
    let session = *session_ref;
    Memo::new(move |_| {
        let scene = session.viewed_scene();
        shown.with(|preset| {
            let block = preset.block(cell)?;
            Some(Tile {
                category: block.model.category,
                short: block.model.short(),
                enabled: preset.is_enabled(cell, scene)?,
                is_changed: session.edits.with(|edits| edits.is_block_edited(block.id)),
                is_scene_specific: preset.is_block_scene_specific(cell),
                bound: bound_marks(preset.bindings_of(cell)),
            })
        })
    })
}

fn tile_look(tile: Option<&Tile>, is_marked: bool) -> String {
    tile.map_or_else(
        || {
            if is_marked {
                EMPTY_SELECTED_CLASS.to_owned()
            } else {
                EMPTY_CELL_CLASS.to_owned()
            }
        },
        |tile| tile.class(is_marked),
    )
}

#[component]
pub fn Outline(is_dashed: Signal<bool>, is_grouped: Signal<bool>) -> impl IntoView {
    view! {
        <svg class=SVG_CLASS aria-hidden="true">
            <Show when=move || is_grouped.get()>
                <rect class="stroke-ring/70 fill-none" style=group_mark_style() />
            </Show>
            <rect
                class="fill-none transition-[stroke]"
                style=outline_style()
                pathLength=DASH_PATH_LENGTH
                stroke-dasharray=move || is_dashed.get().then_some(DASH_PATTERN)
            />
        </svg>
    }
}

#[component]
pub fn TileFace(tile: Tile) -> impl IntoView {
    view! {
        <span class="absolute top-[calc(2*var(--u))] right-[calc(2.5*var(--u))] text-[length:calc(4*var(--u))] leading-none font-medium empty:hidden">
            {tile.bound.clone()}
        </span>
        <Icon kind=category_badge::icon(tile.category) attr:style=icon_style() />
        <Marquee
            text=tile.short.clone()
            class="w-full px-[calc(2*var(--u))] pb-[calc(1.5*var(--u))] text-center text-[length:calc(5.5*var(--u))] leading-tight font-normal"
            text_class=change_marks(tile.is_changed, tile.is_scene_specific)
        />
    }
}
