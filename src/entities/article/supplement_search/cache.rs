//! Extracted supplement text shares the managed HTTP cache's accounting,
//! maintenance and keyed reads. The internal URL is an identifier only.
use std::collections::HashMap;

use http_cache::{CacheManager, HttpResponse, HttpVersion};
use http_cache_semantics::CachePolicy;
use serde::{Deserialize, Serialize};

use super::{SupplementFormat, SupplementTextExtraction};
use crate::cache::{ResolvedCacheConfig, SizeAwareCacheManager};

const SCHEMA: u8 = 1;
const MAX_BYTES: usize = 16 * 1024 * 1024;
const CACHE_URL: &str = "https://biomcp.local/cache/article-supplement-text";

#[derive(Deserialize, Serialize)]
struct CachedSupplementText {
    schema: u8,
    format: String,
    extraction: SupplementTextExtraction,
}

fn key(sha256: &str) -> String {
    format!("biomcp:article-supplement-text:v1:{sha256}")
}

fn manager() -> Option<(SizeAwareCacheManager, ResolvedCacheConfig)> {
    let config = crate::cache::resolve_cache_config().ok()?;
    let manager =
        SizeAwareCacheManager::new(config.cache_root.join("http"), config.clone()).ok()?;
    Some((manager, config))
}

pub(super) async fn read_cached_supplement_text(
    sha256: &str,
    format: SupplementFormat,
) -> Option<SupplementTextExtraction> {
    let (manager, config) = manager()?;
    let (response, policy) = match manager.get(&key(sha256)).await {
        Ok(Some(value)) => value,
        Ok(None) => return None,
        Err(error) => {
            crate::cache::warn_local_http_cache("read", error.as_ref());
            return None;
        }
    };
    if response.body.len() > MAX_BYTES || policy.age(std::time::SystemTime::now()) >= config.max_age
    {
        return None;
    }
    let cached: CachedSupplementText = serde_json::from_slice(&response.body).ok()?;
    (cached.schema == SCHEMA && cached.format == format.as_str()).then_some(cached.extraction)
}

pub(super) async fn write_cached_supplement_text(
    sha256: &str,
    format: SupplementFormat,
    extraction: &SupplementTextExtraction,
) {
    let cached = CachedSupplementText {
        schema: SCHEMA,
        format: format.as_str().to_owned(),
        extraction: extraction.clone(),
    };
    let Ok(body) = serde_json::to_vec(&cached) else {
        return;
    };
    if body.len() > MAX_BYTES {
        tracing::warn!("Supplement text cache entry exceeded its size budget");
        return;
    }
    let Some((manager, config)) = manager() else {
        return;
    };
    let cache_control = format!("max-age={}", config.max_age.as_secs());
    let request = http::Request::builder()
        .uri(CACHE_URL)
        .body(())
        .expect("fixed request");
    let response = http::Response::builder()
        .status(200)
        .header("cache-control", &cache_control)
        .body(())
        .expect("fixed response");
    let policy = CachePolicy::new(&request, &response);
    let response = HttpResponse {
        body,
        headers: HashMap::from([("cache-control".to_owned(), cache_control)]),
        status: 200,
        url: reqwest::Url::parse(CACHE_URL).expect("fixed URL"),
        version: HttpVersion::Http11,
    };
    if let Err(error) = manager.put(key(sha256), response, policy).await {
        crate::cache::warn_local_http_cache("write", error.as_ref());
    }
}
