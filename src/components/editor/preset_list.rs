use std::collections::BTreeSet;

use leptos::{
    ev::{DragEvent, MouseEvent},
    prelude::*,
};
use tw_merge::tw_merge;

use super::side_panel::{
    EMPTY_CLASS, KEY_CLASS, SIDEBAR_ROW_HOVER_CLASS, SIDEBAR_ROW_LAYOUT_CLASS,
    SIDEBAR_ROW_SURFACE_CLASS, SearchField, matches_needle, search_needle,
};
use crate::{
    components::{
        icon::{Icon, IconKind},
        ui::{ContextMenu, MenuItem, MenuState, Tabs},
    },
    model::{clipboard::Clipboard, library::PresetRef, slot::Slot},
    protocol::hid::area::PresetArea,
    session::Session,
};

const PRESET_ROW_CLASS: &str = "data-[picked=true]:bg-sidebar-accent data-[picked=true]:text-sidebar-accent-foreground data-[over=true]:ring-2 data-[over=true]:ring-ring group has-focus-visible:ring-2 has-focus-visible:ring-ring";

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Row {
    area: PresetArea,
    slot: Slot,
    name: Option<String>,
    error: Option<String>,
}

#[component]
pub fn PresetList() -> impl IntoView {
    let session = Session::expect();
    let query = RwSignal::new(String::new());
    let needle = search_needle(query);
    let dragged = RwSignal::new(None::<Slot>);
    let over = RwSignal::new(None::<Slot>);

    let areas = Signal::derive(move || {
        session.library.with(|library| {
            library
                .areas()
                .iter()
                .map(|info| (info.area, info.label.clone()))
                .collect::<Vec<_>>()
        })
    });
    let area = Signal::derive(move || session.browsed_area.get());
    let is_listed = Memo::new(move |_| {
        let area = area.get();
        session.library.with(|library| library.is_listed(area))
    });
    let rows = Memo::new(move |_| {
        let needle = needle.get();
        let area = area.get();
        session.library.with(|library| {
            let Some(index) = library.index(area) else {
                return Vec::new();
            };
            index
                .rows()
                .map(|slot| {
                    let entry = index.get(slot);
                    Row {
                        area,
                        slot,
                        name: entry.map(|entry| entry.name.clone()),
                        error: entry.and_then(|entry| entry.error.clone()),
                    }
                })
                .filter(|row| {
                    needle.is_empty()
                        || matches_needle(&row.slot.to_string(), &needle)
                        || row
                            .name
                            .as_ref()
                            .is_some_and(|name| matches_needle(name, &needle))
                })
                .collect::<Vec<_>>()
        })
    });
    let drag = DragState { dragged, over };
    let menu = MenuState::new();
    let menu_items = Signal::derive(move || menu_items(&session, menu.target.get()));
    session.on_preset_change(move || {
        menu.close();
        dragged.set(None);
        over.set(None);
    });

    view! {
        <aside class="bg-sidebar text-sidebar-foreground flex min-h-0 flex-col border-r">
            <div class="grid shrink-0 border-b">
                <SearchField query label="Search presets" />
                <Tabs
                    options=areas
                    value=area
                    on_change=move |picked: PresetArea| session.browse_area(picked)
                    class="w-full"
                />
            </div>
            <Show when=move || !area.get().is_writable()>
                <p class="text-muted-foreground border-b px-4 py-2 text-xs">
                    "Presets of this area cannot be read over USB. Picking one switches the Anagram to it."
                </p>
            </Show>
            <ul class="grid min-h-0 flex-1 grid-cols-1 content-start overflow-y-auto">
                <For
                    each=move || rows.get()
                    key=|row| row.clone()
                    children=move |row| view! { <PresetRow row drag menu /> }
                />
                <Show when=move || is_listed.get() && rows.with(Vec::is_empty)>
                    <li class=EMPTY_CLASS>"No presets found."</li>
                </Show>
                <Show when=move || !is_listed.get()>
                    <UnlistedArea area />
                </Show>
            </ul>
            <ContextMenu anchor=menu.anchor items=menu_items />
        </aside>
    }
}

