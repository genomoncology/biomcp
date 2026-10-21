//! Locate the successful managed conversion for a later local view.

use std::io::Read as _;
use std::path::PathBuf;

use super::{Article, ArticleFulltextKind};
use crate::error::BioMcpError;
use crate::utils::download;

#[derive(serde::Serialize, serde::Deserialize)]
struct SavedFulltext {
    identities: Vec<String>,
    cache_identifier: String,
    kind: ArticleFulltextKind,
}

fn identity(id: &str) -> String {
    id.trim().to_ascii_lowercase()
}

fn locator_key(id: &str) -> String {
    format!("article-view-v1:{}", download::cache_key(id))
}

pub(super) async fn remember(
    article: &Article,
    requested_id: &str,
    cache_identifier: &str,
    kind: ArticleFulltextKind,
) -> Result<(), BioMcpError> {
    if crate::sources::cache_is_bypassed() {
        return Ok(());
    }
    let config = crate::cache::resolve_cache_config()?;
    let mut identities = [
        Some(requested_id),
        article.pmid.as_deref(),
        article.pmcid.as_deref(),
        article.doi.as_deref(),
    ]
    .into_iter()
    .flatten()
    .map(identity)
    .filter(|id| !id.is_empty())
    .collect::<Vec<_>>();
    identities.sort_unstable();
    identities.dedup();
    let record = SavedFulltext {
        identities,
        cache_identifier: cache_identifier.to_owned(),
        kind,
    };
    tokio::task::spawn_blocking(move || -> std::io::Result<()> {
        let body = serde_json::to_vec(&record)?;
        let mut store = crate::cache::sqlite::Store::open(&config.cache_root)?;
        for id in &record.identities {
            let key = locator_key(id);
            store.put("fulltext", &key, &key, &[], &body, None)?;
        }
        Ok(())
    })
    .await
    .map_err(|error| std::io::Error::other(error.to_string()))??;
    Ok(())
}

/// A missing, stale or unusable locator leaves normal provider retrieval intact.
pub(crate) async fn saved_fulltext_path(id: &str) -> Option<PathBuf> {
    if crate::sources::cache_is_bypassed() {
        return None;
    }
    let config = crate::cache::resolve_cache_config().ok()?;
    let id = identity(id);
    tokio::task::spawn_blocking(move || {
        let bytes = crate::cache::sqlite::Store::open(&config.cache_root)
            .ok()?
            .get("fulltext", &locator_key(&id))
            .ok()??
            .body;
        let record: SavedFulltext = serde_json::from_slice(&bytes).ok()?;
        if !record.identities.contains(&id)
            || !record
                .identities
                .contains(&identity(&record.cache_identifier))
        {
            return None;
        }
        let key = super::fulltext::fulltext_cache_key(record.kind, &record.cache_identifier);
        let path = download::download_path_for_config(&key, &config);
        let mut file = crate::cache::open_managed_read(&path).ok()?;
        let mut text = String::new();
        file.read_to_string(&mut text).ok()?;
        (!text.trim().is_empty()).then_some(path)
    })
    .await
    .ok()?
}
