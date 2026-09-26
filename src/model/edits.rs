use std::collections::{BTreeMap, BTreeSet};

use super::{
    preset::{Block, BlockId, BlockOverride, Cell, Preset},
    slot::Slot,
};
use crate::protocol::actuator::Actuator;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Edits {
    is_preset_edited: bool,
    blocks: BTreeSet<BlockId>,
    values: BTreeSet<(BlockId, usize)>,
    properties: BTreeSet<(BlockId, usize)>,
    bindings: BTreeSet<Actuator>,
}

impl Edits {
    #[must_use]
    pub fn between(baseline: &Preset, draft: &Preset) -> Self {
        let has_removed_block = baseline
            .blocks()
            .any(|(_, block)| draft.cell_of(block.id).is_none());
        let diffs: Vec<BlockDiff> = draft
            .blocks()
            .map(|(cell, block)| BlockDiff::between(baseline, draft, cell, block))
            .collect();
        Self {
            is_preset_edited: baseline.name() != draft.name()
                || baseline.scene_names() != draft.scene_names()
                || baseline.midi_out() != draft.midi_out()
                || baseline.extra_bindings() != draft.extra_bindings()
                || has_removed_block,
            blocks: diffs
                .iter()
                .filter(|diff| diff.is_edited)
                .map(|diff| diff.id)
                .collect(),
            values: diffs
                .iter()
                .flat_map(|diff| diff.values.iter().map(|&index| (diff.id, index)))
                .collect(),
            properties: diffs
                .iter()
                .flat_map(|diff| diff.properties.iter().map(|&index| (diff.id, index)))
                .collect(),
            bindings: Actuator::ALL
                .into_iter()
                .filter(|&actuator| baseline.binding(actuator) != draft.binding(actuator))
                .collect(),
        }
    }

    pub const fn mark_preset(&mut self) {
        self.is_preset_edited = true;
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        !self.is_preset_edited
            && self.blocks.is_empty()
            && self.values.is_empty()
            && self.bindings.is_empty()
    }

    #[must_use]
    pub fn is_block_edited(&self, id: BlockId) -> bool {
        self.blocks.contains(&id) || self.values.iter().any(|(block, _)| *block == id)
    }

    #[must_use]
    pub fn is_value_edited(&self, id: BlockId, index: usize) -> bool {
        self.values.contains(&(id, index))
    }

    #[must_use]
    pub fn is_property_edited(&self, id: BlockId, index: usize) -> bool {
        self.properties.contains(&(id, index))
    }

    #[must_use]
    pub fn is_binding_edited(&self, actuator: Actuator) -> bool {
        self.bindings.contains(&actuator)
    }
}

struct BlockDiff {
    id: BlockId,
    is_edited: bool,
    values: Vec<usize>,
    properties: Vec<usize>,
}

