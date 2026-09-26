use leptos::prelude::*;

use super::{POLL_INTERVAL, settle_deadline};
use crate::{
    backend::DeviceSnapshot,
    host::{self, sleep},
    model::{
        library::{PresetRef, SlotEntry},
        mode::Mode,
        slot::Slot,
    },
    protocol::{
        actuator::Actuator,
        hid::{area::PresetArea, settings::DeviceSettings},
        midi::Command,
    },
    session::Session,
};

const LISTING_EVERY_POLLS: u32 = 10;

#[derive(Debug, PartialEq, Eq)]
enum ListingReaction {
    Keep,
    Reload,
    Warn,
}

fn listing_reaction(
    stored: Option<&SlotEntry>,
    listed: Option<&SlotEntry>,
    has_unsaved_edits: bool,
) -> ListingReaction {
    let is_same = match (stored, listed) {
        (None, None) => true,
        (Some(stored), Some(listed)) => stored.is_same(listed),
        _ => false,
    };
    if is_same {
        ListingReaction::Keep
    } else if has_unsaved_edits {
        ListingReaction::Warn
    } else {
        ListingReaction::Reload
    }
}

impl Session {
    pub(crate) async fn poll(self) {
        let is_polling = move || self.link.is_polling.try_get_untracked().unwrap_or(false);
        let mut failures: u32 = 0;
        let mut ticks: u32 = 0;
        while is_polling() {
            sleep(POLL_INTERVAL).await;
            if !is_polling() {
                break;
            }
            let is_applying = self
                .link
                .audition
                .with_untracked(|audition| audition.in_flight);
            if self.link.busy.get_untracked().is_some() || is_applying {
                continue;
            }
            match self.backend().read_state().await {
                Ok(snapshot) => {
                    failures = 0;
                    self.follow(&snapshot).await;
                    ticks = ticks.wrapping_add(1);
                    if ticks.is_multiple_of(LISTING_EVERY_POLLS) {
                        self.reconcile_listing().await;
                    }
                }
                Err(error) => {
                    failures += 1;
                    if failures == 3 {
                        self.notify(format!("Lost contact with the Anagram: {error}"), true);
                    }
                }
            }
        }
    }

    pub(crate) async fn reconcile_listing(self) {
        let current = self.current.get_untracked();
        if !current.area.is_writable() || !self.is_quiet(current) {
            return;
        }
        let Ok(index) = self.backend().load_index(current.area).await else {
            return;
        };
        if !self.is_quiet(current) {
            return;
        }
        let is_unchanged = self
            .library
            .with_untracked(|library| library.index(current.area) == Some(&index));
        if is_unchanged {
            return;
        }
        let listed = index.get(current.slot).cloned();
        self.library
            .update(|library| library.set_index(current.area, index));
        let stored = self.stored.with_untracked(|stored| stored.identity.clone());
        match listing_reaction(stored.as_ref(), listed.as_ref(), self.has_unsaved_edits()) {
            ListingReaction::Keep => {}
            ListingReaction::Reload => {
                if self.begin(format!("Reloading {}…", current.slot), true) {
                    let import = self.read_into_draft(current).await;
                    self.notify(
                        format!(
                            "{} changed on the Anagram and was reloaded. {}",
                            current.slot,
                            import.unwrap_or_default()
                        )
                        .trim_end()
                        .to_owned(),
                        false,
                    );
                    self.finish();
                }
            }
            ListingReaction::Warn => {
                let change = listed.map_or_else(
                    || format!("{} was deleted on the Anagram.", current.slot),
                    |entry| {
                        format!(
                            "{} now holds \"{}\" on the Anagram.",
                            current.slot, entry.name
                        )
                    },
                );
                let warning =
                    format!("{change} Your edits are kept; saving asks before overwriting.");
                let is_new_warning = self
                    .link
                    .announced_change
                    .with_value(|announced| announced.as_deref() != Some(warning.as_str()));
                if is_new_warning {
                    self.link.announced_change.set_value(Some(warning.clone()));
                    self.notify(warning, true);
                }
            }
        }
    }

