use std::collections::BTreeSet;

use leptos::prelude::*;

use crate::{
    backend::{FileRemoval, UploadRequest},
    host::{self, confirm},
    model::usage::uses_file,
    protocol::{
        hid::files::{UserDir, UserFile},
        ir,
        name::USER_FILE_NAME,
    },
    session::Session,
};

impl Session {
    pub fn refresh_user_files(self, dir: UserDir) {
        host::spawn(async move {
            self.load_user_files(dir).await;
        });
    }

    pub fn ensure_user_files(self, dir: UserDir) {
        let is_listed = self
            .files
            .listed_dirs
            .with_untracked(|listed| listed.contains(&dir));
        let is_first_request = self
            .files
            .requested_dirs
            .try_update_value(|requested| requested.insert(dir))
            .unwrap_or(false);
        if is_listed || !is_first_request {
            return;
        }
        host::spawn(async move {
            if !self.load_user_files(dir).await {
                self.files.requested_dirs.update_value(|requested| {
                    requested.remove(&dir);
                });
            }
        });
    }

    pub fn rename_user_file(self, file: UserFile, name: &str) {
        let name = USER_FILE_NAME.sanitize(name);
        if name.is_empty() || name == file.name {
            return;
        }
        let Some(dir) = file.dir() else {
            return;
        };
        self.run_busy(format!("Renaming {}…", file.name), async move {
            match self.backend().rename_user_file(file, name).await {
                Ok(()) => _ = self.load_user_files(dir).await,
                Err(error) => self.notify(format!("Rename failed: {error}"), true),
            }
        });
    }

    pub fn remove_user_file(self, file: UserFile) {
        let Some(dir) = file.dir() else {
            return;
        };
        let path = file.device_path();
        self.run_busy(format!("Checking which presets use {}…", file.name), async move {
            let name = file.name.clone();
            let usage = match self.presets_using(|document| uses_file(document, &path)).await {
                Ok(usage) => usage,
                Err(error) => {
                    self.notify(format!("Could not check which presets use {name}: {error}"), true);
                    return;
                }
            };
            if !confirm(&usage.question(&format!("\"{name}\""), "those presets lose the file.")) {
                return;
            }
            self.link.busy.set(Some(format!("Deleting {name}…")));
            match self.backend().remove_user_file(file).await {
                Ok(removal) => {
                    self.load_user_files(dir).await;
                    if removal == FileRemoval::EntryOnly {
                        self.notify(
                            format!("{name} was removed from the list; its file was already gone from the Anagram."),
                            false,
                        );
                    }
                }
                Err(error) => self.notify(format!("Delete failed: {error}"), true),
            }
        });
    }

    fn file_ids_in(self, dir: UserDir) -> BTreeSet<u64> {
        self.files
            .user_files
            .with_untracked(|files| files_in(files, dir).map(|file| file.id).collect())
    }

    pub fn upload_file(
        self,
        dir: UserDir,
        name: String,
        file_name: String,
        bytes: Vec<u8>,
        on_done: impl FnOnce(Option<UserFile>) + 'static,
    ) {
        let done = move |uploaded: Option<UserFile>| {
            self.files.upload_progress.set(None);
            on_done(uploaded);
        };
        if name.trim().is_empty() {
            self.notify("Enter a name for the file.", true);
            done(None);
            return;
        }
        let prepared = match dir {
            UserDir::Cabinets => ir::convert(&bytes).map_err(|error| error.to_string()),
            UserDir::NeuralModels => Ok(bytes),
        };
        let payload = match prepared {
            Ok(payload) => payload,
            Err(error) => {
                self.notify(error, true);
                done(None);
                return;
            }
        };
        if !self.begin(format!("Uploading {name}…"), false) {
            done(None);
            return;
        }
        host::spawn(async move {
            if let Err(error) = self.backend().ensure_serial().await {
                self.notify(format!("Upload failed: {error}"), true);
                self.finish();
                done(None);
                return;
            }
            let is_listed = self
                .files
                .listed_dirs
                .with_untracked(|listed| listed.contains(&dir));
            let known =
                (is_listed || self.load_user_files(dir).await).then(|| self.file_ids_in(dir));
            let request = UploadRequest {
                dir,
                name: name.clone(),
                file_name,
                bytes: payload,
            };
            let outcome = self
                .backend()
                .upload(
                    request,
                    Box::new(move |share| self.files.upload_progress.set(Some(share))),
                )
                .await;
            let uploaded = match outcome {
                Ok(device_name) => {
                    let is_relisted = self.load_user_files(dir).await;
                    known.filter(|_| is_relisted).and_then(|known| {
                        self.files.user_files.with_untracked(|files| {
                            newly_uploaded(files, dir, &known, &name, device_name.as_deref())
                                .cloned()
                        })
                    })
                }
                Err(error) => {
                    self.notify(format!("Upload failed: {error}"), true);
                    None
                }
            };
            self.finish();
            done(uploaded);
        });
    }

