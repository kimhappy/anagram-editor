use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Actuator {
    Knob1,
    Knob2,
    Knob3,
    Knob4,
    Knob5,
    Knob6,
    FootA,
    FootB,
    FootC,
    ExpPedal,
}

impl Actuator {
    pub const ALL: [Self; 10] = [
        Self::Knob1,
        Self::Knob2,
        Self::Knob3,
        Self::Knob4,
        Self::Knob5,
        Self::Knob6,
        Self::FootA,
        Self::FootB,
        Self::FootC,
        Self::ExpPedal,
    ];

    #[must_use]
    pub const fn preset_key(self) -> &'static str {
        match self {
            Self::Knob1 => "pot1",
            Self::Knob2 => "pot2",
            Self::Knob3 => "pot3",
            Self::Knob4 => "pot4",
            Self::Knob5 => "pot5",
            Self::Knob6 => "pot6",
            Self::FootA => "foot1",
            Self::FootB => "foot2",
            Self::FootC => "foot3",
            Self::ExpPedal => "exp.pedal",
        }
    }

    #[must_use]
    pub fn from_preset_key(key: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|actuator| actuator.preset_key() == key)
    }

    #[must_use]
    pub const fn settings_key(self) -> &'static str {
        match self {
            Self::Knob1 => "knob1",
            Self::Knob2 => "knob2",
            Self::Knob3 => "knob3",
            Self::Knob4 => "knob4",
            Self::Knob5 => "knob5",
            Self::Knob6 => "knob6",
            Self::FootA => "foot1",
            Self::FootB => "foot2",
            Self::FootC => "foot3",
            Self::ExpPedal => "exp-pedal",
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Knob1 => "Knob 1",
            Self::Knob2 => "Knob 2",
            Self::Knob3 => "Knob 3",
            Self::Knob4 => "Knob 4",
            Self::Knob5 => "Knob 5",
            Self::Knob6 => "Knob 6",
            Self::FootA => "Foot A",
            Self::FootB => "Foot B",
            Self::FootC => "Foot C",
            Self::ExpPedal => "Exp pedal",
        }
    }

    #[must_use]
    pub const fn short(self) -> &'static str {
        match self {
            Self::Knob1 => "1",
            Self::Knob2 => "2",
            Self::Knob3 => "3",
            Self::Knob4 => "4",
            Self::Knob5 => "5",
            Self::Knob6 => "6",
            Self::FootA => "A",
            Self::FootB => "B",
            Self::FootC => "C",
            Self::ExpPedal => "E",
        }
    }

    #[must_use]
    pub const fn default_cc(self) -> u8 {
        match self {
            Self::Knob1 => 20,
            Self::Knob2 => 21,
            Self::Knob3 => 22,
            Self::Knob4 => 23,
            Self::Knob5 => 24,
            Self::Knob6 => 25,
            Self::FootA => 17,
            Self::FootB => 18,
            Self::FootC => 19,
            Self::ExpPedal => 89,
        }
    }

    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }
}

impl fmt::Display for Actuator {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}

#[cfg(test)]
mod tests {
    use super::Actuator;

    #[test]
    fn preset_keys_round_trip() {
        for actuator in Actuator::ALL {
            assert_eq!(
                Actuator::from_preset_key(actuator.preset_key()),
                Some(actuator)
            );
        }
        assert_eq!(Actuator::from_preset_key("pot7"), None);
    }

    #[test]
    fn default_ccs_follow_the_firmware_table() {
        let ccs: Vec<u8> = Actuator::ALL.map(Actuator::default_cc).to_vec();
        assert_eq!(ccs, [20, 21, 22, 23, 24, 25, 17, 18, 19, 89]);
    }
}
