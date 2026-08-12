use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use http_cache::{CacheManager, HttpResponse};
use http_cache_reqwest::CACacheManager;
use http_cache_semantics::CachePolicy;
use tracing::{debug, warn};

use super::{
    CleanOptions, FilesystemSpace, ResolvedCacheConfig, evaluate_cache_limits, execute_cache_clean,
    inspect_filesystem_space, snapshot_cache, summarize_cache_usage,
};
use crate::error::BioMcpError;

type EstimateCacheBytesFn = dyn Fn(&Path) -> io::Result<u64> + Send + Sync;
type InspectSpaceFn = dyn Fn(&Path) -> Result<FilesystemSpace, BioMcpError> + Send + Sync;
type ScheduleEvictionFn =
    dyn Fn(PathBuf, ResolvedCacheConfig, Arc<AtomicU64>, Arc<AtomicBool>) + Send + Sync;

#[derive(Clone)]
struct ManagerServices {
    estimate_cache_bytes: Arc<EstimateCacheBytesFn>,
    inspect_space: Arc<InspectSpaceFn>,
    schedule_eviction: Arc<ScheduleEvictionFn>,
}

pub(crate) struct SizeAwareCacheManager {
    inner: CACacheManager,
    config: ResolvedCacheConfig,
    approx_bytes: Arc<AtomicU64>,
    eviction_running: Arc<AtomicBool>,
    services: ManagerServices,
}

impl SizeAwareCacheManager {
    pub(crate) fn new(path: PathBuf, config: ResolvedCacheConfig) -> Result<Self, BioMcpError> {
        Self::new_at(path, config, current_time_ms())
    }

    fn new_at(
        path: PathBuf,
        config: ResolvedCacheConfig,
        now_ms: u128,
    ) -> Result<Self, BioMcpError> {
        execute_cache_clean(
            &path,
            CleanOptions {
                max_age: None,
                max_size: None,
                dry_run: false,
            },
            &config,
            now_ms,
        )?;
        Ok(Self::build_with_services(path, config, default_services()))
    }

    #[cfg(test)]
    fn new_with_services<E, I, S>(
        path: PathBuf,
        config: ResolvedCacheConfig,
        estimate_cache_bytes: E,
        inspect_space: I,
        schedule_eviction: S,
    ) -> Self
    where
        E: Fn(&Path) -> io::Result<u64> + Send + Sync + 'static,
        I: Fn(&Path) -> Result<FilesystemSpace, BioMcpError> + Send + Sync + 'static,
        S: Fn(PathBuf, ResolvedCacheConfig, Arc<AtomicU64>, Arc<AtomicBool>)
            + Send
            + Sync
            + 'static,
    {
        Self::build_with_services(
            path,
            config,
            ManagerServices {
                estimate_cache_bytes: Arc::new(estimate_cache_bytes),
                inspect_space: Arc::new(inspect_space),
                schedule_eviction: Arc::new(schedule_eviction),
            },
        )
    }

    fn build_with_services(
        path: PathBuf,
        config: ResolvedCacheConfig,
        services: ManagerServices,
    ) -> Self {
        let approx_bytes = match (services.estimate_cache_bytes)(&path) {
            Ok(bytes) => bytes,
            Err(err) => {
                warn!(
                    cache_path = %path.display(),
                    "fast cache size estimate failed; seeding size tracker to 0: {err}"
                );
                0
            }
        };

        Self {
            inner: CACacheManager { path },
            config,
            approx_bytes: Arc::new(AtomicU64::new(approx_bytes)),
            eviction_running: Arc::new(AtomicBool::new(false)),
            services,
        }
    }
}

#[async_trait]
impl CacheManager for SizeAwareCacheManager {
    async fn get(
        &self,
        cache_key: &str,
    ) -> http_cache::Result<Option<(HttpResponse, CachePolicy)>> {
        super::secure_managed_tree(&self.inner.path, false)?;
        self.inner.get(cache_key).await
    }

