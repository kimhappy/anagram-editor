pub mod audio;
pub mod general;
pub mod midi;
pub mod value;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

pub use self::{
    audio::{GlobalEq, InputGain, Mixer},
    general::GeneralSettings,
    midi::{MidiSettings, ThroughMode, midi_keys},
};
use super::area::PresetArea;
use crate::protocol::midi::{Mode, SlotNumber};

pub const SELECTED_PRESET: &str = "selected-preset";
pub const SELECTED_PRESET_AREA: &str = "selected-preset-area";
pub const MODE: &str = "mode";
pub const USER_AREAS: &str = "user-areas";
pub const FAVOURITE_PLUGINS: &str = "plugins.favorite";

const LIVE_KEYS: &[&str] = &[general::BRIGHTNESS];

fn needs_restart(key: &str) -> bool {
    !LIVE_KEYS.contains(&key) && !audio::is_input_gain_name(key)
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Settings(pub Map<String, Value>);

impl Settings {
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.get(key)
    }

    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "settings hold small non-negative whole numbers, written as floats at times"
    )]
    pub(crate) fn u64_or(&self, key: &str, default: u64) -> u64 {
        self.get(key)
            .and_then(Value::as_f64)
            .map_or(default, |number| number.max(0.0) as u64)
    }

    pub(crate) fn bool_or(&self, key: &str, default: bool) -> bool {
        self.get(key).and_then(Value::as_bool).unwrap_or(default)
    }

    pub(crate) fn f64_or(&self, key: &str, default: f64) -> f64 {
        self.get(key)
            .and_then(Value::as_f64)
            .filter(|number| number.is_finite())
            .unwrap_or(default)
    }

    #[expect(
        clippy::cast_possible_truncation,
        reason = "settings hold small whole numbers, written as floats at times"
    )]
    pub(crate) fn i64_or(&self, key: &str, default: i64) -> i64 {
        self.get(key)
            .and_then(Value::as_f64)
            .filter(|number| number.is_finite())
            .map_or(default, |number| number.round() as i64)
    }

    pub(crate) fn str_or(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(Value::as_str)
    }

    #[must_use]
    pub fn strings(&self, key: &str) -> Vec<String> {
        self.get(key)
            .and_then(Value::as_array)
            .map_or_else(Vec::new, |values| {
                values
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect()
            })
    }

    #[must_use]
    pub fn favourite_plugins(&self) -> Vec<String> {
        self.strings(FAVOURITE_PLUGINS)
    }

    #[must_use]
    pub fn device_settings(&self) -> DeviceSettings {
        DeviceSettings {
            midi: MidiSettings::read(self),
            general: GeneralSettings::read(self),
            eq: GlobalEq::read(self),
            mixer: Mixer::read(self),
            input_gain: InputGain::read(self),
        }
    }

    #[must_use]
    pub fn device_state(&self) -> DeviceState {
        DeviceState {
            preset_index: u8::try_from(self.u64_or(SELECTED_PRESET, 0))
                .unwrap_or(u8::MAX)
                .min(SlotNumber::COUNT - 1),
            area: self
                .get(SELECTED_PRESET_AREA)
                .and_then(Value::as_str)
                .and_then(PresetArea::from_setting)
                .unwrap_or(PresetArea::FIRST_USER),
            mode: self
                .get(MODE)
                .and_then(Value::as_str)
                .and_then(Mode::from_setting)
                .unwrap_or_default(),
        }
    }

    #[must_use]
    pub fn user_area_names(&self) -> Vec<String> {
        let names = self.strings(USER_AREAS);
        if names.is_empty() {
            vec!["User 1".to_owned()]
        } else {
            names
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceState {
    pub preset_index: u8,
    pub area: PresetArea,
    pub mode: Mode,
}

impl Default for DeviceState {
    fn default() -> Self {
        Self {
            preset_index: 0,
            area: PresetArea::FIRST_USER,
            mode: Mode::default(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DeviceSettings {
    pub midi: MidiSettings,
    pub general: GeneralSettings,
    pub eq: GlobalEq,
    pub mixer: Mixer,
    pub input_gain: InputGain,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SettingsChange {
    pub changes: Map<String, Value>,
    pub needs_restart: bool,
}

impl SettingsChange {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }
}

impl DeviceSettings {
    fn entries(&self) -> Vec<(String, Value)> {
        [
            self.midi.entries(),
            self.general.entries(),
            self.eq.entries(),
            self.mixer.entries(),
            self.input_gain.entries(),
        ]
        .concat()
    }

    #[must_use]
    pub fn with_changes(&self, changes: &Map<String, Value>) -> Self {
        let mut entries: Map<String, Value> = self.entries().into_iter().collect();
        entries.extend(changes.clone());
        Settings(entries).device_settings()
    }

    #[must_use]
    pub fn diff(&self, target: &Self) -> SettingsChange {
        let changes: Map<String, Value> = self
            .entries()
            .into_iter()
            .zip(target.entries())
            .filter(|((_, from), (_, to))| from != to)
            .map(|(_, change)| change)
            .collect();
        let needs_restart = changes.keys().any(|key| needs_restart(key));
        SettingsChange {
            changes,
            needs_restart,
        }
    }
}

#[must_use]
pub fn favourites_payload(uris: Vec<String>) -> Map<String, Value> {
    std::iter::once((FAVOURITE_PLUGINS.to_owned(), json!(uris))).collect()
}

#[must_use]
pub fn selection_payload(area: PresetArea, preset_index: u8) -> Map<String, Value> {
    [
        (SELECTED_PRESET_AREA.to_owned(), json!(area.setting_value())),
        (SELECTED_PRESET.to_owned(), json!(preset_index)),
    ]
    .into_iter()
    .collect()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        DeviceSettings, Settings, ThroughMode,
        audio::{EqBandId, MixerOutput, StereoPair},
    };
    use crate::protocol::{
        actuator::Actuator,
        hid::area::PresetArea,
        midi::{Channel, Mode},
    };

    fn encode_settings_for_test(settings: serde_json::Map<String, serde_json::Value>) -> String {
        crate::protocol::hid::action::encode_request(&crate::protocol::hid::action::EditSettings {
            settings,
        })
        .expect("encodes")
    }

    fn settings(value: serde_json::Value) -> Settings {
        serde_json::from_value(value).expect("parses")
    }

    #[test]
    fn changes_carry_over_onto_settings_read_later() {
        let opened = DeviceSettings::default();
        let mut edited = opened.clone();
        edited.general.bpm = 150.0;
        edited.eq.update(EqBandId::Band2, |band| band.gain = -4.5);
        let mut live = opened.clone();
        live.mixer
            .update(MixerOutput::ALL[0], |channel| channel.volume = -6.0);
        let rebased = live.with_changes(&opened.diff(&edited).changes);
        assert_eq!(live.with_changes(&serde_json::Map::new()), live);
        assert_eq!(
            rebased,
            DeviceSettings {
                mixer: live.mixer,
                ..edited
            }
        );
    }

    #[test]
    fn reads_live_state_and_defaults() {
        let read = settings(json!({
            "mode": "scene", "selected-preset": 2.0, "selected-preset-area": "user-2",
            "midi.channel": 0, "midi.bindings.foot1.cc": 40, "midi.bindings.knob2.enabled": false,
            "user-areas": ["Live", "Studio"]
        }));
        let state = read.device_state();
        assert_eq!(state.preset_index, 2);
        assert_eq!(state.area, PresetArea::User(2));
        assert_eq!(state.mode, Mode::Scene);
        let midi = read.device_settings().midi;
        assert_eq!(midi.channel, None);
        assert_eq!(midi.send_channel(), Channel::FIRST);
        assert_eq!(midi.bindings.cc(Actuator::FootA), 40);
        assert!(!midi.bindings.is_enabled(Actuator::Knob2));
        assert!(midi.bindings.is_enabled(Actuator::Knob1));
        assert_eq!(midi.through, ThroughMode::Trs);
        assert_eq!(read.user_area_names(), ["Live", "Studio"]);
        assert_eq!(Settings::default().user_area_names(), ["User 1"]);
    }

    #[test]
    fn device_state_clamps_out_of_range_values() {
        let high = settings(json!({"selected-preset": 500, "selected-preset-area": "bogus"}))
            .device_state();
        assert_eq!(high.preset_index, 125);
        assert_eq!(high.area, PresetArea::FIRST_USER);
        assert_eq!(
            settings(json!({"selected-preset": -3.0}))
                .device_state()
                .preset_index,
            0
        );
    }

    #[test]
    fn an_out_of_range_channel_reads_as_omni() {
        assert_eq!(
            settings(json!({"midi.channel": 17}))
                .device_settings()
                .midi
                .channel,
            None
        );
    }

    #[test]
    fn diff_lists_changed_keys_only() {
        let current = DeviceSettings::default();
        let mut target = current.clone();
        target.midi.channel = None;
        target.midi.bindings.set_cc(Actuator::Knob1, 33);
        let change = current.diff(&target);
        assert_eq!(change.changes.len(), 2);
        assert_eq!(change.changes.get("midi.channel"), Some(&json!(0)));
        assert_eq!(
            change.changes.get("midi.bindings.knob1.cc"),
            Some(&json!(33))
        );
        assert!(change.needs_restart);
    }

    #[test]
    fn brightness_and_input_gain_names_apply_without_a_restart() {
        let current = DeviceSettings::default();
        let mut target = current.clone();
        target.general.brightness = 3;
        target.input_gain.presets[2].name = "BASS".to_owned();
        assert!(!current.diff(&target).needs_restart);
        target.mixer.set_linked(StereoPair::Jack, false);
        assert!(current.diff(&target).needs_restart);
    }

    #[test]
    fn a_tempo_change_needs_a_restart_and_stays_a_float() {
        let current = DeviceSettings::default();
        let mut target = current.clone();
        target.general.bpm = 100.0;
        let change = current.diff(&target);
        assert!(change.needs_restart);
        assert_eq!(
            encode_settings_for_test(change.changes),
            r#"{"action":"edit_settings","payload":{"settings":{"transport.bpm":100.0}}}"#
        );
    }

    #[test]
    fn audio_settings_write_floats_and_integer_decibels() {
        let current = DeviceSettings::default();
        let mut target = current.clone();
        target.eq.update(EqBandId::Band2, |band| band.gain = 3.0);
        target
            .mixer
            .update(MixerOutput::XlrLeft, |channel| channel.volume = -2.0);
        target.input_gain.presets[1].db = -3;
        let change = current.diff(&target);
        assert!(change.needs_restart);
        let changes = change.changes;
        assert!(
            changes
                .get("globaleq.2.gain")
                .is_some_and(serde_json::Value::is_f64)
        );
        assert!(
            changes
                .get("mixer.xlr.left.vol")
                .is_some_and(serde_json::Value::is_f64)
        );
        assert!(
            changes
                .get("mixer.xlr.right.vol")
                .is_some_and(serde_json::Value::is_f64)
        );
        assert_eq!(changes.get("input-gain-preset1.db"), Some(&json!(-3)));
    }

    #[test]
    fn unlinked_stereo_outputs_change_alone() {
        let mut mixer = super::Mixer::default();
        mixer.set_linked(StereoPair::Jack, false);
        mixer.update(MixerOutput::JackLeft, |channel| channel.volume = -6.0);
        assert!((mixer.channel(MixerOutput::JackRight).volume).abs() < f64::EPSILON);
        mixer.set_linked(StereoPair::Jack, true);
        assert!((mixer.channel(MixerOutput::JackRight).volume + 6.0).abs() < f64::EPSILON);
        mixer.update(MixerOutput::JackLeft, |channel| channel.volume = -9.0);
        assert!((mixer.channel(MixerOutput::JackRight).volume + 9.0).abs() < f64::EPSILON);
    }

    #[test]
    fn stored_audio_settings_are_read_back() {
        let read = settings(json!({
            "display.brightness": 7, "globaleq.hs.freq": 9000.0, "mixer.jack.left.vol": 6.0,
            "input-gain-preset": 2, "input-gain-preset2.db": -4, "input-gain-preset2.name": "BASS VI",
            "transport.bpm": 184.0, "plugins.favorite": ["urn:a", 3, "urn:b"]
        }));
        let device = read.device_settings();
        assert_eq!(device.general.brightness, 7);
        assert!((device.general.bpm - 184.0).abs() < f64::EPSILON);
        assert!((device.eq.band(EqBandId::HighShelf).freq - 9000.0).abs() < f64::EPSILON);
        assert!((device.mixer.channel(MixerOutput::JackLeft).volume - 6.0).abs() < f64::EPSILON);
        assert_eq!(device.input_gain.active, 2);
        assert_eq!(device.input_gain.presets[2].db, -4);
        assert_eq!(device.input_gain.presets[2].name, "BASS VI");
        assert_eq!(read.favourite_plugins(), ["urn:a", "urn:b"]);
    }
}
