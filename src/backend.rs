use std::{fmt, future::Future, pin::Pin, sync::Arc};

use serde_json::{Map, Value};

use crate::{
    anagram::transfer::TransferError,
    devices::DeviceError,
    model::{
        catalog::Catalog,
        library::{AreaIndex, Library, PresetRef},
        slot::Slot,
    },
    protocol::{
        hid::{
            area::PresetArea,
            envelope::HidError,
            files::{UserDir, UserFile},
            plugin::PluginSummary,
            preset::PresetDocument,
            settings::{DeviceSettings, DeviceState, Settings},
            version::Version,
        },
        midi::Command,
    },
};

pub type LocalFuture<'call, T> = Pin<Box<dyn Future<Output = T> + 'call>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackendError {
    Hid(HidError),
    Device(DeviceError),
    Transfer(TransferError),
    NotWritten { failed: Vec<Slot> },
}

impl fmt::Display for BackendError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Hid(error) => write!(formatter, "{error}"),
            Self::Device(error) => write!(formatter, "{error}"),
            Self::Transfer(error) => write!(formatter, "{error}"),
            Self::NotWritten { failed } => write!(
                formatter,
                "the Anagram could not write {}",
                failed
                    .iter()
                    .map(Slot::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}

impl std::error::Error for BackendError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Hid(error) => Some(error),
            Self::Device(error) => Some(error),
            Self::Transfer(error) => Some(error),
            Self::NotWritten { .. } => None,
        }
    }
}

impl From<HidError> for BackendError {
    fn from(error: HidError) -> Self {
        Self::Hid(error)
    }
}

impl From<DeviceError> for BackendError {
    fn from(error: DeviceError) -> Self {
        Self::Device(error)
    }
}

impl From<TransferError> for BackendError {
    fn from(error: TransferError) -> Self {
        Self::Transfer(error)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DeviceSnapshot {
    pub state: DeviceState,
    pub settings: DeviceSettings,
    pub favourites: Vec<String>,
    pub user_area_names: Vec<String>,
}

impl DeviceSnapshot {
    #[must_use]
    pub fn from_settings(settings: &Settings) -> Self {
        Self {
            state: settings.device_state(),
            settings: settings.device_settings(),
            favourites: settings.favourite_plugins(),
            user_area_names: settings.user_area_names(),
        }
    }

    #[must_use]
    pub fn library(&self) -> Library {
        Library::new(&self.user_area_names, self.state.area)
    }
}

impl Default for DeviceSnapshot {
    fn default() -> Self {
        Self {
            state: DeviceState::default(),
            settings: DeviceSettings::default(),
            favourites: Vec::new(),
            user_area_names: vec!["User 1".to_owned()],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileRemoval {
    Removed,
    EntryOnly,
}

pub struct UploadRequest {
    pub dir: UserDir,
    pub name: String,
    pub file_name: String,
    pub bytes: Vec<u8>,
}

pub trait Backend {
    fn catalog(&self) -> Arc<Catalog>;
    fn snapshot(&self) -> DeviceSnapshot;
    fn is_device(&self, device: &web_sys::HidDevice) -> bool;
    fn can_manage_plugins(&self) -> bool;
    fn firmware_version(&self) -> Version;

    fn load_index(&self, area: PresetArea) -> LocalFuture<'_, Result<AreaIndex, BackendError>>;
    fn fetch_preset(
        &self,
        preset_ref: PresetRef,
    ) -> LocalFuture<'_, Result<Option<PresetDocument>, BackendError>>;
    fn save_preset(
        &self,
        preset_ref: PresetRef,
        document: PresetDocument,
        reselect: bool,
    ) -> LocalFuture<'_, Result<(), BackendError>>;
    fn save_presets(
        &self,
        area: PresetArea,
        documents: Vec<(Slot, PresetDocument)>,
    ) -> LocalFuture<'_, Result<(), BackendError>>;
    fn try_preset(&self, document: PresetDocument) -> LocalFuture<'_, Result<(), BackendError>>;
    fn swap_presets(
        &self,
        area: PresetArea,
        from: Slot,
        to: Slot,
    ) -> LocalFuture<'_, Result<(), BackendError>>;
    fn delete_preset(&self, preset_ref: PresetRef) -> LocalFuture<'_, Result<(), BackendError>>;
    fn select_area(
        &self,
        area: PresetArea,
        slot: Slot,
    ) -> LocalFuture<'_, Result<(), BackendError>>;
    fn read_state(&self) -> LocalFuture<'_, Result<DeviceSnapshot, BackendError>>;
    fn edit_settings(
        &self,
        changes: Map<String, Value>,
    ) -> LocalFuture<'_, Result<(), BackendError>>;
    fn send_midi(&self, command: Command) -> Result<(), BackendError>;
    fn user_files(&self, dir: UserDir) -> LocalFuture<'_, Result<Vec<UserFile>, BackendError>>;
    fn rename_user_file(
        &self,
        file: UserFile,
        name: String,
    ) -> LocalFuture<'_, Result<(), BackendError>>;
    fn remove_user_file(
        &self,
        file: UserFile,
    ) -> LocalFuture<'_, Result<FileRemoval, BackendError>>;
    fn ensure_serial(&self) -> LocalFuture<'_, Result<(), BackendError>>;
    fn has_serial(&self) -> bool;
    fn user_plugins(&self) -> LocalFuture<'_, Result<Vec<PluginSummary>, BackendError>>;
    fn delete_plugin(&self, uri: String) -> LocalFuture<'_, Result<(), BackendError>>;
    fn upload<'call>(
        &'call self,
        request: UploadRequest,
        progress: Box<dyn FnMut(f64) + 'call>,
    ) -> LocalFuture<'call, Result<Option<String>, BackendError>>;
    fn reboot(&self) -> LocalFuture<'_, Result<(), BackendError>>;
    fn send_firmware<'call>(
        &'call self,
        file_name: String,
        package: Vec<u8>,
        progress: Box<dyn FnMut(f64) + 'call>,
    ) -> LocalFuture<'call, Result<(), BackendError>>;
    fn start_firmware_install(&self) -> LocalFuture<'_, Result<(), BackendError>>;
}
