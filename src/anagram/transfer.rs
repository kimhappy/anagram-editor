use std::time::Duration;

use web_sys::SerialPort;

use super::hid::HidClient;
use crate::{
    devices::{self, DeviceError},
    protocol::{
        fios::{COMMAND_SIZE, Sender, Step, is_ack},
        hid::{
            action::{Action, OpenFileReceiver, ReceiverOpened},
            envelope::HidError,
        },
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransferError {
    Hid(HidError),
    Serial(DeviceError),
    NotReady(String),
    Rejected,
}

const ACK_TIMEOUT: Duration = Duration::from_secs(10);

impl std::fmt::Display for TransferError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hid(error) => write!(formatter, "{error}"),
            Self::Serial(error) => write!(formatter, "serial transfer failed: {error}"),
            Self::NotReady(reason) => {
                write!(
                    formatter,
                    "the Anagram was not ready to receive the file: {reason}"
                )
            }
            Self::Rejected => formatter.write_str("the Anagram did not acknowledge a block"),
        }
    }
}

impl std::error::Error for TransferError {}

pub async fn upload<R>(
    hid: &HidClient,
    port: &SerialPort,
    record: R,
    bytes: &[u8],
    mut progress: impl FnMut(f64),
) -> Result<ReceiverOpened, TransferError>
where
    OpenFileReceiver<R>: Action<Data = ReceiverOpened>,
{
    let opened = hid
        .call(OpenFileReceiver(record))
        .await
        .map_err(|error| match error {
            HidError::Device { message, .. } => TransferError::NotReady(message),
            other => TransferError::Hid(other),
        })?;
    devices::serial::open(port, devices::serial::anagram::BAUD_RATE)
        .await
        .map_err(TransferError::Serial)?;
    let outcome = send_all(port, bytes, &mut progress).await;
    devices::serial::close(port).await.unwrap_or_default();
    outcome.map(|()| opened)
}

#[expect(
    clippy::cast_precision_loss,
    clippy::float_arithmetic,
    reason = "progress is a share of the byte count"
)]
async fn send_all(
    port: &SerialPort,
    bytes: &[u8],
    progress: &mut impl FnMut(f64),
) -> Result<(), TransferError> {
    let total = bytes.len().max(1) as f64;
    let mut sender = Sender::new(bytes);
    while let Some(step) = sender.next() {
        match step {
            Step::Write(chunk) => devices::serial::write(port, &chunk)
                .await
                .map_err(TransferError::Serial)?,
            Step::ReadAck => {
                let reply = devices::serial::read_exact(port, COMMAND_SIZE, ACK_TIMEOUT)
                    .await
                    .map_err(TransferError::Serial)?;
                if !is_ack(&reply) {
                    return Err(TransferError::Rejected);
                }
                progress(sender.sent() as f64 / total);
            }
        }
    }
    Ok(())
}
