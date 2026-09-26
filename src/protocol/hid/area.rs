use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PresetArea {
    Factory,
    Alternative,
    Artist,
    User(u8),
}

const FACTORY_SIZE: u8 = 36;

impl PresetArea {
    pub const FIRST_USER: Self = Self::User(1);

    #[must_use]
    pub fn setting_value(self) -> String {
        match self {
            Self::Factory => "factory".to_owned(),
            Self::Alternative => "alternative".to_owned(),
            Self::Artist => "artist".to_owned(),
            Self::User(1) => "user".to_owned(),
            Self::User(number) => format!("user-{number}"),
        }
    }

    #[must_use]
    pub fn from_setting(value: &str) -> Option<Self> {
        match value {
            "factory" => Some(Self::Factory),
            "alternative" => Some(Self::Alternative),
            "artist" => Some(Self::Artist),
            "user" => Some(Self::User(1)),
            _ => value
                .strip_prefix("user-")
                .and_then(|number| number.parse::<u8>().ok())
                .filter(|&number| number >= 2)
                .map(Self::User),
        }
    }

    #[must_use]
    pub fn user_area_payload(self) -> Option<String> {
        match self {
            Self::User(number) if number >= 2 => Some(format!("user-{number}")),
            _ => None,
        }
    }

    #[must_use]
    pub const fn read_only_size(self) -> Option<u8> {
        match self {
            Self::Factory => Some(FACTORY_SIZE),
            Self::Alternative | Self::Artist | Self::User(_) => None,
        }
    }

    #[must_use]
    pub const fn is_writable(self) -> bool {
        matches!(self, Self::User(_))
    }

    #[must_use]
    pub fn label(self, user_area_names: &[String]) -> String {
        match self {
            Self::Factory => "Factory".to_owned(),
            Self::Alternative => "Alternative".to_owned(),
            Self::Artist => "Artist".to_owned(),
            Self::User(number) => user_area_names
                .get(usize::from(number).saturating_sub(1))
                .cloned()
                .unwrap_or_else(|| format!("User {number}")),
        }
    }
}

impl fmt::Display for PresetArea {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.setting_value())
    }
}

#[cfg(test)]
mod tests {
    use super::PresetArea;

    #[test]
    fn setting_values_round_trip() {
        for area in [
            PresetArea::Factory,
            PresetArea::Artist,
            PresetArea::User(1),
            PresetArea::User(3),
        ] {
            assert_eq!(PresetArea::from_setting(&area.setting_value()), Some(area));
        }
        assert_eq!(PresetArea::from_setting("user-1"), None);
        assert_eq!(PresetArea::User(1).user_area_payload(), None);
        assert_eq!(
            PresetArea::User(2).user_area_payload(),
            Some("user-2".to_owned())
        );
        assert!(!PresetArea::Factory.is_writable());
    }
}
