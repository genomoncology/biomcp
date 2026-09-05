mod clean;
mod clear;
mod config;
mod layout;
mod limits;
mod manager;
pub(crate) mod migration;
mod planner;
mod private;
mod provider_capture;
mod write_security;

#[allow(unused_imports)]
pub(crate) use clean::{CleanOptions, CleanReport, execute_cache_clean};
#[allow(unused_imports)]
pub(crate) use clear::{ClearReport, execute_cache_clear};
#[allow(unused_imports)]
pub(crate) use config::{
    CacheConfig, CacheConfigOrigins, ConfigOrigin, DiskFreeThreshold, ResolvedCacheConfig,
    resolve_cache_config,
};
pub(crate) use layout::{content_path, content_root, index_bucket_path};
#[allow(unused_imports)]
pub(crate) use limits::{
    CacheLimitEvaluation, CacheUsage, FilesystemSpace, evaluate_cache_limits,
    inspect_filesystem_space, summarize_cache_usage,
};
pub(crate) use manager::SizeAwareCacheManager;
pub(crate) use migration::{MigrationOutcome, ensure_body_limited_cache_epoch, migrate_http_cache};
#[allow(unused_imports)]
pub(crate) use planner::{
    CacheBlob, CacheCleanupPlan, CacheEntry, CachePlannerError, CacheSnapshot, plan_age_cleanup,
    plan_composite_cleanup, plan_orphan_gc, plan_size_lru, snapshot_cache,
};
pub(crate) use private::{
    lock_cache_operation, lock_cache_operation_async, open_managed_read, open_private,
    prepare_managed_file, secure_managed_dir, secure_managed_file, secure_managed_tree,
};
#[allow(unused_imports)]
pub(crate) use provider_capture::{
    CspecCaptureBinding, ProviderCaptureError, ProviderCaptureManifest, ProviderCaptureProvider,
    ProviderCaptureStore,
};
pub(crate) use write_security::{prepare_write_paths, secure_written_content};
