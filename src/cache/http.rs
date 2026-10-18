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
