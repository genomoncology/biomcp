//! Bounded ORCID v3.0 Public API client for exact author records: the
//! authenticated `GET /<id>/person` and `GET /<id>/works` reads. Credentials
//! are a pre-issued public-read bearer token in `ORCID_ACCESS_TOKEN`; no
//! OAuth, refresh, persistence, or formatting of the token anywhere.

use crate::error::{BioMcpError, SourceContext, SourceProvider};
use crate::sources::{RequestBuilderSourceContextExt, RequestPlan};
use serde::Deserialize;
use std::borrow::Cow;
use std::sync::OnceLock;
use std::time::Duration;
use tokio::sync::{Mutex, Semaphore};
use tokio::time::Instant;

const ORCID_PROD_BASE: &str = "https://pub.orcid.org/v3.0";
const ORCID_BASE_ENV: &str = "BIOMCP_ORCID_BASE";
const ORCID_TOKEN_ENV: &str = "ORCID_ACCESS_TOKEN";
const ORCID_TOKEN_MAX_BYTES: usize = 4096;
const ORCID_ACCEPT: &str = "application/vnd.orcid+json";
const ORCID_DOCS_URL: &str = "https://info.orcid.org/documentation/features/public-api/";
const ORCID_PERSON_BODY_LIMIT: usize = 512 * 1024;
const ORCID_WORKS_BODY_LIMIT: usize = 8 * 1024 * 1024;
/// One initial GET plus at most three retries.
const ORCID_MAX_ATTEMPTS: usize = 4;
/// Every physical attempt starts at least this long after the prior one.
const ORCID_ATTEMPT_PACING: Duration = Duration::from_secs(1);
/// Cap on the retry sleep whether or not `Retry-After` is present.
const ORCID_RETRY_SLEEP_CAP: Duration = Duration::from_secs(15);

fn context() -> SourceContext {
    SourceContext::retry(SourceProvider::ORCID)
}

/// True when the error is a body-limit failure, peeling the source-context
/// wrapper the send path applies.
fn is_body_limit(error: &BioMcpError) -> bool {
    match error {
        BioMcpError::BodyLimit { .. } => true,
        BioMcpError::WithSourceContext { source, .. } => is_body_limit(source),
        _ => false,
    }
}

fn sanitized(message: &'static str) -> BioMcpError {
    BioMcpError::Api {
        api: "orcid".into(),
        message: message.into(),
    }
    .with_source_context(context())
}

/// The bearer token after outer-ASCII-space trimming: present and valid, or
/// the exact frozen credential errors.
#[derive(Debug, Clone, PartialEq, Eq)]
enum OrcidToken {
    Missing,
    Invalid,
    Valid(String),
}

fn classify_token(raw: Option<String>) -> OrcidToken {
    let Some(raw) = raw else {
        return OrcidToken::Missing;
    };
    let trimmed = raw.trim_matches(|b: char| b == ' ');
    if trimmed.is_empty() {
        return OrcidToken::Missing;
    }
    let valid =
        trimmed.len() <= ORCID_TOKEN_MAX_BYTES && trimmed.bytes().all(|b| b.is_ascii_graphic());
    if valid {
        OrcidToken::Valid(trimmed.to_string())
    } else {
        OrcidToken::Invalid
    }
}

fn credential_error(token: &OrcidToken) -> Option<BioMcpError> {
    match token {
        OrcidToken::Valid(_) => None,
        OrcidToken::Missing => Some(BioMcpError::ApiKeyRequired {
            api: "ORCID".into(),
            env_var: ORCID_TOKEN_ENV.into(),
            docs_url: ORCID_DOCS_URL.into(),
        }),
        OrcidToken::Invalid => Some(BioMcpError::ApiCredentialInvalid {
            api: "ORCID".into(),
            env_var: ORCID_TOKEN_ENV.into(),
        }),
    }
}

/// Pure feature request plan: never stores the token, only its mode.
pub(crate) enum OrcidRequest {
    Person { id: String },
    Works { id: String },
}

impl OrcidRequest {
    pub(crate) fn plan(&self) -> RequestPlan {
        let (path, accept) = match self {
            Self::Person { id } => (format!("{id}/person"), ORCID_ACCEPT),
            Self::Works { id } => (format!("{id}/works"), ORCID_ACCEPT),
        };
        RequestPlan::get(path).header("Accept", accept)
    }
}

pub(crate) struct OrcidClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
    credential: OrcidToken,
}

