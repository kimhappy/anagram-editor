use leptos::prelude::*;

use super::{UNREADABLE_RETRY_MILLIS, deadline_after};
use crate::{
    backend::BackendError,
    host::{self, confirm},
    model::{
        clipboard::Clipboard,
        document::{self, ImportWarning},
        library::{PresetRef, SlotEntry, read_only_landing},
        preset::Preset,
        slot::Slot,
    },
    protocol::{
        hid::{area::PresetArea, preset::PresetDocument, uuid::PresetUuid},
        midi::Command,
    },
    session::{NoticeAction, Session, StoredPreset, Unreadable},
};

impl Session {
    pub fn load(self, preset_ref: PresetRef) {
        if !self.has_no_running_task() || !self.confirm_discard() {
            return;
        }
        self.run_replacing(format!("Loading {}…", preset_ref.slot), async move {
            match self.select_slot(preset_ref).await {
                Ok(()) => self.fetch_into_draft(preset_ref).await,
                Err(error) => self.notify(error.to_string(), true),
            }
        });
    }

    async fn select_slot(self, preset_ref: PresetRef) -> Result<(), BackendError> {
        self.settle_audition().await;
        let current = self.current.get_untracked();
        let has_preview = self
            .link
            .audition
            .with_untracked(|audition| audition.slot.is_some());
        if is_bank_loaded(preset_ref.area, current.area, has_preview) {
            self.send(Command::SelectPreset(preset_ref.slot.into()));
            return Ok(());
        }
        if preset_ref.area != current.area {
            self.refresh_index(preset_ref.area).await;
        }
        self.backend()
            .select_area(preset_ref.area, preset_ref.slot)
            .await
    }

    fn adopt_blank(self, preset_ref: PresetRef) {
        let mut preset = Preset::default();
        preset.set_uuid(Some(PresetUuid::random()));
        self.adopt(preset_ref, preset, StoredPreset::default(), true);
    }

    pub fn browse_area(self, area: PresetArea) {
        self.browsed_area.set(area);
        let is_cached = area != self.current.get_untracked().area
            && self
                .library
                .with_untracked(|library| library.index(area).is_some());
        if !is_cached {
            host::spawn(async move {
                self.refresh_index(area).await;
            });
        }
    }

    pub fn new_preset(self, preset_ref: PresetRef) {
        let is_drafting_here =
            self.current.get_untracked() == preset_ref && self.is_new.get_untracked();
        if !preset_ref.area.is_writable()
            || is_drafting_here
            || !self.has_no_running_task()
            || !self.confirm_discard()
        {
            return;
        }
        self.run_replacing(format!("Opening {}…", preset_ref.slot), async move {
            if let Err(error) = self.select_slot(preset_ref).await {
                self.notify(error.to_string(), true);
                return;
            }
            let notice = self.read_into_draft(preset_ref).await;
            let has_opened_stored = self.current.get_untracked() == preset_ref
                && !self.is_new.get_untracked()
                && self.link.unreadable.get_value().is_none();
            if !has_opened_stored {
                return;
            }
            let name = self.draft.with_untracked(|preset| preset.name().to_owned());
            self.notify(
                format!(
                    "{} already holds \"{name}\" on the Anagram; opened it instead. {}",
                    preset_ref.slot,
                    notice.unwrap_or_default()
                )
                .trim_end()
                .to_owned(),
                false,
            );
        });
    }

