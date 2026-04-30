//! Source clients and shared HTTP utilities for upstream biomedical APIs.

use std::borrow::Cow;
use std::future::Future;
use std::path::Path;
use std::sync::OnceLock;
use std::time::Duration;

use http::Extensions;
use http_cache_reqwest::{Cache, CacheMode, CacheOptions, HttpCache, HttpCacheOptions};
use reqwest::StatusCode;
use reqwest::header::{CACHE_CONTROL, HeaderMap, HeaderValue, RETRY_AFTER};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Middleware, Next, RequestBuilder};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use tracing::warn;

use crate::error::BioMcpError;

pub(crate) mod alphagenome;
pub(crate) mod cbioportal;
pub(crate) mod cbioportal_download;
pub(crate) mod cbioportal_study;
pub(crate) mod chembl;
pub(crate) mod civic;
pub(crate) mod clingen;
pub(crate) mod clinicaltrials;
pub(crate) mod complexportal;
pub(crate) mod cpic;
pub(crate) mod cvx;
pub(crate) mod ddinter;
pub(crate) mod dgidb;
pub(crate) mod disgenet;
pub(crate) mod ema;
pub(crate) mod enrichr;
pub(crate) mod europepmc;
pub(crate) mod gnomad;
pub(crate) mod gprofiler;
pub(crate) mod gtex;
pub(crate) mod gtr;
pub(crate) mod gwas;
pub(crate) mod hpa;
pub(crate) mod hpo;
pub(crate) mod interpro;
pub(crate) mod kegg;
pub(crate) mod litsense2;
pub(crate) mod medlineplus;
pub(crate) mod monarch;
pub(crate) mod mychem;
pub(crate) mod mydisease;
pub(crate) mod mygene;
pub(crate) mod myvariant;
pub(crate) mod ncbi_efetch;
pub(crate) mod ncbi_idconv;
pub(crate) mod nci_cts;
pub(crate) mod nih_reporter;
pub(crate) mod ols4;
pub(crate) mod oncokb;
pub(crate) mod openfda;
pub(crate) mod opentargets;
pub(crate) mod pharmgkb;
pub(crate) mod pmc_oa;
pub(crate) mod pubmed;
pub(crate) mod pubtator;
pub(crate) mod quickgo;
pub(crate) mod rate_limit;
pub(crate) mod reactome;
pub(crate) mod seer;
pub(crate) mod semantic_scholar;
pub(crate) mod string;
pub(crate) mod umls;
pub(crate) mod uniprot;
pub(crate) mod vaers;
pub(crate) mod who_ivd;
pub(crate) mod who_pq;
pub(crate) mod wikipathways;

const ERROR_BODY_MAX_BYTES: usize = 2048;
pub(crate) const DEFAULT_MAX_BODY_BYTES: usize = 8 * 1024 * 1024;
pub(crate) const BIOTHINGS_MAX_RESULT_WINDOW: usize = 10_000;

static HTTP_CLIENT: OnceLock<ClientWithMiddleware> = OnceLock::new();
static SEMANTIC_SCHOLAR_SHARED_POOL_HTTP_CLIENT: OnceLock<ClientWithMiddleware> = OnceLock::new();
static STREAMING_HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

tokio::task_local! {
    static NO_CACHE: bool;
}

fn parse_cache_mode(value: Option<&str>) -> Option<CacheMode> {
    match value {
        Some("infinite") => Some(CacheMode::ForceCache),
        Some("off") => Some(CacheMode::NoStore),
        Some("default") | Some("") | None => None,
        Some(other) => {
            warn!("Unknown BIOMCP_CACHE_MODE={other:?}, using default");
            None
        }
    }
}

fn env_cache_mode() -> Option<CacheMode> {
    static MODE: OnceLock<Option<CacheMode>> = OnceLock::new();
    *MODE.get_or_init(|| {
        let mode = std::env::var("BIOMCP_CACHE_MODE")
            .ok()
            .map(|s| s.trim().to_ascii_lowercase());
        parse_cache_mode(mode.as_deref())
    })
}