#[component]
fn UnlistedArea(area: Signal<PresetArea>) -> impl IntoView {
    let session = Session::expect();
    let is_listing = Memo::new(move |_| {
        let area = area.get();
        session.link.listing.with(|listing| listing.contains(&area))
    });
    view! {
    <li class=tw_merge!(EMPTY_CLASS, "flex items-center justify-between gap-2")>
                {move || if is_listing.get() { "Reading the presets of this area…" } else { "The presets of this area could not be listed." }}
                <button
                    type="button"
                    class="hover:bg-sidebar-accent rounded-md px-2 py-1 text-xs not-italic disabled:opacity-50"
                    disabled=move || is_listing.get()
                    on:click=move |_| session.browse_area(area.get_untracked())
                >
                    "Retry"
                </button>
            </li>
        }
}

fn menu_items(session_ref: &Session, slot: Option<Slot>) -> Vec<MenuItem> {
    let session = *session_ref;
    let area = session.current.get().area;
    let is_saved = |candidate: Slot| {
        session.library.with(|library| {
            library
                .entry(PresetRef {
                    area,
                    slot: candidate,
                })
                .is_some()
        })
    };
    let picked: Vec<Slot> = session.picked.with(|picked| {
        picked
            .iter()
            .copied()
            .filter(|candidate| is_saved(*candidate))
            .collect()
    });
    let is_read_only = session.is_read_only() || session.browsed_area.get() != area;
    let can_copy = !is_read_only && !picked.is_empty();
    let can_paste =
        slot.is_some() && !is_read_only && session.clipboard.with(Clipboard::has_presets);
    let count = picked.len();
    let doomed: Vec<Slot> = if count > 1 && slot.is_some_and(|target| picked.contains(&target)) {
        picked.clone()
    } else {
        slot.filter(|target| is_saved(*target))
            .into_iter()
            .collect()
    };
    let can_delete = !is_read_only && !doomed.is_empty();
    vec![
        MenuItem {
            label: if count > 1 { "Copy presets" } else { "Copy" },
            icon: IconKind::Copy,
            disabled: !can_copy,
            action: Callback::new(move |()| session.copy_presets(picked.clone())),
        },
        MenuItem {
            label: "Paste here",
            icon: IconKind::ClipboardPaste,
            disabled: !can_paste,
            action: Callback::new(move |()| {
                if let Some(target) = slot {
                    session.paste_presets(target);
                }
            }),
        },
        MenuItem {
            label: if doomed.len() > 1 {
                "Delete presets"
            } else {
                "Delete"
            },
            icon: IconKind::Trash,
            disabled: !can_delete,
            action: Callback::new(move |()| session.delete_presets(doomed.clone())),
        },
    ]
}

#[derive(Clone, Copy)]
struct DragState {
    dragged: RwSignal<Option<Slot>>,
    over: RwSignal<Option<Slot>>,
}

