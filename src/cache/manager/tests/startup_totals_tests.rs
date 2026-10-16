//! Kept size totals (ticket 2133): start-up seeds its size tracker from a
//! marker under the HTTP cache instead of statting every content file, the
//! daily clean rewrites the marker with an exact total, and cache mode off
//! reads nothing. Pure manager logic over a real temp cache dir.

use std::fs;
use std::path::Path;
use std::sync::atomic::Ordering;

use super::super::SizeAwareCacheManager;
use super::{test_config, write_entry};
use crate::cache::{DiskFreeThreshold, snapshot_cache, summarize_cache_usage};
use crate::test_support::{
    TempDirGuard, reset_startup_cache_file_visits, startup_cache_file_visits,
};

fn totals_marker_path(cache_path: &Path) -> std::path::PathBuf {
    cache_path.join(super::super::CONTENT_TOTALS_MARKER_FILE)
}

fn seed_entries(cache_path: &Path, count: usize) -> u64 {
    let mut total = 0_u64;
    for index in 0..count {
        let bytes = format!("totals-bytes-{index}");
        write_entry(
            cache_path,
            &format!("totals-key-{index}"),
            bytes.as_bytes(),
            1_000 + index as u128,
        );
        total += bytes.len() as u64;
    }
    total
}

#[test]
fn start_up_seeds_kept_totals_without_walking_the_tree_again() {
    let root = TempDirGuard::new("totals-kept");
    let cache_path = root.path().join("http");
    let expected = seed_entries(&cache_path, 300);

    // First construction has no marker: it walks once and records the total.
    let first = SizeAwareCacheManager::new_at(
        cache_path.clone(),
        test_config(root.path(), 1_000_000, DiskFreeThreshold::Percent(10)),
        20_000,
    )
    .expect("first manager");
    assert_eq!(first.approx_bytes.load(Ordering::Relaxed), expected);
    assert!(totals_marker_path(&cache_path).is_file(), "marker written");

    // Steady state: the kept total seeds the tracker with no file visits.
    reset_startup_cache_file_visits();
    let second = SizeAwareCacheManager::new_at(
        cache_path.clone(),
        test_config(root.path(), 1_000_000, DiskFreeThreshold::Percent(10)),
        20_500,
    )
    .expect("second manager");
    assert_eq!(second.approx_bytes.load(Ordering::Relaxed), expected);
    assert_eq!(
        startup_cache_file_visits(),
        0,
        "a start-up with kept totals must not stat content files"
    );
}

#[test]
fn the_daily_clean_rewrites_the_totals_marker_with_an_exact_total() {
    let root = TempDirGuard::new("totals-refresh");
    let cache_path = root.path().join("http");
    write_entry(&cache_path, "expired", b"old", 1_000);
    write_entry(&cache_path, "retained", b"kept", 15_000);
    let mut config = test_config(root.path(), 1_000_000, DiskFreeThreshold::Percent(1));
    config.max_age = std::time::Duration::from_secs(10);

    SizeAwareCacheManager::new_at(cache_path.clone(), config, 20_000)
        .expect("manager runs the day's clean");

    let usage = summarize_cache_usage(&snapshot_cache(&cache_path).expect("snapshot"));
    assert_eq!(
        fs::read_to_string(totals_marker_path(&cache_path)).expect("totals marker"),
        usage.referenced_blob_bytes.to_string(),
        "the marker holds the exact post-clean total"
    );
}

#[test]
fn cache_mode_off_seeds_zero_without_reading_the_cache() {
    let _mode = crate::sources::test_cache_mode::off();
    let root = TempDirGuard::new("totals-mode-off");
    let cache_path = root.path().join("http");
    seed_entries(&cache_path, 20);

    reset_startup_cache_file_visits();
    let manager = SizeAwareCacheManager::new_at(
        cache_path.clone(),
        test_config(root.path(), 1_000_000, DiskFreeThreshold::Percent(10)),
        20_000,
    )
    .expect("manager with the cache mode off");

    assert_eq!(manager.approx_bytes.load(Ordering::Relaxed), 0);
    assert_eq!(startup_cache_file_visits(), 0);
    assert!(
        !totals_marker_path(&cache_path).exists(),
        "cache mode off writes no totals marker"
    );
}
