use serde_json::{Value, json};

use super::{
    Settings,
    value::{Bounds, float},
};

pub const BRIGHTNESS: &str = "display.brightness";
pub const BPM: &str = "transport.bpm";

pub const BRIGHTNESS_LEVELS: std::ops::RangeInclusive<u8> = 1..=7;
pub const BRIGHTNESS_BOUNDS: Bounds = Bounds::linear(1.0, 7.0, 1.0, 0, "");
pub const BPM_BOUNDS: Bounds = Bounds::linear(40.0, 300.0, 0.1, 1, "BPM");

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeneralSettings {
    pub brightness: u8,
    pub bpm: f64,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            brightness: 6,
            bpm: 120.0,
        }
    }
}

impl GeneralSettings {
    #[must_use]
    pub fn read(settings: &Settings) -> Self {
        let defaults = Self::default();
        let brightness = u8::try_from(settings.u64_or(BRIGHTNESS, u64::from(defaults.brightness)))
            .unwrap_or(defaults.brightness)
            .clamp(*BRIGHTNESS_LEVELS.start(), *BRIGHTNESS_LEVELS.end());
        Self {
            brightness,
            bpm: BPM_BOUNDS.clamp(settings.f64_or(BPM, defaults.bpm)),
        }
    }

    #[must_use]
    pub fn entries(&self) -> Vec<(String, Value)> {
        vec![
            (BRIGHTNESS.to_owned(), json!(self.brightness)),
            (BPM.to_owned(), float(self.bpm)),
        ]
    }
}
