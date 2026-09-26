use std::fmt;

use super::actuator::Actuator;

const CONTROL_CHANGE: u8 = 0xB0;
const PROGRAM_CHANGE: u8 = 0xC0;
const TRIGGER: u8 = 1;

pub mod cc {
    pub const MODE: u8 = 85;
    pub const TUNER: u8 = 86;
    pub const SCENE_SELECT: u8 = 107;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Channel(u8);

impl Channel {
    pub const FIRST: Self = Self(1);

    #[must_use]
    pub const fn new(channel: u8) -> Option<Self> {
        if channel >= 1 && channel <= 16 {
            Some(Self(channel))
        } else {
            None
        }
    }

    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }

    const fn nibble(self) -> u8 {
        self.0 - 1
    }
}

impl fmt::Display for Channel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SlotNumber(u8);

impl SlotNumber {
    pub const COUNT: u8 = 126;

    #[must_use]
    pub const fn new(number: u8) -> Option<Self> {
        if number >= 1 && number <= Self::COUNT {
            Some(Self(number))
        } else {
            None
        }
    }

    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Mode {
    #[default]
    Preset,
    Stomp,
    Scene,
}

impl Mode {
    #[must_use]
    pub const fn cc_value(self) -> u8 {
        match self {
            Self::Preset => 1,
            Self::Stomp => 2,
            Self::Scene => 3,
        }
    }

    #[must_use]
    pub const fn setting_value(self) -> &'static str {
        match self {
            Self::Preset => "preset",
            Self::Stomp => "stomp",
            Self::Scene => "scene",
        }
    }

