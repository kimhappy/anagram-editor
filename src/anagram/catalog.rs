use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::hid::HidClient;
use crate::{
    browser,
    model::catalog::Catalog,
    protocol::hid::{
        action::{GetPluginInfo, PluginFilter, ReadAllPlugins},
        envelope::HidError,
        plugin::{PluginInfo, PluginSummary},
        version::Version,
    },
};

const CACHE_KEY: &str = "anagram-editor/catalog/v1";

#[derive(Default, Serialize, Deserialize)]
struct CachedCatalog {
    firmware: String,
    infos: Vec<(String, PluginInfo)>,
}

impl CachedCatalog {
    fn read(firmware: &str) -> BTreeMap<String, PluginInfo> {
        browser::stored_text(CACHE_KEY)
            .and_then(|text| serde_json::from_str::<Self>(&text).ok())
            .filter(|cached| cached.firmware == firmware)
            .map_or_default(|cached| cached.infos.into_iter().collect())
    }

    fn write(firmware: String, infos: &[(String, PluginInfo)]) {
        let cached = serde_json::to_string(&Self {
            firmware,
            infos: infos.to_vec(),
        });
        if let Ok(text) = cached {
            browser::store_text(CACHE_KEY, &text);
        }
    }
}

fn cache_key(summary: &PluginSummary) -> String {
    format!("{}@{}", summary.uri, summary.version)
}

pub async fn load(
    hid: &HidClient,
    firmware: &Version,
    mut progress: impl FnMut(usize, usize),
) -> Result<Catalog, HidError> {
    let summaries = hid.call(ReadAllPlugins { filter: None }).await?;
    let user_uris: BTreeSet<String> = hid
        .call(ReadAllPlugins {
            filter: Some(PluginFilter::User),
        })
        .await?
        .into_iter()
        .map(|summary| summary.uri)
        .collect();
    let total = summaries.len();
    progress(0, total);
    let firmware = firmware.to_string();
    let mut cached = CachedCatalog::read(&firmware);
    cached.retain(|key, _| {
        !user_uris.iter().any(|uri| {
            key.strip_prefix(uri.as_str())
                .is_some_and(|rest| rest.starts_with('@'))
        })
    });
    let mut infos: Vec<(String, PluginInfo)> = Vec::with_capacity(total);
    let mut is_complete = true;
    for (summary, done) in summaries.iter().zip(1..) {
        let key = cache_key(summary);
        let info = match cached.remove(&key) {
            Some(info) => Some(info),
            None => fetch_info(hid, &summary.uri).await?,
        };
        match info {
            Some(info) => infos.push((key, info)),
            None => is_complete = false,
        }
        progress(done, total);
    }
    if is_complete {
        CachedCatalog::write(firmware, &infos);
    }
    Ok(Catalog::from_plugins(infos.iter().map(|(_, info)| info)))
}

async fn fetch_info(hid: &HidClient, uri: &str) -> Result<Option<PluginInfo>, HidError> {
    let request = || GetPluginInfo {
        uri: uri.to_owned(),
    };
    let outcome = match hid.call(request()).await {
        Err(HidError::Timeout(_)) => hid.call(request()).await,
        other => other,
    };
    match outcome {
        Ok(info) => Ok(Some(info)),
        Err(
            error @ (HidError::Malformed { .. } | HidError::Device { .. } | HidError::Timeout(_)),
        ) => {
            web_sys::console::warn_1(&format!("Skipped the block {uri}: {error}").into());
            Ok(None)
        }
        Err(other) => Err(other),
    }
}
