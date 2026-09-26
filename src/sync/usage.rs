use leptos::prelude::*;

use crate::{
    backend::BackendError,
    model::{document, library::PresetRef, usage::Usage},
    protocol::hid::preset::PresetDocument,
    session::Session,
};

impl Session {
    pub(crate) async fn presets_using(
        self,
        uses: impl Fn(&PresetDocument) -> bool,
    ) -> Result<Usage, BackendError> {
        let areas: Vec<_> = self.library.with_untracked(|library| {
            library
                .areas()
                .iter()
                .map(|info| info.area)
                .filter(|area| area.is_writable())
                .collect()
        });
        for area in &areas {
            if !self
                .library
                .with_untracked(|library| library.is_listed(*area))
            {
                self.backend().load_index(*area).await.map(|index| {
                    self.library
                        .update(|library| library.set_index(*area, index));
                })?;
            }
        }
        let is_single_area = areas.len() == 1;
        let stored: Vec<(PresetRef, String)> = self.library.with_untracked(|library| {
            areas
                .iter()
                .filter_map(|area| Some((*area, library.index(*area)?)))
                .flat_map(|(area, index)| {
                    index.entries().filter_map(move |(slot, entry)| {
                        let entry = entry?;
                        let place = if is_single_area {
                            slot.to_string()
                        } else {
                            format!("{} {slot}", library.label(area))
                        };
                        Some((
                            PresetRef { area, slot },
                            format!("{place} \"{}\"", entry.name),
                        ))
                    })
                })
                .collect()
        });
        let is_draft_user = self.has_unsaved_edits()
            && self
                .draft
                .with_untracked(|draft| uses(&document::export(draft, self.scene.get_untracked())));
        let mut usage = Usage {
            users: is_draft_user
                .then(|| "your unsaved edits".to_owned())
                .into_iter()
                .collect(),
            unread: 0,
        };
        let total = stored.len();
        for (done, (preset_ref, label)) in stored.into_iter().enumerate() {
            self.link
                .busy
                .set(Some(format!("Checking presets… {done}/{total}")));
            match self.backend().fetch_preset(preset_ref).await {
                Ok(Some(found)) if uses(&found) => usage.users.push(label),
                Ok(_) => {}
                Err(_unread) => usage.unread += 1,
            }
        }
        Ok(usage)
    }
}
