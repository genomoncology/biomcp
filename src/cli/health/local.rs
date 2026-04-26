//! Local data and cache readiness probes for `biomcp health`.

use std::path::Path;
use std::time::{Duration, Instant};

use bytesize::ByteSize;

use crate::error::BioMcpError;

use super::HealthRow;
use super::catalog::{
    CVX_LOCAL_DATA_AFFECTS, DDINTER_LOCAL_DATA_AFFECTS, EMA_LOCAL_DATA_AFFECTS,
    GTR_LOCAL_DATA_AFFECTS, WHO_IVD_LOCAL_DATA_AFFECTS, WHO_LOCAL_DATA_AFFECTS,
};
use super::http::configured_key;
use super::runner::{ProbeClass, ProbeOutcome, health_row, outcome};

fn local_data_is_stale(root: &Path, files: &[&str], stale_after: Duration) -> bool {
    files.iter().any(|file| {
        root.join(file)
            .metadata()
            .ok()
            .and_then(|metadata| metadata.modified().ok())
            .and_then(|modified| std::time::SystemTime::now().duration_since(modified).ok())
            .is_some_and(|age| age >= stale_after)
    })
}

fn local_data_outcome<F>(
    label: &str,
    root: &Path,
    env_configured: bool,
    required_files: &[&str],
    stale_after: Duration,
    affects: &'static str,
    missing_files: F,
) -> ProbeOutcome
where
    F: for<'a> Fn(&'a Path, &[&'a str]) -> Vec<&'a str>,
{
    let api = format!("{label} ({})", root.display());
    let missing = missing_files(root, required_files);

    if missing.is_empty() {
        let stale = local_data_is_stale(root, required_files, stale_after);
        let (status, class, row_affects) = match (env_configured, stale) {
            (true, false) => ("configured".to_string(), ProbeClass::Healthy, None),
            (true, true) => (
                "configured (stale)".to_string(),
                ProbeClass::Warning,
                Some(affects),
            ),
            (false, false) => (
                "available (default path)".to_string(),
                ProbeClass::Healthy,
                None,
            ),
            (false, true) => (
                "available (default path, stale)".to_string(),
                ProbeClass::Warning,
                Some(affects),
            ),
        };
        return outcome(
            health_row(&api, status, "n/a".into(), row_affects, None),
            class,
        );
    }

    if !env_configured && missing.len() == required_files.len() {
        return outcome(
            health_row(
                &api,
                "not configured".into(),
                "n/a".into(),
                Some(affects),
                None,
            ),
            ProbeClass::Excluded,
        );
    }

    outcome(
        health_row(
            &api,
            format!("error (missing: {})", missing.join(", ")),
            "n/a".into(),
            Some(affects),
            None,
        ),
        ProbeClass::Error,
    )
}

pub(in crate::cli::health) fn ema_local_data_outcome(
    root: &Path,
    env_configured: bool,
) -> ProbeOutcome {
    local_data_outcome(
        "EMA local data",
        root,
        env_configured,
        crate::sources::ema::EMA_REQUIRED_FILES,
        crate::sources::ema::EMA_STALE_AFTER,
        EMA_LOCAL_DATA_AFFECTS,
        crate::sources::ema::ema_missing_files,
    )
}

pub(in crate::cli::health) fn check_ema_local_data() -> ProbeOutcome {
    let env_configured = configured_key("BIOMCP_EMA_DIR").is_some();
    let root = crate::sources::ema::resolve_ema_root();
    ema_local_data_outcome(&root, env_configured)
}

pub(in crate::cli::health) fn cvx_local_data_outcome(
    root: &Path,
    env_configured: bool,
) -> ProbeOutcome {
    local_data_outcome(
        "CDC CVX/MVX local data",
        root,
        env_configured,
        crate::sources::cvx::CVX_REQUIRED_FILES,
        crate::sources::cvx::CVX_STALE_AFTER,
        CVX_LOCAL_DATA_AFFECTS,
        crate::sources::cvx::cvx_missing_files,
    )
}

pub(in crate::cli::health) fn check_cvx_local_data() -> ProbeOutcome {
    let env_configured = configured_key("BIOMCP_CVX_DIR").is_some();
    let root = crate::sources::cvx::resolve_cvx_root();
    cvx_local_data_outcome(&root, env_configured)
}

pub(in crate::cli::health) fn ddinter_local_data_outcome(
    root: &Path,
    env_configured: bool,
) -> ProbeOutcome {
    local_data_outcome(
        "DDInter local data",
        root,
        env_configured,
        crate::sources::ddinter::DDINTER_REQUIRED_FILES,
        crate::sources::ddinter::DDINTER_STALE_AFTER,
        DDINTER_LOCAL_DATA_AFFECTS,
        crate::sources::ddinter::ddinter_missing_files,
    )
}

pub(in crate::cli::health) fn check_ddinter_local_data() -> ProbeOutcome {
    let env_configured = configured_key("BIOMCP_DDINTER_DIR").is_some();
    let root = crate::sources::ddinter::resolve_ddinter_root();
    ddinter_local_data_outcome(&root, env_configured)
}

pub(in crate::cli::health) fn who_local_data_outcome(
    root: &Path,
    env_configured: bool,
) -> ProbeOutcome {
    local_data_outcome(
        "WHO Prequalification local data",
        root,
        env_configured,
        crate::sources::who_pq::WHO_PQ_REQUIRED_FILES,
        crate::sources::who_pq::WHO_PQ_STALE_AFTER,
        WHO_LOCAL_DATA_AFFECTS,
        crate::sources::who_pq::who_pq_missing_files,
    )
}

pub(in crate::cli::health) fn check_who_local_data() -> ProbeOutcome {
    let env_configured = configured_key("BIOMCP_WHO_DIR").is_some();
    let root = crate::sources::who_pq::resolve_who_pq_root();
    who_local_data_outcome(&root, env_configured)
}

pub(in crate::cli::health) fn gtr_local_data_outcome(
    root: &Path,
    env_configured: bool,
) -> ProbeOutcome {
    local_data_outcome(
        "GTR local data",
        root,
        env_configured,
        &crate::sources::gtr::GTR_REQUIRED_FILES,
        crate::sources::gtr::GTR_STALE_AFTER,
        GTR_LOCAL_DATA_AFFECTS,
        |root, required_files| {
            let required = required_files.to_vec();
            crate::sources::gtr::gtr_missing_files(root)
                .into_iter()
                .filter_map(|missing| {
                    required
                        .iter()
                        .copied()
                        .find(|expected| *expected == missing.as_str())
                })
                .collect()
        },
    )
}

pub(in crate::cli::health) fn check_gtr_local_data() -> ProbeOutcome {
    let env_configured = configured_key("BIOMCP_GTR_DIR").is_some();
    let root = crate::sources::gtr::resolve_gtr_root();
    gtr_local_data_outcome(&root, env_configured)
}

pub(in crate::cli::health) fn who_ivd_local_data_outcome(
    root: &Path,
    env_configured: bool,
) -> ProbeOutcome {
    local_data_outcome(
        "WHO IVD local data",
        root,
        env_configured,
        crate::sources::who_ivd::WHO_IVD_REQUIRED_FILES,
        crate::sources::who_ivd::WHO_IVD_STALE_AFTER,
        WHO_IVD_LOCAL_DATA_AFFECTS,
        crate::sources::who_ivd::who_ivd_missing_files,
    )
}

pub(in crate::cli::health) fn check_who_ivd_local_data() -> ProbeOutcome {
    let env_configured = configured_key("BIOMCP_WHO_IVD_DIR").is_some();
    let root = crate::sources::who_ivd::resolve_who_ivd_root();
    who_ivd_local_data_outcome(&root, env_configured)
}

pub(in crate::cli::health) async fn check_cache_dir() -> ProbeOutcome {
    let dir = match crate::cache::resolve_cache_config() {
        Ok(config) => config.cache_root,
        Err(err) => {
            return outcome(
                HealthRow {
                    api: "Cache dir".into(),
                    status: "error".into(),
                    latency: err.to_string(),
                    affects: Some("local cache-backed lookups and downloads".into()),
                    key_configured: None,
                },
                ProbeClass::Error,
            );
        }
    };
    probe_cache_dir(&dir).await
}

pub(in crate::cli::health) async fn check_cache_limits() -> ProbeOutcome {
    let config = match crate::cache::resolve_cache_config() {
        Ok(config) => config,
        Err(err) => {
            return cache_limits_error_outcome(err.to_string());
        }
    };

    match tokio::task::spawn_blocking(move || {
        check_cache_limits_with(
            || Ok(config),
            crate::cache::snapshot_cache,
            crate::cache::inspect_filesystem_space,
        )
    })
    .await
    {
        Ok(outcome) => outcome,
        Err(err) => cache_limits_error_outcome(err.to_string()),
    }
}

pub(in crate::cli::health) async fn probe_cache_dir(dir: &Path) -> ProbeOutcome {
    let start = Instant::now();
    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let probe = dir.join(format!(".biomcp-healthcheck-{suffix}.tmp"));

    let result = async {
        tokio::fs::create_dir_all(&dir).await?;
        tokio::fs::write(&probe, b"ok").await?;
        match tokio::fs::remove_file(&probe).await {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err),
        }
    }
    .await;

    match result {
        Ok(()) => outcome(
            HealthRow {
                api: format!("Cache dir ({})", dir.display()),
                status: "ok".into(),
                latency: format!("{}ms", start.elapsed().as_millis()),
                affects: None,
                key_configured: None,
            },
            ProbeClass::Healthy,
        ),
        Err(err) => outcome(
            HealthRow {
                api: format!("Cache dir ({})", dir.display()),
                status: "error".into(),
                latency: format!("{:?}", err.kind()),
                affects: Some("local cache-backed lookups and downloads".into()),
                key_configured: None,
            },
            ProbeClass::Error,
        ),
    }
}

