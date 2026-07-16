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
use serde::de::DeserializeOwned;
use tracing::warn;

use crate::error::BioMcpError;

pub(crate) mod alphagenome;
pub(crate) mod cancerhotspots;
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
pub(crate) mod figshare;
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
pub(crate) mod mutalyzer;
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
pub(crate) mod orcid;
pub(crate) mod pharmgkb;
pub(crate) mod pmc_oa;
pub(crate) mod provider_url_policy;
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
pub(crate) mod variantvalidator;
pub(crate) mod who_ivd;
pub(crate) mod who_pq;
pub(crate) mod wikipathways;

const ERROR_BODY_MAX_BYTES: usize = 2048;
const MAX_RETRY_AFTER_SLEEP: Duration = Duration::from_secs(5);
const TOTAL_RETRY_SLEEP_BUDGET: Duration = Duration::from_secs(15);
pub(crate) const DEFAULT_MAX_BODY_BYTES: usize = 8 * 1024 * 1024;
pub(crate) const BIOTHINGS_MAX_RESULT_WINDOW: usize = 10_000;

static HTTP_CLIENT: OnceLock<ClientWithMiddleware> = OnceLock::new();
static ORCID_HTTP_CLIENT: OnceLock<ClientWithMiddleware> = OnceLock::new();
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

pub(crate) fn apply_no_store(req: RequestBuilder) -> RequestBuilder {
    req.with_extension(CacheMode::NoStore)
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

// --- Request-construction seam (Tier-2 substrate) ----------------------------
//
// A source client builds a pure `RequestPlan` in a `*_plan()` function (no network,
// no client, no env — directly assertable by a Tier-2 test), then hands it to
// `request_from_plan()` to get a live `RequestBuilder`. The send path (cache mode,
// retry, body limits) is unchanged; only construction becomes a testable seam.

/// HTTP method for a [`RequestPlan`] — the small subset the source clients use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HttpMethod {
    Get,
    Post,
}

/// Outbound request body, as pure data (asserted by Tier-2 tests, applied at send).
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum RequestBody {
    None,
    Form(Vec<(String, String)>),
    #[allow(dead_code)] // used by sources with JSON request bodies (fan-out)
    Json(serde_json::Value),
}

/// A fully-described outbound HTTP request, built without sending.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RequestPlan {
    pub method: HttpMethod,
    /// Path relative to the client base URL (leading slash optional).
    pub path: String,
    pub query: Vec<(String, String)>,
    pub headers: Vec<(String, String)>,
    pub body: RequestBody,
}

impl RequestPlan {
    pub(crate) fn get(path: impl Into<String>) -> Self {
        Self {
            method: HttpMethod::Get,
            path: path.into(),
            query: Vec::new(),
            headers: Vec::new(),
            body: RequestBody::None,
        }
    }

    pub(crate) fn post(path: impl Into<String>) -> Self {
        Self {
            method: HttpMethod::Post,
            path: path.into(),
            query: Vec::new(),
            headers: Vec::new(),
            body: RequestBody::None,
        }
    }

    pub(crate) fn query(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query.push((key.into(), value.into()));
        self
    }

    pub(crate) fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }

    pub(crate) fn form(mut self, form: Vec<(String, String)>) -> Self {
        self.body = RequestBody::Form(form);
        self
    }

    pub(crate) fn json(mut self, json: serde_json::Value) -> Self {
        self.body = RequestBody::Json(json);
        self
    }

    /// First value for a query key (Tier-2 test helper).
    #[cfg(test)]
    pub(crate) fn query_value(&self, key: &str) -> Option<&str> {
        self.query
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// Whether a query key is present at all (Tier-2 test helper).
    #[cfg(test)]
    pub(crate) fn has_query(&self, key: &str) -> bool {
        self.query.iter().any(|(k, _)| k == key)
    }

    /// First value for a header name, case-insensitive (Tier-2 test helper).
    #[cfg(test)]
    pub(crate) fn header_value(&self, key: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v.as_str())
    }
}

