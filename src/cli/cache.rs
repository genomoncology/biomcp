use std::io::{self, Write};
use std::time::Duration;

use clap::Subcommand;

use crate::error::BioMcpError;

#[derive(Subcommand, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheCommand {
    /// Print the SQLite cache database path
    #[command(long_about = "\
Print the SQLite cache database path.

This command is read-only and prints `<resolved cache_root>/biomcp.db`.
Without `--json` the output is the plain path. With `--json` it is a typed
`cache_path` object containing the path.
This command family is CLI-only because it reveals workstation-local filesystem paths.")]
    Path,
    /// Remove legacy cache stores while preserving saved downloads and the current cache
    CleanLegacy {
        /// Skip interactive confirmation
        #[arg(long)]
        yes: bool,
    },
    /// Show HTTP cache and article-session statistics
    #[command(long_about = "\
Show HTTP cache statistics.

Print an on-demand snapshot of HTTP entries, article sessions, blob counts, bytes, age range, and configured cache limits.
Use the global `--json` flag for machine-readable output.
This command is CLI-only because cache commands reveal workstation-local filesystem paths.")]
    Stats,
    /// Remove orphan blobs and optionally evict cache entries by age or size
    #[command(long_about = "\
Remove orphan blobs and optionally evict cache entries by age or size.

This command always garbage-collects orphaned blobs. Use --max-age to remove entries
older than a duration like 30d or 12h, and --max-size to LRU-evict until referenced
blob bytes are under a target like 5G or 500M. Use --dry-run to preview the same
cleanup plan without deleting anything. The global `--json` flag returns the
structured cleanup report.
This command is CLI-only because cache commands reveal workstation-local filesystem paths.")]
    Clean {
        /// Remove entries older than this duration (e.g. 30d, 12h)
        #[arg(long, value_parser = parse_cache_max_age)]
        max_age: Option<Duration>,

        /// LRU-evict until referenced blob bytes are under this size (e.g. 5G, 500M)
        #[arg(long, value_parser = parse_cache_max_size)]
        max_size: Option<u64>,

        /// Show the cleanup plan without deleting anything
        #[arg(long)]
        dry_run: bool,
    },
    /// Remove reusable cache entries and article sessions
    #[command(long_about = "\
Remove reusable cache entries and article sessions.

Deletes reusable entries from <resolved cache_root>/biomcp.db and article sessions. This is a
destructive full wipe; use `biomcp cache clean` for targeted cleanup instead. The managed downloads/ sibling
directory and saved provider captures are preserved. Interactive confirmation is required unless you pass
--yes. Without a TTY and without --yes, this command refuses even under `--json`.

This command is CLI-only because cache commands reveal workstation-local filesystem paths.")]
    Clear {
        /// Skip the confirmation prompt for non-interactive or scripted use
        #[arg(long)]
        yes: bool,
    },
}

fn parse_cache_max_age(value: &str) -> Result<Duration, String> {
    humantime::parse_duration(value)
        .map_err(|err| format!("--max-age must be a duration like 30d or 12h: {err}"))
}

fn parse_cache_max_size(value: &str) -> Result<u64, String> {
    value
        .parse::<bytesize::ByteSize>()
        .map(|size| size.as_u64())
        .map_err(|err| format!("--max-size must be a size like 5G or 500M: {err}"))
}

/// Render the SQLite cache database path without creating cache files.
///
/// # Errors
///
/// Returns an error if cache configuration resolution fails.
pub fn render_path() -> Result<String, BioMcpError> {
    let config = crate::cache::resolve_cache_config()?;
    Ok(render_path_for_config(&config))
}

pub(crate) fn render_path_for_config(config: &crate::cache::ResolvedCacheConfig) -> String {
    config
        .cache_root
        .join(crate::cache::sqlite::DATABASE_NAME)
        .display()
        .to_string()
}

pub(crate) fn execute_clean(
    max_age: Option<Duration>,
    max_size: Option<u64>,
    dry_run: bool,
) -> Result<crate::cache::CleanReport, BioMcpError> {
    let config = crate::cache::resolve_cache_config()?;
    let mut store = crate::cache::sqlite::Store::open(&config.cache_root)?;
    let mut report = crate::cache::sqlite::maintain(
        &mut store,
        &config,
        max_age,
        max_size,
        dry_run,
        crate::cache::sqlite::Maintenance::Clean,
    )?;
    let (orphans, bytes) = store.collect_orphans(dry_run)?;
    report.orphans_removed += orphans;
    report.bytes_freed += bytes;
    if !dry_run {
        super::article::session::maintain_sessions(
            &config.cache_root,
            (crate::cache::sqlite::now_ms() / 1000) as u64,
        )?;
    }
    Ok(report)
}

pub(crate) fn render_clean_text(report: &crate::cache::CleanReport) -> String {
    format!(
        "Cache clean: dry_run={} orphans_removed={} entries_removed={} bytes_freed={} provider_capture_bytes_freed={} errors={}",
        report.dry_run,
        report.orphans_removed,
        report.entries_removed,
        report.bytes_freed,
        report.provider_capture_bytes_freed,
        report.errors.len()
    )
}

