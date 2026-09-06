use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use ssri::Integrity;

use super::{
    CachePlannerError, CacheSnapshot, ConfigOrigin, ResolvedCacheConfig, plan_composite_cleanup,
    snapshot_cache,
};
use crate::error::BioMcpError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CleanOptions {
    pub(crate) max_age: Option<Duration>,
    pub(crate) max_size: Option<u64>,
    pub(crate) dry_run: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct CleanReport {
    pub(crate) dry_run: bool,
    pub(crate) orphans_removed: usize,
    pub(crate) entries_removed: usize,
    pub(crate) bytes_freed: u64,
    pub(crate) provider_capture_bytes_freed: u64,
    pub(crate) errors: Vec<String>,
}

pub(crate) fn execute_cache_clean(
    cache_path: &Path,
    options: CleanOptions,
    config: &ResolvedCacheConfig,
    now_ms: u128,
) -> Result<CleanReport, BioMcpError> {
    execute_cache_clean_with(
        cache_path,
        options,
        config,
        now_ms,
        snapshot_cache,
        |path, key| cacache::remove_sync(path, key),
        |path, integrity| cacache::remove_hash_sync(path, integrity),
    )
}

fn execute_cache_clean_with<S, RK, RB>(
    cache_path: &Path,
    options: CleanOptions,
    config: &ResolvedCacheConfig,
    now_ms: u128,
    snapshotter: S,
    mut remove_key: RK,
    mut remove_blob: RB,
) -> Result<CleanReport, BioMcpError>
where
    S: FnOnce(&Path) -> Result<CacheSnapshot, CachePlannerError>,
    RK: for<'a, 'b> FnMut(&'a Path, &'b str) -> Result<(), cacache::Error>,
    RB: for<'a, 'b> FnMut(&'a Path, &'b Integrity) -> Result<(), cacache::Error>,
{
    check_variant_article_deadline()?;
    let effective_max_age = Some(options.max_age.unwrap_or(config.max_age));
    let effective_max_size =
        resolve_effective_limit(options.max_size, config.max_size, config.origins.max_size);
    let snapshot =
        snapshotter(cache_path).map_err(|err| BioMcpError::Io(std::io::Error::other(err)))?;
    let plan = plan_composite_cleanup(&snapshot, now_ms, effective_max_age, effective_max_size);

    if options.dry_run {
        return Ok(CleanReport {
            dry_run: true,
            orphans_removed: plan
                .blob_removals
                .iter()
                .filter(|blob| blob.refcount == 0)
                .count(),
            entries_removed: plan.entry_removals.len(),
            bytes_freed: plan.reclaimed_blob_bytes,
            provider_capture_bytes_freed: 0,
            errors: Vec::new(),
        });
    }

    let mut planned_key_count_by_integrity = HashMap::new();
    for entry in &plan.entry_removals {
        check_variant_article_deadline()?;
        *planned_key_count_by_integrity
            .entry(entry.integrity.clone())
            .or_insert(0usize) += 1;
    }

    let mut successful_key_count_by_integrity = HashMap::new();
    let mut entries_removed = 0usize;
    let mut orphans_removed = 0usize;
    let mut bytes_freed = 0u64;
    let mut errors = Vec::new();

    for entry in &plan.entry_removals {
        match remove_key(cache_path, &entry.key) {
            Ok(()) => {
                entries_removed += 1;
                *successful_key_count_by_integrity
                    .entry(entry.integrity.clone())
                    .or_insert(0usize) += 1;
            }
            Err(err) => errors.push(format!("failed to remove cache key '{}': {err}", entry.key)),
        }
    }

    for blob in &plan.blob_removals {
        check_variant_article_deadline()?;
        let eligible = if blob.refcount == 0 {
            true
        } else {
            successful_key_count_by_integrity
                .get(&blob.integrity)
                .copied()
                .unwrap_or_default()
                == planned_key_count_by_integrity
                    .get(&blob.integrity)
                    .copied()
                    .unwrap_or_default()
        };

        if !eligible {
            errors.push(format!(
                "skipped cache blob {} because not all planned key removals succeeded",
                blob.integrity
            ));
            continue;
        }

        match remove_blob(cache_path, &blob.integrity) {
            Ok(()) => {
                bytes_freed += blob.size_bytes;
                if blob.refcount == 0 {
                    orphans_removed += 1;
                }
            }
            Err(cacache::Error::IoError(err, _)) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => errors.push(format!(
                "failed to remove cache blob {}: {err}",
                blob.integrity
            )),
        }
    }

    Ok(CleanReport {
        dry_run: false,
        orphans_removed,
        entries_removed,
        bytes_freed,
        provider_capture_bytes_freed: 0,
        errors,
    })
}

fn check_variant_article_deadline() -> Result<(), BioMcpError> {
    if crate::sources::current_variant_article_deadline()
        .is_some_and(|deadline| deadline.is_exhausted())
    {
        Err(BioMcpError::Io(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "variant article invocation deadline exceeded",
        )))
    } else {
        Ok(())
    }
}

