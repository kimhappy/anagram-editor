use leptos::prelude::*;

use crate::{
    model::{
        clipboard::Clipboard,
        library::{PresetRef, SlotEntry},
        preset::Preset,
        slot::Slot,
    },
    protocol::hid::version::Version,
    session::StoredPreset,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReconnectCause {
    Unplugged,
    Restart { reason: String },
    FirmwareInstall { expected: Option<Version> },
}

impl ReconnectCause {
    #[must_use]
    pub fn waiting_text(&self) -> String {
        match self {
            Self::Unplugged => {
                "The Anagram was disconnected. Plug it back in; the editor reconnects by itself."
                    .to_owned()
            }
            Self::Restart { reason } => {
                format!("The Anagram is restarting {reason}. The editor reconnects by itself.")
            }
            Self::FirmwareInstall { expected } => format!(
                "Installing firmware{}. The Anagram restarts into restore mode, installs the update and restarts again. Leave it plugged in; the editor reconnects by itself.",
                expected
                    .as_ref()
                    .map_or_default(|version| format!(" {version}"))
            ),
        }
    }

    #[must_use]
    pub fn arrival_notice(&self, installed: &Version) -> Option<(String, bool)> {
        match self {
            Self::Unplugged => None,
            Self::Restart { reason } => Some((format!("The Anagram restarted {reason}."), false)),
            Self::FirmwareInstall {
                expected: Some(expected),
            } if expected != installed => Some((
                format!(
                    "The Anagram reports firmware {installed}, not {expected}. Check the device screen."
                ),
                true,
            )),
            Self::FirmwareInstall { .. } => {
                Some((format!("Firmware {installed} is installed."), false))
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Reconnect {
    pub cause: ReconnectCause,
    pub deadline: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CarriedDraft {
    pub preset_ref: PresetRef,
    pub stored: StoredPreset,
    pub draft: Preset,
    pub is_new: bool,
    pub has_edits: bool,
    pub scene: Slot,
    pub clipboard: Clipboard,
}

#[derive(Clone, Copy)]
pub struct AppLink {
    pub reconnect: RwSignal<Option<Reconnect>>,
    pub carried: StoredValue<Option<CarriedDraft>>,
}

impl AppLink {
    #[must_use]
    pub fn new() -> Self {
        Self {
            reconnect: RwSignal::new(None),
            carried: StoredValue::new(None),
        }
    }
}

impl Default for AppLink {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CarryFate {
    Restore,
    ToClipboard,
}

#[must_use]
pub fn carried_fate(
    carried: &CarriedDraft,
    device: PresetRef,
    listed: Option<&SlotEntry>,
) -> CarryFate {
    let is_same_slot = carried.preset_ref == device;
    let is_same_content = match (carried.stored.identity.as_ref(), listed) {
        (Some(stored), Some(listed)) => stored.is_same(listed),
        (None, None) => true,
        _ => false,
    };
    if is_same_slot && is_same_content {
        CarryFate::Restore
    } else {
        CarryFate::ToClipboard
    }
}

#[cfg(test)]
mod tests {
    use super::{CarriedDraft, CarryFate, ReconnectCause, carried_fate};
    use crate::{
        model::{
            clipboard::Clipboard,
            library::{PresetRef, SlotEntry},
            preset::Preset,
            slot::Slot,
        },
        protocol::hid::{area::PresetArea, uuid::PresetUuid, version::Version},
        session::StoredPreset,
    };

    fn at(slot: &str) -> PresetRef {
        PresetRef {
            area: PresetArea::FIRST_USER,
            slot: slot.parse().expect("slot"),
        }
    }

    fn entry(seed: u8) -> SlotEntry {
        SlotEntry::new(
            Some("LEAD".to_owned()),
            Some(PresetUuid::from_bytes([seed; 28])),
        )
    }

    fn carried(identity: Option<SlotEntry>) -> CarriedDraft {
        CarriedDraft {
            preset_ref: at("02A"),
            stored: StoredPreset {
                identity,
                landing_scene: Slot::default(),
            },
            draft: Preset::default(),
            is_new: false,
            has_edits: true,
            scene: Slot::default(),
            clipboard: Clipboard::default(),
        }
    }

    #[test]
    fn edits_come_back_only_onto_the_same_unchanged_slot() {
        let edited = carried(Some(entry(1)));
        assert_eq!(
            carried_fate(&edited, at("02A"), Some(&entry(1))),
            CarryFate::Restore
        );
        assert_eq!(
            carried_fate(&edited, at("02B"), Some(&entry(1))),
            CarryFate::ToClipboard
        );
        assert_eq!(
            carried_fate(&edited, at("02A"), Some(&entry(2))),
            CarryFate::ToClipboard
        );
        assert_eq!(
            carried_fate(&carried(None), at("02A"), None),
            CarryFate::Restore
        );
        assert_eq!(
            carried_fate(&carried(None), at("02A"), Some(&entry(1))),
            CarryFate::ToClipboard
        );
    }

    #[test]
    fn a_firmware_install_reports_the_version_it_finds() {
        let expected: Version = "1.19.0.3".parse().expect("version");
        let cause = ReconnectCause::FirmwareInstall {
            expected: Some(expected.clone()),
        };
        assert_eq!(
            cause
                .arrival_notice(&expected)
                .map(|(_, is_error)| is_error),
            Some(false)
        );
        let older: Version = "1.18.0.18".parse().expect("version");
        assert_eq!(
            cause.arrival_notice(&older).map(|(_, is_error)| is_error),
            Some(true)
        );
        assert_eq!(ReconnectCause::Unplugged.arrival_notice(&older), None);
    }
}
