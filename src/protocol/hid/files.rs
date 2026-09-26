use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::lenient;

const USER_FILES_ROOT: &str = "/data/user-files";
pub const FACTORY_CABINETS_ROOT: &str = "/factory-data/cabinets";
pub const FACTORY_NEURAL_MODELS_ROOT: &str = "/factory-data/neural-models";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UserDir {
    Cabinets,
    NeuralModels,
}

impl UserDir {
    pub const ALL: [Self; 2] = [Self::Cabinets, Self::NeuralModels];

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Cabinets => "cabinets",
            Self::NeuralModels => "neural-models",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FirmwareReceiver {
    pub file_name: String,
    pub dir_name: String,
    pub is_firmware: bool,
}

impl FirmwareReceiver {
    #[must_use]
    pub fn new(file_name: &str) -> Self {
        let base = file_name.rsplit(['/', '\\']).next().unwrap_or(file_name);
        Self {
            file_name: base.split_whitespace().collect::<Vec<_>>().join("_"),
            dir_name: String::new(),
            is_firmware: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserFile {
    #[serde(deserialize_with = "lenient::int")]
    pub id: u64,
    #[serde(default)]
    pub name: String,
    pub file_name: String,
    pub dir_name: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "lenient::opt_int"
    )]
    pub file_size: Option<u64>,
    #[serde(default)]
    pub uris: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_file_name: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl UserFile {
    #[must_use]
    pub fn device_path(&self) -> String {
        format!("{USER_FILES_ROOT}/{}/{}", self.dir_name, self.file_name)
    }

    #[must_use]
    pub fn dir(&self) -> Option<UserDir> {
        UserDir::ALL
            .into_iter()
            .find(|dir| dir.name() == self.dir_name)
    }

    #[must_use]
    pub fn renamed(&self, name: &str, timestamp: &str) -> Self {
        let mut record = self.clone();
        name.clone_into(&mut record.name);
        record
            .extra
            .insert("updated_at".to_owned(), Value::String(timestamp.to_owned()));
        record
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadRecord {
    pub file_name: String,
    pub file_size: u64,
    pub name: String,
    pub dir_name: String,
    pub category: String,
    pub tags: Vec<String>,
    pub uris: Vec<String>,
    pub gain: f64,
    pub created_at: String,
    pub updated_at: String,
    pub is_firmware: bool,
}

impl UploadRecord {
    #[must_use]
    pub fn new(dir: UserDir, file_name: &str, name: &str, file_size: u64, timestamp: &str) -> Self {
        Self {
            file_name: file_name.to_owned(),
            file_size,
            name: name.to_owned(),
            dir_name: dir.name().to_owned(),
            category: String::new(),
            tags: Vec::new(),
            uris: Vec::new(),
            gain: 0.0,
            created_at: timestamp.to_owned(),
            updated_at: timestamp.to_owned(),
            is_firmware: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{UserDir, UserFile};

    #[test]
    fn user_paths_keep_the_extension() {
        let record: UserFile = serde_json::from_str(
            r#"{"id":1.0,"name":"Impulse 1","file_name":"A1.wav","dir_name":"cabinets","file_size":10000,"tags":[],"uris":[],"gain":0}"#,
        )
        .expect("parses");
        assert_eq!(record.device_path(), "/data/user-files/cabinets/A1.wav");
        assert_eq!(record.id, 1);
        assert_eq!(record.dir(), Some(UserDir::Cabinets));
        let renamed = record.renamed("Impulse 2", "2026-09-24T00:00:00.000Z");
        assert_eq!(renamed.name, "Impulse 2");
        assert_eq!(
            renamed
                .extra
                .get("updated_at")
                .and_then(|value| value.as_str()),
            Some("2026-09-24T00:00:00.000Z")
        );
    }
}