/// ORCID middleware stack: URL policy admission, the shared per-process rate
/// limiter, and the streamed body limit — but not the shared retry
/// middleware, because this module's attempt loop owns every physical GET.
fn orcid_http_client(base: &str) -> Result<reqwest_middleware::ClientWithMiddleware, BioMcpError> {
    let url: reqwest::Url = base.parse().map_err(|_| BioMcpError::Api {
        api: "orcid".into(),
        message: "ORCID base URL is invalid".into(),
    })?;
    let policy = crate::sources::provider_url_policy::ProviderUrlPolicy::orcid_api(&url)?;
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::CACHE_CONTROL,
        reqwest::header::HeaderValue::from_static("no-store"),
    );
    let client = crate::sources::ordinary_url_policy::http_client_builder(Some(&policy))
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .user_agent(concat!("biomcp-cli/", env!("CARGO_PKG_VERSION")))
        .default_headers(headers)
        .build()
        .map_err(BioMcpError::HttpClientInit)?;
    let builder = reqwest_middleware::ClientBuilder::new(client);
    let builder = crate::sources::ordinary_url_policy::with_initial_policy(builder, Some(&policy));
    Ok(builder
        .with(crate::sources::rate_limit::RateLimitMiddleware::new())
        .with(crate::sources::ResponseBodyLimitMiddleware)
        .build())
}

static ORCID_PERMITS: OnceLock<Semaphore> = OnceLock::new();
static ORCID_LAST_ATTEMPT: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();

fn permits() -> &'static Semaphore {
    ORCID_PERMITS.get_or_init(|| Semaphore::new(2))
}

fn last_attempt() -> &'static Mutex<Option<Instant>> {
    ORCID_LAST_ATTEMPT.get_or_init(|| Mutex::new(None))
}

impl OrcidClient {
    pub fn new() -> Result<Self, BioMcpError> {
        let token_env = std::env::var(ORCID_TOKEN_ENV).ok();
        let credential = classify_token(token_env);
        let base = crate::sources::env_base(ORCID_PROD_BASE, ORCID_BASE_ENV);
        Ok(Self {
            client: orcid_http_client(&base)?,
            base,
            credential,
        })
    }

