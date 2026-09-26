use std::{cell::RefCell, collections::BTreeMap, rc::Rc, sync::Arc};

use leptos::prelude::*;
use serde_json::{Map, Value};

use crate::{
    backend::{Backend, BackendError, DeviceSnapshot, FileRemoval, LocalFuture, UploadRequest},
    host,
    model::{
        catalog::Catalog,
        library::{AreaIndex, PresetRef},
        slot::Slot,
        testing,
    },
    protocol::{
        hid::{
            action::{PresetPosition, preset_filename},
            area::PresetArea,
            files::{UserDir, UserFile},
            plugin::PluginSummary,
            preset::PresetDocument,
            settings::DeviceState,
            uuid::PresetUuid,
            version::Version,
        },
        midi::Command,
    },
    session::Session,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Call {
    Select(PresetRef),
    Save(PresetArea, Vec<Slot>),
    Try,
    Swap(Slot, Slot),
    Delete(PresetRef),
    Settings(Map<String, Value>),
    Midi(Command),
    RemoveFile(String),
    Reboot,
}

pub struct FakeAnagram {
    pub catalog: Arc<Catalog>,
    pub presets: RefCell<BTreeMap<PresetRef, PresetDocument>>,
    pub snapshot: RefCell<DeviceSnapshot>,
    pub calls: RefCell<Vec<Call>>,
}

impl FakeAnagram {
    pub fn new(presets: impl IntoIterator<Item = (PresetRef, PresetDocument)>) -> Rc<Self> {
        Rc::new(Self {
            catalog: Arc::new(testing::catalog()),
            presets: RefCell::new(presets.into_iter().collect()),
            snapshot: RefCell::new(DeviceSnapshot::default()),
            calls: RefCell::new(Vec::new()),
        })
    }

    pub fn calls(&self) -> Vec<Call> {
        self.calls.borrow().clone()
    }

    pub fn stored(&self, preset_ref: PresetRef) -> Option<PresetDocument> {
        self.presets.borrow().get(&preset_ref).cloned()
    }

    fn record(&self, call: Call) {
        self.calls.borrow_mut().push(call);
    }

    fn positions(&self, area: PresetArea) -> Vec<PresetPosition> {
        self.presets
            .borrow()
            .iter()
            .filter(|(preset_ref, _)| preset_ref.area == area)
            .map(|(preset_ref, document)| PresetPosition {
                filename: preset_filename(preset_ref.slot.number()),
                id: document.preset.uuid.as_deref().and_then(PresetUuid::parse),
                name: document.preset.name.clone(),
                error: None,
            })
            .collect()
    }

    fn land(&self, preset_ref: PresetRef) {
        let mut snapshot = self.snapshot.borrow_mut();
        snapshot.state = DeviceState {
            area: preset_ref.area,
            preset_index: preset_ref.slot.byte_index(),
            ..snapshot.state
        };
    }
}

pub fn document(name: &str, seed: u8) -> PresetDocument {
    let mut document = PresetDocument::default();
    document.preset.name = Some(name.to_owned());
    document.preset.uuid = Some(PresetUuid::from_bytes([seed; 28]).as_str().to_owned());
    document
}

pub fn user_slot(text: &str) -> PresetRef {
    PresetRef {
        area: PresetArea::FIRST_USER,
        slot: text.parse().expect("a slot label"),
    }
}

pub fn started(fake: &Rc<FakeAnagram>) -> Session {
    Owner::new().set();
    let session = Session::provide(Rc::clone(fake) as Rc<dyn Backend>);
    session.start();
    session.stop_polling();
    host::run_until_idle();
    fake.calls.borrow_mut().clear();
    session
}

fn ready<'call, T: 'call>(value: T) -> LocalFuture<'call, T> {
    Box::pin(std::future::ready(value))
}

impl Backend for FakeAnagram {
    fn catalog(&self) -> Arc<Catalog> {
        Arc::clone(&self.catalog)
    }

    fn snapshot(&self) -> DeviceSnapshot {
        self.snapshot.borrow().clone()
    }

    fn is_device(&self, _device: &web_sys::HidDevice) -> bool {
        false
    }

    fn can_manage_plugins(&self) -> bool {
        true
    }

    fn firmware_version(&self) -> Version {
        "1.18.0.18".parse().expect("a version")
    }