impl BlockDiff {
    fn between(baseline: &Preset, draft: &Preset, cell: Cell, block: &Block) -> Self {
        let Some((origin, original)) = baseline
            .cell_of(block.id)
            .and_then(|origin| baseline.block(origin).map(|original| (origin, original)))
        else {
            return Self {
                id: block.id,
                is_edited: true,
                values: Vec::new(),
                properties: Vec::new(),
            };
        };
        let before: BTreeMap<Slot, &BlockOverride> = baseline.scene_overrides(block.id).collect();
        let after: BTreeMap<Slot, &BlockOverride> = draft.scene_overrides(block.id).collect();
        let scenes: BTreeSet<Slot> = before.keys().chain(after.keys()).copied().collect();
        let is_override_changed = |index: usize| {
            scenes.iter().any(|scene| {
                before.get(scene).and_then(|found| found.values.get(&index))
                    != after.get(scene).and_then(|found| found.values.get(&index))
            })
        };
        let is_enabled_changed = scenes.iter().any(|scene| {
            before.get(scene).and_then(|found| found.enabled)
                != after.get(scene).and_then(|found| found.enabled)
        });
        Self {
            id: block.id,
            is_edited: origin != cell
                || original.enabled != block.enabled
                || original.model.uri != block.model.uri
                || original.properties != block.properties
                || original.quickpot != block.quickpot
                || is_enabled_changed,
            values: (0..block.values.len())
                .filter(|&index| {
                    block.value(index) != original.value(index) || is_override_changed(index)
                })
                .collect(),
            properties: (0..block.properties.len().max(original.properties.len()))
                .filter(|&index| block.properties.get(index) != original.properties.get(index))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::Edits;
    use crate::{
        model::{
            binding::{Binding, BindingTarget},
            preset::{Block, Cell, EditScope, Preset, Side},
            slot::Slot,
            testing::{self, CHORUS, CHORUS_STEREO, GAIN},
        },
        protocol::actuator::Actuator,
    };

    const FIRST: Cell = Cell { row: 0, column: 0 };
    const SECOND: Cell = Cell { row: 0, column: 1 };

    #[test]
    fn a_changed_file_marks_only_its_own_property() {
        let model = testing::catalog().find(testing::REVERB).expect("reverb");
        let with_files = |first: &str| Block {
            properties: vec![
                first.to_owned(),
                "/data/user-files/cabinets/A1.wav".to_owned(),
            ],
            ..Block::new(Arc::clone(&model))
        };
        let original = with_files("/data/user-files/reverbs/B1.wav");
        let id = original.id;
        let mut baseline = Preset::default();
        baseline.put(FIRST, original);
        let mut draft = Preset::default();
        draft.put(
            FIRST,
            Block {
                id,
                ..with_files("/data/user-files/reverbs/B2.wav")
            },
        );
        let edits = Edits::between(&baseline, &draft);
        assert!(edits.is_property_edited(id, 0));
        assert!(!edits.is_property_edited(id, 1));
        let mut turned = baseline.clone();
        turned.set_value(FIRST, 0, 0.9, EditScope::Preset);
        assert!(!Edits::between(&baseline, &turned).is_property_edited(id, 0));
    }

    fn two_gains() -> Preset {
        let model = testing::catalog().find(GAIN).expect("gain");
        let mut preset = Preset::default();
        preset.put(FIRST, Block::new(Arc::clone(&model)));
        preset.put(SECOND, Block::new(model));
        preset
    }

    fn id_at(preset: &Preset, cell: Cell) -> super::BlockId {
        preset.block(cell).expect("placed").id
    }

    #[test]
    fn an_unchanged_draft_has_no_edits() {
        let baseline = two_gains();
        let draft = baseline.clone();
        assert!(Edits::between(&baseline, &draft).is_empty());
    }

    #[test]
    fn a_value_edit_marks_its_parameter_and_block_until_it_is_back() {
        let baseline = two_gains();
        let mut draft = baseline.clone();
        draft.set_value(FIRST, 0, 6.0, EditScope::Preset);
        let edits = Edits::between(&baseline, &draft);
        assert!(edits.is_value_edited(id_at(&draft, FIRST), 0));
        assert!(edits.is_block_edited(id_at(&draft, FIRST)));
        assert!(!edits.is_block_edited(id_at(&draft, SECOND)));

        draft.set_value(FIRST, 0, 0.0, EditScope::Preset);
        assert!(Edits::between(&baseline, &draft).is_empty());
    }

    #[test]
    fn a_scene_override_counts_as_a_value_edit() {
        let baseline = two_gains();
        let mut draft = baseline.clone();
        let scene = Slot::from_index(1).expect("scene");
        draft.set_value(SECOND, 0, -3.0, EditScope::Scene(scene));
        let edits = Edits::between(&baseline, &draft);
        assert!(edits.is_value_edited(id_at(&draft, SECOND), 0));
        assert!(!edits.is_value_edited(id_at(&draft, FIRST), 0));
    }

    #[test]
    fn moving_toggling_removing_or_renaming_marks_what_changed() {
        let baseline = two_gains();

        let mut moved = baseline.clone();
        assert_eq!(
            moved.insert(FIRST, Cell { row: 1, column: 4 }, Side::Right),
            Ok(())
        );
        let edits = Edits::between(&baseline, &moved);
        assert!(edits.is_block_edited(id_at(&baseline, FIRST)));
        assert!(!edits.is_block_edited(id_at(&baseline, SECOND)));

        let mut toggled = baseline.clone();
        toggled.toggle(SECOND, EditScope::Preset);
        assert!(Edits::between(&baseline, &toggled).is_block_edited(id_at(&baseline, SECOND)));

        let mut removed = baseline.clone();
        removed.remove(FIRST);
        assert!(!Edits::between(&baseline, &removed).is_empty());

        let mut renamed = baseline.clone();
        renamed.set_name("Other");
        assert!(!Edits::between(&baseline, &renamed).is_empty());
    }

    #[test]
    fn changing_the_model_marks_the_block_but_not_its_kept_values() {
        let catalog = testing::catalog();
        let mut baseline = Preset::default();
        baseline.put(FIRST, Block::new(catalog.find(CHORUS).expect("chorus")));
        let mut draft = baseline.clone();
        draft.change_model(FIRST, catalog.find(CHORUS_STEREO).expect("stereo"));
        let edits = Edits::between(&baseline, &draft);
        let id = id_at(&draft, FIRST);
        assert_eq!(id, id_at(&baseline, FIRST));
        assert!(edits.is_block_edited(id));
        assert!(!edits.is_value_edited(id, 0));
    }

    #[test]
    fn a_binding_edit_counts() {
        let baseline = two_gains();
        let mut draft = baseline.clone();
        draft.set_binding(
            Actuator::Knob1,
            Some(Binding::single(
                "Gain",
                BindingTarget {
                    block: id_at(&draft, FIRST),
                    symbol: "gain".to_owned(),
                    min: -20.0,
                    max: 20.0,
                    extra: serde_json::Map::new(),
                },
            )),
        );
        let edits = Edits::between(&baseline, &draft);
        assert!(edits.is_binding_edited(Actuator::Knob1));
        assert!(!edits.is_binding_edited(Actuator::Knob2));
        assert!(!edits.is_block_edited(id_at(&draft, FIRST)));
    }

    #[test]
    fn a_new_preset_is_edited_once_marked() {
        let mut edits = Edits::between(&Preset::default(), &Preset::default());
        assert!(edits.is_empty());
        edits.mark_preset();
        assert!(!edits.is_empty());
    }
}
