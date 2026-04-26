//! Local data and cache readiness tests for `biomcp health`.

use std::io;
use std::path::PathBuf;

use super::super::catalog::{
    CVX_LOCAL_DATA_AFFECTS, EMA_LOCAL_DATA_AFFECTS, GTR_LOCAL_DATA_AFFECTS,
    WHO_IVD_LOCAL_DATA_AFFECTS, WHO_LOCAL_DATA_AFFECTS,
};
use super::super::local::{
    check_cache_dir, check_cache_limits_with, cvx_local_data_outcome, ema_local_data_outcome,
    gtr_local_data_outcome, probe_cache_dir, who_ivd_local_data_outcome, who_local_data_outcome,
};
use super::super::runner::{ProbeClass, report_from_outcomes};
use super::{
    assert_cache_dir_affects, assert_millisecond_latency, block_on, env_lock, fixture_ema_root,
    set_fresh_ema_mtimes, set_stale_ema_mtimes, set_stale_mtime, set_stale_mtime_with_age,
    test_blob, test_config, test_entry, test_snapshot, write_cvx_files, write_ema_files,
    write_gtr_files, write_who_files, write_who_ivd_files,
};
use crate::cache::{CachePlannerError, DiskFreeThreshold, FilesystemSpace};
use crate::test_support::{TempDirGuard, set_env_var};
#[test]
fn ema_local_data_not_configured_when_default_root_is_empty() {
    let root = TempDirGuard::new("health");

    let outcome = ema_local_data_outcome(root.path(), false);

    assert_eq!(outcome.class, ProbeClass::Excluded);
    assert_eq!(
        outcome.row.api,
        format!("EMA local data ({})", root.path().display())
    );
    assert_eq!(outcome.row.status, "not configured");
    assert_eq!(outcome.row.latency, "n/a");
    assert_eq!(outcome.row.affects.as_deref(), Some(EMA_LOCAL_DATA_AFFECTS));
}

#[test]
fn ema_local_data_errors_when_default_root_is_partial() {
    let root = TempDirGuard::new("health");
    write_ema_files(root.path(), &[crate::sources::ema::EMA_REQUIRED_FILES[0]]);

    let outcome = ema_local_data_outcome(root.path(), false);

    assert_eq!(outcome.class, ProbeClass::Error);
    assert_eq!(
        outcome.row.status,
        format!(
            "error (missing: {})",
            crate::sources::ema::EMA_REQUIRED_FILES[1..].join(", ")
        )
    );
    assert_eq!(outcome.row.affects.as_deref(), Some(EMA_LOCAL_DATA_AFFECTS));
}

#[test]
fn ema_local_data_errors_when_env_root_is_missing_files() {
    let root = TempDirGuard::new("health");

    let outcome = ema_local_data_outcome(root.path(), true);

    assert_eq!(outcome.class, ProbeClass::Error);
    assert_eq!(
        outcome.row.status,
        format!(
            "error (missing: {})",
            crate::sources::ema::EMA_REQUIRED_FILES.join(", ")
        )
    );
    assert_eq!(outcome.row.affects.as_deref(), Some(EMA_LOCAL_DATA_AFFECTS));
}

#[test]
fn ema_local_data_reports_available_when_default_root_is_complete() {
    let fixture_root = fixture_ema_root();
    set_stale_ema_mtimes(fixture_root.path());
    set_fresh_ema_mtimes(fixture_root.path());

    let outcome = ema_local_data_outcome(fixture_root.path(), false);

    assert_eq!(outcome.class, ProbeClass::Healthy);
    assert_eq!(
        outcome.row.api,
        format!("EMA local data ({})", fixture_root.path().display())
    );
    assert_eq!(outcome.row.status, "available (default path)");
    assert_eq!(outcome.row.latency, "n/a");
    assert_eq!(outcome.row.affects, None);
}

#[test]
fn ema_local_data_reports_configured_when_env_root_is_complete() {
    let fixture_root = fixture_ema_root();
    set_stale_ema_mtimes(fixture_root.path());
    set_fresh_ema_mtimes(fixture_root.path());

    let outcome = ema_local_data_outcome(fixture_root.path(), true);

    assert_eq!(outcome.class, ProbeClass::Healthy);
    assert_eq!(outcome.row.status, "configured");
    assert_eq!(outcome.row.affects, None);
}