    fn load_index(&self, area: PresetArea) -> LocalFuture<'_, Result<AreaIndex, BackendError>> {
        ready(Ok(if area.is_writable() {
            AreaIndex::from_positions(&self.positions(area))
        } else {
            AreaIndex::read_only(area)
        }))
    }

    fn fetch_preset(
        &self,
        preset_ref: PresetRef,
    ) -> LocalFuture<'_, Result<Option<PresetDocument>, BackendError>> {
        ready(Ok(self.stored(preset_ref)))
    }

    fn save_preset(
        &self,
        preset_ref: PresetRef,
        document: PresetDocument,
        reselect: bool,
    ) -> LocalFuture<'_, Result<(), BackendError>> {
        if reselect {
            self.record(Call::Select(preset_ref));
            self.land(preset_ref);
        }
        self.save_presets(preset_ref.area, vec![(preset_ref.slot, document)])
    }

    fn save_presets(
        &self,
        area: PresetArea,
        documents: Vec<(Slot, PresetDocument)>,
    ) -> LocalFuture<'_, Result<(), BackendError>> {
        self.record(Call::Save(
            area,
            documents.iter().map(|(slot, _)| *slot).collect(),
        ));
        self.presets.borrow_mut().extend(
            documents
                .into_iter()
                .map(|(slot, document)| (PresetRef { area, slot }, document)),
        );
        ready(Ok(()))
    }

    fn try_preset(&self, _document: PresetDocument) -> LocalFuture<'_, Result<(), BackendError>> {
        self.record(Call::Try);
        ready(Ok(()))
    }

    fn swap_presets(
        &self,
        area: PresetArea,
        from: Slot,
        to: Slot,
    ) -> LocalFuture<'_, Result<(), BackendError>> {
        self.record(Call::Swap(from, to));
        let mut presets = self.presets.borrow_mut();
        let first = presets.remove(&PresetRef { area, slot: from });
        let second = presets.remove(&PresetRef { area, slot: to });
        presets.extend(first.map(|document| (PresetRef { area, slot: to }, document)));
        presets.extend(second.map(|document| (PresetRef { area, slot: from }, document)));
        ready(Ok(()))
    }

    fn delete_preset(&self, preset_ref: PresetRef) -> LocalFuture<'_, Result<(), BackendError>> {
        self.record(Call::Delete(preset_ref));
        self.presets.borrow_mut().remove(&preset_ref);
        ready(Ok(()))
    }

    fn select_area(
        &self,
        area: PresetArea,
        slot: Slot,
    ) -> LocalFuture<'_, Result<(), BackendError>> {
        let preset_ref = PresetRef { area, slot };
        self.record(Call::Select(preset_ref));
        self.land(preset_ref);
        ready(Ok(()))
    }

    fn read_state(&self) -> LocalFuture<'_, Result<DeviceSnapshot, BackendError>> {
        ready(Ok(self.snapshot()))
    }

    fn edit_settings(
        &self,
        changes: Map<String, Value>,
    ) -> LocalFuture<'_, Result<(), BackendError>> {
        let mut stored = crate::protocol::hid::settings::Settings::default();
        stored.0.clone_from(&changes);
        let written = stored.device_settings();
        if changes.keys().any(|key| key.starts_with("transport.")) {
            self.snapshot.borrow_mut().settings.general.bpm = written.general.bpm;
        }
        self.record(Call::Settings(changes));
        ready(Ok(()))
    }

    fn send_midi(&self, command: Command) -> Result<(), BackendError> {
        self.record(Call::Midi(command));
        if let Command::SelectPreset(number) = command
            && let Some(slot) = Slot::from_number(number.get())
        {
            let area = self.snapshot.borrow().state.area;
            self.land(PresetRef { area, slot });
        }
        Ok(())
    }

    fn user_files(&self, _dir: UserDir) -> LocalFuture<'_, Result<Vec<UserFile>, BackendError>> {
        ready(Ok(Vec::new()))
    }

    fn rename_user_file(
        &self,
        _file: UserFile,
        _name: String,
    ) -> LocalFuture<'_, Result<(), BackendError>> {
        ready(Ok(()))
    }

    fn remove_user_file(
        &self,
        file: UserFile,
    ) -> LocalFuture<'_, Result<FileRemoval, BackendError>> {
        self.record(Call::RemoveFile(file.file_name));
        ready(Ok(FileRemoval::Removed))
    }

    fn has_serial(&self) -> bool {
        true
    }

    fn ensure_serial(&self) -> LocalFuture<'_, Result<(), BackendError>> {
        ready(Ok(()))
    }

    fn user_plugins(&self) -> LocalFuture<'_, Result<Vec<PluginSummary>, BackendError>> {
        ready(Ok(Vec::new()))
    }

    fn delete_plugin(&self, _uri: String) -> LocalFuture<'_, Result<(), BackendError>> {
        ready(Ok(()))
    }

    fn upload<'call>(
        &'call self,
        _request: UploadRequest,
        _progress: Box<dyn FnMut(f64) + 'call>,
    ) -> LocalFuture<'call, Result<Option<String>, BackendError>> {
        ready(Ok(None))
    }

    fn reboot(&self) -> LocalFuture<'_, Result<(), BackendError>> {
        self.record(Call::Reboot);
        ready(Ok(()))
    }

    fn send_firmware<'call>(
        &'call self,
        _file_name: String,
        _package: Vec<u8>,
        _progress: Box<dyn FnMut(f64) + 'call>,
    ) -> LocalFuture<'call, Result<(), BackendError>> {
        ready(Ok(()))
    }

    fn start_firmware_install(&self) -> LocalFuture<'_, Result<(), BackendError>> {
        ready(Ok(()))
    }
}
