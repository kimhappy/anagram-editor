use std::{collections::BTreeSet, sync::Arc};

use leptos::prelude::*;

use crate::{
    host::{self, confirm},
    model::usage::uses_plugin,
    protocol::hid::{plugin::PluginSummary, settings::favourites_payload},
    session::{Listing, Session},
};

impl Session {
    pub fn toggle_favourite(self, uri: &str) {
        let favourites = self.favourites.get_untracked();
        let toggled = self
            .catalog
            .with_untracked(|catalog| catalog.toggled_favourites(&favourites, uri));
        self.run_busy("Updating favourites…", async move {
            let backend = self.backend();
            let outcome = match backend.edit_settings(favourites_payload(toggled)).await {
                Ok(()) => backend.read_state().await,
                Err(error) => Err(error),
            };
            match outcome {
                Ok(snapshot) => self.favourites.set(snapshot.favourites),
                Err(error) => self.notify(format!("Favourites were not saved: {error}"), true),
            }
        });
    }

    pub fn refresh_user_plugins(self) {
        host::spawn(async move {
            self.load_user_plugins().await;
        });
    }

    pub fn delete_plugin(self, uri: String, name: &str) {
        let listed = match self.files.user_plugins.get_untracked() {
            Listing::Loaded(plugins) => plugins,
            Listing::Loading | Listing::Failed(_) => Vec::new(),
        };
        let name = name.to_owned();
        let catalog = self.catalog.get_untracked();
        self.run_busy(format!("Checking which presets use {name}…"), async move {
            let is_plugin = |block_uri: &str| catalog.is_same_plugin(&uri, block_uri);
            let usage = match self
                .presets_using(|document| uses_plugin(document, is_plugin))
                .await
            {
                Ok(usage) => usage,
                Err(error) => {
                    self.notify(
                        format!("Could not check which presets use {name}: {error}"),
                        true,
                    );
                    return;
                }
            };
            let question = usage.question(
                &format!("the plugin \"{name}\""),
                "those presets lose that block.",
            );
            if !confirm(&format!(
                "{question} It can only be installed again with the Darkglass Suite."
            )) {
                return;
            }
            self.link.busy.set(Some(format!("Deleting {name}…")));
            match self.backend().delete_plugin(uri.clone()).await {
                Ok(()) => self.confirm_plugin_deleted(&listed, &uri, &name).await,
                Err(error) => self.notify(format!("Delete failed: {error}"), true),
            }
        });
    }

    async fn confirm_plugin_deleted(self, listed: &[PluginSummary], uri: &str, name: &str) {
        let Some(remaining) = self.load_user_plugins().await else {
            return;
        };
        if remaining.iter().any(|plugin| plugin.uri == uri) {
            self.notify(format!("The Anagram kept {name}."), true);
            return;
        }
        let removed = gone_uris(listed, &remaining, uri);
        self.catalog
            .update(|catalog| *catalog = Arc::new(catalog.without(&removed)));
        self.notify(format!("Deleted {name}"), false);
    }

    async fn load_user_plugins(self) -> Option<Vec<PluginSummary>> {
        match self.backend().user_plugins().await {
            Ok(plugins) => {
                self.files
                    .user_plugins
                    .set(Listing::Loaded(plugins.clone()));
                Some(plugins)
            }
            Err(error) => {
                let is_listed = self
                    .files
                    .user_plugins
                    .with_untracked(|listing| matches!(listing, Listing::Loaded(_)));
                if !is_listed {
                    self.files
                        .user_plugins
                        .set(Listing::Failed(error.to_string()));
                }
                self.notify(format!("Could not list plugins: {error}"), true);
                None
            }
        }
    }
}

fn gone_uris(before: &[PluginSummary], after: &[PluginSummary], deleted: &str) -> BTreeSet<String> {
    before
        .iter()
        .map(|plugin| plugin.uri.as_str())
        .chain(std::iter::once(deleted))
        .filter(|uri| after.iter().all(|plugin| plugin.uri != *uri))
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::gone_uris;
    use crate::protocol::hid::plugin::PluginSummary;

    fn plugin(uri: &str) -> PluginSummary {
        PluginSummary {
            uri: uri.to_owned(),
            version: String::new(),
            license: 0,
        }
    }

    #[test]
    fn a_deleted_bundle_takes_its_sibling_plugins_along() {
        let before = [plugin("urn:a"), plugin("urn:a-stereo"), plugin("urn:b")];
        let after = [plugin("urn:b")];
        let gone: Vec<String> = gone_uris(&before, &after, "urn:a").into_iter().collect();
        assert_eq!(gone, ["urn:a", "urn:a-stereo"]);
    }
}
