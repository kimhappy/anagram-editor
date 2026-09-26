use std::collections::{BTreeMap, BTreeSet};

use super::{
    preset::{Block, BlockOverride, COLUMNS, Cell, Preset, ROWS},
    slot::Slot,
};
use crate::protocol::hid::preset::PresetDocument;

#[derive(Clone, Debug, PartialEq)]
pub struct CopiedBlock {
    pub offset: Cell,
    pub block: Block,
    pub overrides: BTreeMap<Slot, BlockOverride>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum Clipboard {
    #[default]
    Empty,
    Blocks(Vec<CopiedBlock>),
    Presets(Vec<PresetDocument>),
}

impl Clipboard {
    #[must_use]
    pub const fn has_blocks(&self) -> bool {
        matches!(self, Self::Blocks(blocks) if !blocks.is_empty())
    }

    #[must_use]
    pub const fn has_presets(&self) -> bool {
        matches!(self, Self::Presets(documents) if !documents.is_empty())
    }
}

#[must_use]
pub fn copy_blocks(preset: &Preset, cells: &BTreeSet<Cell>) -> Vec<CopiedBlock> {
    let taken: Vec<(Cell, &Block)> = cells
        .iter()
        .filter_map(|cell| preset.block(*cell).map(|block| (*cell, block)))
        .collect();
    let top = taken.iter().map(|(cell, _)| cell.row).min().unwrap_or(0);
    let left = taken.iter().map(|(cell, _)| cell.column).min().unwrap_or(0);
    taken
        .into_iter()
        .map(|(cell, block)| CopiedBlock {
            offset: Cell {
                row: cell.row - top,
                column: cell.column - left,
            },
            block: block.clone(),
            overrides: preset
                .scene_overrides(block.id)
                .map(|(scene, found)| (scene, found.clone()))
                .collect(),
        })
        .collect()
}

pub fn paste_blocks(preset: &mut Preset, anchor: Cell, copied: &[CopiedBlock]) -> Vec<Cell> {
    copied
        .iter()
        .filter_map(|item| {
            let target = Cell {
                row: anchor.row + item.offset.row,
                column: anchor.column + item.offset.column,
            };
            if target.row >= ROWS || target.column >= COLUMNS {
                return None;
            }
            let block = item.block.duplicate();
            let id = block.id;
            preset.place(target, block).then_some(())?;
            for (scene, found) in &item.overrides {
                preset.set_scene_override(*scene, id, found.clone());
            }
            Some(target)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{copy_blocks, paste_blocks};
    use crate::model::{
        catalog::{FX_LOOP_URI, SEND_URI, SPLIT_URI},
        preset::{Block, Cell, EditScope, Preset},
        slot::Slot,
        testing::{self, DRIVE, GAIN},
    };

    fn block(uri: &str) -> Block {
        Block::new(testing::catalog().find(uri).expect("model exists"))
    }

    const fn cell(row: usize, column: usize) -> Cell {
        Cell { row, column }
    }

    #[test]
    fn pasted_blocks_keep_values_and_overrides_under_new_identities() {
        let scene = Slot::from_index(1).expect("scene");
        let mut preset = Preset::default();
        preset.put(cell(0, 2), block(DRIVE));
        preset.put(cell(0, 3), block(GAIN));
        preset.set_value(cell(0, 2), 0, 7.0, EditScope::Preset);
        preset.set_value(cell(0, 2), 0, 9.0, EditScope::Scene(scene));
        let source_id = preset.block(cell(0, 2)).expect("placed").id;

        let copied = copy_blocks(&preset, &BTreeSet::from([cell(0, 2), cell(0, 3)]));
        assert_eq!(copied.len(), 2);
        assert_eq!(copied[0].offset, cell(0, 0));
        assert_eq!(copied[1].offset, cell(0, 1));

        let placed = paste_blocks(&mut preset, cell(1, 5), &copied);
        assert_eq!(placed, [cell(1, 5), cell(1, 6)]);
        let pasted = preset.block(cell(1, 5)).expect("pasted");
        assert_ne!(pasted.id, source_id);
        assert_eq!(pasted.model.uri, DRIVE);
        assert_eq!(pasted.value(0), Some(7.0));
        assert_eq!(preset.value(cell(1, 5), 0, Some(scene)), Some(9.0));
        assert_eq!(preset.value(cell(0, 2), 0, Some(scene)), Some(9.0));
    }

    #[test]
    fn pasting_past_the_edge_or_into_a_forbidden_cell_drops_what_does_not_fit() {
        let mut preset = Preset::default();
        preset.put(cell(0, 0), block(GAIN));
        preset.put(cell(0, 1), block(SPLIT_URI));
        let copied = copy_blocks(&preset, &BTreeSet::from([cell(0, 0), cell(0, 1)]));

        let placed = paste_blocks(&mut preset, cell(1, 11), &copied);
        assert_eq!(placed, [cell(1, 11)]);
        assert_eq!(
            preset
                .block(cell(1, 11))
                .map(|found| found.model.uri.as_str()),
            Some(GAIN)
        );

        let placed_again = paste_blocks(&mut preset, cell(1, 0), &copied);
        assert_eq!(placed_again, [cell(1, 0)]);
        assert!(preset.block(cell(1, 1)).is_none());
    }

    #[test]
    fn pasting_keeps_send_return_and_fx_loop_exclusive() {
        let mut source = Preset::default();
        source.put(cell(0, 0), block(FX_LOOP_URI));
        source.put(cell(0, 1), block(GAIN));
        let copied = copy_blocks(&source, &BTreeSet::from([cell(0, 0), cell(0, 1)]));

        let mut target = Preset::default();
        target.put(cell(0, 5), block(SEND_URI));
        let placed = paste_blocks(&mut target, cell(1, 0), &copied);
        assert_eq!(placed, [cell(1, 1)]);
        assert!(target.block(cell(1, 0)).is_none());
    }
}
