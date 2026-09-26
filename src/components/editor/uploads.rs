use std::rc::Rc;

use leptos::{html, prelude::*};
use wasm_bindgen_futures::spawn_local;

use crate::{
    browser::prompt,
    components::icon::{Icon, IconKind},
    protocol::{
        hid::files::{UserDir, UserFile},
        name::USER_FILE_NAME,
    },
    session::Session,
};

#[must_use]
pub const fn accepted_extensions(dir: UserDir) -> &'static str {
    match dir {
        UserDir::Cabinets => ".wav",
        UserDir::NeuralModels => ".nam,.aidax,.json",
    }
}

#[must_use]
#[expect(clippy::float_arithmetic, reason = "a share shown as a percentage")]
fn percent_label(share: f64) -> String {
    format!("{:.0}%", share * 100.0)
}

fn chosen_file(input: &NodeRef<html::Input>) -> Option<web_sys::File> {
    input.get_untracked()?.files()?.get(0)
}

#[must_use]
pub fn stem_of(file_name: &str) -> String {
    file_name
        .rsplit_once('.')
        .map_or_else(|| file_name.to_owned(), |(stem, _)| stem.to_owned())
}

#[must_use]
pub fn sanitized_stem(file_name: &str) -> Option<String> {
    USER_FILE_NAME.accept(&stem_of(file_name))
}

#[must_use]
pub fn choose_name(file_name: &str) -> Option<String> {
    let suggested = sanitized_stem(file_name).unwrap_or_default();
    let typed = prompt("Name for this file on the Anagram", &suggested)?;
    USER_FILE_NAME.accept(&typed)
}

fn open_picker(input: NodeRef<html::Input>) {
    if let Some(input) = input.get_untracked() {
        input.set_value("");
        input.click();
    }
}

pub fn file_picker(
    accept: &'static str,
    on_file: impl Fn(web_sys::File) + 'static,
) -> (impl IntoView, impl Fn() + Copy + 'static) {
    let input_ref = NodeRef::<html::Input>::new();
    let input = view! {
        <input
            type="file"
            class="hidden"
            accept=accept
            node_ref=input_ref
            on:change=move |_| {
                if let Some(file) = chosen_file(&input_ref) {
                    on_file(file);
                }
            }
        />
    };
    (input, move || open_picker(input_ref))
}

pub fn upload(
    session_ref: &Session,
    dir: UserDir,
    name: String,
    file: web_sys::File,
    on_done: impl FnOnce(Option<UserFile>) + 'static,
) {
    let session = *session_ref;
    session.files.upload_progress.set(Some(0.0));
    spawn_local(async move {
        let Ok(buffer) = wasm_bindgen_futures::JsFuture::from(file.array_buffer()).await else {
            session.notify("The file could not be read.", true);
            session.files.upload_progress.set(None);
            return;
        };
        let bytes = js_sys::Uint8Array::new(&buffer).to_vec();
        session.upload_file(dir, name, file.name(), bytes, on_done);
    });
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UploadLook {
    #[default]
    Inline,
    Toolbar,
}

const INLINE_CLASS: &str = "text-muted-foreground hover:text-foreground flex h-7 shrink-0 items-center gap-1 rounded-md px-1 text-xs disabled:opacity-50";
const TOOLBAR_CLASS: &str = "border bg-background enabled:hover:bg-accent inline-flex h-8 items-center gap-1.5 rounded-md px-3 text-sm disabled:opacity-50";

#[component]
pub fn UploadButton(
    dir: UserDir,
    #[prop(optional)] look: UploadLook,
    #[prop(optional)] on_uploaded: Option<Rc<dyn Fn(UserFile)>>,
) -> impl IntoView {
    let session = Session::expect();
    let (picker, open_picker) = file_picker(accepted_extensions(dir), move |file| {
        if let Some(name) = choose_name(&file.name()) {
            let on_uploaded = on_uploaded.clone();
            upload(&session, dir, name, file, move |uploaded| {
                if let (Some(uploaded), Some(on_uploaded)) = (uploaded, on_uploaded) {
                    on_uploaded(uploaded);
                }
            });
        }
    });
    let on_click = move |_| {
        if session.backend().has_serial() {
            open_picker();
        } else {
            spawn_local(async move {
                match session.backend().ensure_serial().await {
                    Ok(()) => session.notify(
                        "The Anagram's file port is allowed. Choose Upload again to pick the file.",
                        false,
                    ),
                    Err(error) => {
                        session.notify(format!("The file port was not allowed: {error}"), true);
                    }
                }
            });
        }
    };
    let (class, text) = match look {
        UploadLook::Inline => (INLINE_CLASS, None),
        UploadLook::Toolbar => (TOOLBAR_CLASS, Some("Upload")),
    };
    view! {
        {picker}
        <button
            type="button"
            class=class
            title="Upload a file to the Anagram"
            aria-label=format!("Upload a file to {}", dir.name())
            disabled=move || !session.is_idle()
            on:click=on_click
        >
            <UploadLabel text=text.unwrap_or_default() />
        </button>
    }
}

#[component]
pub fn UploadLabel(#[prop(optional)] text: &'static str) -> impl IntoView {
    view! {
        <UploadProgress>
            <Icon kind=IconKind::Upload />
            {(!text.is_empty()).then_some(text)}
        </UploadProgress>
    }
}

#[component]
fn UploadProgress(children: ChildrenFn) -> impl IntoView {
    let session = Session::expect();
    move || {
        session.files.upload_progress.get().map_or_else(
            || children().into_any(),
            |share| {
                view! {
                    <Icon kind=IconKind::LoaderCircle class="animate-spin" />
                    <span class="tabular-nums">{percent_label(share)}</span>
                }
                .into_any()
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{percent_label, sanitized_stem, stem_of};

    #[test]
    fn stem_of_drops_only_the_last_extension() {
        assert_eq!(stem_of("Room.wav"), "Room");
        assert_eq!(stem_of("amp.v2.nam"), "amp.v2");
        assert_eq!(stem_of("no extension"), "no extension");
        assert_eq!(stem_of(".wav"), "");
        assert_eq!(stem_of(""), "");
    }

    #[test]
    fn sanitized_stem_rejects_names_that_sanitize_to_nothing() {
        assert_eq!(
            sanitized_stem("My IR / v2.1.wav"),
            Some("My IR  v21".to_owned())
        );
        assert_eq!(sanitized_stem("\u{d55c}\u{ae00}.wav"), None);
        assert_eq!(sanitized_stem(".wav"), None);
        assert_eq!(sanitized_stem("  .json"), None);
    }

    #[test]
    fn percent_label_rounds_to_whole_percent() {
        assert_eq!(percent_label(0.0), "0%");
        assert_eq!(percent_label(0.456), "46%");
        assert_eq!(percent_label(1.0), "100%");
    }
}