    fn is_quiet(self, current: PresetRef) -> bool {
        let is_moving = self.link.expected.get_untracked()
            != Some((current.area, current.slot.byte_index()))
            || host::now_millis() < self.link.settling_until.get_untracked();
        let is_applying = self
            .link
            .audition
            .with_untracked(|audition| audition.in_flight);
        self.current.get_untracked() == current
            && self.link.busy.get_untracked().is_none()
            && !self.link.is_replacing_draft.get_value()
            && !is_applying
            && !is_moving
    }

    pub fn stop_polling(self) {
        self.link.is_polling.set(false);
    }

    async fn follow(self, snapshot: &DeviceSnapshot) {
        if self
            .device_settings
            .with_untracked(|settings| *settings != snapshot.settings)
        {
            self.device_settings.set(snapshot.settings.clone());
        }
        if self
            .favourites
            .with_untracked(|favourites| *favourites != snapshot.favourites)
        {
            self.favourites.set(snapshot.favourites.clone());
        }
        self.follow_mode(snapshot.state.mode);
        let seen = (snapshot.state.area, snapshot.state.preset_index);
        let now = host::now_millis();
        if self
            .link
            .unreadable
            .get_value()
            .is_some_and(|unreadable| unreadable.spot != seen || unreadable.retry_at <= now)
        {
            self.link.unreadable.set_value(None);
        }
        let watch = Watch {
            expected: self.link.expected.get_untracked(),
            unreadable: self
                .link
                .unreadable
                .get_value()
                .map(|unreadable| unreadable.spot),
            preview: self.link.audition.with_untracked(|audition| audition.slot),
            is_settling: now < self.link.settling_until.get_untracked(),
            is_applying: self
                .link
                .audition
                .with_untracked(|audition| audition.in_flight),
            is_busy: self.link.busy.get_untracked().is_some(),
        };
        let is_listed = |preset_ref| {
            self.library
                .with_untracked(|library| library.entry(preset_ref).is_some())
        };
        match follow_step(seen, &watch, is_listed) {
            FollowStep::Confirmed => self.link.settling_until.set(0.0),
            FollowStep::Stay => {}
            FollowStep::MovePreview(slot) => {
                self.link
                    .audition
                    .update(|audition| audition.slot = Some(slot));
            }
            FollowStep::Follow(target) => self.follow_to(target).await,
        }
    }

    async fn follow_to(self, target: PresetRef) {
        if !self.begin(format!("Following the Anagram to {}…", target.slot), true) {
            return;
        }
        let discarded = self
            .has_unsaved_edits()
            .then(|| self.draft.with_untracked(|preset| preset.name().to_owned()));
        if target.area != self.current.get_untracked().area {
            self.refresh_index(target.area).await;
        }
        let import_notice = self.read_into_draft(target).await;
        let discard_notice = discarded
            .filter(|_| self.current.get_untracked() == target)
            .map(|name| {
                format!(
                    "Unsaved edits to {name} were discarded when the Anagram switched to {}.",
                    target.slot
                )
            });
        if let Some((text, is_error)) = follow_notice(discard_notice, import_notice) {
            self.notify(text, is_error);
        }
        self.finish();
    }

    fn follow_mode(self, seen: Mode) {
        let is_confirmed = seen == self.mode.get_untracked();
        let is_settling = host::now_millis() < self.link.mode_settling_until.get_untracked();
        if is_confirmed && is_settling {
            self.link.mode_settling_until.set(0.0);
        }
        if !is_confirmed && !is_settling {
            self.mode.set(seen);
        }
    }

