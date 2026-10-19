//! The cache CLI observes committed reusable entries and preserves saved artifacts.
use super::*;
use crate::cache::{CacheConfigOrigins, ConfigOrigin, DiskFreeThreshold, ResolvedCacheConfig};
use crate::test_support::TempDirGuard;

fn config(root: &std::path::Path) -> ResolvedCacheConfig {
    ResolvedCacheConfig {
        cache_root: root.to_path_buf(),
        max_size: 10000000,
        min_disk_free: DiskFreeThreshold::Bytes(1),
        max_age: Duration::from_secs(86400),
        origins: CacheConfigOrigins {
            cache_root: ConfigOrigin::File,
            max_size: ConfigOrigin::Env,
            min_disk_free: ConfigOrigin::Default,
            max_age: ConfigOrigin::Default,
        },
    }
}

#[test]
fn cache_path_names_database_without_creating_it() {
    let root = TempDirGuard::new("sqlite-cache-path");
    let child = root.path().join("missing");
    assert_eq!(
        render_path_for_config(&config(&child)),
        child.join("biomcp.db").display().to_string()
    );
    assert!(!child.exists());
}

#[test]
fn cache_stats_observe_committed_http_and_artifact_rows_and_legacy_sizes() {
    let root = TempDirGuard::new("sqlite-cache-stats");
    let config = config(root.path());
    let mut store = crate::cache::sqlite::Store::open(root.path()).unwrap();
    store
        .put("http", "reply", "reply", b"metadata", b"payload", None)
        .unwrap();
    store
        .put("citation", "edge", "edge", &[], b"evidence", None)
        .unwrap();
    let capture = crate::cache::ProviderCaptureStore::new(root.path())
        .capture_bytes(
            crate::cache::ProviderCaptureProvider::Cspec,
            "text/plain",
            b"source bytes",
        )
        .unwrap();
    std::fs::create_dir(root.path().join("http")).unwrap();
    std::fs::write(root.path().join("http/old"), b"legacy").unwrap();
    let report = collect_cache_stats_for_config(&config).unwrap();
    assert_eq!(report.http_entry_count, 1);
    assert!(report.provider_capture_bytes >= capture.byte_length);
    assert_eq!(report.legacy_cache_bytes["http"], 6);
    assert_eq!(report.max_size_origin, CacheStatsOrigin::Env);
    assert_eq!(report.max_age_origin, CacheStatsOrigin::Default);
    assert_eq!(
        report.path,
        root.path().join("biomcp.db").display().to_string()
    );
    assert_eq!(
        report.blob_bytes,
        report.referenced_blob_bytes + report.provider_capture_bytes
    );
    assert!(report.age_range.is_some());
    let json = serde_json::to_value(&report).unwrap();
    assert_eq!(json["max_size_origin"], "env");
    assert!(report.to_markdown().contains("| Legacy cache bytes |"));
}

#[test]
fn managed_clear_removes_reusable_rows_and_preserves_saved_and_legacy_files() {
    let root = TempDirGuard::new("sqlite-cache-clear");
    let config = config(root.path());
    let mut store = crate::cache::sqlite::Store::open(root.path()).unwrap();
    store
        .put("citation", "edge", "edge", &[], b"evidence", None)
        .unwrap();
    let capture = crate::cache::ProviderCaptureStore::new(root.path())
        .capture_bytes(
            crate::cache::ProviderCaptureProvider::Cspec,
            "text/plain",
            b"saved capture",
        )
        .unwrap();
    for name in ["downloads", "citation-evidence"] {
        std::fs::create_dir(root.path().join(name)).unwrap();
        std::fs::write(root.path().join(name).join("keep"), b"saved").unwrap();
    }
    let report = execute_managed_clear(&config).unwrap();
    assert_eq!(report.bytes_freed, Some(8));
    assert_eq!(report.entries_removed, 1);
    assert!(store.get("citation", "edge").unwrap().is_none());
    assert_eq!(
        crate::cache::ProviderCaptureStore::new(root.path())
            .read(&capture.capture_id)
            .unwrap(),
        b"saved capture"
    );
    for name in ["downloads", "citation-evidence"] {
        assert_eq!(
            std::fs::read(root.path().join(name).join("keep")).unwrap(),
            b"saved"
        );
    }
}

#[test]
fn reusable_size_limits_and_clean_preserve_saved_captures() {
    let root = TempDirGuard::new("sqlite-capture-retention");
    let mut config = config(root.path());
    config.max_size = 1;
    let captures = crate::cache::ProviderCaptureStore::new(root.path());
    let capture = captures
        .capture_bytes(
            crate::cache::ProviderCaptureProvider::Cspec,
            "text/plain",
            b"saved capture",
        )
        .unwrap();
    let mut store = crate::cache::sqlite::Store::open(root.path()).unwrap();
    store
        .put("http", "reply", "reply", b"", b"reusable", None)
        .unwrap();
    for mode in [
        crate::cache::sqlite::Maintenance::Pressure,
        crate::cache::sqlite::Maintenance::Clean,
    ] {
        crate::cache::sqlite::maintain(&mut store, &config, None, None, false, mode).unwrap();
        assert_eq!(
            captures.read(&capture.capture_id).unwrap(),
            b"saved capture"
        );
        assert!(store.get("http", "reply").unwrap().is_none());
    }
    execute_managed_clear(&config).unwrap();
    assert_eq!(
        captures.read(&capture.capture_id).unwrap(),
        b"saved capture"
    );
}