#[test]
fn ema_local_data_json_reports_healthy_row_without_affects() {
    let fixture_root = fixture_ema_root();
    set_stale_ema_mtimes(fixture_root.path());
    set_fresh_ema_mtimes(fixture_root.path());
    let report = report_from_outcomes(vec![ema_local_data_outcome(fixture_root.path(), false)]);

    let value = serde_json::to_value(&report).expect("serialize health report");
    let rows = value["rows"].as_array().expect("rows array");
    let row = rows.first().expect("EMA row");

    assert_eq!(
        row["api"],
        format!("EMA local data ({})", fixture_root.path().display())
    );
    assert_eq!(row["status"], "available (default path)");
    assert_eq!(row["latency"], "n/a");
    assert!(row.get("affects").is_none());
    assert!(row.get("key_configured").is_none());
}

#[test]
fn ema_local_data_json_reports_error_row_with_affects() {
    let root = TempDirGuard::new("health");
    write_ema_files(root.path(), &[crate::sources::ema::EMA_REQUIRED_FILES[0]]);
    let report = report_from_outcomes(vec![ema_local_data_outcome(root.path(), false)]);

    let value = serde_json::to_value(&report).expect("serialize health report");
    let rows = value["rows"].as_array().expect("rows array");
    let row = rows.first().expect("EMA row");

    assert_eq!(
        row["status"],
        format!(
            "error (missing: {})",
            crate::sources::ema::EMA_REQUIRED_FILES[1..].join(", ")
        )
    );
    assert_eq!(row["affects"], EMA_LOCAL_DATA_AFFECTS);
    assert!(row.get("key_configured").is_none());
}

#[test]
fn cvx_local_data_not_configured_when_default_root_is_empty() {
    let root = TempDirGuard::new("health");

    let outcome = cvx_local_data_outcome(root.path(), false);

    assert_eq!(outcome.class, ProbeClass::Excluded);
    assert_eq!(
        outcome.row.api,
        format!("CDC CVX/MVX local data ({})", root.path().display())
    );
    assert_eq!(outcome.row.status, "not configured");
    assert_eq!(outcome.row.latency, "n/a");
    assert_eq!(outcome.row.affects.as_deref(), Some(CVX_LOCAL_DATA_AFFECTS));
}

#[test]
fn cvx_local_data_errors_when_default_root_is_partial() {
    let root = TempDirGuard::new("health");
    write_cvx_files(root.path(), &[crate::sources::cvx::CVX_FILE]);

    let outcome = cvx_local_data_outcome(root.path(), false);

    assert_eq!(outcome.class, ProbeClass::Error);
    assert_eq!(
        outcome.row.status,
        format!(
            "error (missing: {})",
            crate::sources::cvx::CVX_REQUIRED_FILES[1..].join(", ")
        )
    );
    assert_eq!(outcome.row.affects.as_deref(), Some(CVX_LOCAL_DATA_AFFECTS));
}

#[test]
fn cvx_local_data_reports_available_when_default_root_is_complete() {
    let root = TempDirGuard::new("health");
    write_cvx_files(root.path(), crate::sources::cvx::CVX_REQUIRED_FILES);

    let outcome = cvx_local_data_outcome(root.path(), false);

    assert_eq!(outcome.class, ProbeClass::Healthy);
    assert_eq!(
        outcome.row.api,
        format!("CDC CVX/MVX local data ({})", root.path().display())
    );
    assert_eq!(outcome.row.status, "available (default path)");
    assert_eq!(outcome.row.affects, None);
}

#[test]
fn cvx_local_data_reports_configured_stale_when_env_root_is_complete_but_old() {
    let root = TempDirGuard::new("health");
    write_cvx_files(root.path(), crate::sources::cvx::CVX_REQUIRED_FILES);
    set_stale_mtime_with_age(
        &root.path().join(crate::sources::cvx::MVX_FILE),
        crate::sources::cvx::CVX_STALE_AFTER + std::time::Duration::from_secs(60),
    );

    let outcome = cvx_local_data_outcome(root.path(), true);

    assert_eq!(outcome.class, ProbeClass::Warning);
    assert_eq!(outcome.row.status, "configured (stale)");
    assert_eq!(outcome.row.affects.as_deref(), Some(CVX_LOCAL_DATA_AFFECTS));
}