    pub fn save(self) {
        let current = self.current.get_untracked();
        if !current.area.is_writable() {
            self.notify("This area is read-only.", true);
            return;
        }
        self.run_busy(format!("Saving {}…", current.slot), async move {
            self.settle_audition().await;
            let found = match self.fetch_listed(current).await {
                Ok(document) => document.as_ref().map(SlotEntry::of_document),
                Err(error) => {
                    self.notify(
                        format!("Could not check {} before saving: {error}", current.slot),
                        true,
                    );
                    return;
                }
            };
            let expected = self.stored.with_untracked(|stored| stored.identity.clone());
            let draft_name = self.draft.with_untracked(|preset| preset.name().to_owned());
            if let Some(question) =
                overwrite_question(current.slot, expected.as_ref(), found.as_ref(), &draft_name)
                && !confirm(&question)
            {
                self.refresh_index(current.area).await;
                self.notify(format!("{} was not saved.", current.slot), true);
                return;
            }
            if self.draft.with_untracked(|preset| preset.uuid().is_none()) {
                self.draft
                    .update(|preset| preset.set_uuid(Some(PresetUuid::random())));
            }
            let preset = self.draft.get_untracked();
            let scene = self.scene.get_untracked();
            let document = document::export(&preset, scene);
            let identity = SlotEntry::of_document(&document);
            match self.backend().save_preset(current, document, true).await {
                Ok(()) => {
                    if self.current.get_untracked() == current {
                        self.baseline.set(preset);
                        self.stored.set(StoredPreset {
                            identity: Some(identity),
                            landing_scene: scene,
                        });
                        self.is_new.set(false);
                        self.await_device(current);
                    }
                    let is_listed = self.refresh_index(current.area).await;
                    self.resume_preview();
                    if is_listed {
                        self.notify(format!("Saved {}", current.slot), false);
                    } else {
                        self.notify(
                            format!(
                                "Saved {}, but the preset list could not be refreshed.",
                                current.slot
                            ),
                            true,
                        );
                    }
                }
                Err(error) => {
                    self.refresh_index(current.area).await;
                    self.resume_after_failure(format!("Save failed: {error}"));
                }
            }
        });
    }

    pub fn copy_presets(self, slots: Vec<Slot>) {
        let current = self.current.get_untracked();
        if !current.area.is_writable() {
            self.notify("Presets of this area cannot be read.", true);
            return;
        }
        self.run_busy("Copying…", async move {
            let mut reads = Vec::with_capacity(slots.len());
            for slot in slots {
                reads.push(self.copy_source(current, slot).await);
            }
            let (documents, unread) = split_reads(reads);
            if let Some(summary) = copy_summary(documents.len(), &unread) {
                self.notify(summary, !unread.is_empty());
            }
            if !documents.is_empty() {
                self.clipboard.set(Clipboard::Presets(documents));
            }
        });
    }

    async fn copy_source(
        self,
        current: PresetRef,
        slot: Slot,
    ) -> Result<Option<PresetDocument>, (Slot, BackendError)> {
        if slot == current.slot && self.has_unsaved_edits() {
            return Ok(Some(self.draft.with_untracked(|preset| {
                document::export(preset, self.scene.get_untracked())
            })));
        }
        self.backend()
            .fetch_preset(PresetRef { slot, ..current })
            .await
            .map_err(|error| (slot, error))
    }

    pub fn paste_presets(self, target: Slot) {
        let Clipboard::Presets(documents) = self.clipboard.get_untracked() else {
            return;
        };
        let current = self.current.get_untracked();
        let area = current.area;
        if !area.is_writable() {
            self.notify("This area is read-only.", true);
            return;
        }
        if !self.has_no_running_task() {
            return;
        }
        if !self
            .library
            .with_untracked(|library| library.is_listed(area))
        {
            self.notify(
                "The presets of this area are not listed yet; paste again once they are.",
                true,
            );
            return;
        }
        let targets: Vec<PresetRef> = (target.index()..)
            .map_while(Slot::from_index)
            .take(documents.len())
            .map(|slot| PresetRef { area, slot })
            .collect();
        let overflow = documents.len() - targets.len();
        if overflow > 0
            && !confirm(&format!(
                "Only {} preset(s) fit from {target}. Leave out the last {overflow}?",
                targets.len()
            ))
        {
            return;
        }
        let taken = self.library.with_untracked(|library| {
            targets
                .iter()
                .filter(|preset_ref| library.entry(**preset_ref).is_some())
                .count()
        });
        if taken > 0 && !confirm(&format!("Overwrite {taken} preset(s) from {target}?")) {
            return;
        }
        let is_over_current = targets.contains(&current);
        if is_over_current && !self.confirm_discard() {
            return;
        }
        self.run_task(
            format!("Pasting at {target}…"),
            is_over_current,
            async move {
                if !self.park_preview().await {
                    return;
                }
                let written = targets.len();
                let pasted: Vec<(Slot, PresetDocument)> = targets
                    .iter()
                    .zip(documents)
                    .map(|(preset_ref, mut document)| {
                        document.preset.uuid = Some(PresetUuid::random().as_str().to_owned());
                        (preset_ref.slot, document)
                    })
                    .collect();
                let outcome = self.backend().save_presets(area, pasted).await;
                self.refresh_index(area).await;
                let shown = self.current.get_untracked();
                let is_shown_written = targets.contains(&shown)
                    && !matches!(&outcome, Err(BackendError::NotWritten { failed }) if failed.contains(&shown.slot));
                if is_shown_written {
                    self.fetch_into_draft(shown).await;
                } else {
                    self.land_after_bank_reload();
                }
                match outcome {
                    Ok(()) => self.notify(format!("Pasted {written} preset(s) from {target}"), false),
                    Err(error) => self.notify(format!("Paste at {target} failed: {error}"), true),
                }
            },
        );
    }

