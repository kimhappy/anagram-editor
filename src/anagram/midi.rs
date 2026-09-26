use std::cell::Cell;

use web_sys::{MidiAccess, MidiOutput};

use crate::{
    devices::{DeviceError, midi},
    protocol::{
        hid::settings::MidiSettings,
        midi::{Channel, Command},
    },
};

pub struct MidiLink {
    output: MidiOutput,
    channel: Cell<Channel>,
}

impl MidiLink {
    #[must_use]
    pub fn connect(access: &MidiAccess) -> Option<Self> {
        let output = midi::find_port(midi::outputs(access), midi::ANAGRAM_PORT_NAME)?;
        Some(Self {
            output,
            channel: Cell::new(Channel::FIRST),
        })
    }

    pub fn apply(&self, settings: &MidiSettings) {
        self.channel.set(settings.send_channel());
    }

    pub fn send(&self, command: Command) -> Result<(), DeviceError> {
        midi::send(&self.output, &command.encode(self.channel.get()))
    }
}
