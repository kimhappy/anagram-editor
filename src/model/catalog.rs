use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use super::{
    category::Category,
    unit::{self, Reading, Unit},
};
use crate::protocol::hid::{
    plugin::{Designation, PluginInfo, PortInfo, PropertyInfo, port_flags, property_flags},
    preset::BlockDoc,
};

pub const SPLIT_URI: &str = "urn:darkglass-anagram:split";
pub const MERGE_URI: &str = "urn:darkglass-anagram:merge";
pub const SEND_URI: &str = "urn:darkglass-anagram:send";
pub const RETURN_URI: &str = "urn:darkglass-anagram:return";
pub const FX_LOOP_URI: &str = "urn:darkglass-anagram:fxloop";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoutingRole {
    Split,
    Merge,
    Send,
    Return,
    FxLoop,
}

impl RoutingRole {
    #[must_use]
    pub fn of(uri: &str) -> Option<Self> {
        match uri {
            SPLIT_URI => Some(Self::Split),
            MERGE_URI => Some(Self::Merge),
            SEND_URI => Some(Self::Send),
            RETURN_URI => Some(Self::Return),
            FX_LOOP_URI => Some(Self::FxLoop),
            _ => None,
        }
    }
}

#[must_use]
pub const fn category_group(raw: u8) -> Category {
    match raw {
        5 | 30 => Category::AmpCab,
        2 | 3 => Category::Drive,
        4..=19 => Category::Dynamics,
        25..=28 | 32 | 33 => Category::Modulation,
        1 | 29 => Category::Ambience,
        _ => Category::Utility,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScalePoint {
    pub label: String,
    pub value: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ParamKind {
    Continuous { logarithmic: bool },
    Integer,
    Toggle,
    Enum(Vec<ScalePoint>),
    Opaque,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParamSpec {
    pub symbol: String,
    pub name: String,
    pub short: String,
    pub flags: u32,
    pub designation: Designation,
    pub kind: ParamKind,
    pub default: f64,
    pub min: f64,
    pub max: f64,
    pub unit: String,
    pub decimals: Option<usize>,
}

const ONE_DECIMAL_UNIT: &str = "1dPt";

impl ParamSpec {
    #[must_use]
    pub fn from_port(port: &PortInfo) -> Self {
        let (unit, decimals) = if port.unit.trim().eq_ignore_ascii_case(ONE_DECIMAL_UNIT) {
            (String::new(), Some(1))
        } else {
            (port.unit.clone(), None)
        };
        let kind = if port.has(port_flags::ENUMERATED) {
            ParamKind::Enum(
                port.scale_points
                    .iter()
                    .map(|point| ScalePoint {
                        label: point.label.clone(),
                        value: point.value,
                    })
                    .collect(),
            )
        } else if port.has(port_flags::TOGGLED) {
            ParamKind::Toggle
        } else if port.has(port_flags::INTEGER) {
            ParamKind::Integer
        } else {
            ParamKind::Continuous {
                logarithmic: port.has(port_flags::LOGARITHMIC),
            }
        };
        let (min, max) = if port.max > port.min {
            (port.min, port.max)
        } else {
            #[expect(
                clippy::float_arithmetic,
                reason = "a degenerate range gets a unit width"
            )]
            let max = port.min + 1.0;
            (port.min, max)
        };
        Self {
            symbol: port.symbol.clone(),
            name: if port.name.is_empty() {
                port.symbol.clone()
            } else {
                port.name.clone()
            },
            short: port.shortname.clone(),
            flags: port.flags,
            designation: port.designation(),
            kind,
            default: port.def.clamp(min, max),
            min,
            max,
            unit,
            decimals,
        }
    }

    #[must_use]
    pub fn opaque(symbol: &str, name: Option<&str>, value: f64) -> Self {
        Self {
            symbol: symbol.to_owned(),
            name: name.unwrap_or(symbol).to_owned(),
            short: String::new(),
            flags: port_flags::CONTROL,
            designation: Designation::None,
            kind: ParamKind::Opaque,
            default: value,
            min: value,
            max: value,
            unit: String::new(),
            decimals: None,
        }
    }

    #[must_use]
    pub const fn clamp(&self, value: f64) -> f64 {
        match self.kind {
            ParamKind::Opaque => value,
            ParamKind::Integer => value.clamp(self.min, self.max).round(),
            ParamKind::Continuous { .. } | ParamKind::Toggle | ParamKind::Enum(_) => {
                value.clamp(self.min, self.max)
            }
        }
    }

    #[must_use]
    pub const fn allowed_in_scenes(&self) -> bool {
        use port_flags::{EXPENSIVE, MAY_UPDATE_BLOCKED_STATE, OUTPUT};
        self.flags & (OUTPUT | EXPENSIVE | MAY_UPDATE_BLOCKED_STATE) == 0
    }

    #[must_use]
    pub const fn suits_expression_pedal(&self) -> bool {
        self.flags & port_flags::EXPRESSION_PEDAL != 0
    }

    #[must_use]
    pub const fn allowed_in_bindings(&self) -> bool {
        use port_flags::{MAY_UPDATE_BLOCKED_STATE, OUTPUT};
        self.flags & (OUTPUT | MAY_UPDATE_BLOCKED_STATE) == 0
    }

    #[must_use]
    pub const fn allowed_in_quickpot(&self) -> bool {
        use port_flags::{HIDDEN, MAY_UPDATE_BLOCKED_STATE, OUTPUT};
        self.flags & (OUTPUT | HIDDEN | MAY_UPDATE_BLOCKED_STATE) == 0
    }

    #[must_use]
    pub const fn should_save(&self) -> bool {
        use port_flags::{HIDDEN, OUTPUT, SAVED_TO_PRESET};
        if self.flags & OUTPUT != 0 {
            return false;
        }
        self.flags & (HIDDEN | SAVED_TO_PRESET) != HIDDEN
    }

    #[must_use]
    pub fn is_on(&self, value: f64) -> bool {
        value > self.min
    }

    #[must_use]
    pub const fn toggle_value(&self, on: bool) -> f64 {
        if on { self.max } else { self.min }
    }

    #[must_use]
    #[expect(clippy::float_arithmetic, reason = "distance to each scale point")]
    pub fn enum_index(&self, value: f64) -> Option<usize> {
        let ParamKind::Enum(points) = &self.kind else {
            return None;
        };
        points
            .iter()
            .enumerate()
            .min_by(|(_, left), (_, right)| {
                (left.value - value)
                    .abs()
                    .total_cmp(&(right.value - value).abs())
            })
            .map(|(index, _)| index)
    }

    #[must_use]
    pub fn enum_value(&self, index: usize) -> Option<f64> {
        self.scale_points().get(index).map(|point| point.value)
    }

    #[must_use]
    pub fn enum_labels(&self) -> Vec<String> {
        self.scale_points()
            .iter()
            .map(|point| point.label.clone())
            .collect()
    }

    fn scale_points(&self) -> &[ScalePoint] {
        match &self.kind {
            ParamKind::Enum(points) => points,
            ParamKind::Continuous { .. }
            | ParamKind::Integer
            | ParamKind::Toggle
            | ParamKind::Opaque => &[],
        }
    }

    #[must_use]
    #[expect(clippy::float_arithmetic, reason = "a share of the range")]
    pub fn step(&self) -> f64 {
        match self.kind {
            ParamKind::Integer | ParamKind::Toggle => 1.0,
            ParamKind::Continuous { .. } | ParamKind::Enum(_) | ParamKind::Opaque => {
                ((self.max - self.min) / 100.0).max(f64::EPSILON)
            }
        }
    }

    #[must_use]
    pub const fn is_logarithmic(&self) -> bool {
        matches!(self.kind, ParamKind::Continuous { logarithmic: true })
    }

    #[must_use]
    pub fn format(&self, value: f64) -> String {
        match &self.kind {
            ParamKind::Enum(_) => self
                .enum_index(value)
                .and_then(|index| self.enum_labels().get(index).cloned())
                .unwrap_or_else(|| format!("{value}")),
            ParamKind::Toggle => if self.is_on(value) { "On" } else { "Off" }.to_owned(),
            ParamKind::Integer | ParamKind::Continuous { .. } | ParamKind::Opaque => {
                let Reading { number, unit } = self.reading(value);
                format!("{number} {unit}").trim_end().to_owned()
            }
        }
    }

    #[must_use]
    pub fn reading(&self, value: f64) -> Reading {
        unit::reading(&Unit::parse(&self.unit), value, self.precision())
    }

    #[must_use]
    pub fn parse_typed(&self, typed: &str, shown: f64) -> Option<f64> {
        unit::parse_typed(&Unit::parse(&self.unit), typed, shown)
    }

    #[must_use]
    pub fn precision(&self) -> usize {
        match &self.kind {
            ParamKind::Integer | ParamKind::Toggle | ParamKind::Enum(_) => 0,
            ParamKind::Continuous { .. } | ParamKind::Opaque => {
                self.decimals.unwrap_or_else(|| {
                    if self.unit.eq_ignore_ascii_case("db") || self.step() >= 0.1 {
                        1
                    } else {
                        2
                    }
                })
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PropertySpec {
    pub uri: String,
    pub name: String,
    pub short: String,
    pub flags: u32,
    pub default_path: String,
}

impl PropertySpec {
    #[must_use]
    pub fn from_info(info: &PropertyInfo) -> Self {
        Self {
            uri: info.uri.clone(),
            name: if info.name.is_empty() {
                info.uri.clone()
            } else {
                info.name.clone()
            },
            short: info.shortname.clone(),
            flags: info.flags,
            default_path: info.defpath.clone(),
        }
    }

    #[must_use]
    pub const fn is_read_only(&self) -> bool {
        self.flags & property_flags::READ_ONLY != 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelOrigin {
    Device,
    Document,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BlockModel {
    pub uri: String,
    pub name: String,
    pub abbreviation: String,
    pub category: Category,
    pub flags: u32,
    pub has_audio: bool,
    pub params: Vec<ParamSpec>,
    pub properties: Vec<PropertySpec>,
    pub origin: ModelOrigin,
}

impl BlockModel {
    #[must_use]
    pub fn from_plugin(info: &PluginInfo) -> Self {
        Self {
            uri: info.uri.clone(),
            name: if info.name.is_empty() {
                info.uri.clone()
            } else {
                info.name.clone()
            },
            abbreviation: info.abbreviation.clone(),
            category: category_group(info.category),
            flags: info.flags,
            has_audio: info.ports.iter().any(|port| port.has(port_flags::AUDIO)),
            params: info
                .ports
                .iter()
                .filter(|port| port.is_user_control())
                .map(ParamSpec::from_port)
                .collect(),
            properties: info
                .properties
                .iter()
                .filter(|property| !property.is_read_only())
                .map(PropertySpec::from_info)
                .collect(),
            origin: ModelOrigin::Device,
        }
    }

    #[must_use]
    pub fn from_document(uri: &str, doc: Option<&BlockDoc>) -> Self {
        let params = doc
            .map_or_else(Vec::new, |doc| sorted_by_key(&doc.parameters))
            .into_iter()
            .map(|param| ParamSpec::opaque(&param.symbol, param.name.as_deref(), param.value))
            .collect();
        let properties = doc
            .map_or_else(Vec::new, |doc| sorted_by_key(&doc.properties))
            .into_iter()
            .map(|property| PropertySpec {
                uri: property.uri.clone(),
                name: property
                    .name
                    .clone()
                    .unwrap_or_else(|| property.uri.clone()),
                short: String::new(),
                flags: property_flags::IS_PATH,
                default_path: property.value.clone(),
            })
            .collect();
        Self {
            uri: uri.to_owned(),
            name: uri.rsplit(':').next().unwrap_or(uri).to_owned(),
            abbreviation: String::new(),
            category: Category::Utility,
            flags: 0,
            has_audio: true,
            params,
            properties,
            origin: ModelOrigin::Document,
        }
    }

    #[must_use]
    pub fn short(&self) -> String {
        if !self.abbreviation.is_empty() {
            return self.abbreviation.clone();
        }
        let initials: String = self
            .name
            .split_whitespace()
            .filter_map(|word| word.chars().next())
            .take(3)
            .collect();
        if initials.len() >= 2 {
            initials.to_uppercase()
        } else {
            self.name.chars().take(3).collect::<String>().to_uppercase()
        }
    }

    #[must_use]
    pub fn default_quickpot(&self) -> Option<String> {
        self.params
            .iter()
            .find(|spec| spec.designation == Designation::QuickPot)
            .map(|spec| spec.symbol.clone())
    }

    #[must_use]
    pub fn param_index(&self, symbol: &str) -> Option<usize> {
        self.params.iter().position(|spec| spec.symbol == symbol)
    }

    #[must_use]
    pub fn clamped(&self, symbol: &str, value: f64) -> Option<(usize, f64)> {
        let index = self.param_index(symbol)?;
        Some((index, self.params.get(index)?.clamp(value)))
    }

    #[must_use]
    pub fn property_index(&self, uri: &str) -> Option<usize> {
        self.properties.iter().position(|spec| spec.uri == uri)
    }

    #[must_use]
    pub fn is_unknown(&self) -> bool {
        self.origin == ModelOrigin::Document
    }

    #[must_use]
    pub fn expression_param(&self) -> Option<&ParamSpec> {
        self.params
            .iter()
            .find(|spec| spec.suits_expression_pedal() && spec.allowed_in_bindings())
    }

    #[must_use]
    pub fn routing_role(&self) -> Option<RoutingRole> {
        RoutingRole::of(&self.uri)
    }

    #[must_use]
    pub fn is_split(&self) -> bool {
        self.uri == SPLIT_URI
    }

    #[must_use]
    pub fn is_merge(&self) -> bool {
        self.uri == MERGE_URI
    }
}

#[must_use]
pub fn sorted_by_key<T>(members: &BTreeMap<String, T>) -> Vec<&T> {
    let mut entries: Vec<(u32, &T)> = members
        .iter()
        .filter_map(|(key, value)| key.parse::<u32>().ok().map(|number| (number, value)))
        .collect();
    entries.sort_by_key(|(number, _)| *number);
    entries.into_iter().map(|(_, value)| value).collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Channels {
    Mono,
    Stereo,
}

impl Channels {
    pub const ALL: [Self; 2] = [Self::Mono, Self::Stereo];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Mono => "Mono",
            Self::Stereo => "Stereo",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Variants {
    pub mono: Arc<BlockModel>,
    pub stereo: Arc<BlockModel>,
}

impl Variants {
    #[must_use]
    pub const fn get(&self, channels: Channels) -> &Arc<BlockModel> {
        match channels {
            Channels::Mono => &self.mono,
            Channels::Stereo => &self.stereo,
        }
    }

    #[must_use]
    pub fn channels_of(&self, uri: &str) -> Option<Channels> {
        if self.mono.uri == uri {
            Some(Channels::Mono)
        } else if self.stereo.uri == uri {
            Some(Channels::Stereo)
        } else {
            None
        }
    }
}

const SYSTEM_INSTANCES: [&str; 1] = ["urn:darkglass:fil4#stereo"];

fn mono_candidates(stereo_uri: &str) -> Vec<String> {
    let base = stereo_uri
        .strip_suffix("Stereo")
        .or_else(|| stereo_uri.strip_suffix("#stereo"));
    base.map_or_else(Vec::new, |base| {
        vec![base.to_owned(), format!("{base}#mono")]
    })
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Catalog {
    models: BTreeMap<String, Arc<BlockModel>>,
    variants: BTreeMap<String, Variants>,
}

impl Catalog {
    pub fn from_plugins<'info>(infos: impl IntoIterator<Item = &'info PluginInfo>) -> Self {
        Self::from_models(infos.into_iter().map(BlockModel::from_plugin))
    }

    pub fn from_models(models: impl IntoIterator<Item = BlockModel>) -> Self {
        let models: BTreeMap<String, Arc<BlockModel>> = models
            .into_iter()
            .filter(|model| !SYSTEM_INSTANCES.contains(&model.uri.as_str()))
            .map(|model| (model.uri.clone(), Arc::new(model)))
            .collect();
        let variants = models
            .values()
            .filter_map(|stereo| {
                let mono = mono_candidates(&stereo.uri)
                    .into_iter()
                    .find_map(|candidate| models.get(&candidate))
                    .filter(|mono| mono.name == stereo.name)?;
                Some(Variants {
                    mono: Arc::clone(mono),
                    stereo: Arc::clone(stereo),
                })
            })
            .flat_map(|pair| {
                [
                    (pair.mono.uri.clone(), pair.clone()),
                    (pair.stereo.uri.clone(), pair),
                ]
            })
            .collect();
        Self { models, variants }
    }

    #[must_use]
    pub fn without(&self, removed: &BTreeSet<String>) -> Self {
        let is_kept = |uri: &String| !removed.contains(uri);
        Self {
            models: self
                .models
                .iter()
                .filter(|(uri, _)| is_kept(uri))
                .map(|(uri, model)| (uri.clone(), Arc::clone(model)))
                .collect(),
            variants: self
                .variants
                .iter()
                .filter(|(uri, pair)| {
                    is_kept(uri) && is_kept(&pair.mono.uri) && is_kept(&pair.stereo.uri)
                })
                .map(|(uri, pair)| (uri.clone(), pair.clone()))
                .collect(),
        }
    }

    #[must_use]
    pub fn find(&self, uri: &str) -> Option<Arc<BlockModel>> {
        self.models.get(uri).cloned()
    }

    #[must_use]
    pub fn variants(&self, uri: &str) -> Option<&Variants> {
        self.variants.get(uri)
    }

    #[must_use]
    pub fn is_same_plugin(&self, uri: &str, other: &str) -> bool {
        uri == other
            || self
                .variants(uri)
                .is_some_and(|pair| pair.channels_of(other).is_some())
    }

    pub fn models(&self) -> impl Iterator<Item = &Arc<BlockModel>> {
        self.models.values()
    }

    #[must_use]
    pub fn is_favourite(&self, uris: &[String], uri: &str) -> bool {
        uris.iter()
            .any(|favourite| self.is_same_plugin(favourite, uri))
    }

    #[must_use]
    pub fn toggled_favourites(&self, uris: &[String], uri: &str) -> Vec<String> {
        if self.is_favourite(uris, uri) {
            uris.iter()
                .filter(|favourite| !self.is_same_plugin(favourite, uri))
                .cloned()
                .collect()
        } else {
            uris.iter()
                .cloned()
                .chain(std::iter::once(uri.to_owned()))
                .collect()
        }
    }

    pub fn favourites_first<'model>(
        &self,
        uris: &[String],
        models: impl Iterator<Item = &'model Arc<BlockModel>>,
    ) -> Vec<Arc<BlockModel>> {
        let (favourites, others): (Vec<_>, Vec<_>) = models
            .cloned()
            .partition(|model| self.is_favourite(uris, &model.uri));
        favourites.into_iter().chain(others).collect()
    }

    pub fn in_category(&self, category: Category) -> impl Iterator<Item = &Arc<BlockModel>> {
        let mut models: Vec<&Arc<BlockModel>> = self
            .models
            .values()
            .filter(move |model| model.category == category && model.has_audio)
            .filter(|model| {
                self.variants(&model.uri)
                    .is_none_or(|pair| pair.channels_of(&model.uri) == Some(Channels::Mono))
            })
            .collect();
        models.sort_by_key(|model| model.name.to_lowercase());
        models.into_iter()
    }

    #[must_use]
    pub fn resolve(&self, uri: &str, doc: Option<&BlockDoc>) -> Arc<BlockModel> {
        self.find(uri)
            .unwrap_or_else(|| Arc::new(BlockModel::from_document(uri, doc)))
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.models.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.models.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::{Catalog, Channels, ParamKind, category_group};
    use crate::{
        model::{
            category::Category,
            testing::{self, CHORUS, CHORUS_STEREO, DRIVE, GAIN},
        },
        protocol::hid::preset::{BlockDoc, ParamDoc},
    };

    #[test]
    fn categories_follow_the_picker_groups() {
        assert_eq!(category_group(6), Category::Dynamics);
        assert_eq!(category_group(30), Category::AmpCab);
        assert_eq!(category_group(33), Category::Modulation);
        assert_eq!(category_group(22), Category::Utility);
        assert_eq!(category_group(29), Category::Ambience);
    }

    #[test]
    fn favourites_match_either_channel_form_and_come_first() {
        let catalog = testing::catalog();
        let uris = [
            testing::CHORUS_STEREO.to_owned(),
            "urn:gone".to_owned(),
            testing::GAIN.to_owned(),
            testing::CHORUS.to_owned(),
        ];
        assert!(catalog.is_favourite(&uris, testing::CHORUS));
        let all: Vec<_> = catalog.models().cloned().collect();
        let first_two: Vec<String> = catalog
            .favourites_first(&uris, all.iter())
            .iter()
            .take(2)
            .map(|model| model.name.clone())
            .collect();
        assert!(
            first_two
                .iter()
                .all(|name| name == "Chorus" || name == "Gain")
        );
        let without_chorus = catalog.toggled_favourites(&uris, testing::CHORUS);
        assert_eq!(
            without_chorus,
            ["urn:gone".to_owned(), testing::GAIN.to_owned()]
        );
        assert_eq!(
            catalog
                .toggled_favourites(&without_chorus, testing::DRIVE)
                .last()
                .map(String::as_str),
            Some(testing::DRIVE)
        );
    }

    #[test]
    fn plugin_info_becomes_a_model_with_user_controls_only() {
        let catalog = testing::catalog();
        let gain = catalog.find(GAIN).expect("gain");
        assert_eq!(gain.params.len(), 1);
        assert_eq!(gain.param_index("gain"), Some(0));
        assert_eq!(gain.params[0].format(-3.0), "-3.0 dB");
        assert_eq!(gain.short(), "GAI");

        let drive = catalog.find(DRIVE).expect("drive");
        let mode = &drive.params[drive.param_index("mode").expect("mode")];
        assert!(matches!(&mode.kind, ParamKind::Enum(points) if points.len() == 2));
        assert_eq!(mode.enum_index(2.0), Some(1));
        assert_eq!(mode.format(2.0), "Hard");
        assert!((drive.params[0].clamp(11.0) - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn mono_and_stereo_forms_pair_up_and_list_once() {
        let catalog = testing::catalog();
        let pair = catalog.variants(CHORUS_STEREO).expect("paired");
        assert_eq!(pair.mono.uri, CHORUS);
        assert_eq!(pair.channels_of(CHORUS_STEREO), Some(Channels::Stereo));
        assert!(catalog.is_same_plugin(CHORUS, CHORUS_STEREO));
        assert!(!catalog.is_same_plugin(CHORUS, DRIVE));
        let listed: Vec<&str> = catalog
            .in_category(Category::Modulation)
            .map(|model| model.uri.as_str())
            .collect();
        assert_eq!(listed, [CHORUS]);
    }

    #[test]
    fn the_global_eq_instance_is_no_stereo_form_of_the_parametric_eq_block() {
        let catalog = Catalog::from_plugins(&[
            testing::plugin("urn:darkglass:fil4#mono", "Parametric EQ", 17, Vec::new()),
            testing::plugin("urn:darkglass:fil4#stereo", "Parametric EQ", 17, Vec::new()),
        ]);
        assert!(catalog.find("urn:darkglass:fil4#mono").is_some());
        assert!(catalog.find("urn:darkglass:fil4#stereo").is_none());
        assert!(catalog.variants("urn:darkglass:fil4#mono").is_none());
    }

    #[test]
    fn the_one_decimal_pseudo_unit_sets_the_precision_and_hides_itself() {
        let catalog = testing::catalog();
        let chorus = catalog.find(CHORUS).expect("chorus");
        let wet = &chorus.params[chorus.param_index("wet").expect("wet")];
        assert_eq!(wet.unit, "");
        assert_eq!(wet.precision(), 1);
        assert_eq!(wet.format(5.0), "5.0");
        let depth = &chorus.params[chorus.param_index("depth").expect("depth")];
        assert_eq!(depth.precision(), 2);
    }

    #[test]
    fn unknown_uris_resolve_to_a_document_model() {
        let catalog = testing::catalog();
        let mut doc = BlockDoc {
            uri: "urn:example:mystery".to_owned(),
            ..BlockDoc::default()
        };
        doc.parameters.insert(
            "1".to_owned(),
            ParamDoc {
                name: Some("Depth".to_owned()),
                symbol: "depth".to_owned(),
                value: 0.4,
                extra: serde_json::Map::new(),
            },
        );
        let model = catalog.resolve("urn:example:mystery", Some(&doc));
        assert!(model.is_unknown());
        assert_eq!(model.params.len(), 1);
        assert_eq!(model.params[0].kind, ParamKind::Opaque);
        assert_eq!(model.short(), "MYS");
    }

    #[test]
    fn removing_a_plugin_drops_its_model_and_its_variant_pair() {
        let catalog = testing::catalog();
        assert!(catalog.variants(CHORUS).is_some());
        let removed = std::collections::BTreeSet::from([CHORUS_STEREO.to_owned()]);
        let pruned = catalog.without(&removed);
        assert!(pruned.find(CHORUS_STEREO).is_none());
        assert!(pruned.find(CHORUS).is_some());
        assert!(pruned.variants(CHORUS).is_none());
        assert!(pruned.find(DRIVE).is_some());
        assert_eq!(pruned.len(), catalog.len() - 1);
    }
}