#[test]
fn who_local_data_not_configured_when_default_root_is_empty() {
    let root = TempDirGuard::new("health");

    let outcome = who_local_data_outcome(root.path(), false);

    assert_eq!(outcome.class, ProbeClass::Excluded);
    assert_eq!(
        outcome.row.api,
        format!(
            "WHO Prequalification local data ({})",
            root.path().display()
        )
    );
    assert_eq!(outcome.row.status, "not configured");
    assert_eq!(outcome.row.affects.as_deref(), Some(WHO_LOCAL_DATA_AFFECTS));
}

#[test]
fn who_local_data_errors_when_env_root_is_missing_file() {
    let root = TempDirGuard::new("health");

    let outcome = who_local_data_outcome(root.path(), true);

    assert_eq!(outcome.class, ProbeClass::Error);
    assert_eq!(
        outcome.row.status,
        format!(
            "error (missing: {})",
            crate::sources::who_pq::WHO_PQ_REQUIRED_FILES.join(", ")
        )
    );
    assert_eq!(outcome.row.affects.as_deref(), Some(WHO_LOCAL_DATA_AFFECTS));
}

#[test]
fn who_local_data_reports_available_when_default_root_is_complete() {
    let root = TempDirGuard::new("health");
    write_who_files(root.path(), crate::sources::who_pq::WHO_PQ_REQUIRED_FILES);

    let outcome = who_local_data_outcome(root.path(), false);

    assert_eq!(outcome.class, ProbeClass::Healthy);
    assert_eq!(
        outcome.row.api,
        format!(
            "WHO Prequalification local data ({})",
            root.path().display()
        )
    );
    assert_eq!(outcome.row.status, "available (default path)");
    assert_eq!(outcome.row.affects, None);
}

#[test]
fn who_local_data_reports_configured_when_env_root_is_complete() {
    let root = TempDirGuard::new("health");
    write_who_files(root.path(), crate::sources::who_pq::WHO_PQ_REQUIRED_FILES);

    let outcome = who_local_data_outcome(root.path(), true);

    assert_eq!(outcome.class, ProbeClass::Healthy);
    assert_eq!(outcome.row.status, "configured");
    assert_eq!(outcome.row.affects, None);
}

#[test]
fn who_local_data_reports_configured_stale_when_env_root_is_complete_but_old() {
    let root = TempDirGuard::new("health");
    write_who_files(root.path(), crate::sources::who_pq::WHO_PQ_REQUIRED_FILES);
    set_stale_mtime(
        &root
            .path()
            .join(crate::sources::who_pq::WHO_PQ_API_CSV_FILE),
    );

    let outcome = who_local_data_outcome(root.path(), true);

    assert_eq!(outcome.class, ProbeClass::Warning);
    assert_eq!(outcome.row.status, "configured (stale)");
    assert_eq!(outcome.row.affects.as_deref(), Some(WHO_LOCAL_DATA_AFFECTS));
}

#[test]
fn who_local_data_reports_default_path_stale_when_complete_but_old() {
    let root = TempDirGuard::new("health");
    write_who_files(root.path(), crate::sources::who_pq::WHO_PQ_REQUIRED_FILES);
    set_stale_mtime(
        &root
            .path()
            .join(crate::sources::who_pq::WHO_PQ_API_CSV_FILE),
    );

    let outcome = who_local_data_outcome(root.path(), false);

    assert_eq!(outcome.class, ProbeClass::Warning);
    assert_eq!(outcome.row.status, "available (default path, stale)");
    assert_eq!(outcome.row.affects.as_deref(), Some(WHO_LOCAL_DATA_AFFECTS));
}

#[test]
fn who_local_data_errors_when_only_api_file_is_missing() {
    let root = TempDirGuard::new("health");
    write_who_files(
        root.path(),
        &[
            crate::sources::who_pq::WHO_PQ_CSV_FILE,
            crate::sources::who_pq::WHO_VACCINES_CSV_FILE,
        ],
    );

    let outcome = who_local_data_outcome(root.path(), true);

    assert_eq!(outcome.class, ProbeClass::Error);
    assert_eq!(outcome.row.status, "error (missing: who_api.csv)");
    assert_eq!(outcome.row.affects.as_deref(), Some(WHO_LOCAL_DATA_AFFECTS));
}

