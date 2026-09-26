use std::{collections::BTreeSet, sync::Arc};

use serde_json::{Map, Value};

use super::{Anagram, transfer};
use crate::{
    backend::{Backend, BackendError, DeviceSnapshot, FileRemoval, LocalFuture, UploadRequest},
    devices,
    model::{
        catalog::Catalog,
        library::{AreaIndex, PresetRef},
        slot::Slot,
    },
    protocol::{
        hid::{
            action::{
                Action, DeletePlugin, DeletePreset, EditPresets, EditSettings, ExportSettings,
                FetchPreset, FindUserFiles, FirmwareTransferCompleted, GetPresetPositions,
                NamedDocument, PluginFilter, ReadAllPlugins, Reboot, RemoveUserFile, SwapPresets,
                TryPreset, UpdateUserFile, WriteReport, preset_filename,
            },
            area::PresetArea,
            envelope::HidError,
            files::{FirmwareReceiver, UploadRecord, UserDir, UserFile},
            plugin::PluginSummary,
            preset::PresetDocument,
            settings::selection_payload,
            version::Version,
        },
        midi::Command,
    },
};

const RELOAD_FILENAME: &str = "999.json";
const PLUGIN_MANAGEMENT: &str = "plugin-management";

impl Anagram {
    async fn perform<A: Action>(&self, action: A) -> Result<(), BackendError> {
        self.hid.call(action).await?;
        Ok(())
    }

    async fn reload_bank(&self) -> Result<(), BackendError> {
        self.perform(DeletePreset {
            filename: RELOAD_FILENAME.to_owned(),
            user_area: None,
        })
        .await
    }

    async fn select(&self, area: PresetArea, slot: Slot) -> Result<(), BackendError> {
        self.perform(EditSettings {
            settings: selection_payload(area, slot.byte_index()),
        })
        .await
    }

    async fn serial_port(&self) -> Result<web_sys::SerialPort, BackendError> {
        if let Some(port) = self.serial.borrow().clone() {
            return Ok(port);
        }
        let filter = devices::serial::anagram::FILTER;
        let port = match devices::serial::granted_port(filter).await? {
            Some(port) => port,
            None => devices::serial::request_port(&[filter]).await?,
        };
        *self.serial.borrow_mut() = Some(port.clone());
        Ok(port)
    }
}

impl Backend for Anagram {
    fn catalog(&self) -> Arc<Catalog> {
        Arc::clone(&self.catalog)
    }

    fn snapshot(&self) -> DeviceSnapshot {
        self.snapshot.borrow().clone()
    }

    fn is_device(&self, device: &web_sys::HidDevice) -> bool {
        self.hid.is_device(device)
    }

    fn firmware_version(&self) -> Version {
        self.firmware.version.clone()
    }

    fn can_manage_plugins(&self) -> bool {
        self.firmware
            .capabilities
            .iter()
            .any(|capability| capability == PLUGIN_MANAGEMENT)
    }