    /// The credential states for health reporting: same validator, zero I/O.
    pub(crate) fn credential_state() -> &'static str {
        match classify_token(std::env::var(ORCID_TOKEN_ENV).ok()) {
            OrcidToken::Missing => "excluded",
            OrcidToken::Invalid => "error",
            OrcidToken::Valid(_) => "configured",
        }
    }

    pub(crate) async fn person(
        &self,
        id: &str,
        deadline: Instant,
    ) -> Result<OrcidPersonResponse, BioMcpError> {
        let bytes = self
            .feature_bytes(
                OrcidRequest::Person { id: id.into() },
                ORCID_PERSON_BODY_LIMIT,
                deadline,
            )
            .await?;
        let response: OrcidPersonResponse = decode_strict(&bytes, "person")?;
        response.validate(id)
    }

    pub(crate) async fn works(
        &self,
        id: &str,
        deadline: Instant,
    ) -> Result<OrcidWorksResponse, BioMcpError> {
        let bytes = self
            .feature_bytes(
                OrcidRequest::Works { id: id.into() },
                ORCID_WORKS_BODY_LIMIT,
                deadline,
            )
            .await?;
        let response: OrcidWorksResponse = decode_strict(&bytes, "works")?;
        response.validate(id)
    }

    async fn feature_bytes(
        &self,
        request: OrcidRequest,
        body_limit: usize,
        deadline: Instant,
    ) -> Result<Vec<u8>, BioMcpError> {
        if let Some(error) = credential_error(&self.credential) {
            return Err(error);
        }
        let OrcidToken::Valid(token) = &self.credential else {
            unreachable!("credential_error returned None above");
        };
        let _permit = tokio::time::timeout_at(deadline, permits().acquire())
            .await
            .map_err(|_| sanitized("ORCID is unavailable; retry later"))?
            .map_err(|_| sanitized("ORCID is unavailable; retry later"))?;
        self.attempt_loop(&request, token, body_limit, deadline)
            .await
    }

    async fn attempt_loop(
        &self,
        request: &OrcidRequest,
        token: &str,
        body_limit: usize,
        deadline: Instant,
    ) -> Result<Vec<u8>, BioMcpError> {
        let mut retry_after = None;
        for attempt in 0..ORCID_MAX_ATTEMPTS {
            if attempt > 0 {
                let base = retry_after
                    .map(Duration::from_secs)
                    .unwrap_or(Duration::from_secs(1));
                tokio::time::sleep(base.min(ORCID_RETRY_SLEEP_CAP)).await;
            }
            if !self.pace_attempt(deadline).await {
                return Err(sanitized("ORCID is unavailable; retry later"));
            }
            match self.one_attempt(request, token, body_limit).await {
                Attempt::Done(bytes) => return Ok(bytes),
                Attempt::Retry { after } => retry_after = after,
                Attempt::Fail(error) => return Err(error),
            }
        }
        // Every physical attempt was a retryable class: the four-GET cap is
        // exhausted and the bounded outcome is the sanitized unavailability,
        // never a panic or a fifth request.
        Err(sanitized("ORCID is unavailable; retry later"))
    }

    /// Pacing gate: every physical attempt starts at least one second after
    /// the previous ORCID attempt start, process-wide. Returns false at the
    /// deadline.
    async fn pace_attempt(&self, deadline: Instant) -> bool {
        loop {
            let wait = {
                let mut guard = last_attempt().lock().await;
                match *guard {
                    None => {
                        *guard = Some(Instant::now());
                        return true;
                    }
                    Some(previous) => {
                        let elapsed = previous.elapsed();
                        if elapsed >= ORCID_ATTEMPT_PACING {
                            *guard = Some(Instant::now());
                            return true;
                        }
                        ORCID_ATTEMPT_PACING - elapsed
                    }
                }
            };
            if tokio::time::timeout_at(deadline, tokio::time::sleep(wait))
                .await
                .is_err()
            {
                return false;
            }
        }
    }

    async fn one_attempt(&self, request: &OrcidRequest, token: &str, body_limit: usize) -> Attempt {
        let plan = request.plan();
        let req = crate::sources::request_from_plan(&self.client, self.base.as_ref(), &plan)
            .header("Authorization", format!("Bearer {token}"))
            .header("Cache-Control", "no-store");
        // The policy middleware rejects a declared Content-Length above the
        // feature body cap before any bytes are read, so an oversized person
        // or works body fails the pre-read check instead of streaming first.
        let req = crate::sources::with_response_body_limit(req, body_limit, "ORCID");
        let resp = match req.send_with_source_context(context()).await {
            Ok(resp) => resp,
            Err(error) => {
                // A declared oversize body is a bounded hard failure, never a
                // retry that burns the four-GET budget streaming it again.
                if is_body_limit(&error) {
                    return Attempt::Fail(error);
                }
                return Attempt::Retry { after: None };
            }
        };
        let status = resp.status();
        if status.as_u16() == 401 || status.as_u16() == 403 {
            return Attempt::Fail(BioMcpError::ApiKeyRejected {
                api: "ORCID".into(),
                env_var: ORCID_TOKEN_ENV.into(),
                docs_url: ORCID_DOCS_URL.into(),
            });
        }
        if status.as_u16() == 404 {
            return Attempt::Fail(sanitized("HTTP 404"));
        }
        let retry_after_header = resp
            .headers()
            .get("Retry-After")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.trim().parse::<u64>().ok());
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = match crate::sources::read_limited_source_body_with_limit(
            resp,
            SourceContext::narrow(SourceProvider::ORCID),
            body_limit,
        )
        .await
        {
            Ok(bytes) => bytes,
            Err(error) => {
                // A streamed oversize body is the same bounded hard failure.
                if is_body_limit(&error) {
                    return Attempt::Fail(error);
                }
                return Attempt::Retry { after: None };
            }
        };
        if status.as_u16() == 429 || status.is_server_error() {
            return Attempt::Retry {
                after: retry_after_header,
            };
        }
        if !status.is_success() {
            return Attempt::Fail(sanitized("ORCID is unavailable; retry later"));
        }
        match ensure_orcid_content_type(content_type.as_ref()) {
            Ok(()) => Attempt::Done(bytes),
            Err(error) => Attempt::Fail(error),
        }
    }
}

enum Attempt {
    Done(Vec<u8>),
    Retry { after: Option<u64> },
    Fail(BioMcpError),
}

