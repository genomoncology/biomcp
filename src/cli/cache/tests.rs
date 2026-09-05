//! Cache CLI tests.

use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::time::Duration;

use ssri::Integrity;

use super::{
    CacheStatsAgeRange, CacheStatsOrigin, CacheStatsReport, build_cache_stats_report,
    collect_cache_stats_report_with, render_path_for_config,
};
use crate::cache::{
    CacheBlob, CacheConfigOrigins, CacheEntry, CacheSnapshot, ConfigOrigin, DiskFreeThreshold,
    ProviderCaptureProvider, ProviderCaptureStore, ResolvedCacheConfig,
};

#[test]
fn render_path_for_config_appends_http_to_resolved_cache_root() {
    let config = test_config(
        "/tmp/resolved-cache",
        10_000_000_000,
        86_400,
        CacheConfigOrigins {
            cache_root: ConfigOrigin::Default,
            max_size: ConfigOrigin::Default,
            min_disk_free: ConfigOrigin::Default,
            max_age: ConfigOrigin::Default,
        },
    );

    assert_eq!(render_path_for_config(&config), "/tmp/resolved-cache/http");
}

#[test]
fn render_path_for_config_keeps_relative_cache_roots_relative() {
    let config = test_config(
        "relative-cache",
        10_000_000_000,
        86_400,
        CacheConfigOrigins {
            cache_root: ConfigOrigin::File,
            max_size: ConfigOrigin::Default,
            min_disk_free: ConfigOrigin::Default,
            max_age: ConfigOrigin::Default,
        },
    );

    assert_eq!(
        render_path_for_config(&config),
        PathBuf::from("relative-cache/http").display().to_string()
    );
}

#[test]
fn cache_stats_report_separates_provider_capture_namespace_bytes() {
    let root = crate::test_support::TempDirGuard::new("cache-stats-provider-captures");
    let manifest = ProviderCaptureStore::new(root.path())
        .capture_bytes(
            ProviderCaptureProvider::Cspec,
            "application/octet-stream",
            b"exact provider bytes",
        )
        .expect("capture provider bytes");
    let report = build_cache_stats_report(
        &test_snapshot(root.path().join("http"), Vec::new(), Vec::new()),
        &test_config(
            root.path(),
            10_000_000_000,
            86_400,
            CacheConfigOrigins {
                cache_root: ConfigOrigin::Default,
                max_size: ConfigOrigin::Default,
                min_disk_free: ConfigOrigin::Default,
                max_age: ConfigOrigin::Default,
            },
        ),
    )
    .expect("build managed cache stats");

    assert!(
        report.provider_capture_bytes >= manifest.byte_length,
        "stats must separately account for a valid provider capture"
    );
}

#[test]
fn render_path_for_config_does_not_create_or_migrate_directories() {
    let root = crate::test_support::TempDirGuard::new("cache-cli-path");
    let env_cache = root.path().join("env-cache");
    let legacy = env_cache.join("http-cacache");
    std::fs::create_dir_all(&legacy).expect("create legacy cache dir");
    let config = test_config(
        env_cache.clone(),
        10_000_000_000,
        86_400,
        CacheConfigOrigins {
            cache_root: ConfigOrigin::Env,
            max_size: ConfigOrigin::Default,
            min_disk_free: ConfigOrigin::Default,
            max_age: ConfigOrigin::Default,
        },
    );

    let rendered = render_path_for_config(&config);
    assert_eq!(rendered, env_cache.join("http").display().to_string());
    assert!(legacy.exists());
    assert!(!env_cache.join("http").exists());
}

