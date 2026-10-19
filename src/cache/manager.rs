use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use async_trait::async_trait;
use http_cache::{CacheManager, HttpResponse};
use http_cache_semantics::CachePolicy;
use serde::{Deserialize, Serialize};

use super::{ResolvedCacheConfig, sqlite};
use crate::error::BioMcpError;

/// Marker header stamped on a cached response served past its freshness
/// window (see `SizeAwareCacheManager::get`). The header survives only on
/// direct stale serves: http-cache's conditional fetch serves `put`'s
/// return after a 304/200 revalidation, and `put` strips the marker, so a
/// freshly revalidated response can never carry it.
pub(crate) const STALE_SERVE_AGE_HEADER: &str = "x-biomcp-cache-stale-age";

/// Stamp the stale-serve marker from the stored policy when the entry is
/// past its effective freshness window. Day-cache requests use their day
/// window instead of the provider's policy. The age comes from the policy
/// clock, not the file's write time.
fn stamp_stale_serve_marker(response: &mut HttpResponse, policy: &CachePolicy) {
    let now = SystemTime::now();
    let age = policy.age(now);
    let is_stale = if crate::sources::day_cache_request_active() {
        age >= crate::sources::DAY_CACHE_TTL
    } else {
        policy.is_stale(now)
    };
    if !is_stale {
        return;
    }
    let age_seconds = age.as_secs();
    response
        .headers
        .insert(STALE_SERVE_AGE_HEADER.to_string(), age_seconds.to_string());
}

/// A day-cache lookup whose stored answer is at least a day old: the
/// entry serves nothing, so the library refetches and re-stores a fresh
/// answer. `BIOMCP_CACHE_MODE=infinite` keeps its meaning — serve
/// expired entries without revalidation — so the day window yields to
/// it.
fn day_cache_entry_expired(policy: &CachePolicy) -> bool {
    crate::sources::day_cache_request_active()
        && !crate::sources::cache_is_infinite()
        && policy.age(SystemTime::now()) >= crate::sources::DAY_CACHE_TTL
}

/// Strip the stale-serve marker. Called on every `put` path — both the
/// revalidated cached response and a fresh 200 — so the stored entry and
/// the served response never carry a marker a later serve would misread.
pub(super) fn strip_stale_serve_marker(response: &mut HttpResponse) {
    response.headers.remove(STALE_SERVE_AGE_HEADER);
}

#[derive(Serialize, Deserialize)]
struct StoredResponse {
    response: HttpResponse,
    policy: CachePolicy,
    vary: String,
}

type Observer = Arc<dyn Fn(&Path, &str) + Send + Sync>;

#[derive(Clone)]
pub(crate) struct SqliteCacheManager {
    config: ResolvedCacheConfig,
    observe_get: Observer,
    after_put: Observer,
}

// Existing internal callers share the new backend while their API stays stable.
pub(crate) type SizeAwareCacheManager = SqliteCacheManager;

impl SqliteCacheManager {
    pub(crate) fn new(_path: PathBuf, config: ResolvedCacheConfig) -> Result<Self, BioMcpError> {
        if !crate::sources::cache_is_bypassed() {
            let mut store = sqlite::Store::open(&config.cache_root)?;
            sqlite::maintain(
                &mut store,
                &config,
                None,
                None,
                false,
                sqlite::Maintenance::Startup,
            )?;
        }
        Ok(Self {
            config,
            observe_get: Arc::new(|_, _| {}),
            after_put: Arc::new(|_, _| {}),
        })
    }

    pub(crate) async fn new_with_deadline(
        path: PathBuf,
        config: ResolvedCacheConfig,
        deadline: &crate::sources::VariantArticleDeadline,
    ) -> Result<Self, BioMcpError> {
        deadline.ensure_time_io()?;
        let manager = tokio::task::spawn_blocking(move || Self::new(path, config))
            .await
            .map_err(|error| BioMcpError::Io(io::Error::other(error)))??;
        deadline.ensure_time_io()?;
        Ok(manager)
    }

    #[cfg(test)]
    pub(crate) fn new_with_cache_observers<G, A>(
        path: PathBuf,
        config: ResolvedCacheConfig,
        observe_get: G,
        after_put: A,
    ) -> Self
    where
        G: Fn(&Path, &str) + Send + Sync + 'static,
        A: Fn(&Path, &str) + Send + Sync + 'static,
    {
        let _ = path;
        Self {
            config,
            observe_get: Arc::new(observe_get),
            after_put: Arc::new(after_put),
        }
    }
}

