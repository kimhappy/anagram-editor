use std::time::Duration;

use leptos::prelude::*;
use serde_json::{Map, Value};

use crate::{
    host,
    lifecycle::{CarriedDraft, CarryFate, Reconnect, ReconnectCause, carried_fate},
    model::{clipboard::Clipboard, document, library::PresetRef},
    protocol::hid::{settings::selection_payload, version::Version},
    session::Session,
};

const RESTART_WAIT: Duration = Duration::from_secs(120);
const INSTALL_WAIT: Duration = Duration::from_mins(20);
const FIRMWARE_SETTLE: Duration = Duration::from_secs(2);

impl Session {
    pub(crate) fn restart_with(self, mut changes: Map<String, Value>, reason: String) {
        self.run_busy("Restarting the Anagram…", async move {
            self.settle_audition().await;
            let current = self.current.get_untracked();
            changes.extend(selection_payload(current.area, current.slot.byte_index()));
            let backend = self.backend();
            if let Err(error) = backend.edit_settings(changes).await {
                self.notify(format!("Settings were not written: {error}"), true);
                return;
            }
            self.expect_reconnect(ReconnectCause::Restart { reason }, RESTART_WAIT);
            if let Err(error) = backend.reboot().await {
                self.abandon_reconnect();
                self.notify(format!("The Anagram did not restart: {error}"), true);
            }
        });
    }

    #[must_use]
    pub fn install_firmware(
        self,
        file_name: String,
        package: Vec<u8>,
        expected: Option<Version>,
    ) -> bool {
        self.run_busy(format!("Sending {file_name}…"), async move {
            if let Err(error) = self.backend().ensure_serial().await {
                self.notify(format!("Firmware was not sent: {error}"), true);
                return;
            }
            let sent = self
                .backend()
                .send_firmware(
                    file_name,
                    package,
                    Box::new(move |share| self.files.upload_progress.set(Some(share))),
                )
                .await;
            self.files.upload_progress.set(None);
            if let Err(error) = sent {
                self.notify(format!("Firmware was not sent: {error}"), true);
                return;
            }
            host::sleep(FIRMWARE_SETTLE).await;
            self.expect_reconnect(ReconnectCause::FirmwareInstall { expected }, INSTALL_WAIT);
            if let Err(error) = self.backend().start_firmware_install().await {
                self.abandon_reconnect();
                self.notify(
                    format!("The Anagram did not start the install: {error}"),
                    true,
                );
            }
        })
    }

    fn expect_reconnect(self, cause: ReconnectCause, wait: Duration) {
        self.stop_polling();
        if let Some(app) = self.app {
            app.reconnect.set(Some(Reconnect {
                cause,
                deadline: Some(wait.as_secs_f64().mul_add(1000.0, host::now_millis())),
            }));
        }
    }

    fn abandon_reconnect(self) {
        if let Some(app) = self.app {
            app.reconnect.set(None);
        }
        self.link.is_polling.set(true);
        host::spawn(self.poll());
    }

    #[must_use]
    pub fn carry(self) -> Option<CarriedDraft> {
        let has_edits = self.has_unsaved_edits();
        let clipboard = self.clipboard.get_untracked();
        (has_edits || clipboard != Clipboard::default()).then(|| CarriedDraft {
            preset_ref: self.current.get_untracked(),
            stored: self.stored.get_untracked(),
            draft: self.draft.get_untracked(),
            is_new: self.is_new.get_untracked(),
            has_edits,
            scene: self.scene.get_untracked(),
            clipboard,
        })
    }

    pub(crate) fn take_carried(self) -> Option<CarriedDraft> {
        let app = self.app?;
        app.carried.try_update_value(Option::take).flatten()
    }

    pub(crate) fn restore(self, carried: CarriedDraft, device: PresetRef) {
        if carried.clipboard != Clipboard::default() {
            self.clipboard.set(carried.clipboard.clone());
        }
        if !carried.has_edits {
            return;
        }
        let listed = self
            .library
            .with_untracked(|library| library.entry(device).cloned());
        let name = carried.draft.name().to_owned();
        match carried_fate(&carried, device, listed.as_ref()) {
            CarryFate::Restore => {
                self.draft.set(carried.draft);
                self.is_new.set(carried.is_new);
                self.scene.set(carried.scene);
                self.audition();
                self.notify(
                    format!("Unsaved edits to {name} were kept while the Anagram was away."),
                    false,
                );
            }
            CarryFate::ToClipboard => {
                self.clipboard.set(Clipboard::Presets(vec![document::export(
                    &carried.draft,
                    carried.scene,
                )]));
                self.notify(
                    format!(
                        "The Anagram came back on another preset. Unsaved edits to {name} are on the clipboard; paste them into a slot to keep them."
                    ),
                    true,
                );
            }
        }
    }

    pub(crate) fn announce_arrival(self) {
        let Some(app) = self.app else {
            return;
        };
        let Some(reconnect) = app.reconnect.try_update(Option::take).flatten() else {
            return;
        };
        let installed = self.backend().firmware_version();
        if let Some((text, is_error)) = reconnect.cause.arrival_notice(&installed) {
            self.notify(text, is_error);
        }
    }
}