fn join_base_path(base: &str, path: &str) -> String {
    format!(
        "{}/{}",
        base.trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}

/// Turn a pure [`RequestPlan`] into a live `RequestBuilder` against `client`/`base`.
pub(crate) fn request_from_plan(
    client: &ClientWithMiddleware,
    base: &str,
    plan: &RequestPlan,
) -> RequestBuilder {
    let url = join_base_path(base, &plan.path);
    let mut req = match plan.method {
        HttpMethod::Get => client.get(&url),
        HttpMethod::Post => client.post(&url),
    };
    for (key, value) in &plan.headers {
        req = req.header(key.as_str(), value.as_str());
    }
    if !plan.query.is_empty() {
        req = req.query(&plan.query);
    }
    match &plan.body {
        RequestBody::None => {}
        RequestBody::Form(form) => req = req.form(form),
        RequestBody::Json(json) => req = req.json(json),
    }
    req
}

/// Decode an already-read JSON response body with the standard status / (optional)
/// content-type checks. Pure over `(status, content_type, bytes)` so Tier-3 tests can
/// exercise success, HTTP-error, and bad-content-type paths against committed fixture
/// bytes — no server, no client.
pub(crate) fn decode_json<T: DeserializeOwned>(
    api: &str,
    status: StatusCode,
    content_type: Option<&HeaderValue>,
    bytes: &[u8],
    require_json_content_type: bool,
) -> Result<T, BioMcpError> {
    if !status.is_success() {
        let excerpt = body_excerpt(bytes);
        return Err(BioMcpError::Api {
            api: api.to_string(),
            message: format!("HTTP {status}: {excerpt}"),
        });
    }
    if require_json_content_type {
        ensure_json_content_type(api, content_type, bytes)?;
    }
    serde_json::from_slice(bytes).map_err(|source| BioMcpError::ApiJson {
        api: api.to_string(),
        source,
    })
}

fn parse_retry_after_header(headers: &HeaderMap) -> Option<Duration> {
    // Retry-After is interpreted as integer seconds when present.
    let raw = headers.get(RETRY_AFTER)?.to_str().ok()?.trim();
    if raw.is_empty() {
        return None;
    }
    let mut seconds = 0_u64;
    for byte in raw.bytes() {
        if !byte.is_ascii_digit() {
            return None;
        }
        seconds = seconds
            .saturating_mul(10)
            .saturating_add(u64::from(byte - b'0'));
    }
    Some(Duration::from_secs(seconds))
}

fn retry_sleep_duration(
    attempt: u32,
    retry_after_floor: Option<Duration>,
    sleep_budget_used: Duration,
) -> Option<Duration> {
    let backoff_ms = 100_u64.saturating_mul(2_u64.saturating_pow(attempt));
    let backoff = Duration::from_millis(backoff_ms);
    let capped_floor = retry_after_floor.map(|floor| floor.min(MAX_RETRY_AFTER_SLEEP));
    let target = match capped_floor {
        Some(floor) if floor > backoff => floor,
        _ => backoff,
    };
    let remaining = TOTAL_RETRY_SLEEP_BUDGET.checked_sub(sleep_budget_used)?;
    if remaining.is_zero() {
        return None;
    }
    Some(target.min(remaining))
}

#[derive(Clone, Copy, Debug, Default)]
struct RetrySleepState {
    attempt: u32,
    sleep_budget_used: Duration,
}

fn next_retry_sleep(
    state: &mut RetrySleepState,
    retry_after_floor: Option<Duration>,
) -> Option<Duration> {
    let duration = retry_sleep_duration(state.attempt, retry_after_floor, state.sleep_budget_used);
    state.attempt = state.attempt.saturating_add(1);
    if let Some(duration) = duration {
        state.sleep_budget_used = state.sleep_budget_used.saturating_add(duration);
    }
    duration
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
    Orcid,
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
            && let Some(retry_after_floor) = parse_retry_after_header(response.headers())
        {
            let duration = {
                if extensions.get::<RetrySleepState>().is_none() {
                    extensions.insert(RetrySleepState::default());
                }
                let state = extensions
                    .get_mut::<RetrySleepState>()
                    .expect("retry sleep state should exist");
                next_retry_sleep(state, Some(retry_after_floor))
            };
            if let Some(duration) = duration {
                tokio::time::sleep(duration).await;
            }
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
    build_http_client_with_config(kind, config, None)
}

fn build_http_client_with_config(
    kind: SharedHttpClientKind,
    config: crate::cache::ResolvedCacheConfig,
    provider_policy: Option<&provider_url_policy::ProviderUrlPolicy>,
) -> Result<ClientWithMiddleware, BioMcpError> {
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

    let mut base_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .user_agent(concat!("biomcp-cli/", env!("CARGO_PKG_VERSION")))
        .default_headers(default_headers);
    if let Some(policy) = provider_policy {
        base_client = base_client
            .no_proxy()
            .dns_resolver(policy.dns_resolver())
            .redirect(policy.redirect_policy());
    } else {
        base_client = base_client.redirect(match kind {
            SharedHttpClientKind::Orcid => rate_limit::orcid_redirect_policy(),
            _ => rate_limit::redirect_policy(),
        });
    }
    let base_client = base_client.build().map_err(BioMcpError::HttpClientInit)?;

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
        SharedHttpClientKind::Default | SharedHttpClientKind::Orcid => {
            builder.with(RetryAfterTooManyRequestsMiddleware)
        }
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

pub(crate) fn orcid_shared_client() -> Result<ClientWithMiddleware, BioMcpError> {
    if let Some(client) = ORCID_HTTP_CLIENT.get() {
        return Ok(client.clone());
    }

    let client = build_http_client(SharedHttpClientKind::Orcid)?;

    match ORCID_HTTP_CLIENT.set(client.clone()) {
        Ok(()) => Ok(client),
        Err(_) => ORCID_HTTP_CLIENT
            .get()
            .cloned()
            .ok_or_else(|| BioMcpError::Api {
                api: "http-client".into(),
                message: "ORCID shared HTTP client initialization race".into(),
            }),
    }
}

pub(crate) fn semantic_scholar_provider_client(
    policy: &provider_url_policy::ProviderUrlPolicy,
    authenticated: bool,
) -> Result<ClientWithMiddleware, BioMcpError> {
    let kind = if authenticated {
        SharedHttpClientKind::Default
    } else {
        SharedHttpClientKind::SemanticScholarSharedPool
    };
    let config = crate::cache::resolve_cache_config()?;
    build_http_client_with_config(kind, config, Some(policy))
}

pub(crate) fn provider_url_client(
    policy: &provider_url_policy::ProviderUrlPolicy,
) -> Result<ClientWithMiddleware, BioMcpError> {
    let config = crate::cache::resolve_cache_config()?;
    build_http_client_with_config(SharedHttpClientKind::Default, config, Some(policy))
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
        .redirect(rate_limit::redirect_policy())
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
    retry_send_with_sleep(api, max_retries, build_request, tokio::time::sleep).await
}

async fn retry_send_with_sleep<F, Fut, S, SleepFut>(
    api: &str,
    max_retries: u32,
    build_request: F,
    mut sleep_fn: S,
) -> Result<reqwest::Response, BioMcpError>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<reqwest::Response, reqwest::Error>>,
    S: FnMut(Duration) -> SleepFut,
    SleepFut: Future<Output = ()>,
{
    let total_attempts = max_retries.saturating_add(1);
    let mut last_http_err: Option<reqwest::Error> = None;
    let mut last_server_status: Option<reqwest::StatusCode> = None;
    let mut retry_sleep_state = RetrySleepState::default();

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

        if attempt + 1 < total_attempts
            && let Some(duration) = next_retry_sleep(&mut retry_sleep_state, retry_after_floor)
        {
            sleep_fn(duration).await;
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
            return Err(BioMcpError::BodyLimit {
                source_name: api.to_string(),
                max_bytes,
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
    use crate::cache::{CacheConfigOrigins, ConfigOrigin, DiskFreeThreshold, ResolvedCacheConfig};
    use crate::test_support::TempDirGuard;
    use std::path::Path;
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    };

    fn test_response(
        status: StatusCode,
        headers: &[(&'static str, &'static str)],
        body: &'static str,
    ) -> reqwest::Response {
        let mut builder = http::Response::builder().status(status);
        for (name, value) in headers {
            builder = builder.header(*name, *value);
        }
        builder
            .body(reqwest::Body::from(body))
            .expect("test response")
            .into()
    }

    fn test_cache_config(cache_root: impl Into<std::path::PathBuf>) -> ResolvedCacheConfig {
        ResolvedCacheConfig {
            cache_root: cache_root.into(),
            max_size: 10_000_000_000,
            min_disk_free: DiskFreeThreshold::Percent(10),
            max_age: Duration::from_secs(86_400),
            origins: CacheConfigOrigins {
                cache_root: ConfigOrigin::Default,
                max_size: ConfigOrigin::Default,
                min_disk_free: ConfigOrigin::Default,
                max_age: ConfigOrigin::Default,
            },
        }
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

    #[tokio::test]
    async fn read_limited_body_with_limit_rejects_oversized_body() {
        let err = read_limited_body_with_limit(
            test_response(StatusCode::OK, &[], "abcdef"),
            "test-api",
            5,
        )
        .await
        .expect_err("body over limit should fail");

        assert!(matches!(err, BioMcpError::BodyLimit { .. }));
        assert!(err.to_string().contains("test-api"));
        assert!(err.to_string().contains("Response body exceeded 5 bytes"));
    }

    #[tokio::test]
    async fn read_limited_body_with_limit_accepts_body_within_limit() {
        let body = read_limited_body_with_limit(
            test_response(StatusCode::OK, &[], "abcde"),
            "test-api",
            5,
        )
        .await
        .expect("body at limit should pass");

        assert_eq!(body, b"abcde");
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
    fn ticket_403_retry_after_normal_floor_is_honored() {
        assert_eq!(
            retry_sleep_duration(0, Some(Duration::from_secs(2)), Duration::ZERO),
            Some(Duration::from_secs(2))
        );
    }

    #[test]
    fn ticket_403_retry_after_malformed_values_fall_back_to_backoff() {
        let mut headers = HeaderMap::new();
        headers.insert(
            RETRY_AFTER,
            HeaderValue::from_static("Wed, 21 Oct 2015 07:28:00 GMT"),
        );
        assert_eq!(parse_retry_after_header(&headers), None);
        assert_eq!(
            retry_sleep_duration(1, parse_retry_after_header(&headers), Duration::ZERO),
            Some(Duration::from_millis(200))
        );
    }

    #[test]
    fn ticket_403_retry_after_extreme_values_are_capped() {
        let mut headers = HeaderMap::new();
        headers.insert(
            RETRY_AFTER,
            HeaderValue::from_static("999999999999999999999999999999999999"),
        );
        assert_eq!(
            retry_sleep_duration(0, parse_retry_after_header(&headers), Duration::ZERO),
            Some(MAX_RETRY_AFTER_SLEEP)
        );
    }

    #[tokio::test]
    async fn ticket_403_retry_send_uses_the_shared_retry_sleep_budget() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let sleeps = Arc::new(Mutex::new(Vec::new()));
        let err = retry_send_with_sleep(
            "test-api",
            4,
            {
                let attempts = attempts.clone();
                move || {
                    let attempts = attempts.clone();
                    async move {
                        attempts.fetch_add(1, Ordering::SeqCst);
                        Ok(test_response(
                            StatusCode::TOO_MANY_REQUESTS,
                            &[(RETRY_AFTER.as_str(), "999")],
                            "",
                        ))
                    }
                }
            },
            {
                let sleeps = sleeps.clone();
                move |duration| {
                    let sleeps = sleeps.clone();
                    async move {
                        sleeps.lock().expect("record sleep").push(duration);
                    }
                }
            },
        )
        .await
        .expect_err("retry_send should exhaust repeated 429 responses");

        assert!(
            err.to_string()
                .contains("HTTP 429 Too Many Requests after 5 attempts"),
            "unexpected retry_send error: {err}"
        );
        assert_eq!(attempts.load(Ordering::SeqCst), 5);
        assert_eq!(
            *sleeps.lock().expect("read sleeps"),
            vec![
                MAX_RETRY_AFTER_SLEEP,
                MAX_RETRY_AFTER_SLEEP,
                MAX_RETRY_AFTER_SLEEP
            ]
        );
    }

    #[tokio::test]
    async fn retry_send_with_sleep_retries_on_too_many_requests() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let resp = retry_send_with_sleep(
            "test-api",
            2,
            {
                let attempts = attempts.clone();
                move || {
                    let attempts = attempts.clone();
                    async move {
                        let attempt = attempts.fetch_add(1, Ordering::SeqCst);
                        let status = if attempt == 0 {
                            StatusCode::TOO_MANY_REQUESTS
                        } else {
                            StatusCode::OK
                        };
                        Ok(test_response(status, &[], "ok"))
                    }
                }
            },
            |_| async {},
        )
        .await
        .expect("retry_send should retry on 429");

        assert_eq!(resp.status(), reqwest::StatusCode::OK);
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
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
        let root = TempDirGuard::new("http-cache-migration");
        let override_root = root.path().join("override-root");
        let legacy_dir = override_root.join("http-cacache");
        std::fs::create_dir_all(&legacy_dir).expect("create legacy dir");
        std::fs::write(legacy_dir.join("sentinel.txt"), b"cached payload").expect("write sentinel");
        let config = test_cache_config(&override_root);

        let result = build_http_client_with_config(SharedHttpClientKind::Default, config, None);

        assert!(
            result.is_ok(),
            "client should initialize even with legacy cache migration"
        );
        assert!(override_root.join("http").is_dir());
        assert!(override_root.join("http").join("sentinel.txt").is_file());
        assert!(!override_root.join("http-cacache").exists());
    }
}
