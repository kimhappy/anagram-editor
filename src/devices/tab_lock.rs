use std::cell::RefCell;

use wasm_bindgen::{JsCast, JsValue, prelude::wasm_bindgen};
use wasm_bindgen_futures::JsFuture;

use super::DeviceError;

const LOCK_NAME: &str = "anagram-editor-device";

#[wasm_bindgen(inline_js = "
export function request_exclusive(name) {
  if (!navigator.locks) {
    return Promise.resolve(() => {});
  }
  return new Promise((granted) => {
    navigator.locks.request(name, { ifAvailable: true }, (lock) => {
      if (!lock) {
        granted(null);
        return undefined;
      }
      return new Promise((release) => granted(release));
    });
  });
}
")]
extern "C" {
    fn request_exclusive(name: &str) -> js_sys::Promise;
}

thread_local! {
    static RELEASE: RefCell<Option<js_sys::Function>> = const { RefCell::new(None) };
}

pub async fn claim() -> Result<bool, DeviceError> {
    if RELEASE.with_borrow(Option::is_some) {
        return Ok(true);
    }
    let granted = JsFuture::from(request_exclusive(LOCK_NAME)).await?;
    let Some(release) = granted.dyn_ref::<js_sys::Function>() else {
        return Ok(false);
    };
    RELEASE.set(Some(release.clone()));
    Ok(true)
}

pub fn release() {
    if let Some(release) = RELEASE.take() {
        release.call0(&JsValue::NULL).unwrap_or_default();
    }
}