    pub fn delete_presets(self, slots: Vec<Slot>) {
        let area = self.current.get_untracked().area;
        let doomed: Vec<(PresetRef, SlotEntry)> = self.library.with_untracked(|library| {
            slots
                .iter()
                .map(|&slot| PresetRef { area, slot })
                .filter_map(|preset_ref| Some((preset_ref, library.entry(preset_ref)?.clone())))
                .collect()
        });
        let current = self.current.get_untracked();
        let is_current = doomed.iter().any(|(preset_ref, _)| *preset_ref == current);
        if doomed.is_empty()
            || !self.has_no_running_task()
            || (is_current && !self.confirm_discard())
            || !confirm(&delete_question(&doomed))
        {
            return;
        }
        let label = format!(
            "Deleting {}…",
            slot_list(doomed.iter().map(|(preset_ref, _)| preset_ref.slot))
        );
        self.run_task(label, is_current, async move {
            if !self.park_preview().await {
                return;
            }
            let backend = self.backend();
            let mut outcome = DeleteOutcome::default();
            for (preset_ref, entry) in doomed {
                let slot = preset_ref.slot;
                match backend.fetch_preset(preset_ref).await {
                    Ok(stored)
                        if stored.as_ref().is_some_and(|document| {
                            entry.is_same(&SlotEntry::of_document(document))
                        }) =>
                    {
                        match backend.delete_preset(preset_ref).await {
                            Ok(()) => outcome.deleted.push(slot),
                            Err(error) => outcome.failed.push((slot, error.to_string())),
                        }
                    }
                    Ok(_) => outcome.changed.push(slot),
                    Err(error) => outcome.failed.push((slot, error.to_string())),
                }
            }
            self.refresh_index(area).await;
            let shown = self.current.get_untracked();
            if shown.area == area && outcome.deleted.contains(&shown.slot) {
                self.adopt_blank(shown);
            } else {
                self.land_after_bank_reload();
            }
            let (text, is_error) = outcome.summary();
            self.notify(text, is_error);
        });
    }

    pub fn swap_by_drag(self, from: Slot, to: Slot) {
        let area = self.current.get_untracked().area;
        let name_at = |slot: Slot| {
            self.library.with_untracked(|library| {
                library
                    .entry(PresetRef { area, slot })
                    .map(|entry| entry.name.clone())
            })
        };
        let question = match (name_at(from), name_at(to)) {
            (Some(moved), Some(other)) => {
                format!("Swap {from} \"{moved}\" and {to} \"{other}\"?")
            }
            (Some(moved), None) => format!("Move {from} \"{moved}\" to the empty slot {to}?"),
            (None, _) => return,
        };
        if confirm(&question) {
            self.swap_presets(from, to);
        }
    }