    pub fn set_mode(self, mode: Mode) {
        if self.mode.get_untracked() == mode {
            return;
        }
        self.mode.set(mode);
        self.link.mode_settling_until.set(settle_deadline());
        self.send(Command::SelectMode(mode));
        if mode == Mode::Scene {
            self.send(Command::SelectScene(self.scene.get_untracked().into()));
        }
    }

    pub fn set_scene(self, scene: Slot) {
        self.scene.set(scene);
        if self.mode.get_untracked() == Mode::Scene {
            self.send(Command::SelectScene(scene.into()));
        }
    }

    #[must_use]
    pub fn test_binding(self, actuator: Actuator, value: u8) -> bool {
        let bindings = self
            .midi_settings
            .with_untracked(|settings| settings.bindings);
        if !bindings.is_enabled(actuator) {
            self.notify(
                format!(
                    "CC Enable is off for {}; turn it on in the device settings to test it.",
                    actuator.label()
                ),
                true,
            );
            return false;
        }
        self.send(Command::BindingValue {
            cc: bindings.cc(actuator),
            value: value.min(127),
        })
    }

    pub fn toggle_tuner(self) {
        if self.send(Command::ToggleTuner) {
            self.ui.is_tuner_open.update(|is_open| *is_open = !*is_open);
        }
    }

    pub fn apply_device_settings(self, opened: &DeviceSettings, target: &DeviceSettings) {
        let change = opened.diff(target);
        if change.is_empty() {
            return;
        }
        if change.needs_restart {
            self.restart_with(change.changes, "with the new settings".to_owned());
            return;
        }
        self.run_busy("Writing settings…", async move {
            let backend = self.backend();
            if let Err(error) = backend.edit_settings(change.changes).await {
                self.notify(format!("Settings were not written: {error}"), true);
                return;
            }
            match backend.read_state().await {
                Ok(snapshot) => self.device_settings.set(snapshot.settings),
                Err(error) => self.notify(
                    format!("Settings were written, but could not be read back: {error}"),
                    true,
                ),
            }
        });
    }
}

struct Watch {
    expected: Option<(PresetArea, u8)>,
    unreadable: Option<(PresetArea, u8)>,
    preview: Option<Slot>,
    is_settling: bool,
    is_applying: bool,
    is_busy: bool,
}

#[derive(Debug, PartialEq, Eq)]
enum FollowStep {
    Confirmed,
    Stay,
    MovePreview(Slot),
    Follow(PresetRef),
}

fn follow_step(
    seen: (PresetArea, u8),
    watch: &Watch,
    is_listed: impl Fn(PresetRef) -> bool,
) -> FollowStep {
    if Some(seen) == watch.expected {
        return FollowStep::Confirmed;
    }
    let preview = watch
        .preview
        .map(|slot| (PresetArea::FIRST_USER, slot.byte_index()));
    let is_ignored = Some(seen) == preview || Some(seen) == watch.unreadable;
    if is_ignored || watch.is_settling || watch.is_applying {
        return FollowStep::Stay;
    }
    let Some(slot) = Slot::from_index(usize::from(seen.1)) else {
        return FollowStep::Stay;
    };
    let target = PresetRef { area: seen.0, slot };
    if preview.is_some() && seen.0 == PresetArea::FIRST_USER && !is_listed(target) {
        FollowStep::MovePreview(slot)
    } else if watch.is_busy {
        FollowStep::Stay
    } else {
        FollowStep::Follow(target)
    }
}