    #[must_use]
    pub fn from_setting(value: &str) -> Option<Self> {
        match value {
            "preset" => Some(Self::Preset),
            "stomp" => Some(Self::Stomp),
            "scene" => Some(Self::Scene),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BindingCcs {
    cc: [u8; 10],
    enabled: [bool; 10],
}

impl BindingCcs {
    #[must_use]
    pub fn cc(&self, actuator: Actuator) -> u8 {
        self.cc
            .get(actuator.index())
            .copied()
            .unwrap_or_else(|| actuator.default_cc())
    }

    #[must_use]
    pub fn is_enabled(&self, actuator: Actuator) -> bool {
        self.enabled.get(actuator.index()).copied().unwrap_or(true)
    }

    pub fn set_cc(&mut self, actuator: Actuator, cc: u8) {
        if let Some(slot) = self.cc.get_mut(actuator.index()) {
            *slot = cc.clamp(1, 127);
        }
    }

    #[must_use]
    pub fn holder_of(&self, cc: u8, except: Actuator) -> Option<Actuator> {
        Actuator::ALL
            .into_iter()
            .find(|actuator| *actuator != except && self.cc(*actuator) == cc)
    }

    pub fn assign(&mut self, actuator: Actuator, cc: u8) -> Option<Actuator> {
        let displaced = self.holder_of(cc, actuator);
        if let Some(other) = displaced {
            self.set_cc(other, other.default_cc());
        }
        self.set_cc(actuator, cc);
        displaced
    }

    pub fn set_enabled(&mut self, actuator: Actuator, enabled: bool) {
        if let Some(slot) = self.enabled.get_mut(actuator.index()) {
            *slot = enabled;
        }
    }
}

impl Default for BindingCcs {
    fn default() -> Self {
        Self {
            cc: Actuator::ALL.map(Actuator::default_cc),
            enabled: [true; 10],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    SelectMode(Mode),
    ToggleTuner,
    SelectPreset(SlotNumber),
    SelectScene(SlotNumber),
    BindingValue { cc: u8, value: u8 },
}

impl Command {
    #[must_use]
    pub fn encode(self, channel: Channel) -> Vec<u8> {
        let control = |controller: u8, value: u8| {
            vec![CONTROL_CHANGE | channel.nibble(), controller, value & 0x7F]
        };
        match self {
            Self::SelectMode(mode) => control(cc::MODE, mode.cc_value()),
            Self::ToggleTuner => control(cc::TUNER, TRIGGER),
            Self::SelectPreset(number) => vec![PROGRAM_CHANGE | channel.nibble(), number.get()],
            Self::SelectScene(number) => control(cc::SCENE_SELECT, number.get()),
            Self::BindingValue { cc, value } => control(cc, value),
        }
    }
}

pub const ALLOWED_CCS: [u8; 85] = [
    1, 2, 3, 4, 5, 6, 8, 9, 10, 11, 12, 13, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46,
    47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70,
    71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 88, 90, 91, 92, 93, 94, 95, 96, 97, 98,
    99, 100, 101, 120, 121, 122, 123, 124, 125, 126, 127,
];

#[cfg(test)]
mod tests {
    use super::{ALLOWED_CCS, BindingCcs, Channel, Command, Mode, SlotNumber};
    use crate::protocol::actuator::Actuator;

    fn encode(command: Command) -> Vec<u8> {
        command.encode(Channel::FIRST)
    }

    #[test]
    fn taking_a_used_cc_sends_the_other_binding_back_to_its_default() {
        let mut bindings = BindingCcs::default();
        assert_eq!(
            bindings.holder_of(17, Actuator::Knob1),
            Some(Actuator::FootA)
        );
        assert_eq!(bindings.holder_of(20, Actuator::Knob1), None);
        assert_eq!(bindings.assign(Actuator::FootB, 40), None);
        assert_eq!(bindings.assign(Actuator::Knob1, 40), Some(Actuator::FootB));
        assert_eq!(bindings.cc(Actuator::Knob1), 40);
        assert_eq!(bindings.cc(Actuator::FootB), 18);
    }

    #[test]
    fn worked_examples_from_the_spec() {
        assert_eq!(encode(Command::SelectMode(Mode::Scene)), [0xB0, 0x55, 0x03]);
        assert_eq!(
            encode(Command::SelectPreset(
                SlotNumber::new(42).expect("in range")
            )),
            [0xC0, 0x2A]
        );
        assert_eq!(
            encode(Command::SelectScene(SlotNumber::new(2).expect("in range"))),
            [0xB0, 0x6B, 0x02]
        );
        assert_eq!(encode(Command::ToggleTuner), [0xB0, 0x56, 0x01]);
        assert_eq!(
            encode(Command::BindingValue { cc: 20, value: 64 }),
            [0xB0, 20, 64]
        );
    }

    #[test]
    fn channel_lands_in_the_status_nibble() {
        let bytes = Command::ToggleTuner.encode(Channel::new(14).expect("valid"));
        assert_eq!(bytes, [0xBD, 0x56, 0x01]);
    }

    #[test]
    fn forty_three_ccs_are_reserved() {
        let is_reserved = |cc: u8| !ALLOWED_CCS.contains(&cc);
        let reserved = (0..=127).filter(|&cc| is_reserved(cc)).count();
        assert_eq!(reserved, 43);
        assert_eq!(ALLOWED_CCS.len(), 85);
        assert!(is_reserved(85) && is_reserved(107) && is_reserved(17));
        assert!(!is_reserved(1) && !is_reserved(88));
    }

    #[test]
    fn channels_run_from_one_to_sixteen() {
        assert_eq!(Channel::new(0), None);
        assert_eq!(Channel::new(17), None);
        assert_eq!(Channel::new(16).map(Channel::get), Some(16));
    }

    #[test]
    fn binding_ccs_are_clamped_to_the_cc_range() {
        let mut bindings = BindingCcs::default();
        bindings.set_cc(Actuator::Knob1, 0);
        bindings.set_cc(Actuator::Knob2, 200);
        assert_eq!(bindings.cc(Actuator::Knob1), 1);
        assert_eq!(bindings.cc(Actuator::Knob2), 127);
    }

    #[test]
    fn slot_numbers_stay_in_range() {
        assert_eq!(SlotNumber::new(0), None);
        assert_eq!(SlotNumber::new(127), None);
        assert_eq!(SlotNumber::new(126).map(SlotNumber::get), Some(126));
    }
}
