mod audition;
mod device;
#[cfg(test)]
mod fake;
mod files;
#[cfg(test)]
mod flows;
mod follow;
mod plugins;
mod presets;
mod usage;

use std::time::Duration;

use leptos::prelude::*;

use crate::{
    host::{self, confirm},
    model::library::PresetRef,
    protocol::{hid::area::PresetArea, midi::Command},
    session::Session,
};

const POLL_INTERVAL: Duration = Duration::from_millis(500);
const SETTLE_GRACE_MILLIS: f64 = 4000.0;
const UNREADABLE_RETRY_MILLIS: f64 = 30_000.0;

impl Session {
    pub fn start(self) {
        if !self.begin("Reading presets…", true) {
            return;
        }
        self.link.is_polling.set(true);
        host::spawn(async move {
            let current = self.current.get_untracked();
            self.refresh_index(current.area).await;
            if current.area != PresetArea::FIRST_USER {
                self.refresh_index(PresetArea::FIRST_USER).await;
            }
            self.fetch_into_draft(current).await;
            self.announce_arrival();
            if let Some(carried) = self.take_carried() {
                self.restore(carried, current);
            }
            self.finish();
            self.poll().await;
        });
    }

    #[must_use]
    pub fn is_read_only(self) -> bool {
        !self.current.get().area.is_writable()
    }

    #[must_use]
    pub fn confirm_discard(self) -> bool {
        !self.has_unsaved_edits()
            || confirm(&format!(
                "Discard unsaved edits to \"{}\"?",
                self.draft.with_untracked(|preset| preset.name().to_owned())
            ))
    }

    pub(crate) fn has_no_running_task(self) -> bool {
        self.link.busy.with_untracked(Option::is_none)
    }

    pub(crate) fn begin(self, label: impl Into<String>, replaces_draft: bool) -> bool {
        let is_idle = self.has_no_running_task();
        if is_idle {
            self.link.busy.set(Some(label.into()));
            if replaces_draft {
                self.hold_draft();
            }
        }
        is_idle
    }

    fn hold_draft(self) {
        self.link.is_replacing_draft.set_value(true);
        self.abandon_drag();
    }

    pub(crate) fn run_busy(
        self,
        label: impl Into<String>,
        task: impl Future<Output = ()> + 'static,
    ) -> bool {
        self.run_task(label, false, task)
    }

    pub(crate) fn run_replacing(
        self,
        label: impl Into<String>,
        task: impl Future<Output = ()> + 'static,
    ) -> bool {
        self.run_task(label, true, task)
    }

    pub(crate) fn run_task(
        self,
        label: impl Into<String>,
        replaces_draft: bool,
        task: impl Future<Output = ()> + 'static,
    ) -> bool {
        if !self.begin(label, replaces_draft) {
            return false;
        }
        host::spawn(async move {
            task.await;
            self.finish();
        });
        true
    }

    pub(crate) fn finish(self) {
        self.link.busy.set(None);
        self.link.is_replacing_draft.set_value(false);
        self.pump_audition();
    }

    pub(crate) async fn refresh_index(self, area: PresetArea) -> bool {
        self.link.listing.update(|listing| {
            listing.insert(area);
        });
        let listed = self.backend().load_index(area).await;
        self.link.listing.update(|listing| {
            listing.remove(&area);
        });
        match listed {
            Ok(index) => {
                self.library
                    .update(|library| library.set_index(area, index));
                true
            }
            Err(error) => {
                self.notify(format!("Could not list presets: {error}"), true);
                false
            }
        }
    }

    pub(crate) fn await_device(self, preset_ref: PresetRef) {
        self.link
            .expected
            .set(Some((preset_ref.area, preset_ref.slot.byte_index())));
        self.link.settling_until.set(settle_deadline());
    }

    pub(crate) fn send(self, command: Command) -> bool {
        self.backend()
            .send_midi(command)
            .inspect_err(|error| self.notify(format!("MIDI send failed: {error}"), true))
            .is_ok()
    }
}

fn settle_deadline() -> f64 {
    deadline_after(SETTLE_GRACE_MILLIS)
}

#[expect(clippy::float_arithmetic, reason = "a deadline in milliseconds")]
fn deadline_after(millis: f64) -> f64 {
    host::now_millis() + millis
}