fn ensure_orcid_content_type(
    content_type: Option<&reqwest::header::HeaderValue>,
) -> Result<(), BioMcpError> {
    let raw = content_type
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .unwrap_or_default();
    let base = raw.split(';').next().unwrap_or_default().trim();
    if base.eq_ignore_ascii_case(ORCID_ACCEPT) || base.eq_ignore_ascii_case("application/json") {
        return Ok(());
    }
    Err(sanitized("ORCID is unavailable; retry later"))
}

fn decode_strict<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
    kind: &str,
) -> Result<T, BioMcpError> {
    serde_json::from_slice(bytes).map_err(|_| {
        sanitized(match kind {
            "person" => "person response is unavailable; retry later",
            _ => "works response is unavailable; retry later",
        })
    })
}

// ---- Wire structs and validation ----

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct OrcidPersonResponse {
    path: String,
    name: Option<OrcidName>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct OrcidName {
    visibility: Option<String>,
    #[serde(rename = "credit-name")]
    credit_name: Option<OrcidTextValue>,
    #[serde(rename = "given-names")]
    given_names: Option<OrcidTextValue>,
    #[serde(rename = "family-name")]
    family_name: Option<OrcidTextValue>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct OrcidTextValue {
    value: Option<String>,
}

impl OrcidPersonResponse {
    fn validate(self, id: &str) -> Result<Self, BioMcpError> {
        if self.path != format!("/{id}/person") {
            return Err(sanitized("person response is unavailable; retry later"));
        }
        Ok(self)
    }

    /// First nonblank public name by the frozen precedence, or a contract
    /// error when no usable public name exists.
    pub(crate) fn public_display_name(&self) -> Result<String, BioMcpError> {
        let name = self.name.as_ref().ok_or_else(contract)?;
        if name.visibility() != Some("PUBLIC") {
            return Err(contract());
        }
        fn field(value: &Option<OrcidTextValue>) -> Option<&str> {
            value
                .as_ref()
                .and_then(|v| v.value.as_deref())
                .and_then(|value| usable_text(value, 1024))
        }
        if let Some(credit) = field(&name.credit_name) {
            return Ok(credit.to_string());
        }
        let given = field(&name.given_names);
        let family = field(&name.family_name);
        if let (Some(given), Some(family)) = (given, family) {
            return Ok(format!("{given} {family}"));
        }
        given.or(family).map(str::to_string).ok_or_else(contract)
    }
}

impl OrcidName {
    fn visibility(&self) -> Option<&str> {
        self.visibility.as_deref()
    }
}

fn contract() -> BioMcpError {
    sanitized("person response is unavailable; retry later")
}

// ---- Works wire structs and validation ----

const ORCID_MAX_GROUPS: usize = 10_000;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct OrcidWorksResponse {
    path: String,
    // A present `group` array is part of the works contract; an absent key is
    // a malformed response and fails rather than reading as empty.
    group: Vec<OrcidWorkGroup>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct OrcidWorkGroup {
    #[serde(rename = "work-summary", default)]
    work_summary: Vec<OrcidWorkSummary>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct OrcidWorkSummary {
    visibility: Option<String>,
    #[serde(rename = "put-code")]
    put_code: Option<u64>,
    #[serde(rename = "display-index")]
    display_index: Option<String>,
    title: Option<OrcidWorkTitle>,
    #[serde(rename = "journal-title")]
    journal_title: Option<OrcidTextValue>,
    #[serde(rename = "publication-date")]
    publication_date: Option<OrcidPublicationDate>,
    #[serde(rename = "external-ids")]
    external_ids: Option<OrcidExternalIds>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct OrcidWorkTitle {
    title: Option<OrcidTextValue>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct OrcidPublicationDate {
    year: Option<OrcidTextValue>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct OrcidExternalIds {
    #[serde(rename = "external-id", default)]
    external_id: Option<Vec<OrcidExternalId>>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct OrcidExternalId {
    #[serde(rename = "external-id-type")]
    kind: Option<String>,
    #[serde(rename = "external-id-value")]
    value: Option<String>,
    #[serde(rename = "external-id-relationship")]
    relationship: Option<String>,
}

/// One selected PUBLIC representative with its validated public pieces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OrcidSelectedWork {
    pub put_code: u64,
    pub title: String,
    pub journal: Option<String>,
    pub year: Option<u32>,
    pub external_ids: Vec<(String, String)>,
}

impl OrcidWorksResponse {
    fn validate(self, id: &str) -> Result<Self, BioMcpError> {
        if self.path != format!("/{id}/works") || self.group.len() > ORCID_MAX_GROUPS {
            return Err(works_contract());
        }
        Ok(self)
    }

    /// Groups privacy-filtered to their selected PUBLIC representatives, in
    /// provider order, with every public summary strictly validated.
    pub(crate) fn selected_works(&self) -> Result<Vec<OrcidSelectedWork>, BioMcpError> {
        let mut selected = Vec::new();
        for group in &self.group {
            if group.work_summary.is_empty() || group.work_summary.len() > 64 {
                return Err(works_contract());
            }
            let mut public = Vec::new();
            for summary in &group.work_summary {
                if summary.visibility.as_deref() != Some("PUBLIC") {
                    continue;
                }
                public.push((summary, validated_summary(summary)?));
            }
            let Some((_, best)) = public
                .into_iter()
                .min_by(|a, b| representative_key(a).cmp(&representative_key(b)))
            else {
                continue;
            };
            selected.push(best);
        }
        Ok(selected)
    }
}

fn representative_key(
    entry: &(&OrcidWorkSummary, OrcidSelectedWork),
) -> (std::cmp::Reverse<u64>, u64) {
    let index = entry
        .0
        .display_index
        .as_deref()
        .and_then(|raw| raw.parse::<u64>().ok())
        .unwrap_or(0);
    (std::cmp::Reverse(index), entry.1.put_code)
}

fn validated_summary(summary: &OrcidWorkSummary) -> Result<OrcidSelectedWork, BioMcpError> {
    let put_code = summary
        .put_code
        .filter(|code| *code != 0)
        .ok_or_else(works_contract)?;
    let display_index = summary
        .display_index
        .as_deref()
        .ok_or_else(works_contract)?;
    if !canonical_display_index(display_index) {
        return Err(works_contract());
    }
    let title = summary
        .title
        .as_ref()
        .and_then(|title| title.title.as_ref())
        .and_then(|value| value.value.as_deref())
        .ok_or_else(works_contract)?;
    let title = usable_text(title, 4096).ok_or_else(works_contract)?;
    let journal = match summary
        .journal_title
        .as_ref()
        .and_then(|value| value.value.as_deref())
    {
        None => None,
        Some(value) => Some(
            usable_text(value, 1024)
                .ok_or_else(works_contract)?
                .to_string(),
        ),
    };
    let year = match summary
        .publication_date
        .as_ref()
        .and_then(|date| date.year.as_ref())
        .and_then(|year| year.value.as_deref())
    {
        None => None,
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.len() != 4 || !trimmed.bytes().all(|b| b.is_ascii_digit()) {
                return Err(works_contract());
            }
            let year = trimmed.parse::<u32>().map_err(|_| works_contract())?;
            if !(1000..=9999).contains(&year) {
                return Err(works_contract());
            }
            Some(year)
        }
    };
    let mut external_ids = Vec::new();
    if let Some(ids) = summary.external_ids.as_ref() {
        let entries = ids.external_id.as_deref().unwrap_or(&[]);
        if entries.len() > 64 {
            return Err(works_contract());
        }
        for entry in entries {
            let (Some(kind), Some(value), Some(relationship)) = (
                entry.kind.as_deref(),
                entry.value.as_deref(),
                entry.relationship.as_deref(),
            ) else {
                return Err(works_contract());
            };
            if relationship != "SELF" {
                continue;
            }
            let kind = kind.to_ascii_lowercase();
            if kind.is_empty()
                || kind.len() > 32
                || !kind.bytes().all(|b| {
                    b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'.' | b'_' | b'-')
                })
            {
                continue;
            }
            if let Some(value) = usable_text(value, 512) {
                external_ids.push((kind, value.to_string()));
            }
        }
    }
    Ok(OrcidSelectedWork {
        put_code,
        title: title.to_string(),
        journal,
        year,
        external_ids,
    })
}

fn canonical_display_index(raw: &str) -> bool {
    let bytes = raw.as_bytes();
    match bytes.split_first() {
        Some((first, rest)) => {
            (first.is_ascii_digit() && *first != b'0' || *first == b'0' && rest.is_empty())
                && rest.iter().all(|b| b.is_ascii_digit())
        }
        None => false,
    }
}

fn usable_text(value: &str, max_bytes: usize) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > max_bytes || trimmed.chars().any(|c| c.is_control()) {
        return None;
    }
    Some(trimmed)
}

fn works_contract() -> BioMcpError {
    sanitized("works response is unavailable; retry later")
}

#[cfg(test)]
mod tests;