#[component]
fn PresetRow(row: Row, drag: DragState, menu: MenuState<Slot>) -> impl IntoView {
    let session = Session::expect();
    let preset_ref = PresetRef {
        area: row.area,
        slot: row.slot,
    };
    let slot = row.slot;
    let is_saved = row.name.is_some();
    let is_current = move || session.current.get() == preset_ref;
    let is_read_only = !row.area.is_writable();
    let can_edit = move || !is_read_only && session.current.get().area == preset_ref.area;
    let is_picked = move || {
        can_edit()
            && session
                .picked
                .with(|picked| picked.len() > 1 && picked.contains(&slot))
    };
    let stored_label = row.name.clone().unwrap_or_else(|| {
        if is_read_only {
            String::new()
        } else {
            "Empty".to_owned()
        }
    });
    let unreadable_reason = row.error;
    let is_drafting = move || is_current() && session.is_new.get();
    let label = move || {
        if is_drafting() {
            session.draft.with(|preset| preset.name().to_owned())
        } else {
            stored_label.clone()
        }
    };

    let on_click = move |event: MouseEvent| {
        if (event.ctrl_key() || event.meta_key()) && untrack(can_edit) {
            session.toggle_picked(slot);
            return;
        }
        if untrack(is_current) {
            if untrack(can_edit) {
                session.picked.set(BTreeSet::from([slot]));
            }
            return;
        }
        if is_saved || is_read_only {
            session.load(preset_ref);
        } else {
            session.new_preset(preset_ref);
        }
    };
    let on_context_menu = move |event: MouseEvent| {
        let is_in_group = session
            .picked
            .with_untracked(|picked| picked.contains(&slot));
        if !is_in_group && untrack(can_edit) {
            session.picked.set(BTreeSet::from([slot]));
        }
        menu.open(&event, slot);
    };
    let on_drag_start = move |event: DragEvent| {
        if let Some(transfer) = event.data_transfer() {
            transfer.set_effect_allowed("move");
            transfer
                .set_data("text/plain", &slot.to_string())
                .unwrap_or_default();
        }
        drag.dragged.set(Some(slot));
    };
    let on_drag_over = move |event: DragEvent| {
        if drag
            .dragged
            .get_untracked()
            .is_some_and(|from| from != slot)
        {
            event.prevent_default();
            if drag.over.get_untracked() != Some(slot) {
                drag.over.set(Some(slot));
            }
        }
    };
    let on_drop = move |event: DragEvent| {
        event.prevent_default();
        if let Some(from) = drag.dragged.get_untracked()
            && from != slot
        {
            session.swap_by_drag(from, slot);
        }
        drag.dragged.set(None);
        drag.over.set(None);
    };

    view! {
        <li
            draggable=move || (is_saved && can_edit()).to_string()
            on:dragstart=on_drag_start
            on:dragend=move |_| {
                drag.dragged.set(None);
                drag.over.set(None);
            }
            on:dragover=on_drag_over
            on:dragleave=move |_| {
                if drag.over.get_untracked() == Some(slot) {
                    drag.over.set(None);
                }
            }
            on:drop=on_drop
        >
            <div
                data-active=move || is_current().to_string()
                data-picked=move || is_picked().to_string()
                data-over=move || (drag.over.get() == Some(slot)).to_string()
                class=tw_merge!(
                    SIDEBAR_ROW_SURFACE_CLASS,
                    SIDEBAR_ROW_HOVER_CLASS,
                    SIDEBAR_ROW_LAYOUT_CLASS,
                    PRESET_ROW_CLASS
                )
                on:click=on_click
                on:contextmenu=on_context_menu
            >
                <button
                    type="button"
                    class="flex min-w-0 flex-1 items-center gap-1.5 self-stretch text-left outline-none"
                    aria-current=move || is_current().then_some("true")
                    title=unreadable_reason
                >
                    <span class=KEY_CLASS>{slot.to_string()}</span>
                    <span class=move || {
                        let tone = if !is_saved && !is_drafting() {
                            "text-muted-foreground italic"
                        } else if is_current() && session.is_dirty.get() {
                            "italic"
                        } else {
                            "not-italic"
                        };
                        format!("min-w-0 flex-1 truncate pr-1 {tone}")
                    }>{label}</span>
                </button>
                <Show when=move || is_saved && can_edit()>
                    <button
                        type="button"
                        class="text-muted-foreground hover:text-destructive rounded-sm p-1 opacity-0 transition-opacity group-hover:opacity-100 focus-visible:opacity-100"
                        aria-label=format!("Delete preset {slot}")
                        disabled=move || !session.is_idle()
                        title="Delete this preset from the Anagram"
                        on:click=move |event| {
                            event.stop_propagation();
                            session.delete_presets(vec![slot]);
                        }
                    >
                        <Icon kind=IconKind::Trash />
                    </button>
                </Show>
            </div>
        </li>
    }
}
