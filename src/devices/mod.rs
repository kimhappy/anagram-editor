pub mod hid;
pub mod midi;
pub mod serial;
pub mod tab_lock;

pub mod anagram {
    pub const USB_VENDOR_ID: u16 = 0x2FA6;
    pub const USB_PRODUCT_ID: u16 = 0x2500;
}

use std::{fmt, future::Future, pin::pin, task::Poll, time::Duration};

use gloo_timers::future::TimeoutFuture;
use wasm_bindgen::{JsCast, JsValue};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeviceError {
    Unsupported(&'static str),
    Browser(String),
}

impl From<JsValue> for DeviceError {
    fn from(value: JsValue) -> Self {
        let message = value
            .dyn_ref::<js_sys::Error>()
            .map(|error| String::from(error.message()))
            .or_else(|| value.as_string())
            .unwrap_or_else(|| format!("{value:?}"));
        Self::Browser(message)
    }
}

impl fmt::Display for DeviceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported(api) => write!(formatter, "{api} is not available in this browser"),
            Self::Browser(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for DeviceError {}

fn navigator() -> web_sys::Navigator {
    web_sys::window()
        .expect("window must exist in a browser context")
        .navigator()
}

#[must_use]
pub fn is_linux() -> bool {
    let agent = navigator().user_agent().unwrap_or_default();
    agent.contains("Linux") && !agent.contains("Android")
}

fn navigator_has(property: &str) -> bool {
    js_sys::Reflect::has(&navigator(), &JsValue::from_str(property)).unwrap_or(false)
}

pub async fn with_timeout<T>(future: impl Future<Output = T>, timeout: Duration) -> Option<T> {
    let mut future = pin!(future);
    let millis = u32::try_from(timeout.as_millis()).unwrap_or(u32::MAX);
    let mut timer = pin!(TimeoutFuture::new(millis));
    std::future::poll_fn(|context| {
        if let Poll::Ready(value) = future.as_mut().poll(context) {
            return Poll::Ready(Some(value));
        }
        if timer.as_mut().poll(context).is_ready() {
            return Poll::Ready(None);
        }
        Poll::Pending
    })
    .await
}