    pub fn swap_presets(self, from: Slot, to: Slot) {
        let area = self.current.get_untracked().area;
        if from == to || !area.is_writable() {
            return;
        }
        self.run_busy(format!("Moving {from} to {to}…"), async move {
            if !self.park_preview().await {
                return;
            }
            let outcome = self.backend().swap_presets(area, from, to).await;
            match outcome {
                Ok(()) => {
                    self.refresh_index(area).await;
                    let current = self.current.get_untracked();
                    let moved = swapped(current.slot, from, to);
                    self.picked.update(|picked| {
                        *picked = picked.iter().map(|&slot| swapped(slot, from, to)).collect();
                    });
                    if moved != current.slot {
                        self.current.set(PresetRef { area, slot: moved });
                        self.await_device(PresetRef { area, slot: moved });
                        self.send(Command::SelectPreset(moved.into()));
                    }
                    self.land_after_bank_reload();
                    self.notify_with(
                        format!("Swapped {from} and {to}"),
                        false,
                        Some(NoticeAction::SwapBack { area, from, to }),
                    );
                }
                Err(error) => self.resume_after_failure(format!("Swap failed: {error}")),
            }
        });
    }

    async fn fetch_listed(
        self,
        preset_ref: PresetRef,
    ) -> Result<Option<PresetDocument>, BackendError> {
        let fetched = self.backend().fetch_preset(preset_ref).await?;
        let is_listed = self
            .library
            .with_untracked(|library| library.entry(preset_ref).is_some());
        if fetched.is_none() && is_listed {
            self.backend().fetch_preset(preset_ref).await
        } else {
            Ok(fetched)
        }
    }

    pub(crate) async fn fetch_into_draft(self, preset_ref: PresetRef) {
        if let Some(notice) = self.read_into_draft(preset_ref).await {
            self.notify(notice, false);
        }
    }

    pub(crate) async fn read_into_draft(self, preset_ref: PresetRef) -> Option<String> {
        self.clear_preview();
        self.await_device(preset_ref);
        if !preset_ref.area.is_writable() {
            self.adopt(
                preset_ref,
                Preset::empty("Read-only preset"),
                StoredPreset {
                    identity: None,
                    landing_scene: read_only_landing(preset_ref.area, preset_ref.slot),
                },
                false,
            );
            return None;
        }
        match self.fetch_listed(preset_ref).await {
            Ok(Some(document)) => {
                self.link.unreadable.set_value(None);
                let catalog = self.catalog.get_untracked();
                let imported = document::import(&document, &catalog);
                let notice = import_notice(preset_ref.slot, &imported.warnings);
                self.adopt(
                    preset_ref,
                    imported.preset,
                    StoredPreset {
                        identity: Some(SlotEntry::of_document(&document)),
                        landing_scene: imported.landing_scene,
                    },
                    false,
                );
                notice
            }
            Ok(None) => {
                self.link.unreadable.set_value(None);
                self.adopt_blank(preset_ref);
                None
            }
            Err(error) => {
                self.link.unreadable.set_value(Some(Unreadable {
                    spot: (preset_ref.area, preset_ref.slot.byte_index()),
                    retry_at: deadline_after(UNREADABLE_RETRY_MILLIS),
                }));
                self.await_device(self.current.get_untracked());
                self.notify(format!("Could not read {}: {error}", preset_ref.slot), true);
                None
            }
        }
    }
}

fn is_bank_loaded(target: PresetArea, current: PresetArea, has_preview: bool) -> bool {
    target == current && (!has_preview || target == PresetArea::FIRST_USER)
}

fn split_reads<T, E>(reads: Vec<Result<Option<T>, E>>) -> (Vec<T>, Vec<E>) {
    let (read, unread): (Vec<_>, Vec<_>) = reads.into_iter().partition(Result::is_ok);
    (
        read.into_iter().filter_map(Result::ok).flatten().collect(),
        unread.into_iter().filter_map(Result::err).collect(),
    )
}

fn import_notice(slot: Slot, warnings: &[ImportWarning]) -> Option<String> {
    let unknown_plugins = warnings
        .iter()
        .filter(|warning| matches!(warning, ImportWarning::UnknownPlugin { .. }))
        .count();
    let unmapped = warnings.len() - unknown_plugins;
    let plugins = (unknown_plugins > 0).then(|| {
        format!(
            "{unknown_plugins} block(s) use plugins this Anagram lacks; saving keeps them, but the Anagram does not load them."
        )
    });
    let items = (unmapped > 0)
        .then(|| format!("{unmapped} item(s) the editor could not map are left out when saving."));
    let clauses: Vec<String> = [plugins, items].into_iter().flatten().collect();
    (!clauses.is_empty()).then(|| format!("{slot} loaded: {}", clauses.join(" ")))
}

