use crate::model::preset::COLUMNS;

const CELL: usize = 30;
const HALF_CELL: usize = 15;
const PITCH: usize = CELL + GAP;
const GAP: usize = 8;
pub const BUS: usize = 20;
const ROW_BOTTOM: usize = CELL;
const BUS_MIDDLE: usize = CELL + 10;
const LOWER_ROW_TOP: usize = CELL + BUS;
const LOWER_ROW_MIDDLE: usize = LOWER_ROW_TOP + HALF_CELL;
const LAST_COLUMN: usize = COLUMNS - 1;
pub const GRID_WIDTH: usize = COLUMNS * PITCH - GAP;
const GRID_HEIGHT: usize = 2 * CELL + BUS;
const PADDING_X: usize = 8;
const PADDING_Y: usize = 12;
const TRASH_ROOM: &str = "11rem";
const LINE: &str = "0.75 * var(--u)";
const EDGE: &str = "0.5 * var(--u)";
const GROUP_GAP: &str = "2px";
const GROUP_STROKE: &str = "2px";
const CONTENT_DROP: &str = "2.5 * var(--u)";
const ICON_SIZE: usize = 10;

pub fn units(count: usize) -> String {
    format!("calc({count} * var(--u))")
}

pub fn scale_style() -> String {
    format!(
        "--u: clamp(0.125rem, min(100cqw / {}, (100cqh - {TRASH_ROOM}) / {GRID_HEIGHT}), 0.1875rem); padding: {} {}",
        GRID_WIDTH + 2 * PADDING_X,
        units(PADDING_Y),
        units(PADDING_X),
    )
}

pub fn row_style() -> String {
    format!(
        "grid-template-columns: repeat({COLUMNS}, {}); column-gap: {}",
        units(CELL),
        units(GAP)
    )
}

pub fn cell_style(has_block: bool) -> String {
    format!(
        "width: {0}; height: {0}; border-radius: var(--radius-lg); gap: {1}; padding-top: {2}",
        units(CELL),
        units(3),
        if has_block {
            format!("calc({CONTENT_DROP})")
        } else {
            "0".to_owned()
        }
    )
}

pub fn icon_style() -> String {
    format!("width: {0}; height: {0}", units(ICON_SIZE))
}

pub fn outline_style() -> String {
    format!(
        "x: calc({EDGE} / 2); y: calc({EDGE} / 2); width: calc(100% - {EDGE}); height: calc(100% - {EDGE}); rx: max(0px, calc(var(--radius-lg) - {EDGE} / 2)); stroke: var(--edge); stroke-width: calc({EDGE})"
    )
}

pub fn group_mark_style() -> String {
    format!(
        "x: calc(-1 * ({GROUP_GAP} + {GROUP_STROKE} / 2)); y: calc(-1 * ({GROUP_GAP} + {GROUP_STROKE} / 2)); width: calc(100% + 2 * {GROUP_GAP} + {GROUP_STROKE}); height: calc(100% + 2 * {GROUP_GAP} + {GROUP_STROKE}); rx: var(--radius-lg); stroke-width: {GROUP_STROKE}"
    )
}

pub fn main_line_style() -> String {
    format!(
        "x: 0; width: 100%; y: calc({} * var(--u) - {LINE} / 2); height: calc({LINE})",
        PADDING_Y + HALF_CELL
    )
}

const fn column_middle(column: usize) -> usize {
    column * PITCH + HALF_CELL
}

fn horizontal_line(y: usize, from_x: usize, to_x: usize) -> String {
    format!(
        "y: calc({y} * var(--u) - {LINE} / 2); x: calc({from_x} * var(--u) - {LINE} / 2); width: calc({} * var(--u) + {LINE}); height: calc({LINE})",
        to_x.saturating_sub(from_x)
    )
}

fn vertical_line(x: usize, from_y: usize, to_y: usize) -> String {
    format!(
        "x: calc({x} * var(--u) - {LINE} / 2); y: calc({from_y} * var(--u) - {LINE} / 2); height: calc({} * var(--u) + {LINE}); width: calc({LINE})",
        to_y.saturating_sub(from_y)
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Junction {
    Split,
    Merge,
}

impl Junction {
    const fn edge(self) -> usize {
        match self {
            Self::Split => column_middle(0),
            Self::Merge => column_middle(LAST_COLUMN),
        }
    }

    pub fn lines(self, column: usize) -> [String; 3] {
        let x = column_middle(column);
        let edge = self.edge();
        [
            vertical_line(x, ROW_BOTTOM, BUS_MIDDLE),
            horizontal_line(BUS_MIDDLE, x.min(edge), x.max(edge)),
            vertical_line(edge, BUS_MIDDLE, LOWER_ROW_TOP),
        ]
    }
}

pub fn lower_row_line() -> String {
    horizontal_line(
        LOWER_ROW_MIDDLE,
        column_middle(0),
        column_middle(LAST_COLUMN),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        BUS_MIDDLE, Junction, LOWER_ROW_TOP, ROW_BOTTOM, column_middle, horizontal_line,
        vertical_line,
    };

    #[test]
    fn split_runs_from_its_column_back_to_the_first() {
        let x = column_middle(3);
        let first = column_middle(0);
        assert_eq!(
            Junction::Split.lines(3),
            [
                vertical_line(x, ROW_BOTTOM, BUS_MIDDLE),
                horizontal_line(BUS_MIDDLE, first, x),
                vertical_line(first, BUS_MIDDLE, LOWER_ROW_TOP),
            ]
        );
    }

    #[test]
    fn merge_runs_from_its_column_on_to_the_last() {
        let x = column_middle(2);
        let last = column_middle(super::LAST_COLUMN);
        assert_eq!(
            Junction::Merge.lines(2),
            [
                vertical_line(x, ROW_BOTTOM, BUS_MIDDLE),
                horizontal_line(BUS_MIDDLE, x, last),
                vertical_line(last, BUS_MIDDLE, LOWER_ROW_TOP),
            ]
        );
    }
}