    async fn load_user_files(self, dir: UserDir) -> bool {
        match self.backend().user_files(dir).await {
            Ok(files) => {
                self.files.user_files.update(|known| {
                    known.retain(|file| file.dir_name != dir.name());
                    known.extend(files);
                });
                self.files.listed_dirs.update(|listed| {
                    if !listed.contains(&dir) {
                        listed.push(dir);
                    }
                });
                true
            }
            Err(error) => {
                self.notify(format!("Could not list user files: {error}"), true);
                false
            }
        }
    }
}

fn files_in(files: &[UserFile], dir: UserDir) -> impl Iterator<Item = &UserFile> {
    files.iter().filter(move |file| file.dir_name == dir.name())
}

fn newly_uploaded<'files>(
    files: &'files [UserFile],
    dir: UserDir,
    known: &BTreeSet<u64>,
    name: &str,
    device_name: Option<&str>,
) -> Option<&'files UserFile> {
    let fresh: Vec<&UserFile> = files_in(files, dir)
        .filter(|file| !known.contains(&file.id))
        .collect();
    let stored = device_name.and_then(|device_name| {
        fresh
            .iter()
            .copied()
            .find(|file| file.file_name == device_name)
    });
    let named = fresh
        .iter()
        .copied()
        .filter(|file| file.name == name)
        .max_by_key(|file| file.id);
    stored.or(named).or(match fresh.as_slice() {
        [only] => Some(*only),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::newly_uploaded;
    use crate::protocol::hid::files::{UserDir, UserFile};

    fn user_file(id: u64, name: &str, dir: UserDir) -> UserFile {
        UserFile {
            id,
            name: name.to_owned(),
            file_name: format!("{name}.wav"),
            dir_name: dir.name().to_owned(),
            file_size: None,
            uris: Vec::new(),
            original_file_name: None,
            extra: serde_json::Map::new(),
        }
    }

    #[test]
    fn the_uploaded_file_is_the_new_one_with_the_uploaded_name() {
        let files = [
            user_file(1, "OLD", UserDir::Cabinets),
            user_file(2, "MINE", UserDir::Cabinets),
            user_file(3, "OTHER", UserDir::Cabinets),
            user_file(4, "MINE", UserDir::NeuralModels),
        ];
        let known = BTreeSet::from([1]);
        let found = newly_uploaded(&files, UserDir::Cabinets, &known, "MINE", None);
        assert_eq!(found.map(|file| file.id), Some(2));
        assert!(newly_uploaded(&files, UserDir::Cabinets, &known, "GONE", None).is_none());
        let by_device_name =
            newly_uploaded(&files, UserDir::Cabinets, &known, "GONE", Some("OTHER.wav"));
        assert_eq!(by_device_name.map(|file| file.id), Some(3));
    }

    #[test]
    fn a_single_new_file_is_taken_even_when_the_device_renamed_it() {
        let files = [
            user_file(1, "OLD", UserDir::Cabinets),
            user_file(2, "MINE-1", UserDir::Cabinets),
        ];
        let known = BTreeSet::from([1]);
        let found = newly_uploaded(&files, UserDir::Cabinets, &known, "MINE", None);
        assert_eq!(found.map(|file| file.id), Some(2));
    }
}