#[test]
fn who_local_data_errors_when_only_vaccine_file_is_missing() {
    let root = TempDirGuard::new("health");
    write_who_files(
        root.path(),
        &[
            crate::sources::who_pq::WHO_PQ_CSV_FILE,
            crate::sources::who_pq::WHO_PQ_API_CSV_FILE,
        ],
    );

    let outcome = who_local_data_outcome(root.path(), true);

    assert_eq!(outcome.class, ProbeClass::Error);
    assert_eq!(outcome.row.status, "error (missing: who_vaccines.csv)");
    assert_eq!(outcome.row.affects.as_deref(), Some(WHO_LOCAL_DATA_AFFECTS));
}

#[test]
fn who_ivd_local_data_not_configured_when_default_root_is_empty() {
    let root = TempDirGuard::new("health");

    let outcome = who_ivd_local_data_outcome(root.path(), false);

    assert_eq!(outcome.class, ProbeClass::Excluded);
    assert_eq!(
        outcome.row.api,
        format!("WHO IVD local data ({})", root.path().display())
    );
    assert_eq!(outcome.row.status, "not configured");
    assert_eq!(
        outcome.row.affects.as_deref(),
        Some(WHO_IVD_LOCAL_DATA_AFFECTS)
    );
}

#[test]
fn who_ivd_local_data_errors_when_env_root_is_missing_file() {
    let root = TempDirGuard::new("health");

    let outcome = who_ivd_local_data_outcome(root.path(), true);

    assert_eq!(outcome.class, ProbeClass::Error);
    assert_eq!(
        outcome.row.status,
        format!(
            "error (missing: {})",
            crate::sources::who_ivd::WHO_IVD_REQUIRED_FILES.join(", ")
        )
    );
    assert_eq!(
        outcome.row.affects.as_deref(),
        Some(WHO_IVD_LOCAL_DATA_AFFECTS)
    );
}

#[test]
fn who_ivd_local_data_reports_available_when_default_root_is_complete() {
    let root = TempDirGuard::new("health");
    write_who_ivd_files(root.path(), crate::sources::who_ivd::WHO_IVD_REQUIRED_FILES);

    let outcome = who_ivd_local_data_outcome(root.path(), false);

    assert_eq!(outcome.class, ProbeClass::Healthy);
    assert_eq!(
        outcome.row.api,
        format!("WHO IVD local data ({})", root.path().display())
    );
    assert_eq!(outcome.row.status, "available (default path)");
    assert_eq!(outcome.row.affects, None);
}

#[test]
fn who_ivd_local_data_reports_configured_when_env_root_is_complete() {
    let root = TempDirGuard::new("health");
    write_who_ivd_files(root.path(), crate::sources::who_ivd::WHO_IVD_REQUIRED_FILES);

    let outcome = who_ivd_local_data_outcome(root.path(), true);

    assert_eq!(outcome.class, ProbeClass::Healthy);
    assert_eq!(outcome.row.status, "configured");
    assert_eq!(outcome.row.affects, None);
}

#[test]
fn who_ivd_local_data_reports_configured_stale_when_env_root_is_complete_but_old() {
    let root = TempDirGuard::new("health");
    write_who_ivd_files(root.path(), crate::sources::who_ivd::WHO_IVD_REQUIRED_FILES);
    set_stale_mtime(&root.path().join(crate::sources::who_ivd::WHO_IVD_CSV_FILE));

    let outcome = who_ivd_local_data_outcome(root.path(), true);

    assert_eq!(outcome.class, ProbeClass::Warning);
    assert_eq!(outcome.row.status, "configured (stale)");
    assert_eq!(
        outcome.row.affects.as_deref(),
        Some(WHO_IVD_LOCAL_DATA_AFFECTS)
    );
}

#[test]
fn gtr_local_data_not_configured_when_default_root_is_empty() {
    let root = TempDirGuard::new("health");

    let outcome = gtr_local_data_outcome(root.path(), false);

    assert_eq!(outcome.class, ProbeClass::Excluded);
    assert_eq!(
        outcome.row.api,
        format!("GTR local data ({})", root.path().display())
    );
    assert_eq!(outcome.row.status, "not configured");
    assert_eq!(outcome.row.affects.as_deref(), Some(GTR_LOCAL_DATA_AFFECTS));
}

