use serde_json::{Map, Value};

use super::preset::BlockId;
use crate::protocol::name::PRESET_NAME;

pub const BYPASS_SYMBOL: &str = ":bypass";
pub const BASS_CABINET_SYMBOL: &str = ":bass-cab";
pub const GUITAR_CABINET_SYMBOL: &str = ":guitar-cab";
pub const AMP_MODEL_SYMBOL: &str = ":amp-model";
pub const PEDAL_MODEL_SYMBOL: &str = ":pedal-model";

#[must_use]
pub fn default_binding_name(block_name: &str, param: Option<&str>) -> String {
    PRESET_NAME.sanitize(param.unwrap_or(block_name))
}

#[derive(Clone, Debug, PartialEq)]
pub struct BindingTarget {
    pub block: BlockId,
    pub symbol: String,
    pub min: f64,
    pub max: f64,
    pub extra: Map<String, Value>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Binding {
    pub name: String,
    pub targets: Vec<BindingTarget>,
    pub value: f64,
    pub properties: Vec<Value>,
    pub extra: Map<String, Value>,
}

impl Binding {
    #[must_use]
    pub fn single(name: &str, target: BindingTarget) -> Self {
        Self {
            name: name.to_owned(),
            targets: vec![target],
            value: 0.0,
            properties: Vec::new(),
            extra: Map::new(),
        }
    }

    pub fn retarget(&mut self, target: BindingTarget, old_default: &str, new_default: &str) {
        if self.name.is_empty() || self.name == old_default {
            new_default.clone_into(&mut self.name);
        }
        match self.targets.first_mut() {
            Some(first) if first.block == target.block => {
                let extra = std::mem::take(&mut first.extra);
                *first = BindingTarget { extra, ..target };
            }
            Some(first) => *first = target,
            None => self.targets.push(target),
        }
    }

    #[must_use]
    pub fn target(&self) -> Option<&BindingTarget> {
        self.targets.first()
    }

    pub fn target_mut(&mut self) -> Option<&mut BindingTarget> {
        self.targets.first_mut()
    }

    #[must_use]
    pub fn drives(&self, block: BlockId) -> bool {
        self.targets.iter().any(|target| target.block == block)
    }

    #[must_use]
    pub fn drives_param(&self, block: BlockId, symbol: &str) -> bool {
        self.targets
            .iter()
            .any(|target| target.block == block && target.symbol == symbol)
    }

    pub fn forget(&mut self, block: BlockId) -> bool {
        self.targets.retain(|target| target.block != block);
        self.targets.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Map, json};

    use super::{Binding, BindingTarget, default_binding_name};
    use crate::model::preset::BlockId;

    fn target(block: BlockId, symbol: &str) -> BindingTarget {
        BindingTarget {
            block,
            symbol: symbol.to_owned(),
            min: 0.0,
            max: 1.0,
            extra: Map::new(),
        }
    }

    #[test]
    fn retargeting_keeps_the_other_targets_and_a_custom_name() {
        let fresh = || {
            crate::model::preset::Block::new(std::sync::Arc::new(
                crate::model::catalog::BlockModel::from_document("urn:test:block", None),
            ))
            .id
        };
        let (first, second, third) = (fresh(), fresh(), fresh());
        let mut macro_binding = Binding {
            targets: vec![target(first, "gain"), target(second, "mix")],
            value: 0.4,
            properties: vec![json!({"uri": "urn:x"})],
            ..Binding::single("Gain", target(first, "gain"))
        };
        macro_binding.retarget(target(third, "drive"), "Gain", "Drive");
        assert_eq!(macro_binding.name, "Drive");
        assert_eq!(macro_binding.targets.len(), 2);
        assert_eq!(macro_binding.targets[1].symbol, "mix");
        assert!((macro_binding.value - 0.4).abs() < f64::EPSILON);
        assert_eq!(macro_binding.properties.len(), 1);

        let mut tagged = target(first, "gain");
        tagged.extra.insert("curve".to_owned(), json!("log"));
        let mut custom = Binding::single("Boost", tagged);
        custom.retarget(target(first, "level"), "Gain", "Level");
        assert_eq!(custom.name, "Boost");
        assert_eq!(custom.targets[0].extra.get("curve"), Some(&json!("log")));
    }

    #[test]
    fn a_bypass_binding_is_named_after_its_block() {
        assert_eq!(default_binding_name("Hall Reverb", None), "Hall Reverb");
        assert_eq!(
            default_binding_name("Hall Reverb", Some("Decay time long")),
            "Decay time long"
        );
    }
}
