//! Start-up clean gating: with the cache mode off construction never scans
//! the HTTP cache, and with the cache on the clean runs only when the daily
//! marker under the HTTP cache is missing or at least a day old, and each
//! clean rewrites the marker. Pure manager logic over a real temp cache dir.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::super::SizeAwareCacheManager;
use super::{test_config, write_entry};
use crate::cache::DiskFreeThreshold;
use crate::test_support::TempDirGuard;

/// The start-up clean marker's on-disk name, pinned here so a rename in the
/// manager cannot silently strand old markers.
const STARTUP_CLEAN_MARKER: &str = ".last-startup-clean";

fn marker_path(cache_path: &Path) -> PathBuf {
    cache_path.join(STARTUP_CLEAN_MARKER)
}

fn write_marker(cache_path: &Path, last_clean_ms: u128) {
    fs::write(
        marker_path(cache_path),
        last_clean_ms.to_string(),
    )
    .expect("write startup clean marker");
}

/// A config whose ten-second age limit expires the entry written at 1 ms.
fn expired_entry_config(cache_root: &Path) -> crate::cache::ResolvedCacheConfig {
    let mut config = test_config(cache_root, 1_000_000, DiskFreeThreshold::Percent(1));
    config.max_age = Duration::from_secs(10);
    config
}

fn expired_entry_survives(cache_path: &Path) -> bool {
    cacache::metadata_sync(cache_path, "expired")
        .expect("expired metadata")
        .is_some()
}

#[test]
fn cache_mode_off_skips_the_startup_clean() {
    let _mode = crate::sources::test_cache_mode::off();
    let root = TempDirGuard::new("startup-clean-mode-off");
    let cache_path = root.path().join("http");
    write_entry(&cache_path, "expired", b"old", 1_000);

    SizeAwareCacheManager::new_at(
        cache_path.clone(),
        expired_entry_config(root.path()),
        20_000,
    )
    .expect("open manager with the cache mode off");

    assert!(
        expired_entry_survives(&cache_path),
        "an expired entry survives construction with the cache mode off"
    );
    assert!(
        !marker_path(&cache_path).exists(),
        "no start-up clean marker is written with the cache mode off"
    );
}

#[test]
fn a_fresh_startup_clean_marker_skips_the_clean() {
    let root = TempDirGuard::new("startup-clean-fresh-marker");
    let cache_path = root.path().join("http");
    write_entry(&cache_path, "expired", b"old", 1_000);
    write_marker(&cache_path, 19_999);

    SizeAwareCacheManager::new_at(
        cache_path.clone(),
        expired_entry_config(root.path()),
        20_000,
    )
    .expect("open manager with a fresh marker");

    assert!(
        expired_entry_survives(&cache_path),
        "an expired entry survives construction when the marker is fresh"
    );
    assert_eq!(
        fs::read_to_string(marker_path(&cache_path)).expect("read marker"),
        "19999",
        "a skipped clean leaves the marker untouched"
    );
}

#[test]
fn a_marker_older_than_a_day_runs_the_clean_and_rewrites_the_marker() {
    let root = TempDirGuard::new("startup-clean-old-marker");
    let cache_path = root.path().join("http");
    write_entry(&cache_path, "expired", b"old", 1_000);
    let now_ms = 100_000_000;
    write_marker(&cache_path, now_ms - 25 * 60 * 60 * 1_000);

    SizeAwareCacheManager::new_at(
        cache_path.clone(),
        expired_entry_config(root.path()),
        now_ms,
    )
    .expect("open manager with a day-old marker");

    assert!(
        !expired_entry_survives(&cache_path),
        "a marker older than a day still lets the clean remove the expired entry"
    );
    assert_eq!(
        fs::read_to_string(marker_path(&cache_path)).expect("read marker"),
        now_ms.to_string(),
        "the clean rewrites the marker with its run time"
    );
}

#[test]
fn a_missing_startup_clean_marker_runs_the_clean_and_writes_the_marker() {
    let root = TempDirGuard::new("startup-clean-missing-marker");
    let cache_path = root.path().join("http");
    write_entry(&cache_path, "expired", b"old", 1_000);

    SizeAwareCacheManager::new_at(
        cache_path.clone(),
        expired_entry_config(root.path()),
        20_000,
    )
    .expect("open manager with no marker");

    assert!(
        !expired_entry_survives(&cache_path),
        "a missing marker lets the clean remove the expired entry"
    );
    assert_eq!(
        fs::read_to_string(marker_path(&cache_path)).expect("read marker"),
        "20000",
        "the clean writes the marker with its run time"
    );
}
