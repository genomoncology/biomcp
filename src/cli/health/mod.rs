//! Stable facade and report rendering for `biomcp health`.

mod catalog;
mod http;
mod local;
mod runner;

use crate::error::BioMcpError;

#[derive(Debug, Clone, Copy, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Ok,
    Error,
    Excluded,
    Available,
    Configured,
    NotConfigured,
    Warning,
    Unavailable,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct HealthRow {
    pub api: String,
    pub status: HealthStatus,
    pub latency: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub affects: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_configured: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_env_var: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missing_files: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_built: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct HealthReport {
    pub healthy: usize,
    pub warning: usize,
    pub excluded: usize,
    pub error: usize,
    pub total: usize,
    pub rows: Vec<HealthRow>,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthExitPolicy {
    ReportOnly,
    FailOnError,
}

#[derive(serde::Serialize)]
struct HealthOutput<'a> {
    #[serde(flatten)]
    report: &'a HealthReport,
    exit_policy: HealthExitPolicy,
    ok: bool,
}

impl HealthReport {
    pub fn all_healthy(&self) -> bool {
        self.error == 0
    }

    pub fn to_markdown(&self) -> String {
        self.to_markdown_with_policy(false)
    }

    pub fn to_markdown_with_policy(&self, fail_on_error: bool) -> String {
        let mut out = String::new();
        let show_affects = self.rows.iter().any(|row| row.affects.is_some());

        out.push_str("# BioMCP Health Check\n\n");
        if show_affects {
            out.push_str("| API | Status | Latency | Affects |\n");
            out.push_str("|-----|--------|---------|---------|\n");
            for row in &self.rows {
                let affects = row.affects.as_deref().unwrap_or("-");
                let status = markdown_status(row);
                out.push_str(&format!(
                    "| {} | {} | {} | {} |\n",
                    row.api, status, row.latency, affects
                ));
            }
        } else {
            out.push_str("| API | Status | Latency |\n");
            out.push_str("|-----|--------|---------|\n");
            for row in &self.rows {
                let status = markdown_status(row);
                out.push_str(&format!("| {} | {} | {} |\n", row.api, status, row.latency));
            }
        }

        out.push_str(&format!(
            "\nStatus: {} ok, {} error, {} excluded",
            self.healthy, self.error, self.excluded
        ));
        if self.warning > 0 {
            out.push_str(&format!(", {} warning", self.warning));
        }
        out.push('\n');
        out.push_str(&format!(
            "Exit policy: {}; result: {}\n",
            if fail_on_error {
                "fail_on_error"
            } else {
                "report_only"
            },
            if self.all_healthy() {
                "ok"
            } else {
                "errors present"
            }
        ));
        out
    }

    pub fn to_json(&self, fail_on_error: bool) -> Result<String, BioMcpError> {
        crate::render::json::to_pretty(&HealthOutput {
            report: self,
            exit_policy: if fail_on_error {
                HealthExitPolicy::FailOnError
            } else {
                HealthExitPolicy::ReportOnly
            },
            ok: self.all_healthy(),
        })
    }
}

fn markdown_status(row: &HealthRow) -> String {
    match (row.status, row.key_configured) {
        (HealthStatus::Ok, Some(true)) => "ok (key configured)".to_string(),
        (HealthStatus::Error, Some(true)) => "error (key configured)".to_string(),
        (HealthStatus::Error, Some(false)) => "error (key not configured)".to_string(),
        (HealthStatus::Excluded, _) => format!(
            "excluded (set {})",
            row.required_env_var.as_deref().unwrap_or_default()
        ),
        (HealthStatus::Available, Some(false)) => {
            "available (unauthenticated, shared rate limit)".to_string()
        }
        (HealthStatus::Available, _) if row.local_path.is_some() => {
            if row.stale == Some(true) {
                "available (default path, stale)".to_string()
            } else {
                "available (default path)".to_string()
            }
        }
        (HealthStatus::Configured, Some(true)) => "configured (authenticated)".to_string(),
        (HealthStatus::Configured, _) if row.stale == Some(true) => {
            "configured (stale)".to_string()
        }
        (HealthStatus::Unavailable, _) if row.required_env_var.is_some() => format!(
            "unavailable (set {} for reliable access)",
            row.required_env_var.as_deref().unwrap_or_default()
        ),
        (HealthStatus::Unavailable, _) if row.not_built == Some(true) => {
            "unavailable (not built)".to_string()
        }
        (HealthStatus::Unavailable, _) => "unavailable".to_string(),
        (HealthStatus::NotConfigured, _) => "not configured".to_string(),
        (HealthStatus::Error, _) if row.missing_files.is_some() => format!(
            "error (missing: {})",
            row.missing_files.as_deref().unwrap_or_default().join(", ")
        ),
        (HealthStatus::Ok, _) => "ok".to_string(),
        (HealthStatus::Error, _) => "error".to_string(),
        (HealthStatus::Available, _) => "available".to_string(),
        (HealthStatus::Configured, _) => "configured".to_string(),
        (HealthStatus::Warning, _) => "warning".to_string(),
    }
}

/// Runs connectivity checks for configured upstream APIs and local EMA/CVX/WHO/GTR/WHO IVD/cache readiness.
///
/// # Errors
///
/// Returns an error when the shared HTTP client cannot be created.
pub async fn check(apis_only: bool, apis: &[String]) -> Result<HealthReport, BioMcpError> {
    let selected = select_sources(apis)?;
    runner::check(apis_only || !apis.is_empty(), &selected).await
}

pub(crate) async fn command(
    args: crate::cli::system::HealthArgs,
    json: bool,
) -> Result<crate::cli::CommandOutcome, BioMcpError> {
    let report = check(args.apis_only, &args.apis).await?;
    let text = if json {
        report.to_json(args.fail_on_error)?
    } else {
        report.to_markdown_with_policy(args.fail_on_error)
    };
    Ok(crate::cli::CommandOutcome::stdout_with_exit(
        text,
        u8::from(args.fail_on_error && !report.all_healthy()),
    ))
}

fn select_sources(requested: &[String]) -> Result<Vec<catalog::SourceDescriptor>, BioMcpError> {
    let catalog = catalog::health_sources();
    if requested.is_empty() {
        return Ok(catalog.to_vec());
    }

    let mut selected = Vec::new();
    for raw in requested {
        let name = raw.trim();
        let matches = catalog
            .iter()
            .filter(|source| source.api.eq_ignore_ascii_case(name))
            .copied()
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [source] => {
                if !selected.iter().any(|chosen: &catalog::SourceDescriptor| {
                    chosen.api.eq_ignore_ascii_case(source.api)
                }) {
                    selected.push(*source);
                }
            }
            [] => {
                let needle = name.to_ascii_lowercase();
                let mut suggestions = catalog
                    .iter()
                    .filter(|source| source.api.to_ascii_lowercase().contains(&needle))
                    .map(|source| source.api)
                    .take(3)
                    .collect::<Vec<_>>();
                if suggestions.is_empty() {
                    suggestions.extend(catalog.iter().take(3).map(|source| source.api));
                }
                return Err(BioMcpError::InvalidArgument(format!(
                    "Unknown health API {name:?}. Canonical suggestions: {}.",
                    suggestions.join(", ")
                )));
            }
            _ => {
                return Err(BioMcpError::InvalidArgument(format!(
                    "Ambiguous health API {name:?}; matching canonical names: {}.",
                    matches
                        .iter()
                        .map(|source| source.api)
                        .collect::<Vec<_>>()
                        .join(", ")
                )));
            }
        }
    }
    Ok(selected)
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use crate::cache::{
        CacheBlob, CacheConfigOrigins, CacheEntry, CacheSnapshot, ConfigOrigin, DiskFreeThreshold,
        ResolvedCacheConfig,
    };
    use crate::test_support::TempDirGuard;
    use ssri::Integrity;

    fn block_on<F: Future>(future: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("health test runtime")
            .block_on(future)
    }

    fn fixture_ema_root() -> TempDirGuard {
        let root = TempDirGuard::new("health-ema");
        write_ema_files(root.path(), crate::sources::ema::EMA_REQUIRED_FILES);
        root
    }

    fn write_ema_files(root: &Path, files: &[&str]) {
        for file in files {
            std::fs::write(root.join(file), b"{}").expect("write EMA fixture file");
        }
    }

    fn write_who_files(root: &Path, files: &[&str]) {
        for file in files {
            let bytes: &[u8] = match *file {
                crate::sources::who_pq::WHO_PQ_CSV_FILE => {
                    b"WHO Reference Number,INN, Dosage Form and Strength,Product Type,Therapeutic Area,Applicant,Dosage Form,Basis of Listing,Basis of alternative listing,Date of Prequalification\n"
                }
                crate::sources::who_pq::WHO_PQ_API_CSV_FILE => {
                    b"WHO Product ID,INN,Grade,Therapeutic area,Applicant organization,Date of prequalification,Confirmation of Prequalification Document Date\n"
                }
                crate::sources::who_pq::WHO_VACCINES_CSV_FILE => {
                    b"Date of Prequalification ,Vaccine Type,Commercial Name,Presentation,No. of doses,Manufacturer,Responsible NRA\n"
                }
                other => panic!("unexpected WHO fixture file: {other}"),
            };
            std::fs::write(root.join(file), bytes).expect("write WHO fixture file");
        }
    }

    fn write_cvx_files(root: &Path, files: &[&str]) {
        for file in files {
            let bytes: &[u8] = match *file {
                crate::sources::cvx::CVX_FILE => {
                    b"62|HPV, quadrivalent|human papilloma virus vaccine, quadrivalent||Active|False|2020/06/02\n"
                }
                crate::sources::cvx::TRADENAME_FILE => {
                    b"GARDASIL|HPV, quadrivalent|62|Merck and Co., Inc.|MSD|Active|Active|2010/05/28|\n"
                }
                crate::sources::cvx::MVX_FILE => {
                    b"MSD|Merck and Co., Inc.||Active|2012/10/18\n"
                }
                other => panic!("unexpected CVX fixture file: {other}"),
            };
            std::fs::write(root.join(file), bytes).expect("write CVX fixture file");
        }
    }

    fn write_gtr_files(root: &Path, files: &[&str]) {
        for file in files {
            match *file {
                crate::sources::gtr::GTR_TEST_VERSION_FILE => std::fs::write(
                    root.join(file),
                    include_bytes!("../../../spec/fixtures/gtr/test_version.gz"),
                )
                .expect("write GTR gzip fixture"),
                crate::sources::gtr::GTR_CONDITION_GENE_FILE => std::fs::write(
                    root.join(file),
                    include_str!("../../../spec/fixtures/gtr/test_condition_gene.txt"),
                )
                .expect("write GTR tsv fixture"),
                other => panic!("unexpected GTR fixture file: {other}"),
            }
        }
    }

    fn write_who_ivd_files(root: &Path, files: &[&str]) {
        for file in files {
            let bytes: &[u8] = match *file {
                crate::sources::who_ivd::WHO_IVD_CSV_FILE => {
                    b"Product name,Product Code,WHO Product ID,Assay Format,Regulatory Version,Manufacturer name,Pathogen/Disease/Marker,Year prequalification\n"
                }
                other => panic!("unexpected WHO IVD fixture file: {other}"),
            };
            std::fs::write(root.join(file), bytes).expect("write WHO IVD fixture file");
        }
    }

    fn set_stale_mtime_with_age(path: &Path, age: std::time::Duration) {
        let file = std::fs::OpenOptions::new()
            .write(true)
            .open(path)
            .expect("fixture file should open");
        file.set_modified(
            std::time::SystemTime::now()
                .checked_sub(age)
                .expect("stale time should be valid"),
        )
        .expect("mtime should update");
    }

    fn set_stale_mtime(path: &Path) {
        set_stale_mtime_with_age(path, std::time::Duration::from_secs(73 * 60 * 60));
    }

    fn set_stale_ema_mtimes(root: &Path) {
        for file_name in crate::sources::ema::EMA_REQUIRED_FILES {
            set_stale_mtime(&root.join(file_name));
        }
    }

    fn set_fresh_ema_mtimes(root: &Path) {
        for file_name in crate::sources::ema::EMA_REQUIRED_FILES {
            let file = std::fs::OpenOptions::new()
                .write(true)
                .open(root.join(file_name))
                .expect("fixture file should open");
            file.set_modified(std::time::SystemTime::now())
                .expect("mtime should update");
        }
    }

    fn assert_cache_dir_affects(value: Option<&str>) {
        assert_eq!(value, Some("local cache-backed lookups and downloads"));
    }

    fn assert_millisecond_latency(value: &str) {
        let digits = value
            .strip_suffix("ms")
            .expect("latency should end with ms");
        assert!(
            !digits.is_empty() && digits.chars().all(|ch| ch.is_ascii_digit()),
            "unexpected latency: {value}"
        );
    }

    fn update_max(target: &AtomicUsize, candidate: usize) {
        let mut observed = target.load(Ordering::SeqCst);
        while candidate > observed {
            match target.compare_exchange(observed, candidate, Ordering::SeqCst, Ordering::SeqCst) {
                Ok(_) => break,
                Err(actual) => observed = actual,
            }
        }
    }

    fn test_integrity(bytes: &[u8]) -> Integrity {
        Integrity::from(bytes)
    }

    fn test_entry(key: &str, bytes: &[u8], time_ms: u128) -> CacheEntry {
        CacheEntry {
            key: key.to_string(),
            integrity: test_integrity(bytes),
            time_ms,
            size_bytes: bytes.len() as u64,
        }
    }

    fn test_blob(label: &str, bytes: &[u8], refcount: usize) -> CacheBlob {
        CacheBlob {
            integrity: test_integrity(bytes),
            path: PathBuf::from(format!("content-v2/mock/{label}.blob")),
            size_bytes: bytes.len() as u64,
            refcount,
        }
    }

    fn test_snapshot(
        cache_path: impl Into<PathBuf>,
        entries: Vec<CacheEntry>,
        blobs: Vec<CacheBlob>,
    ) -> CacheSnapshot {
        CacheSnapshot {
            cache_path: cache_path.into(),
            entries,
            blobs,
        }
    }

    fn test_config(
        cache_root: impl Into<PathBuf>,
        max_size: u64,
        min_disk_free: DiskFreeThreshold,
    ) -> ResolvedCacheConfig {
        ResolvedCacheConfig {
            cache_root: cache_root.into(),
            max_size,
            min_disk_free,
            max_age: Duration::from_secs(86_400),
            origins: CacheConfigOrigins {
                cache_root: ConfigOrigin::Default,
                max_size: ConfigOrigin::Default,
                min_disk_free: ConfigOrigin::Default,
                max_age: ConfigOrigin::Default,
            },
        }
    }

    #[test]
    fn requested_health_sources_are_exact_case_insensitive_and_deduplicated() {
        let selected =
            super::select_sources(&[" mygene ".into(), "MYGENE".into(), "MyVariant".into()])
                .unwrap();
        assert_eq!(
            selected.iter().map(|source| source.api).collect::<Vec<_>>(),
            ["MyGene", "MyVariant"]
        );
    }

    #[test]
    fn unknown_health_source_returns_bounded_canonical_suggestions() {
        let error = super::select_sources(&["not-a-provider".into()]).unwrap_err();
        let message = error.to_string();
        assert!(message.contains("Unknown health API"));
        assert!(message.contains("Canonical suggestions"));
        assert!(message.split(',').count() <= 3);
    }

    #[test]
    fn health_json_exposes_automation_exit_policy() {
        let report = super::HealthReport {
            healthy: 0,
            warning: 0,
            excluded: 0,
            error: 1,
            total: 1,
            rows: Vec::new(),
        };
        let output: serde_json::Value =
            serde_json::from_str(&report.to_json(true).unwrap()).unwrap();
        assert_eq!(output["exit_policy"], "fail_on_error");
        assert_eq!(output["ok"], false);
    }

    mod catalog;
    mod http;
    mod local;
    mod runner;
}