fn follow_notice(discard: Option<String>, import: Option<String>) -> Option<(String, bool)> {
    match (discard, import) {
        (Some(discard), Some(import)) => Some((format!("{discard} {import}"), true)),
        (Some(discard), None) => Some((discard, true)),
        (None, Some(import)) => Some((import, false)),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{FollowStep, ListingReaction, Watch, follow_notice, follow_step, listing_reaction};
    use crate::{
        model::{
            library::{PresetRef, SlotEntry},
            slot::Slot,
        },
        protocol::hid::{area::PresetArea, uuid::PresetUuid},
    };

    fn entry(name: &str, seed: u8) -> SlotEntry {
        SlotEntry::new(
            Some(name.to_owned()),
            Some(PresetUuid::from_bytes([seed; 28])),
        )
    }

    #[test]
    fn a_changed_listing_reloads_a_clean_draft_and_warns_about_edits() {
        let lead = entry("LEAD", 1);
        let other = entry("OTHER", 2);
        assert_eq!(
            listing_reaction(Some(&lead), Some(&lead), true),
            ListingReaction::Keep
        );
        assert_eq!(listing_reaction(None, None, false), ListingReaction::Keep);
        assert_eq!(
            listing_reaction(Some(&lead), Some(&other), false),
            ListingReaction::Reload
        );
        assert_eq!(
            listing_reaction(Some(&lead), None, false),
            ListingReaction::Reload
        );
        assert_eq!(
            listing_reaction(None, Some(&other), false),
            ListingReaction::Reload
        );
        assert_eq!(
            listing_reaction(Some(&lead), Some(&other), true),
            ListingReaction::Warn
        );
    }

    #[test]
    fn a_discard_turns_the_follow_notice_into_an_error() {
        let text = |value: &str| Some(value.to_owned());
        assert_eq!(
            follow_notice(text("A"), text("B")),
            Some(("A B".to_owned(), true))
        );
        assert_eq!(follow_notice(text("A"), None), Some(("A".to_owned(), true)));
        assert_eq!(
            follow_notice(None, text("B")),
            Some(("B".to_owned(), false))
        );
        assert_eq!(follow_notice(None, None), None);
    }

    fn watch() -> Watch {
        Watch {
            expected: Some((PresetArea::FIRST_USER, 0)),
            unreadable: None,
            preview: None,
            is_settling: false,
            is_applying: false,
            is_busy: false,
        }
    }

    fn step(seen: u8, watch: &Watch) -> FollowStep {
        follow_step((PresetArea::FIRST_USER, seen), watch, |preset_ref| {
            preset_ref.slot.byte_index() < 4
        })
    }

    #[test]
    fn the_expected_slot_confirms_and_another_one_is_followed() {
        assert_eq!(step(0, &watch()), FollowStep::Confirmed);
        assert_eq!(
            step(2, &watch()),
            FollowStep::Follow(PresetRef {
                area: PresetArea::FIRST_USER,
                slot: Slot::from_index(2).expect("slot")
            })
        );
    }

    #[test]
    fn following_waits_while_settling_applying_busy_or_unreadable() {
        let with = |change: fn(&mut Watch)| {
            let mut watch = watch();
            change(&mut watch);
            step(2, &watch)
        };
        assert_eq!(with(|watch| watch.is_settling = true), FollowStep::Stay);
        assert_eq!(with(|watch| watch.is_applying = true), FollowStep::Stay);
        assert_eq!(with(|watch| watch.is_busy = true), FollowStep::Stay);
        assert_eq!(
            with(|watch| watch.unreadable = Some((PresetArea::FIRST_USER, 2))),
            FollowStep::Stay
        );
    }

    #[test]
    fn a_preview_moving_to_an_empty_slot_is_tracked() {
        let previewing = Watch {
            preview: Slot::from_index(4),
            ..watch()
        };
        assert_eq!(step(4, &previewing), FollowStep::Stay);
        assert_eq!(
            step(2, &previewing),
            FollowStep::Follow(PresetRef {
                area: PresetArea::FIRST_USER,
                slot: Slot::from_index(2).expect("slot")
            })
        );
        assert_eq!(
            step(
                9,
                &Watch {
                    is_busy: true,
                    ..previewing
                }
            ),
            FollowStep::MovePreview(Slot::from_index(9).expect("slot"))
        );
        assert_eq!(
            step(9, &previewing),
            FollowStep::MovePreview(Slot::from_index(9).expect("slot"))
        );
    }
}
