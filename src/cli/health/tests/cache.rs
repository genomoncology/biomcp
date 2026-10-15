//! Health cache row tests: cache limits and the cache directory probe.

use std::io;
use std::path::PathBuf;

use super::super::HealthStatus;
use super::super::local::{check_cache_dir_with, check_cache_limits_with, probe_cache_dir};
use super::super::runner::ProbeClass;
use super::{
    assert_cache_dir_affects, assert_millisecond_latency, block_on, test_blob, test_config,
    test_entry, test_snapshot,
};
use crate::cache::{CachePlannerError, DiskFreeThreshold, FilesystemSpace};
use crate::error::BioMcpError;
use crate::test_support::TempDirGuard;

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
    assert_eq!(outcome.row.status, HealthStatus::Ok);
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
    assert_eq!(outcome.row.status, HealthStatus::Warning);
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
    assert_eq!(outcome.row.status, HealthStatus::Warning);
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
    assert_eq!(outcome.row.status, HealthStatus::Error);
    assert!(outcome.row.latency.contains("boom"));
}

#[test]
fn check_cache_dir_success_row_uses_resolved_path_and_ok_contract() {
    let root = TempDirGuard::new("health");
    let cache_root = root.path().join("resolved-cache");
    let config = test_config(&cache_root, 1_024, DiskFreeThreshold::Percent(10));

    let outcome = block_on(check_cache_dir_with(|| Ok(config), false));

    assert_eq!(outcome.class, ProbeClass::Healthy);
    assert_eq!(
        outcome.row.api,
        format!("Cache dir ({})", cache_root.display())
    );
    assert_eq!(outcome.row.status, HealthStatus::Ok);
    assert_millisecond_latency(&outcome.row.latency);
    assert_eq!(outcome.row.affects, None);
    assert_eq!(outcome.row.key_configured, None);
}

#[test]
fn probe_cache_dir_failure_preserves_error_contract() {
    let root = TempDirGuard::new("health");
    let blocking_path = root.path().join("not-a-dir");
    std::fs::write(&blocking_path, b"occupied").expect("blocking file should exist");

    let outcome = block_on(probe_cache_dir(&blocking_path, false));

    assert_eq!(outcome.class, ProbeClass::Error);
    assert_eq!(
        outcome.row.api,
        format!("Cache dir ({})", blocking_path.display())
    );
    assert_eq!(outcome.row.status, HealthStatus::Error);
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
fn mcp_cache_dir_row_names_the_cache_without_its_path() {
    let root = TempDirGuard::new("health");

    let outcome = block_on(probe_cache_dir(&root.path().join("cache"), true));

    assert_eq!(outcome.class, ProbeClass::Healthy);
    assert_eq!(outcome.row.api, "Cache dir");
    assert_eq!(outcome.row.local_path, None);
}

#[test]
fn check_cache_dir_config_error_matches_pinned_contract() {
    let root = TempDirGuard::new("health");
    let config_path = root.path().join("config-home/biomcp/cache.toml");
    let err = BioMcpError::InvalidArgument(format!(
        "{}: [cache].max_size must be greater than 0",
        config_path.display()
    ));

    let outcome = block_on(check_cache_dir_with(|| Err(err), false));

    assert_eq!(outcome.class, ProbeClass::Error);
    assert_eq!(outcome.row.api, "Cache dir");
    assert_eq!(outcome.row.status, HealthStatus::Error);
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
