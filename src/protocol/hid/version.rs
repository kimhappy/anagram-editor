use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub text: String,
}

impl Version {
    pub const MINIMUM_SUPPORTED: (u32, u32) = (1, 18);

    #[must_use]
    pub fn is_supported(&self) -> bool {
        (self.major, self.minor) >= Self::MINIMUM_SUPPORTED
    }

    #[must_use]
    pub fn is_newer_than(&self, other: &Self) -> bool {
        numbers(&self.text) > numbers(&other.text)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseVersionError;

impl FromStr for Version {
    type Err = ParseVersionError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let cleaned: String = text
            .trim()
            .chars()
            .filter(|character| *character != 'v' && !character.is_whitespace())
            .collect();
        let mut parts = cleaned.split('.').map(|part| part.parse::<u32>().ok());
        let major = parts.next().flatten().ok_or(ParseVersionError)?;
        let minor = parts.next().flatten().ok_or(ParseVersionError)?;
        let patch = parts.next().flatten().unwrap_or(0);
        Ok(Self {
            major,
            minor,
            patch,
            text: cleaned,
        })
    }
}

fn numbers(text: &str) -> Vec<u32> {
    text.split('.')
        .map(|part| part.parse::<u32>().unwrap_or(0))
        .collect()
}

impl fmt::Display for Version {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.text)
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        text.parse()
            .map_err(|_unparsed| serde::de::Error::custom(format!("not a version: {text:?}")))
    }
}

#[cfg(test)]
mod tests {
    use super::Version;

    #[test]
    fn parses_the_device_form() {
        let version: Version = "v1.18.0.18\n".parse().expect("parses");
        assert_eq!((version.major, version.minor, version.patch), (1, 18, 0));
        assert_eq!(version.to_string(), "1.18.0.18");
        assert!(version.is_supported());
        let old: Version = "v1.17.3.36".parse().expect("parses");
        assert!(!old.is_supported());
        assert_eq!("firmware".parse::<Version>().ok(), None);
    }
}