fn copy_summary(copied: usize, unread: &[(Slot, BackendError)]) -> Option<String> {
    match unread {
        [] => (copied > 0).then(|| format!("Copied {copied} preset(s)")),
        [(_, first), ..] => Some(format!(
            "Copied {copied} of {} preset(s); could not read {}: {first}",
            copied + unread.len(),
            slot_list(unread.iter().map(|(slot, _)| *slot))
        )),
    }
}

fn swapped(slot: Slot, from: Slot, to: Slot) -> Slot {
    if slot == from {
        to
    } else if slot == to {
        from
    } else {
        slot
    }
}

fn overwrite_question(
    slot: Slot,
    expected: Option<&SlotEntry>,
    found: Option<&SlotEntry>,
    draft: &str,
) -> Option<String> {
    match (expected, found) {
        (None, None) => None,
        (Some(expected), Some(found)) if expected.is_same(found) => None,
        (None, Some(found)) => Some(format!(
            "{slot} is no longer empty on the Anagram: it holds \"{}\". Overwrite it with \"{draft}\"?",
            found.name
        )),
        (Some(_), Some(found)) => Some(format!(
            "{slot} on the Anagram now holds \"{}\", not the preset you opened. Overwrite it with \"{draft}\"?",
            found.name
        )),
        (Some(_), None) => Some(format!(
            "{slot} was deleted on the Anagram. Save \"{draft}\" there again?"
        )),
    }
}