pub(in crate::cli::health) fn check_cache_limits_with<R, S, I>(
    resolve_config: R,
    snapshotter: S,
    inspect_space: I,
) -> ProbeOutcome
where
    R: FnOnce() -> Result<crate::cache::ResolvedCacheConfig, BioMcpError>,
    S: FnOnce(&Path) -> Result<crate::cache::CacheSnapshot, crate::cache::CachePlannerError>,
    I: FnOnce(&Path) -> Result<crate::cache::FilesystemSpace, BioMcpError>,
{
    let config = match resolve_config() {
        Ok(config) => config,
        Err(err) => return cache_limits_error_outcome(err.to_string()),
    };
    let cache_path = config.cache_root.join("http");
    let snapshot = match snapshotter(&cache_path) {
        Ok(snapshot) => snapshot,
        Err(err) => return cache_limits_error_outcome(err.to_string()),
    };
    let space = match inspect_space(&config.cache_root) {
        Ok(space) => space,
        Err(err) => return cache_limits_error_outcome(err.to_string()),
    };
    let evaluation = crate::cache::evaluate_cache_limits(&snapshot, &config, space);

    if evaluation.over_max_size || evaluation.below_min_disk_free {
        return outcome(
            HealthRow {
                api: "Cache limits".into(),
                status: "warning".into(),
                latency: cache_limits_warning_message(&config, space, &evaluation),
                affects: None,
                key_configured: None,
            },
            ProbeClass::Warning,
        );
    }

    outcome(
        HealthRow {
            api: "Cache limits".into(),
            status: "ok".into(),
            latency: "within limits".into(),
            affects: None,
            key_configured: None,
        },
        ProbeClass::Healthy,
    )
}

fn cache_limits_warning_message(
    config: &crate::cache::ResolvedCacheConfig,
    space: crate::cache::FilesystemSpace,
    evaluation: &crate::cache::CacheLimitEvaluation,
) -> String {
    let mut clauses = Vec::new();
    if evaluation.over_max_size {
        clauses.push(format!(
            "referenced bytes {} exceed max_size {}",
            evaluation.usage.referenced_blob_bytes, config.max_size
        ));
    }
    if evaluation.below_min_disk_free {
        clauses.push(format!(
            "available disk {} is below min_disk_free {}",
            ByteSize(space.available_bytes),
            config.min_disk_free.display()
        ));
    }
    format!("{}; run biomcp cache clean", clauses.join("; "))
}

fn cache_limits_error_outcome(message: String) -> ProbeOutcome {
    outcome(
        HealthRow {
            api: "Cache limits".into(),
            status: "error".into(),
            latency: message,
            affects: None,
            key_configured: None,
        },
        ProbeClass::Error,
    )
}