fn resolve_effective_limit<T: Copy>(
    option: Option<T>,
    config_value: T,
    origin: ConfigOrigin,
) -> Option<T> {
    option.or_else(|| (origin != ConfigOrigin::Default).then_some(config_value))
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::io;
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::time::Duration;

    use ssri::Integrity;

    use super::{CleanOptions, CleanReport, execute_cache_clean, execute_cache_clean_with};
    use crate::cache::{
        CacheConfigOrigins, CachePlannerError, ConfigOrigin, DiskFreeThreshold,
        ResolvedCacheConfig, snapshot_cache,
    };
    use crate::error::BioMcpError;
    use crate::test_support::TempDirGuard;

    fn write_entry(cache_path: &Path, key: &str, bytes: &[u8], time_ms: u128) -> Integrity {
        let mut writer = cacache::WriteOpts::new()
            .size(bytes.len())
            .time(time_ms)
            .open_sync(cache_path, key)
            .expect("open writer");
        writer.write_all(bytes).expect("write bytes");
        writer.commit().expect("commit writer")
    }

    fn make_orphan(cache_path: &Path, key: &str, bytes: &[u8], time_ms: u128) -> Integrity {
        let integrity = write_entry(cache_path, key, bytes, time_ms);
        cacache::remove_sync(cache_path, key).expect("tombstone key");
        integrity
    }

    fn blob_path_for_integrity(cache_path: &Path, integrity: &Integrity) -> PathBuf {
        let (algorithm, hex) = integrity.to_hex();
        cache_path
            .join("content-v2")
            .join(algorithm.to_string())
            .join(&hex[0..2])
            .join(&hex[2..4])
            .join(&hex[4..])
    }

    fn test_config(
        cache_root: &Path,
        max_size: u64,
        max_age: Duration,
        max_size_origin: ConfigOrigin,
        max_age_origin: ConfigOrigin,
    ) -> ResolvedCacheConfig {
        ResolvedCacheConfig {
            cache_root: cache_root.to_path_buf(),
            max_size,
            min_disk_free: DiskFreeThreshold::Percent(10),
            max_age,
            origins: CacheConfigOrigins {
                cache_root: ConfigOrigin::Default,
                max_size: max_size_origin,
                min_disk_free: ConfigOrigin::Default,
                max_age: max_age_origin,
            },
        }
    }

    fn snapshot_keys(cache_path: &Path) -> Vec<String> {
        snapshot_cache(cache_path)
            .expect("snapshot cache")
            .entries
            .into_iter()
            .map(|entry| entry.key)
            .collect()
    }

    fn seed_parity_fixture(cache_path: &Path) {
        let _ = make_orphan(cache_path, "orphan", b"orphan-bytes", 100);
        let _ = write_entry(cache_path, "old", b"old-bytes", 100);
    }

    #[test]
    fn cache_clean_dry_run_reports_orphan_plan_without_deleting() {
        let root = TempDirGuard::new("dry-run");
        let cache_path = root.path().join("http");
        let orphan = make_orphan(&cache_path, "orphan", b"orphan-bytes", 100);
        let config = test_config(
            root.path(),
            10_000_000_000,
            Duration::from_secs(86_400),
            ConfigOrigin::Default,
            ConfigOrigin::Default,
        );

        let report = execute_cache_clean(
            &cache_path,
            CleanOptions {
                max_age: None,
                max_size: None,
                dry_run: true,
            },
            &config,
            1_000,
        )
        .expect("dry run should succeed");

        assert_eq!(
            report,
            CleanReport {
                dry_run: true,
                orphans_removed: 1,
                entries_removed: 0,
                bytes_freed: 12,
                provider_capture_bytes_freed: 0,
                errors: Vec::new(),
            }
        );
        assert!(
            snapshot_cache(&cache_path)
                .expect("snapshot")
                .blobs
                .iter()
                .any(|blob| blob.integrity == orphan)
        );
    }

    #[test]
    fn cache_clean_destructive_removes_orphan_blobs() {
        let root = TempDirGuard::new("orphan");
        let cache_path = root.path().join("http");
        let orphan = make_orphan(&cache_path, "orphan", b"orphan-bytes", 100);
        let config = test_config(
            root.path(),
            10_000_000_000,
            Duration::from_secs(86_400),
            ConfigOrigin::Default,
            ConfigOrigin::Default,
        );

        let report = execute_cache_clean(
            &cache_path,
            CleanOptions {
                max_age: None,
                max_size: None,
                dry_run: false,
            },
            &config,
            1_000,
        )
        .expect("clean should succeed");

        assert_eq!(report.errors, Vec::<String>::new());
        assert_eq!(report.orphans_removed, 1);
        assert_eq!(report.entries_removed, 0);
        assert_eq!(report.bytes_freed, 12);
        assert!(!blob_path_for_integrity(&cache_path, &orphan).exists());
    }

    #[test]
    fn cache_clean_age_cleanup_removes_only_old_entries() {
        let root = TempDirGuard::new("age");
        let cache_path = root.path().join("http");
        let old = write_entry(&cache_path, "old", b"old", 100);
        let _ = write_entry(&cache_path, "new", b"new", 900);
        let config = test_config(
            root.path(),
            10_000_000_000,
            Duration::from_secs(86_400),
            ConfigOrigin::Default,
            ConfigOrigin::Default,
        );

        let report = execute_cache_clean(
            &cache_path,
            CleanOptions {
                max_age: Some(Duration::from_millis(500)),
                max_size: None,
                dry_run: false,
            },
            &config,
            1_000,
        )
        .expect("age clean should succeed");

        assert_eq!(report.entries_removed, 1);
        assert_eq!(report.bytes_freed, 3);
        assert_eq!(snapshot_keys(&cache_path), vec!["new"]);
        assert!(!blob_path_for_integrity(&cache_path, &old).exists());
    }

    #[test]
    fn cache_clean_size_cleanup_uses_explicit_config_without_flag() {
        let root = TempDirGuard::new("size-origin");
        let cache_path = root.path().join("http");
        let old = write_entry(&cache_path, "old", b"old1", 100);
        let _ = write_entry(&cache_path, "new", b"new2", 200);
        let config = test_config(
            root.path(),
            4,
            Duration::from_secs(86_400),
            ConfigOrigin::File,
            ConfigOrigin::Default,
        );

        let report = execute_cache_clean(
            &cache_path,
            CleanOptions {
                max_age: None,
                max_size: None,
                dry_run: false,
            },
            &config,
            1_000,
        )
        .expect("size clean should succeed");

        assert_eq!(report.entries_removed, 1);
        assert_eq!(report.bytes_freed, 4);
        assert_eq!(snapshot_keys(&cache_path), vec!["new"]);
        assert!(!blob_path_for_integrity(&cache_path, &old).exists());
    }

    #[test]
    fn cache_clean_default_origin_enforces_retention_without_size_eviction() {
        let root = TempDirGuard::new("default-origin");
        let cache_path = root.path().join("http");
        let _ = write_entry(&cache_path, "old", b"old", 100);
        let config = test_config(
            root.path(),
            0,
            Duration::ZERO,
            ConfigOrigin::Default,
            ConfigOrigin::Default,
        );

        let report = execute_cache_clean(
            &cache_path,
            CleanOptions {
                max_age: None,
                max_size: None,
                dry_run: false,
            },
            &config,
            1_000,
        )
        .expect("default-origin clean should succeed");

        assert_eq!(report.entries_removed, 1);
        assert_eq!(report.bytes_freed, 3);
        assert!(snapshot_keys(&cache_path).is_empty());
    }

    #[test]
    fn cache_clean_shared_integrity_blob_waits_for_all_keys() {
        let root = TempDirGuard::new("shared-integrity");
        let cache_path = root.path().join("http");
        let integrity = write_entry(&cache_path, "a", b"shared-bytes", 100);
        let _ = write_entry(&cache_path, "b", b"shared-bytes", 101);
        let config = test_config(
            root.path(),
            10_000_000_000,
            Duration::from_secs(86_400),
            ConfigOrigin::Default,
            ConfigOrigin::Default,
        );
        let blob_calls = Cell::new(0);

        let report = execute_cache_clean_with(
            &cache_path,
            CleanOptions {
                max_age: Some(Duration::from_millis(500)),
                max_size: None,
                dry_run: false,
            },
            &config,
            1_000,
            snapshot_cache,
            |cache, key| {
                if key == "b" {
                    return Err(cacache::Error::IoError(
                        io::Error::other("boom"),
                        "forced key failure".into(),
                    ));
                }
                cacache::remove_sync(cache, key)
            },
            |cache, sri| {
                blob_calls.set(blob_calls.get() + 1);
                cacache::remove_hash_sync(cache, sri)
            },
        )
        .expect("shared-integrity clean should return report");

        assert_eq!(report.entries_removed, 1);
        assert_eq!(report.bytes_freed, 0);
        assert_eq!(blob_calls.get(), 0);
        assert_eq!(snapshot_keys(&cache_path), vec!["b"]);
        assert!(blob_path_for_integrity(&cache_path, &integrity).exists());
        assert!(
            report
                .errors
                .iter()
                .any(|err| err.contains("failed to remove cache key 'b'"))
        );
        assert!(
            report
                .errors
                .iter()
                .any(|err| err.contains("not all planned key removals succeeded"))
        );
    }

    #[test]
    fn cache_clean_blob_not_found_is_benign_drift() {
        let root = TempDirGuard::new("blob-not-found");
        let cache_path = root.path().join("http");
        let _ = write_entry(&cache_path, "old", b"old", 100);
        let config = test_config(
            root.path(),
            10_000_000_000,
            Duration::from_secs(86_400),
            ConfigOrigin::Default,
            ConfigOrigin::Default,
        );

        let report = execute_cache_clean_with(
            &cache_path,
            CleanOptions {
                max_age: Some(Duration::from_millis(500)),
                max_size: None,
                dry_run: false,
            },
            &config,
            1_000,
            snapshot_cache,
            |path, key| cacache::remove_sync(path, key),
            |_cache, _sri| {
                Err(cacache::Error::IoError(
                    io::Error::new(io::ErrorKind::NotFound, "gone"),
                    "gone".into(),
                ))
            },
        )
        .expect("not-found clean should return report");

        assert_eq!(report.entries_removed, 1);
        assert_eq!(report.bytes_freed, 0);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn cache_clean_dry_run_matches_destructive_on_equivalent_seed() {
        let dry_root = TempDirGuard::new("parity-dry");
        let destructive_root = TempDirGuard::new("parity-destructive");
        let dry_cache = dry_root.path().join("http");
        let destructive_cache = destructive_root.path().join("http");
        seed_parity_fixture(&dry_cache);
        seed_parity_fixture(&destructive_cache);
        let dry_config = test_config(
            dry_root.path(),
            10_000_000_000,
            Duration::from_secs(86_400),
            ConfigOrigin::Default,
            ConfigOrigin::Default,
        );
        let destructive_config = test_config(
            destructive_root.path(),
            10_000_000_000,
            Duration::from_secs(86_400),
            ConfigOrigin::Default,
            ConfigOrigin::Default,
        );

        let dry_report = execute_cache_clean(
            &dry_cache,
            CleanOptions {
                max_age: Some(Duration::from_millis(500)),
                max_size: None,
                dry_run: true,
            },
            &dry_config,
            1_000,
        )
        .expect("dry run should succeed");
        let destructive_report = execute_cache_clean(
            &destructive_cache,
            CleanOptions {
                max_age: Some(Duration::from_millis(500)),
                max_size: None,
                dry_run: false,
            },
            &destructive_config,
            1_000,
        )
        .expect("destructive run should succeed");

        assert_eq!(
            dry_report.orphans_removed,
            destructive_report.orphans_removed
        );
        assert_eq!(
            dry_report.entries_removed,
            destructive_report.entries_removed
        );
        assert_eq!(dry_report.bytes_freed, destructive_report.bytes_freed);
        assert!(dry_report.errors.is_empty());
        assert!(destructive_report.errors.is_empty());
    }

    #[test]
    fn cache_clean_snapshot_failure_returns_io_error() {
        let root = TempDirGuard::new("snapshot-failure");
        let cache_path = root.path().join("http");
        let config = test_config(
            root.path(),
            10_000_000_000,
            Duration::from_secs(86_400),
            ConfigOrigin::Default,
            ConfigOrigin::Default,
        );

        let err = execute_cache_clean_with(
            &cache_path,
            CleanOptions {
                max_age: None,
                max_size: None,
                dry_run: false,
            },
            &config,
            1_000,
            |_| {
                Err(CachePlannerError::Io {
                    path: cache_path.clone(),
                    source: io::Error::other("snapshot failed"),
                })
            },
            |path, key| cacache::remove_sync(path, key),
            |path, integrity| cacache::remove_hash_sync(path, integrity),
        )
        .expect_err("snapshot failure should surface");

        assert!(matches!(err, BioMcpError::Io(_)));
    }
}
