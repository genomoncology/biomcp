use std::borrow::Cow;

use reqwest::{StatusCode, Url};
use tracing::debug;

use crate::error::{BioMcpError, SourceContext, SourceProvider};
use crate::sources::RequestBuilderSourceContextExt;
use crate::sources::provider_url_policy::{ProviderUrlPolicy, pmc_linked_asset_path};

const ARTICLE_FULLTEXT_API: &str = "article";
const PMC_ARTICLE_BASE: &str = "https://pmc.ncbi.nlm.nih.gov";
pub(crate) const PMC_ARTICLE_BASE_ENV: &str = "BIOMCP_PMC_HTML_BASE";
pub(crate) const LINKED_ASSET_BODY_LIMIT: usize = 8 * 1024 * 1024;
const LINKED_PRODUCTION_ORIGINS: &[&str] = &[
    "https://pmc.ncbi.nlm.nih.gov",
    "https://www.ncbi.nlm.nih.gov",
    "https://www.ebi.ac.uk",
    "https://europepmc.org",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PmcHtmlCacheState {
    Hit,
    Miss,
    Bypass,
}

#[derive(Debug)]
pub(crate) enum PmcHtmlFetchOutcome {
    Data { html: String, url: Url },
    Empty,
    Unusable(BioMcpError),
    Failed(BioMcpError),
}

#[derive(Debug)]
pub(crate) struct PmcHtmlFetch {
    pub outcome: PmcHtmlFetchOutcome,
    pub cache_state: PmcHtmlCacheState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PmcLinkedTarget {
    pub url: Url,
    pub canonical_identity: String,
    pub filename: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PmcLinkedFetch {
    Bytes {
        bytes: Vec<u8>,
        media_type: Option<String>,
    },
    HealthyAbsent,
    AccessOrLicenceDenied,
    SourceUnavailable,
}

#[derive(Clone)]
pub(crate) struct PmcArticleClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
    numeric_pmcid: String,
    policy: ProviderUrlPolicy,
}

impl PmcArticleClient {
    pub(crate) fn new(pmcid: &str) -> Result<Self, BioMcpError> {
        let numeric = normalized_numeric_pmcid(pmcid)
            .ok_or_else(|| source_error("invalid PMC article identity"))?;
        let base = pmc_article_base();
        let base_url = Url::parse(base.as_ref()).map_err(|_| source_error("invalid base URL"))?;
        validate_selected_linked_origin(&base_url)?;
        let policy = ProviderUrlPolicy::pmc_linked_article_asset(Some(&base_url), numeric)?;
        Ok(Self {
            client: crate::sources::provider_url_client(&policy)?,
            base,
            numeric_pmcid: numeric.to_string(),
            policy,
        })
    }

    pub(crate) fn linked_target(
        &self,
        raw_href: &str,
        relative_to_bin: bool,
    ) -> Result<PmcLinkedTarget, ()> {
        let raw = raw_href.trim();
        if raw.is_empty()
            || raw.contains(['\r', '\n', '\\'])
            || contains_encoded_separator_or_traversal(raw)
        {
            return Err(());
        }
        let url = match Url::parse(raw) {
            Ok(url) => url,
            Err(_) if !raw.contains("://") && relative_to_bin => {
                let base = Url::parse(self.base.as_ref()).map_err(|_| ())?;
                if raw.starts_with('/') {
                    base.join(raw).map_err(|_| ())?
                } else {
                    base.join(&format!(
                        "/articles/instance/{}/bin/{raw}",
                        self.numeric_pmcid
                    ))
                    .map_err(|_| ())?
                }
            }
            Err(_) if !raw.contains("://") => {
                let mut article = Url::parse(self.base.as_ref()).map_err(|_| ())?;
                article.set_path(&format!("/articles/PMC{}/", self.numeric_pmcid));
                article.join(raw).map_err(|_| ())?
            }
            Err(_) => return Err(()),
        };
        self.policy.validate_url(&url).map_err(|_| ())?;
        let asset_path = pmc_linked_asset_path(&url, &self.numeric_pmcid).ok_or(())?;
        let filename = asset_path
            .rsplit('/')
            .next()
            .filter(|value| !value.is_empty())
            .ok_or(())?;
        Ok(PmcLinkedTarget {
            canonical_identity: format!("pmc:PMC{}:{asset_path}", self.numeric_pmcid),
            url,
            filename: filename.to_string(),
        })
    }

    #[cfg(test)]
    pub(crate) async fn fetch(&self, target: &PmcLinkedTarget) -> PmcLinkedFetch {
        self.fetch_with_limit(target, LINKED_ASSET_BODY_LIMIT).await
    }

    async fn fetch_with_limit(
        &self,
        target: &PmcLinkedTarget,
        body_limit: usize,
    ) -> PmcLinkedFetch {
        // The client's DNS resolver and redirect policy use this same PMCID-scoped
        // ProviderUrlPolicy, so a redirect cannot broaden the accepted route.
        let response = match crate::sources::with_response_body_limit(
            crate::sources::apply_no_store(self.client.get(target.url.clone())),
            body_limit,
            "pmc-linked-asset",
        )
        .send_with_source_context(SourceContext::retry(SourceProvider::PMC_OPEN_ACCESS))
        .await
        {
            Ok(response) => response,
            Err(_) => return PmcLinkedFetch::SourceUnavailable,
        };
        if let Some(outcome) = classify_linked_status(response.status()) {
            return outcome;
        }
        let media_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.split(';').next().unwrap_or(value).trim().to_string())
            .filter(|value| !value.is_empty());
        match crate::sources::read_limited_source_body_with_limit(
            response,
            SourceContext::narrow(SourceProvider::PMC_OPEN_ACCESS),
            body_limit,
        )
        .await
        {
            Ok(bytes) => PmcLinkedFetch::Bytes {
                bytes: bytes.to_vec(),
                media_type,
            },
            Err(_) => PmcLinkedFetch::SourceUnavailable,
        }
    }

    #[cfg(test)]
    pub(crate) async fn fetch_first_available(
        &self,
        targets: &[PmcLinkedTarget],
    ) -> PmcLinkedFetch {
        self.fetch_first_available_with_limit(targets, LINKED_ASSET_BODY_LIMIT)
            .await
    }

    pub(crate) async fn fetch_first_available_with_limit(
        &self,
        targets: &[PmcLinkedTarget],
        body_limit: usize,
    ) -> PmcLinkedFetch {
        let mut strongest_failure = PmcLinkedFetch::HealthyAbsent;
        for target in targets {
            match self.fetch_with_limit(target, body_limit).await {
                bytes @ PmcLinkedFetch::Bytes { .. } => return bytes,
                PmcLinkedFetch::SourceUnavailable => {
                    strongest_failure = PmcLinkedFetch::SourceUnavailable;
                }
                PmcLinkedFetch::AccessOrLicenceDenied
                    if !matches!(strongest_failure, PmcLinkedFetch::SourceUnavailable) =>
                {
                    strongest_failure = PmcLinkedFetch::AccessOrLicenceDenied;
                }
                PmcLinkedFetch::HealthyAbsent | PmcLinkedFetch::AccessOrLicenceDenied => {}
            }
        }
        strongest_failure
    }
}

fn classify_linked_status(status: StatusCode) -> Option<PmcLinkedFetch> {
    match status {
        StatusCode::NOT_FOUND | StatusCode::NO_CONTENT => Some(PmcLinkedFetch::HealthyAbsent),
        StatusCode::UNAUTHORIZED
        | StatusCode::FORBIDDEN
        | StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS => Some(PmcLinkedFetch::AccessOrLicenceDenied),
        status if !status.is_success() => Some(PmcLinkedFetch::SourceUnavailable),
        _ => None,
    }
}

pub(crate) async fn fetch_html(pmcid: &str, requested_id: &str) -> PmcHtmlFetch {
    let failed = |err| PmcHtmlFetch {
        outcome: PmcHtmlFetchOutcome::Failed(err),
        cache_state: PmcHtmlCacheState::Bypass,
    };
    let url = match pmc_article_url(pmcid) {
        Ok(url) => url,
        Err(err) => return failed(err),
    };
    let client = match crate::sources::shared_client() {
        Ok(client) => client,
        Err(err) => return failed(err),
    };
    let cache_bypassed = crate::sources::cache_is_bypassed();
    let response = match crate::sources::apply_cache_mode(client.get(url.clone()))
        .send_with_source_context(SourceContext::retry(SourceProvider::PMC_OPEN_ACCESS))
        .await
    {
        Ok(response) => response,
        Err(err) => return failed(err),
    };
    let cache_state = html_cache_state(&response, cache_bypassed);
    if documented_fulltext_absence(response.status()) {
        return PmcHtmlFetch {
            outcome: PmcHtmlFetchOutcome::Empty,
            cache_state,
        };
    }
    if !response.status().is_success() {
        return PmcHtmlFetch {
            outcome: PmcHtmlFetchOutcome::Failed(
                BioMcpError::Api {
                    api: ARTICLE_FULLTEXT_API.to_string(),
                    message: format!("PMC HTML returned HTTP {}", response.status()),
                }
                .with_source_context(SourceContext::retry(SourceProvider::PMC_OPEN_ACCESS)),
            ),
            cache_state,
        };
    }
    if !html_content_type_is_supported(response.headers().get(reqwest::header::CONTENT_TYPE)) {
        return PmcHtmlFetch {
            outcome: PmcHtmlFetchOutcome::Unusable(
                BioMcpError::Api {
                    api: ARTICLE_FULLTEXT_API.to_string(),
                    message: "PMC HTML returned unsupported content".to_string(),
                }
                .with_source_context(SourceContext::retry(SourceProvider::PMC_OPEN_ACCESS)),
            ),
            cache_state,
        };
    }
    let bytes = match crate::sources::read_limited_source_body(
        response,
        SourceContext::narrow(SourceProvider::PMC_OPEN_ACCESS),
    )
    .await
    {
        Ok(bytes) => bytes,
        Err(err) => {
            debug!(?err, requested_id, pmcid, "PMC HTML body read failed");
            return PmcHtmlFetch {
                outcome: PmcHtmlFetchOutcome::Failed(err),
                cache_state,
            };
        }
    };
    match String::from_utf8(bytes.to_vec()) {
        Ok(html) => PmcHtmlFetch {
            outcome: PmcHtmlFetchOutcome::Data { html, url },
            cache_state,
        },
        Err(_) => PmcHtmlFetch {
            outcome: PmcHtmlFetchOutcome::Unusable(
                BioMcpError::Api {
                    api: ARTICLE_FULLTEXT_API.to_string(),
                    message: "PMC HTML response was not valid UTF-8".to_string(),
                }
                .with_source_context(SourceContext::retry(SourceProvider::PMC_OPEN_ACCESS)),
            ),
            cache_state,
        },
    }
}

pub(crate) async fn html(pmcid: &str) -> Result<Option<String>, BioMcpError> {
    match fetch_html(pmcid, pmcid).await.outcome {
        PmcHtmlFetchOutcome::Data { html, .. } => Ok(Some(html)),
        PmcHtmlFetchOutcome::Empty => Ok(None),
        PmcHtmlFetchOutcome::Unusable(err) | PmcHtmlFetchOutcome::Failed(err) => Err(err),
    }
}

fn pmc_article_base() -> Cow<'static, str> {
    crate::sources::env_base(PMC_ARTICLE_BASE, PMC_ARTICLE_BASE_ENV)
}

