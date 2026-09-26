use std::time::Duration;

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Map, Value};

use super::{
    envelope::{HidError, Request, decode_payload},
    files::{FirmwareReceiver, UploadRecord, UserFile},
    plugin::{PluginInfo, PluginSummary},
    preset::PresetDocument,
    settings::Settings,
    uuid::{self, PresetUuid},
    version::Version,
};
use crate::protocol::midi::SlotNumber;

pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(7);
const RECEIVER_TIMEOUT: Duration = Duration::from_secs(15);
const PLUGIN_LIST_TIMEOUT: Duration = Duration::from_secs(20);

pub trait Action: Serialize {
    const NAME: &'static str;
    const REPLY: Option<&'static str> = None;
    const TIMEOUT: Duration = DEFAULT_TIMEOUT;
    const DATA_KEY: Option<&'static str> = Some("data");
    const REPORTS_OWN_STATUS: bool = false;
    const VERIFIES_REPLY: bool = false;
    type Data: DeserializeOwned;

    fn accepts(&self, _data: &Self::Data) -> bool {
        true
    }
}

pub trait Notification: Serialize {
    const NAME: &'static str;
}

#[must_use]
pub fn reply_name<A: Action>() -> String {
    A::REPLY.map_or_else(|| format!("{}_res", A::NAME), str::to_owned)
}

pub fn encode_request<A: Action>(action: &A) -> Result<String, HidError> {
    encode(A::NAME, action)
}

pub fn encode_notification<N: Notification>(notification: &N) -> Result<String, HidError> {
    encode(N::NAME, notification)
}

fn encode(name: &'static str, payload: &impl Serialize) -> Result<String, HidError> {
    serde_json::to_string(&Request {
        action: name,
        payload,
    })
    .map_err(|error| HidError::Encoding(error.to_string()))
}

pub fn decode_reply<A: Action>(text: &str) -> Result<A::Data, HidError> {
    decode_payload(A::NAME, text, A::DATA_KEY, !A::REPORTS_OWN_STATUS)
}

#[must_use]
pub fn preset_filename(number: u8) -> String {
    format!("{number}.json")
}

#[must_use]
pub fn parse_preset_filename(filename: &str) -> Option<u8> {
    filename
        .strip_suffix(".json")?
        .parse::<u8>()
        .ok()
        .filter(|&number| SlotNumber::new(number).is_some())
}

#[expect(
    clippy::empty_structs_with_brackets,
    reason = "serialises as an empty object"
)]
#[derive(Serialize)]
pub struct FetchFirmwareVersion {}

#[derive(Clone, Debug, Deserialize)]
pub struct FirmwareVersion {
    pub version: Version,
    #[serde(default)]
    pub edition: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

impl Action for FetchFirmwareVersion {
    const NAME: &'static str = "fetch_firmware_version";
    type Data = FirmwareVersion;
}

#[expect(
    clippy::empty_structs_with_brackets,
    reason = "serialises as an empty object"
)]
#[derive(Serialize)]
pub struct ExportSettings {}

#[derive(Clone, Debug, Deserialize)]
pub struct SettingsExport {
    pub settings: Settings,
}

impl Action for ExportSettings {
    const NAME: &'static str = "export_settings";
    const DATA_KEY: Option<&'static str> = None;
    type Data = SettingsExport;
}

#[derive(Serialize)]
pub struct EditSettings {
    pub settings: Map<String, Value>,
}

#[expect(
    clippy::empty_structs_with_brackets,
    reason = "deserialises from any object"
)]
#[derive(Deserialize)]
pub struct NoData {}

