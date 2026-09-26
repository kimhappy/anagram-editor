use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::lenient;

pub const DOCUMENT_TYPE: &str = "preset";
pub const DOCUMENT_VERSION: u32 = 1;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PresetDocument {
    pub preset: PresetBody,
    #[serde(rename = "type", default = "document_type")]
    pub kind: String,
    #[serde(default = "document_version", deserialize_with = "lenient::int")]
    pub version: u32,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

fn document_type() -> String {
    DOCUMENT_TYPE.to_owned()
}

const fn document_version() -> u32 {
    DOCUMENT_VERSION
}

impl PresetDocument {
    #[must_use]
    pub fn new(preset: PresetBody) -> Self {
        Self {
            preset,
            kind: document_type(),
            version: DOCUMENT_VERSION,
            extra: Map::new(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PresetBody {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "lenient::opt_int"
    )]
    pub scene: Option<u32>,
    #[serde(
        rename = "sceneNames",
        default,
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub scene_names: BTreeMap<String, String>,
    #[serde(default)]
    pub bindings: BTreeMap<String, BindingDoc>,
    #[serde(default)]
    pub chains: BTreeMap<String, ChainDoc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<MetadataDoc>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ChainDoc {
    #[serde(default)]
    pub blocks: BTreeMap<String, BlockDoc>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BlockDoc {
    pub uri: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub parameters: BTreeMap<String, ParamDoc>,
    #[serde(default)]
    pub properties: BTreeMap<String, PropertyDoc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quickpot: Option<String>,
    #[serde(default)]
    pub scenes: BTreeMap<String, SceneDeltaDoc>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ParamDoc {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub symbol: String,
    #[serde(default)]
    pub value: f64,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PropertyDoc {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub uri: String,
    #[serde(default)]
    pub value: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SceneDeltaDoc {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub parameters: Vec<SymbolValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub properties: Option<Vec<Value>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SymbolValue {
    pub symbol: String,
    #[serde(default)]
    pub value: f64,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BindingDoc {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default)]
    pub parameters: Vec<BindingTargetDoc>,
    #[serde(default)]
    pub properties: Vec<Value>,
    #[serde(default)]
    pub value: f64,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BindingTargetDoc {
    #[serde(deserialize_with = "lenient::saturating_int")]
    pub block: u32,
    #[serde(deserialize_with = "lenient::saturating_int")]
    pub row: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    pub symbol: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetadataDoc {
    #[serde(
        rename = "midi_messages_on_preset_change",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub on_preset_change: Vec<MidiOutDoc>,
    #[serde(
        rename = "midi_messages_on_scene_change",
        default,
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub on_scene_change: BTreeMap<String, Vec<MidiOutDoc>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MidiOutDoc {
    #[serde(
        rename = "type",
        default = "unknown_kind",
        deserialize_with = "lenient::saturating_int"
    )]
    pub kind: u8,
    #[serde(default, deserialize_with = "lenient::saturating_int")]
    pub channel: u8,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "lenient::saturating_opt_int"
    )]
    pub number: Option<u8>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "lenient::saturating_opt_int"
    )]
    pub value: Option<u8>,
    #[serde(
        rename = "slotNumber",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub slot_number: Option<bool>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

pub const MIDI_OUT_CONTROL_CHANGE: u8 = 0;
pub const MIDI_OUT_PROGRAM_CHANGE: u8 = 1;

const fn unknown_kind() -> u8 {
    u8::MAX
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::{BindingTargetDoc, MidiOutDoc, PresetDocument};

    #[test]
    fn out_of_range_numbers_saturate_instead_of_failing_the_preset() {
        let message: MidiOutDoc =
            serde_json::from_value(json!({"type": 0, "channel": 300, "number": 7, "value": 300}))
                .expect("parses");
        assert_eq!(
            (message.channel, message.number, message.value),
            (255, Some(7), Some(255))
        );
        let target: BindingTargetDoc =
            serde_json::from_value(json!({"block": -1, "row": 1, "symbol": "gain"}))
                .expect("parses");
        assert_eq!((target.block, target.row), (0, 1));
    }

    const SKELETON: &str = r#"{
      "preset": {
        "bindings": {
          "foot1": { "name": "Split", "parameters": [{ "block": 2, "row": 1, "min": 0, "max": 1, "symbol": ":bypass" }], "properties": [], "value": 1 }
        },
        "chains": {
          "1": { "blocks": { "1": {
            "enabled": true,
            "parameters": { "1": { "name": "Input", "symbol": "Input", "value": 7.2 } },
            "properties": {},
            "quickpot": "Input",
            "scenes": { "2": { "enabled": false, "parameters": [{ "symbol": "Input", "value": 7.7 }] } },
            "uri": "urn:darkglass:FETComp"
          } } }
        },
        "name": "Ephemeris II",
        "scene": 0,
        "sceneNames": { "1": "NEBULA" },
        "background": "space.png",
        "metadata": {
          "midi_messages_on_preset_change": [{ "channel": 1, "slotNumber": true, "type": 1 }],
          "midi_messages_on_scene_change": { "0": [{ "channel": 1, "number": 0, "type": 0, "value": 0 }] }
        },
        "uuid": "197fb05c1ed44dcc-8aaf4f87-d1e07ce0-73d21f2f4ac7e9ad7b2ec02d"
      },
      "type": "preset",
      "version": 1.0
    }"#;

    #[test]
    fn round_trips_semantically() {
        let document: PresetDocument = serde_json::from_str(SKELETON).expect("parses");
        assert_eq!(document.version, 1);
        assert_eq!(
            document.preset.extra.get("background"),
            Some(&json!("space.png"))
        );
        let scene = document
            .preset
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.on_scene_change.get("0"))
            .and_then(|messages| messages.first())
            .expect("scene message");
        assert_eq!(
            (scene.kind, scene.value, scene.slot_number),
            (0, Some(0), None)
        );

        let written = serde_json::to_value(&document).expect("serialises");
        let original: Value = serde_json::from_str(SKELETON).expect("parses");
        assert_eq!(normalised(written), normalised(original));
    }

    #[expect(
        clippy::cast_possible_truncation,
        reason = "only whole floats below 1e15 are narrowed"
    )]
    fn normalised(value: Value) -> Value {
        match value {
            Value::Number(number) => number
                .as_f64()
                .filter(|float| float.fract() == 0.0 && float.abs() < 1e15)
                .map_or(Value::Number(number), |float| json!(float as i64)),
            Value::Array(items) => Value::Array(items.into_iter().map(normalised).collect()),
            Value::Object(members) => Value::Object(
                members
                    .into_iter()
                    .map(|(key, member)| (key, normalised(member)))
                    .collect(),
            ),
            other => other,
        }
    }
}
