mod clean;
mod clear;
mod config;
mod limits;
mod manager;
pub(crate) mod migration;
mod planner;
mod private;
mod provider_capture;

use std::path::{Path, PathBuf};

use sha1::{Digest, Sha1};
use ssri::Integrity;

pub(crate) const CONTENT_DIR: &str = "content-v2";
pub(crate) const INDEX_DIR: &str = "index-v5";
pub(crate) const KEY_LOCK_DIR: &str = ".biomcp-key-locks";
pub(crate) const TEMP_DIR: &str = "tmp";

pub(crate) fn index_bucket_path(cache_path: &Path, key: &str) -> PathBuf {
    let hex = key_digest(key);
    cache_path
        .join(INDEX_DIR)
        .join(&hex[..2])
        .join(&hex[2..4])
        .join(&hex[4..])
}

pub(crate) fn key_lock_path(cache_root: &Path, key: &str) -> PathBuf {
    cache_root
        .join(KEY_LOCK_DIR)
        .join(format!("{}.lock", &key_digest(key)[..2]))
}

fn key_digest(key: &str) -> String {
    Sha1::digest(key.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub(crate) fn content_root(cache_path: &Path) -> PathBuf {
    cache_path.join(CONTENT_DIR)
}

pub(crate) fn content_path(cache_path: &Path, integrity: &Integrity) -> PathBuf {
    let (algorithm, hex) = integrity.to_hex();
    content_root(cache_path)
        .join(algorithm.to_string())
        .join(&hex[..2])
        .join(&hex[2..4])
        .join(&hex[4..])
}

#[allow(unused_imports)]
pub(crate) use clean::{CleanOptions, CleanReport, execute_cache_clean};
#[allow(unused_imports)]
pub(crate) use clear::{ClearReport, execute_cache_clear};
#[allow(unused_imports)]
pub(crate) use config::{
    CacheConfig, CacheConfigOrigins, ConfigOrigin, DiskFreeThreshold, ResolvedCacheConfig,
    resolve_cache_config,
};
#[allow(unused_imports)]
pub(crate) use limits::{
    CacheLimitEvaluation, CacheUsage, FilesystemSpace, evaluate_cache_limits,
    inspect_filesystem_space, summarize_cache_usage,
};
pub(crate) use manager::SizeAwareCacheManager;

pub(crate) use migration::{
    MigrationOutcome, ensure_body_limited_cache_epoch, ensure_body_limited_cache_epoch_until,
    migrate_http_cache, migrate_http_cache_with_deadline,
};
#[allow(unused_imports)]
pub(crate) use planner::{
    CacheBlob, CacheCleanupPlan, CacheEntry, CachePlannerError, CacheSnapshot, plan_age_cleanup,
    plan_composite_cleanup, plan_orphan_gc, plan_size_lru, snapshot_cache,
};
pub(crate) use private::{
    lock_cache_key_async, lock_cache_maintenance, lock_cache_maintenance_until, lock_cache_shared,
    lock_cache_shared_until, open_managed_read, open_private, prepare_write_paths,
    secure_managed_tree, secure_managed_tree_until, secure_written_content,
    try_lock_cache_maintenance,
};
#[allow(unused_imports)]
pub(crate) use provider_capture::{
    CspecCaptureBinding, ProviderCaptureError, ProviderCaptureManifest, ProviderCaptureProvider,
    ProviderCaptureStore,
};

#[cfg(test)]
mod layout_tests {
    use std::fs;

    use super::index_bucket_path;
    use crate::test_support::TempDirGuard;

    #[test]
    fn derived_index_bucket_is_the_file_cacache_writes() {
        let root = TempDirGuard::new("cache-index-layout");
        for key in [
            "plain-ascii-key",
            "https://example.test/search?q=BRAF%20V600E&lang=é",
        ] {
            cacache::write_sync(root.path(), key, b"value").expect("write cache entry");
            let bucket = index_bucket_path(root.path(), key);
            assert!(
                bucket.is_file(),
                "derived bucket missing: {}",
                bucket.display()
            );
            assert!(
                fs::read_to_string(bucket)
                    .expect("read index bucket")
                    .contains(key),
                "derived bucket does not contain key"
            );
        }
    }
}
