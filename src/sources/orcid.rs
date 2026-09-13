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
        let resp = match req.send_with_source_context(context()).await {
            Ok(resp) => resp,
            Err(_) => return Attempt::Retry { after: None },
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
            Err(_) => return Attempt::Retry { after: None },
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
    #[serde(default)]
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
mod tests {
    use super::*;

    #[test]
    fn request_plans_carry_exact_paths_headers_and_bearer_mode_only() {
        let person = OrcidRequest::Person {
            id: "0000-0002-1825-0097".into(),
        };
        let works = OrcidRequest::Works {
            id: "0000-0002-1825-0097".into(),
        };
        let plan = person.plan();
        assert_eq!(plan.path, "0000-0002-1825-0097/person");
        assert_eq!(plan.header_value("Accept"), Some(ORCID_ACCEPT));
        assert!(
            !plan
                .headers
                .iter()
                .any(|(k, _)| k.eq_ignore_ascii_case("authorization"))
        );
        let plan = works.plan();
        assert_eq!(plan.path, "0000-0002-1825-0097/works");
    }

    #[test]
    fn token_classification_matches_the_frozen_three_states() {
        assert_eq!(classify_token(None), OrcidToken::Missing);
        assert_eq!(classify_token(Some("   ".into())), OrcidToken::Missing);
        assert_eq!(
            classify_token(Some("secret-token".into())),
            OrcidToken::Valid("secret-token".into())
        );
        assert_eq!(
            classify_token(Some(" bad token".into())),
            OrcidToken::Invalid
        );
        assert_eq!(
            classify_token(Some("tab\ttoken".into())),
            OrcidToken::Invalid
        );
        assert_eq!(classify_token(Some("é".into())), OrcidToken::Invalid);
        assert_eq!(classify_token(Some("x".repeat(4097))), OrcidToken::Invalid);
        assert_eq!(
            classify_token(Some("x".repeat(4096))),
            OrcidToken::Valid("x".repeat(4096))
        );
    }

    #[test]
    fn person_path_and_public_name_precedence() {
        let response: OrcidPersonResponse = serde_json::from_str(
            r#"{"path":"/0000-0002-1825-0097/person","name":{"visibility":"PUBLIC","given-names":{"value":" Josiah "},"family-name":{"value":"Carberry"}}}"#,
        )
        .unwrap();
        let response = response.validate("0000-0002-1825-0097").unwrap();
        assert_eq!(response.public_display_name().unwrap(), "Josiah Carberry");
        let wrong: OrcidPersonResponse =
            serde_json::from_str(r#"{"path":"/other/person","name":null}"#).unwrap();
        assert!(wrong.validate("0000-0002-1825-0097").is_err());
    }

    #[test]
    fn works_validation_selects_representatives_and_rejects_bad_shapes() {
        let body = r#"{"path":"/i/works","group":[{"work-summary":[
            {"visibility":"PUBLIC","put-code":7,"display-index":"2","title":{"title":{"value":"Second"}}},
            {"visibility":"PUBLIC","put-code":9,"display-index":"10","title":{"title":{"value":"First"}}},
            {"visibility":"PRIVATE","put-code":99,"display-index":"99","title":{"title":{"value":"private"}}}
        ]}]}"#;
        let response: OrcidWorksResponse = serde_json::from_str(body).unwrap();
        let selected = response.validate("i").unwrap().selected_works().unwrap();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].put_code, 9, "greatest display-index wins");
        assert_eq!(selected[0].title, "First");

        let zero_code = r#"{"path":"/i/works","group":[{"work-summary":[
            {"visibility":"PUBLIC","put-code":0,"display-index":"1","title":{"title":{"value":"x"}}}
        ]}]}"#;
        let response: OrcidWorksResponse = serde_json::from_str(zero_code).unwrap();
        assert!(response.validate("i").unwrap().selected_works().is_err());

        let leading_zero_index = r#"{"path":"/i/works","group":[{"work-summary":[
            {"visibility":"PUBLIC","put-code":1,"display-index":"01","title":{"title":{"value":"x"}}}
        ]}]}"#;
        let response: OrcidWorksResponse = serde_json::from_str(leading_zero_index).unwrap();
        assert!(response.validate("i").unwrap().selected_works().is_err());
    }

    // ---- Ticket 1142 closing pass: loopback transport and works bounds ----

    mod closing {
        use super::super::*;
        use std::sync::Arc;
        use std::sync::Mutex as StdMutex;

        const VALID_ID: &str = "0000-0002-1825-0097";

        fn person_body() -> String {
            format!(
                r#"{{"path":"/{VALID_ID}/person","name":{{"visibility":"PUBLIC","given-names":{{"value":"Josiah"}},"family-name":{{"value":"Carberry"}}}}}}"#
            )
        }

        /// Scripted loopback: each accepted connection consumes the next
        /// `(status, retry_after, body)` response; every request line is
        /// logged.
        async fn spawn_scripted_orcid(
            responses: Vec<(u16, Option<&'static str>, String)>,
        ) -> (String, Arc<StdMutex<Vec<String>>>) {
            use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
            let requests = Arc::new(StdMutex::new(Vec::new()));
            let logged = requests.clone();
            let counter = Arc::new(StdMutex::new(0usize));
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                .await
                .expect("bind orcid fixture");
            let address = listener.local_addr().expect("orcid fixture address");
            tokio::spawn(async move {
                while let Ok((mut stream, _)) = listener.accept().await {
                    let logged = logged.clone();
                    let responses = responses.clone();
                    let counter = counter.clone();
                    tokio::spawn(async move {
                        let mut request = vec![0_u8; 16 * 1024];
                        let length = stream.read(&mut request).await.unwrap_or(0);
                        let request = String::from_utf8_lossy(&request[..length]);
                        if let Some(target) = request.split_whitespace().nth(1) {
                            logged.lock().unwrap().push(target.to_string());
                        }
                        let index = {
                            let mut guard = counter.lock().unwrap();
                            let index = *guard;
                            *guard += 1;
                            index
                        };
                        let (status, retry_after, body) = responses
                            .get(index)
                            .cloned()
                            .unwrap_or((500, None, String::new()));
                        let retry = retry_after
                            .map(|value| format!("Retry-After: {value}\r\n"))
                            .unwrap_or_default();
                        let response = format!(
                            "HTTP/1.1 {status} X\r\nContent-Type: application/vnd.orcid+json\r\n{retry}Content-Length: {}\r\nConnection: close\r\n\r\n{}",
                            body.len(),
                            body
                        );
                        let _ = stream.write_all(response.as_bytes()).await;
                    });
                }
            });
            (format!("http://{address}"), requests)
        }

        struct OrcidEnv {
            previous: Vec<(&'static str, Option<std::ffi::OsString>)>,
        }
        impl OrcidEnv {
            fn new(base: &str) -> Self {
                let mut env = Self {
                    previous: Vec::new(),
                };
                for (key, value) in [
                    (ORCID_BASE_ENV, base.to_string()),
                    (ORCID_TOKEN_ENV, "fixture-public-read-token".to_string()),
                    ("BIOMCP_TEST_UNPACED_ORIGIN", base.to_string()),
                ] {
                    env.previous.push((key, std::env::var_os(key)));
                    // SAFETY: serial-guarded test environment mutation.
                    unsafe { std::env::set_var(key, value) };
                }
                env
            }
        }
        impl Drop for OrcidEnv {
            fn drop(&mut self) {
                for (key, value) in self.previous.drain(..) {
                    // SAFETY: restoring the serial-guarded environment.
                    unsafe {
                        match value {
                            Some(value) => std::env::set_var(key, value),
                            None => std::env::remove_var(key),
                        }
                    }
                }
            }
        }

        fn deadline(seconds: u64) -> Instant {
            Instant::now() + Duration::from_secs(seconds)
        }

        #[tokio::test]
        #[serial_test::serial(source_env)]
        async fn transient_429_retries_once_under_the_pacing_gap_then_succeeds() {
            let (base, requests) = spawn_scripted_orcid(vec![
                (429, Some("1"), String::new()),
                (200, None, person_body()),
            ])
            .await;
            let _env = OrcidEnv::new(&base);
            let started = std::time::Instant::now();
            let person = OrcidClient::new()
                .expect("client")
                .person(VALID_ID, deadline(30))
                .await
                .expect("second attempt succeeds");
            assert_eq!(
                person.public_display_name().expect("public name"),
                "Josiah Carberry"
            );
            let elapsed = started.elapsed();
            assert!(
                elapsed >= Duration::from_secs(1),
                "paced retry took {elapsed:?}"
            );
            let logged = requests.lock().unwrap().clone();
            assert_eq!(logged, vec!["/0000-0002-1825-0097/person".to_string(); 2]);
        }

        #[tokio::test]
        #[serial_test::serial(source_env)]
        async fn persistent_server_errors_stop_at_four_physical_gets_with_a_sanitized_error() {
            let (base, requests) = spawn_scripted_orcid(vec![(500, None, String::new()); 4]).await;
            let _env = OrcidEnv::new(&base);
            let error = OrcidClient::new()
                .expect("client")
                .person(VALID_ID, deadline(30))
                .await
                .expect_err("cap exhausted");
            assert_eq!(error.code(), "api");
            let logged = requests.lock().unwrap().clone();
            assert_eq!(logged.len(), 4, "exactly four physical GETs");
        }

        #[tokio::test]
        #[serial_test::serial(source_env)]
        async fn rejected_credentials_fail_immediately_without_retry() {
            let (base, requests) = spawn_scripted_orcid(vec![(401, None, String::new())]).await;
            let _env = OrcidEnv::new(&base);
            let error = OrcidClient::new()
                .expect("client")
                .person(VALID_ID, deadline(30))
                .await
                .expect_err("401");
            assert!(matches!(error, BioMcpError::ApiKeyRejected { .. }));
            assert_eq!(requests.lock().unwrap().len(), 1);
        }

        #[tokio::test]
        #[serial_test::serial(source_env)]
        async fn not_found_is_a_sanitized_single_get() {
            let (base, requests) = spawn_scripted_orcid(vec![(404, None, String::new())]).await;
            let _env = OrcidEnv::new(&base);
            let error = OrcidClient::new()
                .expect("client")
                .person(VALID_ID, deadline(30))
                .await
                .expect_err("404");
            assert_eq!(error.code(), "api");
            assert_eq!(requests.lock().unwrap().len(), 1);
        }

        #[tokio::test]
        #[serial_test::serial(source_env)]
        async fn client_errors_are_never_retried() {
            let (base, requests) = spawn_scripted_orcid(vec![(400, None, String::new())]).await;
            let _env = OrcidEnv::new(&base);
            let error = OrcidClient::new()
                .expect("client")
                .person(VALID_ID, deadline(30))
                .await
                .expect_err("400");
            assert_eq!(error.code(), "api");
            assert_eq!(requests.lock().unwrap().len(), 1);
        }

        #[tokio::test]
        #[serial_test::serial(source_env)]
        async fn an_expired_deadline_admits_no_second_attempt_and_paces_the_first() {
            let (base, requests) = spawn_scripted_orcid(vec![(200, None, person_body()); 2]).await;
            let _env = OrcidEnv::new(&base);
            let client = OrcidClient::new().expect("client");
            client
                .person(VALID_ID, deadline(30))
                .await
                .expect("first call succeeds");
            // The second logical call cannot admit a physical attempt before
            // its absolute deadline: the one-second pacing gap is longer than
            // the remaining budget, so the bounded error returns with no new
            // GET.
            let error = client
                .person(VALID_ID, Instant::now() + Duration::from_millis(100))
                .await
                .expect_err("deadline");
            assert_eq!(error.code(), "api");
            assert_eq!(requests.lock().unwrap().len(), 1);
        }

        fn summary(
            put_code: u64,
            display_index: &str,
            title: &str,
            pmid: Option<&str>,
            visibility: &str,
        ) -> serde_json::Value {
            let mut summary = serde_json::json!({
                "visibility": visibility,
                "put-code": put_code,
                "display-index": display_index,
                "title": {"title": {"value": title}},
            });
            if let Some(pmid) = pmid {
                summary["external-ids"] = serde_json::json!({
                    "external-id": [
                        {"external-id-type": "pmid", "external-id-value": pmid, "external-id-relationship": "SELF"}
                    ]
                });
            }
            summary
        }

        fn works_body(groups: &[serde_json::Value]) -> String {
            serde_json::json!({"path": "/i/works", "group": groups}).to_string()
        }

        fn selected(body: &str) -> Result<Vec<OrcidSelectedWork>, BioMcpError> {
            let response: OrcidWorksResponse = serde_json::from_str(body).unwrap();
            response.validate("i")?.selected_works()
        }

        #[test]
        fn a_group_without_summaries_is_a_contract_error() {
            let body = works_body(&[serde_json::json!({"work-summary": []})]);
            assert!(
                selected(&body).is_err(),
                "1-64 summaries per group is the bound"
            );
        }

        #[test]
        fn one_and_sixty_four_summaries_are_the_group_bound_edges() {
            for count in [1_usize, 64] {
                let summaries: Vec<_> = (0..count)
                    .map(|index| summary(1000 + index as u64, "1", "Title", None, "PUBLIC"))
                    .collect();
                let body = works_body(&[serde_json::json!({"work-summary": summaries})]);
                let works = selected(&body).expect("inside the bound");
                assert_eq!(works.len(), 1);
            }
            let summaries: Vec<_> = (0..65)
                .map(|index| summary(index as u64 + 1, "1", "Title", None, "PUBLIC"))
                .collect();
            let body = works_body(&[serde_json::json!({"work-summary": summaries})]);
            assert!(
                selected(&body).is_err(),
                "65 summaries per group is a contract error"
            );
        }

        #[test]
        fn ten_thousand_groups_is_the_hard_bound() {
            let group =
                serde_json::json!({"work-summary": [summary(7, "1", "Only", None, "PUBLIC")]});
            for count in [10_000_usize, 10_001] {
                let body = works_body(&vec![group.clone(); count]);
                let result = selected(&body);
                if count == 10_000 {
                    result.expect("at the bound");
                } else {
                    result.expect_err("past the bound");
                }
            }
        }

        #[test]
        fn representative_ties_prefer_the_lowest_put_code_then_original_order() {
            let body = works_body(&[serde_json::json!({"work-summary": [
                summary(5, "2", "Higher code", None, "PUBLIC"),
                summary(3, "2", "Lower code wins", None, "PUBLIC")
            ]})]);
            let works = selected(&body).expect("valid");
            assert_eq!(works[0].title, "Lower code wins");

            let body = works_body(&[serde_json::json!({"work-summary": [
                summary(7, "2", "First in order wins ties", None, "PUBLIC"),
                summary(7, "2", "Second", None, "PUBLIC")
            ]})]);
            let works = selected(&body).expect("valid");
            assert_eq!(works[0].title, "First in order wins ties");
        }

        #[test]
        fn a_present_year_outside_1000_9999_is_a_contract_error() {
            let mut bad = summary(7, "1", "Bad year", None, "PUBLIC");
            bad["publication-date"] = serde_json::json!({"year": {"value": "0999"}});
            let body = works_body(&[serde_json::json!({"work-summary": [bad]})]);
            assert!(selected(&body).is_err(), "0999 is outside 1000-9999");
            let mut good = summary(8, "1", "Good year", None, "PUBLIC");
            good["publication-date"] = serde_json::json!({"year": {"value": "2024"}});
            let body = works_body(&[serde_json::json!({"work-summary": [good]})]);
            assert_eq!(selected(&body).expect("valid")[0].year, Some(2024));
        }

        #[test]
        fn identifiers_from_excluded_locations_never_reach_the_selected_work() {
            let hostile = serde_json::json!({
                "external-ids": {"external-id": [
                    {"external-id-type": "pmid", "external-id-value": "9991", "external-id-relationship": "SELF"}
                ]},
                "work-summary": [
                    {"visibility": "PRIVATE", "put-code": 91, "display-index": "50",
                     "title": {"title": {"value": "Private summary"}},
                     "external-ids": {"external-id": [
                        {"external-id-type": "pmid", "external-id-value": "9992", "external-id-relationship": "SELF"}
                     ]}},
                    {"visibility": "PUBLIC", "put-code": 92, "display-index": "1",
                     "title": {"title": {"value": "Public but not selected"}},
                     "external-ids": {"external-id": [
                        {"external-id-type": "pmid", "external-id-value": "9993", "external-id-relationship": "SELF"}
                     ]}},
                    summary(42, "2", "Selected representative", Some("42"), "PUBLIC")
                ]
            });
            let body = works_body(&[hostile]);
            let works = selected(&body).expect("valid");
            assert_eq!(works.len(), 1);
            assert_eq!(works[0].put_code, 42);
            let flattened: Vec<(String, String)> = works[0].external_ids.clone();
            assert_eq!(flattened, vec![("pmid".into(), "42".into())]);
            for (_, value) in flattened {
                assert!(
                    !value.contains("999"),
                    "excluded identifier leaked: {value}"
                );
            }
        }
    }

    #[test]
    fn content_type_gate_accepts_only_the_two_orcid_media_types() {
        let header = |raw: &str| reqwest::header::HeaderValue::from_str(raw).unwrap();
        assert!(ensure_orcid_content_type(Some(&header("application/vnd.orcid+json"))).is_ok());
        assert!(
            ensure_orcid_content_type(Some(&header("application/json; charset=utf-8"))).is_ok()
        );
        assert!(ensure_orcid_content_type(Some(&header("APPLICATION/JSON"))).is_ok());
        assert!(ensure_orcid_content_type(Some(&header("text/html"))).is_err());
        assert!(ensure_orcid_content_type(None).is_err());
    }
}
