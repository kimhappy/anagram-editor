use serde_json::{Value, json};

use super::Settings;
use crate::protocol::{
    actuator::Actuator,
    midi::{BindingCcs, Channel},
};

pub mod midi_keys {
    use crate::protocol::actuator::Actuator;

    pub const CHANNEL: &str = "midi.channel";
    pub const THROUGH_MODE: &str = "midi.midi-through.mode";
    pub const PC_THROUGH: &str = "midi.pc-through";
    pub const IGNORE_REDUNDANT_PC: &str = "midi.ignore-redundant-pc";
    pub const TOGGLE_MODE: &str = "midi.toggle-mode";

    #[must_use]
    pub fn binding_cc(actuator: Actuator) -> String {
        format!("midi.bindings.{}.cc", actuator.settings_key())
    }

    #[must_use]
    pub fn binding_enabled(actuator: Actuator) -> String {
        format!("midi.bindings.{}.enabled", actuator.settings_key())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ThroughMode {
    #[default]
    Trs,
    Usb,
    TrsAndUsb,
    Off,
}

impl ThroughMode {
    pub const ALL: [Self; 4] = [Self::Trs, Self::Usb, Self::TrsAndUsb, Self::Off];

    #[must_use]
    pub const fn from_setting(value: u64) -> Self {
        match value {
            1 => Self::Usb,
            2 => Self::TrsAndUsb,
            3 => Self::Off,
            _ => Self::Trs,
        }
    }

    #[must_use]
    pub const fn setting_value(self) -> u64 {
        match self {
            Self::Trs => 0,
            Self::Usb => 1,
            Self::TrsAndUsb => 2,
            Self::Off => 3,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Trs => "3.5mm",
            Self::Usb => "USB",
            Self::TrsAndUsb => "3.5mm + USB",
            Self::Off => "Off",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MidiSettings {
    pub channel: Option<Channel>,
    pub through: ThroughMode,
    pub pc_through: bool,
    pub ignore_redundant_pc: bool,
    pub toggle_logic: bool,
    pub bindings: BindingCcs,
}

impl Default for MidiSettings {
    fn default() -> Self {
        Self {
            channel: Some(Channel::FIRST),
            through: ThroughMode::Trs,
            pc_through: true,
            ignore_redundant_pc: true,
            toggle_logic: false,
            bindings: BindingCcs::default(),
        }
    }
}

impl MidiSettings {
    #[must_use]
    pub fn read(settings: &Settings) -> Self {
        let defaults = Self::default();
        let mut bindings = BindingCcs::default();
        for actuator in Actuator::ALL {
            let cc = settings.u64_or(
                &midi_keys::binding_cc(actuator),
                u64::from(actuator.default_cc()),
            );
            bindings.set_cc(actuator, u8::try_from(cc).unwrap_or(u8::MAX));
            bindings.set_enabled(
                actuator,
                settings.bool_or(&midi_keys::binding_enabled(actuator), true),
            );
        }
        Self {
            channel: Channel::new(
                u8::try_from(settings.u64_or(midi_keys::CHANNEL, 1)).unwrap_or(1),
            ),
            through: ThroughMode::from_setting(settings.u64_or(midi_keys::THROUGH_MODE, 0)),
            pc_through: settings.bool_or(midi_keys::PC_THROUGH, defaults.pc_through),
            ignore_redundant_pc: settings
                .bool_or(midi_keys::IGNORE_REDUNDANT_PC, defaults.ignore_redundant_pc),
            toggle_logic: settings.bool_or(midi_keys::TOGGLE_MODE, defaults.toggle_logic),
            bindings,
        }
    }

    #[must_use]
    pub fn send_channel(&self) -> Channel {
        self.channel.unwrap_or(Channel::FIRST)
    }

    #[must_use]
    pub const fn is_pc_through_available(&self) -> bool {
        !matches!(self.through, ThroughMode::Off)
    }

    #[must_use]
    pub fn entries(&self) -> Vec<(String, Value)> {
        let general = [
            (
                midi_keys::CHANNEL,
                json!(self.channel.map_or(0, Channel::get)),
            ),
            (midi_keys::THROUGH_MODE, json!(self.through.setting_value())),
            (midi_keys::PC_THROUGH, json!(self.pc_through)),
            (
                midi_keys::IGNORE_REDUNDANT_PC,
                json!(self.ignore_redundant_pc),
            ),
            (midi_keys::TOGGLE_MODE, json!(self.toggle_logic)),
        ]
        .map(|(key, value)| (key.to_owned(), value));
        let bindings = Actuator::ALL.into_iter().flat_map(|actuator| {
            [
                (
                    midi_keys::binding_cc(actuator),
                    json!(self.bindings.cc(actuator)),
                ),
                (
                    midi_keys::binding_enabled(actuator),
                    json!(self.bindings.is_enabled(actuator)),
                ),
            ]
        });
        general.into_iter().chain(bindings).collect()
    }
}