    fn load_index(&self, area: PresetArea) -> LocalFuture<'_, Result<AreaIndex, BackendError>> {
        Box::pin(async move {
            if !area.is_writable() {
                return Ok(AreaIndex::read_only(area));
            }
            let positions = self
                .hid
                .call(GetPresetPositions {
                    user_area: area.user_area_payload(),
                })
                .await?
                .into_positions()
                .map_err(|message| {
                    BackendError::Hid(HidError::Device {
                        action: GetPresetPositions::NAME,
                        message,
                    })
                })?;
            Ok(AreaIndex::from_positions(&positions.files))
        })
    }

    fn fetch_preset(
        &self,
        preset_ref: PresetRef,
    ) -> LocalFuture<'_, Result<Option<PresetDocument>, BackendError>> {
        Box::pin(async move {
            if !preset_ref.area.is_writable() {
                return Ok(None);
            }
            let fetched = self
                .hid
                .call(FetchPreset {
                    filename: preset_filename(preset_ref.slot.number()),
                    user_area: preset_ref.area.user_area_payload(),
                })
                .await?;
            Ok(fetched.into_document())
        })
    }

    fn save_preset(
        &self,
        preset_ref: PresetRef,
        document: PresetDocument,
        reselect: bool,
    ) -> LocalFuture<'_, Result<(), BackendError>> {
        Box::pin(async move {
            if reselect {
                self.select(preset_ref.area, preset_ref.slot).await?;
            }
            let report = self
                .hid
                .call(EditPresets {
                    presets: vec![NamedDocument {
                        name: preset_filename(preset_ref.slot.number()),
                        content: document,
                    }],
                    user_area: preset_ref.area.user_area_payload(),
                })
                .await?;
            written(&report)
        })
    }

    fn save_presets(
        &self,
        area: PresetArea,
        documents: Vec<(Slot, PresetDocument)>,
    ) -> LocalFuture<'_, Result<(), BackendError>> {
        Box::pin(async move {
            let presets = documents
                .into_iter()
                .map(|(slot, content)| NamedDocument {
                    name: preset_filename(slot.number()),
                    content,
                })
                .collect();
            let report = self
                .hid
                .call(EditPresets {
                    presets,
                    user_area: area.user_area_payload(),
                })
                .await?;
            written(&report)
        })
    }

    fn try_preset(&self, document: PresetDocument) -> LocalFuture<'_, Result<(), BackendError>> {
        Box::pin(async move {
            self.perform(TryPreset {
                preset_data: document,
            })
            .await
        })
    }

    fn swap_presets(
        &self,
        area: PresetArea,
        from: Slot,
        to: Slot,
    ) -> LocalFuture<'_, Result<(), BackendError>> {
        Box::pin(async move {
            self.perform(SwapPresets {
                from_index: from.number(),
                to_index: to.number(),
                user_area: area.user_area_payload(),
            })
            .await
        })
    }

    fn delete_preset(&self, preset_ref: PresetRef) -> LocalFuture<'_, Result<(), BackendError>> {
        Box::pin(async move {
            self.perform(DeletePreset {
                filename: preset_filename(preset_ref.slot.number()),
                user_area: preset_ref.area.user_area_payload(),
            })
            .await
        })
    }

    fn select_area(
        &self,
        area: PresetArea,
        slot: Slot,
    ) -> LocalFuture<'_, Result<(), BackendError>> {
        Box::pin(async move {
            self.select(area, slot).await?;
            self.reload_bank().await
        })
    }

    fn read_state(&self) -> LocalFuture<'_, Result<DeviceSnapshot, BackendError>> {
        Box::pin(async move {
            let exported = self.hid.call(ExportSettings {}).await?;
            let snapshot = DeviceSnapshot::from_settings(&exported.settings);
            self.midi.apply(&snapshot.settings.midi);
            *self.snapshot.borrow_mut() = snapshot.clone();
            Ok(snapshot)
        })
    }

    fn edit_settings(
        &self,
        changes: Map<String, Value>,
    ) -> LocalFuture<'_, Result<(), BackendError>> {
        Box::pin(async move { self.perform(EditSettings { settings: changes }).await })
    }

    fn send_midi(&self, command: Command) -> Result<(), BackendError> {
        self.midi.send(command)?;
        Ok(())
    }

    fn user_files(&self, dir: UserDir) -> LocalFuture<'_, Result<Vec<UserFile>, BackendError>> {
        Box::pin(async move {
            let every_file = self
                .hid
                .call(FindUserFiles {
                    page_num: 0,
                    selected_dir: dir.name().to_owned(),
                })
                .await?;
            Ok(files_of(dir, every_file))
        })
    }

    fn rename_user_file(
        &self,
        file: UserFile,
        name: String,
    ) -> LocalFuture<'_, Result<(), BackendError>> {
        Box::pin(async move {
            self.perform(UpdateUserFile(file.renamed(&name, &now())))
                .await
        })
    }

    fn remove_user_file(
        &self,
        file: UserFile,
    ) -> LocalFuture<'_, Result<FileRemoval, BackendError>> {
        Box::pin(async move {
            let removal = self
                .hid
                .call(RemoveUserFile {
                    id: file.id,
                    file_name: file.file_name,
                    dir_name: file.dir_name,
                })
                .await?;
            Ok(if removal.removed_the_file() {
                FileRemoval::Removed
            } else {
                FileRemoval::EntryOnly
            })
        })
    }

    fn has_serial(&self) -> bool {
        self.serial.borrow().is_some()
    }

    fn ensure_serial(&self) -> LocalFuture<'_, Result<(), BackendError>> {
        Box::pin(async move {
            self.serial_port().await?;
            Ok(())
        })
    }

    fn user_plugins(&self) -> LocalFuture<'_, Result<Vec<PluginSummary>, BackendError>> {
        Box::pin(async move {
            Ok(self
                .hid
                .call(ReadAllPlugins {
                    filter: Some(PluginFilter::User),
                })
                .await?)
        })
    }

    fn delete_plugin(&self, uri: String) -> LocalFuture<'_, Result<(), BackendError>> {
        Box::pin(async move { self.perform(DeletePlugin { uri }).await })
    }

    fn upload<'call>(
        &'call self,
        request: UploadRequest,
        progress: Box<dyn FnMut(f64) + 'call>,
    ) -> LocalFuture<'call, Result<Option<String>, BackendError>> {
        Box::pin(async move {
            let port = self.serial_port().await?;
            let record = UploadRecord::new(
                request.dir,
                &request.file_name,
                &request.name,
                request.bytes.len() as u64,
                &now(),
            );
            let opened =
                transfer::upload(&self.hid, &port, record, &request.bytes, progress).await?;
            Ok(opened.file_name)
        })
    }

    fn reboot(&self) -> LocalFuture<'_, Result<(), BackendError>> {
        Box::pin(async move {
            self.hid.notify(Reboot {}).await?;
            Ok(())
        })
    }

    fn send_firmware<'call>(
        &'call self,
        file_name: String,
        package: Vec<u8>,
        progress: Box<dyn FnMut(f64) + 'call>,
    ) -> LocalFuture<'call, Result<(), BackendError>> {
        Box::pin(async move {
            let port = self.serial_port().await?;
            let receiver = FirmwareReceiver::new(&file_name);
            transfer::upload(&self.hid, &port, receiver, &package, progress).await?;
            Ok(())
        })
    }

    fn start_firmware_install(&self) -> LocalFuture<'_, Result<(), BackendError>> {
        Box::pin(async move {
            self.hid.notify(FirmwareTransferCompleted {}).await?;
            Ok(())
        })
    }
}

