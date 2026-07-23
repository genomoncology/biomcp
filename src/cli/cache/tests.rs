//! Cache CLI tests.

use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::time::Duration;

use ssri::Integrity;

use super::{
    CacheStatsAgeRange, CacheStatsOrigin, CacheStatsReport, build_cache_stats_report,
    collect_cache_stats_report, collect_cache_stats_report_with, render_path_for_config,
};
use crate::cache::{
    CacheBlob, CacheConfigOrigins, CacheEntry, CacheSnapshot, ConfigOrigin, DiskFreeThreshold,
    ResolvedCacheConfig,
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
#[serial_test::serial]
fn cache_stats_accounts_for_provider_capture_namespace_bytes() {
    let root = crate::test_support::TempDirGuard::new("cache-stats-provider-captures");
    let capture = root
        .path()
        .join("captures")
        .join("cspec")
        .join("sha256")
        .join("ab")
        .join("abcdef");
    std::fs::create_dir_all(capture.parent().expect("capture parent"))
        .expect("create capture namespace");
    std::fs::write(&capture, b"exact provider bytes").expect("seed capture bytes");

    let previous = std::env::var_os("BIOMCP_CACHE_DIR");
    // SAFETY: this serial test restores the process-wide configuration variable.
    unsafe { std::env::set_var("BIOMCP_CACHE_DIR", root.path()) };
    let report = collect_cache_stats_report().expect("collect managed cache stats");
    // SAFETY: restore the process-wide configuration variable before assertions can panic.
    unsafe {
        if let Some(value) = previous {
            std::env::set_var("BIOMCP_CACHE_DIR", value);
        } else {
            std::env::remove_var("BIOMCP_CACHE_DIR");
        }
    }

    assert_eq!(
        report.blob_bytes, 20,
        "stats must account for capture bytes"
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
    let snapshot = test_snapshot("/tmp/cache/http", Vec::new(), Vec::new());
    let config = test_config(
        "/tmp/cache",
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
            path: "/tmp/cache/http".into(),
            blob_bytes: 0,
            referenced_blob_bytes: 0,
            blob_count: 0,
            orphan_count: 0,
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
    let snapshot = test_snapshot(
        "/tmp/cache/http",
        vec![test_entry("retained", b"live-bytes", 100)],
        vec![
            test_blob("retained", b"live-bytes", 1),
            test_blob("orphan", b"orphan-bytes", 0),
        ],
    );
    let config = test_config(
        "/tmp/cache",
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
    let snapshot = test_snapshot(
        "/tmp/cache/http",
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
        "/tmp/cache",
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
    let snapshot = test_snapshot("/tmp/cache/http", Vec::new(), Vec::new());
    let config = test_config(
        "/tmp/cache",
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
| Age range | 100 .. 500 |
| Max size | 5000 bytes (env) |
| Min disk free | 10% (default) |
| Max age | 7200 s (file) |
"
    );
}

#[test]
fn collect_cache_stats_report_calls_snapshot_once_for_resolved_http_path() {
    let config = test_config(
        "/tmp/resolved-cache",
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
    assert_eq!(
        seen_path.borrow().as_ref(),
        Some(&PathBuf::from("/tmp/resolved-cache/http"))
    );
    assert_eq!(report.path, "/tmp/resolved-cache/http");
}
