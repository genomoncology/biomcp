use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use fs2::FileExt;

use crate::error::BioMcpError;

use super::dispatch::ArticleSuggestion;

const STORE_VERSION: u8 = 1;
const SESSION_TTL_SECS: u64 = 10 * 60;
const MAX_ACTIVE_SESSIONS: usize = 1024;
const STORE_DIR: &str = "sessions";
const STORE_FILE: &str = "article-search-loop-breaker.json";
const LOCK_FILE: &str = "article-search-loop-breaker.lock";
const INVALID_SESSION_MESSAGE: &str = "--session must be 1-128 ASCII characters containing only letters, digits, '.', '_', ':', or '-'";

pub(super) struct SessionSearch<'a> {
    pub token: &'a str,
    pub keyword: Option<&'a str>,
    pub pmids: &'a [String],
    pub next_commands: &'a [String],
    pub now_epoch_secs: u64,
}

#[derive(Debug, thiserror::Error)]
enum StoreError {
    #[error("session store IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("session store JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
struct Store {
    version: u8,
    sessions: BTreeMap<String, SessionEntry>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct SessionEntry {
    updated_at_epoch_secs: u64,
    keyword: String,
    terms: Vec<String>,
    pmids: Vec<String>,
}

pub(super) fn validate_token(token: &str) -> Result<(), BioMcpError> {
    let valid_len = !token.is_empty() && token.len() <= 128;
    let valid_chars = token
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'));

    if valid_len && valid_chars {
        Ok(())
    } else {
        Err(BioMcpError::InvalidArgument(
            INVALID_SESSION_MESSAGE.to_string(),
        ))
    }
}

fn normalized_terms(keyword: &str) -> BTreeSet<String> {
    keyword
        .to_ascii_lowercase()
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|term| !term.is_empty())
        .filter(|term| !is_stopword(term))
        .map(str::to_string)
        .collect()
}

fn overlap_score(previous: &BTreeSet<String>, current: &BTreeSet<String>) -> f64 {
    if previous.is_empty() || current.is_empty() {
        return 0.0;
    }

    let intersection = previous.intersection(current).count();
    let union = previous.union(current).count();
    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

pub(super) fn record_success_and_suggestions(
    cache_root: &Path,
    search: SessionSearch<'_>,
) -> Vec<ArticleSuggestion> {
    match record_success_and_suggestions_inner(cache_root, search) {
        Ok(suggestions) => suggestions,
        Err(err) => {
            tracing::warn!("Article search session loop-breaker store unavailable: {err}");
            Vec::new()
        }
    }
}

pub(super) fn current_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

fn store_path(cache_root: &Path) -> PathBuf {
    cache_root.join(STORE_DIR).join(STORE_FILE)
}

fn lock_path(cache_root: &Path) -> PathBuf {
    cache_root.join(STORE_DIR).join(LOCK_FILE)
}

fn is_stopword(term: &str) -> bool {
    matches!(
        term,
        "a" | "an"
            | "and"
            | "are"
            | "article"
            | "articles"
            | "clinical"
            | "does"
            | "for"
            | "in"
            | "is"
            | "literature"
            | "of"
            | "on"
            | "or"
            | "paper"
            | "papers"
            | "publication"
            | "publications"
            | "review"
            | "reviews"
            | "study"
            | "studies"
            | "the"
            | "to"
            | "what"
            | "with"
    )
}

fn record_success_and_suggestions_inner(
    cache_root: &Path,
    search: SessionSearch<'_>,
) -> Result<Vec<ArticleSuggestion>, StoreError> {
    let session_dir = cache_root.join(STORE_DIR);
    fs::create_dir_all(&session_dir)?;
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(lock_path(cache_root))?;
    lock.try_lock_exclusive()?;

    let now = search.now_epoch_secs;
    let mut store = read_store(&store_path(cache_root))?;
    prune_expired(&mut store, now);

    let raw_keyword = search.keyword.unwrap_or_default().trim();
    let current_terms = normalized_terms(raw_keyword);
    let previous = store.sessions.get(search.token).cloned();
    let suggestions = previous
        .as_ref()
        .filter(|entry| !entry.terms.is_empty() && !current_terms.is_empty())
        .and_then(|entry| {
            let previous_terms = entry.terms.iter().cloned().collect::<BTreeSet<_>>();
            (overlap_score(&previous_terms, &current_terms) >= 0.60)
                .then(|| loop_suggestions(entry, raw_keyword, search.next_commands))
        })
        .unwrap_or_default();

    let pmids = search
        .pmids
        .iter()
        .map(|pmid| pmid.trim())
        .filter(|pmid| !pmid.is_empty())
        .take(crate::entities::article::ARTICLE_BATCH_MAX_IDS)
        .map(str::to_string)
        .collect::<Vec<_>>();
    store.sessions.insert(
        search.token.to_string(),
        SessionEntry {
            updated_at_epoch_secs: now,
            keyword: raw_keyword.to_string(),
            terms: current_terms.into_iter().collect(),
            pmids,
        },
    );
    prune_capacity(&mut store);
    write_store_atomic(cache_root, &store)?;

    Ok(suggestions)
}

fn read_store(path: &Path) -> Result<Store, StoreError> {
    match fs::read_to_string(path) {
        Ok(contents) => match serde_json::from_str::<Store>(&contents) {
            Ok(mut store) => {
                store.version = STORE_VERSION;
                Ok(store)
            }
            Err(_) => Ok(Store {
                version: STORE_VERSION,
                sessions: BTreeMap::new(),
            }),
        },
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(Store {
            version: STORE_VERSION,
            sessions: BTreeMap::new(),
        }),
        Err(err) => Err(err.into()),
    }
}

fn write_store_atomic(cache_root: &Path, store: &Store) -> Result<(), StoreError> {
    let path = store_path(cache_root);
    let dir = path.parent().expect("fixed session store has parent");
    let bytes = serde_json::to_vec_pretty(store)?;
    let tmp_path = create_temp_path(dir)?;

    let write_result = (|| -> Result<(), StoreError> {
        let mut tmp = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp_path)?;
        tmp.write_all(&bytes)?;
        tmp.flush()?;
        tmp.sync_all()?;
        fs::rename(&tmp_path, &path)?;
        let _ = File::open(dir).and_then(|file| file.sync_all());
        Ok(())
    })();

    if write_result.is_err() {
        let _ = fs::remove_file(&tmp_path);
    }
    write_result
}

fn create_temp_path(dir: &Path) -> Result<PathBuf, StoreError> {
    let pid = std::process::id();
    for attempt in 0..100 {
        let path = dir.join(format!(".{STORE_FILE}.{pid}.{attempt}.tmp"));
        if !path.exists() {
            return Ok(path);
        }
    }
    Err(std::io::Error::new(
        ErrorKind::AlreadyExists,
        "could not allocate article session store temp path",
    )
    .into())
}

fn prune_expired(store: &mut Store, now_epoch_secs: u64) {
    store.sessions.retain(|_, entry| {
        now_epoch_secs.saturating_sub(entry.updated_at_epoch_secs) <= SESSION_TTL_SECS
    });
}

fn prune_capacity(store: &mut Store) {
    if store.sessions.len() <= MAX_ACTIVE_SESSIONS {
        return;
    }

    let remove_count = store.sessions.len() - MAX_ACTIVE_SESSIONS;
    let mut oldest = store
        .sessions
        .iter()
        .map(|(token, entry)| (entry.updated_at_epoch_secs, token.clone()))
        .collect::<Vec<_>>();
    oldest.sort();
    for (_, token) in oldest.into_iter().take(remove_count) {
        store.sessions.remove(&token);
    }
}

fn loop_suggestions(
    previous: &SessionEntry,
    current_keyword: &str,
    next_commands: &[String],
) -> Vec<ArticleSuggestion> {
    let mut suggestions = Vec::new();
    if !previous.pmids.is_empty() {
        suggestions.push(ArticleSuggestion {
            command: format!("biomcp article batch {}", previous.pmids.join(" ")),
            reason: "Use the prior search's article set instead of reformulating the keyword."
                .to_string(),
            sections: Vec::new(),
        });
    }

    if !current_keyword.is_empty() {
        suggestions.push(ArticleSuggestion {
            command: format!(
                "biomcp discover {}",
                crate::render::markdown::shell_quote_arg(current_keyword)
            ),
            reason: "Map the topic to structured biomedical entities before searching again."
                .to_string(),
            sections: Vec::new(),
        });
    }

    if let Some(command) = first_date_retry_command(next_commands) {
        suggestions.push(ArticleSuggestion {
            command: command.to_string(),
            reason: "Narrow the current article search by publication year instead of changing the wording."
                .to_string(),
            sections: Vec::new(),
        });
    }

    suggestions
}

fn first_date_retry_command(next_commands: &[String]) -> Option<&str> {
    next_commands.iter().map(String::as_str).find(|command| {
        command.starts_with("biomcp search article ")
            && command.contains(" --year-min ")
            && command.contains(" --year-max ")
    })
}

#[cfg(test)]
mod tests;