fn pmc_article_url(pmcid: &str) -> Result<Url, BioMcpError> {
    let base = pmc_article_base();
    let mut url = Url::parse(base.as_ref()).map_err(|err| BioMcpError::Api {
        api: ARTICLE_FULLTEXT_API.to_string(),
        message: format!("invalid PMC HTML base URL: {err}"),
    })?;
    {
        let mut segments = url.path_segments_mut().map_err(|_| BioMcpError::Api {
            api: ARTICLE_FULLTEXT_API.to_string(),
            message: "invalid PMC HTML base URL".to_string(),
        })?;
        segments.push("articles");
        segments.push(pmcid);
        segments.push("");
    }
    Ok(url)
}

fn html_cache_state(response: &reqwest::Response, cache_bypassed: bool) -> PmcHtmlCacheState {
    if cache_bypassed {
        return PmcHtmlCacheState::Bypass;
    }
    match response
        .headers()
        .get(http_cache::XCACHE)
        .and_then(|value| value.to_str().ok())
    {
        Some("HIT") => PmcHtmlCacheState::Hit,
        Some("MISS") => PmcHtmlCacheState::Miss,
        _ => PmcHtmlCacheState::Bypass,
    }
}

fn html_content_type_is_supported(content_type: Option<&reqwest::header::HeaderValue>) -> bool {
    let Some(content_type) = content_type.and_then(|value| value.to_str().ok()) else {
        return false;
    };
    let media_type = content_type
        .split(';')
        .next()
        .map(str::trim)
        .unwrap_or_default();
    media_type.eq_ignore_ascii_case("text/html")
        || media_type.eq_ignore_ascii_case("application/xhtml+xml")
}