#[async_trait]
impl CacheManager for SqliteCacheManager {
    async fn get(&self, key: &str) -> http_cache::Result<Option<(HttpResponse, CachePolicy)>> {
        if crate::sources::cache_is_bypassed() {
            return Ok(None);
        }
        let key = key.to_owned();
        let config = self.config.clone();
        let context = sqlite::request_key::current();
        (self.observe_get)(&config.cache_root.join(sqlite::DATABASE_NAME), &key);
        let mut value = tokio::task::spawn_blocking(move || -> io::Result<_> {
            let mut store = sqlite::Store::open(&config.cache_root)?;
            let records = if let Some(context) = &context {
                store.variants("http", &context.base)?
            } else {
                store.get("http", &key)?.into_iter().collect()
            };
            for record in records {
                let mut stored: StoredResponse =
                    serde_json::from_slice(&record.metadata).map_err(io::Error::other)?;
                if let Some(context) = &context
                    && context.variant(&stored.vary).as_deref() != Some(record.key.as_str())
                {
                    continue;
                }
                let body = if let Some(context) = &context {
                    let Some(row) = store.get("http", &record.key)? else {
                        continue;
                    };
                    // A publisher can replace this variant after the metadata
                    // probe. Headers, policy and body must come from one row.
                    stored = serde_json::from_slice(&row.metadata).map_err(io::Error::other)?;
                    if context.variant(&stored.vary).as_deref() != Some(row.key.as_str()) {
                        continue;
                    }
                    row.body
                } else {
                    record.body
                };
                stored.response.body = body;
                if let Some(context) = &context {
                    stored.response.url = context.original_url.clone();
                }
                return Ok(Some((stored.response, stored.policy)));
            }
            Ok(None)
        })
        .await??;
        if let Some((response, policy)) = &mut value {
            if day_cache_entry_expired(policy) {
                return Ok(None);
            }
            stamp_stale_serve_marker(response, policy);
        }
        Ok(value)
    }

    async fn put(
        &self,
        key: String,
        mut response: HttpResponse,
        policy: CachePolicy,
    ) -> http_cache::Result<HttpResponse> {
        strip_stale_serve_marker(&mut response);
        if crate::sources::cache_is_bypassed() {
            return Ok(response);
        }
        let context = sqlite::request_key::current();
        let vary = response
            .headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("vary"))
            .map(|(_, v)| v.clone())
            .unwrap_or_default();
        let canonical_url = context
            .as_ref()
            .map(|context| context.canonical_url.clone());
        let (base, key) = match context {
            Some(context) => match context.variant(&vary) {
                Some(key) => (context.base, key),
                None => return Ok(response),
            },
            None => (key.clone(), key),
        };
        let mut metadata_response = response.clone();
        metadata_response.body.clear();
        if let Some(url) = canonical_url {
            metadata_response.url = url;
        }
        let metadata = serde_json::to_vec(&StoredResponse {
            response: metadata_response,
            policy,
            vary,
        })?;
        let body = response.body.clone();
        let config = self.config.clone();
        let publication = crate::sources::current_cache_publication_state().unwrap_or_default();
        // Await committed publication and its bounded cleanup before returning to a
        // short-lived runtime. An armed publication uses the existing safe-return scope.
        publication.arm();
        let operation = tokio::task::spawn_blocking(move || -> io::Result<()> {
            let mut store = sqlite::Store::open(&config.cache_root)?;
            store.put("http", &key, &base, &metadata, &body, None)?;
            sqlite::maintain(
                &mut store,
                &config,
                None,
                Some(config.max_size),
                false,
                sqlite::Maintenance::Pressure,
            )?;
            Ok(())
        });
        crate::sources::with_variant_article_safe_return(operation).await??;
        (self.after_put)(
            &self.config.cache_root.join(sqlite::DATABASE_NAME),
            &base_key_for_observer(&response),
        );
        Ok(response)
    }

    async fn delete(&self, key: &str) -> http_cache::Result<()> {
        if crate::sources::cache_is_bypassed() {
            return Ok(());
        }
        let config = self.config.clone();
        let key = key.to_owned();
        let context = sqlite::request_key::current();
        tokio::task::spawn_blocking(move || -> io::Result<()> {
            let mut store = sqlite::Store::open(&config.cache_root)?;
            if let Some(context) = context {
                for row in store.variants("http", &context.base)? {
                    store.delete("http", &row.key)?;
                }
                Ok(())
            } else {
                store.delete("http", &key)
            }
        })
        .await??;
        Ok(())
    }
}