#[test]
fn explicit_cache_maintenance_waits_for_an_active_cache_operation() {
    let root = crate::test_support::TempDirGuard::new("cache-maintenance-coordination");
    let config = test_config(
        root.path(),
        10_000_000_000,
        86_400,
        CacheConfigOrigins {
            cache_root: ConfigOrigin::Default,
            max_size: ConfigOrigin::Default,
            min_disk_free: ConfigOrigin::Default,
            max_age: ConfigOrigin::Default,
        },
    );
    let operation = crate::cache::lock_cache_operation(root.path()).expect("hold operation lock");
    let (started_tx, started_rx) = std::sync::mpsc::channel();
    let (finished_tx, finished_rx) = std::sync::mpsc::channel();
    let maintenance = std::thread::spawn(move || {
        started_tx.send(()).expect("report maintenance start");
        let result = super::execute_managed_clear(&config);
        finished_tx.send(()).expect("report maintenance finish");
        result
    });
    started_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("maintenance thread started");
    assert!(
        finished_rx
            .recv_timeout(Duration::from_millis(200))
            .is_err(),
        "maintenance must wait for the active cache operation"
    );
    drop(operation);
    maintenance
        .join()
        .expect("maintenance thread")
        .expect("managed clear");
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
    max_age_secs: u64,
    origins: CacheConfigOrigins,
) -> ResolvedCacheConfig {
    ResolvedCacheConfig {
        cache_root: cache_root.into(),
        max_size,
        min_disk_free: DiskFreeThreshold::Percent(10),
        max_age: Duration::from_secs(max_age_secs),
        origins,
    }
}

#[test]
fn build_cache_stats_report_empty_snapshot_has_zero_counts_null_age_and_default_origins() {
    let root = crate::test_support::TempDirGuard::new("cache-stats-empty");
    let cache_path = root.path().join("http");
    let snapshot = test_snapshot(&cache_path, Vec::new(), Vec::new());
    let config = test_config(
        root.path(),
        10_000_000_000,
        86_400,
        CacheConfigOrigins {
            cache_root: ConfigOrigin::Default,
            max_size: ConfigOrigin::Default,
            min_disk_free: ConfigOrigin::Default,
            max_age: ConfigOrigin::Default,
        },
    );

    let report = build_cache_stats_report(&snapshot, &config).expect("empty snapshot report");

    assert_eq!(
        report,
        CacheStatsReport {
            path: cache_path.display().to_string(),
            blob_bytes: 0,
            referenced_blob_bytes: 0,
            blob_count: 0,
            orphan_count: 0,
            http_entry_count: 0,
            session_count: 0,
            provider_capture_bytes: 0,
            age_range: None,
            max_size_bytes: 10_000_000_000,
            max_size_origin: CacheStatsOrigin::Default,
            min_disk_free: "10%".into(),
            min_disk_free_origin: CacheStatsOrigin::Default,
            max_age_secs: 86_400,
            max_age_origin: CacheStatsOrigin::Default,
        }
    );

    let json = crate::render::json::to_pretty(&report).expect("json");
    let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
    assert!(value["age_range"].is_null());
    assert_eq!(value["referenced_blob_bytes"], 0);
    assert_eq!(value["max_size_origin"], "default");
    assert_eq!(value["min_disk_free"], "10%");
    assert_eq!(value["min_disk_free_origin"], "default");
    assert_eq!(value["max_age_origin"], "default");
    assert!(
        report
            .to_markdown()
            .contains("| Referenced blob bytes | 0 |")
    );
    assert!(report.to_markdown().contains("| Age range | none |"));
}

#[test]
fn build_cache_stats_report_counts_orphans_and_includes_all_blob_bytes() {
    let root = crate::test_support::TempDirGuard::new("cache-stats-orphans");
    let snapshot = test_snapshot(
        root.path().join("http"),
        vec![test_entry("retained", b"live-bytes", 100)],
        vec![
            test_blob("retained", b"live-bytes", 1),
            test_blob("orphan", b"orphan-bytes", 0),
        ],
    );
    let config = test_config(
        root.path(),
        1_024,
        3_600,
        CacheConfigOrigins {
            cache_root: ConfigOrigin::Default,
            max_size: ConfigOrigin::Default,
            min_disk_free: ConfigOrigin::Default,
            max_age: ConfigOrigin::Default,
        },
    );

    let report = build_cache_stats_report(&snapshot, &config).expect("report");
    assert_eq!(
        report.blob_bytes,
        b"live-bytes".len() as u64 + b"orphan-bytes".len() as u64
    );
    assert_eq!(report.referenced_blob_bytes, b"live-bytes".len() as u64);
    assert_eq!(report.blob_count, 2);
    assert_eq!(report.orphan_count, 1);
}

