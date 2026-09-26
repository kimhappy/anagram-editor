#![cfg(target_os = "linux")]
#![expect(
    clippy::tests_outside_test_module,
    reason = "device tests fail loudly and run only when asked for"
)]

use std::{
    fs::{self, File, OpenOptions},
    io::{ErrorKind, Read, Write},
    os::unix::fs::OpenOptionsExt,
    sync::{Mutex, MutexGuard},
    thread,
    time::{Duration, Instant},
};

use anagram_editor::protocol::hid::{
    action::{
        Action, ExportSettings, FetchFirmwareVersion, GetPresetPositions, ReadAllPlugins,
        decode_reply, encode_request, parse_preset_filename,
    },
    framing::{REPORT_SIZE, Reassembler, frame_request},
};

const ANAGRAM_ID: &str = "00002FA6:00002500";
const REPORT_GAP: Duration = Duration::from_millis(10);
const POLL_GAP: Duration = Duration::from_millis(5);

static DEVICE: Mutex<()> = Mutex::new(());

struct Anagram<'lock> {
    node: File,
    _exclusive: MutexGuard<'lock, ()>,
}

impl Anagram<'_> {
    fn open() -> Self {
        let exclusive = DEVICE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let name = fs::read_dir("/sys/class/hidraw")
            .expect("hidraw class is readable")
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .find(|name| {
                fs::read_to_string(format!("/sys/class/hidraw/{name}/device/uevent"))
                    .is_ok_and(|uevent| uevent.contains(ANAGRAM_ID))
            })
            .expect("an Anagram is plugged in over USB");
        let node = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(format!("/dev/{name}"))
            .expect("the hidraw node is accessible; see the udev rule in README.md");
        Self {
            node,
            _exclusive: exclusive,
        }
    }

    fn call<A: Action>(&mut self, action: &A) -> Result<A::Data, String> {
        let request = encode_request(action).map_err(|error| error.to_string())?;
        for report in frame_request(&request) {
            self.node
                .write_all(&report)
                .map_err(|error| format!("writing a report failed: {error}"))?;
            thread::sleep(REPORT_GAP);
        }
        let mut reassembler = Reassembler::default();
        let deadline = Instant::now() + A::TIMEOUT;
        let mut report = [0; REPORT_SIZE];
        while Instant::now() < deadline {
            match self.node.read(&mut report) {
                Ok(0) => thread::sleep(POLL_GAP),
                Ok(read) => {
                    if let Some(document) =
                        report.get(..read).and_then(|chunk| reassembler.push(chunk))
                    {
                        return decode_reply::<A>(&document).map_err(|error| error.to_string());
                    }
                }
                Err(error) if error.kind() == ErrorKind::WouldBlock => thread::sleep(POLL_GAP),
                Err(error) => return Err(format!("reading the hidraw node failed: {error}")),
            }
        }
        Err(format!("{} received no reply", A::NAME))
    }
}

#[test]
#[ignore = "needs an Anagram over USB"]
fn firmware_is_supported() {
    let firmware = Anagram::open()
        .call(&FetchFirmwareVersion {})
        .expect("the firmware version is read");
    assert!(
        firmware.version.is_supported(),
        "firmware {} is older than 1.18",
        firmware.version
    );
}

#[test]
#[ignore = "needs an Anagram over USB"]
fn settings_name_the_selected_preset() {
    let exported = Anagram::open()
        .call(&ExportSettings {})
        .expect("the settings are exported");
    let state = exported.settings.device_state();
    assert!(state.preset_index < 126);
}

#[test]
#[ignore = "needs an Anagram over USB"]
fn preset_positions_use_slot_filenames() {
    let positions = Anagram::open()
        .call(&GetPresetPositions { user_area: None })
        .expect("the preset positions are listed")
        .into_positions()
        .expect("the listing succeeded");
    for position in &positions.files {
        assert!(
            parse_preset_filename(&position.filename).is_some(),
            "{} is not a slot filename",
            position.filename
        );
    }
}

#[test]
#[ignore = "needs an Anagram over USB"]
fn every_plugin_is_listed_with_a_uri() {
    let plugins = Anagram::open()
        .call(&ReadAllPlugins { filter: None })
        .expect("the plugins are listed");
    assert!(plugins.len() >= 100);
    assert!(plugins.iter().all(|plugin| !plugin.uri.is_empty()));
}
