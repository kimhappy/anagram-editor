use std::time::Duration;

use leptos::prelude::*;

use crate::{
    host::{self, sleep},
    model::{document, slot::Slot},
    protocol::hid::{area::PresetArea, preset::PresetDocument, settings::selection_payload},
    session::{Audition, Session},
};

impl Session {
    pub(crate) async fn settle_audition(self) {
        while self
            .link
            .audition
            .try_with_untracked(|audition| audition.in_flight)
            .unwrap_or(false)
        {
            sleep(Duration::from_millis(50)).await;
        }
    }

    pub(crate) fn clear_preview(self) {
        self.link.audition.update(Audition::clear);
    }

    pub(crate) fn resume_preview(self) {
        self.clear_preview();
        if self.has_unsaved_edits() {
            self.audition();
        }
    }

    pub(crate) fn land_after_bank_reload(self) {
        if !self.has_unsaved_edits() {
            self.scene
                .set(self.stored.with_untracked(|stored| stored.landing_scene));
        }
        self.resume_preview();
    }

    pub(crate) fn resume_after_failure(self, text: String) {
        self.notify(text, true);
        self.resume_preview();
    }

    pub(crate) async fn park_preview(self) -> bool {
        self.settle_audition().await;
        let current = self.current.get_untracked();
        let selection = selection_payload(current.area, current.slot.byte_index());
        match self.backend().edit_settings(selection).await {
            Ok(()) => true,
            Err(error) => {
                self.notify(format!("Could not restore the selection: {error}"), true);
                false
            }
        }
    }

    pub(crate) fn audition(self) {
        if self.is_read_only() {
            return;
        }
        let document = self
            .draft
            .with_untracked(|preset| document::export(preset, self.scene.get_untracked()));
        self.link
            .audition
            .update(|audition| audition.pending = Some(document));
        self.pump_audition();
    }

    fn first_free_preview_slot(self) -> Option<Slot> {
        self.library
            .with_untracked(|library| library.index(PresetArea::FIRST_USER)?.first_free_slot())
    }

    pub(crate) fn pump_audition(self) {
        let Some(is_full) = self.library.try_with_untracked(|library| {
            library
                .index(PresetArea::FIRST_USER)
                .is_some_and(|index| index.first_free_slot().is_none())
        }) else {
            return;
        };
        if self.link.busy.get_untracked().is_some() {
            return;
        }
        let next = self
            .link
            .audition
            .try_update(|audition| next_audition(audition, is_full));
        match next.flatten() {
            Some(Ok(document)) => host::spawn(async move {
                let backend = self.backend();
                let landed = match backend.try_preset(document).await {
                    Ok(()) => Ok(backend.read_state().await.map_or_else(
                        |_unread| self.first_free_preview_slot(),
                        |snapshot| Slot::from_index(usize::from(snapshot.state.preset_index)),
                    )),
                    Err(error) => Err(error),
                };
                self.link.audition.update(|audition| {
                    audition.in_flight = false;
                    if let Ok(slot) = &landed {
                        audition.slot = *slot;
                    }
                });
                if let Err(error) = landed {
                    self.notify(format!("Preview failed: {error}"), true);
                }
                self.pump_audition();
            }),
            Some(Err(())) => self.notify(
                "The Anagram has no free user slot to preview edits in. Edits are kept and saved as usual.",
                true,
            ),
            None => {}
        }
    }
}

fn next_audition(audition: &mut Audition, is_full: bool) -> Option<Result<PresetDocument, ()>> {
    if audition.in_flight || audition.is_blocked {
        None
    } else if is_full && audition.pending.is_some() {
        audition.is_blocked = true;
        audition.pending = None;
        Some(Err(()))
    } else {
        let document = audition.pending.take()?;
        audition.in_flight = true;
        Some(Ok(document))
    }
}

#[cfg(test)]
mod tests {
    use super::next_audition;
    use crate::{protocol::hid::preset::PresetDocument, session::Audition};

    #[test]
    fn a_pending_audition_is_sent_once_and_blocked_when_full() {
        let pending = || Audition {
            pending: Some(PresetDocument::default()),
            ..Audition::default()
        };
        let mut sendable = pending();
        assert_eq!(
            next_audition(&mut sendable, false),
            Some(Ok(PresetDocument::default()))
        );
        assert!(sendable.in_flight);
        assert_eq!(next_audition(&mut sendable, false), None);
        let mut blocked = pending();
        assert_eq!(next_audition(&mut blocked, true), Some(Err(())));
        assert!(blocked.is_blocked && blocked.pending.is_none());
        assert_eq!(next_audition(&mut Audition::default(), false), None);
    }
}
