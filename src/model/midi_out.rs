use std::collections::BTreeMap;

use serde_json::Map;

use super::slot::Slot;
use crate::protocol::hid::preset::{
    MIDI_OUT_CONTROL_CHANGE, MIDI_OUT_PROGRAM_CHANGE, MetadataDoc, MidiOutDoc,
};

const MIDI_DATA_MAX: u8 = 0x7F;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MidiOutValue {
    SlotNumber,
    Fixed(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MidiOutMessage {
    ControlChange {
        channel: u8,
        controller: u8,
        value: MidiOutValue,
    },
    ProgramChange {
        channel: u8,
        program: MidiOutValue,
    },
}

impl MidiOutMessage {
    #[must_use]
    pub const fn channel(self) -> u8 {
        match self {
            Self::ControlChange { channel, .. } | Self::ProgramChange { channel, .. } => channel,
        }
    }

    #[must_use]
    pub fn from_doc(doc: &MidiOutDoc) -> Option<Self> {
        let channel = doc.channel.clamp(1, 16);
        let literal = |field: Option<u8>| {
            if doc.slot_number == Some(true) {
                Some(MidiOutValue::SlotNumber)
            } else {
                field.map(|number| MidiOutValue::Fixed(number.min(MIDI_DATA_MAX)))
            }
        };
        match doc.kind {
            MIDI_OUT_CONTROL_CHANGE => Some(Self::ControlChange {
                channel,
                controller: doc.number?.min(MIDI_DATA_MAX),
                value: literal(doc.value)?,
            }),
            MIDI_OUT_PROGRAM_CHANGE => Some(Self::ProgramChange {
                channel,
                program: literal(doc.number)?,
            }),
            _ => None,
        }
    }

    #[must_use]
    pub fn to_doc(self) -> MidiOutDoc {
        let base = MidiOutDoc {
            channel: self.channel(),
            ..MidiOutDoc::default()
        };
        match self {
            Self::ControlChange {
                controller, value, ..
            } => MidiOutDoc {
                kind: MIDI_OUT_CONTROL_CHANGE,
                number: Some(controller),
                value: value.fixed(),
                slot_number: value.slot_number(),
                ..base
            },
            Self::ProgramChange { program, .. } => MidiOutDoc {
                kind: MIDI_OUT_PROGRAM_CHANGE,
                number: program.fixed(),
                slot_number: program.slot_number(),
                ..base
            },
        }
    }
}

impl MidiOutValue {
    const fn fixed(self) -> Option<u8> {
        match self {
            Self::Fixed(fixed) => Some(fixed),
            Self::SlotNumber => None,
        }
    }

    const fn slot_number(self) -> Option<bool> {
        match self {
            Self::SlotNumber => Some(true),
            Self::Fixed(_) => None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MidiOut {
    pub on_preset: Vec<MidiOutMessage>,
    pub on_scene: BTreeMap<Slot, Vec<MidiOutMessage>>,
    pub extra: Map<String, serde_json::Value>,
}

impl MidiOut {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.on_preset.is_empty() && self.on_scene.is_empty() && self.extra.is_empty()
    }

    #[must_use]
    pub fn from_doc(doc: &MetadataDoc) -> (Self, usize) {
        let convert = |messages: &[MidiOutDoc]| -> Vec<MidiOutMessage> {
            messages
                .iter()
                .filter_map(MidiOutMessage::from_doc)
                .collect()
        };
        let on_preset = convert(&doc.on_preset_change);
        let chosen = doc
            .on_scene_change
            .iter()
            .filter_map(|(key, messages)| {
                let index = key.parse::<usize>().ok()?;
                let is_canonical = *key == index.to_string();
                Some((Slot::from_index(index)?, (is_canonical, messages)))
            })
            .fold(
                BTreeMap::<Slot, (bool, &Vec<MidiOutDoc>)>::new(),
                |mut chosen, (scene, candidate)| {
                    let is_better = chosen
                        .get(&scene)
                        .is_none_or(|(is_canonical, _)| !is_canonical && candidate.0);
                    if is_better {
                        chosen.insert(scene, candidate);
                    }
                    chosen
                },
            );
        let placed: Vec<(Slot, Vec<MidiOutMessage>)> = chosen
            .into_iter()
            .map(|(scene, (_, messages))| (scene, convert(messages)))
            .collect();
        let total =
            doc.on_preset_change.len() + doc.on_scene_change.values().map(Vec::len).sum::<usize>();
        let kept = on_preset.len()
            + placed
                .iter()
                .map(|(_, messages)| messages.len())
                .sum::<usize>();
        let on_scene = placed
            .into_iter()
            .filter(|(_, messages)| !messages.is_empty())
            .collect();
        (
            Self {
                on_preset,
                on_scene,
                extra: doc.extra.clone(),
            },
            total - kept,
        )
    }

    #[must_use]
    pub fn to_doc(&self) -> MetadataDoc {
        MetadataDoc {
            on_preset_change: self
                .on_preset
                .iter()
                .map(|message| message.to_doc())
                .collect(),
            on_scene_change: self
                .on_scene
                .iter()
                .filter(|(_, messages)| !messages.is_empty())
                .map(|(scene, messages)| {
                    (
                        scene.index().to_string(),
                        messages.iter().map(|message| message.to_doc()).collect(),
                    )
                })
                .collect(),
            extra: self.extra.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MidiOut, MidiOutMessage, MidiOutValue};
    use crate::{model::slot::Slot, protocol::hid::preset::MetadataDoc};

    #[test]
    fn both_message_shapes_round_trip() {
        let doc: MetadataDoc = serde_json::from_str(
            r#"{"midi_messages_on_preset_change":[{"channel":1,"slotNumber":true,"type":1},{"channel":2,"number":5,"type":1}],
                "midi_messages_on_scene_change":{"0":[{"channel":1,"number":3,"slotNumber":true,"type":0}],"2":[{"channel":1,"number":0,"type":0,"value":0}]}}"#,
        )
        .expect("parses");
        let (out, dropped) = MidiOut::from_doc(&doc);
        assert_eq!(dropped, 0);
        assert_eq!(
            out.on_preset,
            [
                MidiOutMessage::ProgramChange {
                    channel: 1,
                    program: MidiOutValue::SlotNumber
                },
                MidiOutMessage::ProgramChange {
                    channel: 2,
                    program: MidiOutValue::Fixed(5)
                },
            ]
        );
        assert_eq!(
            out.on_scene
                .get(&Slot::default())
                .and_then(|messages| messages.first()),
            Some(&MidiOutMessage::ControlChange {
                channel: 1,
                controller: 3,
                value: MidiOutValue::SlotNumber
            })
        );
        assert!(
            out.on_scene
                .contains_key(&Slot::from_index(2).expect("slot"))
        );
        assert_eq!(out.to_doc(), doc);
    }

    #[test]
    fn out_of_range_numbers_clamp_and_duplicate_scene_keys_count_as_dropped() {
        let doc: MetadataDoc = serde_json::from_str(
            r#"{"midi_messages_on_preset_change":[{"channel":1,"number":128,"value":200,"type":0}],
                "midi_messages_on_scene_change":{"01":[{"channel":1,"number":1,"type":1}],"1":[{"channel":1,"number":2,"type":1}],"003":[{"channel":1,"number":3,"type":1}]}}"#,
        )
        .expect("parses");
        let (out, dropped) = MidiOut::from_doc(&doc);
        assert_eq!(
            out.on_preset,
            [MidiOutMessage::ControlChange {
                channel: 1,
                controller: 127,
                value: MidiOutValue::Fixed(127)
            }]
        );
        let program = |number| {
            Some(vec![MidiOutMessage::ProgramChange {
                channel: 1,
                program: MidiOutValue::Fixed(number),
            }])
        };
        assert_eq!(
            out.on_scene
                .get(&Slot::from_index(1).expect("slot"))
                .cloned(),
            program(2)
        );
        assert_eq!(
            out.on_scene
                .get(&Slot::from_index(3).expect("slot"))
                .cloned(),
            program(3)
        );
        assert_eq!(dropped, 1);
    }
}