fn documented_fulltext_absence(status: StatusCode) -> bool {
    matches!(status, StatusCode::NOT_FOUND | StatusCode::NO_CONTENT)
}

fn normalized_numeric_pmcid(pmcid: &str) -> Option<&str> {
    let value = pmcid.trim();
    let numeric = value
        .strip_prefix("PMC")
        .or_else(|| value.strip_prefix("pmc"))?;
    (!numeric.is_empty() && numeric.chars().all(|ch| ch.is_ascii_digit())).then_some(numeric)
}

fn contains_encoded_separator_or_traversal(raw: &str) -> bool {
    let mut value = raw.to_ascii_lowercase();
    loop {
        if value.contains("%2f")
            || value.contains("%5c")
            || value.contains("%2e")
            || value.contains("../")
            || value.contains("/..")
        {
            return true;
        }
        let Some(decoded) = decode_percent_once(&value) else {
            return true;
        };
        if decoded == value {
            break;
        }
        value = decoded;
    }
    false
}

fn decode_percent_once(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let high = hex_digit(*bytes.get(index + 1)?)?;
            let low = hex_digit(*bytes.get(index + 2)?)?;
            decoded.push((high << 4) | low);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok()
}

fn hex_digit(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn validate_selected_linked_origin(base: &Url) -> Result<(), BioMcpError> {
    let origin_matches = |candidate: &Url| {
        base.scheme() == candidate.scheme()
            && base.host_str() == candidate.host_str()
            && base.port_or_known_default() == candidate.port_or_known_default()
    };
    if LINKED_PRODUCTION_ORIGINS.iter().any(|raw| {
        Url::parse(raw)
            .ok()
            .is_some_and(|candidate| origin_matches(&candidate))
    }) {
        return Ok(());
    }
    let test_origin = std::env::var("BIOMCP_TEST_UNPACED_ORIGIN")
        .ok()
        .and_then(|raw| Url::parse(raw.trim()).ok());
    if test_origin.as_ref().is_some_and(|test| {
        origin_matches(test)
            && test.path() == "/"
            && test.query().is_none()
            && test.fragment().is_none()
            && test
                .host_str()
                .and_then(|host| host.parse::<std::net::IpAddr>().ok())
                .is_some_and(|address| address.is_loopback())
    }) {
        Ok(())
    } else {
        Err(source_error("selected origin is not allowlisted"))
    }
}

fn source_error(reason: &str) -> BioMcpError {
    BioMcpError::Api {
        api: "PMC linked article asset".to_string(),
        message: format!("PMC linked article asset source unavailable: {reason}"),
    }
    .with_source_context(SourceContext::retry(SourceProvider::PMC_OPEN_ACCESS))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    struct TestEnv {
        previous: Vec<(&'static str, Option<std::ffi::OsString>)>,
    }

    impl TestEnv {
        fn new() -> Self {
            Self {
                previous: Vec::new(),
            }
        }

        fn set(&mut self, key: &'static str, value: &str) {
            if !self.previous.iter().any(|(existing, _)| *existing == key) {
                self.previous.push((key, std::env::var_os(key)));
            }
            // SAFETY: these environment-mutating tests share one serial-test key.
            unsafe { std::env::set_var(key, value) };
        }
    }

    impl Drop for TestEnv {
        fn drop(&mut self) {
            for (key, previous) in self.previous.drain(..).rev() {
                // SAFETY: these environment-mutating tests share one serial-test key.
                unsafe {
                    if let Some(value) = previous {
                        std::env::set_var(key, value);
                    } else {
                        std::env::remove_var(key);
                    }
                }
            }
        }
    }

    #[test]
    fn exact_provider_routes_match_pmcid_and_preserve_signed_fetch_url() {
        let policy = ProviderUrlPolicy::pmc_linked_article_asset(None, "123457").unwrap();
        let client = PmcArticleClient {
            client: crate::sources::test_client().unwrap(),
            base: Cow::Borrowed(PMC_ARTICLE_BASE),
            numeric_pmcid: "123457".to_string(),
            policy,
        };
        for raw in [
            "https://pmc.ncbi.nlm.nih.gov/articles/instance/123457/bin/s1.xlsx?token=signed#part",
            "https://pmc.ncbi.nlm.nih.gov/articles/PMC123457/bin/s1.xlsx",
            "https://www.ncbi.nlm.nih.gov/pmc/articles/PMC123457/bin/s1.xlsx",
            "https://europepmc.org/articles/PMC123457/bin/s1.xlsx",
            "https://www.ebi.ac.uk/europepmc/articles/PMC123457/bin/s1.xlsx",
        ] {
            let target = client.linked_target(raw, false).expect(raw);
            assert_eq!(target.filename, "s1.xlsx");
            assert_eq!(target.canonical_identity, "pmc:PMC123457:s1.xlsx");
            if raw.contains("token=") {
                assert_eq!(target.url.query(), Some("token=signed"));
                assert_eq!(target.url.fragment(), Some("part"));
                assert!(!target.canonical_identity.contains("signed"));
            }
        }
    }

    #[test]
    fn linked_http_statuses_fold_to_closed_coverage_outcomes() {
        assert_eq!(
            classify_linked_status(StatusCode::NOT_FOUND),
            Some(PmcLinkedFetch::HealthyAbsent)
        );
        assert_eq!(
            classify_linked_status(StatusCode::NO_CONTENT),
            Some(PmcLinkedFetch::HealthyAbsent)
        );
        for status in [
            StatusCode::UNAUTHORIZED,
            StatusCode::FORBIDDEN,
            StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS,
        ] {
            assert_eq!(
                classify_linked_status(status),
                Some(PmcLinkedFetch::AccessOrLicenceDenied)
            );
        }
        for status in [StatusCode::TOO_MANY_REQUESTS, StatusCode::BAD_GATEWAY] {
            assert_eq!(
                classify_linked_status(status),
                Some(PmcLinkedFetch::SourceUnavailable)
            );
        }
        assert_eq!(classify_linked_status(StatusCode::OK), None);
    }

    #[tokio::test]
    #[serial_test::serial(article_resolver_env)]
    async fn recorded_pow_interstitial_is_not_returned_as_bytes() {
        let body = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/pmc_article/pmc3040717-supplementary-tables-pow.html"
        ));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = [0_u8; 2048];
            let _ = stream.read(&mut request).await.unwrap();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            stream.write_all(response.as_bytes()).await.unwrap();
            stream.write_all(body).await.unwrap();
        });
        let mut env = TestEnv::new();
        env.set("BIOMCP_TEST_UNPACED_ORIGIN", &base);
        env.set(PMC_ARTICLE_BASE_ENV, &base);
        let client = PmcArticleClient::new("PMC3040717").unwrap();
        let target = client
            .linked_target(
                "/articles/instance/3040717/bin/NIHMS265402-supplement-Supplementary_Tables.xls",
                false,
            )
            .unwrap();

        let outcome = client.fetch(&target).await;
        assert!(
            !matches!(outcome, PmcLinkedFetch::Bytes { .. }),
            "PMC proof-of-work HTML must not be published as supplement bytes"
        );
        assert!(
            format!("{outcome:?}").contains("ProofOfWork"),
            "the named PMC gate outcome must survive source classification"
        );
        server.await.unwrap();
    }

    #[tokio::test]
    #[serial_test::serial(article_resolver_env)]
    async fn declared_binary_html_is_not_returned_as_bytes() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = [0_u8; 2048];
            let _ = stream.read(&mut request).await.unwrap();
            let body = b"<!doctype html><title>Unexpected response</title>";
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            stream.write_all(response.as_bytes()).await.unwrap();
            stream.write_all(body).await.unwrap();
        });
        let mut env = TestEnv::new();
        env.set("BIOMCP_TEST_UNPACED_ORIGIN", &base);
        env.set(PMC_ARTICLE_BASE_ENV, &base);
        let client = PmcArticleClient::new("PMC3040717").unwrap();
        let target = client
            .linked_target("/articles/instance/3040717/bin/supplement.xls", false)
            .unwrap();

        assert!(
            !matches!(client.fetch(&target).await, PmcLinkedFetch::Bytes { .. }),
            "a declared binary delivered as HTML must be rejected even without a known PoW marker"
        );
        server.await.unwrap();
    }

    #[tokio::test]
    #[serial_test::serial(article_resolver_env)]
    async fn equal_identity_routes_continue_until_one_returns_bytes() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            for _ in 0..2 {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut request = [0_u8; 2048];
                let read = stream.read(&mut request).await.unwrap();
                let request = String::from_utf8_lossy(&request[..read]);
                let response = if request.contains("attempt=stale") {
                    "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                        .to_string()
                } else {
                    let body = "fallback bytes";
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    )
                };
                stream.write_all(response.as_bytes()).await.unwrap();
            }
        });
        let mut env = TestEnv::new();
        env.set("BIOMCP_TEST_UNPACED_ORIGIN", &base);
        env.set(PMC_ARTICLE_BASE_ENV, &base);
        let client = PmcArticleClient::new("PMC123457").unwrap();
        let stale = client
            .linked_target("/articles/instance/123457/bin/s1.xlsx?attempt=stale", false)
            .unwrap();
        let fresh = client
            .linked_target("/articles/instance/123457/bin/s1.xlsx?attempt=fresh", false)
            .unwrap();
        assert_eq!(stale.canonical_identity, fresh.canonical_identity);
        assert_eq!(
            client.fetch_first_available(&[stale, fresh]).await,
            PmcLinkedFetch::Bytes {
                bytes: b"fallback bytes".to_vec(),
                media_type: Some("application/octet-stream".to_string()),
            }
        );
        server.await.unwrap();
    }

    #[tokio::test]
    #[serial_test::serial(article_resolver_env)]
    async fn rejected_targets_are_precontact() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let mut env = TestEnv::new();
        env.set("BIOMCP_TEST_UNPACED_ORIGIN", &base);
        env.set(PMC_ARTICLE_BASE_ENV, &base);
        let client = PmcArticleClient::new("PMC123457").unwrap();
        for raw in [
            "/articles/instance/999/bin/no.xlsx",
            "/articles/instance/123457/not-bin/no.xlsx",
            "https://user:secret@pmc.ncbi.nlm.nih.gov/articles/instance/123457/bin/no.xlsx",
            "/articles/instance/123457/bin/%2e%2e%2fno.xlsx",
            "/articles/instance/123457/bin/%252e%252e/secret.pdf",
            "/articles/instance/123457/bin/%252fsecret.pdf",
        ] {
            assert!(client.linked_target(raw, false).is_err(), "accepted {raw}");
        }
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(100), listener.accept())
                .await
                .is_err(),
            "rejected target was contacted"
        );
    }

    #[tokio::test]
    #[serial_test::serial(article_resolver_env)]
    async fn unsafe_redirect_is_rejected_before_redirect_target_contact() {
        let unsafe_target = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let redirect = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", redirect.local_addr().unwrap());
        let target_url = format!(
            "http://{}/articles/instance/999/bin/stolen.xlsx",
            unsafe_target.local_addr().unwrap()
        );
        let server = tokio::spawn(async move {
            let (mut stream, _) = redirect.accept().await.unwrap();
            let mut request = [0_u8; 1024];
            let _ = stream.read(&mut request).await.unwrap();
            let response = format!(
                "HTTP/1.1 302 Found\r\nLocation: {target_url}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            );
            stream.write_all(response.as_bytes()).await.unwrap();
        });
        let mut env = TestEnv::new();
        env.set("BIOMCP_TEST_UNPACED_ORIGIN", &base);
        env.set(PMC_ARTICLE_BASE_ENV, &base);
        let client = PmcArticleClient::new("PMC123457").unwrap();
        let linked_target = client
            .linked_target("/articles/instance/123457/bin/s1.xlsx", false)
            .unwrap();
        assert_eq!(
            client.fetch(&linked_target).await,
            PmcLinkedFetch::SourceUnavailable
        );
        server.await.unwrap();
        assert!(
            tokio::time::timeout(
                std::time::Duration::from_millis(100),
                unsafe_target.accept()
            )
            .await
            .is_err(),
            "unsafe redirect target was contacted"
        );
    }
}
