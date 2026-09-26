use std::collections::BTreeMap;

use super::slot::Slot;
use crate::protocol::hid::{
    action::{PresetPosition, parse_preset_filename},
    area::PresetArea,
    preset::PresetDocument,
    uuid::PresetUuid,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AreaInfo {
    pub area: PresetArea,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlotEntry {
    pub name: String,
    pub uuid: Option<PresetUuid>,
    pub error: Option<String>,
}

impl SlotEntry {
    #[must_use]
    pub fn new(name: Option<String>, uuid: Option<PresetUuid>) -> Self {
        Self {
            name: name.unwrap_or_else(|| UNREADABLE_NAME.to_owned()),
            uuid,
            error: None,
        }
    }

    #[must_use]
    pub fn of_document(document: &PresetDocument) -> Self {
        Self::new(
            document.preset.name.clone(),
            document.preset.uuid.as_deref().and_then(PresetUuid::parse),
        )
    }

    #[must_use]
    pub fn is_same(&self, other: &Self) -> bool {
        match (&self.uuid, &other.uuid) {
            (Some(mine), Some(theirs)) => mine == theirs,
            (None, None) => self.name == other.name,
            _ => false,
        }
    }
}

const UNREADABLE_NAME: &str = "Unreadable preset";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AreaIndex {
    ReadOnly { size: Option<u8> },
    Listed(Vec<Option<SlotEntry>>),
}

impl AreaIndex {
    #[must_use]
    pub const fn read_only(area: PresetArea) -> Self {
        Self::ReadOnly {
            size: area.read_only_size(),
        }
    }

    #[must_use]
    pub fn from_positions(positions: &[PresetPosition]) -> Self {
        let mut slots = vec![None; usize::from(Slot::COUNT)];
        for position in positions {
            let Some(slot) = parse_preset_filename(&position.filename).and_then(Slot::from_number)
            else {
                continue;
            };
            if let Some(target) = slots.get_mut(slot.index()) {
                *target = Some(SlotEntry {
                    error: position.error.clone().filter(|error| !error.is_empty()),
                    ..SlotEntry::new(position.name.clone(), position.id.clone())
                });
            }
        }
        Self::Listed(slots)
    }

    #[must_use]
    pub fn get(&self, slot: Slot) -> Option<&SlotEntry> {
        match self {
            Self::Listed(slots) => slots.get(slot.index())?.as_ref(),
            Self::ReadOnly { .. } => None,
        }
    }

    pub fn set(&mut self, slot: Slot, entry: Option<SlotEntry>) {
        if let Self::Listed(slots) = self
            && let Some(target) = slots.get_mut(slot.index())
        {
            *target = entry;
        }
    }

    pub fn entries(&self) -> impl Iterator<Item = (Slot, Option<&SlotEntry>)> {
        let slots = match self {
            Self::Listed(slots) => slots.as_slice(),
            Self::ReadOnly { .. } => &[],
        };
        Slot::all().zip(slots.iter().map(Option::as_ref))
    }

    #[must_use]
    pub fn first_free_slot(&self) -> Option<Slot> {
        self.entries()
            .find(|(_, entry)| entry.is_none())
            .map(|(slot, _)| slot)
    }

    #[must_use]
    pub fn holds(&self, slot: Slot) -> bool {
        match self {
            Self::Listed(_) => true,
            Self::ReadOnly { size } => size.is_none_or(|size| slot.byte_index() < size),
        }
    }

    pub fn rows(&self) -> impl Iterator<Item = Slot> {
        Slot::all().filter(move |slot| self.holds(*slot))
    }
}

#[must_use]
pub fn read_only_landing(area: PresetArea, slot: Slot) -> Slot {
    let landings: &[(u8, u8)] = match area {
        PresetArea::Factory => &[(18, 2), (20, 2), (22, 2), (23, 1)],
        PresetArea::Alternative => &[(5, 1), (11, 1)],
        PresetArea::Artist | PresetArea::User(_) => &[],
    };
    landings
        .iter()
        .find(|(number, _)| *number == slot.number())
        .and_then(|(_, scene)| Slot::from_index(usize::from(*scene)))
        .unwrap_or_default()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PresetRef {
    pub area: PresetArea,
    pub slot: Slot,
}

impl Default for PresetRef {
    fn default() -> Self {
        Self {
            area: PresetArea::FIRST_USER,
            slot: Slot::default(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Library {
    areas: Vec<AreaInfo>,
    index: BTreeMap<PresetArea, AreaIndex>,
}

impl Library {
    #[must_use]
    pub fn new(user_area_names: &[String], current: PresetArea) -> Self {
        let user_count = user_area_names.len().clamp(1, 126);
        let users =
            (1..=user_count).map(|number| PresetArea::User(u8::try_from(number).unwrap_or(1)));
        let visited = [PresetArea::Alternative, PresetArea::Artist]
            .into_iter()
            .filter(|area| *area == current);
        let areas: Vec<AreaInfo> = std::iter::once(PresetArea::Factory)
            .chain(visited)
            .chain(users)
            .map(|area| AreaInfo {
                label: area.label(user_area_names),
                area,
            })
            .collect();
        let index = areas
            .iter()
            .filter(|info| !info.area.is_writable())
            .map(|info| (info.area, AreaIndex::read_only(info.area)))
            .collect();
        Self { areas, index }
    }

    #[must_use]
    pub fn areas(&self) -> &[AreaInfo] {
        &self.areas
    }

    #[must_use]
    pub fn label(&self, area: PresetArea) -> String {
        self.areas
            .iter()
            .find(|info| info.area == area)
            .map_or_else(|| area.to_string(), |info| info.label.clone())
    }

    #[must_use]
    pub fn index(&self, area: PresetArea) -> Option<&AreaIndex> {
        self.index.get(&area)
    }

    #[must_use]
    pub fn is_listed(&self, area: PresetArea) -> bool {
        self.index.contains_key(&area)
    }

    pub fn set_index(&mut self, area: PresetArea, index: AreaIndex) {
        self.index.insert(area, index);
    }

    #[must_use]
    pub fn entry(&self, preset_ref: PresetRef) -> Option<&SlotEntry> {
        self.index(preset_ref.area)?.get(preset_ref.slot)
    }
}

#[cfg(test)]
mod tests {
    use super::{AreaIndex, Library, SlotEntry, read_only_landing};
    use crate::{
        model::slot::Slot,
        protocol::hid::{action::PresetPosition, area::PresetArea, uuid::PresetUuid},
    };

    #[test]
    fn positions_land_in_their_slots_whatever_their_order() {
        let positions = vec![
            PresetPosition {
                filename: "20.json".to_owned(),
                id: None,
                name: Some("CAPTURE".to_owned()),
                error: None,
            },
            PresetPosition {
                filename: "1.json".to_owned(),
                id: None,
                name: Some("JPOP".to_owned()),
                error: None,
            },
        ];
        let index = AreaIndex::from_positions(&positions);
        assert_eq!(
            index.get(Slot::default()).map(|entry| entry.name.as_str()),
            Some("JPOP")
        );
        assert_eq!(
            index
                .get(Slot::from_number(20).expect("slot"))
                .map(|entry| entry.name.as_str()),
            Some("CAPTURE")
        );
        assert_eq!(index.first_free_slot(), Slot::from_number(2));
    }

    #[test]
    fn areas_list_factory_then_users_and_the_visited_read_only_area() {
        let library = Library::new(
            &["Live".to_owned(), "Studio".to_owned()],
            PresetArea::Artist,
        );
        let areas: Vec<PresetArea> = library.areas().iter().map(|info| info.area).collect();
        assert_eq!(
            areas,
            [
                PresetArea::Factory,
                PresetArea::Artist,
                PresetArea::User(1),
                PresetArea::User(2)
            ]
        );
        assert_eq!(library.label(PresetArea::User(2)), "Studio");
        assert!(library.is_listed(PresetArea::Factory));
        assert!(!library.is_listed(PresetArea::User(1)));
        let at_home = Library::new(&[], PresetArea::FIRST_USER);
        assert!(
            !at_home
                .areas()
                .iter()
                .any(|info| info.area == PresetArea::Artist)
        );
    }

    #[test]
    fn read_only_areas_show_their_known_size_or_every_slot() {
        let factory = AreaIndex::read_only(PresetArea::Factory);
        assert_eq!(factory.rows().count(), 36);
        assert!(!factory.holds(Slot::from_number(37).expect("slot")));
        assert_eq!(AreaIndex::read_only(PresetArea::Artist).rows().count(), 126);
        assert_eq!(AreaIndex::Listed(Vec::new()).rows().count(), 126);
    }

    #[test]
    fn a_few_read_only_presets_land_on_a_later_scene() {
        let number = |value| Slot::from_number(value).expect("slot");
        let scene = |value| Slot::from_index(value).expect("scene");
        assert_eq!(read_only_landing(PresetArea::Factory, number(18)), scene(2));
        assert_eq!(read_only_landing(PresetArea::Factory, number(23)), scene(1));
        assert_eq!(read_only_landing(PresetArea::Factory, number(1)), scene(0));
        assert_eq!(
            read_only_landing(PresetArea::Alternative, number(11)),
            scene(1)
        );
    }

    fn entry(name: &str, seed: Option<u8>) -> SlotEntry {
        SlotEntry::new(
            Some(name.to_owned()),
            seed.map(|seed| PresetUuid::from_bytes([seed; 28])),
        )
    }

    #[test]
    fn entries_match_by_uuid_or_by_name_without_one() {
        assert!(entry("LEAD", Some(1)).is_same(&entry("RENAMED", Some(1))));
        assert!(!entry("LEAD", Some(1)).is_same(&entry("LEAD", Some(2))));
        assert!(entry("LEAD", None).is_same(&entry("LEAD", None)));
        assert!(!entry("LEAD", None).is_same(&entry("RHYTHM", None)));
        assert!(!entry("LEAD", None).is_same(&entry("LEAD", Some(1))));
    }
}
