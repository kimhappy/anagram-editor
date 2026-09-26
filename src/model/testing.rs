use std::sync::OnceLock;

use super::catalog::{Catalog, FX_LOOP_URI, MERGE_URI, RETURN_URI, SEND_URI, SPLIT_URI};
use crate::protocol::hid::plugin::{PluginInfo, PortInfo, ScalePoint, port_flags};

pub const GAIN: &str = "urn:test:Gain";
pub const DRIVE: &str = "urn:test:Drive";
pub const CHORUS: &str = "urn:test:Chorus";
pub const CHORUS_STEREO: &str = "urn:test:ChorusStereo";
pub const FILTER: &str = "urn:test:Filter";
pub const REVERB: &str = "urn:test:Reverb";

fn control(symbol: &str, name: &str, def: f64, min: f64, max: f64, unit: &str) -> PortInfo {
    PortInfo {
        symbol: symbol.to_owned(),
        name: name.to_owned(),
        flags: port_flags::CONTROL,
        def,
        min,
        max,
        unit: unit.to_owned(),
        ..PortInfo::default()
    }
}

fn audio(symbol: &str, flags: u32) -> PortInfo {
    PortInfo {
        symbol: symbol.to_owned(),
        flags: port_flags::AUDIO | flags,
        ..PortInfo::default()
    }
}

#[must_use]
pub fn plugin(uri: &str, name: &str, category: u8, ports: Vec<PortInfo>) -> PluginInfo {
    PluginInfo {
        uri: uri.to_owned(),
        name: name.to_owned(),
        category,
        ports: [audio("in", 0), audio("out", port_flags::OUTPUT)]
            .into_iter()
            .chain(ports)
            .collect(),
        ..PluginInfo::default()
    }
}

fn plugins() -> Vec<PluginInfo> {
    let enabled = PortInfo {
        symbol: ":bypass".to_owned(),
        flags: port_flags::CONTROL | port_flags::TOGGLED,
        designation: 1,
        max: 1.0,
        ..PortInfo::default()
    };
    let meter = PortInfo {
        symbol: "meter".to_owned(),
        flags: port_flags::CONTROL | port_flags::OUTPUT,
        max: 1.0,
        ..PortInfo::default()
    };
    let mode = PortInfo {
        symbol: "mode".to_owned(),
        name: "Mode".to_owned(),
        flags: port_flags::CONTROL | port_flags::ENUMERATED,
        max: 2.0,
        scale_points: vec![
            ScalePoint {
                label: "Soft".to_owned(),
                value: 0.0,
            },
            ScalePoint {
                label: "Hard".to_owned(),
                value: 2.0,
            },
        ],
        ..PortInfo::default()
    };
    let size = PortInfo {
        flags: port_flags::CONTROL | port_flags::EXPENSIVE,
        ..control("size", "Size", 5.0, 0.0, 10.0, "")
    };
    let chorus_ports = || {
        vec![
            control("rate", "Rate", 1.0, 0.1, 10.0, "Hz"),
            control("depth", "Depth", 0.5, 0.0, 1.0, ""),
            control("wet", "Wet", 5.0, 0.0, 10.0, "1dPt"),
        ]
    };
    vec![
        plugin(
            GAIN,
            "Gain",
            22,
            vec![
                enabled,
                meter,
                control("gain", "Gain", 0.0, -20.0, 20.0, "dB"),
            ],
        ),
        plugin(
            DRIVE,
            "Drive",
            2,
            vec![
                control("drive", "Drive", 5.0, 0.0, 10.0, ""),
                control("level", "Level", 0.0, -12.0, 12.0, "dB"),
                mode,
            ],
        ),
        plugin(CHORUS, "Chorus", 25, chorus_ports()),
        plugin(CHORUS_STEREO, "Chorus", 25, chorus_ports()),
        plugin(
            FILTER,
            "Filter",
            29,
            vec![control("cutoff", "Cutoff", 200.0, 20.0, 2000.0, "Hz")],
        ),
        plugin(
            REVERB,
            "Reverb",
            1,
            vec![size, control("mix", "Mix", 0.5, 0.0, 1.0, "")],
        ),
        plugin(SPLIT_URI, "Split", 34, Vec::new()),
        plugin(MERGE_URI, "Merge", 34, Vec::new()),
        plugin(SEND_URI, "Send", 34, Vec::new()),
        plugin(RETURN_URI, "Return", 34, Vec::new()),
        plugin(FX_LOOP_URI, "FX loop", 34, Vec::new()),
    ]
}

pub fn catalog() -> Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG
        .get_or_init(|| Catalog::from_plugins(&plugins()))
        .clone()
}
