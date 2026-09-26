mod band;
mod branches;
mod cell;
mod drag;
mod geometry;

use leptos::{ev::PointerEvent, html, prelude::*};

use self::{
    band::BandDrag,
    branches::Branches,
    cell::{CellContext, ChainCell},
    drag::{Drag, TrashZone, preview_memos},
    geometry::{BUS, GRID_WIDTH, main_line_style, row_style, scale_style, units},
};
use crate::{
    components::{
        icon::IconKind,
        ui::{ContextMenu, MenuItem, MenuState, focus_without_scroll, is_within},
    },
    model::{
        clipboard::Clipboard,
        preset::{COLUMNS, Cell, Preset},
    },
    session::Session,
};

const DASH_PATH_LENGTH: &str = "112";
const DASH_PATTERN: &str = "4 3";

const CELL_CLASS: &str = "relative flex cursor-pointer select-none flex-col items-center justify-center transition-[color,background-color,box-shadow,opacity] outline-none focus-visible:ring-3 focus-visible:ring-ring/50";
const EMPTY_CELL_CLASS: &str = "[--edge:var(--color-zinc-200)] bg-background text-zinc-300 hover:[--edge:var(--color-zinc-300)] hover:bg-accent hover:text-zinc-400";
const EMPTY_SELECTED_CLASS: &str = "[--edge:var(--color-zinc-300)] bg-accent text-zinc-400";
const BAND_CLASS: &str = "border-ring bg-ring/10 pointer-events-none fixed z-40 border";
const BYPASSED_CLASS: &str = "[--edge:var(--color-zinc-200)] bg-background text-zinc-300 hover:[--edge:var(--color-zinc-300)] hover:bg-accent";
const BYPASSED_SELECTED_CLASS: &str = "[--edge:var(--color-zinc-300)] bg-zinc-50 text-zinc-400";
const SVG_CLASS: &str = "pointer-events-none absolute inset-0 size-full overflow-visible";
const LINE_CLASS: &str = "fill-zinc-200";
const TRASH_CLASS: &str = "absolute inset-x-6 bottom-6 flex h-16 items-center justify-center gap-2 rounded-lg border-[3px] border-dashed text-sm font-normal transition-colors";

pub const CHAIN_GRID_SELECTOR: &str = "[data-chain-grid]";

fn section_class(is_read_only: bool) -> String {
    format!(
        "flex min-h-0 flex-1 overflow-auto outline-none transition-opacity [container-type:size] {}",
        if is_read_only { "opacity-40" } else { "" },
    )
}

#[component]
pub fn ChainGrid() -> impl IntoView {
    let session = Session::expect();
    let drag = Drag::new();
    let grid_ref = NodeRef::<html::Div>::new();
    let (shown, shaking_row) = preview_memos(&session, drag);
    let routing = Memo::new(move |_| shown.with(Preset::routing));

    let menu = MenuState::new();
    let menu_items = Signal::derive(move || menu_items(&session, menu.target.get()));
    session.on_preset_change(move || {
        menu.close();
        drag.clear();
    });

    let section_ref = NodeRef::<html::Section>::new();
    let band_drag = BandDrag::new(&session, section_ref, grid_ref);

    let context = CellContext {
        shown,
        shaking_row,
        drag,
        menu,
    };

    view! {
        <div class="relative flex min-h-0 min-w-0 flex-1 flex-col">
            <section
                node_ref=section_ref
                tabindex="-1"
                data-chain-grid=""
                class=move || section_class(session.is_read_only())
                inert=move || session.is_read_only()
                on:click=move |_| band_drag.click()
                on:pointerdown=move |event| {
                    focus_unless_on_cell(section_ref, &event);
                    band_drag.start(&event);
                }
                on:pointermove=move |event| band_drag.track(&event)
                on:pointerup=move |event| band_drag.finish(&event)
                on:pointercancel=move |event| band_drag.cancel(&event)
                on:lostpointercapture=move |event| band_drag.cancel(&event)
            >
                <div class="relative m-auto w-max min-w-full" style=scale_style()>
                    <svg class=SVG_CLASS aria-hidden="true">
                        <rect class=LINE_CLASS style=main_line_style() />
                    </svg>
                    <div
                        node_ref=grid_ref
                        class="relative mx-auto"
                        style=format!("width: {}", units(GRID_WIDTH))
                        on:dragover=move |event| drag.allow_drop(&event)
                        on:dragleave=move |event| drag.leave(grid_ref, &event)
                        on:drop=move |event| drag.drop_on_grid(&session, &event)
                    >
                        <Branches routing />
                        <ChainRow row=0 context />
                        <div style=format!("height: {}", units(BUS))></div>
                        <ChainRow row=1 context />
                    </div>
                </div>
            </section>
            <Show when=move || drag.source.with(Option::is_some)>
                <TrashZone drag />
            </Show>
            {move || {
                band_drag
                    .shown_band()
                    .map(|band| view! { <div class=BAND_CLASS style=band.style()></div> })
            }}
            <ContextMenu anchor=menu.anchor items=menu_items />
        </div>
    }
}

fn menu_items(session_ref: &Session, cell: Option<Cell>) -> Vec<MenuItem> {
    let session = *session_ref;
    let group = session.group_or(cell);
    let has_blocks = session
        .draft
        .with(|preset| group.iter().any(|member| preset.block(*member).is_some()));
    let can_paste =
        cell.is_some() && !session.is_read_only() && session.clipboard.with(Clipboard::has_blocks);
    let count = group.len().max(1);
    vec![
        MenuItem {
            label: if count > 1 { "Copy blocks" } else { "Copy" },
            icon: IconKind::Copy,
            disabled: !has_blocks,
            action: Callback::new(move |()| session.copy_blocks(cell)),
        },
        MenuItem {
            label: "Paste",
            icon: IconKind::ClipboardPaste,
            disabled: !can_paste,
            action: Callback::new(move |()| {
                if let Some(anchor) = cell {
                    session.paste_blocks(anchor);
                }
            }),
        },
        MenuItem {
            label: if count > 1 { "Delete blocks" } else { "Delete" },
            icon: IconKind::Trash,
            disabled: !has_blocks || session.is_read_only(),
            action: Callback::new(move |()| session.delete_blocks(cell)),
        },
    ]
}

fn focus_unless_on_cell(section_ref: NodeRef<html::Section>, event: &PointerEvent) {
    if !is_within(event, "[data-row]")
        && let Some(section) = section_ref.get_untracked()
    {
        focus_without_scroll(&section);
    }
}

#[component]
fn ChainRow(row: usize, context: CellContext) -> impl IntoView {
    view! {
        <div class="relative grid" style=row_style()>
            {(0..COLUMNS).map(|column| view! { <ChainCell cell=Cell { row, column } context /> }).collect_view()}
        </div>
    }
}
