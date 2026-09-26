use leptos::prelude::*;
use tw_merge::tw_merge;

use super::{side_panel::EMPTY_CLASS, uploads};
use crate::{
    components::{
        icon::IconKind,
        ui::{Button, ButtonSize, ButtonVariant, Dialog, IconButton, InlineField, Tabs},
    },
    protocol::{
        hid::files::{UserDir, UserFile},
        name::USER_FILE_NAME,
    },
    session::{Listing, Modal, Session},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shelf {
    Files(UserDir),
    Plugins,
}

const FILE_SHELVES: [(Shelf, &str); 2] = [
    (Shelf::Files(UserDir::Cabinets), "Cabinet IRs"),
    (Shelf::Files(UserDir::NeuralModels), "Neural models"),
];

#[component]
pub fn UserFilesDialog() -> impl IntoView {
    let session = Session::expect();
    let shelf = RwSignal::new(Shelf::Files(UserDir::Cabinets));
    let can_manage_plugins = session.backend().can_manage_plugins();
    Effect::new(move |_| {
        if session.ui.is_open(Modal::UserFiles).get() {
            for dir in UserDir::ALL {
                session.refresh_user_files(dir);
            }
            if can_manage_plugins {
                session.refresh_user_plugins();
            }
        }
    });
    let shelves = Signal::derive(move || {
        FILE_SHELVES
            .iter()
            .map(|&(option, label)| (option, label.to_owned()))
            .chain(can_manage_plugins.then(|| (Shelf::Plugins, "Plugins".to_owned())))
            .collect::<Vec<_>>()
    });

    view! {
        <Dialog
            is_open=session.ui.is_open(Modal::UserFiles)
            on_close=move |()| session.ui.close()
            title="User files"
            class="max-w-2xl gap-5"
        >
            <div class="flex items-center justify-between gap-4">
                <Tabs options=shelves value=shelf on_change=move |picked| shelf.set(picked) />
                {move || match shelf.get() {
                    Shelf::Files(dir) => view! { <uploads::UploadButton dir look=uploads::UploadLook::Toolbar /> }.into_any(),
                    Shelf::Plugins => ().into_any(),
                }}
            </div>
            {move || match shelf.get() {
                Shelf::Files(dir) => view! { <FileList dir /> }.into_any(),
                Shelf::Plugins => view! { <PluginList /> }.into_any(),
            }}
        </Dialog>
    }
}

#[component]
fn FileList(dir: UserDir) -> impl IntoView {
    let session = Session::expect();
    let files = Memo::new(move |_| {
        session.files.user_files.with(|files| {
            let mut listed: Vec<UserFile> = files
                .iter()
                .filter(|file| file.dir_name == dir.name())
                .cloned()
                .collect();
            listed.sort_by_key(|file| file.name.to_lowercase());
            listed
        })
    });
    view! {
        <ul class=SHELF_LIST_CLASS>
            <For each=move || files.get() key=|file| file.id children=move |file| view! { <FileRow file files /> } />
            <Show when=move || files.with(Vec::is_empty)>
                <li class=EMPTY_CLASS>"No files in this folder yet."</li>
            </Show>
        </ul>
    }
}

const SHELF_LIST_CLASS: &str = "grid max-h-[50vh] grid-cols-1 gap-1 overflow-y-auto";
const SHELF_ROW_CLASS: &str =
    "hover:bg-accent/50 flex items-center gap-3 rounded-md px-2 py-1 text-sm";

#[derive(Clone, Debug, PartialEq, Eq)]
struct PluginEntry {
    uri: String,
    name: String,
    license: &'static str,
}

#[component]
fn PluginList() -> impl IntoView {
    let session = Session::expect();
    let entries = Memo::new(move |_| {
        let plugins = match session.files.user_plugins.get() {
            Listing::Loading => return Listing::Loading,
            Listing::Failed(reason) => return Listing::Failed(reason),
            Listing::Loaded(plugins) => plugins,
        };
        let entries = session.catalog.with(|catalog| {
            let mut entries: Vec<PluginEntry> = plugins
                .iter()
                .filter(|plugin| {
                    catalog
                        .variants(&plugin.uri)
                        .is_none_or(|pair| pair.mono.uri == plugin.uri)
                })
                .map(|plugin| PluginEntry {
                    uri: plugin.uri.clone(),
                    name: catalog
                        .find(&plugin.uri)
                        .map_or_else(|| plugin.uri.clone(), |model| model.name.clone()),
                    license: license_label(plugin.license),
                })
                .collect();
            entries.sort_by_key(|entry| entry.name.to_lowercase());
            entries
        });
        Listing::Loaded(entries)
    });
    view! {
        <ul class=SHELF_LIST_CLASS>
            {move || match entries.get() {
                Listing::Loading => view! { <li class=EMPTY_CLASS>"Reading plugins…"</li> }.into_any(),
                Listing::Failed(reason) => view! {
                    <li class=tw_merge!(EMPTY_CLASS, "flex items-center justify-between gap-2")>
                        {format!("Could not list plugins: {reason}")}
                        <Button variant=ButtonVariant::Outline size=ButtonSize::Sm on:click=move |_| session.refresh_user_plugins()>
                            "Retry"
                        </Button>
                    </li>
                }
                .into_any(),
                Listing::Loaded(listed) if listed.is_empty() => {
                    view! { <li class=EMPTY_CLASS>"No plugins were installed by a user."</li> }.into_any()
                }
                Listing::Loaded(listed) => listed
                    .into_iter()
                    .map(|entry| view! { <PluginRow entry /> })
                    .collect_view()
                    .into_any(),
            }}
        </ul>
    }
}

#[component]
fn PluginRow(entry: PluginEntry) -> impl IntoView {
    let session = Session::expect();
    let PluginEntry { uri, name, license } = entry;
    let label = format!("Delete {name}");
    let deleted_name = name.clone();
    view! {
        <li class=SHELF_ROW_CLASS>
            <div class="grid min-w-0 flex-1">
                <span class="truncate">{name}</span>
                <span class="text-muted-foreground truncate text-xs">{uri.clone()}</span>
            </div>
            <span class="text-muted-foreground w-16 shrink-0 text-right text-xs">{license}</span>
            <IconButton
                kind=IconKind::Trash
                class="size-8"
                label
                attr:title="Delete this plugin from the Anagram"
                disabled=Signal::derive(move || !session.is_idle())
                on:click=move |_| session.delete_plugin(uri.clone(), &deleted_name)
            />
        </li>
    }
}

const fn license_label(license: u8) -> &'static str {
    match license {
        0 => "Free",
        1 => "Trial",
        2 => "Licensed",
        _ => "",
    }
}