fn written(report: &WriteReport) -> Result<(), BackendError> {
    if report.is_success() {
        return Ok(());
    }
    let failed: Vec<Slot> = report
        .failed_numbers()
        .into_iter()
        .filter_map(Slot::from_number)
        .collect();
    if failed.is_empty() {
        Err(BackendError::Hid(HidError::Device {
            action: EditPresets::NAME,
            message: report
                .err_message
                .clone()
                .filter(|message| !message.is_empty())
                .unwrap_or_else(|| "device reported failure".to_owned()),
        }))
    } else {
        Err(BackendError::NotWritten { failed })
    }
}

fn files_of(dir: UserDir, every_file: Vec<UserFile>) -> Vec<UserFile> {
    let mut seen = BTreeSet::new();
    every_file
        .into_iter()
        .filter(|file| file.dir_name == dir.name() && seen.insert(file.id))
        .collect()
}

fn now() -> String {
    js_sys::Date::new_0()
        .to_iso_string()
        .as_string()
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::files_of;
    use crate::protocol::hid::files::{UserDir, UserFile};

    fn user_file(id: u64, dir: UserDir) -> UserFile {
        UserFile {
            id,
            name: String::new(),
            file_name: String::new(),
            dir_name: dir.name().to_owned(),
            file_size: None,
            uris: Vec::new(),
            original_file_name: None,
            extra: serde_json::Map::new(),
        }
    }

    #[test]
    fn the_full_listing_is_narrowed_to_one_directory_without_repeats() {
        let every_file = vec![
            user_file(1, UserDir::Cabinets),
            user_file(2, UserDir::NeuralModels),
            user_file(1, UserDir::Cabinets),
            user_file(3, UserDir::Cabinets),
        ];
        let ids: Vec<u64> = files_of(UserDir::Cabinets, every_file)
            .iter()
            .map(|file| file.id)
            .collect();
        assert_eq!(ids, [1, 3]);
    }
}
