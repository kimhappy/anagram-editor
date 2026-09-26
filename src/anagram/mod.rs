pub mod backend;
pub mod catalog;
pub mod hid;
pub mod midi;
pub mod transfer;

use std::{cell::RefCell, fmt, sync::Arc, time::Duration};

use web_sys::SerialPort;

use self::{hid::HidClient, midi::MidiLink};
use crate::{
    backend::DeviceSnapshot,
    devices::{self, DeviceError},
    model::catalog::Catalog,
    protocol::hid::{
        action::{ExportSettings, FetchFirmwareVersion, FirmwareVersion},
        envelope::HidError,
        version::Version,
    },
};

const FIRST_CALL_DELAY: Duration = Duration::from_secs(1);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConnectStep {
    RequestingHid,
    ReadingFirmware,
    RequestingMidi,
    LoadingCatalog { done: usize, total: usize },
    ReadingSettings,
}

impl ConnectStep {
    #[must_use]
    #[expect(
        clippy::cast_precision_loss,
        clippy::float_arithmetic,
        reason = "a count of plugins as a share"
    )]
    pub fn share(&self) -> Option<f64> {
        match self {
            Self::LoadingCatalog { done, total } if *total > 0 => {
                Some(*done as f64 / *total as f64)
            }
            Self::RequestingHid
            | Self::ReadingFirmware
            | Self::RequestingMidi
            | Self::LoadingCatalog { .. }
            | Self::ReadingSettings => None,
        }
    }
}

impl fmt::Display for ConnectStep {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RequestingHid => formatter.write_str("Waiting for the device chooser"),
            Self::ReadingFirmware => formatter.write_str("Reading the firmware version"),
            Self::RequestingMidi => formatter.write_str("Opening the USB MIDI port"),
            Self::LoadingCatalog { done, total } => {
                write!(formatter, "Loading blocks ({done} / {total})")
            }
            Self::ReadingSettings => formatter.write_str("Reading the device state"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConnectError {
    OpenInAnotherTab,
    NoDevice,
    OpenFailed(DeviceError),
    Hid(HidError),
    FirmwareTooOld(Version),
    MidiPortMissing,
    Browser(DeviceError),
}

impl fmt::Display for ConnectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OpenInAnotherTab => formatter.write_str(
                "The editor is already connected in another tab. Close that tab or disconnect it first.",
            ),
            Self::NoDevice => formatter.write_str("No Anagram was chosen."),
            Self::OpenFailed(error) => {
                write!(formatter, "{error} Close other programs using the Anagram.")
            }
            Self::Hid(error) => write!(formatter, "The device did not answer: {error}"),
            Self::FirmwareTooOld(version) => write!(
                formatter,
                "Firmware {version} is too old. The editor needs 1.18 or newer; update the Anagram with the Darkglass Suite first."
            ),
            Self::MidiPortMissing => formatter.write_str(
                "The Anagram MIDI port was not found. Turn on USB MIDI in the device's MIDI settings and allow MIDI access in the browser.",
            ),
            Self::Browser(error) => write!(formatter, "{error}"),
        }
    }
}

impl ConnectError {
    #[must_use]
    pub const fn is_retryable(&self) -> bool {
        !matches!(self, Self::OpenInAnotherTab | Self::FirmwareTooOld(_))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceChoice {
    AskIfNeeded,
    GrantedOnly,
}

impl From<HidError> for ConnectError {
    fn from(error: HidError) -> Self {
        Self::Hid(error)
    }
}

impl From<DeviceError> for ConnectError {
    fn from(error: DeviceError) -> Self {
        Self::Browser(error)
    }
}

async fn granted_anagram() -> Result<Option<web_sys::HidDevice>, DeviceError> {
    let filter = devices::hid::anagram::FILTER;
    Ok(devices::hid::granted_devices()
        .await?
        .into_iter()
        .find(|device| {
            device.vendor_id() == filter.vendor_id
                && filter.product_id.is_none_or(|id| device.product_id() == id)
        }))
}

pub struct Anagram {
    hid: HidClient,
    midi: MidiLink,
    firmware: FirmwareVersion,
    catalog: Arc<Catalog>,
    snapshot: RefCell<DeviceSnapshot>,
    serial: RefCell<Option<SerialPort>>,
}

impl Anagram {
    pub async fn connect(
        choice: DeviceChoice,
        progress: impl FnMut(ConnectStep),
    ) -> Result<Self, ConnectError> {
        if !devices::tab_lock::claim().await? {
            return Err(ConnectError::OpenInAnotherTab);
        }
        let connected = Self::open(choice, progress).await;
        if connected.is_err() && choice == DeviceChoice::AskIfNeeded {
            devices::tab_lock::release();
        }
        connected
    }

    async fn open(
        choice: DeviceChoice,
        mut progress: impl FnMut(ConnectStep),
    ) -> Result<Self, ConnectError> {
        progress(ConnectStep::RequestingHid);
        let device = match granted_anagram().await? {
            Some(device) => device,
            None if choice == DeviceChoice::GrantedOnly => return Err(ConnectError::NoDevice),
            None => devices::hid::request_devices(&[devices::hid::anagram::FILTER])
                .await?
                .into_iter()
                .next()
                .ok_or(ConnectError::NoDevice)?,
        };
        let hid = HidClient::open(device).await.map_err(|error| match error {
            HidError::Transport(cause) => ConnectError::OpenFailed(cause),
            other => ConnectError::Hid(other),
        })?;

        progress(ConnectStep::ReadingFirmware);
        gloo_timers::future::sleep(FIRST_CALL_DELAY).await;
        let firmware = hid.call(FetchFirmwareVersion {}).await?;
        if !firmware.version.is_supported() {
            return Err(ConnectError::FirmwareTooOld(firmware.version));
        }

        progress(ConnectStep::RequestingMidi);
        let access = devices::midi::request_access(false).await?;
        let midi = MidiLink::connect(&access).ok_or(ConnectError::MidiPortMissing)?;

        let catalog = catalog::load(&hid, &firmware.version, |done, total| {
            progress(ConnectStep::LoadingCatalog { done, total });
        })
        .await?;

        progress(ConnectStep::ReadingSettings);
        let exported = hid.call(ExportSettings {}).await?;
        let snapshot = DeviceSnapshot::from_settings(&exported.settings);
        midi.apply(&snapshot.settings.midi);

        let serial = devices::serial::granted_port(devices::serial::anagram::FILTER)
            .await
            .ok()
            .flatten();
        Ok(Self {
            hid,
            midi,
            firmware,
            catalog: Arc::new(catalog),
            snapshot: RefCell::new(snapshot),
            serial: RefCell::new(serial),
        })
    }
}