#[component]
fn FileRow(file: UserFile, files: Memo<Vec<UserFile>>) -> impl IntoView {
    let session = Session::expect();
    let id = file.id;
    let initial = StoredValue::new(file);
    let record = Memo::new(move |_| {
        files
            .with(|files| files.iter().find(|listed| listed.id == id).cloned())
            .unwrap_or_else(|| initial.get_value())
    });
    let name = Signal::derive(move || record.with(|current| current.name.clone()));
    let is_in_use = Signal::derive(move || record.with(|current| !current.uris.is_empty()));
    view! {
        <li class=SHELF_ROW_CLASS>
            <div class="min-w-0 flex-1">
                <InlineField
                    text=name
                    label="File name"
                    maxlength=USER_FILE_NAME.max_length
                    class="h-8 w-full px-2 text-left"
                    on_commit=move |typed: String| session.rename_user_file(record.get_untracked(), &typed)
                >
                    <span class="block truncate">{name}</span>
                </InlineField>
            </div>
            <span
                class="text-muted-foreground w-40 shrink-0 truncate text-xs"
                title=move || record.with(|current| current.original_file_name.clone().unwrap_or_default())
            >
                {move || record.with(|current| current.file_name.clone())}
            </span>
            <span class="text-muted-foreground w-16 shrink-0 text-right text-xs tabular-nums">
                {move || record.with(|current| current.file_size.map_or_else(String::new, size_label))}
            </span>
            <IconButton
                kind=IconKind::Trash
                class="size-8"
                label=Signal::derive(move || format!("Delete {}", name.get()))
                attr:title=move || if is_in_use.get() { "Used by a preset" } else { "Delete this file from the Anagram" }
                disabled=Signal::derive(move || is_in_use.get() || !session.is_idle())
                on:click=move |_| session.remove_user_file(record.get_untracked())
            />
        </li>
    }
}

#[expect(
    clippy::cast_precision_loss,
    clippy::float_arithmetic,
    reason = "a size shown with one decimal"
)]
fn size_label(bytes: u64) -> String {
    let kilobytes = bytes.div_ceil(1000);
    if kilobytes >= 1000 {
        format!("{:.1} MB", bytes as f64 / 1_000_000.0)
    } else {
        format!("{kilobytes} kB")
    }
}

#[cfg(test)]
mod tests {
    use super::size_label;

    #[test]
    fn size_label_switches_to_megabytes_once_kilobytes_reach_1000() {
        assert_eq!(size_label(0), "0 kB");
        assert_eq!(size_label(1), "1 kB");
        assert_eq!(size_label(10_000), "10 kB");
        assert_eq!(size_label(998_999), "999 kB");
        assert_eq!(size_label(999_001), "1.0 MB");
        assert_eq!(size_label(999_500), "1.0 MB");
        assert_eq!(size_label(1_000_000), "1.0 MB");
        assert_eq!(size_label(12_345_678), "12.3 MB");
    }
}
