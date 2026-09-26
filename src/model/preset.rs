use std::{
    array, cmp,
    collections::{BTreeMap, BTreeSet},
    fmt, mem,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
};

use serde_json::{Map, Value};

use super::{
    binding::{Binding, BindingTarget},
    catalog::{BlockModel, ParamSpec, RoutingRole},
    midi_out::MidiOut,
    slot::Slot,
};
use crate::protocol::{actuator::Actuator, hid::uuid::PresetUuid};

pub const ROWS: usize = 2;
pub const COLUMNS: usize = 12;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Cell {
    pub row: usize,
    pub column: usize,
}

impl Cell {
    #[must_use]
    pub fn from_document(row: u32, block: u32) -> Option<Self> {
        let row = usize::try_from(row.checked_sub(1)?).ok()?;
        let column = usize::try_from(block.checked_sub(1)?).ok()?;
        (row < ROWS && column < COLUMNS).then_some(Self { row, column })
    }

    #[must_use]
    pub fn to_document(self) -> (u32, u32) {
        (
            u32::try_from(self.row + 1).unwrap_or(1),
            u32::try_from(self.column + 1).unwrap_or(1),
        )
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Row {}, Slot {}", self.row + 1, self.column + 1)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlockId(u32);

static NEXT_BLOCK_ID: AtomicU32 = AtomicU32::new(0);

impl BlockId {
    fn fresh() -> Self {
        Self(NEXT_BLOCK_ID.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Placement {
    allowed: Vec<(Option<RoutingRole>, bool)>,
}

impl Placement {
    #[must_use]
    pub fn allows(&self, model: &BlockModel) -> bool {
        let role = model.routing_role();
        self.allowed
            .iter()
            .find(|(known, _)| *known == role)
            .is_none_or(|(_, is_allowed)| *is_allowed)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FieldExtras {
    pub params: BTreeMap<String, Map<String, Value>>,
    pub properties: BTreeMap<String, Map<String, Value>>,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub id: BlockId,
    pub model: Arc<BlockModel>,
    pub enabled: bool,
    pub values: Vec<f64>,
    pub properties: Vec<String>,
    pub quickpot: Option<String>,
    pub extra: Map<String, Value>,
    pub field_extras: FieldExtras,
}

impl Block {
    #[must_use]
    pub fn new(model: Arc<BlockModel>) -> Self {
        Self {
            id: BlockId::fresh(),
            enabled: true,
            values: default_values(&model),
            properties: default_properties(&model),
            quickpot: model.default_quickpot(),
            extra: Map::new(),
            field_extras: FieldExtras::default(),
            model,
        }
    }

    #[must_use]
    pub fn duplicate(&self) -> Self {
        Self {
            id: BlockId::fresh(),
            ..self.clone()
        }
    }

    fn retarget(&mut self, model: Arc<BlockModel>) -> Arc<BlockModel> {
        let values = overlay(
            default_values(&model),
            self.values
                .iter()
                .zip(&self.model.params)
                .filter_map(|(old_value, spec)| model.clamped(&spec.symbol, *old_value)),
        );
        let properties = overlay(
            default_properties(&model),
            self.properties
                .iter()
                .zip(&self.model.properties)
                .filter_map(|(old_path, spec)| {
                    Some((model.property_index(&spec.uri)?, old_path.clone()))
                }),
        );
        let is_quickpot_gone = self
            .quickpot
            .as_deref()
            .is_some_and(|symbol| !is_host_symbol(symbol) && model.param_index(symbol).is_none());
        if is_quickpot_gone {
            self.quickpot = model.default_quickpot();
        }
        if model.name != self.model.name {
            self.field_extras = FieldExtras::default();
        }
        self.values = values;
        self.properties = properties;
        mem::replace(&mut self.model, model)
    }

    #[must_use]
    pub fn spec(&self, index: usize) -> Option<&ParamSpec> {
        self.model.params.get(index)
    }

    #[must_use]
    pub fn value(&self, index: usize) -> Option<f64> {
        self.values.get(index).copied()
    }
}

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && (Arc::ptr_eq(&self.model, &other.model) || self.model.uri == other.model.uri)
            && self.enabled == other.enabled
            && self.values == other.values
            && self.properties == other.properties
            && self.quickpot == other.quickpot
            && self.extra == other.extra
            && self.field_extras == other.field_extras
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlockOverride {
    pub enabled: Option<bool>,
    pub values: BTreeMap<usize, f64>,
    pub properties: Option<Vec<Value>>,
    pub extra: Map<String, Value>,
    pub value_extras: BTreeMap<String, Map<String, Value>>,
}

impl BlockOverride {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.enabled.is_none()
            && self.values.is_empty()
            && self.properties.as_ref().is_none_or(Vec::is_empty)
            && self.extra.is_empty()
            && self.value_extras.is_empty()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditScope {
    Preset,
    AllScenes,
    Scene(Slot),
}

impl EditScope {
    #[must_use]
    pub const fn viewed_scene(self) -> Option<Slot> {
        match self {
            Self::Scene(scene) => Some(scene),
            Self::Preset | Self::AllScenes => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoveError {
    Missing,
    RowFull,
    Forbidden,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Routing {
    pub split: Option<usize>,
    pub merge: Option<usize>,
}

impl Routing {
    const fn carries(self, cell: Cell) -> bool {
        cell.row == 0 || self.split.is_some()
    }

    fn precedes(self, upstream: Cell, downstream: Cell) -> bool {
        match upstream.row.cmp(&downstream.row) {
            cmp::Ordering::Equal => upstream.column < downstream.column,
            cmp::Ordering::Less => self.split.is_some_and(|split| upstream.column < split),
            cmp::Ordering::Greater => self.merge.is_some_and(|merge| merge < downstream.column),
        }
    }
}

type Grid = [[Option<Block>; COLUMNS]; ROWS];
type SceneOverrides = BTreeMap<BlockId, BlockOverride>;

#[derive(Clone, Debug, PartialEq)]
pub struct Preset {
    name: String,
    uuid: Option<PresetUuid>,
    grid: Grid,
    scenes: BTreeMap<Slot, SceneOverrides>,
    scene_names: BTreeMap<Slot, String>,
    bindings: BTreeMap<Actuator, Binding>,
    extra_bindings: BTreeMap<String, Binding>,
    midi_out: MidiOut,
    extra: Map<String, Value>,
    chain_extras: BTreeMap<String, Map<String, Value>>,
}

impl Preset {
    pub fn empty(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            uuid: None,
            grid: array::from_fn(|_| array::from_fn(|_| None)),
            scenes: BTreeMap::new(),
            scene_names: BTreeMap::new(),
            bindings: BTreeMap::new(),
            extra_bindings: BTreeMap::new(),
            midi_out: MidiOut::default(),
            extra: Map::new(),
            chain_extras: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: &str) {
        name.clone_into(&mut self.name);
    }

    #[must_use]
    pub const fn uuid(&self) -> Option<&PresetUuid> {
        self.uuid.as_ref()
    }

    pub fn set_uuid(&mut self, uuid: Option<PresetUuid>) {
        self.uuid = uuid;
    }

    #[must_use]
    pub const fn scene_names(&self) -> &BTreeMap<Slot, String> {
        &self.scene_names
    }

    #[must_use]
    pub fn scene_name(&self, scene: Slot) -> Option<&str> {
        self.scene_names.get(&scene).map(String::as_str)
    }

    #[must_use]
    pub fn last_defined_scene(&self) -> Option<Slot> {
        self.scenes
            .keys()
            .chain(self.scene_names.keys())
            .max()
            .copied()
    }

    #[must_use]
    pub fn scene_override(&self, id: BlockId, scene: Slot) -> Option<&BlockOverride> {
        self.override_of(id, Some(scene))
    }

    pub fn set_scene_name(&mut self, scene: Slot, name: Option<&str>) {
        match name.map(str::trim).filter(|name| !name.is_empty()) {
            Some(name) => self.scene_names.insert(scene, name.to_owned()),
            None => self.scene_names.remove(&scene),
        };
    }

    #[must_use]
    pub const fn bindings(&self) -> &BTreeMap<Actuator, Binding> {
        &self.bindings
    }

    #[must_use]
    pub fn binding(&self, actuator: Actuator) -> Option<&Binding> {
        self.bindings.get(&actuator)
    }

    pub fn set_binding(&mut self, actuator: Actuator, binding: Option<Binding>) {
        match binding.filter(|binding| !binding.targets.is_empty()) {
            Some(binding) => self.bindings.insert(actuator, binding),
            None => self.bindings.remove(&actuator),
        };
    }

    pub fn bindings_of(&self, cell: Cell) -> impl Iterator<Item = Actuator> + '_ {
        let id = self.block(cell).map(|block| block.id);
        self.bindings
            .iter()
            .filter(move |(_, binding)| id.is_some_and(|id| binding.drives(id)))
            .map(|(actuator, _)| *actuator)
    }

    pub fn bindings_of_param<'preset>(
        &'preset self,
        cell: Cell,
        symbol: &'preset str,
    ) -> impl Iterator<Item = Actuator> + 'preset {
        let id = self.block(cell).map(|block| block.id);
        self.bindings
            .iter()
            .filter(move |(_, binding)| id.is_some_and(|id| binding.drives_param(id, symbol)))
            .map(|(actuator, _)| *actuator)
    }

    #[must_use]
    pub const fn extra_bindings(&self) -> &BTreeMap<String, Binding> {
        &self.extra_bindings
    }

    pub fn set_extra_binding(&mut self, key: &str, binding: Option<Binding>) {
        match binding.filter(|binding| !binding.targets.is_empty()) {
            Some(binding) => self.extra_bindings.insert(key.to_owned(), binding),
            None => self.extra_bindings.remove(key),
        };
    }

    #[must_use]
    pub const fn midi_out(&self) -> &MidiOut {
        &self.midi_out
    }

    pub const fn midi_out_mut(&mut self) -> &mut MidiOut {
        &mut self.midi_out
    }

    #[must_use]
    pub const fn extra(&self) -> &Map<String, Value> {
        &self.extra
    }

    #[must_use]
    pub const fn chain_extras(&self) -> &BTreeMap<String, Map<String, Value>> {
        &self.chain_extras
    }

    pub fn set_chain_extras(&mut self, extras: BTreeMap<String, Map<String, Value>>) {
        self.chain_extras = extras;
    }

    pub fn set_extra(&mut self, extra: Map<String, Value>) {
        self.extra = extra;
    }

    #[must_use]
    pub fn block(&self, cell: Cell) -> Option<&Block> {
        self.grid.get(cell.row)?.get(cell.column)?.as_ref()
    }

    pub fn block_mut(&mut self, cell: Cell) -> Option<&mut Block> {
        self.grid.get_mut(cell.row)?.get_mut(cell.column)?.as_mut()
    }

    pub fn blocks(&self) -> impl Iterator<Item = (Cell, &Block)> {
        self.grid.iter().enumerate().flat_map(|(row, slots)| {
            slots.iter().enumerate().filter_map(move |(column, slot)| {
                slot.as_ref().map(|block| (Cell { row, column }, block))
            })
        })
    }

    #[must_use]
    pub fn cell_of(&self, id: BlockId) -> Option<Cell> {
        self.blocks()
            .find(|(_, block)| block.id == id)
            .map(|(cell, _)| cell)
    }

    #[must_use]
    pub fn is_enabled(&self, cell: Cell, scene: Option<Slot>) -> Option<bool> {
        let block = self.block(cell)?;
        let overridden = self
            .override_of(block.id, scene)
            .and_then(|found| found.enabled);
        Some(overridden.unwrap_or(block.enabled))
    }

    #[must_use]
    pub fn value(&self, cell: Cell, index: usize, scene: Option<Slot>) -> Option<f64> {
        let block = self.block(cell)?;
        self.override_of(block.id, scene)
            .and_then(|found| found.values.get(&index))
            .copied()
            .or_else(|| block.value(index))
    }

    #[must_use]
    pub fn is_block_scene_specific(&self, cell: Cell) -> bool {
        self.block(cell)
            .is_some_and(|block| self.scene_overrides(block.id).next().is_some())
    }

    #[must_use]
    pub fn is_value_scene_specific(&self, cell: Cell, index: usize) -> bool {
        self.block(cell).is_some_and(|block| {
            self.scene_overrides(block.id)
                .any(|(_, found)| found.values.contains_key(&index))
        })
    }

    pub fn scene_overrides(&self, id: BlockId) -> impl Iterator<Item = (Slot, &BlockOverride)> {
        self.scenes
            .iter()
            .filter_map(move |(scene, overrides)| overrides.get(&id).map(|found| (*scene, found)))
    }

    pub fn change_model(&mut self, cell: Cell, model: Arc<BlockModel>) {
        let Some(block) = self.slot_mut(cell).and_then(Option::as_mut) else {
            return;
        };
        let id = block.id;
        let previous = block.retarget(Arc::clone(&model));
        self.retain_bindings(|binding| {
            binding.targets = binding
                .targets
                .drain(..)
                .filter_map(|target| retargeted(target, id, &model))
                .collect();
            !binding.targets.is_empty()
        });
        for found in self
            .scenes
            .values_mut()
            .filter_map(|overrides| overrides.get_mut(&id))
        {
            found.values = found
                .values
                .iter()
                .filter_map(|(old_index, value)| {
                    let spec = previous.params.get(*old_index)?;
                    model
                        .clamped(&spec.symbol, *value)
                        .filter(|(new_index, _)| is_scene_param(&model, *new_index))
                })
                .collect();
        }
        self.prune();
    }

    pub fn set_scene_override(&mut self, scene: Slot, id: BlockId, override_: BlockOverride) {
        if override_.is_empty() {
            return;
        }
        self.scenes.entry(scene).or_default().insert(id, override_);
    }

    pub fn toggle(&mut self, cell: Cell, scope: EditScope) {
        if let Some(enabled) = self.is_enabled(cell, scope.viewed_scene()) {
            self.set_enabled(cell, !enabled, scope);
        }
    }

    pub fn set_enabled(&mut self, cell: Cell, enabled: bool, scope: EditScope) {
        self.edit(
            cell,
            scope,
            |block| block.enabled = enabled,
            |found| found.enabled = Some(enabled),
            |found| found.enabled = None,
        );
    }

    pub fn set_value(&mut self, cell: Cell, index: usize, value: f64, scope: EditScope) {
        let Some(spec) = self.block(cell).and_then(|block| block.spec(index)) else {
            return;
        };
        let value = spec.clamp(value);
        let scope = match scope {
            EditScope::Scene(_) if !spec.allowed_in_scenes() => EditScope::AllScenes,
            EditScope::Preset | EditScope::AllScenes | EditScope::Scene(_) => scope,
        };
        self.edit(
            cell,
            scope,
            |block| {
                if let Some(slot) = block.values.get_mut(index) {
                    *slot = value;
                }
            },
            |found| {
                found.values.insert(index, value);
            },
            |found| {
                found.values.remove(&index);
            },
        );
    }

    pub fn set_property(&mut self, cell: Cell, index: usize, path: &str) {
        if let Some(slot) = self
            .block_mut(cell)
            .and_then(|block| block.properties.get_mut(index))
        {
            path.clone_into(slot);
        }
    }

    pub fn set_quickpot(&mut self, cell: Cell, symbol: Option<&str>) {
        if let Some(block) = self.block_mut(cell) {
            block.quickpot = symbol.map(str::to_owned);
        }
    }

    #[must_use]
    pub fn routing(&self) -> Routing {
        let column_of = |is_match: fn(&BlockModel) -> bool| {
            self.blocks()
                .find(|(cell, block)| cell.row == 0 && is_match(&block.model))
                .map(|(cell, _)| cell.column)
        };
        Routing {
            split: column_of(BlockModel::is_split),
            merge: column_of(BlockModel::is_merge),
        }
    }

    #[must_use]
    pub fn can_place(&self, cell: Cell, block: &Block) -> bool {
        let replaced_role = self.block(cell).and_then(|old| old.model.routing_role());
        let is_routing_change = replaced_role.is_some() || block.model.routing_role().is_some();
        if !is_routing_change || !self.has_valid_routing() {
            return true;
        }
        let mut next = self.clone();
        next.place_unchecked(cell, block.clone());
        next.has_valid_routing()
    }

    #[must_use]
    pub fn placement<'model>(
        &self,
        cell: Cell,
        models: impl IntoIterator<Item = &'model Arc<BlockModel>>,
    ) -> Placement {
        let mut allowed: Vec<(Option<RoutingRole>, bool)> = Vec::new();
        for model in models {
            let role = model.routing_role();
            if allowed.iter().all(|(known, _)| *known != role) {
                allowed.push((role, self.can_place(cell, &Block::new(Arc::clone(model)))));
            }
        }
        Placement { allowed }
    }

    pub fn place(&mut self, cell: Cell, block: Block) -> bool {
        let is_allowed = self.can_place(cell, &block);
        if is_allowed {
            self.place_unchecked(cell, block);
        }
        is_allowed
    }

    pub fn put(&mut self, cell: Cell, block: Block) {
        self.place_unchecked(cell, block);
    }

    #[must_use]
    pub fn can_remove(&self, cells: &BTreeSet<Cell>) -> bool {
        let mut next = self.clone();
        for cell in cells {
            next.remove(*cell);
        }
        !self.has_valid_routing() || next.has_valid_routing()
    }

    pub fn remove(&mut self, cell: Cell) {
        if let Some(removed) = self.slot_mut(cell).and_then(Option::take) {
            self.forget(removed.id);
        }
    }

    pub fn insert(&mut self, from: Cell, to: Cell, side: Side) -> Result<(), MoveError> {
        *self = self.try_insert(from, to, side)?;
        Ok(())
    }

    #[must_use]
    pub fn can_insert(&self, from: Cell, to: Cell) -> bool {
        [Side::Left, Side::Right]
            .into_iter()
            .any(|side| self.try_insert(from, to, side).is_ok())
    }

    fn try_insert(&self, from: Cell, to: Cell, side: Side) -> Result<Self, MoveError> {
        if self.block(from).is_none() {
            return Err(MoveError::Missing);
        }
        let mut next = self.clone();
        if from == to {
            return Ok(next);
        }
        next.insert_unchecked(from, to, side)?;
        if self.has_valid_routing() && !next.has_valid_routing() {
            return Err(MoveError::Forbidden);
        }
        Ok(next)
    }

    fn insert_unchecked(&mut self, from: Cell, to: Cell, side: Side) -> Result<(), MoveError> {
        let moved = self
            .slot_mut(from)
            .and_then(Option::take)
            .ok_or(MoveError::Missing)?;
        let target = self.make_room(to, side).ok_or(MoveError::RowFull)?;
        *target = Some(moved);
        Ok(())
    }

    fn place_unchecked(&mut self, cell: Cell, block: Block) {
        if let Some(slot) = self.slot_mut(cell) {
            let replaced = slot.replace(block).map(|old| old.id);
            if let Some(id) = replaced {
                self.forget(id);
            }
        }
    }

    fn has_valid_routing(&self) -> bool {
        self.routing_validity().unwrap_or(false)
    }

    fn routing_validity(&self) -> Result<bool, RoutingError> {
        let split = self.top_row_column(RoutingRole::Split)?;
        let merge = self.top_row_column(RoutingRole::Merge)?;
        let routing = Routing { split, merge };
        let send = self.single_cell(RoutingRole::Send)?;
        let return_point = self.single_cell(RoutingRole::Return)?;
        let fx_loops = self.cells_with(RoutingRole::FxLoop).len();
        let is_loop_exclusive = fx_loops == 0 || (send.is_none() && return_point.is_none());
        let is_send_first = match (send, return_point) {
            (Some(send), Some(return_point)) => routing.precedes(send, return_point),
            _ => true,
        };
        Ok(is_ordered(split, merge)
            && [send, return_point]
                .into_iter()
                .flatten()
                .all(|cell| routing.carries(cell))
            && is_send_first
            && fx_loops <= 1
            && is_loop_exclusive)
    }

    fn single_cell(&self, role: RoutingRole) -> Result<Option<Cell>, RoutingError> {
        match self.cells_with(role).as_slice() {
            [] => Ok(None),
            [only] => Ok(Some(*only)),
            _ => Err(RoutingError),
        }
    }

    fn cells_with(&self, role: RoutingRole) -> Vec<Cell> {
        self.blocks()
            .filter(|(_, block)| block.model.routing_role() == Some(role))
            .map(|(cell, _)| cell)
            .collect()
    }

    fn top_row_column(&self, role: RoutingRole) -> Result<Option<usize>, RoutingError> {
        match self.single_cell(role)? {
            Some(only) if only.row != 0 => Err(RoutingError),
            only => Ok(only.map(|cell| cell.column)),
        }
    }

    fn make_room(&mut self, cell: Cell, side: Side) -> Option<&mut Option<Block>> {
        let row = self.grid.get_mut(cell.row)?;
        let gap_after = row
            .get(cell.column..)?
            .iter()
            .position(Option::is_none)
            .map(|offset| cell.column + offset);
        let gap_before = row.get(..=cell.column)?.iter().rposition(Option::is_none);
        let pushes_right = match side {
            Side::Right => gap_after.is_some(),
            Side::Left => gap_before.is_none(),
        };
        if pushes_right {
            let room = row.get_mut(cell.column..=gap_after?)?;
            room.rotate_right(1);
            room.first_mut()
        } else {
            let room = row.get_mut(gap_before?..=cell.column)?;
            room.rotate_left(1);
            room.last_mut()
        }
    }

    fn slot_mut(&mut self, cell: Cell) -> Option<&mut Option<Block>> {
        self.grid.get_mut(cell.row)?.get_mut(cell.column)
    }

    fn override_of(&self, id: BlockId, scene: Option<Slot>) -> Option<&BlockOverride> {
        self.scenes.get(&scene?)?.get(&id)
    }

    fn edit(
        &mut self,
        cell: Cell,
        scope: EditScope,
        edit_preset: impl FnOnce(&mut Block),
        edit_scene: impl FnOnce(&mut BlockOverride),
        reset_scene: impl Fn(&mut BlockOverride),
    ) {
        let Some(block) = self.slot_mut(cell).and_then(Option::as_mut) else {
            return;
        };
        let id = block.id;
        match scope {
            EditScope::Preset => edit_preset(block),
            EditScope::AllScenes => {
                edit_preset(block);
                self.scenes
                    .values_mut()
                    .filter_map(|overrides| overrides.get_mut(&id))
                    .for_each(reset_scene);
            }
            EditScope::Scene(scene) => {
                edit_scene(self.scenes.entry(scene).or_default().entry(id).or_default());
            }
        }
        self.prune();
    }

    fn forget(&mut self, id: BlockId) {
        self.scenes.values_mut().for_each(|overrides| {
            overrides.remove(&id);
        });
        self.retain_bindings(|binding| !binding.forget(id));
        self.prune();
    }

    fn retain_bindings(&mut self, mut keep: impl FnMut(&mut Binding) -> bool) {
        self.bindings.retain(|_, binding| keep(binding));
        self.extra_bindings.retain(|_, binding| keep(binding));
    }

    fn prune(&mut self) {
        self.scenes.values_mut().for_each(|overrides| {
            overrides.retain(|_, found| !found.is_empty());
        });
        self.scenes.retain(|_, overrides| !overrides.is_empty());
    }
}

impl Default for Preset {
    fn default() -> Self {
        Self::empty("Untitled")
    }
}

struct RoutingError;

fn default_values(model: &BlockModel) -> Vec<f64> {
    model.params.iter().map(|spec| spec.default).collect()
}

fn default_properties(model: &BlockModel) -> Vec<String> {
    model
        .properties
        .iter()
        .map(|spec| spec.default_path.clone())
        .collect()
}

#[must_use]
pub fn overlay<T>(defaults: Vec<T>, updates: impl IntoIterator<Item = (usize, T)>) -> Vec<T> {
    let mut updates: BTreeMap<usize, T> = updates.into_iter().collect();
    defaults
        .into_iter()
        .enumerate()
        .map(|(index, default)| updates.remove(&index).unwrap_or(default))
        .collect()
}

fn is_host_symbol(symbol: &str) -> bool {
    symbol.starts_with(':')
}

fn is_scene_param(model: &BlockModel, index: usize) -> bool {
    model
        .params
        .get(index)
        .is_some_and(ParamSpec::allowed_in_scenes)
}

fn retargeted(target: BindingTarget, id: BlockId, model: &BlockModel) -> Option<BindingTarget> {
    if target.block != id || is_host_symbol(&target.symbol) {
        return Some(target);
    }
    let spec = model
        .param_index(&target.symbol)
        .and_then(|index| model.params.get(index))?;
    let clamped = (spec.clamp(target.min), spec.clamp(target.max));
    let is_collapsed =
        clamped.0.total_cmp(&clamped.1).is_eq() && target.min.total_cmp(&target.max).is_ne();
    let (min, max) = match (is_collapsed, target.min > target.max) {
        (false, _) => clamped,
        (true, false) => (spec.min, spec.max),
        (true, true) => (spec.max, spec.min),
    };
    Some(BindingTarget { min, max, ..target })
}

const fn is_ordered(first: Option<usize>, second: Option<usize>) -> bool {
    match (first, second) {
        (Some(first), Some(second)) => first < second,
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{Block, COLUMNS, Cell, EditScope, MoveError, Preset, Side, overlay};
    use crate::{
        model::{
            binding::{Binding, BindingTarget},
            catalog::{BlockModel, FX_LOOP_URI, MERGE_URI, RETURN_URI, SEND_URI, SPLIT_URI},
            slot::Slot,
            testing::{self, CHORUS, CHORUS_STEREO, DRIVE, FILTER, GAIN, REVERB},
        },
        protocol::actuator::Actuator,
    };

    const fn cell(row: usize, column: usize) -> Cell {
        Cell { row, column }
    }

    fn block(uri: &str) -> Block {
        Block::new(testing::catalog().find(uri).expect("model exists"))
    }

    fn preset_with(blocks: &[(Cell, &str)]) -> Preset {
        let mut preset = Preset::default();
        for &(at, uri) in blocks {
            assert!(preset.place(at, block(uri)), "{uri} fits at {at:?}");
        }
        preset
    }

    fn uri_at(preset: &Preset, at: Cell) -> Option<&str> {
        preset.block(at).map(|found| found.model.uri.as_str())
    }

    fn scene(index: usize) -> Slot {
        Slot::from_index(index).expect("scene exists")
    }

    #[test]
    fn insert_moves_into_empty_cell() {
        let mut preset = preset_with(&[(cell(0, 1), DRIVE)]);
        assert_eq!(preset.insert(cell(0, 1), cell(0, 4), Side::Right), Ok(()));
        assert_eq!(uri_at(&preset, cell(0, 4)), Some(DRIVE));
        assert_eq!(uri_at(&preset, cell(0, 1)), None);
    }

    #[test]
    fn insert_pushes_blocks_right_up_to_the_first_gap() {
        let mut preset = preset_with(&[
            (cell(0, 0), GAIN),
            (cell(0, 1), DRIVE),
            (cell(0, 2), CHORUS),
        ]);
        assert_eq!(preset.insert(cell(0, 2), cell(0, 0), Side::Right), Ok(()));
        let row: Vec<Option<&str>> = (0..4)
            .map(|column| uri_at(&preset, cell(0, column)))
            .collect();
        assert_eq!(row, [Some(CHORUS), Some(GAIN), Some(DRIVE), None]);
    }

    #[test]
    fn insert_pushes_blocks_left_when_asked_and_a_gap_precedes() {
        let mut preset = preset_with(&[
            (cell(0, 1), GAIN),
            (cell(0, 2), DRIVE),
            (cell(0, 3), CHORUS),
        ]);
        assert_eq!(preset.insert(cell(0, 3), cell(0, 1), Side::Left), Ok(()));
        let row: Vec<Option<&str>> = (0..4)
            .map(|column| uri_at(&preset, cell(0, column)))
            .collect();
        assert_eq!(row, [Some(GAIN), Some(CHORUS), Some(DRIVE), None]);
    }

    #[test]
    fn insert_is_rejected_when_the_row_is_full() {
        let mut blocks: Vec<(Cell, &str)> =
            (0..COLUMNS).map(|column| (cell(1, column), GAIN)).collect();
        blocks.push((cell(0, 0), DRIVE));
        let mut preset = preset_with(&blocks);
        assert_eq!(
            preset.insert(cell(0, 0), cell(1, 5), Side::Right),
            Err(MoveError::RowFull)
        );
        assert_eq!(preset.insert(cell(1, 0), cell(1, 0), Side::Right), Ok(()));
        assert_eq!(
            preset.insert(cell(0, 5), cell(1, 0), Side::Right),
            Err(MoveError::Missing)
        );
    }

    #[test]
    fn insert_reorders_a_full_row_through_the_gap_it_leaves() {
        let mut blocks: Vec<(Cell, &str)> =
            (0..COLUMNS).map(|column| (cell(0, column), GAIN)).collect();
        blocks[0].1 = DRIVE;
        let mut preset = preset_with(&blocks);
        let moved = preset.block(cell(0, 0)).expect("placed").id;
        assert!(preset.can_insert(cell(0, 0), cell(0, 5)));
        assert_eq!(preset.insert(cell(0, 0), cell(0, 5), Side::Right), Ok(()));
        assert_eq!(preset.cell_of(moved), Some(cell(0, 5)));
        assert_eq!(uri_at(&preset, cell(0, 0)), Some(GAIN));
        assert_eq!(preset.blocks().count(), COLUMNS);
    }

    #[test]
    fn split_and_merge_stay_single_in_the_top_row() {
        let mut preset = Preset::default();
        assert!(!preset.place(cell(1, 0), block(SPLIT_URI)));
        assert!(preset.place(cell(0, 2), block(SPLIT_URI)));
        assert!(!preset.place(cell(0, 5), block(SPLIT_URI)));
        assert!(!preset.place(cell(0, 1), block(MERGE_URI)));
        assert!(preset.place(cell(0, 4), block(MERGE_URI)));
        assert_eq!(
            preset.insert(cell(0, 4), cell(0, 0), Side::Right),
            Err(MoveError::Forbidden)
        );
        assert_eq!(
            preset.insert(cell(0, 4), cell(1, 0), Side::Right),
            Err(MoveError::Forbidden)
        );
        assert_eq!(preset.routing().split, Some(2));
        assert_eq!(preset.routing().merge, Some(4));
    }

    #[test]
    fn an_imported_preset_with_broken_routing_stays_editable() {
        let mut preset = Preset::default();
        preset.put(cell(0, 1), block(SPLIT_URI));
        preset.put(cell(0, 3), block(SPLIT_URI));
        assert!(preset.place(cell(0, 5), block(GAIN)));
        assert_eq!(preset.insert(cell(0, 5), cell(1, 0), Side::Left), Ok(()));
        assert!(preset.place(cell(0, 3), block(MERGE_URI)));
        assert_eq!(preset.routing().merge, Some(3));
    }

    #[test]
    fn send_and_return_follow_the_split_rules_and_exclude_the_fx_loop() {
        let mut preset = Preset::default();
        assert!(!preset.place(cell(1, 0), block(SEND_URI)));
        assert!(preset.place(cell(0, 3), block(SEND_URI)));
        assert!(!preset.place(cell(0, 6), block(SEND_URI)));
        assert!(!preset.place(cell(0, 1), block(RETURN_URI)));
        assert!(preset.place(cell(0, 5), block(RETURN_URI)));
        assert!(!preset.place(cell(1, 2), block(RETURN_URI)));
        assert!(!preset.place(cell(1, 0), block(FX_LOOP_URI)));
        assert_eq!(
            preset.insert(cell(0, 5), cell(0, 0), Side::Right),
            Err(MoveError::Forbidden)
        );

        preset.remove(cell(0, 3));
        assert!(!preset.place(cell(1, 0), block(FX_LOOP_URI)));
        preset.remove(cell(0, 5));
        assert!(preset.place(cell(1, 0), block(FX_LOOP_URI)));
        assert!(!preset.place(cell(1, 4), block(FX_LOOP_URI)));
        assert!(!preset.place(cell(0, 3), block(SEND_URI)));
        assert!(!preset.place(cell(0, 5), block(RETURN_URI)));
        assert!(preset.place(cell(0, 2), block(SPLIT_URI)));
        assert!(preset.place(cell(0, 4), block(MERGE_URI)));
    }

    #[test]
    fn send_and_return_reach_the_lower_row_through_the_split_in_signal_order() {
        let mut preset = Preset::default();
        assert!(!preset.place(cell(1, 4), block(SEND_URI)));
        assert!(preset.place(cell(0, 3), block(SPLIT_URI)));
        assert!(preset.place(cell(0, 8), block(MERGE_URI)));
        assert!(preset.place(cell(1, 4), block(SEND_URI)));
        assert!(!preset.place(cell(1, 2), block(RETURN_URI)));
        assert!(!preset.place(cell(0, 1), block(RETURN_URI)));
        assert!(!preset.place(cell(0, 5), block(RETURN_URI)));
        assert!(preset.place(cell(0, 10), block(RETURN_URI)));
        assert!(!preset.place(cell(1, 6), block(FX_LOOP_URI)));
        assert!(!preset.can_remove(&[cell(0, 8)].into()));
        assert!(!preset.can_remove(&[cell(0, 3)].into()));
        assert!(preset.can_remove(&[cell(1, 4), cell(0, 3)].into()));

        preset.remove(cell(0, 10));
        assert!(preset.place(cell(1, 9), block(RETURN_URI)));
        assert_eq!(
            preset.insert(cell(1, 9), cell(1, 1), Side::Right),
            Err(MoveError::Forbidden)
        );

        let mut before_split = preset_with(&[(cell(0, 1), SEND_URI)]);
        assert!(before_split.place(cell(0, 4), block(SPLIT_URI)));
        assert!(before_split.place(cell(1, 2), block(RETURN_URI)));
        assert_eq!(
            before_split.insert(cell(0, 4), cell(0, 0), Side::Left),
            Err(MoveError::Forbidden)
        );
    }

    #[test]
    fn remove_clears_the_slot_its_overrides_and_its_bindings() {
        let at = cell(0, 0);
        let mut preset = preset_with(&[(at, DRIVE)]);
        let id = preset.block(at).expect("placed").id;
        preset.set_value(at, 0, 8.0, EditScope::Scene(scene(1)));
        preset.set_binding(
            Actuator::FootA,
            Some(Binding::single(
                "Drive",
                BindingTarget {
                    block: id,
                    symbol: "drive".to_owned(),
                    min: 0.0,
                    max: 10.0,
                    extra: serde_json::Map::new(),
                },
            )),
        );
        assert!(preset.is_block_scene_specific(at));
        assert_eq!(
            preset.bindings_of(at).collect::<Vec<_>>(),
            [Actuator::FootA]
        );

        preset.remove(at);
        assert!(preset.block(at).is_none());
        assert!(preset.binding(Actuator::FootA).is_none());
        assert_eq!(preset.scene_overrides(id).count(), 0);
    }

    #[test]
    fn scene_edits_stay_in_their_scene_and_values_are_clamped() {
        let at = cell(0, 0);
        let mut preset = preset_with(&[(at, DRIVE)]);
        preset.set_value(at, 0, 99.0, EditScope::Scene(scene(2)));
        assert_eq!(preset.value(at, 0, Some(scene(2))), Some(10.0));
        assert_eq!(preset.value(at, 0, Some(scene(1))), Some(5.0));
        assert_eq!(preset.value(at, 0, None), Some(5.0));
        assert!(preset.is_value_scene_specific(at, 0));
        assert!(!preset.is_value_scene_specific(at, 1));

        preset.toggle(at, EditScope::Scene(scene(2)));
        assert_eq!(preset.is_enabled(at, Some(scene(2))), Some(false));
        assert_eq!(preset.is_enabled(at, None), Some(true));
    }

    #[test]
    fn a_scene_edit_to_a_param_scenes_cannot_hold_applies_to_every_scene() {
        let at = cell(0, 0);
        let mut preset = preset_with(&[(at, REVERB)]);
        preset.set_value(at, 1, 0.2, EditScope::Scene(scene(1)));
        preset.set_value(at, 0, 8.0, EditScope::Scene(scene(1)));
        assert_eq!(preset.value(at, 0, None), Some(8.0));
        assert_eq!(preset.value(at, 0, Some(scene(2))), Some(8.0));
        assert!(!preset.is_value_scene_specific(at, 0));
        assert!(preset.is_value_scene_specific(at, 1));
    }

    #[test]
    fn extra_bindings_follow_their_block_and_leave_with_it() {
        let mut preset = preset_with(&[(cell(0, 0), DRIVE)]);
        let id = preset.block(cell(0, 0)).expect("placed").id;
        let target = BindingTarget {
            block: id,
            symbol: "drive".to_owned(),
            min: 0.0,
            max: 10.0,
            extra: serde_json::Map::new(),
        };
        preset.set_extra_binding("foot4", Some(Binding::single("Drive", target)));
        assert_eq!(preset.insert(cell(0, 0), cell(1, 2), Side::Right), Ok(()));
        assert!(preset.extra_bindings().contains_key("foot4"));
        preset.remove(cell(1, 2));
        assert!(preset.extra_bindings().is_empty());
    }

    #[test]
    fn changing_the_model_clamps_binding_ranges_to_the_new_spec() {
        let at = cell(0, 0);
        let mut preset = preset_with(&[(at, DRIVE)]);
        let id = preset.block(at).expect("placed").id;
        let drive = testing::catalog().find(DRIVE).expect("drive");
        let mut narrow = BlockModel {
            uri: "urn:test:NarrowDrive".to_owned(),
            ..(*drive).clone()
        };
        narrow.params[0].max = 4.0;
        preset.set_binding(
            Actuator::Knob1,
            Some(Binding::single(
                "Drive",
                BindingTarget {
                    block: id,
                    symbol: "drive".to_owned(),
                    min: 2.0,
                    max: 9.0,
                    extra: serde_json::Map::new(),
                },
            )),
        );
        preset.change_model(at, Arc::new(narrow));
        let target = preset
            .binding(Actuator::Knob1)
            .and_then(Binding::target)
            .expect("still bound");
        assert_eq!((target.min, target.max), (2.0, 4.0));
    }

    #[test]
    fn a_range_that_clamps_to_a_point_widens_to_the_new_spec() {
        let at = cell(0, 0);
        let mut preset = preset_with(&[(at, CHORUS)]);
        let id = preset.block(at).expect("placed").id;
        let depth = |min, max| {
            Some(Binding::single(
                "Depth",
                BindingTarget {
                    block: id,
                    symbol: "depth".to_owned(),
                    min,
                    max,
                    extra: serde_json::Map::new(),
                },
            ))
        };
        preset.set_binding(Actuator::Knob1, depth(0.0, 1.0));
        preset.set_binding(Actuator::Knob2, depth(1.0, 0.0));
        let chorus = testing::catalog().find(CHORUS).expect("chorus");
        let mut wide = BlockModel {
            uri: "urn:test:WideChorus".to_owned(),
            ..(*chorus).clone()
        };
        wide.params[1].min = 20.0;
        wide.params[1].max = 2000.0;
        preset.change_model(at, Arc::new(wide));
        let range = |actuator| {
            preset
                .binding(actuator)
                .and_then(Binding::target)
                .map(|target| (target.min, target.max))
        };
        assert_eq!(range(Actuator::Knob1), Some((20.0, 2000.0)));
        assert_eq!(range(Actuator::Knob2), Some((2000.0, 20.0)));
    }

    #[test]
    fn overrides_follow_a_moved_block() {
        let mut preset = preset_with(&[(cell(0, 0), DRIVE)]);
        preset.set_value(cell(0, 0), 0, 8.0, EditScope::Scene(scene(1)));
        assert_eq!(preset.insert(cell(0, 0), cell(1, 3), Side::Right), Ok(()));
        assert_eq!(preset.value(cell(1, 3), 0, Some(scene(1))), Some(8.0));
        assert!(preset.is_value_scene_specific(cell(1, 3), 0));
    }

    #[test]
    fn editing_all_scenes_clears_the_overrides() {
        let at = cell(0, 0);
        let mut preset = preset_with(&[(at, DRIVE)]);
        preset.set_value(at, 0, 8.0, EditScope::Scene(scene(1)));
        preset.set_value(at, 0, 2.0, EditScope::AllScenes);
        assert_eq!(preset.value(at, 0, Some(scene(1))), Some(2.0));
        assert!(!preset.is_value_scene_specific(at, 0));
        assert!(!preset.is_block_scene_specific(at));
    }

    #[test]
    fn changing_the_form_keeps_values_overrides_and_identity() {
        let at = cell(0, 0);
        let mut preset = preset_with(&[(at, CHORUS)]);
        let id = preset.block(at).expect("placed").id;
        preset.set_value(at, 0, 3.0, EditScope::Preset);
        preset.set_value(at, 2, 8.0, EditScope::Scene(scene(1)));
        let stereo = testing::catalog().find(CHORUS_STEREO).expect("stereo form");
        preset.change_model(at, stereo);
        let block = preset.block(at).expect("still placed");
        assert_eq!(block.id, id);
        assert_eq!(block.model.uri, CHORUS_STEREO);
        assert_eq!(block.value(0), Some(3.0));
        assert_eq!(preset.value(at, 2, Some(scene(1))), Some(8.0));

        preset.set_quickpot(at, Some("rate"));
        let rate = BindingTarget {
            block: id,
            symbol: "rate".to_owned(),
            min: 0.1,
            max: 10.0,
            extra: serde_json::Map::new(),
        };
        preset.set_binding(Actuator::Knob1, Some(Binding::single("Rate", rate)));
        preset.change_model(at, testing::catalog().find(FILTER).expect("filter"));
        let retargeted = preset.block(at).expect("still placed");
        assert_eq!(retargeted.values.len(), 1);
        assert_eq!(retargeted.quickpot, retargeted.model.default_quickpot());
        assert!(preset.binding(Actuator::Knob1).is_none());
        assert!(!preset.is_block_scene_specific(at));
    }

    #[test]
    fn overlay_replaces_known_indices_and_keeps_defaults_elsewhere() {
        assert_eq!(
            overlay(vec![1, 2, 3], [(2, 30), (0, 10), (0, 11), (7, 70)]),
            [11, 2, 30]
        );
    }
}