#[test]
fn gtr_local_data_errors_when_default_root_is_partial() {
    let root = TempDirGuard::new("health");
    write_gtr_files(root.path(), &[crate::sources::gtr::GTR_TEST_VERSION_FILE]);

    let outcome = gtr_local_data_outcome(root.path(), false);

    assert_eq!(outcome.class, ProbeClass::Error);
    assert_eq!(
        outcome.row.status,
        format!(
            "error (missing: {})",
            crate::sources::gtr::GTR_CONDITION_GENE_FILE
        )
    );
    assert_eq!(outcome.row.affects.as_deref(), Some(GTR_LOCAL_DATA_AFFECTS));
}

#[test]
fn gtr_local_data_reports_available_when_default_root_is_complete() {
    let root = TempDirGuard::new("health");
    write_gtr_files(root.path(), &crate::sources::gtr::GTR_REQUIRED_FILES);

    let outcome = gtr_local_data_outcome(root.path(), false);

    assert_eq!(outcome.class, ProbeClass::Healthy);
    assert_eq!(outcome.row.status, "available (default path)");
    assert_eq!(outcome.row.affects, None);
}

#[test]
fn gtr_local_data_reports_configured_stale_when_env_root_is_complete_but_old() {
    let root = TempDirGuard::new("health");
    write_gtr_files(root.path(), &crate::sources::gtr::GTR_REQUIRED_FILES);
    set_stale_mtime_with_age(
        &root.path().join(crate::sources::gtr::GTR_TEST_VERSION_FILE),
        crate::sources::gtr::GTR_STALE_AFTER + std::time::Duration::from_secs(60),
    );

    let outcome = gtr_local_data_outcome(root.path(), true);

    assert_eq!(outcome.class, ProbeClass::Warning);
    assert_eq!(outcome.row.status, "configured (stale)");
    assert_eq!(outcome.row.affects.as_deref(), Some(GTR_LOCAL_DATA_AFFECTS));
}

#[test]
fn check_cache_limits_within_limits_returns_healthy_row() {
    let config = test_config("/tmp/cache", 1_024, DiskFreeThreshold::Percent(10));
    let snapshot = test_snapshot(
        "/tmp/cache/http",
        vec![test_entry("retained", b"live-bytes", 100)],
        vec![test_blob("retained", b"live-bytes", 1)],
    );

    let outcome = check_cache_limits_with(
        || Ok(config),
        |_| Ok(snapshot.clone()),
        |_| {
            Ok(FilesystemSpace {
                available_bytes: 90,
                total_bytes: 100,
            })
        },
    );

    assert_eq!(outcome.class, ProbeClass::Healthy);
    assert_eq!(outcome.row.api, "Cache limits");
    assert_eq!(outcome.row.status, "ok");
    assert_eq!(outcome.row.latency, "within limits");
}

#[test]
fn check_cache_limits_warns_when_referenced_bytes_exceed_max_size() {
    let config = test_config("/tmp/cache", 5, DiskFreeThreshold::Percent(10));
    let snapshot = test_snapshot(
        "/tmp/cache/http",
        vec![
            test_entry("old", b"abcde", 100),
            test_entry("new", b"fghij", 200),
        ],
        vec![test_blob("old", b"abcde", 1), test_blob("new", b"fghij", 1)],
    );

    let outcome = check_cache_limits_with(
        || Ok(config),
        |_| Ok(snapshot.clone()),
        |_| {
            Ok(FilesystemSpace {
                available_bytes: 90,
                total_bytes: 100,
            })
        },
    );

    assert_eq!(outcome.class, ProbeClass::Warning);
    assert_eq!(outcome.row.status, "warning");
    assert!(outcome.row.latency.contains("referenced bytes"));
    assert!(outcome.row.latency.contains("biomcp cache clean"));
}

#[test]
fn check_cache_limits_warns_when_disk_floor_is_violated() {
    let config = test_config("/tmp/cache", 1_024, DiskFreeThreshold::Percent(20));
    let snapshot = test_snapshot(
        "/tmp/cache/http",
        vec![test_entry("retained", b"live-bytes", 100)],
        vec![test_blob("retained", b"live-bytes", 1)],
    );

    let outcome = check_cache_limits_with(
        || Ok(config),
        |_| Ok(snapshot.clone()),
        |_| {
            Ok(FilesystemSpace {
                available_bytes: 10,
                total_bytes: 100,
            })
        },
    );

    assert_eq!(outcome.class, ProbeClass::Warning);
    assert_eq!(outcome.row.status, "warning");
    assert!(outcome.row.latency.contains("available disk"));
    assert!(outcome.row.latency.contains("biomcp cache clean"));
}

