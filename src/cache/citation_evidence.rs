//! Reusable directed citation evidence in the shared SQLite store.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};

use crate::entities::article::graph::citation_evidence::{
    ArticleCitationEvidenceResult, CitationEvidenceStatus,
};

const CITATION_EVIDENCE_KEY_VERSION: &str = "v1";
const CITATION_EVIDENCE_SCHEMA_VERSION: u32 = 1;
const CITATION_EVIDENCE_CACHE_TTL_MS: u64 = 2_592_000_000;

#[derive(serde::Serialize, serde::Deserialize)]
struct CitationEvidenceRecord {
    schema_version: u32,
    citing_paper_id: String,
    cited_paper_id: String,
    stored_at_unix_ms: u64,
    expires_at_unix_ms: u64,
    result: ArticleCitationEvidenceResult,
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or(0)
}

/// The debug-only seam mirrors the citation deadline seams: a test shrinks
/// the thirty-day TTL to zero instead of waiting for expiry.
fn ttl_ms() -> u64 {
    #[cfg(debug_assertions)]
    if let Ok(value) = std::env::var("BIOMCP_TEST_CITATION_CACHE_TTL_MS")
        && let Ok(millis) = value.trim().parse::<u64>()
    {
        return millis;
    }
    CITATION_EVIDENCE_CACHE_TTL_MS
}

fn record_key(citing_pid: &str, cited_pid: &str) -> String {
    let key = format!(
        "{CITATION_EVIDENCE_KEY_VERSION}|{}|{}",
        citing_pid.to_ascii_lowercase(),
        cited_pid.to_ascii_lowercase()
    );
    format!("{:x}", Sha256::digest(key.as_bytes()))
}

/// The statuses that carry usable evidence. A failure status is never cached:
/// it usually means transient provider unavailability, and pinning one for
/// the TTL would present a stale failure as current.
const fn is_evidence_status(status: CitationEvidenceStatus) -> bool {
    matches!(
        status,
        CitationEvidenceStatus::ContextFromProvider
            | CitationEvidenceStatus::ContextFromFulltext
            | CitationEvidenceStatus::ReferenceConfirmedWithoutPassage
    )
}

/// Reads the stored result for a resolved directed edge.
///
/// Every miss returns `None`: absent, unreadable, malformed, a different
/// schema version, a key mismatch, expiry, a bypassed cache, or a forced
/// full-text call whose stored entry holds provider context only.
fn read_citation_evidence_sync(
    cache_root: &Path,
    citing_pid: &str,
    cited_pid: &str,
    force_fulltext: bool,
    infinite: bool,
) -> Option<ArticleCitationEvidenceResult> {
    let key = record_key(citing_pid, cited_pid);
    let bytes = super::sqlite::Store::open(cache_root)
        .ok()?
        .get("citation", &key)
        .ok()??
        .body;
    let record: CitationEvidenceRecord = serde_json::from_slice(&bytes).ok()?;
    if record.schema_version != CITATION_EVIDENCE_SCHEMA_VERSION {
        return None;
    }
    if record.citing_paper_id != citing_pid.to_ascii_lowercase()
        || record.cited_paper_id != cited_pid.to_ascii_lowercase()
    {
        return None;
    }
    if !infinite && record.expires_at_unix_ms <= now_unix_ms() {
        return None;
    }
    if force_fulltext && record.result.status == CitationEvidenceStatus::ContextFromProvider {
        return None;
    }
    Some(record.result)
}

/// Publishes an evidence-bearing result. A bypassed cache, a non-evidence
/// status, and every write failure leave the caller's outcome untouched.
pub(crate) async fn write_citation_evidence(
    cache_root: &Path,
    citing_pid: &str,
    cited_pid: &str,
    result: &ArticleCitationEvidenceResult,
) {
    if crate::sources::cache_is_bypassed() || !is_evidence_status(result.status) {
        return;
    }
    let cache_root = cache_root.to_path_buf();
    let citing_pid = citing_pid.to_owned();
    let cited_pid = cited_pid.to_owned();
    let result = result.clone();
    match tokio::task::spawn_blocking(move || {
        publish(&cache_root, &citing_pid, &cited_pid, &result)
    })
    .await
    {
        Ok(Ok(())) => {}
        Ok(Err(error)) => super::warn_local_http_cache("citation-write", &error),
        Err(error) => super::warn_local_http_cache("citation-write", &error),
    }
}

fn publish(
    cache_root: &Path,
    citing_pid: &str,
    cited_pid: &str,
    result: &ArticleCitationEvidenceResult,
) -> std::io::Result<()> {
    let stored_at_unix_ms = now_unix_ms();
    let record = CitationEvidenceRecord {
        schema_version: CITATION_EVIDENCE_SCHEMA_VERSION,
        citing_paper_id: citing_pid.to_ascii_lowercase(),
        cited_paper_id: cited_pid.to_ascii_lowercase(),
        stored_at_unix_ms,
        expires_at_unix_ms: stored_at_unix_ms.saturating_add(ttl_ms()),
        result: result.clone(),
    };
    let bytes = serde_json::to_vec(&record).map_err(std::io::Error::other)?;
    let key = record_key(citing_pid, cited_pid);
    let mut store = super::sqlite::Store::open(cache_root)?;
    store.put(
        "citation",
        &key,
        &key,
        &[],
        &bytes,
        Some(record.expires_at_unix_ms.min(i64::MAX as u64) as i64),
    )?;
    let mut config = super::resolve_cache_config().map_err(std::io::Error::other)?;
    config.cache_root = cache_root.to_path_buf();
    super::sqlite::maintain(
        &mut store,
        &config,
        None,
        None,
        false,
        super::sqlite::Maintenance::Pressure,
    )?;
    Ok(())
}

pub(crate) async fn read_citation_evidence(
    cache_root: &Path,
    citing_pid: &str,
    cited_pid: &str,
    force_fulltext: bool,
) -> Option<ArticleCitationEvidenceResult> {
    if crate::sources::cache_is_bypassed() {
        return None;
    }
    let infinite = crate::sources::cache_is_infinite();
    let root = cache_root.to_path_buf();
    let citing = citing_pid.to_owned();
    let cited = cited_pid.to_owned();
    tokio::task::spawn_blocking(move || {
        read_citation_evidence_sync(&root, &citing, &cited, force_fulltext, infinite)
    })
    .await
    .ok()
    .flatten()
}
