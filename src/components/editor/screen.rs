use std::rc::Rc;

use leptos::{ev, prelude::*};
use send_wrapper::SendWrapper;
use wasm_bindgen::JsCast;

use super::{
    bindings_panel::BindingsDialog,
    block_browser::BlockBrowser,
    block_settings::BlockSettings,
    chain_grid::{CHAIN_GRID_SELECTOR, ChainGrid},
    device_settings::DeviceSettingsDialog,
    header::Header,
    preset_list::PresetList,
    preset_settings::PresetSettingsPanel,
    user_files::UserFilesDialog,
};
use crate::{
    backend::Backend,
    browser::has_text_selection,
    components::ui::is_within,
    model::clipboard::Clipboard,
    session::{Session, SidePanel},
};

#[component]
pub fn EditorScreen(backend: SendWrapper<Rc<dyn Backend>>) -> impl IntoView {
    let session = Session::provide(backend.take());
    session.start();
    on_cleanup(move || {
        session.stop_polling();
        if let Some(app) = session.app {
            app.carried.set_value(session.carry());
        }
    });
    let is_modal_open = move || session.ui.modal.get().is_some();
    let shortcuts = window_event_listener(ev::keydown, move |event| {
        let is_ignored = untrack(is_modal_open) || is_typing(&event) || is_in_popup(&event);
        if !is_ignored
            && let Some(shortcut) = Shortcut::of(&event)
            && shortcut.run(&session, FocusPlace::of(&event))
        {
            event.prevent_default();
        }
    });
    on_cleanup(move || shortcuts.remove());
    let unload_guard = window_event_listener(ev::beforeunload, move |event| {
        let is_sending = session
            .files
            .upload_progress
            .with_untracked(Option::is_some);
        if session.has_unsaved_edits() || is_sending {
            event.prevent_default();
            event.set_return_value("Unsaved edits");
        }
    });
    on_cleanup(move || unload_guard.remove());

    view! {
        <div class="h-screen overflow-x-auto overflow-y-hidden">
            <div class="flex h-full min-w-[calc(21rem+21rem+29.5rem)] flex-col" inert=is_modal_open>
                <Header />
                <div class="grid min-h-0 flex-1 grid-cols-[21rem_minmax(0,1fr)_21rem]">
                    <PresetList />
                    <main class="flex min-h-0 min-w-0 flex-col">
                        <ChainGrid />
                        <BlockSettings />
                    </main>
                    {move || match session.ui.panel.get() {
                        SidePanel::Browser => view! { <BlockBrowser /> }.into_any(),
                        SidePanel::PresetSettings => view! { <PresetSettingsPanel /> }.into_any(),
                    }}
                </div>
            </div>
            <DeviceSettingsDialog />
            <UserFilesDialog />
            <BindingsDialog />
        </div>
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Shortcut {
    Undo,
    Redo,
    DeleteBlocks,
    CopyBlocks,
    PasteBlocks,
    Deselect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FocusPlace {
    Grid,
    Body,
    Elsewhere,
}

impl FocusPlace {
    fn of(event: &ev::KeyboardEvent) -> Self {
        let is_on_body = event
            .target()
            .and_then(|target| target.dyn_into::<web_sys::HtmlElement>().ok())
            .is_some_and(|element| element.tag_name() == "BODY");
        if is_within(event, CHAIN_GRID_SELECTOR) {
            Self::Grid
        } else if is_on_body {
            Self::Body
        } else {
            Self::Elsewhere
        }
    }
}

impl Shortcut {
    pub(super) fn of(event: &ev::KeyboardEvent) -> Option<Self> {
        Self::from_keys(
            &event.key(),
            event.ctrl_key(),
            event.meta_key(),
            event.shift_key(),
        )
    }

    fn from_keys(key: &str, ctrl: bool, meta: bool, shift: bool) -> Option<Self> {
        let key = key.to_ascii_lowercase();
        match (ctrl || meta, shift, key.as_str()) {
            (true, false, "z") => Some(Self::Undo),
            (true, true, "z") => Some(Self::Redo),
            (true, false, "y") if ctrl => Some(Self::Redo),
            (true, false, "c") => Some(Self::CopyBlocks),
            (true, false, "v") => Some(Self::PasteBlocks),
            (false, false, "delete" | "backspace") => Some(Self::DeleteBlocks),
            (false, false, "escape") => Some(Self::Deselect),
            _ => None,
        }
    }

    const fn reaches_selection(self, place: FocusPlace) -> bool {
        matches!(
            (self, place),
            (_, FocusPlace::Grid) | (Self::CopyBlocks | Self::Deselect, FocusPlace::Body)
        )
    }

    fn run(self, session: &Session, place: FocusPlace) -> bool {
        let selected = session
            .selected
            .get_untracked()
            .filter(|_| self.reaches_selection(place));
        let can_edit = selected.is_some() && !untrack(|| session.is_read_only());
        match self {
            Self::Undo => session.undo(),
            Self::Redo => session.redo(),
            Self::DeleteBlocks if can_edit => session.delete_blocks(selected),
            Self::CopyBlocks if selected.is_some() && !has_text_selection() => {
                session.copy_blocks(selected);
            }
            Self::PasteBlocks if can_edit && has_copied_blocks(session) => {
                if let Some(anchor) = selected {
                    session.paste_blocks(anchor);
                }
            }
            Self::Deselect if selected.is_some() => session.deselect(),
            Self::DeleteBlocks | Self::CopyBlocks | Self::PasteBlocks | Self::Deselect => {
                return false;
            }
        }
        true
    }
}

fn has_copied_blocks(session: &Session) -> bool {
    session
        .clipboard
        .with_untracked(|clipboard| matches!(clipboard, Clipboard::Blocks(_)))
}

fn is_in_popup(event: &ev::KeyboardEvent) -> bool {
    is_within(event, "[role=menu], [role=listbox], [aria-expanded=true]")
}

pub(super) fn is_typing(event: &ev::KeyboardEvent) -> bool {
    event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::HtmlElement>().ok())
        .is_some_and(|element| {
            let is_text_input =
                element
                    .dyn_ref::<web_sys::HtmlInputElement>()
                    .is_some_and(|input| {
                        matches!(
                            input.type_().as_str(),
                            "text" | "search" | "number" | "email" | "url" | "password" | "tel"
                        )
                    });
            is_text_input || element.tag_name() == "TEXTAREA" || element.is_content_editable()
        })
}

#[cfg(test)]
mod tests {
    use super::{FocusPlace, Shortcut};

    #[test]
    fn only_copy_and_deselect_reach_the_selection_from_the_body() {
        let reaches = |shortcut: Shortcut| {
            [FocusPlace::Grid, FocusPlace::Body, FocusPlace::Elsewhere]
                .map(|place| shortcut.reaches_selection(place))
        };
        assert_eq!(reaches(Shortcut::DeleteBlocks), [true, false, false]);
        assert_eq!(reaches(Shortcut::PasteBlocks), [true, false, false]);
        assert_eq!(reaches(Shortcut::CopyBlocks), [true, true, false]);
        assert_eq!(reaches(Shortcut::Deselect), [true, true, false]);
        assert_eq!(reaches(Shortcut::Undo), [true, false, false]);
    }

    fn keys(key: &str, ctrl: bool, meta: bool, shift: bool) -> Option<Shortcut> {
        Shortcut::from_keys(key, ctrl, meta, shift)
    }

    #[test]
    fn undo_and_redo_follow_platform_chords() {
        assert_eq!(keys("z", true, false, false), Some(Shortcut::Undo));
        assert_eq!(keys("z", false, true, false), Some(Shortcut::Undo));
        assert_eq!(keys("Z", false, true, true), Some(Shortcut::Redo));
        assert_eq!(keys("z", true, false, true), Some(Shortcut::Redo));
        assert_eq!(keys("y", true, false, false), Some(Shortcut::Redo));
        assert_eq!(keys("y", false, true, false), None);
        assert_eq!(keys("z", false, false, false), None);
    }

    #[test]
    fn block_shortcuts_need_exact_modifiers() {
        assert_eq!(keys("c", true, false, false), Some(Shortcut::CopyBlocks));
        assert_eq!(keys("v", false, true, false), Some(Shortcut::PasteBlocks));
        assert_eq!(
            keys("Delete", false, false, false),
            Some(Shortcut::DeleteBlocks)
        );
        assert_eq!(
            keys("Backspace", false, false, false),
            Some(Shortcut::DeleteBlocks)
        );
        assert_eq!(keys("Delete", false, false, true), None);
        assert_eq!(keys("Backspace", true, false, false), None);
        assert_eq!(
            keys("Escape", false, false, false),
            Some(Shortcut::Deselect)
        );
        assert_eq!(keys("Escape", false, false, true), None);
        assert_eq!(keys("c", true, false, true), None);
    }
}
