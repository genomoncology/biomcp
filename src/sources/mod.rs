//! Source clients and shared HTTP utilities for upstream biomedical APIs.

use std::borrow::Cow;
use std::future::Future;
use std::sync::OnceLock;
use std::time::Duration;

use http_cache_reqwest::{
    CACacheManager, Cache, CacheMode, CacheOptions, HttpCache, HttpCacheOptions,
};
use reqwest::header::{CACHE_CONTROL, HeaderMap, HeaderValue, RETRY_AFTER};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, RequestBuilder};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use tracing::warn;

use crate::error::BioMcpError;

pub(crate) mod alphagenome;
pub(crate) mod cbioportal;
pub(crate) mod chembl;
pub(crate) mod civic;
pub(crate) mod clinicaltrials;
pub(crate) mod cpic;
pub(crate) mod enrichr;
pub(crate) mod europepmc;
pub(crate) mod gprofiler;
pub(crate) mod gwas;
pub(crate) mod hpo;
pub(crate) mod interpro;
pub(crate) mod monarch;
pub(crate) mod mychem;
pub(crate) mod mydisease;
pub(crate) mod mygene;
pub(crate) mod myvariant;
pub(crate) mod ncbi_idconv;
pub(crate) mod nci_cts;
pub(crate) mod oncokb;
pub(crate) mod openfda;
pub(crate) mod opentargets;
pub(crate) mod pharmgkb;
pub(crate) mod pmc_oa;
pub(crate) mod pubtator;
pub(crate) mod quickgo;
pub(crate) mod rate_limit;
pub(crate) mod reactome;
pub(crate) mod string;
pub(crate) mod uniprot;

const ERROR_BODY_MAX_BYTES: usize = 2048;
pub(crate) const DEFAULT_MAX_BODY_BYTES: usize = 8 * 1024 * 1024;
pub(crate) const BIOTHINGS_MAX_RESULT_WINDOW: usize = 10_000;

static HTTP_CLIENT: OnceLock<ClientWithMiddleware> = OnceLock::new();
static STREAMING_HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

tokio::task_local! {
    static NO_CACHE: bool;
}

pub(crate) async fn with_no_cache<R, F>(no_cache: bool, fut: F) -> R
where
    F: Future<Output = R>,
{
    NO_CACHE.scope(no_cache, fut).await
}

pub(crate) fn apply_cache_mode(req: RequestBuilder) -> RequestBuilder {
    match NO_CACHE.try_with(|v| *v) {
        Ok(true) => req.with_extension(CacheMode::NoStore),
        _ => req,
    }
}

pub(crate) fn apply_cache_mode_with_auth(
    req: RequestBuilder,
    authenticated: bool,
) -> RequestBuilder {
    if authenticated {
        return req.with_extension(CacheMode::NoStore);
    }
    apply_cache_mode(req)
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
/// - Cache: Disk-based HTTP cache in XDG cache directory
/// - Cache TTL: `Cache-Control: max-stale=86400` makes “no caching headers” responses usable for 24h
pub(crate) fn shared_client() -> Result<ClientWithMiddleware, BioMcpError> {
    if let Some(client) = HTTP_CLIENT.get() {
        return Ok(client.clone());
    }

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

    let cache_path = crate::utils::download::biomcp_cache_dir().join("http-cacache");
    std::fs::create_dir_all(&cache_path)?;

    let cache_options = HttpCacheOptions {
        cache_options: Some(CacheOptions {
            // Shared-cache semantics: do not store private/authenticated responses.
            shared: true,
            ..CacheOptions::default()
        }),
        ..HttpCacheOptions::default()
    };

    let client = ClientBuilder::new(base_client)
        .with(Cache(HttpCache {
            mode: CacheMode::Default,
            manager: CACacheManager { path: cache_path },
            options: cache_options,
        }))
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .with(rate_limit::RateLimitMiddleware::new())
        .build();

    match HTTP_CLIENT.set(client.clone()) {
        Ok(()) => Ok(client),
        Err(_) => HTTP_CLIENT.get().cloned().ok_or_else(|| BioMcpError::Api {
            api: "http-client".into(),
            message: "Shared HTTP client initialization race".into(),
        }),
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

pub(crate) fn ensure_json_content_type(
    api: &str,
    content_type: Option<&HeaderValue>,
    body: &[u8],
) -> Result<(), BioMcpError> {
    let Some(content_type) = content_type else {
        return Ok(());
    };

    let raw = match content_type.to_str() {
        Ok(v) => v.trim(),
        Err(_) => {
            warn!(
                source = api,
                "Response content-type header was not valid UTF-8; attempting JSON parse"
            );
            return Ok(());
        }
    };
    if raw.is_empty() {
        return Ok(());
    }

    let media_type = raw
        .split(';')
        .next()
        .map(str::trim)
        .unwrap_or_default()
        .to_ascii_lowercase();
    let is_html = matches!(media_type.as_str(), "text/html" | "application/xhtml+xml");
    if is_html {
        return Err(BioMcpError::Api {
            api: api.to_string(),
            message: format!(
                "Unexpected HTML response (content-type: {raw}): {}",
                body_excerpt(body)
            ),
        });
    }

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

pub(crate) async fn read_limited_body(
    mut resp: reqwest::Response,
    api: &str,
) -> Result<Vec<u8>, BioMcpError> {
    let mut body: Vec<u8> = Vec::new();

    while let Some(chunk) = resp.chunk().await? {
        let next_len = body.len().saturating_add(chunk.len());
        if next_len > DEFAULT_MAX_BODY_BYTES {
            return Err(BioMcpError::Api {
                api: api.to_string(),
                message: format!("Response body exceeded {DEFAULT_MAX_BODY_BYTES} bytes"),
            });
        }
        body.extend_from_slice(&chunk);
    }

    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

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
}