    async fn put(
        &self,
        cache_key: String,
        res: HttpResponse,
        policy: CachePolicy,
    ) -> http_cache::Result<HttpResponse> {
        super::secure_managed_tree(&self.inner.path, false)?;
        let response = self.inner.put(cache_key.clone(), res, policy).await?;
        super::secure_managed_tree(&self.inner.path, true)?;

        match cacache::metadata(&self.inner.path, &cache_key).await {
            Ok(Some(metadata)) => {
                self.approx_bytes
                    .fetch_add(metadata.size as u64, Ordering::Relaxed);
            }
            Ok(None) => warn!(
                cache_key,
                cache_path = %self.inner.path.display(),
                "cache metadata missing after put; leaving approximate size unchanged"
            ),
            Err(err) => warn!(
                cache_key,
                cache_path = %self.inner.path.display(),
                "cache metadata lookup failed after put; leaving approximate size unchanged: {err}"
            ),
        }

        let approx_bytes = self.approx_bytes.load(Ordering::Relaxed);
        let below_min_disk_free = match (self.services.inspect_space)(&self.config.cache_root) {
            Ok(space) => self
                .config
                .min_disk_free
                .is_violated(space.available_bytes, space.total_bytes),
            Err(err) => {
                warn!(
                    cache_root = %self.config.cache_root.display(),
                    "cache filesystem inspection failed after put; skipping disk-pressure trigger: {err}"
                );
                false
            }
        };

        if (approx_bytes > self.config.max_size || below_min_disk_free)
            && self
                .eviction_running
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
        {
            (self.services.schedule_eviction)(
                self.inner.path.clone(),
                self.config.clone(),
                Arc::clone(&self.approx_bytes),
                Arc::clone(&self.eviction_running),
            );
        }

        Ok(response)
    }

    async fn delete(&self, cache_key: &str) -> http_cache::Result<()> {
        super::secure_managed_tree(&self.inner.path, false)?;
        self.inner.delete(cache_key).await
    }
}

fn default_services() -> ManagerServices {
    ManagerServices {
        estimate_cache_bytes: Arc::new(estimate_cache_bytes_fast),
        inspect_space: Arc::new(inspect_filesystem_space),
        schedule_eviction: Arc::new(spawn_eviction_task),
    }
}

fn estimate_cache_bytes_fast(path: &Path) -> io::Result<u64> {
    let content_root = path.join("content-v2");
    match fs::symlink_metadata(&content_root) {
        Ok(metadata) if metadata.is_dir() => sum_tree_bytes(&content_root),
        Ok(metadata) if metadata.file_type().is_symlink() => Ok(0),
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{} is not a directory", content_root.display()),
        )),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(0),
        Err(err) => Err(err),
    }
}

fn sum_tree_bytes(path: &Path) -> io::Result<u64> {
    let mut total = 0_u64;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        let metadata = fs::symlink_metadata(&entry_path)?;
        let file_type = metadata.file_type();
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            total = total.saturating_add(sum_tree_bytes(&entry_path)?);
        } else if file_type.is_file() {
            total = total.saturating_add(metadata.len());
        }
    }
    Ok(total)
}

fn current_time_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn spawn_eviction_task(
    cache_path: PathBuf,
    config: ResolvedCacheConfig,
    approx_bytes: Arc<AtomicU64>,
    eviction_running: Arc<AtomicBool>,
) {
    tokio::spawn(async move {
        struct ResetFlag(Arc<AtomicBool>);

        impl Drop for ResetFlag {
            fn drop(&mut self) {
                self.0.store(false, Ordering::Release);
            }
        }

        let _reset_flag = ResetFlag(Arc::clone(&eviction_running));
        let warn_path = cache_path.clone();
        let result = tokio::task::spawn_blocking(move || {
            run_eviction_cycle(&cache_path, &config, approx_bytes.as_ref())
        })
        .await;

        match result {
            Ok(Ok(())) => {}
            Ok(Err(err)) => warn!(
                cache_path = %warn_path.display(),
                "cache eviction cycle failed: {err}"
            ),
            Err(err) => warn!(
                cache_path = %warn_path.display(),
                "cache eviction task join failed: {err}"
            ),
        }
    });
}

fn run_eviction_cycle(
    cache_path: &Path,
    config: &ResolvedCacheConfig,
    approx_bytes: &AtomicU64,
) -> Result<(), BioMcpError> {
    run_eviction_cycle_with(
        cache_path,
        config,
        approx_bytes,
        snapshot_cache,
        inspect_filesystem_space,
        execute_cache_clean,
        current_time_ms,
    )
}

