#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NameRule {
    pub max_length: usize,
}

pub const PRESET_NAME: NameRule = NameRule { max_length: 16 };

pub const USER_FILE_NAME: NameRule = NameRule { max_length: 32 };

pub const INPUT_GAIN_NAME: NameRule = NameRule { max_length: 16 };

impl NameRule {
    #[must_use]
    pub fn sanitize(self, name: &str) -> String {
        name.trim()
            .chars()
            .filter(|&character| is_allowed(character))
            .take(self.max_length)
            .collect::<String>()
            .trim()
            .to_owned()
    }

    #[must_use]
    pub fn accept(self, name: &str) -> Option<String> {
        Some(self.sanitize(name)).filter(|accepted| !accepted.is_empty())
    }
}

const fn is_allowed(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, ' ' | '_' | '-')
}

#[cfg(test)]
mod tests {
    use super::{PRESET_NAME, USER_FILE_NAME};

    #[test]
    fn preset_names_drop_dots_and_stop_at_16() {
        assert_eq!(
            PRESET_NAME.sanitize("  Ephemeris II / v2!  "),
            "Ephemeris II  v2"
        );
        assert_eq!(
            PRESET_NAME.sanitize("abcdefghijklmnopqrstuvwxyz"),
            "abcdefghijklmnop"
        );
    }

    #[test]
    fn file_names_follow_the_suite_rule_and_stop_at_32() {
        assert_eq!(USER_FILE_NAME.sanitize("  My IR / v2.1  "), "My IR  v21");
        assert_eq!(USER_FILE_NAME.sanitize(&"a".repeat(40)).len(), 32);
        assert_eq!(USER_FILE_NAME.accept(" ... "), None);
    }
}