pub(crate) fn render_clear_text(report: &crate::cache::ClearReport) -> String {
    if report.bytes_freed.is_none() && report.entries_removed == 0 {
        return "Cache clear cancelled: bytes_freed=null entries_removed=0".to_string();
    }

    let bytes_freed = report
        .bytes_freed
        .map(|bytes| bytes.to_string())
        .unwrap_or_else(|| "null".to_string());
    format!(
        "Cache clear: bytes_freed={bytes_freed} entries_removed={}",
        report.entries_removed
    )
}

pub(crate) fn prompt_clear_confirmation(cache_path: &std::path::Path) -> Result<bool, BioMcpError> {
    let mut stderr = io::stderr();
    write!(
        &mut stderr,
        "This will permanently delete managed cache entries at {}. Continue? [y/N]: ",
        cache_path.display()
    )?;
    stderr.flush()?;

    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    let answer = answer.trim();
    Ok(answer.eq_ignore_ascii_case("y") || answer.eq_ignore_ascii_case("yes"))
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct CacheStatsAgeRange {
    pub(crate) oldest_ms: u64,
    pub(crate) newest_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum CacheStatsOrigin {
    Env,
    File,
    Default,
}

impl CacheStatsOrigin {
    fn as_str(self) -> &'static str {
        match self {
            Self::Env => "env",
            Self::File => "file",
            Self::Default => "default",
        }
    }
}

impl From<crate::cache::ConfigOrigin> for CacheStatsOrigin {
    fn from(value: crate::cache::ConfigOrigin) -> Self {
        match value {
            crate::cache::ConfigOrigin::Env => Self::Env,
            crate::cache::ConfigOrigin::File => Self::File,
            crate::cache::ConfigOrigin::Default => Self::Default,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct CacheStatsReport {
    pub(crate) path: String,
    pub(crate) legacy_cache_bytes: std::collections::BTreeMap<String, u64>,
    pub(crate) blob_bytes: u64,
    pub(crate) referenced_blob_bytes: u64,
    pub(crate) blob_count: usize,
    pub(crate) orphan_count: usize,
    pub(crate) http_entry_count: usize,
    pub(crate) session_count: usize,
    pub(crate) provider_capture_bytes: u64,
    pub(crate) age_range: Option<CacheStatsAgeRange>,
    pub(crate) max_size_bytes: u64,
    pub(crate) max_size_origin: CacheStatsOrigin,
    pub(crate) min_disk_free: String,
    pub(crate) min_disk_free_origin: CacheStatsOrigin,
    pub(crate) max_age_secs: u64,
    pub(crate) max_age_origin: CacheStatsOrigin,
}

impl CacheStatsReport {
    pub(crate) fn to_markdown(&self) -> String {
        let age_display = match &self.age_range {
            Some(range) => format!("{} .. {}", range.oldest_ms, range.newest_ms),
            None => "none".to_string(),
        };
        [
            format!("| Path | {} |", self.path),
            format!("| Legacy cache bytes | {:?} |", self.legacy_cache_bytes),
            format!("| Blob bytes | {} |", self.blob_bytes),
            format!("| Referenced blob bytes | {} |", self.referenced_blob_bytes),
            format!("| Blob files | {} |", self.blob_count),
            format!("| Orphan blobs | {} |", self.orphan_count),
            format!("| HTTP cache entries | {} |", self.http_entry_count),
            format!("| Article sessions | {} |", self.session_count),
            format!(
                "| Provider capture bytes | {} |",
                self.provider_capture_bytes
            ),
            format!("| Age range | {age_display} |"),
            format!(
                "| Max size | {} bytes ({}) |",
                self.max_size_bytes,
                self.max_size_origin.as_str()
            ),
            format!(
                "| Min disk free | {} ({}) |",
                self.min_disk_free,
                self.min_disk_free_origin.as_str()
            ),
            format!(
                "| Max age | {} s ({}) |",
                self.max_age_secs,
                self.max_age_origin.as_str()
            ),
            String::new(), // trailing newline
        ]
        .join("\n")
    }
}

pub(crate) fn collect_cache_stats_report() -> Result<CacheStatsReport, BioMcpError> {
    let config = crate::cache::resolve_cache_config()?;
    collect_cache_stats_for_config(&config)
}

fn collect_cache_stats_for_config(
    config: &crate::cache::ResolvedCacheConfig,
) -> Result<CacheStatsReport, BioMcpError> {
    let store = crate::cache::sqlite::Store::open(&config.cache_root)?;
    let connection = store.connection();
    let (bytes, reusable_bytes, count): (u64, u64, usize) = connection
        .query_row(
            "SELECT (SELECT bytes FROM cache_meta WHERE id=1),(SELECT optional_bytes FROM cache_meta WHERE id=1),count(*) FROM entries WHERE namespace='http'",
            [],
            |r| {
                Ok((
                    crate::cache::sqlite::read_u64(r, 0)?,
                    crate::cache::sqlite::read_u64(r, 1)?,
                    r.get::<_, i64>(2)? as usize,
                ))
            },
        )
        .map_err(crate::cache::sqlite::sqlite_error)?;
    let (oldest, newest): (Option<i64>, Option<i64>) = connection
        .query_row(
            "SELECT min(created),max(created) FROM entries WHERE namespace='http'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(crate::cache::sqlite::sqlite_error)?;
    let capture_bytes = connection
        .query_row(
            "SELECT coalesce(sum(bytes),0) FROM entries WHERE namespace='capture'",
            [],
            |r| crate::cache::sqlite::read_u64(r, 0),
        )
        .map_err(crate::cache::sqlite::sqlite_error)?;
    let blobs = connection
        .query_row(
            "SELECT count(DISTINCT blob) FROM entries WHERE blob IS NOT NULL",
            [],
            |r| Ok(r.get::<_, i64>(0)? as usize),
        )
        .map_err(crate::cache::sqlite::sqlite_error)?;
    Ok(CacheStatsReport {
        path: render_path_for_config(config),
        blob_bytes: bytes,
        referenced_blob_bytes: reusable_bytes,
        blob_count: blobs,
        orphan_count: 0,
        http_entry_count: count,
        session_count: super::article::session::maintain_sessions(
            &config.cache_root,
            (crate::cache::sqlite::now_ms() / 1000) as u64,
        )?,
        provider_capture_bytes: capture_bytes,
        age_range: oldest
            .zip(newest)
            .map(|(oldest_ms, newest_ms)| CacheStatsAgeRange {
                oldest_ms: oldest_ms as u64,
                newest_ms: newest_ms as u64,
            }),
        max_size_bytes: config.max_size,
        max_size_origin: config.origins.max_size.into(),
        min_disk_free: config.min_disk_free.display(),
        min_disk_free_origin: config.origins.min_disk_free.into(),
        max_age_secs: config.max_age.as_secs(),
        max_age_origin: config.origins.max_age.into(),
        legacy_cache_bytes: legacy_sizes(&config.cache_root)?,
    })
}

pub(crate) fn execute_managed_clear(
    config: &crate::cache::ResolvedCacheConfig,
) -> Result<crate::cache::ClearReport, BioMcpError> {
    let mut store = crate::cache::sqlite::Store::open(&config.cache_root)?;
    let (bytes, count) = store.transaction(|tx, root| {
        let rows = {
            let mut statement = tx
                .prepare("SELECT namespace,key FROM entries WHERE namespace!='capture'")
                .map_err(crate::cache::sqlite::sqlite_error)?;
            statement
                .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
                .map_err(crate::cache::sqlite::sqlite_error)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(crate::cache::sqlite::sqlite_error)?
        };
        let mut freed = 0;
        for (namespace, key) in &rows {
            freed += crate::cache::sqlite::delete_record(tx, root, namespace, key)?;
        }
        Ok((freed, rows.len()))
    })?;
    store.collect_blobs()?;
    store.collect_orphans(false)?;
    store.release_space()?;
    let sessions = crate::cache::execute_cache_clear(&config.cache_root.join("sessions"))?;
    Ok(crate::cache::ClearReport {
        bytes_freed: sessions.bytes_freed.map(|value| value + bytes),
        entries_removed: count + sessions.entries_removed,
    })
}

const LEGACY_STORES: &[&str] = &["http", "http-cacache", "citation-evidence", "captures"];

fn legacy_sizes(
    root: &std::path::Path,
) -> Result<std::collections::BTreeMap<String, u64>, BioMcpError> {
    fn size(path: &std::path::Path) -> std::io::Result<u64> {
        let metadata = match std::fs::symlink_metadata(path) {
            Ok(m) => m,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(0),
            Err(e) => return Err(e),
        };
        if metadata.is_symlink() {
            return Ok(0);
        }
        if !metadata.is_dir() {
            return Ok(metadata.len());
        }
        std::fs::read_dir(path)?.try_fold(0, |total, entry| Ok(total + size(&entry?.path())?))
    }
    LEGACY_STORES
        .iter()
        .map(|name| Ok(((*name).to_owned(), size(&root.join(name))?)))
        .collect()
}

pub(crate) fn execute_legacy_clear(
    config: &crate::cache::ResolvedCacheConfig,
) -> Result<crate::cache::ClearReport, BioMcpError> {
    let mut report = crate::cache::ClearReport {
        bytes_freed: Some(0),
        entries_removed: 0,
    };
    for name in LEGACY_STORES {
        let result = crate::cache::execute_cache_clear(&config.cache_root.join(name))?;
        report.bytes_freed = report
            .bytes_freed
            .zip(result.bytes_freed)
            .map(|(a, b)| a + b);
        report.entries_removed += result.entries_removed;
    }
    Ok(report)
}

#[cfg(test)]
mod tests;
