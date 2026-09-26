use leptos::prelude::*;

use crate::{model::history::History, protocol::hid::settings::DeviceSettings};

#[derive(Clone, Copy)]
pub struct SettingsDraft {
    current: RwSignal<DeviceSettings>,
    gesture_start: StoredValue<Option<DeviceSettings>>,
    history: StoredValue<History<DeviceSettings>>,
}

impl SettingsDraft {
    pub fn new() -> Self {
        Self {
            current: RwSignal::new(DeviceSettings::default()),
            gesture_start: StoredValue::new(None),
            history: StoredValue::new(History::default()),
        }
    }

    pub fn start_over(self, settings: DeviceSettings) {
        self.gesture_start.set_value(None);
        self.history.set_value(History::default());
        self.current.set(settings);
    }

    pub fn replace(self, settings: DeviceSettings) {
        self.current.set(settings);
    }

    pub fn with<T>(self, read: impl FnOnce(&DeviceSettings) -> T) -> T {
        self.current.with(read)
    }

    pub fn with_untracked<T>(self, read: impl FnOnce(&DeviceSettings) -> T) -> T {
        self.current.with_untracked(read)
    }

    pub fn get_untracked(self) -> DeviceSettings {
        self.current.get_untracked()
    }

    pub fn update(self, change: impl FnOnce(&mut DeviceSettings)) {
        if self.gesture_start.with_value(Option::is_some) {
            self.current.update(change);
            return;
        }
        let before = self.current.get_untracked();
        self.current.update(change);
        self.record(before);
    }

    pub fn begin_gesture(self) {
        if self.gesture_start.with_value(Option::is_none) {
            self.gesture_start
                .set_value(Some(self.current.get_untracked()));
        }
    }

    pub fn end_gesture(self) {
        if let Some(before) = self.gesture_start.try_update_value(Option::take).flatten() {
            self.record(before);
        }
    }

    fn record(self, before: DeviceSettings) {
        self.current.with_untracked(|after| {
            self.history
                .update_value(|history| history.record(before, after));
        });
    }

    pub fn undo(self) {
        self.step(History::undo);
    }

    pub fn redo(self) {
        self.step(History::redo);
    }

    fn step(
        self,
        walk: fn(&mut History<DeviceSettings>, DeviceSettings) -> Option<DeviceSettings>,
    ) {
        self.end_gesture();
        let current = self.current.get_untracked();
        if let Some(restored) = self
            .history
            .try_update_value(|history| walk(history, current))
            .flatten()
        {
            self.current.set(restored);
        }
    }
}
