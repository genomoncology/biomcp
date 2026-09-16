//! Persistent citation-evidence sidecar for ticket 1199.
//!
//! One file per resolved directed edge under
//! `<cache root>/citation-evidence/v1/`, published atomically and read before
//! the graph traversal. The store never changes a command outcome: every read
//! failure is a miss, and a write failure is reported through tracing only.

use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};

use crate::entities::article::graph::citation_evidence::{
    ArticleCitationEvidenceResult, CitationEvidenceStatus,
};

pub(crate) const CITATION_EVIDENCE_DIR: &str = "citation-evidence";
const CITATION_EVIDENCE_KEY_VERSION: &str = "v1";
const CITATION_EVIDENCE_SCHEMA_VERSION: u32 = 1;
const CITATION_EVIDENCE_CACHE_TTL_MS: u64 = 2_592_000_000;
const CITATION_EVIDENCE_MAX_RECORD_BYTES: u64 = 8 * 1024 * 1024;

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

fn version_dir(cache_root: &Path) -> PathBuf {
    cache_root
        .join(CITATION_EVIDENCE_DIR)
        .join(CITATION_EVIDENCE_KEY_VERSION)
}

fn record_path(cache_root: &Path, citing_pid: &str, cited_pid: &str) -> PathBuf {
    let key = format!(
        "{CITATION_EVIDENCE_KEY_VERSION}|{}|{}",
        citing_pid.to_ascii_lowercase(),
        cited_pid.to_ascii_lowercase()
    );
    let digest = Sha256::digest(key.as_bytes());
    let name = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    version_dir(cache_root).join(format!("{name}.json"))
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
pub(crate) fn read_citation_evidence(
    cache_root: &Path,
    citing_pid: &str,
    cited_pid: &str,
    force_fulltext: bool,
) -> Option<ArticleCitationEvidenceResult> {
    if crate::sources::cache_is_bypassed() {
        return None;
    }
    let path = record_path(cache_root, citing_pid, cited_pid);
    if !fs::symlink_metadata(&path).ok()?.file_type().is_file() {
        return None;
    }
    let file = crate::cache::open_managed_read(&path).ok()?;
    let mut bytes = Vec::new();
    file.take(CITATION_EVIDENCE_MAX_RECORD_BYTES + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    if u64::try_from(bytes.len()).ok()? > CITATION_EVIDENCE_MAX_RECORD_BYTES {
        return None;
    }
    let record: CitationEvidenceRecord = serde_json::from_slice(&bytes).ok()?;
    if record.schema_version != CITATION_EVIDENCE_SCHEMA_VERSION {
        return None;
    }
    if record.citing_paper_id != citing_pid.to_ascii_lowercase()
        || record.cited_paper_id != cited_pid.to_ascii_lowercase()
    {
        return None;
    }
    if !crate::sources::cache_is_infinite() && record.expires_at_unix_ms <= now_unix_ms() {
        return None;
    }
    if force_fulltext && record.result.status == CitationEvidenceStatus::ContextFromProvider {
        return None;
    }
    Some(record.result)
}

/// Publishes an evidence-bearing result. A bypassed cache, a non-evidence
/// status, and every write failure leave the caller's outcome untouched.
pub(crate) fn write_citation_evidence(
    cache_root: &Path,
    citing_pid: &str,
    cited_pid: &str,
    result: &ArticleCitationEvidenceResult,
) {
    if crate::sources::cache_is_bypassed() || !is_evidence_status(result.status) {
        return;
    }
    if let Err(error) = publish(cache_root, citing_pid, cited_pid, result) {
        tracing::warn!(%error, "citation-evidence sidecar write failed");
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
    let path = record_path(cache_root, citing_pid, cited_pid);
    let dir = version_dir(cache_root);
    // The sidecar is a private managed tree. A regular file in its place
    // fails here, the record is never published, and the outcome stands.
    crate::cache::secure_managed_tree(&dir, true, None)?;
    let temporary = temp_path(&dir);
    let write_result = (|| -> std::io::Result<()> {
        let mut file = crate::cache::open_private(
            OpenOptions::new().write(true).create_new(true),
            &temporary,
        )?;
        file.write_all(&bytes)?;
        file.flush()?;
        file.sync_all()?;
        fs::rename(&temporary, &path)?;
        let _ = fs::File::open(&dir).and_then(|file| file.sync_all());
        Ok(())
    })();
    if write_result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    write_result
}

fn temp_path(dir: &Path) -> PathBuf {
    let pid = std::process::id();
    for attempt in 0..100 {
        let path = dir.join(format!(".record.{pid}.{attempt}.tmp"));
        if !path.exists() {
            return path;
        }
    }
    dir.join(format!(".record.{pid}.tmp"))
}
