use leptos::{ev::DragEvent, html, prelude::*};
use tw_merge::tw_merge;
use wasm_bindgen::JsCast;

use super::TRASH_CLASS;
use crate::{
    components::icon::{Icon, IconKind},
    model::preset::{Cell, MoveError, Preset, Side},
    session::Session,
};

#[derive(Clone, Copy)]
pub struct Drag {
    pub source: RwSignal<Option<Cell>>,
    pub target: RwSignal<Option<(Cell, Side)>>,
    pub is_over_trash: RwSignal<bool>,
}

impl Drag {
    pub fn new() -> Self {
        Self {
            source: RwSignal::new(None),
            target: RwSignal::new(None),
            is_over_trash: RwSignal::new(false),
        }
    }

    pub fn clear(self) {
        self.source.set(None);
        self.target.set(None);
        self.is_over_trash.set(false);
    }

    pub fn allow_drop(self, event: &DragEvent) {
        if self.source.get_untracked().is_some() {
            event.prevent_default();
        }
    }

    pub fn leave(self, grid_ref: NodeRef<html::Div>, event: &DragEvent) {
        let is_still_inside = grid_ref.get_untracked().is_some_and(|grid| {
            event
                .related_target()
                .and_then(|target| target.dyn_into::<web_sys::Node>().ok())
                .is_some_and(|node| grid.contains(Some(&node)))
        });
        if !is_still_inside {
            self.target.set(None);
        }
    }

    pub fn start(self, session: &Session, cell: Cell, event: &DragEvent) {
        if let Some(transfer) = event.data_transfer() {
            transfer.set_effect_allowed("move");
            transfer
                .set_data("text/plain", &cell.to_string())
                .unwrap_or_default();
        }
        session.select(cell);
        self.source.set(Some(cell));
    }

    pub fn is_landing_on(self, cell: Cell) -> bool {
        self.source
            .get()
            .is_some_and(|source| self.target.get().map_or(source, |(target, _)| target) == cell)
            && !self.is_over_trash.get()
    }

    pub fn forbids(self, session: &Session, cell: Cell) -> bool {
        self.source.get().is_some_and(|source| {
            session.draft.with(|preset| {
                let is_routing_block = preset
                    .block(source)
                    .is_some_and(|block| block.model.routing_role().is_some());
                is_routing_block && !preset.can_insert(source, cell)
            })
        })
    }

    pub fn hover(self, cell: Cell, event: &DragEvent) {
        if self.source.get_untracked().is_some() {
            let hovered = Some((cell, hovered_side(event)));
            if self.target.get_untracked() != hovered {
                self.target.set(hovered);
            }
        }
    }

    pub fn drop_on_grid(self, session: &Session, event: &DragEvent) {
        event.prevent_default();
        if let (Some(source), Some((target, side))) =
            (self.source.get_untracked(), self.target.get_untracked())
        {
            session.move_block(source, target, side);
        }
        self.clear();
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Preview {
    preset: Preset,
    outcome: Result<(), MoveError>,
}

#[derive(Clone, Copy)]
pub struct ShownPreset {
    preview: Memo<Option<Preview>>,
    draft: RwSignal<Preset>,
}

impl ShownPreset {
    pub fn with<T>(self, read: impl Fn(&Preset) -> T) -> T {
        self.preview
            .with(|preview| preview.as_ref().map(|preview| read(&preview.preset)))
            .unwrap_or_else(|| self.draft.with(&read))
    }
}

pub fn preview_memos(session_ref: &Session, drag: Drag) -> (ShownPreset, Memo<Option<usize>>) {
    let session = *session_ref;
    let preview = Memo::new(move |_| {
        let (source, (target, side)) = drag.source.get().zip(drag.target.get())?;
        let mut preset = session.draft.get();
        let outcome = preset.insert(source, target, side);
        Some(Preview { preset, outcome })
    });
    let shown = ShownPreset {
        preview,
        draft: session.draft,
    };
    let shaking_row = Memo::new(move |_| {
        let is_row_full = preview.with(|preview| {
            preview
                .as_ref()
                .is_some_and(|preview| preview.outcome == Err(MoveError::RowFull))
        });
        drag.target
            .get()
            .filter(|_| is_row_full)
            .map(|(cell, _)| cell.row)
    });
    (shown, shaking_row)
}

#[component]
pub fn TrashZone(drag: Drag) -> impl IntoView {
    let session = Session::expect();
    let on_drag_enter = move |event: DragEvent| {
        event.prevent_default();
        drag.target.set(None);
        drag.is_over_trash.set(true);
    };
    let on_drop = move |event: DragEvent| {
        event.prevent_default();
        if let Some(source) = drag.source.get_untracked() {
            session.delete_block(source);
        }
        drag.clear();
    };
    view! {
        <div
            class=move || {
                tw_merge!(
                    TRASH_CLASS,
                    if drag.is_over_trash.get() {
                        "border-red-500 bg-red-100 text-red-700"
                    } else {
                        "border-red-300 bg-red-50 text-red-600"
                    }
                )
            }
            on:dragenter=on_drag_enter
            on:dragover=on_drag_enter
            on:dragleave=move |_| drag.is_over_trash.set(false)
            on:drop=on_drop
        >
            <Icon kind=IconKind::Trash class="pointer-events-none size-5" />
            <span class="pointer-events-none">"Drop here to delete"</span>
        </div>
    }
}

#[expect(
    clippy::float_arithmetic,
    reason = "pointer and element positions are CSS pixels"
)]
pub fn hovered_side(event: &DragEvent) -> Side {
    let middle = event
        .current_target()
        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
        .map(|element| {
            let rect = element.get_bounding_client_rect();
            rect.left() + rect.width() / 2.0
        });
    match middle {
        Some(middle) if event.client_x() < middle => Side::Left,
        _ => Side::Right,
    }
}
