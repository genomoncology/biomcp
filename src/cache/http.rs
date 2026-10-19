//! Optional HTTP cache failures must not masquerade as provider failures.
use std::error::Error;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use http_cache::{CacheManager, HttpResponse};
use http_cache_semantics::CachePolicy;

use super::SizeAwareCacheManager;

pub(crate) fn warn_local_http_cache(operation: &'static str, error: &(dyn Error + 'static)) {
    static WARNED: AtomicBool = AtomicBool::new(false);
    let mut cause = Some(error);
    let mut local_io = None;
    while let Some(error) = cause {
        if let Some(error) = error.downcast_ref::<io::Error>() {
            local_io = Some(error);
            break;
        }
        cause = error.source();
    }
    // Never log error text: cache errors can contain URLs, keys and paths.
    if local_io.is_some_and(|error| error.kind() == io::ErrorKind::WouldBlock) {
        tracing::debug!(
            operation,
            category = "local_io",
            io_kind = ?local_io.map(io::Error::kind),
            os_code = ?local_io.and_then(io::Error::raw_os_error),
            "local HTTP cache {operation} skipped while another cache operation is active"
        );
        return;
    }
    if WARNED.swap(true, Ordering::Relaxed) {
        return;
    }
    tracing::warn!(
        operation,
        category = if local_io.is_some() { "local_io" } else { "cache_state_or_decode" },
        io_kind = ?local_io.map(io::Error::kind),
        os_code = ?local_io.and_then(io::Error::raw_os_error),
        "local HTTP cache {operation} failed; continuing"
    );
}

pub(crate) struct HttpCacheManager(pub(crate) SizeAwareCacheManager);

fn invocation_expired() -> bool {
    crate::sources::current_variant_article_deadline().is_some_and(|d| d.is_exhausted())
}

#[async_trait]
impl CacheManager for HttpCacheManager {
    async fn get(&self, key: &str) -> http_cache::Result<Option<(HttpResponse, CachePolicy)>> {
        match self.0.get(key).await {
            Err(error) if !invocation_expired() => {
                warn_local_http_cache("read", error.as_ref());
                Ok(None)
            }
            result => result,
        }
    }

    async fn put(
        &self,
        key: String,
        mut response: HttpResponse,
        policy: CachePolicy,
    ) -> http_cache::Result<HttpResponse> {
        super::manager::strip_stale_serve_marker(&mut response);
        let answer = response.clone();
        let publication = crate::sources::current_cache_publication_state();
        match self.0.put(key, response, policy).await {
            Err(error)
                if !invocation_expired()
                    || publication.is_some_and(|p| p.marker().load(Ordering::Acquire)) =>
            {
                warn_local_http_cache("write", error.as_ref());
                Ok(answer)
            }
            result => result,
        }
    }

    async fn delete(&self, key: &str) -> http_cache::Result<()> {
        match self.0.delete(key).await {
            Err(error) if !invocation_expired() => {
                warn_local_http_cache("delete", error.as_ref());
                Ok(())
            }
            result => result,
        }
    }
}

pub(crate) fn bypass_request(request: &reqwest::Request, extensions: &http::Extensions) -> bool {
    extensions.get::<http_cache_reqwest::CacheMode>()
        == Some(&http_cache_reqwest::CacheMode::NoStore)
        || [
            "authorization",
            "proxy-authorization",
            "cookie",
            "x-api-key",
            "api-key",
            "x-auth-token",
        ]
        .iter()
        .any(|name| request.headers().contains_key(*name))
        || (!matches!(request.method().as_str(), "GET" | "HEAD")
            && extensions.get::<http_cache_reqwest::CacheMode>()
                != Some(&http_cache_reqwest::CacheMode::IgnoreRules))
}

/// Scope real request bytes and headers around the library's Parts-only key API.
pub(crate) struct RequestCache(pub(crate) http_cache_reqwest::Cache<HttpCacheManager>);

#[async_trait]
impl reqwest_middleware::Middleware for RequestCache {
    async fn handle(
        &self,
        mut request: reqwest::Request,
        extensions: &mut http::Extensions,
        next: reqwest_middleware::Next<'_>,
    ) -> reqwest_middleware::Result<reqwest::Response> {
        if bypass_request(&request, extensions) {
            return next.run(request, extensions).await;
        }
        let Some(context) = super::sqlite::request_key::RequestContext::from_request(&request)
        else {
            // A streaming body has no stable byte identity. Bypass rather than
            // collapse distinct requests or consume the caller's stream.
            return next.run(request, extensions).await;
        };
        extensions.insert(OriginalCacheUrl(context.original_url.clone()));
        *request.url_mut() = context.canonical_url.clone();
        super::sqlite::request_key::REQUEST
            .scope(
                context,
                reqwest_middleware::Middleware::handle(&self.0, request, extensions, next),
            )
            .await
    }
}

#[derive(Clone)]
struct OriginalCacheUrl(reqwest::Url);

/// Restore the outer trust policy's approved URL before retry and transport.
pub(crate) struct RestoreCacheUrl;

#[async_trait]
impl reqwest_middleware::Middleware for RestoreCacheUrl {
    async fn handle(
        &self,
        mut request: reqwest::Request,
        extensions: &mut http::Extensions,
        next: reqwest_middleware::Next<'_>,
    ) -> reqwest_middleware::Result<reqwest::Response> {
        if let Some(original) = extensions.get::<OriginalCacheUrl>() {
            *request.url_mut() = original.0.clone();
        }
        let mut response = next.run(request, extensions).await?;
        // The pinned adapter folds headers to single values. Preserve all Vary
        // fields before that conversion so each request value enters our key.
        let values = response
            .headers()
            .get_all(http::header::VARY)
            .iter()
            .map(|value| value.to_str())
            .collect::<Result<Vec<_>, _>>();
        let vary = values
            .map(|values| values.join(","))
            .unwrap_or_else(|_| "*".into());
        if !vary.is_empty() {
            response.headers_mut().insert(
                http::header::VARY,
                http::HeaderValue::from_str(&vary).unwrap_or(http::HeaderValue::from_static("*")),
            );
        }
        Ok(response)
    }
}
