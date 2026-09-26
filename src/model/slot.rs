use std::{fmt, str::FromStr};

use crate::protocol::midi::SlotNumber;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Slot(u8);

impl Slot {
    pub const COUNT: u8 = SlotNumber::COUNT;
    const PER_GROUP: u8 = 3;

    pub fn all() -> impl Iterator<Item = Self> {
        (0..Self::COUNT).map(Self)
    }

    #[must_use]
    pub fn from_index(index: usize) -> Option<Self> {
        u8::try_from(index)
            .ok()
            .filter(|&index| index < Self::COUNT)
            .map(Self)
    }

    #[must_use]
    pub const fn index(self) -> usize {
        self.0 as usize
    }

    #[must_use]
    pub const fn byte_index(self) -> u8 {
        self.0
    }

    #[must_use]
    pub const fn number(self) -> u8 {
        self.0 + 1
    }

    #[must_use]
    pub const fn from_number(number: u8) -> Option<Self> {
        if number >= 1 && number <= Self::COUNT {
            Some(Self(number - 1))
        } else {
            None
        }
    }

    #[must_use]
    pub const fn last_of_group(self) -> Self {
        Self(self.0 - self.0.rem_euclid(Self::PER_GROUP) + Self::PER_GROUP - 1)
    }

    #[must_use]
    pub const fn next(self) -> Self {
        Self((self.0 + 1).rem_euclid(Self::COUNT))
    }

    #[must_use]
    pub const fn previous(self) -> Self {
        Self((self.0 + Self::COUNT - 1).rem_euclid(Self::COUNT))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseSlotError;

impl FromStr for Slot {
    type Err = ParseSlotError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let text = text.trim();
        let (group, letter) = text
            .split_at_checked(text.len().saturating_sub(1))
            .ok_or(ParseSlotError)?;
        let group = group.parse::<u8>().ok().ok_or(ParseSlotError)?;
        let offset = match letter.to_ascii_uppercase().as_str() {
            "A" => 0,
            "B" => 1,
            "C" => 2,
            _ => return Err(ParseSlotError),
        };
        let index = group
            .checked_sub(1)
            .and_then(|group| group.checked_mul(Self::PER_GROUP))
            .and_then(|start| start.checked_add(offset))
            .filter(|&index| index < Self::COUNT)
            .ok_or(ParseSlotError)?;
        Ok(Self(index))
    }
}

impl fmt::Display for Slot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let group = self.0.div_euclid(Self::PER_GROUP) + 1;
        let letter = char::from(b'A' + self.0.rem_euclid(Self::PER_GROUP));
        write!(formatter, "{group:02}{letter}")
    }
}

impl From<Slot> for SlotNumber {
    fn from(slot: Slot) -> Self {
        Self::new(slot.number()).expect("every slot has a MIDI number")
    }
}

#[cfg(test)]
mod tests {
    use super::Slot;

    #[test]
    fn numbering_runs_from_01a_to_42c() {
        let labels: Vec<String> = Slot::all().map(|slot| slot.to_string()).collect();
        assert_eq!(labels.first().map(String::as_str), Some("01A"));
        assert_eq!(labels.get(1).map(String::as_str), Some("01B"));
        assert_eq!(labels.get(3).map(String::as_str), Some("02A"));
        assert_eq!(labels.last().map(String::as_str), Some("42C"));
    }

    #[test]
    fn parsing_accepts_only_real_slots() {
        assert_eq!(
            "02b".parse::<Slot>().map(|slot| slot.to_string()),
            Ok("02B".to_owned())
        );
        assert_eq!(
            " 1C ".parse::<Slot>().map(|slot| slot.to_string()),
            Ok("01C".to_owned())
        );
        assert_eq!("43A".parse::<Slot>().ok(), None);
        assert_eq!("00A".parse::<Slot>().ok(), None);
        assert_eq!("01D".parse::<Slot>().ok(), None);
        assert_eq!("A".parse::<Slot>().ok(), None);
        assert_eq!("가A".parse::<Slot>().ok(), None);
    }

    #[test]
    fn stepping_wraps_around() {
        let first = Slot::default();
        assert_eq!(first.previous().to_string(), "42C");
        assert_eq!(first.previous().next(), first);
    }
}
