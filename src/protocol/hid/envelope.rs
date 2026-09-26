use std::fmt;

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Map, Value};

use crate::devices::DeviceError;

#[derive(Serialize)]
pub struct Request<'action, P> {
    pub action: &'action str,
    pub payload: P,
}

#[derive(Deserialize)]
pub struct Reply {
    #[serde(default)]
    pub action: Option<String>,
    pub payload: Value,
}

#[derive(Debug, Deserialize)]
pub struct Status {
    #[serde(default)]
    pub success: Option<i64>,
    #[serde(default)]
    pub err_message: Option<String>,
}

impl Status {
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.success.map_or_else(
            || self.err_message.as_deref().is_none_or(str::is_empty),
            |code| code == 1,
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HidError {
    Timeout(&'static str),
    Device {
        action: &'static str,
        message: String,
    },
    Malformed {
        action: &'static str,
        detail: String,
    },
    Transport(DeviceError),
    Encoding(String),
}

impl fmt::Display for HidError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Timeout(action) => write!(formatter, "{action}: no reply from the device"),
            Self::Device { action, message } => write!(formatter, "{action}: {message}"),
            Self::Malformed { action, detail } => {
                write!(formatter, "{action}: unexpected reply ({detail})")
            }
            Self::Transport(error) => write!(formatter, "{error}"),
            Self::Encoding(detail) => write!(formatter, "request could not be encoded: {detail}"),
        }
    }
}

impl std::error::Error for HidError {}

impl From<DeviceError> for HidError {
    fn from(error: DeviceError) -> Self {
        Self::Transport(error)
    }
}

pub fn decode_payload<D: DeserializeOwned>(
    action: &'static str,
    text: &str,
    data_key: Option<&str>,
    checks_status: bool,
) -> Result<D, HidError> {
    let malformed = |detail: String| HidError::Malformed { action, detail };
    let reply: Reply = serde_json::from_str(text).map_err(|error| malformed(error.to_string()))?;
    let status: Status = serde_json::from_value(reply.payload.clone())
        .map_err(|error| malformed(error.to_string()))?;
    if checks_status && !status.is_success() {
        return Err(HidError::Device {
            action,
            message: status
                .err_message
                .filter(|message| !message.is_empty())
                .unwrap_or_else(|| "device reported failure".to_owned()),
        });
    }
    let value = match data_key {
        Some(key) => reply.payload.get(key).cloned().unwrap_or(Value::Null),
        None => reply.payload,
    };
    if value.is_null() {
        return [Value::Object(Map::new()), Value::Array(Vec::new())]
            .into_iter()
            .find_map(|empty| serde_json::from_value(empty).ok())
            .ok_or_else(|| malformed("reply carries no data".to_owned()));
    }
    serde_json::from_value(value).map_err(|error| malformed(error.to_string()))
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::{HidError, decode_payload};

    #[derive(Debug, Deserialize, PartialEq)]
    struct Version {
        version: String,
    }

    #[test]
    fn data_is_read_from_the_data_member() {
        let text = r#"{"action":"x_res","payload":{"success":1,"err_message":"","data":{"version":"v1.18.0\n"}}}"#;
        let parsed: Version = decode_payload("x", text, Some("data"), true).expect("parsed");
        assert_eq!(parsed.version, "v1.18.0\n");
    }

    #[test]
    fn a_reply_without_success_is_read_when_err_message_is_empty() {
        let text = r#"{"action":"fetch_firmware_version_res","payload":{"data":{"version":"v1.18.0.18\n"},"err_message":""}}"#;
        let parsed: Version =
            decode_payload("fetch_firmware_version", text, Some("data"), true).expect("parsed");
        assert_eq!(parsed.version, "v1.18.0.18\n");
    }

    #[test]
    fn a_reply_without_success_fails_on_an_err_message() {
        let text = r#"{"payload":{"err_message":"boom"}}"#;
        let result: Result<Version, HidError> = decode_payload("x", text, Some("data"), true);
        assert_eq!(
            result,
            Err(HidError::Device {
                action: "x",
                message: "boom".to_owned()
            })
        );
    }

    #[test]
    fn missing_data_reads_as_an_empty_collection_or_fails() {
        let text = r#"{"payload":{"success":1}}"#;
        let list: Vec<u8> = decode_payload("x", text, Some("data"), true).expect("empty list");
        assert_eq!(list, Vec::<u8>::new());
        let result: Result<Version, HidError> = decode_payload("x", text, Some("data"), true);
        assert_eq!(
            result,
            Err(HidError::Malformed {
                action: "x",
                detail: "reply carries no data".to_owned()
            })
        );
    }

    #[test]
    fn failure_becomes_a_device_error() {
        let text = r#"{"payload":{"success":0,"err_message":"No plugin available"}}"#;
        let result: Result<Version, HidError> =
            decode_payload("get_plugin_info", text, Some("data"), true);
        assert_eq!(
            result,
            Err(HidError::Device {
                action: "get_plugin_info",
                message: "No plugin available".to_owned()
            })
        );
    }
}
