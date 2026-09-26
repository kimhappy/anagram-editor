use std::collections::BTreeSet;

use leptos::{ev::PointerEvent, html, prelude::*};
use wasm_bindgen::JsCast;

use crate::{components::ui::is_within, model::preset::Cell, session::Session};

const BAND_THRESHOLD: f64 = 4.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Band {
    pointer_id: i32,
    origin: (f64, f64),
    corner: (f64, f64),
}

impl Band {
    const fn rect(self) -> (f64, f64, f64, f64) {
        let left = self.origin.0.min(self.corner.0);
        let top = self.origin.1.min(self.corner.1);
        let right = self.origin.0.max(self.corner.0);
        let bottom = self.origin.1.max(self.corner.1);
        (left, top, right, bottom)
    }

    #[expect(clippy::float_arithmetic, reason = "viewport pixel distances")]
    fn has_moved(self) -> bool {
        (self.corner.0 - self.origin.0).abs() > BAND_THRESHOLD
            || (self.corner.1 - self.origin.1).abs() > BAND_THRESHOLD
    }

    #[expect(clippy::float_arithmetic, reason = "viewport pixel sizes")]
    pub fn style(self) -> String {
        let (left, top, right, bottom) = self.rect();
        format!(
            "left: {left}px; top: {top}px; width: {}px; height: {}px",
            right - left,
            bottom - top
        )
    }
}

#[derive(Clone, Copy)]
pub struct BandDrag {
    session: Session,
    band: RwSignal<Option<Band>>,
    did_band: StoredValue<bool>,
    section_ref: NodeRef<html::Section>,
    grid_ref: NodeRef<html::Div>,
}

impl BandDrag {
    pub fn new(
        session: &Session,
        section_ref: NodeRef<html::Section>,
        grid_ref: NodeRef<html::Div>,
    ) -> Self {
        Self {
            session: *session,
            band: RwSignal::new(None),
            did_band: StoredValue::new(false),
            section_ref,
            grid_ref,
        }
    }

    pub fn shown_band(self) -> Option<Band> {
        self.band.get().filter(|band| band.has_moved())
    }

    pub fn start(self, event: &PointerEvent) {
        if event.button() != 0 || is_within(event, BLOCK_SELECTOR) {
            return;
        }
        let at = (event.client_x(), event.client_y());
        self.band.set(Some(Band {
            pointer_id: event.pointer_id(),
            origin: at,
            corner: at,
        }));
    }

    pub fn track(self, event: &PointerEvent) {
        let Some(previous) = self.current(event) else {
            return;
        };
        if event.buttons() & PRIMARY_BUTTON == 0 {
            self.cancel(event);
            return;
        }
        let was_moving = previous.has_moved();
        let current = Band {
            corner: (event.client_x(), event.client_y()),
            ..previous
        };
        self.band.set(Some(current));
        if !current.has_moved() {
            return;
        }
        if !was_moving && let Some(section) = self.section_ref.get_untracked() {
            section
                .set_pointer_capture(event.pointer_id())
                .unwrap_or_default();
        }
        if let Some(grid) = self.grid_ref.get_untracked() {
            self.session.set_marked(blocks_in(&grid, current));
        }
    }

    pub fn finish(self, event: &PointerEvent) {
        if let Some(current) = self.current(event) {
            self.band.set(None);
            self.did_band.set_value(current.has_moved());
        }
    }

    pub fn cancel(self, event: &PointerEvent) {
        if self.current(event).is_some() {
            self.band.set(None);
            self.did_band.set_value(false);
        }
    }

    pub fn click(self) {
        if self.did_band.get_value() {
            self.did_band.set_value(false);
            return;
        }
        self.session.deselect();
    }

    fn current(self, event: &PointerEvent) -> Option<Band> {
        self.band
            .get_untracked()
            .filter(|band| band.pointer_id == event.pointer_id())
    }
}

const BLOCK_SELECTOR: &str = "[data-block='true']";
const PRIMARY_BUTTON: u16 = 1;

fn blocks_in(grid: &web_sys::Element, band: Band) -> BTreeSet<Cell> {
    let (left, top, right, bottom) = band.rect();
    let Ok(tiles) = grid.query_selector_all(BLOCK_SELECTOR) else {
        return BTreeSet::new();
    };
    (0..tiles.length())
        .filter_map(|position| tiles.get(position)?.dyn_into::<web_sys::Element>().ok())
        .filter(|tile| {
            let rect = tile.get_bounding_client_rect();
            rect.left() < right && rect.right() > left && rect.top() < bottom && rect.bottom() > top
        })
        .filter_map(|tile| {
            Some(Cell {
                row: tile.get_attribute("data-row")?.parse().ok()?,
                column: tile.get_attribute("data-column")?.parse().ok()?,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::Band;

    const fn band(origin: (f64, f64), corner: (f64, f64)) -> Band {
        Band {
            pointer_id: 1,
            origin,
            corner,
        }
    }

    #[test]
    fn rect_normalizes_any_drag_direction() {
        assert_eq!(
            band((50.0, 40.0), (10.0, 90.0)).rect(),
            (10.0, 40.0, 50.0, 90.0)
        );
    }

    #[test]
    fn a_band_moves_only_past_the_threshold() {
        assert!(!band((0.0, 0.0), (4.0, -4.0)).has_moved());
        assert!(band((0.0, 0.0), (0.0, 4.5)).has_moved());
    }
}
