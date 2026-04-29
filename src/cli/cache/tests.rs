//! Cache CLI tests.

use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::time::Duration;

use ssri::Integrity;
use tokio::sync::MutexGuard;

use super::{
    CacheStatsAgeRange, CacheStatsOrigin, CacheStatsReport, build_cache_stats_report,
    collect_cache_stats_report_with, render_path,
};
use crate::cache::{
    CacheBlob, CacheConfigOrigins, CacheEntry, CacheSnapshot, ConfigOrigin, DiskFreeThreshold,
    ResolvedCacheConfig,
};
use crate::error::BioMcpError;
use crate::test_support::{TempDirGuard, set_env_var};

fn env_lock() -> MutexGuard<'static, ()> {
    crate::test_support::env_lock().blocking_lock()
}

#[test]
fn default_path_uses_xdg_cache_home_http_subdir() {
    let _lock = env_lock();
    let root = TempDirGuard::new("default");
    let cache_home = root.path().join("cache-home");
    let config_home = root.path().join("config-home");
    std::fs::create_dir_all(&cache_home).expect("create cache home");
    std::fs::create_dir_all(&config_home).expect("create config home");
    let _cache_home = set_env_var("XDG_CACHE_HOME", Some(&cache_home.to_string_lossy()));
    let _config_home = set_env_var("XDG_CONFIG_HOME", Some(&config_home.to_string_lossy()));
    let _cache_dir = set_env_var("BIOMCP_CACHE_DIR", None);
    let _cache_size = set_env_var("BIOMCP_CACHE_MAX_SIZE", None);

    let rendered = render_path().expect("default cache path should render");
    assert_eq!(
        rendered,
        cache_home.join("biomcp").join("http").display().to_string()
    );
}

#[test]
fn env_override_uses_biomcp_cache_dir_http_subdir() {
    let _lock = env_lock();
    let root = TempDirGuard::new("env");
    let cache_home = root.path().join("cache-home");
    let config_home = root.path().join("config-home");
    let env_cache = root.path().join("env-cache");
    std::fs::create_dir_all(&cache_home).expect("create cache home");
    std::fs::create_dir_all(&config_home).expect("create config home");
    let _cache_home = set_env_var("XDG_CACHE_HOME", Some(&cache_home.to_string_lossy()));
    let _config_home = set_env_var("XDG_CONFIG_HOME", Some(&config_home.to_string_lossy()));
    let _cache_dir = set_env_var("BIOMCP_CACHE_DIR", Some(&env_cache.to_string_lossy()));
    let _cache_size = set_env_var("BIOMCP_CACHE_MAX_SIZE", None);

    let rendered = render_path().expect("env cache path should render");
    assert_eq!(rendered, env_cache.join("http").display().to_string());
}

#[test]
fn cache_toml_override_uses_configured_http_subdir() {
    let _lock = env_lock();
    let root = TempDirGuard::new("toml");
    let cache_home = root.path().join("cache-home");
    let config_dir = root.path().join("config-home").join("biomcp");
    let configured_root = root.path().join("configured-cache");
    std::fs::create_dir_all(&cache_home).expect("create cache home");
    std::fs::create_dir_all(&config_dir).expect("create config dir");
    std::fs::write(
        config_dir.join("cache.toml"),
        format!("[cache]\ndir = \"{}\"\n", configured_root.display()),
    )
    .expect("write cache.toml");
    let _cache_home = set_env_var("XDG_CACHE_HOME", Some(&cache_home.to_string_lossy()));
    let _config_home = set_env_var(
        "XDG_CONFIG_HOME",
        Some(&config_dir.parent().expect("config home").to_string_lossy()),
    );
    let _cache_dir = set_env_var("BIOMCP_CACHE_DIR", None);
    let _cache_size = set_env_var("BIOMCP_CACHE_MAX_SIZE", None);

    let rendered = render_path().expect("config cache path should render");
    assert_eq!(rendered, configured_root.join("http").display().to_string());
}