fn slot_list(slots: impl Iterator<Item = Slot>) -> String {
    slots
        .map(|slot| slot.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn delete_question(doomed: &[(PresetRef, SlotEntry)]) -> String {
    match doomed {
        [(preset_ref, entry)] => format!(
            "Delete preset {} \"{}\" from the Anagram?",
            preset_ref.slot, entry.name
        ),
        _ => format!(
            "Delete {} presets ({}) from the Anagram?",
            doomed.len(),
            slot_list(doomed.iter().map(|(preset_ref, _)| preset_ref.slot))
        ),
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
struct DeleteOutcome {
    deleted: Vec<Slot>,
    changed: Vec<Slot>,
    failed: Vec<(Slot, String)>,
}

impl DeleteOutcome {
    fn summary(&self) -> (String, bool) {
        let deleted = (!self.deleted.is_empty())
            .then(|| format!("Deleted {}.", slot_list(self.deleted.iter().copied())));
        let changed = (!self.changed.is_empty()).then(|| {
            format!(
                "{} changed on the Anagram and {} kept; the list was refreshed.",
                slot_list(self.changed.iter().copied()),
                if self.changed.len() == 1 {
                    "was"
                } else {
                    "were"
                }
            )
        });
        let failed = self.failed.first().map(|(_, reason)| {
            format!(
                "Could not delete {}: {reason}",
                slot_list(self.failed.iter().map(|(failed, _)| *failed))
            )
        });
        let is_error = changed.is_some() || failed.is_some();
        let text = [deleted, changed, failed]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" ");
        (text, is_error)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DeleteOutcome, copy_summary, import_notice, is_bank_loaded, overwrite_question,
        split_reads, swapped,
    };
    use crate::{
        backend::BackendError,
        model::{document::ImportWarning, library::SlotEntry, preset::Cell, slot::Slot},
        protocol::hid::{area::PresetArea, envelope::HidError, uuid::PresetUuid},
    };

    fn slot(text: &str) -> Slot {
        text.parse::<Slot>().expect("slot")
    }

    #[test]
    fn swapping_exchanges_the_two_slots_and_keeps_the_rest() {
        let (from, to, other) = (slot("01A"), slot("02B"), slot("03C"));
        assert_eq!(swapped(from, from, to), to);
        assert_eq!(swapped(to, from, to), from);
        assert_eq!(swapped(other, from, to), other);
    }

    #[test]
    fn a_preview_elsewhere_forces_a_bank_reload() {
        let first = PresetArea::FIRST_USER;
        assert!(is_bank_loaded(first, first, true));
        assert!(!is_bank_loaded(first, PresetArea::Factory, false));
        assert!(!is_bank_loaded(
            PresetArea::Factory,
            PresetArea::Factory,
            true
        ));
        assert!(is_bank_loaded(
            PresetArea::Factory,
            PresetArea::Factory,
            false
        ));
    }

    #[test]
    fn reads_split_into_documents_and_unread_slots() {
        let reads = vec![Ok(Some(1)), Ok(None), Err(slot("02A")), Ok(Some(2))];
        assert_eq!(split_reads(reads), (vec![1, 2], vec![slot("02A")]));
    }

    fn unknown_plugin() -> ImportWarning {
        ImportWarning::UnknownPlugin {
            cell: Cell { row: 0, column: 0 },
            uri: "urn:missing".to_owned(),
        }
    }

    fn unknown_symbol() -> ImportWarning {
        ImportWarning::UnknownSymbol {
            cell: Cell { row: 0, column: 0 },
            uri: "urn:test:Gain".to_owned(),
            symbol: "gone".to_owned(),
        }
    }

    #[test]
    fn the_import_notice_tells_kept_plugins_from_dropped_items() {
        assert_eq!(import_notice(slot("01A"), &[]), None);
        assert_eq!(
            import_notice(slot("01A"), &[unknown_plugin()]).as_deref(),
            Some(
                "01A loaded: 1 block(s) use plugins this Anagram lacks; saving keeps them, but the Anagram does not load them."
            )
        );
        assert_eq!(
            import_notice(slot("01A"), &[unknown_symbol(), unknown_symbol()]).as_deref(),
            Some("01A loaded: 2 item(s) the editor could not map are left out when saving.")
        );
        assert!(
            import_notice(slot("01A"), &[unknown_plugin(), unknown_symbol()])
                .is_some_and(|notice| notice.contains("lacks") && notice.contains("left out"))
        );
    }

    #[test]
    fn a_copy_summary_names_the_slots_that_could_not_be_read_and_why() {
        let timeout = || BackendError::Hid(HidError::Timeout("fetch_anagram_preset"));
        assert_eq!(copy_summary(3, &[]).as_deref(), Some("Copied 3 preset(s)"));
        assert_eq!(copy_summary(0, &[]), None);
        assert_eq!(
            copy_summary(2, &[(slot("07A"), timeout()), (slot("09C"), timeout())]).as_deref(),
            Some(
                "Copied 2 of 4 preset(s); could not read 07A, 09C: fetch_anagram_preset: no reply from the device"
            )
        );
    }

    fn entry(name: &str, seed: u8) -> SlotEntry {
        SlotEntry::new(
            Some(name.to_owned()),
            Some(PresetUuid::from_bytes([seed; 28])),
        )
    }

    #[test]
    fn saving_asks_only_when_the_slot_changed_on_the_device() {
        let at = slot("04B");
        let lead = entry("LEAD", 1);
        assert_eq!(overwrite_question(at, None, None, "NEW"), None);
        assert_eq!(
            overwrite_question(at, Some(&lead), Some(&entry("RENAMED", 1)), "NEW"),
            None
        );
        assert!(
            overwrite_question(at, None, Some(&lead), "NEW")
                .is_some_and(|question| question.contains("no longer empty"))
        );
        assert!(
            overwrite_question(at, Some(&lead), Some(&entry("OTHER", 2)), "NEW")
                .is_some_and(|question| question.contains("now holds \"OTHER\""))
        );
        assert!(
            overwrite_question(at, Some(&lead), None, "NEW")
                .is_some_and(|question| question.contains("was deleted"))
        );
    }

    #[test]
    fn a_delete_summary_reports_every_outcome() {
        let outcome = DeleteOutcome {
            deleted: vec![slot("01A"), slot("01B")],
            changed: vec![slot("02A")],
            failed: vec![(slot("03A"), "timeout".to_owned())],
        };
        assert_eq!(
            outcome.summary(),
            (
                "Deleted 01A, 01B. 02A changed on the Anagram and was kept; the list was refreshed. Could not delete 03A: timeout"
                    .to_owned(),
                true
            )
        );
        let clean = DeleteOutcome {
            deleted: vec![slot("01A")],
            ..DeleteOutcome::default()
        };
        assert_eq!(clean.summary(), ("Deleted 01A.".to_owned(), false));
    }
}
