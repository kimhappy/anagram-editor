use serde::{Deserialize, Serialize};

use super::lenient;

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct PluginSummary {
    pub uri: String,
    #[serde(default)]
    pub version: String,
    #[serde(rename = "type", default, deserialize_with = "lenient::int")]
    pub license: u8,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct PluginInfo {
    pub uri: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub abbreviation: String,
    #[serde(default)]
    pub version: String,
    #[serde(default, deserialize_with = "lenient::int")]
    pub category: u8,
    #[serde(default, deserialize_with = "lenient::int")]
    pub flags: u32,
    #[serde(default)]
    pub ports: Vec<PortInfo>,
    #[serde(default)]
    pub properties: Vec<PropertyInfo>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct PortInfo {
    pub symbol: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub shortname: String,
    #[serde(default, deserialize_with = "lenient::int")]
    pub flags: u32,
    #[serde(default, deserialize_with = "lenient::int")]
    pub designation: u8,
    #[serde(default)]
    pub def: f64,
    #[serde(default)]
    pub min: f64,
    #[serde(default)]
    pub max: f64,
    #[serde(default)]
    pub unit: String,
    #[serde(rename = "scalePoints", default)]
    pub scale_points: Vec<ScalePoint>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct ScalePoint {
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub value: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct PropertyInfo {
    pub uri: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub shortname: String,
    #[serde(default, deserialize_with = "lenient::int")]
    pub flags: u32,
    #[serde(default)]
    pub defpath: String,
}

pub mod port_flags {
    pub const AUDIO: u32 = 1;
    pub const CONTROL: u32 = 2;
    pub const OUTPUT: u32 = 4;
    pub const SIDECHAIN: u32 = 8;
    pub const TOGGLED: u32 = 16;
    pub const INTEGER: u32 = 32;
    pub const ENUMERATED: u32 = 64;
    pub const LOGARITHMIC: u32 = 128;
    pub const HIDDEN: u32 = 256;
    pub const EXPENSIVE: u32 = 512;
    pub const EXPRESSION_PEDAL: u32 = 1024;
    pub const MAY_UPDATE_BLOCKED_STATE: u32 = 2048;
    pub const SAVED_TO_PRESET: u32 = 4096;
}

pub mod property_flags {
    pub const IS_PATH: u32 = 1;
    pub const IS_PARAMETER: u32 = 2;
    pub const READ_ONLY: u32 = 4;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Designation {
    None,
    Enabled,
    Bpm,
    Reset,
    QuickPot,
    Other(u8),
}

impl From<u8> for Designation {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::None,
            1 => Self::Enabled,
            2 => Self::Bpm,
            3 => Self::Reset,
            4 => Self::QuickPot,
            other => Self::Other(other),
        }
    }
}

impl PortInfo {
    #[must_use]
    pub const fn has(&self, flag: u32) -> bool {
        self.flags & flag != 0
    }

    #[must_use]
    pub fn designation(&self) -> Designation {
        Designation::from(self.designation)
    }

    #[must_use]
    pub fn is_user_control(&self) -> bool {
        use port_flags::{CONTROL, HIDDEN, OUTPUT, SAVED_TO_PRESET};
        let is_visible = !self.has(HIDDEN) || self.has(SAVED_TO_PRESET);
        let is_host_driven = matches!(
            self.designation(),
            Designation::Enabled | Designation::Bpm | Designation::Reset
        );
        self.has(CONTROL) && !self.has(OUTPUT) && is_visible && !is_host_driven
    }
}

impl PropertyInfo {
    #[must_use]
    pub const fn is_read_only(&self) -> bool {
        self.flags & property_flags::READ_ONLY != 0
    }
}

#[cfg(test)]
mod tests {
    use super::{Designation, PluginInfo, port_flags};

    const FET_COMP: &str = r#"{
      "uri": "urn:darkglass:FETComp", "name": "FET Compressor", "abbreviation": "FET", "version": "1",
      "category": 6, "flags": 11, "bundlepath": "/usr/lib/lv2/fetcomp.lv2/",
      "ports": [
        { "symbol": "lv2_enabled", "name": "Enabled", "flags": 18, "designation": 1, "def": 1, "min": 0, "max": 1, "unit": "", "scalePoints": [] },
        { "symbol": "Blend", "name": "Blend", "shortname": "", "flags": 34, "designation": 0, "def": 100.0, "min": 0.0, "max": 100.0, "unit": "%", "scalePoints": [] },
        { "symbol": "Ratio", "name": "Ratio", "shortname": "", "flags": 98, "designation": 0, "def": 1.0, "min": 1.0, "max": 5.0, "unit": "", "scalePoints": [ { "label": "1:1", "value": 1.0 } ] },
        { "symbol": "GR", "name": "Gain reduction", "flags": 6, "designation": 0, "def": 0, "min": -60, "max": 0, "unit": "db" }
      ],
      "properties": []
    }"#;

    #[test]
    fn filters_user_controls() {
        let info: PluginInfo = serde_json::from_str(FET_COMP).expect("parses");
        let controls: Vec<&str> = info
            .ports
            .iter()
            .filter(|port| port.is_user_control())
            .map(|port| port.symbol.as_str())
            .collect();
        assert_eq!(controls, ["Blend", "Ratio"]);
        let ratio = info.ports.get(2).expect("ratio");
        assert!(ratio.has(port_flags::ENUMERATED) && ratio.has(port_flags::INTEGER));
        assert_eq!(
            info.ports.first().map(super::PortInfo::designation),
            Some(Designation::Enabled)
        );
    }
}