#[test]
fn build_cache_stats_report_uses_index_entry_timestamps_only_for_age_range() {
    let root = crate::test_support::TempDirGuard::new("cache-stats-age-range");
    let snapshot = test_snapshot(
        root.path().join("http"),
        vec![
            test_entry("older", b"shared", 100),
            test_entry("newer", b"other", 500),
        ],
        vec![
            test_blob("shared", b"shared", 1),
            test_blob("other", b"other", 1),
            test_blob("orphan", b"orphan", 0),
        ],
    );
    let config = test_config(
        root.path(),
        2_048,
        7_200,
        CacheConfigOrigins {
            cache_root: ConfigOrigin::Default,
            max_size: ConfigOrigin::Default,
            min_disk_free: ConfigOrigin::Default,
            max_age: ConfigOrigin::Default,
        },
    );

    let report = build_cache_stats_report(&snapshot, &config).expect("report");
    assert_eq!(
        report.age_range,
        Some(CacheStatsAgeRange {
            oldest_ms: 100,
            newest_ms: 500,
        })
    );
    assert!(
        report
            .to_markdown()
            .lines()
            .any(|line| line == "| Age range | 100 .. 500 |")
    );
}

#[test]
fn cache_stats_report_json_serializes_env_and_file_origins_lowercase() {
    let root = crate::test_support::TempDirGuard::new("cache-stats-origins");
    let snapshot = test_snapshot(root.path().join("http"), Vec::new(), Vec::new());
    let config = test_config(
        root.path(),
        5_000,
        7_200,
        CacheConfigOrigins {
            cache_root: ConfigOrigin::Default,
            max_size: ConfigOrigin::Env,
            min_disk_free: ConfigOrigin::File,
            max_age: ConfigOrigin::File,
        },
    );

    let report = build_cache_stats_report(&snapshot, &config).expect("report");
    let json = crate::render::json::to_pretty(&report).expect("json");
    let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
    assert_eq!(value["max_size_origin"], "env");
    assert_eq!(value["min_disk_free_origin"], "file");
    assert_eq!(value["max_age_origin"], "file");
}

#[test]
fn cache_stats_report_markdown_is_heading_free_and_stable() {
    let report = CacheStatsReport {
        path: "/tmp/cache/http".into(),
        blob_bytes: 42,
        referenced_blob_bytes: 24,
        blob_count: 3,
        orphan_count: 1,
        http_entry_count: 1,
        session_count: 2,
        provider_capture_bytes: 0,
        age_range: Some(CacheStatsAgeRange {
            oldest_ms: 100,
            newest_ms: 500,
        }),
        max_size_bytes: 5_000,
        max_size_origin: CacheStatsOrigin::Env,
        min_disk_free: "10%".into(),
        min_disk_free_origin: CacheStatsOrigin::Default,
        max_age_secs: 7_200,
        max_age_origin: CacheStatsOrigin::File,
    };

    assert_eq!(
        report.to_markdown(),
        "\
| Path | /tmp/cache/http |
| Blob bytes | 42 |
| Referenced blob bytes | 24 |
| Blob files | 3 |
| Orphan blobs | 1 |
| HTTP cache entries | 1 |
| Article sessions | 2 |
| Provider capture bytes | 0 |
| Age range | 100 .. 500 |
| Max size | 5000 bytes (env) |
| Min disk free | 10% (default) |
| Max age | 7200 s (file) |
"
    );
}

#[test]
fn collect_cache_stats_report_calls_snapshot_once_for_resolved_http_path() {
    let root = crate::test_support::TempDirGuard::new("cache-stats-collector");
    let expected_path = root.path().join("http");
    let config = test_config(
        root.path(),
        5_000,
        7_200,
        CacheConfigOrigins {
            cache_root: ConfigOrigin::Default,
            max_size: ConfigOrigin::Env,
            min_disk_free: ConfigOrigin::Default,
            max_age: ConfigOrigin::File,
        },
    );
    let calls = Cell::new(0);
    let seen_path = RefCell::new(None);

    let report = collect_cache_stats_report_with(
        || Ok(config),
        |path: &Path| {
            calls.set(calls.get() + 1);
            *seen_path.borrow_mut() = Some(path.to_path_buf());
            Ok(test_snapshot(
                path.to_path_buf(),
                vec![test_entry("entry", b"blob", 100)],
                vec![test_blob("blob", b"blob", 1)],
            ))
        },
    )
    .expect("collector report");

    assert_eq!(calls.get(), 1);
    assert_eq!(seen_path.borrow().as_ref(), Some(&expected_path));
    assert_eq!(report.path, expected_path.display().to_string());
}
