//! Stable facade and report rendering for `biomcp health`.

mod catalog;
mod http;
mod local;
mod runner;

use crate::error::BioMcpError;

#[derive(Debug, Clone, serde::Serialize)]
pub struct HealthRow {
    pub api: String,
    pub status: String,
    pub latency: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub affects: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_configured: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct HealthReport {
    pub healthy: usize,
    pub warning: usize,
    pub excluded: usize,
    pub total: usize,
    pub rows: Vec<HealthRow>,
}

impl HealthReport {
    pub fn all_healthy(&self) -> bool {
        self.healthy + self.warning + self.excluded == self.total
    }

    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        let show_affects = self.rows.iter().any(|row| row.affects.is_some());
        let errors = self
            .total
            .saturating_sub(self.healthy + self.warning + self.excluded);

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
            self.healthy, errors, self.excluded
        ));
        if self.warning > 0 {
            out.push_str(&format!(", {} warning", self.warning));
        }
        out.push('\n');
        out
    }
}

fn markdown_status(row: &HealthRow) -> String {
    match (row.status.as_str(), row.key_configured) {
        ("ok", Some(true)) => "ok (key configured)".to_string(),
        ("error", Some(true)) => "error (key configured)".to_string(),
        ("error", Some(false)) => "error (key not configured)".to_string(),
        _ => row.status.clone(),
    }
}

/// Runs connectivity checks for configured upstream APIs and local EMA/CVX/WHO/GTR/WHO IVD/cache readiness.
///
/// # Errors
///
/// Returns an error when the shared HTTP client cannot be created.
pub async fn check(apis_only: bool) -> Result<HealthReport, BioMcpError> {
    runner::check(apis_only).await
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use super::catalog::{ProbeKind, SourceDescriptor, health_sources};
    use crate::cache::{
        CacheBlob, CacheConfigOrigins, CacheEntry, CacheSnapshot, ConfigOrigin, DiskFreeThreshold,
        ResolvedCacheConfig,
    };
    use crate::test_support::TempDirGuard;
    use ssri::Integrity;
    use tokio::sync::MutexGuard;

    fn block_on<F: Future>(future: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("health test runtime")
            .block_on(future)
    }

    fn env_lock() -> MutexGuard<'static, ()> {
        crate::test_support::env_lock().blocking_lock()
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

    fn semantic_scholar_source(url: &'static str) -> SourceDescriptor {
        let source = health_sources()
            .iter()
            .find(|source| source.api == "Semantic Scholar")
            .expect("semantic scholar health source");
        let ProbeKind::OptionalAuthGet {
            env_var,
            header_name,
            header_value_prefix,
            unauthenticated_ok_status,
            authenticated_ok_status,
            unauthenticated_rate_limited_status,
            ..
        } = source.probe
        else {
            panic!("semantic scholar should use optional auth get");
        };

        SourceDescriptor {
            api: source.api,
            affects: source.affects,
            probe: ProbeKind::OptionalAuthGet {
                url,
                env_var,
                header_name,
                header_value_prefix,
                unauthenticated_ok_status,
                authenticated_ok_status,
                unauthenticated_rate_limited_status,
            },
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

    mod catalog;
    mod http;
    mod local;
    mod runner;
}
