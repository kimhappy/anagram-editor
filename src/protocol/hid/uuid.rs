use std::fmt::{self, Write};

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

const BYTES: usize = 28;
const LENGTH: usize = BYTES * 2 + 3;
const GROUPS: [usize; 4] = [16, 8, 8, 24];

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct PresetUuid(String);

impl PresetUuid {
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let groups: Vec<&str> = text.split('-').collect();
        let is_shaped = text.len() == LENGTH
            && groups.len() == GROUPS.len()
            && groups
                .iter()
                .zip(GROUPS)
                .all(|(group, length)| group.len() == length)
            && groups
                .iter()
                .all(|group| group.chars().all(|digit| digit.is_ascii_hexdigit()));
        is_shaped.then(|| Self(text.to_ascii_lowercase()))
    }

    #[must_use]
    pub fn from_bytes(mut bytes: [u8; BYTES]) -> Self {
        bytes[6] = 0x40 | (bytes[6] & 0x0F);
        bytes[8] = 0x80 | (bytes[8] & 0x3F);
        let hex = bytes
            .iter()
            .fold(String::with_capacity(BYTES * 2), |mut hex, byte| {
                write!(hex, "{byte:02x}").unwrap_or_default();
                hex
            });
        let groups: Vec<&str> = GROUPS
            .iter()
            .scan(0, |offset, &length| {
                let start = *offset;
                *offset += length;
                Some(hex.get(start..start + length).unwrap_or_default())
            })
            .collect();
        Self(groups.join("-"))
    }

    #[must_use]
    pub fn random() -> Self {
        Self::from_bytes(crate::host::random_bytes())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub fn lenient<'de, D>(deserializer: D) -> Result<Option<PresetUuid>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    Ok(value.as_str().and_then(PresetUuid::parse))
}

impl fmt::Display for PresetUuid {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::PresetUuid;

    #[test]
    fn accepts_the_device_format_only() {
        let text = "197fb05c1ed44dcc-8aaf4f87-d1e07ce0-73d21f2f4ac7e9ad7b2ec02d";
        assert_eq!(
            PresetUuid::parse(text).map(|uuid| uuid.to_string()),
            Some(text.to_owned())
        );
        assert_eq!(
            PresetUuid::parse("123e4567-e89b-12d3-a456-426614174000"),
            None
        );
    }

    #[test]
    fn generated_ids_have_the_shape() {
        let uuid = PresetUuid::from_bytes([0xFF; 28]);
        assert_eq!(uuid.as_str().len(), 59);
        assert!(PresetUuid::parse(uuid.as_str()).is_some());
        assert_eq!(uuid.as_str().get(12..14), Some("4f"));
        assert_eq!(uuid.as_str().get(0..2), Some("ff"));
    }
}
