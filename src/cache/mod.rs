mod citation_evidence;
mod clean;
mod config;
pub(crate) mod http;
mod limits;
#[path = "clear.rs"]
mod maintenance;
pub(crate) mod manager;
mod private;
mod provider_capture;
pub(crate) mod sqlite;

pub(crate) use citation_evidence::{read_citation_evidence, write_citation_evidence};
pub(crate) use clean::CleanReport;
#[cfg(test)]
pub(crate) use config::{CacheConfigOrigins, DiskFreeThreshold};
pub(crate) use config::{ConfigOrigin, ResolvedCacheConfig, resolve_cache_config};
pub(crate) use http::{HttpCacheManager, warn_local_http_cache};
pub(crate) use limits::{
    CacheLimitEvaluation, CacheUsage, FilesystemSpace, evaluate_cache_usage,
    inspect_filesystem_space,
};
pub(crate) use maintenance::{ClearReport, execute_cache_clear};
pub(crate) use manager::SizeAwareCacheManager;
pub(crate) use private::{open_managed_read, open_private, secure_managed_tree};
pub(crate) use provider_capture::{
    CspecCaptureBinding, ProviderCaptureError, ProviderCaptureManifest, ProviderCaptureStore,
};

#[cfg(test)]
pub(crate) use provider_capture::ProviderCaptureProvider;