fn run_eviction_cycle_with<S, I, C, N>(
    cache_path: &Path,
    config: &ResolvedCacheConfig,
    approx_bytes: &AtomicU64,
    mut snapshotter: S,
    mut inspect_space: I,
    mut cleaner: C,
    now_ms: N,
) -> Result<(), BioMcpError>
where
    S: FnMut(&Path) -> Result<super::CacheSnapshot, super::CachePlannerError>,
    I: FnMut(&Path) -> Result<FilesystemSpace, BioMcpError>,
    C: FnMut(
        &Path,
        CleanOptions,
        &ResolvedCacheConfig,
        u128,
    ) -> Result<super::CleanReport, BioMcpError>,
    N: FnOnce() -> u128,
{
    let snapshot_before =
        snapshotter(cache_path).map_err(|err| BioMcpError::Io(io::Error::other(err)))?;
    let space_before = inspect_space(&config.cache_root)?;
    let evaluation = evaluate_cache_limits(&snapshot_before, config, space_before);

    if !evaluation.over_max_size && !evaluation.below_min_disk_free {
        approx_bytes.store(evaluation.usage.referenced_blob_bytes, Ordering::Relaxed);
        return Ok(());
    }

    let report = cleaner(
        cache_path,
        CleanOptions {
            max_age: Some(config.max_age),
            max_size: Some(evaluation.effective_max_size),
            dry_run: false,
        },
        config,
        now_ms(),
    )?;

    if !report.errors.is_empty() {
        warn!(
            cache_path = %cache_path.display(),
            errors = report.errors.len(),
            "cache eviction completed with cleanup errors"
        );
    }

    let snapshot_after =
        snapshotter(cache_path).map_err(|err| BioMcpError::Io(io::Error::other(err)))?;
    let usage_after = summarize_cache_usage(&snapshot_after);
    approx_bytes.store(usage_after.referenced_blob_bytes, Ordering::Relaxed);

    let space_after = inspect_space(&config.cache_root)?;
    if config
        .min_disk_free
        .is_violated(space_after.available_bytes, space_after.total_bytes)
    {
        debug!(
            cache_path = %cache_path.display(),
            min_disk_free = %config.min_disk_free.display(),
            available_bytes = space_after.available_bytes,
            total_bytes = space_after.total_bytes,
            "cache cleanup completed but free disk space is still below the configured floor"
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
    use std::time::Duration;

    use http::Request;
    use http::Response;
    use http_cache::{HttpResponse, HttpVersion};
    use http_cache_semantics::CachePolicy;
    use tracing::Event;
    use tracing::field::{Field, Visit};
    use tracing_subscriber::layer::{Context, SubscriberExt};
    use tracing_subscriber::{Layer, Registry};

    use super::{
        FilesystemSpace, SizeAwareCacheManager, estimate_cache_bytes_fast, run_eviction_cycle_with,
    };
    use crate::cache::{
        CacheConfigOrigins, ConfigOrigin, DiskFreeThreshold, ResolvedCacheConfig, snapshot_cache,
    };
    use crate::test_support::TempDirGuard;
    use http_cache::CacheManager;

    fn write_entry(cache_path: &Path, key: &str, bytes: &[u8], time_ms: u128) {
        let mut writer = cacache::WriteOpts::new()
            .size(bytes.len())
            .time(time_ms)
            .open_sync(cache_path, key)
            .expect("open writer");
        std::io::Write::write_all(&mut writer, bytes).expect("write bytes");
        writer.commit().expect("commit writer");
    }

    fn make_orphan(cache_path: &Path, key: &str, bytes: &[u8], time_ms: u128) {
        write_entry(cache_path, key, bytes, time_ms);
        cacache::remove_sync(cache_path, key).expect("remove orphaned key");
    }

    fn test_config(
        cache_root: &Path,
        max_size: u64,
        min_disk_free: DiskFreeThreshold,
    ) -> ResolvedCacheConfig {
        ResolvedCacheConfig {
            cache_root: cache_root.to_path_buf(),
            max_size,
            min_disk_free,
            max_age: Duration::from_secs(86_400),
            origins: CacheConfigOrigins {
                cache_root: ConfigOrigin::Default,
                max_size: ConfigOrigin::Default,
                min_disk_free: ConfigOrigin::Default,
                max_age: ConfigOrigin::Default,
            },
        }
    }

    fn test_policy() -> CachePolicy {
        let request = Request::builder()
            .method("GET")
            .uri("https://example.test/cache-key")
            .body(())
            .expect("request");
        let response = Response::builder()
            .status(200)
            .header("cache-control", "max-age=60")
            .body(())
            .expect("response");
        CachePolicy::new(&request, &response)
    }

    fn test_http_response(body: &[u8]) -> HttpResponse {
        HttpResponse {
            body: body.to_vec(),
            headers: HashMap::from([("cache-control".to_string(), "max-age=60".to_string())]),
            status: 200,
            url: reqwest::Url::parse("https://example.test/cache-key").expect("url"),
            version: HttpVersion::Http11,
        }
    }

    #[derive(Clone, Debug)]
    struct CapturedEvent {
        level: tracing::Level,
        message: String,
    }

    #[derive(Clone, Default)]
    struct CapturingLayer {
        events: Arc<Mutex<Vec<CapturedEvent>>>,
    }

    impl<S> Layer<S> for CapturingLayer
    where
        S: tracing::Subscriber,
    {
        fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
            let mut visitor = MessageVisitor::default();
            event.record(&mut visitor);
            self.events
                .lock()
                .expect("capture events")
                .push(CapturedEvent {
                    level: *event.metadata().level(),
                    message: visitor.message.unwrap_or_default(),
                });
        }
    }

    #[derive(Default)]
    struct MessageVisitor {
        message: Option<String>,
    }

    impl Visit for MessageVisitor {
        fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
            if field.name() == "message" {
                self.message = Some(format!("{value:?}"));
            }
        }
    }

    #[test]
    fn estimate_cache_bytes_fast_returns_zero_for_missing_tree() {
        let root = TempDirGuard::new("estimate-empty");
        assert_eq!(
            estimate_cache_bytes_fast(&root.path().join("http")).expect("estimate"),
            0
        );
    }

    #[test]
    fn estimate_cache_bytes_fast_sums_content_tree_file_sizes() {
        let root = TempDirGuard::new("estimate-sum");
        let content_root = root
            .path()
            .join("http")
            .join("content-v2")
            .join("sha256")
            .join("aa")
            .join("bb");
        fs::create_dir_all(&content_root).expect("content tree");
        fs::write(content_root.join("blob-a"), b"abc").expect("blob a");
        fs::write(content_root.join("blob-b"), b"defgh").expect("blob b");

        assert_eq!(
            estimate_cache_bytes_fast(&root.path().join("http")).expect("estimate"),
            8
        );
    }

    #[test]
    fn run_eviction_cycle_false_positive_resyncs_without_cleaning() {
        let root = TempDirGuard::new("false-positive");
        let cache_path = root.path().join("http");
        write_entry(&cache_path, "retained", b"live-bytes", 100);
        let snapshot = snapshot_cache(&cache_path).expect("snapshot");
        let config = test_config(root.path(), 100, DiskFreeThreshold::Percent(10));
        let approx_bytes = AtomicU64::new(999);
        let cleaner_calls = AtomicUsize::new(0);

        run_eviction_cycle_with(
            &cache_path,
            &config,
            &approx_bytes,
            |_| Ok(snapshot.clone()),
            |_| {
                Ok(FilesystemSpace {
                    available_bytes: 90,
                    total_bytes: 100,
                })
            },
            |_, _, _, _| {
                cleaner_calls.fetch_add(1, Ordering::SeqCst);
                unreachable!("cleaner should not run when exact snapshot is within limits");
            },
            || 1_000,
        )
        .expect("cycle");

        assert_eq!(cleaner_calls.load(Ordering::SeqCst), 0);
        assert_eq!(
            approx_bytes.load(Ordering::Relaxed),
            b"live-bytes".len() as u64
        );
    }

    #[test]
    fn run_eviction_cycle_uses_exact_snapshot_state_for_size_eviction() {
        let before_root = TempDirGuard::new("oversize-before");
        let after_root = TempDirGuard::new("oversize-after");
        let before_cache = before_root.path().join("http");
        let after_cache = after_root.path().join("http");
        write_entry(&before_cache, "old", b"abcde", 100);
        write_entry(&before_cache, "new", b"fghij", 200);
        write_entry(&after_cache, "new", b"fghij", 200);
        let before_snapshot = snapshot_cache(&before_cache).expect("before snapshot");
        let after_snapshot = snapshot_cache(&after_cache).expect("after snapshot");
        let config = test_config(before_root.path(), 5, DiskFreeThreshold::Percent(10));
        let approx_bytes = AtomicU64::new(0);
        let snapshot_calls = AtomicUsize::new(0);
        let seen_max_size = Arc::new(AtomicU64::new(u64::MAX));

        run_eviction_cycle_with(
            &before_cache,
            &config,
            &approx_bytes,
            |_| {
                let call = snapshot_calls.fetch_add(1, Ordering::SeqCst);
                Ok(if call == 0 {
                    before_snapshot.clone()
                } else {
                    after_snapshot.clone()
                })
            },
            |_| {
                Ok(FilesystemSpace {
                    available_bytes: 90,
                    total_bytes: 100,
                })
            },
            {
                let seen_max_size = Arc::clone(&seen_max_size);
                move |_, options, _, _| {
                    seen_max_size.store(options.max_size.expect("max_size"), Ordering::SeqCst);
                    Ok(crate::cache::CleanReport {
                        dry_run: false,
                        orphans_removed: 0,
                        entries_removed: 1,
                        bytes_freed: 5,
                        provider_capture_bytes_freed: 0,
                        errors: Vec::new(),
                    })
                }
            },
            || 1_000,
        )
        .expect("cycle");

        assert_eq!(seen_max_size.load(Ordering::SeqCst), 5);
        assert_eq!(approx_bytes.load(Ordering::Relaxed), 5);
    }

    #[test]
    fn run_eviction_cycle_uses_effective_max_size_for_disk_pressure() {
        let before_root = TempDirGuard::new("disk-pressure-before");
        let after_root = TempDirGuard::new("disk-pressure-after");
        let before_cache = before_root.path().join("http");
        let after_cache = after_root.path().join("http");
        write_entry(&before_cache, "retained", b"live-bytes", 100);
        make_orphan(&before_cache, "orphan", b"orph", 101);
        write_entry(&after_cache, "retained", b"live-byt", 100);
        let before_snapshot = snapshot_cache(&before_cache).expect("before snapshot");
        let after_snapshot = snapshot_cache(&after_cache).expect("after snapshot");
        let config = test_config(before_root.path(), 100, DiskFreeThreshold::Percent(20));
        let approx_bytes = AtomicU64::new(0);
        let snapshot_calls = AtomicUsize::new(0);
        let inspect_calls = AtomicUsize::new(0);
        let seen_max_size = Arc::new(AtomicU64::new(u64::MAX));

        run_eviction_cycle_with(
            &before_cache,
            &config,
            &approx_bytes,
            |_| {
                let call = snapshot_calls.fetch_add(1, Ordering::SeqCst);
                Ok(if call == 0 {
                    before_snapshot.clone()
                } else {
                    after_snapshot.clone()
                })
            },
            |_| {
                let call = inspect_calls.fetch_add(1, Ordering::SeqCst);
                Ok(if call == 0 {
                    FilesystemSpace {
                        available_bytes: 14,
                        total_bytes: 100,
                    }
                } else {
                    FilesystemSpace {
                        available_bytes: 20,
                        total_bytes: 100,
                    }
                })
            },
            {
                let seen_max_size = Arc::clone(&seen_max_size);
                move |_, options, _, _| {
                    seen_max_size.store(options.max_size.expect("max_size"), Ordering::SeqCst);
                    Ok(crate::cache::CleanReport {
                        dry_run: false,
                        orphans_removed: 1,
                        entries_removed: 0,
                        bytes_freed: 6,
                        provider_capture_bytes_freed: 0,
                        errors: Vec::new(),
                    })
                }
            },
            || 1_000,
        )
        .expect("cycle");

        assert_eq!(seen_max_size.load(Ordering::SeqCst), 8);
        assert_eq!(approx_bytes.load(Ordering::Relaxed), 8);
    }

    fn captured_eviction_events(
        after_available_bytes: u64,
        errors: Vec<String>,
    ) -> Vec<CapturedEvent> {
        let before_root = TempDirGuard::new("log-capture-before");
        let after_root = TempDirGuard::new("log-capture-after");
        let before_cache = before_root.path().join("http");
        let after_cache = after_root.path().join("http");
        write_entry(&before_cache, "retained", b"live-bytes", 100);
        write_entry(&after_cache, "retained", b"live-bytes", 100);
        let before_snapshot = snapshot_cache(&before_cache).expect("before snapshot");
        let after_snapshot = snapshot_cache(&after_cache).expect("after snapshot");
        let config = test_config(before_root.path(), 100, DiskFreeThreshold::Percent(20));
        let approx_bytes = AtomicU64::new(0);
        let snapshot_calls = AtomicUsize::new(0);
        let inspect_calls = AtomicUsize::new(0);
        let capture = CapturingLayer::default();
        let events = Arc::clone(&capture.events);
        let subscriber = Registry::default().with(capture);

        tracing::subscriber::with_default(subscriber, || {
            run_eviction_cycle_with(
                &before_cache,
                &config,
                &approx_bytes,
                |_| {
                    let call = snapshot_calls.fetch_add(1, Ordering::SeqCst);
                    Ok(if call == 0 {
                        before_snapshot.clone()
                    } else {
                        after_snapshot.clone()
                    })
                },
                |_| {
                    let call = inspect_calls.fetch_add(1, Ordering::SeqCst);
                    Ok(FilesystemSpace {
                        available_bytes: if call == 0 { 10 } else { after_available_bytes },
                        total_bytes: 100,
                    })
                },
                |_, _, _, _| {
                    Ok(crate::cache::CleanReport {
                        dry_run: false,
                        orphans_removed: 0,
                        entries_removed: 0,
                        bytes_freed: 0,
                        provider_capture_bytes_freed: 0,
                        errors: errors.clone(),
                    })
                },
                || 1_000,
            )
            .expect("cycle");
        });

        events.lock().expect("captured events").clone()
    }

    #[test]
    fn run_eviction_cycle_logs_successful_cleanup_still_under_floor_at_debug() {
        let events = captured_eviction_events(15, Vec::new());
        let message =
            "cache cleanup completed but free disk space is still below the configured floor";
        assert!(
            events.iter().any(
                |event| event.level == tracing::Level::DEBUG && event.message.contains(message)
            ),
            "expected under-floor cleanup message at debug; captured events: {events:?}"
        );
        assert!(
            !events
                .iter()
                .any(|event| event.level == tracing::Level::WARN && event.message.contains(message)),
            "under-floor cleanup success must not warn; captured events: {events:?}"
        );
    }

    #[test]
    fn run_eviction_cycle_logs_cleanup_errors_at_warn() {
        let events = captured_eviction_events(25, vec!["simulated cleanup failure".to_string()]);
        let message = "cache eviction completed with cleanup errors";
        assert!(
            events
                .iter()
                .any(|event| event.level == tracing::Level::WARN && event.message.contains(message)),
            "expected cleanup errors to warn; captured events: {events:?}"
        );
    }

    #[test]
    fn run_eviction_cycle_propagates_snapshot_error() {
        let root = TempDirGuard::new("eviction-error");
        let cache_path = root.path().join("http");
        let config = test_config(root.path(), 100, DiskFreeThreshold::Percent(10));
        let approx_bytes = AtomicU64::new(999);

        let result = run_eviction_cycle_with(
            &cache_path,
            &config,
            &approx_bytes,
            |_| {
                Err(crate::cache::CachePlannerError::Io {
                    path: cache_path.clone(),
                    source: std::io::Error::other("simulated snapshot failure"),
                })
            },
            |_| {
                Ok(FilesystemSpace {
                    available_bytes: 90,
                    total_bytes: 100,
                })
            },
            |_, _, _, _| unreachable!("cleaner should not run when snapshot fails"),
            || 1_000,
        );

        assert!(result.is_err(), "expected error to propagate from snapshot");
        // approx_bytes should be unchanged since the cycle failed before resync
        assert_eq!(approx_bytes.load(Ordering::Relaxed), 999);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn new_manager_seeds_approximate_bytes_from_fast_estimate() {
        let root = TempDirGuard::new("seed-estimate");
        let content_root = root
            .path()
            .join("http")
            .join("content-v2")
            .join("sha256")
            .join("aa")
            .join("bb");
        fs::create_dir_all(&content_root).expect("content tree");
        fs::write(content_root.join("blob-a"), b"abc").expect("blob a");
        fs::write(content_root.join("blob-b"), b"defgh").expect("blob b");
        let manager = SizeAwareCacheManager::new(
            root.path().join("http"),
            test_config(root.path(), 100, DiskFreeThreshold::Percent(10)),
        )
        .expect("new cache manager");

        assert_eq!(manager.approx_bytes.load(Ordering::Relaxed), 8);
    }

    #[test]
    fn opening_manager_physically_removes_expired_entries_below_size_limit() {
        let root = TempDirGuard::new("open-age-maintenance");
        let cache_path = root.path().join("http");
        write_entry(&cache_path, "expired", b"old", 1_000);
        write_entry(&cache_path, "retained", b"new", 15_000);
        let mut config = test_config(root.path(), 1_000_000, DiskFreeThreshold::Percent(1));
        config.max_age = Duration::from_secs(10);

        SizeAwareCacheManager::new_at(cache_path.clone(), config, 20_000)
            .expect("open maintained manager");

        assert!(
            cacache::metadata_sync(&cache_path, "expired")
                .expect("expired metadata")
                .is_none()
        );
        assert!(
            cacache::metadata_sync(&cache_path, "retained")
                .expect("retained metadata")
                .is_some()
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn put_schedules_eviction_for_preexisting_oversized_cache_with_ample_disk() {
        let root = TempDirGuard::new("schedule-oversized");
        let scheduled = Arc::new(AtomicUsize::new(0));
        let manager = SizeAwareCacheManager::new_with_services(
            root.path().join("http"),
            test_config(root.path(), 1, DiskFreeThreshold::Percent(10)),
            |_| Ok(2),
            |_| {
                Ok(FilesystemSpace {
                    available_bytes: 90,
                    total_bytes: 100,
                })
            },
            {
                let scheduled = Arc::clone(&scheduled);
                move |_, _, _, _| {
                    scheduled.fetch_add(1, Ordering::SeqCst);
                }
            },
        );

        manager
            .put("oversized".into(), test_http_response(b"x"), test_policy())
            .await
            .expect("put");

        assert_eq!(scheduled.load(Ordering::SeqCst), 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn put_schedules_eviction_when_disk_floor_is_violated() {
        let root = TempDirGuard::new("schedule-disk-floor");
        let scheduled = Arc::new(AtomicUsize::new(0));
        let manager = SizeAwareCacheManager::new_with_services(
            root.path().join("http"),
            test_config(root.path(), u64::MAX / 2, DiskFreeThreshold::Percent(20)),
            |_| Ok(0),
            |_| {
                Ok(FilesystemSpace {
                    available_bytes: 10,
                    total_bytes: 100,
                })
            },
            {
                let scheduled = Arc::clone(&scheduled);
                move |_, _, _, _| {
                    scheduled.fetch_add(1, Ordering::SeqCst);
                }
            },
        );

        manager
            .put("disk-floor".into(), test_http_response(b"x"), test_policy())
            .await
            .expect("put");

        assert_eq!(scheduled.load(Ordering::SeqCst), 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn put_debounces_duplicate_eviction_scheduling() {
        let root = TempDirGuard::new("schedule-debounce");
        let scheduled = Arc::new(AtomicUsize::new(0));
        let manager = SizeAwareCacheManager::new_with_services(
            root.path().join("http"),
            test_config(root.path(), 1, DiskFreeThreshold::Percent(10)),
            |_| Ok(2),
            |_| {
                Ok(FilesystemSpace {
                    available_bytes: 90,
                    total_bytes: 100,
                })
            },
            {
                let scheduled = Arc::clone(&scheduled);
                move |_, _, _, _| {
                    scheduled.fetch_add(1, Ordering::SeqCst);
                }
            },
        );

        manager
            .put("first".into(), test_http_response(b"x"), test_policy())
            .await
            .expect("first put");
        manager
            .put("second".into(), test_http_response(b"y"), test_policy())
            .await
            .expect("second put");

        assert_eq!(scheduled.load(Ordering::SeqCst), 1);
    }
}