#[test]
fn relative_cache_toml_path_stays_relative() {
    let _lock = env_lock();
    let root = TempDirGuard::new("relative");
    let cache_home = root.path().join("cache-home");
    let config_dir = root.path().join("config-home").join("biomcp");
    std::fs::create_dir_all(&cache_home).expect("create cache home");
    std::fs::create_dir_all(&config_dir).expect("create config dir");
    std::fs::write(
        config_dir.join("cache.toml"),
        "[cache]\ndir = \"relative-cache\"\n",
    )
    .expect("write cache.toml");
    let _cache_home = set_env_var("XDG_CACHE_HOME", Some(&cache_home.to_string_lossy()));
    let _config_home = set_env_var(
        "XDG_CONFIG_HOME",
        Some(&config_dir.parent().expect("config home").to_string_lossy()),
    );
    let _cache_dir = set_env_var("BIOMCP_CACHE_DIR", None);
    let _cache_size = set_env_var("BIOMCP_CACHE_MAX_SIZE", None);

    let rendered = render_path().expect("relative cache path should render");
    assert_eq!(
        rendered,
        PathBuf::from("relative-cache/http").display().to_string()
    );
}

#[test]
fn malformed_cache_config_propagates_existing_error() {
    let _lock = env_lock();
    let root = TempDirGuard::new("invalid");
    let cache_home = root.path().join("cache-home");
    let config_dir = root.path().join("config-home").join("biomcp");
    std::fs::create_dir_all(&cache_home).expect("create cache home");
    std::fs::create_dir_all(&config_dir).expect("create config dir");
    std::fs::write(config_dir.join("cache.toml"), "[cache\nmax_size = 1\n")
        .expect("write invalid cache.toml");
    let _cache_home = set_env_var("XDG_CACHE_HOME", Some(&cache_home.to_string_lossy()));
    let _config_home = set_env_var(
        "XDG_CONFIG_HOME",
        Some(&config_dir.parent().expect("config home").to_string_lossy()),
    );
    let _cache_dir = set_env_var("BIOMCP_CACHE_DIR", None);
    let _cache_size = set_env_var("BIOMCP_CACHE_MAX_SIZE", None);

    let err = render_path().expect_err("invalid cache config should fail");
    let message = err.to_string();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(message.contains("cache.toml"));
}

#[test]
fn render_path_does_not_create_http_or_root_directories() {
    let _lock = env_lock();
    let root = TempDirGuard::new("no-create");
    let cache_home = root.path().join("cache-home");
    let config_home = root.path().join("config-home");
    let env_cache = root.path().join("env-cache");
    std::fs::create_dir_all(&cache_home).expect("create cache home");
    std::fs::create_dir_all(&config_home).expect("create config home");
    let _cache_home = set_env_var("XDG_CACHE_HOME", Some(&cache_home.to_string_lossy()));
    let _config_home = set_env_var("XDG_CONFIG_HOME", Some(&config_home.to_string_lossy()));
    let _cache_dir = set_env_var("BIOMCP_CACHE_DIR", Some(&env_cache.to_string_lossy()));
    let _cache_size = set_env_var("BIOMCP_CACHE_MAX_SIZE", None);

    let rendered = render_path().expect("cache path should render");
    assert_eq!(rendered, env_cache.join("http").display().to_string());
    assert!(!env_cache.exists());
    assert!(!env_cache.join("http").exists());
}

#[test]
fn render_path_does_not_migrate_legacy_http_cacache_directory() {
    let _lock = env_lock();
    let root = TempDirGuard::new("no-migrate");
    let cache_home = root.path().join("cache-home");
    let config_home = root.path().join("config-home");
    let env_cache = root.path().join("env-cache");
    let legacy = env_cache.join("http-cacache");
    std::fs::create_dir_all(&cache_home).expect("create cache home");
    std::fs::create_dir_all(&config_home).expect("create config home");
    std::fs::create_dir_all(&legacy).expect("create legacy cache dir");
    let _cache_home = set_env_var("XDG_CACHE_HOME", Some(&cache_home.to_string_lossy()));
    let _config_home = set_env_var("XDG_CONFIG_HOME", Some(&config_home.to_string_lossy()));
    let _cache_dir = set_env_var("BIOMCP_CACHE_DIR", Some(&env_cache.to_string_lossy()));
    let _cache_size = set_env_var("BIOMCP_CACHE_MAX_SIZE", None);

    let rendered = render_path().expect("cache path should render");
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
