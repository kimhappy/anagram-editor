use std::sync::OnceLock;

use serde::Deserialize;

use super::binding::{
    AMP_MODEL_SYMBOL, BASS_CABINET_SYMBOL, GUITAR_CABINET_SYMBOL, PEDAL_MODEL_SYMBOL,
};
use crate::protocol::hid::files::{
    FACTORY_CABINETS_ROOT, FACTORY_NEURAL_MODELS_ROOT, UserDir, UserFile,
};

const FACTORY_CABINETS: &str = include_str!("../../assets/factory-cabinets.json");
const FACTORY_NEURAL_MODELS: &str = include_str!("../../assets/factory-neural-models.json");

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileChoice {
    pub label: String,
    pub device_path: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileKind {
    BassCabinet,
    GuitarCabinet,
    UserCabinet,
    NeuralAmp,
    NeuralPedal,
    NeuralMisc,
}

impl FileKind {
    #[must_use]
    pub fn for_block(uri: &str) -> Option<Self> {
        match uri {
            "urn:darkglass-anagram:cabinet-bass" => Some(Self::BassCabinet),
            "urn:darkglass-anagram:cabinet-guitar" => Some(Self::GuitarCabinet),
            "urn:darkglass-anagram:cabinet-user" => Some(Self::UserCabinet),
            "urn:darkglass:neural:amp" => Some(Self::NeuralAmp),
            "urn:darkglass:neural:pedal" => Some(Self::NeuralPedal),
            _ if uri.starts_with("urn:darkglass:neural") => Some(Self::NeuralMisc),
            _ => None,
        }
    }

    #[must_use]
    pub const fn quickpot_symbol(self) -> Option<&'static str> {
        match self {
            Self::BassCabinet => Some(BASS_CABINET_SYMBOL),
            Self::GuitarCabinet => Some(GUITAR_CABINET_SYMBOL),
            Self::NeuralAmp => Some(AMP_MODEL_SYMBOL),
            Self::NeuralPedal => Some(PEDAL_MODEL_SYMBOL),
            Self::UserCabinet | Self::NeuralMisc => None,
        }
    }

    #[must_use]
    pub const fn user_dir(self) -> UserDir {
        match self {
            Self::BassCabinet | Self::GuitarCabinet | Self::UserCabinet => UserDir::Cabinets,
            Self::NeuralAmp | Self::NeuralPedal | Self::NeuralMisc => UserDir::NeuralModels,
        }
    }

    #[must_use]
    pub fn choices(self, user_files: &[UserFile]) -> Vec<FileChoice> {
        let factory = match self {
            Self::BassCabinet => factory_cabinets(&cabinets().bass),
            Self::GuitarCabinet => factory_cabinets(&cabinets().guitar),
            Self::UserCabinet => Vec::new(),
            Self::NeuralAmp => factory_models(&neural_models().groups.amp),
            Self::NeuralPedal => factory_models(&neural_models().groups.pedal),
            Self::NeuralMisc => factory_models(&neural_models().all),
        };
        let user = user_files
            .iter()
            .filter(|file| file.dir_name == self.user_dir().name())
            .map(|file| FileChoice {
                label: if file.name.is_empty() {
                    file.file_name.clone()
                } else {
                    file.name.clone()
                },
                device_path: file.device_path(),
            });
        factory.into_iter().chain(user).collect()
    }
}

#[must_use]
pub fn describe_path(path: &str) -> String {
    let file_name = path.rsplit('/').next().unwrap_or(path);
    let stem = file_name
        .rsplit_once('.')
        .map_or(file_name, |(stem, _)| stem);
    cabinets()
        .bass
        .iter()
        .chain(&cabinets().guitar)
        .find(|ir| ir.file == stem)
        .map(cabinet_label)
        .or_else(|| {
            cabinets()
                .artist
                .iter()
                .find(|ir| ir.file == stem)
                .map(|ir| ir.description.clone())
        })
        .unwrap_or_else(|| stem.to_owned())
}

#[derive(Default, Deserialize)]
struct CabinetCatalog {
    #[serde(default)]
    bass: Vec<CabinetIr>,
    #[serde(default)]
    guitar: Vec<CabinetIr>,
    #[serde(default)]
    artist: Vec<ArtistIr>,
}

#[derive(Deserialize)]
struct CabinetIr {
    cabinet: String,
    mic: String,
    position: String,
    file: String,
}

#[derive(Deserialize)]
struct ArtistIr {
    description: String,
    file: String,
}

#[derive(Default, Deserialize)]
struct NeuralCatalog {
    #[serde(default)]
    all: Vec<String>,
    #[serde(default)]
    groups: NeuralGroups,
}

#[derive(Default, Deserialize)]
struct NeuralGroups {
    #[serde(default)]
    amp: Vec<String>,
    #[serde(default)]
    pedal: Vec<String>,
}

fn cabinets() -> &'static CabinetCatalog {
    static CATALOG: OnceLock<CabinetCatalog> = OnceLock::new();
    CATALOG.get_or_init(|| serde_json::from_str(FACTORY_CABINETS).unwrap_or_default())
}

fn neural_models() -> &'static NeuralCatalog {
    static CATALOG: OnceLock<NeuralCatalog> = OnceLock::new();
    CATALOG.get_or_init(|| serde_json::from_str(FACTORY_NEURAL_MODELS).unwrap_or_default())
}

fn cabinet_label(ir: &CabinetIr) -> String {
    format!("{} · {} · {}", ir.cabinet, ir.mic, ir.position)
}

fn factory_cabinets(irs: &[CabinetIr]) -> Vec<FileChoice> {
    irs.iter()
        .map(|ir| FileChoice {
            label: cabinet_label(ir),
            device_path: format!("{FACTORY_CABINETS_ROOT}/{}.dat", ir.file),
        })
        .collect()
}

fn factory_models(names: &[String]) -> Vec<FileChoice> {
    names
        .iter()
        .map(|name| FileChoice {
            label: name.clone(),
            device_path: format!("{FACTORY_NEURAL_MODELS_ROOT}/{name}.dat"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{FileKind, describe_path};

    #[test]
    fn factory_catalogs_are_bundled() {
        assert_eq!(FileKind::BassCabinet.choices(&[]).len(), 924);
        assert_eq!(FileKind::GuitarCabinet.choices(&[]).len(), 108);
        assert_eq!(FileKind::NeuralAmp.choices(&[]).len(), 9);
        assert!(FileKind::NeuralMisc.choices(&[]).len() >= 60);
        assert_eq!(
            describe_path("/factory-data/cabinets/Markb410_103_1_90-MP.dat"),
            "Black & Yellow 4x10 · Condenser 103 · Cap On Axis"
        );
        assert_eq!(describe_path("/data/user-files/cabinets/A2.wav"), "A2");
        assert_eq!(
            FileKind::for_block("urn:darkglass:neural:pedal"),
            Some(FileKind::NeuralPedal)
        );
    }
}