fn resolve_cache_mode(
    no_cache: bool,
    authenticated: bool,
    env_mode: Option<CacheMode>,
) -> Option<CacheMode> {
    if no_cache || authenticated {
        return Some(CacheMode::NoStore);
    }
    env_mode
}

pub(crate) async fn with_no_cache<R, F>(no_cache: bool, fut: F) -> R
where
    F: Future<Output = R>,
{
    NO_CACHE.scope(no_cache, fut).await
}

pub(crate) fn is_no_cache_enabled() -> bool {
    matches!(NO_CACHE.try_with(|v| *v), Ok(true))
}

pub(crate) fn apply_cache_mode(req: RequestBuilder) -> RequestBuilder {
    let no_cache = is_no_cache_enabled();
    if let Some(mode) = resolve_cache_mode(no_cache, false, env_cache_mode()) {
        return req.with_extension(mode);
    }
    req
}

pub(crate) fn apply_cache_mode_with_auth(
    req: RequestBuilder,
    authenticated: bool,
) -> RequestBuilder {
    let no_cache = is_no_cache_enabled();
    if let Some(mode) = resolve_cache_mode(no_cache, authenticated, env_cache_mode()) {
        return req.with_extension(mode);
    }
    req
}

pub(crate) fn env_base(default: &'static str, env_var: &str) -> Cow<'static, str> {
    std::env::var(env_var)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .map(Cow::Owned)
        .unwrap_or_else(|| Cow::Borrowed(default))
}

pub(crate) fn is_valid_gene_symbol(symbol: &str) -> bool {
    !symbol.is_empty()
        && symbol
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

pub(crate) fn ncbi_api_key() -> Option<String> {
    std::env::var("NCBI_API_KEY")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub(crate) fn s2_api_key() -> Option<String> {
    std::env::var("S2_API_KEY")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub(crate) fn append_ncbi_api_key(req: RequestBuilder, api_key: Option<&str>) -> RequestBuilder {
    if let Some(key) = api_key {
        return req.query(&[("api_key", key)]);
    }
    req
}

fn parse_retry_after_header(headers: &HeaderMap) -> Option<Duration> {
    // Retry-After is interpreted as integer seconds when present.
    headers
        .get(RETRY_AFTER)?
        .to_str()
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()
        .map(Duration::from_secs)
}

fn retry_sleep_duration(attempt: u32, retry_after_floor: Option<Duration>) -> Duration {
    let backoff_ms = 100_u64.saturating_mul(2_u64.saturating_pow(attempt));
    let backoff = Duration::from_millis(backoff_ms);
    match retry_after_floor {
        Some(floor) if floor > backoff => floor,
        _ => backoff,
    }
}

/// Returns a shared HTTP client with retry and caching middleware.
///
/// - Retry: 3 attempts with exponential backoff for transient errors
/// - Retry log level: `DEBUG` — retry attempts are suppressed at the default `WARN` verbosity and
///   visible with `RUST_LOG=debug`
/// - Cache: Disk-based HTTP cache under the resolved canonical cache root
///   (`BIOMCP_CACHE_DIR`, `cache.toml`, or XDG default)
/// - Cache TTL: `Cache-Control: max-stale=86400` makes “no caching headers” responses usable for 24h
#[derive(Clone, Copy)]
enum SharedHttpClientKind {
    Default,
    SemanticScholarSharedPool,
}

#[derive(Debug, thiserror::Error)]
#[error("semantic scholar shared-pool rate limit exceeded")]
struct SemanticScholarSharedPoolRateLimitError;

struct RetryAfterTooManyRequestsMiddleware;

#[async_trait::async_trait]
impl Middleware for RetryAfterTooManyRequestsMiddleware {
    async fn handle(
        &self,
        req: reqwest::Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<reqwest::Response> {
        let response = next.run(req, extensions).await?;
        if response.status() == StatusCode::TOO_MANY_REQUESTS
            && let Some(duration) = parse_retry_after_header(response.headers())
        {
            tokio::time::sleep(duration).await;
        }
        Ok(response)
    }
}

#[derive(Clone, Copy, Debug)]
struct SemanticScholarSharedPoolRateLimitMiddleware;

#[async_trait::async_trait]
impl Middleware for SemanticScholarSharedPoolRateLimitMiddleware {
    async fn handle(
        &self,
        req: reqwest::Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<reqwest::Response> {
        let response = next.run(req, extensions).await?;
        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            return Err(reqwest_middleware::Error::middleware(
                SemanticScholarSharedPoolRateLimitError,
            ));
        }
        Ok(response)
    }
}

fn apply_migration_non_fatal<M, W>(cache_root: &Path, migrate: M, warn_fn: W)
where
    M: FnOnce(&Path) -> std::io::Result<crate::cache::MigrationOutcome>,
    W: FnOnce(&std::io::Error),
{
    if let Err(err) = migrate(cache_root) {
        warn_fn(&err);
    }
}

fn build_http_client(kind: SharedHttpClientKind) -> Result<ClientWithMiddleware, BioMcpError> {
    let config = crate::cache::resolve_cache_config()?;
    let cache_root = config.cache_root.clone();
    apply_migration_non_fatal(&cache_root, crate::cache::migrate_http_cache, |err| {
        warn!(
            cache_root = %cache_root.display(),
            "HTTP cache directory migration failed; continuing with normal cache initialization: {err}"
        );
    });
    let cache_path = cache_root.join("http");
    std::fs::create_dir_all(&cache_path)?;

    let mut default_headers = HeaderMap::new();
    default_headers.insert(CACHE_CONTROL, HeaderValue::from_static("max-stale=86400"));

    let base_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .user_agent(concat!("biomcp-cli/", env!("CARGO_PKG_VERSION")))
        .default_headers(default_headers)
        .build()
        .map_err(BioMcpError::HttpClientInit)?;

    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);

    let cache_options = HttpCacheOptions {
        cache_options: Some(CacheOptions {
            // Shared-cache semantics: do not store private/authenticated responses.
            shared: true,
            ..CacheOptions::default()
        }),
        ..HttpCacheOptions::default()
    };

    let builder = ClientBuilder::new(base_client).with(Cache(HttpCache {
        mode: CacheMode::Default,
        manager: crate::cache::SizeAwareCacheManager::new(cache_path, config),
        options: cache_options,
    }));
    let builder = builder.with(
        RetryTransientMiddleware::new_with_policy(retry_policy)
            .with_retry_log_level(tracing::Level::DEBUG),
    );
    let builder = match kind {
        SharedHttpClientKind::Default => builder.with(RetryAfterTooManyRequestsMiddleware),
        SharedHttpClientKind::SemanticScholarSharedPool => {
            builder.with(SemanticScholarSharedPoolRateLimitMiddleware)
        }
    };
    Ok(builder.with(rate_limit::RateLimitMiddleware::new()).build())
}

#[cfg(test)]
pub(crate) fn test_client() -> Result<ClientWithMiddleware, BioMcpError> {
    let base_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .connect_timeout(std::time::Duration::from_secs(10))
        .user_agent(concat!("biomcp-cli/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(BioMcpError::HttpClientInit)?;
    Ok(reqwest_middleware::ClientBuilder::new(base_client).build())
}

pub(crate) fn shared_client() -> Result<ClientWithMiddleware, BioMcpError> {
    if let Some(client) = HTTP_CLIENT.get() {
        return Ok(client.clone());
    }

    let client = build_http_client(SharedHttpClientKind::Default)?;

    match HTTP_CLIENT.set(client.clone()) {
        Ok(()) => Ok(client),
        Err(_) => HTTP_CLIENT.get().cloned().ok_or_else(|| BioMcpError::Api {
            api: "http-client".into(),
            message: "Shared HTTP client initialization race".into(),
        }),
    }
}

pub(crate) fn semantic_scholar_shared_pool_client() -> Result<ClientWithMiddleware, BioMcpError> {
    if let Some(client) = SEMANTIC_SCHOLAR_SHARED_POOL_HTTP_CLIENT.get() {
        return Ok(client.clone());
    }

    let client = build_http_client(SharedHttpClientKind::SemanticScholarSharedPool)?;

    match SEMANTIC_SCHOLAR_SHARED_POOL_HTTP_CLIENT.set(client.clone()) {
        Ok(()) => Ok(client),
        Err(_) => SEMANTIC_SCHOLAR_SHARED_POOL_HTTP_CLIENT
            .get()
            .cloned()
            .ok_or_else(|| BioMcpError::Api {
                api: "http-client".into(),
                message: "Semantic Scholar shared-pool HTTP client initialization race".into(),
            }),
    }
}

pub(crate) fn is_semantic_scholar_shared_pool_rate_limit_error(
    err: &reqwest_middleware::Error,
) -> bool {
    match err {
        reqwest_middleware::Error::Middleware(source) => {
            source
                .chain()
                .any(|cause| cause.is::<SemanticScholarSharedPoolRateLimitError>())
                || source
                    .to_string()
                    .contains("semantic scholar shared-pool rate limit exceeded")
        }
        reqwest_middleware::Error::Reqwest(_) => false,
    }
}

/// Returns a shared HTTP client without middleware.
///
/// Use this for requests with streaming bodies (e.g., multipart) that cannot be cloned and therefore
/// cannot pass through the retry/cache middleware stack.
pub(crate) fn streaming_http_client() -> Result<reqwest::Client, BioMcpError> {
    if let Some(client) = STREAMING_HTTP_CLIENT.get() {
        return Ok(client.clone());
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .user_agent(concat!("biomcp-cli/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(BioMcpError::HttpClientInit)?;

    match STREAMING_HTTP_CLIENT.set(client.clone()) {
        Ok(()) => Ok(client),
        Err(_) => STREAMING_HTTP_CLIENT
            .get()
            .cloned()
            .ok_or_else(|| BioMcpError::Api {
                api: "http-client".into(),
                message: "Shared streaming HTTP client initialization race".into(),
            }),
    }
}

/// Retry wrapper for streaming requests that bypass middleware.
///
/// `build_request` is invoked on each attempt so non-cloneable request bodies
/// can be reconstructed safely.
pub(crate) async fn retry_send<F, Fut>(
    api: &str,
    max_retries: u32,
    build_request: F,
) -> Result<reqwest::Response, BioMcpError>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<reqwest::Response, reqwest::Error>>,
{
    let total_attempts = max_retries.saturating_add(1);
    let mut last_http_err: Option<reqwest::Error> = None;
    let mut last_server_status: Option<reqwest::StatusCode> = None;

    for attempt in 0..total_attempts {
        let mut retry_after_floor = None;
        match build_request().await {
            Ok(resp)
                if resp.status().is_server_error()
                    || resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS =>
            {
                let status = resp.status();
                if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                    retry_after_floor = parse_retry_after_header(resp.headers());
                }
                last_server_status = Some(status);
            }
            Ok(resp) => return Ok(resp),
            Err(err) => {
                if err.is_timeout() || err.is_connect() {
                    last_http_err = Some(err);
                } else {
                    return Err(BioMcpError::Http(err));
                }
            }
        }

        if attempt + 1 < total_attempts {
            tokio::time::sleep(retry_sleep_duration(attempt, retry_after_floor)).await;
        }
    }

    if let Some(status) = last_server_status {
        return Err(BioMcpError::Api {
            api: api.to_string(),
            message: format!("HTTP {status} after {total_attempts} attempts"),
        });
    }

    if let Some(err) = last_http_err {
        return Err(BioMcpError::Http(err));
    }

    Err(BioMcpError::Api {
        api: api.to_string(),
        message: format!("All retry attempts exhausted after {total_attempts} attempts"),
    })
}

pub(crate) fn body_excerpt(bytes: &[u8]) -> String {
    let full = String::from_utf8_lossy(bytes);

    let truncated: &str = if full.len() > ERROR_BODY_MAX_BYTES {
        let mut end = ERROR_BODY_MAX_BYTES;
        while end > 0 && !full.is_char_boundary(end) {
            end -= 1;
        }
        &full[..end]
    } else {
        full.as_ref()
    };

    let mut s = truncated.trim().replace(['\n', '\r', '\t'], " ");
    if full.len() > ERROR_BODY_MAX_BYTES {
        s.push_str(" …");
    }
    s
}

fn html_sniff_prefix(body: &[u8]) -> String {
    let prefix_len = body.len().min(128);
    String::from_utf8_lossy(&body[..prefix_len])
        .trim_start()
        .to_ascii_lowercase()
}

pub(crate) fn response_body_is_html(content_type: Option<&HeaderValue>, body: &[u8]) -> bool {
    if let Some(content_type) = content_type
        && let Ok(raw) = content_type.to_str()
    {
        let raw = raw.trim();
        if !raw.is_empty() {
            let media_type = raw
                .split(';')
                .next()
                .map(str::trim)
                .unwrap_or_default()
                .to_ascii_lowercase();
            return matches!(media_type.as_str(), "text/html" | "application/xhtml+xml");
        }
    }

    let sniff = html_sniff_prefix(body);
    sniff.starts_with("<!doctype") || sniff.starts_with("<html")
}

pub(crate) fn summarize_http_error_body(content_type: Option<&HeaderValue>, body: &[u8]) -> String {
    if response_body_is_html(content_type, body) {
        "HTML error page".to_string()
    } else {
        body_excerpt(body)
    }
}

pub(crate) fn ensure_json_content_type(
    api: &str,
    content_type: Option<&HeaderValue>,
    body: &[u8],
) -> Result<(), BioMcpError> {
    let mut invalid_content_type = false;
    let raw = match content_type {
        Some(content_type) => match content_type.to_str() {
            Ok(value) => Some(value.trim()),
            Err(_) => {
                invalid_content_type = true;
                None
            }
        },
        None => None,
    };

    if response_body_is_html(content_type, body) {
        let message = match raw.filter(|value| !value.is_empty()) {
            Some(raw) => format!(
                "Unexpected HTML response (content-type: {raw}): {}",
                summarize_http_error_body(content_type, body)
            ),
            None => format!(
                "Unexpected HTML response: {}",
                summarize_http_error_body(content_type, body)
            ),
        };
        return Err(BioMcpError::Api {
            api: api.to_string(),
            message,
        });
    }

    if invalid_content_type {
        warn!(
            source = api,
            "Response content-type header was not valid UTF-8; attempting JSON parse"
        );
        return Ok(());
    }

    let Some(raw) = raw.filter(|value| !value.is_empty()) else {
        return Ok(());
    };

    let media_type = raw
        .split(';')
        .next()
        .map(str::trim)
        .unwrap_or_default()
        .to_ascii_lowercase();
    let is_json = media_type == "application/json"
        || media_type == "text/json"
        || media_type.ends_with("+json");
    if !is_json {
        warn!(
            source = api,
            content_type = raw,
            "Unexpected non-JSON content type; attempting JSON parse for compatibility"
        );
    }

    Ok(())
}

pub(crate) fn validate_biothings_result_window(
    context: &str,
    limit: usize,
    offset: usize,
) -> Result<(), BioMcpError> {
    if offset >= BIOTHINGS_MAX_RESULT_WINDOW {
        return Err(BioMcpError::InvalidArgument(format!(
            "--offset must be less than {BIOTHINGS_MAX_RESULT_WINDOW} for {context}"
        )));
    }

    if offset.saturating_add(limit) > BIOTHINGS_MAX_RESULT_WINDOW {
        return Err(BioMcpError::InvalidArgument(format!(
            "--offset + --limit must be <= {BIOTHINGS_MAX_RESULT_WINDOW} for {context}"
        )));
    }

    Ok(())
}

pub(crate) async fn read_limited_body_with_limit(
    mut resp: reqwest::Response,
    api: &str,
    max_bytes: usize,
) -> Result<Vec<u8>, BioMcpError> {
    let mut body: Vec<u8> = Vec::new();

    while let Some(chunk) = resp.chunk().await? {
        let next_len = body.len().saturating_add(chunk.len());
        if next_len > max_bytes {
            return Err(BioMcpError::Api {
                api: api.to_string(),
                message: format!("Response body exceeded {max_bytes} bytes"),
            });
        }
        body.extend_from_slice(&chunk);
    }

    Ok(body)
}

pub(crate) async fn read_limited_body(
    resp: reqwest::Response,
    api: &str,
) -> Result<Vec<u8>, BioMcpError> {
    read_limited_body_with_limit(resp, api, DEFAULT_MAX_BODY_BYTES).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{TempDirGuard, set_env_var};
    use std::path::Path;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    use tokio::sync::MutexGuard;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn env_lock() -> MutexGuard<'static, ()> {
        crate::test_support::env_lock().blocking_lock()
    }

    #[test]
    fn parse_cache_mode_returns_none_for_default_or_unset() {
        assert!(parse_cache_mode(None).is_none());
        assert!(parse_cache_mode(Some("default")).is_none());
        assert!(parse_cache_mode(Some("")).is_none());
    }

    #[test]
    fn parse_cache_mode_returns_force_cache_for_infinite() {
        assert!(matches!(
            parse_cache_mode(Some("infinite")),
            Some(CacheMode::ForceCache)
        ));
    }

    #[test]
    fn parse_cache_mode_returns_no_store_for_off() {
        assert!(matches!(
            parse_cache_mode(Some("off")),
            Some(CacheMode::NoStore)
        ));
    }

    #[test]
    fn parse_cache_mode_returns_none_for_unknown_values() {
        assert!(parse_cache_mode(Some("bogus")).is_none());
    }

    #[test]
    fn resolve_cache_mode_prioritizes_no_cache_over_env() {
        assert!(matches!(
            resolve_cache_mode(true, false, Some(CacheMode::ForceCache)),
            Some(CacheMode::NoStore)
        ));
    }

    #[test]
    fn resolve_cache_mode_prioritizes_auth_over_env() {
        assert!(matches!(
            resolve_cache_mode(false, true, Some(CacheMode::ForceCache)),
            Some(CacheMode::NoStore)
        ));
    }

    #[test]
    fn resolve_cache_mode_uses_env_when_no_overrides() {
        assert!(matches!(
            resolve_cache_mode(false, false, Some(CacheMode::ForceCache)),
            Some(CacheMode::ForceCache)
        ));
    }

    #[test]
    fn resolve_cache_mode_defaults_to_none() {
        assert!(resolve_cache_mode(false, false, None).is_none());
    }

    #[test]
    fn response_body_is_html_detects_html_from_content_type() {
        assert!(response_body_is_html(
            Some(&HeaderValue::from_static("text/html; charset=utf-8")),
            b"upstream failure",
        ));
    }

    #[test]
    fn response_body_is_html_detects_html_from_doctype_without_header() {
        assert!(response_body_is_html(
            None,
            b"<!DOCTYPE html><html><body>upstream error</body></html>",
        ));
    }

    #[test]
    fn summarize_http_error_body_sanitizes_html() {
        let summary = summarize_http_error_body(
            None,
            b"<html><head><title>404</title></head><body>File not found</body></html>",
        );
        assert!(summary.contains("HTML error page"));
        assert!(!summary.contains("<html"));
        assert!(!summary.contains("<head"));
    }

    #[test]
    fn summarize_http_error_body_preserves_json_excerpt() {
        let summary = summarize_http_error_body(
            Some(&HeaderValue::from_static("application/json")),
            br#"{"error":"not found","code":404}"#,
        );
        assert!(summary.contains("not found"));
        assert!(!summary.contains("HTML error page"));
    }

    #[test]
    fn ensure_json_content_type_rejects_html() {
        let err = ensure_json_content_type(
            "mygene.info",
            Some(&HeaderValue::from_static("text/html; charset=utf-8")),
            b"<html><body>upstream error</body></html>",
        )
        .expect_err("html should be rejected");
        let msg = err.to_string();
        assert!(msg.contains("mygene.info"));
        assert!(msg.contains("HTML"));
    }

    #[test]
    fn ensure_json_content_type_accepts_json() {
        let ok = ensure_json_content_type(
            "mygene.info",
            Some(&HeaderValue::from_static("application/json; charset=utf-8")),
            b"{\"ok\":true}",
        );
        assert!(ok.is_ok());
    }

    #[test]
    fn ensure_json_content_type_allows_non_json_compat_mode() {
        let ok = ensure_json_content_type(
            "mygene.info",
            Some(&HeaderValue::from_static("text/plain")),
            b"{\"ok\":true}",
        );
        assert!(ok.is_ok());
    }

    #[test]
    fn validate_biothings_result_window_accepts_bounds() {
        let ok = validate_biothings_result_window("MyVariant search", 10, 9_990);
        assert!(ok.is_ok());
    }

    #[test]
    fn validate_biothings_result_window_rejects_offset_at_window() {
        let err = validate_biothings_result_window("MyVariant search", 5, 10_000)
            .expect_err("offset at window should fail");
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
        assert!(err.to_string().contains("--offset must be less than 10000"));
    }

    #[test]
    fn validate_biothings_result_window_rejects_window_overflow() {
        let err = validate_biothings_result_window("MyVariant search", 6, 9_995)
            .expect_err("offset + limit overflow should fail");
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
        assert!(
            err.to_string()
                .contains("--offset + --limit must be <= 10000")
        );
    }

    #[test]
    fn parse_retry_after_header_parses_integer_seconds() {
        let mut headers = HeaderMap::new();
        headers.insert(RETRY_AFTER, HeaderValue::from_static("2"));
        assert_eq!(
            parse_retry_after_header(&headers),
            Some(Duration::from_secs(2))
        );
    }

    #[test]
    fn retry_sleep_duration_uses_retry_after_as_floor() {
        assert_eq!(
            retry_sleep_duration(0, Some(Duration::from_secs(2))),
            Duration::from_secs(2)
        );
        assert_eq!(
            retry_sleep_duration(2, Some(Duration::from_millis(100))),
            Duration::from_millis(400)
        );
    }

    #[tokio::test]
    async fn retry_send_retries_on_too_many_requests() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/retry"))
            .and(query_param("attempt", "0"))
            .respond_with(ResponseTemplate::new(429))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/retry"))
            .and(query_param("attempt", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .expect(1)
            .mount(&server)
            .await;

        let client = reqwest::Client::new();
        let url = format!("{}/retry", server.uri());
        let attempts = Arc::new(AtomicUsize::new(0));
        let resp = retry_send("test-api", 2, {
            let client = client.clone();
            let url = url.clone();
            let attempts = attempts.clone();
            move || {
                let client = client.clone();
                let url = url.clone();
                let attempts = attempts.clone();
                async move {
                    let attempt = attempts.fetch_add(1, Ordering::SeqCst);
                    client
                        .get(&url)
                        .query(&[("attempt", attempt.to_string())])
                        .send()
                        .await
                }
            }
        })
        .await
        .expect("retry_send should retry on 429");

        assert_eq!(resp.status(), reqwest::StatusCode::OK);
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn http_cache_dir_default_root_uses_xdg_cache_home_biomcp_http() {
        let _lock = env_lock();
        let root = TempDirGuard::new("http-cache-default-root");
        let cache_home = root.path().join("cache-home");
        let config_home = root.path().join("config-home");
        let _cache_home = set_env_var("XDG_CACHE_HOME", Some(&cache_home.to_string_lossy()));
        let _config_home = set_env_var("XDG_CONFIG_HOME", Some(&config_home.to_string_lossy()));
        let _cache_dir = set_env_var("BIOMCP_CACHE_DIR", None);
        let _min_disk_free = set_env_var("BIOMCP_CACHE_MIN_DISK_FREE", None);

        let path = crate::cache::resolve_cache_config()
            .expect("default cache root should resolve")
            .cache_root
            .join("http");

        assert_eq!(path, cache_home.join("biomcp").join("http"));
    }

    #[test]
    fn http_cache_dir_env_override_uses_biomcp_cache_dir_http() {
        let _lock = env_lock();
        let root = TempDirGuard::new("http-cache-env-override");
        let cache_home = root.path().join("cache-home");
        let config_home = root.path().join("config-home");
        let override_root = root.path().join("override-root");
        let _cache_home = set_env_var("XDG_CACHE_HOME", Some(&cache_home.to_string_lossy()));
        let _config_home = set_env_var("XDG_CONFIG_HOME", Some(&config_home.to_string_lossy()));
        let _cache_dir = set_env_var("BIOMCP_CACHE_DIR", Some(&override_root.to_string_lossy()));
        let _min_disk_free = set_env_var("BIOMCP_CACHE_MIN_DISK_FREE", None);

        let path = crate::cache::resolve_cache_config()
            .expect("env override cache root should resolve")
            .cache_root
            .join("http");

        assert_eq!(path, override_root.join("http"));
    }

    #[test]
    fn apply_migration_non_fatal_warns_and_continues_on_error() {
        let mut warned: Vec<std::io::ErrorKind> = Vec::new();

        apply_migration_non_fatal(
            Path::new("/unused"),
            |_| {
                Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "test error",
                ))
            },
            |err: &std::io::Error| warned.push(err.kind()),
        );

        assert_eq!(warned, vec![std::io::ErrorKind::PermissionDenied]);
    }

    #[test]
    fn build_http_client_renames_legacy_http_cache_before_client_init() {
        let _lock = env_lock();
        let root = TempDirGuard::new("http-cache-migration");
        let cache_home = root.path().join("cache-home");
        let config_home = root.path().join("config-home");
        let override_root = root.path().join("override-root");
        let legacy_dir = override_root.join("http-cacache");
        std::fs::create_dir_all(&legacy_dir).expect("create legacy dir");
        std::fs::write(legacy_dir.join("sentinel.txt"), b"cached payload").expect("write sentinel");

        let _cache_home = set_env_var("XDG_CACHE_HOME", Some(&cache_home.to_string_lossy()));
        let _config_home = set_env_var("XDG_CONFIG_HOME", Some(&config_home.to_string_lossy()));
        let _cache_dir = set_env_var("BIOMCP_CACHE_DIR", Some(&override_root.to_string_lossy()));
        let _min_disk_free = set_env_var("BIOMCP_CACHE_MIN_DISK_FREE", None);

        let result = build_http_client(SharedHttpClientKind::Default);

        assert!(
            result.is_ok(),
            "client should initialize even with legacy cache migration"
        );
        assert!(override_root.join("http").is_dir());
        assert!(override_root.join("http").join("sentinel.txt").is_file());
        assert!(!override_root.join("http-cacache").exists());
    }
}