impl Action for EditSettings {
    const NAME: &'static str = "edit_settings";
    const DATA_KEY: Option<&'static str> = None;
    type Data = NoData;
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPresetPositions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_area: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PresetPosition {
    pub filename: String,
    #[serde(default, deserialize_with = "uuid::lenient")]
    pub id: Option<PresetUuid>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct PresetPositions {
    #[serde(default)]
    pub files: Vec<PresetPosition>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct PositionsReply {
    #[serde(default)]
    pub data: PresetPositions,
    #[serde(default)]
    pub err_message: Option<String>,
}

const LISTING_FAILURE: &str = "Error: An Exception occurred";

impl PositionsReply {
    pub fn into_positions(self) -> Result<PresetPositions, String> {
        match self.err_message {
            Some(message) if message.starts_with(LISTING_FAILURE) => Err(message),
            _ => Ok(self.data),
        }
    }
}

impl Action for GetPresetPositions {
    const NAME: &'static str = "get_anagram_preset_positions";
    const DATA_KEY: Option<&'static str> = None;
    const REPORTS_OWN_STATUS: bool = true;
    type Data = PositionsReply;
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchPreset {
    pub filename: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_area: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct PresetFile {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub content: Option<PresetDocument>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchedPreset {
    #[serde(default)]
    pub preset_file: Option<PresetFile>,
}

impl FetchedPreset {
    #[must_use]
    pub fn into_document(self) -> Option<PresetDocument> {
        self.preset_file?.content
    }
}

impl Action for FetchPreset {
    const NAME: &'static str = "fetch_anagram_preset";
    const VERIFIES_REPLY: bool = true;
    type Data = FetchedPreset;

    fn accepts(&self, data: &Self::Data) -> bool {
        data.preset_file
            .as_ref()
            .and_then(|file| file.name.as_deref())
            .is_none_or(|name| name == self.filename)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NamedDocument {
    pub name: String,
    pub content: PresetDocument,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditPresets {
    pub presets: Vec<NamedDocument>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_area: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct WriteReport {
    #[serde(default)]
    pub success: Option<i64>,
    #[serde(default)]
    pub file_errors: Vec<String>,
    #[serde(default)]
    pub err_message: Option<String>,
}

impl WriteReport {
    #[must_use]
    pub fn failed_numbers(&self) -> Vec<u8> {
        self.file_errors
            .iter()
            .filter_map(|path| parse_preset_filename(path.rsplit('/').next().unwrap_or(path)))
            .collect()
    }

    #[must_use]
    pub fn is_success(&self) -> bool {
        self.success == Some(1) && self.file_errors.is_empty()
    }
}

impl Action for EditPresets {
    const NAME: &'static str = "edit_anagram_presets";
    const DATA_KEY: Option<&'static str> = None;
    const REPORTS_OWN_STATUS: bool = true;
    type Data = WriteReport;
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TryPreset {
    pub preset_data: PresetDocument,
}

impl Action for TryPreset {
    const NAME: &'static str = "try_anagram_preset";
    const DATA_KEY: Option<&'static str> = None;
    type Data = NoData;
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapPresets {
    pub from_index: u8,
    pub to_index: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_area: Option<String>,
}

impl Action for SwapPresets {
    const NAME: &'static str = "swap_anagram_presets";
    const DATA_KEY: Option<&'static str> = None;
    type Data = NoData;
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeletePreset {
    pub filename: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_area: Option<String>,
}

impl Action for DeletePreset {
    const NAME: &'static str = "delete_anagram_preset";
    const DATA_KEY: Option<&'static str> = None;
    type Data = NoData;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum PluginFilter {
    #[serde(rename = "USER")]
    User,
}

#[derive(Serialize)]
pub struct ReadAllPlugins {
    #[serde(rename = "filterOption", skip_serializing_if = "Option::is_none")]
    pub filter: Option<PluginFilter>,
}

impl Action for ReadAllPlugins {
    const NAME: &'static str = "read_all_plugins";
    const TIMEOUT: Duration = PLUGIN_LIST_TIMEOUT;
    type Data = Vec<PluginSummary>;
}

#[derive(Serialize)]
pub struct GetPluginInfo {
    pub uri: String,
}

#[derive(Serialize)]
pub struct DeletePlugin {
    pub uri: String,
}

impl Action for DeletePlugin {
    const NAME: &'static str = "delete_plugin";
    const DATA_KEY: Option<&'static str> = None;
    type Data = NoData;
}

impl Action for GetPluginInfo {
    const NAME: &'static str = "get_plugin_info";
    const VERIFIES_REPLY: bool = true;
    type Data = PluginInfo;

    fn accepts(&self, data: &Self::Data) -> bool {
        data.uri.is_empty() || data.uri == self.uri
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FindUserFiles {
    pub page_num: u32,
    pub selected_dir: String,
}

impl Action for FindUserFiles {
    const NAME: &'static str = "find_user_files";
    type Data = Vec<UserFile>;
}

#[derive(Serialize)]
#[serde(transparent)]
pub struct OpenFileReceiver<R>(pub R);

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiverOpened {
    #[serde(default)]
    pub file_name: Option<String>,
    #[serde(default)]
    pub id: Option<Value>,
}

impl Action for OpenFileReceiver<UploadRecord> {
    const NAME: &'static str = "open_file_receiver";
    const TIMEOUT: Duration = RECEIVER_TIMEOUT;
    type Data = ReceiverOpened;
}

impl Action for OpenFileReceiver<FirmwareReceiver> {
    const NAME: &'static str = "open_file_receiver";
    type Data = ReceiverOpened;
}

#[derive(Serialize)]
#[serde(transparent)]
pub struct UpdateUserFile(pub UserFile);

impl Action for UpdateUserFile {
    const NAME: &'static str = "update_user_file";
    const REPLY: Option<&'static str> = Some("update_user_files_res");
    type Data = UserFile;
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveUserFile {
    pub id: u64,
    pub file_name: String,
    pub dir_name: String,
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
pub struct UserFileRemoval {
    #[serde(default)]
    pub fs_success: Option<i64>,
}

impl UserFileRemoval {
    #[must_use]
    pub fn removed_the_file(self) -> bool {
        self.fs_success != Some(0)
    }
}

impl Action for RemoveUserFile {
    const NAME: &'static str = "remove_user_file";
    const DATA_KEY: Option<&'static str> = None;
    type Data = UserFileRemoval;
}

#[expect(
    clippy::empty_structs_with_brackets,
    reason = "serialises as an empty object"
)]
#[derive(Serialize)]
pub struct Reboot {}

impl Notification for Reboot {
    const NAME: &'static str = "reboot";
}

#[expect(
    clippy::empty_structs_with_brackets,
    reason = "serialises as an empty object"
)]
#[derive(Serialize)]
pub struct FirmwareTransferCompleted {}

impl Notification for FirmwareTransferCompleted {
    const NAME: &'static str = "firmware_transfer_completed";
}

#[cfg(test)]
mod tests {
    use super::{
        Action, DeletePlugin, EditPresets, ExportSettings, FetchFirmwareVersion, FetchPreset,
        FirmwareTransferCompleted, GetPresetPositions, OpenFileReceiver, PluginFilter,
        PresetPosition, ReadAllPlugins, Reboot, RemoveUserFile, decode_reply, encode_notification,
        encode_request, parse_preset_filename, preset_filename,
    };

    #[test]
    fn a_fetched_preset_for_another_file_is_not_accepted() {
        let request = FetchPreset {
            filename: preset_filename(5),
            user_area: None,
        };
        let reply = |name: &str| {
            decode_reply::<FetchPreset>(&format!(
                r#"{{"payload":{{"success":1,"data":{{"presetFile":{{"name":"{name}","content":{{"preset":{{}}}}}}}}}}}}"#
            ))
            .expect("decodes")
        };
        assert!(request.accepts(&reply("5.json")));
        assert!(!request.accepts(&reply("4.json")));
        let empty =
            decode_reply::<FetchPreset>(r#"{"payload":{"success":1,"data":{"presetFile":{}}}}"#)
                .expect("decodes");
        assert!(request.accepts(&empty));
    }

    #[test]
    fn one_unreadable_preset_does_not_fail_the_whole_listing() {
        let reply = decode_reply::<GetPresetPositions>(
            r#"{"payload":{"data":{"files":[{"filename":"1.json","id":null,"name":null,"error":"'uuid'"},{"filename":"2.json","id":null,"name":"LEAD"}]},"err_message":"'uuid'"}}"#,
        )
        .expect("decodes");
        let positions = reply.into_positions().expect("lists");
        assert_eq!(positions.files.len(), 2);
        let failed = decode_reply::<GetPresetPositions>(
            r#"{"payload":{"data":{"files":[]},"err_message":"Error: An Exception occurred - no such directory"}}"#,
        )
        .expect("decodes");
        assert!(failed.into_positions().ok().is_none());
    }

    #[test]
    fn a_partial_write_names_the_files_that_failed() {
        let report = decode_reply::<EditPresets>(
            r#"{"payload":{"success":0,"file_errors":["/data/user-files/presets/7.json"],"err_message":"File operation failed"}}"#,
        )
        .expect("decodes without failing");
        assert!(!report.is_success());
        assert_eq!(report.failed_numbers(), [7]);
        let clean = decode_reply::<EditPresets>(
            r#"{"payload":{"success":1,"file_errors":[],"err_message":""}}"#,
        )
        .expect("decodes");
        assert!(clean.is_success());
    }

    #[test]
    fn a_removal_reports_whether_the_file_itself_went() {
        let removal = |fs_success: u8| {
            decode_reply::<RemoveUserFile>(&format!(
                r#"{{"payload":{{"data":{{"id":3}},"success":1,"fs_success":{fs_success},"err_message":""}}}}"#
            ))
            .expect("decodes")
        };
        assert!(removal(1).removed_the_file());
        assert!(!removal(0).removed_the_file());
    }

    #[test]
    fn an_opened_receiver_names_the_stored_file() {
        let opened = decode_reply::<OpenFileReceiver<crate::protocol::hid::files::UploadRecord>>(
            r#"{"payload":{"success":1,"err_message":"","data":{"fileName":"B9.wav","id":8}}}"#,
        )
        .expect("decodes");
        assert_eq!(opened.file_name.as_deref(), Some("B9.wav"));
    }

    #[test]
    fn a_firmware_receiver_takes_the_bare_file_name() {
        let receiver =
            crate::protocol::hid::files::FirmwareReceiver::new("Downloads/anagram 1.19 fw.tar");
        assert_eq!(
            encode_request(&OpenFileReceiver(receiver)).expect("encodes"),
            r#"{"action":"open_file_receiver","payload":{"fileName":"anagram_1.19_fw.tar","dirName":"","isFirmware":true}}"#
        );
    }

    #[test]
    fn notifications_are_bare_action_names() {
        assert_eq!(
            encode_notification(&Reboot {}).expect("encodes"),
            r#"{"action":"reboot","payload":{}}"#
        );
        assert_eq!(
            encode_notification(&FirmwareTransferCompleted {}).expect("encodes"),
            r#"{"action":"firmware_transfer_completed","payload":{}}"#
        );
    }

    #[test]
    fn listed_ids_that_are_not_device_ids_become_none() {
        let listed: Vec<PresetPosition> = serde_json::from_str(
            r#"[{"filename":"1.json","id":"123e4567-e89b-12d3-a456-426614174000"},{"filename":"2.json","id":5},{"filename":"3.json","id":"197FB05C1ED44DCC-8AAF4F87-D1E07CE0-73D21F2F4AC7E9AD7B2EC02D"}]"#,
        )
        .expect("the list parses");
        let ids: Vec<Option<&str>> = listed
            .iter()
            .map(|position| position.id.as_ref().map(super::PresetUuid::as_str))
            .collect();
        assert_eq!(
            ids,
            [
                None,
                None,
                Some("197fb05c1ed44dcc-8aaf4f87-d1e07ce0-73d21f2f4ac7e9ad7b2ec02d")
            ]
        );
    }

    #[test]
    fn requests_match_the_suite_byte_for_byte() {
        assert_eq!(
            encode_request(&FetchFirmwareVersion {}).expect("encodes"),
            r#"{"action":"fetch_firmware_version","payload":{}}"#
        );
        assert_eq!(
            encode_request(&FetchPreset {
                filename: preset_filename(35),
                user_area: None
            })
            .expect("encodes"),
            r#"{"action":"fetch_anagram_preset","payload":{"filename":"35.json"}}"#
        );
        assert_eq!(
            encode_request(&GetPresetPositions {
                user_area: Some("user-2".to_owned())
            })
            .expect("encodes"),
            r#"{"action":"get_anagram_preset_positions","payload":{"userArea":"user-2"}}"#
        );
        assert_eq!(
            encode_request(&ReadAllPlugins {
                filter: Some(PluginFilter::User)
            })
            .expect("encodes"),
            r#"{"action":"read_all_plugins","payload":{"filterOption":"USER"}}"#
        );
        assert_eq!(
            encode_request(&DeletePlugin {
                uri: "urn:darkglass:RoomReverb".to_owned()
            })
            .expect("encodes"),
            r#"{"action":"delete_plugin","payload":{"uri":"urn:darkglass:RoomReverb"}}"#
        );
    }

    #[test]
    fn replies_decode_into_their_data() {
        let firmware = decode_reply::<FetchFirmwareVersion>(
            r#"{"payload":{"success":1,"data":{"version":"v1.18.0.18\n","edition":"PBL","capabilities":["settings"]}}}"#,
        )
        .expect("decodes");
        assert_eq!(firmware.version.to_string(), "1.18.0.18");
        assert_eq!(firmware.capabilities, ["settings"]);

        let exported = decode_reply::<ExportSettings>(
            r#"{"action":"export_settings_res","payload":{"settings":{"mode":"preset"},"soundcard":null,"success":1,"err_message":""}}"#,
        )
        .expect("decodes");
        assert_eq!(
            exported.settings.device_state().mode,
            crate::protocol::midi::Mode::Preset
        );

        let missing = decode_reply::<FetchPreset>(
            r#"{"payload":{"success":1,"err_message":"file does not exist","data":{"presetFile":{}}}}"#,
        )
        .expect("decodes");
        assert!(missing.into_document().is_none());
    }

    #[test]
    fn filenames_round_trip() {
        assert_eq!(parse_preset_filename("7.json"), Some(7));
        assert_eq!(parse_preset_filename("0.json"), None);
        assert_eq!(parse_preset_filename("999.json"), None);
        assert_eq!(parse_preset_filename("7.txt"), None);
    }
}
