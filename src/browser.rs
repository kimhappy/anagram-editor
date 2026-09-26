use wasm_bindgen::{JsCast, JsValue};

#[must_use]
pub fn stored_text(key: &str) -> Option<String> {
    web_sys::window()?
        .local_storage()
        .ok()??
        .get_item(key)
        .ok()?
}

pub fn store_text(key: &str, value: &str) {
    let stored = web_sys::window()
        .and_then(|window| window.local_storage().ok().flatten())
        .is_some_and(|storage| storage.set_item(key, value).is_ok());
    if !stored {
        web_sys::console::warn_1(&format!("Could not keep {key} in local storage.").into());
    }
}

#[must_use]
pub fn prompt(message: &str, default: &str) -> Option<String> {
    web_sys::window()?
        .prompt_with_message_and_default(message, default)
        .ok()
        .flatten()
}

#[must_use]
pub fn has_text_selection() -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    let property =
        |target: &JsValue, name: &str| js_sys::Reflect::get(target, &JsValue::from_str(name)).ok();
    property(&window, "getSelection")
        .and_then(|method| method.dyn_into::<js_sys::Function>().ok())
        .and_then(|method| method.call0(&window).ok())
        .filter(|selection| !selection.is_null() && !selection.is_undefined())
        .and_then(|selection| property(&selection, "isCollapsed"))
        .is_some_and(|is_collapsed| is_collapsed.as_bool() == Some(false))
}