fn base_key_for_observer(response: &HttpResponse) -> String {
    format!("GET:{}", response.url)
}

#[cfg(test)]
mod tests {
    use super::ResolvedCacheConfig;
    use crate::cache::{CacheConfigOrigins, ConfigOrigin, DiskFreeThreshold};
    use std::path::Path;
    use std::time::Duration;

    fn test_config(
        root: &Path,
        max_size: u64,
        min_disk_free: DiskFreeThreshold,
    ) -> ResolvedCacheConfig {
        ResolvedCacheConfig {
            cache_root: root.to_path_buf(),
            max_size,
            min_disk_free,
            max_age: Duration::from_secs(86400),
            origins: CacheConfigOrigins {
                cache_root: ConfigOrigin::Default,
                max_size: ConfigOrigin::Default,
                min_disk_free: ConfigOrigin::Default,
                max_age: ConfigOrigin::Default,
            },
        }
    }
    #[path = "stale_marker_tests.rs"]
    mod stale_marker_tests;

    #[tokio::test]
    #[serial_test::serial(source_env)]
    async fn concurrent_replacement_keeps_http_headers_and_body_from_the_same_row() {
        use super::*;
        use std::sync::atomic::{AtomicBool, Ordering};

        let _mode = crate::sources::test_cache_mode::infinite();
        let root = crate::test_support::TempDirGuard::new("sqlite-coherent-response");
        let manager = SqliteCacheManager::new(
            root.path().join(sqlite::DATABASE_NAME),
            test_config(root.path(), 100000000, DiskFreeThreshold::Bytes(1)),
        )
        .unwrap();
        let request = reqwest::Client::new()
            .get("https://example.test/resource")
            .build()
            .unwrap();
        let context = sqlite::request_key::RequestContext::from_request(&request).unwrap();
        let key = context.variant("").unwrap();
        let request = http::Request::get("https://example.test/resource")
            .body(())
            .unwrap();
        let response = http::Response::builder()
            .header("cache-control", "public,max-age=3600")
            .body(())
            .unwrap();
        let mut stored = StoredResponse {
            response: HttpResponse {
                body: Vec::new(),
                headers: std::collections::HashMap::from([
                    ("x-generation".into(), "0".into()),
                    ("x-provider-padding".into(), "x".repeat(256 * 1024)),
                ]),
                status: 200,
                url: reqwest::Url::parse("https://example.test/resource").unwrap(),
                version: http_cache::HttpVersion::Http11,
            },
            policy: CachePolicy::new(&request, &response),
            vary: String::new(),
        };
        let mut store = sqlite::Store::open(root.path()).unwrap();
        store
            .put(
                "http",
                &key,
                &context.base,
                &serde_json::to_vec(&stored).unwrap(),
                b"0",
                None,
            )
            .unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = stop.clone();
        let writer_root = root.path().to_path_buf();
        let writer = std::thread::spawn(move || {
            let mut store = sqlite::Store::open(&writer_root).unwrap();
            let mut generation = 0;
            let mut replacements = 0;
            while !stopped.load(Ordering::Relaxed) {
                generation += 1;
                let body = generation.to_string();
                stored
                    .response
                    .headers
                    .insert("x-generation".into(), body.clone());
                if store
                    .put(
                        "http",
                        &key,
                        &context.base,
                        &serde_json::to_vec(&stored).unwrap(),
                        body.as_bytes(),
                        None,
                    )
                    .is_ok()
                {
                    replacements += 1;
                }
            }
            replacements
        });
        let mut mismatch = None;
        let mut hits = 0;
        for _ in 0..100 {
            let context = sqlite::request_key::RequestContext::from_request(
                &reqwest::Client::new()
                    .get("https://example.test/resource")
                    .build()
                    .unwrap(),
            )
            .unwrap();
            if let Ok(Some((response, _))) = sqlite::request_key::REQUEST
                .scope(context, manager.get("unused"))
                .await
            {
                hits += 1;
                if response.headers["x-generation"].as_bytes() != response.body {
                    mismatch = Some(response.headers["x-generation"].clone());
                    break;
                }
            }
        }
        stop.store(true, Ordering::Relaxed);
        assert!(writer.join().unwrap() > 0, "writer must replace a response");
        assert!(hits > 0, "reader must serve a cached response");
        assert!(
            mismatch.is_none(),
            "headers came from an older cached response: {mismatch:?}"
        );
    }
}
