use std::{
    cell::RefCell,
    collections::{BTreeSet, VecDeque},
    rc::Rc,
    sync::Arc,
};

use leptos::prelude::*;
use send_wrapper::SendWrapper;

use crate::{
    backend::Backend,
    lifecycle::AppLink,
    model::{
        binding::{Binding, BindingTarget, default_binding_name},
        catalog::{BlockModel, Catalog},
        category::Category,
        clipboard::{self, Clipboard},
        edits::Edits,
        library::{Library, PresetRef, SlotEntry},
        midi_out::MidiOutMessage,
        mode::{Mode, SceneControl},
        preset::{Block, BlockId, Cell, EditScope, Preset, Side},
        slot::Slot,
    },
    protocol::{
        actuator::Actuator,
        hid::{
            area::PresetArea,
            files::{UserDir, UserFile},
            plugin::PluginSummary,
            preset::PresetDocument,
            settings::{DeviceSettings, MidiSettings},
        },
        name::PRESET_NAME,
    },
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SidePanel {
    #[default]
    Browser,
    PresetSettings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoticeAction {
    SwapBack {
        area: PresetArea,
        from: Slot,
        to: Slot,
    },
}

impl NoticeAction {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::SwapBack { .. } => "Swap back",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notice {
    pub text: String,
    pub is_error: bool,
    pub action: Option<NoticeAction>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Audition {
    pub pending: Option<PresetDocument>,
    pub in_flight: bool,
    pub slot: Option<Slot>,
    pub is_blocked: bool,
}

impl Audition {
    pub fn clear(&mut self) {
        self.pending = None;
        self.slot = None;
        self.is_blocked = false;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DragOrigin {
    cell: Cell,
    index: usize,
    block: BlockId,
    before: Preset,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StoredPreset {
    pub identity: Option<SlotEntry>,
    pub landing_scene: Slot,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Unreadable {
    pub spot: (PresetArea, u8),
    pub retry_at: f64,
}

const UNDO_DEPTH: usize = 100;

thread_local! {
    static SESSION_OWNER: RefCell<Option<Owner>> = const { RefCell::new(None) };
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Listing<T> {
    #[default]
    Loading,
    Failed(String),
    Loaded(T),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Modal {
    DeviceSettings,
    UserFiles,
    Bindings,
}

#[derive(Clone, Copy)]
pub struct Ui {
    pub panel: RwSignal<SidePanel>,
    pub browsing: RwSignal<Option<Category>>,
    pub is_tuner_open: RwSignal<bool>,
    pub modal: RwSignal<Option<Modal>>,
}

impl Ui {
    fn new() -> Self {
        Self {
            panel: RwSignal::new(SidePanel::default()),
            browsing: RwSignal::new(None),
            is_tuner_open: RwSignal::new(false),
            modal: RwSignal::new(None),
        }
    }

    #[must_use]
    pub fn is_open(self, modal: Modal) -> Signal<bool> {
        Signal::derive(move || self.modal.get() == Some(modal))
    }

    pub fn open(self, modal: Modal) {
        self.modal.set(Some(modal));
    }

    pub fn close(self) {
        self.modal.set(None);
    }
}

#[derive(Clone, Copy)]
pub struct Link {
    pub busy: RwSignal<Option<String>>,
    pub is_replacing_draft: StoredValue<bool>,
    pub audition: RwSignal<Audition>,
    pub expected: RwSignal<Option<(PresetArea, u8)>>,
    pub unreadable: StoredValue<Option<Unreadable>>,
    pub settling_until: RwSignal<f64>,
    pub mode_settling_until: RwSignal<f64>,
    pub is_polling: RwSignal<bool>,
    pub announced_change: StoredValue<Option<String>>,
    pub listing: RwSignal<BTreeSet<PresetArea>>,
}

impl Link {
    fn new(expected: (PresetArea, u8)) -> Self {
        Self {
            busy: RwSignal::new(None),
            is_replacing_draft: StoredValue::new(false),
            audition: RwSignal::new(Audition::default()),
            expected: RwSignal::new(Some(expected)),
            unreadable: StoredValue::new(None),
            settling_until: RwSignal::new(0.0),
            mode_settling_until: RwSignal::new(0.0),
            is_polling: RwSignal::new(false),
            announced_change: StoredValue::new(None),
            listing: RwSignal::new(BTreeSet::new()),
        }
    }
}

#[derive(Clone, Copy)]
pub struct DeviceFiles {
    pub user_files: RwSignal<Vec<UserFile>>,
    pub user_plugins: RwSignal<Listing<Vec<PluginSummary>>>,
    pub listed_dirs: RwSignal<Vec<UserDir>>,
    pub requested_dirs: StoredValue<BTreeSet<UserDir>>,
    pub upload_progress: RwSignal<Option<f64>>,
}

impl DeviceFiles {
    fn new() -> Self {
        Self {
            user_files: RwSignal::new(Vec::new()),
            user_plugins: RwSignal::new(Listing::Loading),
            listed_dirs: RwSignal::new(Vec::new()),
            requested_dirs: StoredValue::new(BTreeSet::new()),
            upload_progress: RwSignal::new(None),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Session {
    pub backend: StoredValue<SendWrapper<Rc<dyn Backend>>>,
    pub catalog: RwSignal<Arc<Catalog>>,
    pub library: RwSignal<Library>,
    pub current: RwSignal<PresetRef>,
    pub browsed_area: RwSignal<PresetArea>,
    pub draft: RwSignal<Preset>,
    pub baseline: RwSignal<Preset>,
    pub stored: RwSignal<StoredPreset>,
    pub edits: Memo<Edits>,
    pub is_dirty: Memo<bool>,
    pub is_new: RwSignal<bool>,
    pub selected: RwSignal<Option<Cell>>,
    pub marked: RwSignal<BTreeSet<Cell>>,
    pub picked: RwSignal<BTreeSet<Slot>>,
    pub clipboard: RwSignal<Clipboard>,
    pub undo_stack: RwSignal<VecDeque<Preset>>,
    pub redo_stack: RwSignal<VecDeque<Preset>>,
    pub drag_origin: RwSignal<Option<DragOrigin>>,
    pub mode: RwSignal<Mode>,
    pub scene_control: RwSignal<SceneControl>,
    pub scene: RwSignal<Slot>,
    pub device_settings: RwSignal<DeviceSettings>,
    pub midi_settings: Memo<MidiSettings>,
    pub favourites: RwSignal<Vec<String>>,
    pub notice: RwSignal<Option<Notice>>,
    pub ui: Ui,
    pub link: Link,
    pub files: DeviceFiles,
    pub app: Option<AppLink>,
}

impl Session {
    pub fn provide(backend: Rc<dyn Backend>) -> Self {
        let snapshot = backend.snapshot();
        let app = use_context::<AppLink>();
        let component_owner = Owner::current();
        let root = Owner::new_root(None);
        let session = root.with(|| {
            let draft = RwSignal::new(Preset::default());
            let baseline = RwSignal::new(Preset::default());
            let is_new = RwSignal::new(false);
            let edits = Memo::new(move |_| {
                let mut edits =
                    draft.with(|draft| baseline.with(|baseline| Edits::between(baseline, draft)));
                if is_new.get() {
                    edits.mark_preset();
                }
                edits
            });
            let is_dirty = Memo::new(move |_| !edits.with(Edits::is_empty));
            let device_settings = RwSignal::new(snapshot.settings.clone());
            Self {
                catalog: RwSignal::new(backend.catalog()),
                library: RwSignal::new(snapshot.library()),
                current: RwSignal::new(PresetRef {
                    area: snapshot.state.area,
                    slot: Slot::from_index(usize::from(snapshot.state.preset_index))
                        .unwrap_or_default(),
                }),
                browsed_area: RwSignal::new(snapshot.state.area),
                draft,
                baseline,
                stored: RwSignal::new(StoredPreset::default()),
                edits,
                is_dirty,
                is_new,
                selected: RwSignal::new(None),
                marked: RwSignal::new(BTreeSet::new()),
                picked: RwSignal::new(BTreeSet::new()),
                clipboard: RwSignal::new(Clipboard::default()),
                undo_stack: RwSignal::new(VecDeque::new()),
                redo_stack: RwSignal::new(VecDeque::new()),
                drag_origin: RwSignal::new(None),
                mode: RwSignal::new(snapshot.state.mode),
                scene_control: RwSignal::new(SceneControl::default()),
                scene: RwSignal::new(Slot::default()),
                device_settings,
                midi_settings: Memo::new(move |_| device_settings.with(|settings| settings.midi)),
                favourites: RwSignal::new(snapshot.favourites.clone()),
                notice: RwSignal::new(None),
                ui: Ui::new(),
                link: Link::new((snapshot.state.area, snapshot.state.preset_index)),
                files: DeviceFiles::new(),
                app,
                backend: StoredValue::new(SendWrapper::new(backend)),
            }
        });
        if let Some(owner) = component_owner {
            owner.set();
        }
        SESSION_OWNER.with(|slot| {
            if let Some(previous) = slot.replace(Some(root)) {
                previous.cleanup();
            }
        });
        provide_context(session);
        session
    }

    #[must_use]
    pub fn expect() -> Self {
        expect_context()
    }

    pub(crate) fn backend(self) -> Rc<dyn Backend> {
        self.backend.with_value(|backend| Rc::clone(backend))
    }

    pub fn notify(self, text: impl Into<String>, is_error: bool) {
        self.notify_with(text, is_error, None);
    }

    pub fn notify_with(
        self,
        text: impl Into<String>,
        is_error: bool,
        action: Option<NoticeAction>,
    ) {
        self.notice.set(Some(Notice {
            text: text.into(),
            is_error,
            action,
        }));
    }

    pub fn run_notice_action(self, action: NoticeAction) {
        self.clear_notice();
        match action {
            NoticeAction::SwapBack { area, from, to } => {
                if self.current.get_untracked().area == area {
                    self.swap_presets(to, from);
                }
            }
        }
    }

    #[must_use]
    pub fn is_idle(self) -> bool {
        self.link.busy.with(Option::is_none) && self.files.upload_progress.with(Option::is_none)
    }

    #[must_use]
    pub fn has_unsaved_edits(self) -> bool {
        let is_untouched_blank = self.is_new.get_untracked()
            && self
                .draft
                .with_untracked(|draft| self.baseline.with_untracked(|baseline| draft == baseline));
        self.is_dirty.get_untracked() && !is_untouched_blank
    }

    pub fn on_preset_change(self, run: impl Fn() + 'static) {
        Effect::watch(move || self.baseline.track(), move |(), _, _| run(), false);
    }

    pub fn clear_notice(self) {
        self.notice.set(None);
    }

    pub(crate) fn adopt(
        self,
        preset_ref: PresetRef,
        preset: Preset,
        stored: StoredPreset,
        is_new: bool,
    ) {
        self.current.set(preset_ref);
        self.browsed_area.set(preset_ref.area);
        self.baseline.set(preset.clone());
        self.draft.set(preset);
        self.scene.set(stored.landing_scene);
        self.stored.set(stored);
        self.is_new.set(is_new);
        self.selected.set(None);
        self.marked.set(BTreeSet::new());
        self.picked.set(BTreeSet::from([preset_ref.slot]));
        self.undo_stack.set(VecDeque::new());
        self.redo_stack.set(VecDeque::new());
        self.ui.browsing.set(None);
        self.link.audition.update(Audition::clear);
    }

    pub fn rename(self, name: &str) {
        let name = PRESET_NAME.sanitize(name);
        let is_new_name = self.draft.with_untracked(|preset| preset.name() != name);
        if !name.is_empty() && is_new_name {
            self.edit(|preset| preset.set_name(&name));
        }
    }

    fn edit(self, change: impl FnOnce(&mut Preset)) -> bool {
        self.edit_with(change)
            .is_some_and(|((), is_changed)| is_changed)
    }

    fn accepts_edits(self) -> bool {
        !self.is_read_only() && !self.link.is_replacing_draft.get_value()
    }

    fn edit_with<T>(self, change: impl FnOnce(&mut Preset) -> T) -> Option<(T, bool)> {
        if !self.accepts_edits() {
            return None;
        }
        let before = self.draft.get_untracked();
        self.drag_origin.set(None);
        let output = self.draft.try_update(change)?;
        let is_changed = self.draft.with_untracked(|draft| *draft != before);
        if is_changed {
            self.remember(before);
        }
        Some((output, is_changed))
    }

    fn remember(self, before: Preset) {
        push_bounded(self.undo_stack, before);
        self.redo_stack.update(VecDeque::clear);
    }

    pub fn undo(self) {
        self.step_history(self.undo_stack, self.redo_stack);
    }

    pub fn redo(self) {
        self.step_history(self.redo_stack, self.undo_stack);
    }

    fn step_history(self, from: RwSignal<VecDeque<Preset>>, to: RwSignal<VecDeque<Preset>>) {
        if self.link.is_replacing_draft.get_value() {
            return;
        }
        let Some(mut restored) = from.try_update(VecDeque::pop_back).flatten() else {
            return;
        };
        let current = self.draft.get_untracked();
        restored.set_uuid(current.uuid().cloned());
        push_bounded(to, current);
        self.drag_origin.set(None);
        self.draft.set(restored);
        self.marked.set(BTreeSet::new());
        self.audition();
    }

    pub fn select(self, cell: Cell) {
        self.selected.set(Some(cell));
        self.marked.set(BTreeSet::new());
        self.ui.browsing.set(
            self.draft
                .with_untracked(|preset| preset.block(cell).map(|block| block.model.category)),
        );
        if self.ui.panel.get_untracked() == SidePanel::PresetSettings {
            self.ui.panel.set(SidePanel::Browser);
        }
    }

    pub fn deselect(self) {
        self.selected.set(None);
        self.marked.set(BTreeSet::new());
        self.ui.browsing.set(None);
    }

    pub fn toggle_marked(self, cell: Cell) {
        if self.block_id(cell).is_none() {
            return;
        }
        self.marked.update(|marked| toggle_member(marked, cell));
    }

    pub fn set_marked(self, cells: BTreeSet<Cell>) {
        if self.marked.with_untracked(|marked| *marked != cells) {
            self.marked.set(cells);
        }
    }

    #[must_use]
    pub fn group_or(self, cell: Option<Cell>) -> BTreeSet<Cell> {
        let marked = self.marked.get_untracked();
        if marked.is_empty() {
            cell.into_iter().collect()
        } else {
            marked
        }
    }

    pub fn copy_blocks(self, cell: Option<Cell>) {
        let cells = self.group_or(cell);
        let copied = self
            .draft
            .with_untracked(|preset| clipboard::copy_blocks(preset, &cells));
        if copied.is_empty() {
            return;
        }
        self.notify(format!("Copied {} block(s)", copied.len()), false);
        self.clipboard.set(Clipboard::Blocks(copied));
    }

    pub fn paste_blocks(self, anchor: Cell) {
        let Clipboard::Blocks(copied) = self.clipboard.get_untracked() else {
            return;
        };
        if !self.accepts_edits() {
            return;
        }
        let placed = self
            .edit_with(|preset| clipboard::paste_blocks(preset, anchor, &copied))
            .map_or_default(|(placed, _)| placed);
        if placed.len() < copied.len() {
            self.notify(
                format!(
                    "{} block(s) did not fit and were not pasted.",
                    copied.len() - placed.len()
                ),
                true,
            );
        }
        if let Some(first) = placed.first().copied() {
            self.select(first);
            self.marked.set(placed.into_iter().collect());
            self.audition();
        }
    }

    pub fn delete_blocks(self, cell: Option<Cell>) {
        let cells = self.group_or(cell);
        if cells.is_empty() || !self.allows_removing(&cells) {
            return;
        }
        let is_removed = self.edit(|preset| {
            for member in &cells {
                preset.remove(*member);
            }
        });
        if is_removed {
            self.deselect();
            self.audition();
        }
    }

    pub fn toggle_picked(self, slot: Slot) {
        self.picked.update(|picked| toggle_member(picked, slot));
    }

    pub fn change_model(self, cell: Cell, model: Arc<BlockModel>) {
        if self.edit(|preset| preset.change_model(cell, model)) {
            self.audition();
        }
    }

    #[must_use]
    pub fn viewed_scene(self) -> Option<Slot> {
        (self.mode.get() == Mode::Scene).then(|| self.scene.get())
    }

    pub fn toggle(self, cell: Cell) {
        let scope = self.edit_scope();
        if self.edit(|preset| preset.toggle(cell, scope)) {
            self.audition();
        }
    }

    pub fn move_block(self, from: Cell, to: Cell, side: Side) {
        let (outcome, is_changed) = self
            .edit_with(|preset| preset.insert(from, to, side))
            .unwrap_or((Ok(()), false));
        if is_changed {
            self.audition();
        }
        self.select(if outcome.is_ok() { to } else { from });
    }

    pub fn place(self, model: &Arc<BlockModel>) {
        let Some(cell) = self.selected.get_untracked() else {
            return;
        };
        let catalog = self.catalog.get_untracked();
        let is_same_model = self.draft.with_untracked(|preset| {
            preset
                .block(cell)
                .is_some_and(|block| catalog.is_same_plugin(&block.model.uri, &model.uri))
        });
        let is_placed = !is_same_model
            && self.edit(|preset| {
                preset.place(cell, Block::new(Arc::clone(model)));
            });
        if is_placed {
            self.audition();
        }
    }

    fn allows_removing(self, cells: &BTreeSet<Cell>) -> bool {
        let is_allowed = self.draft.with_untracked(|preset| preset.can_remove(cells));
        if !is_allowed {
            self.notify(
                "Remove the Send or Return in the second row or after the Merge first.",
                true,
            );
        }
        is_allowed
    }

    pub fn delete_block(self, cell: Cell) {
        if !self.allows_removing(&BTreeSet::from([cell])) {
            return;
        }
        if self.edit(|preset| preset.remove(cell)) {
            self.deselect();
            self.audition();
        }
    }

    pub fn set_value_preview(self, cell: Cell, index: usize, value: f64) {
        self.set_values_preview(cell, &[(index, value)]);
    }

    pub fn set_values_preview(self, cell: Cell, values: &[(usize, f64)]) {
        let Some(&(first, _)) = values.first() else {
            return;
        };
        if self.link.is_replacing_draft.get_value() {
            return;
        }
        let Some(block) = self.block_id(cell) else {
            return;
        };
        let is_same_drag = self.drag_origin.with_untracked(|origin| {
            origin.as_ref().is_some_and(|origin| {
                (origin.cell, origin.index, origin.block) == (cell, first, block)
            })
        });
        if !is_same_drag {
            self.drag_origin.set(Some(DragOrigin {
                cell,
                index: first,
                block,
                before: self.draft.get_untracked(),
            }));
        }
        let scope = self.edit_scope();
        self.draft.update(|preset| {
            for &(index, value) in values {
                preset.set_value(cell, index, value, scope);
            }
        });
    }

    pub fn abandon_drag(self) {
        if let Some(origin) = self.drag_origin.try_update(Option::take).flatten() {
            self.draft.set(origin.before);
        }
    }

    pub fn set_value(self, cell: Cell, index: usize, value: f64) {
        self.set_values(cell, &[(index, value)]);
    }

    pub fn set_values(self, cell: Cell, values: &[(usize, f64)]) {
        let Some(&(first, _)) = values.first() else {
            return;
        };
        let origin = self.drag_origin.try_update(Option::take).flatten();
        if self.link.is_replacing_draft.get_value() {
            return;
        }
        let before = match origin {
            Some(origin) if (origin.cell, origin.index) == (cell, first) => {
                if self.block_id(cell) != Some(origin.block) {
                    return;
                }
                origin.before
            }
            _ => self.draft.get_untracked(),
        };
        let scope = self.edit_scope();
        self.draft.update(|preset| {
            for &(index, value) in values {
                preset.set_value(cell, index, value, scope);
            }
        });
        if self.draft.with_untracked(|draft| *draft != before) {
            self.remember(before);
            self.audition();
        }
    }

    pub fn set_property(self, cell: Cell, index: usize, path: &str) {
        if self.edit(|preset| preset.set_property(cell, index, path)) {
            self.audition();
        }
    }

    pub fn set_quickpot(self, cell: Cell, symbol: Option<&str>) {
        if self.edit(|preset| preset.set_quickpot(cell, symbol)) {
            self.audition();
        }
    }

    pub fn bind_expression_pedal(self, cell: Cell) {
        let Some((block, spec)) = self.draft.with_untracked(|preset| {
            let block = preset.block(cell)?;
            let spec = block.model.expression_param()?.clone();
            Some((block.id, spec))
        }) else {
            return;
        };
        let block_name = self.draft.with_untracked(|preset| {
            preset
                .block(cell)
                .map_or_default(|found| found.model.name.clone())
        });
        let current = self
            .draft
            .with_untracked(|preset| preset.binding(Actuator::ExpPedal).cloned());
        let other = current
            .as_ref()
            .filter(|binding| !binding.drives(block))
            .map(|binding| binding.name.clone());
        let is_confirmed = other.is_none_or(|other| {
            crate::host::confirm(&format!(
                "The expression pedal drives \"{other}\". Bind it to {block_name} instead?"
            ))
        });
        if !is_confirmed {
            return;
        }
        let binding = Binding::single(
            &default_binding_name(&block_name, Some(&spec.name)),
            BindingTarget {
                block,
                symbol: spec.symbol.clone(),
                min: spec.min,
                max: spec.max,
                extra: serde_json::Map::new(),
            },
        );
        self.set_binding(Actuator::ExpPedal, Some(binding));
    }

    pub fn set_binding(self, actuator: Actuator, binding: Option<Binding>) {
        if self.edit(|preset| preset.set_binding(actuator, binding)) {
            self.audition();
        }
    }

    pub fn set_preset_midi_out(self, messages: Vec<MidiOutMessage>) {
        self.edit(|preset| preset.midi_out_mut().on_preset = messages);
    }

    pub fn set_scene_midi_out(self, scene: Slot, messages: Vec<MidiOutMessage>) {
        self.edit(|preset| {
            let on_scene = &mut preset.midi_out_mut().on_scene;
            if messages.is_empty() {
                on_scene.remove(&scene);
            } else {
                on_scene.insert(scene, messages);
            }
        });
    }

    #[must_use]
    pub fn is_block_edited(self, cell: Cell) -> bool {
        self.block_id(cell)
            .is_some_and(|id| self.edits.with(|edits| edits.is_block_edited(id)))
    }

    #[must_use]
    pub fn is_property_edited(self, cell: Cell, index: usize) -> bool {
        self.block_id(cell)
            .is_some_and(|id| self.edits.with(|edits| edits.is_property_edited(id, index)))
    }

    #[must_use]
    pub fn is_value_edited(self, cell: Cell, index: usize) -> bool {
        self.block_id(cell)
            .is_some_and(|id| self.edits.with(|edits| edits.is_value_edited(id, index)))
    }

    pub(crate) fn block_id(self, cell: Cell) -> Option<BlockId> {
        self.draft
            .with_untracked(|preset| preset.block(cell).map(|block| block.id))
    }

    fn edit_scope(self) -> EditScope {
        edit_scope(
            self.mode.get_untracked(),
            self.scene_control.get_untracked(),
            self.scene.get_untracked(),
        )
    }
}

const fn edit_scope(mode: Mode, control: SceneControl, scene: Slot) -> EditScope {
    match (mode, control) {
        (Mode::Scene, SceneControl::All) => EditScope::AllScenes,
        (Mode::Scene, SceneControl::Active) => EditScope::Scene(scene),
        _ => EditScope::Preset,
    }
}

fn toggle_member<T: Ord>(set: &mut BTreeSet<T>, member: T) {
    if !set.remove(&member) {
        set.insert(member);
    }
}

fn push_bounded(stack: RwSignal<VecDeque<Preset>>, preset: Preset) {
    stack.update(|stack| {
        if stack.len() >= UNDO_DEPTH {
            stack.pop_front();
        }
        stack.push_back(preset);
    });
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{edit_scope, toggle_member};
    use crate::model::{
        mode::{Mode, SceneControl},
        preset::EditScope,
        slot::Slot,
    };

    #[test]
    fn edits_outside_scene_mode_change_the_preset() {
        let scene = Slot::default();
        assert_eq!(
            edit_scope(Mode::Preset, SceneControl::Active, scene),
            EditScope::Preset
        );
        assert_eq!(
            edit_scope(Mode::Scene, SceneControl::All, scene),
            EditScope::AllScenes
        );
        assert_eq!(
            edit_scope(Mode::Scene, SceneControl::Active, scene),
            EditScope::Scene(scene)
        );
    }

    #[test]
    fn toggling_a_member_twice_leaves_the_set_unchanged() {
        let mut set = BTreeSet::from([1, 2]);
        toggle_member(&mut set, 3);
        assert_eq!(set, BTreeSet::from([1, 2, 3]));
        toggle_member(&mut set, 3);
        assert_eq!(set, BTreeSet::from([1, 2]));
    }
}
