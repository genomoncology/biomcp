use std::io::{self, Write};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use clap::Subcommand;

use crate::error::BioMcpError;

#[derive(Subcommand, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheCommand {
    /// Print the managed HTTP cache path as plain text (`--json` is ignored)
    #[command(long_about = "\
Print the managed HTTP cache path as plain text.

This command is read-only and prints `<resolved cache_root>/http`.
The global `--json` flag is ignored for this command and output stays plain text.
This command family is CLI-only because it reveals workstation-local filesystem paths.")]
    Path,
    /// Show HTTP cache statistics
    #[command(long_about = "\
Show HTTP cache statistics.

Print an on-demand snapshot of blob counts, bytes, age range, and configured cache limits.
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
    /// Wipe the entire managed HTTP cache directory
    #[command(long_about = "\
Wipe the entire managed HTTP cache directory.

Deletes all contents of <resolved cache_root>/http. This is a destructive full wipe;
use `biomcp cache clean` for targeted cleanup instead. The managed downloads/ sibling
directory is never touched. Interactive confirmation is required unless you pass
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

/// Render the managed HTTP cache path without creating or migrating cache directories.
///
/// # Errors
///
/// Returns an error if cache configuration resolution fails.
pub fn render_path() -> Result<String, BioMcpError> {
    let config = crate::cache::resolve_cache_config()?;
    Ok(config.cache_root.join("http").display().to_string())
}

pub(crate) fn execute_clean(
    max_age: Option<Duration>,
    max_size: Option<u64>,
    dry_run: bool,
) -> Result<crate::cache::CleanReport, BioMcpError> {
    let config = crate::cache::resolve_cache_config()?;
    let cache_path = config.cache_root.join("http");
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| {
            BioMcpError::InvalidArgument(format!("system clock is before the Unix epoch: {err}"))
        })?
        .as_millis();
    crate::cache::execute_cache_clean(
        &cache_path,
        crate::cache::CleanOptions {
            max_age,
            max_size,
            dry_run,
        },
        &config,
        now_ms,
    )
}

pub(crate) fn render_clean_text(report: &crate::cache::CleanReport) -> String {
    format!(
        "Cache clean: dry_run={} orphans_removed={} entries_removed={} bytes_freed={} errors={}",
        report.dry_run,
        report.orphans_removed,
        report.entries_removed,
        report.bytes_freed,
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
        "This will permanently delete the managed HTTP cache at {}. Continue? [y/N]: ",
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
    pub(crate) blob_bytes: u64,
    pub(crate) referenced_blob_bytes: u64,
    pub(crate) blob_count: usize,
    pub(crate) orphan_count: usize,
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
            format!("| Blob bytes | {} |", self.blob_bytes),
            format!("| Referenced blob bytes | {} |", self.referenced_blob_bytes),
            format!("| Blob files | {} |", self.blob_count),
            format!("| Orphan blobs | {} |", self.orphan_count),
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

fn checked_timestamp_ms(timestamp: u128) -> Result<u64, BioMcpError> {
    u64::try_from(timestamp).map_err(|_| {
        BioMcpError::InvalidArgument(format!(
            "cache entry timestamp {timestamp} does not fit into u64"
        ))
    })
}

pub(crate) fn build_cache_stats_report(
    snapshot: &crate::cache::CacheSnapshot,
    config: &crate::cache::ResolvedCacheConfig,
) -> Result<CacheStatsReport, BioMcpError> {
    let age_range = match (
        snapshot.entries.iter().map(|entry| entry.time_ms).min(),
        snapshot.entries.iter().map(|entry| entry.time_ms).max(),
    ) {
        (Some(oldest), Some(newest)) => Some(CacheStatsAgeRange {
            oldest_ms: checked_timestamp_ms(oldest)?,
            newest_ms: checked_timestamp_ms(newest)?,
        }),
        (None, None) => None,
        _ => unreachable!("min/max over the same iterator source must agree"),
    };
    let usage = crate::cache::summarize_cache_usage(snapshot);

    Ok(CacheStatsReport {
        path: snapshot.cache_path.display().to_string(),
        blob_bytes: usage.total_blob_bytes,
        referenced_blob_bytes: usage.referenced_blob_bytes,
        blob_count: snapshot.blobs.len(),
        orphan_count: snapshot
            .blobs
            .iter()
            .filter(|blob| blob.refcount == 0)
            .count(),
        age_range,
        max_size_bytes: config.max_size,
        max_size_origin: CacheStatsOrigin::from(config.origins.max_size),
        min_disk_free: config.min_disk_free.display(),
        min_disk_free_origin: CacheStatsOrigin::from(config.origins.min_disk_free),
        max_age_secs: config.max_age.as_secs(),
        max_age_origin: CacheStatsOrigin::from(config.origins.max_age),
    })
}

pub(crate) fn collect_cache_stats_report() -> Result<CacheStatsReport, BioMcpError> {
    collect_cache_stats_report_with(
        crate::cache::resolve_cache_config,
        crate::cache::snapshot_cache,
    )
}

fn collect_cache_stats_report_with<R, S>(
    resolve_config: R,
    snapshotter: S,
) -> Result<CacheStatsReport, BioMcpError>
where
    R: FnOnce() -> Result<crate::cache::ResolvedCacheConfig, BioMcpError>,
    S: FnOnce(
        &std::path::Path,
    ) -> Result<crate::cache::CacheSnapshot, crate::cache::CachePlannerError>,
{
    let config = resolve_config()?;
    let http_path = config.cache_root.join("http");
    let snapshot =
        snapshotter(&http_path).map_err(|err| BioMcpError::Io(std::io::Error::other(err)))?;
    build_cache_stats_report(&snapshot, &config)
}

#[cfg(test)]
mod tests;
