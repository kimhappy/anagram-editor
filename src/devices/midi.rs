use wasm_bindgen::JsCast;
use web_sys::{MidiAccess, MidiOptions, MidiOutput, MidiPort};

use super::{DeviceError, navigator, navigator_has};

pub const ANAGRAM_PORT_NAME: &str = "Anagram";

#[must_use]
pub fn is_supported() -> bool {
    navigator_has("requestMIDIAccess")
}

pub async fn request_access(sysex: bool) -> Result<MidiAccess, DeviceError> {
    if !is_supported() {
        return Err(DeviceError::Unsupported("Web MIDI"));
    }
    let options = MidiOptions::new();
    options.set_sysex(sysex);
    let access = navigator()
        .request_midi_access_with_options(&options)?
        .await?;
    Ok(access.unchecked_into())
}

#[must_use]
pub fn outputs(access: &MidiAccess) -> Vec<MidiOutput> {
    collect_ports(access.outputs().values())
}

#[must_use]
pub fn find_port<T: AsRef<MidiPort>>(ports: Vec<T>, name: &str) -> Option<T> {
    ports.into_iter().find(|port| {
        port.as_ref()
            .name()
            .is_some_and(|port_name| port_name.contains(name))
    })
}

pub fn send(output: &MidiOutput, message: &[u8]) -> Result<(), DeviceError> {
    output.send(&js_sys::Uint8Array::from(message))?;
    Ok(())
}

fn collect_ports<T: JsCast>(values: js_sys::Iterator) -> Vec<T> {
    values
        .into_iter()
        .filter_map(Result::ok)
        .map(JsCast::unchecked_into)
        .collect()
}