#[test]
fn check_cache_limits_reports_snapshot_errors_as_error_rows() {
    let config = test_config("/tmp/cache", 1_024, DiskFreeThreshold::Percent(10));

    let outcome = check_cache_limits_with(
        || Ok(config),
        |_| {
            Err(CachePlannerError::Io {
                path: PathBuf::from("/tmp/cache/http"),
                source: io::Error::other("boom"),
            })
        },
        |_| {
            Ok(FilesystemSpace {
                available_bytes: 90,
                total_bytes: 100,
            })
        },
    );

    assert_eq!(outcome.class, ProbeClass::Error);
    assert_eq!(outcome.row.api, "Cache limits");
    assert_eq!(outcome.row.status, "error");
    assert!(outcome.row.latency.contains("boom"));
}

#[test]
fn check_cache_dir_success_row_uses_resolved_path_and_ok_contract() {
    let _lock = env_lock();
    let root = TempDirGuard::new("health");
    let cache_home = root.path().join("cache-home");
    let config_home = root.path().join("config-home");
    let _cache_home = set_env_var("XDG_CACHE_HOME", Some(&cache_home.to_string_lossy()));
    let _config_home = set_env_var("XDG_CONFIG_HOME", Some(&config_home.to_string_lossy()));
    let _cache_dir = set_env_var("BIOMCP_CACHE_DIR", None);

    let outcome = block_on(check_cache_dir());

    assert_eq!(outcome.class, ProbeClass::Healthy);
    assert_eq!(
        outcome.row.api,
        format!("Cache dir ({})", cache_home.join("biomcp").display())
    );
    assert_eq!(outcome.row.status, "ok");
    assert_millisecond_latency(&outcome.row.latency);
    assert_eq!(outcome.row.affects, None);
    assert_eq!(outcome.row.key_configured, None);
}

#[test]
fn probe_cache_dir_failure_preserves_error_contract() {
    let root = TempDirGuard::new("health");
    let blocking_path = root.path().join("not-a-dir");
    std::fs::write(&blocking_path, b"occupied").expect("blocking file should exist");

    let outcome = block_on(probe_cache_dir(&blocking_path));

    assert_eq!(outcome.class, ProbeClass::Error);
    assert_eq!(
        outcome.row.api,
        format!("Cache dir ({})", blocking_path.display())
    );
    assert_eq!(outcome.row.status, "error");
    assert!(
        outcome.row.latency.contains("AlreadyExists")
            || outcome.row.latency.contains("NotADirectory")
            || outcome.row.latency.contains("PermissionDenied"),
        "unexpected latency: {}",
        outcome.row.latency
    );
    assert_cache_dir_affects(outcome.row.affects.as_deref());
    assert_eq!(outcome.row.key_configured, None);
}

#[test]
fn check_cache_dir_config_error_matches_pinned_contract() {
    let _lock = env_lock();
    let root = TempDirGuard::new("health");
    let cache_home = root.path().join("cache-home");
    let config_home = root.path().join("config-home");
    let config_dir = config_home.join("biomcp");
    std::fs::create_dir_all(&config_dir).expect("config dir should exist");
    let config_path = config_dir.join("cache.toml");
    std::fs::write(&config_path, "[cache]\nmax_size = 0\n").expect("cache.toml should exist");
    let _cache_home = set_env_var("XDG_CACHE_HOME", Some(&cache_home.to_string_lossy()));
    let _config_home = set_env_var("XDG_CONFIG_HOME", Some(&config_home.to_string_lossy()));
    let _cache_dir = set_env_var("BIOMCP_CACHE_DIR", None);

    let outcome = block_on(check_cache_dir());

    assert_eq!(outcome.class, ProbeClass::Error);
    assert_eq!(outcome.row.api, "Cache dir");
    assert_eq!(outcome.row.status, "error");
    assert_eq!(
        outcome.row.latency,
        format!(
            "Invalid argument: {}: [cache].max_size must be greater than 0",
            config_path.display()
        )
    );
    assert_cache_dir_affects(outcome.row.affects.as_deref());
    assert_eq!(outcome.row.key_configured, None);
}
