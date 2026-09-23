//! Read-only client for the one FHIR server the operator names in
//! `BIOMCP_FHIR_BASE`.
//!
//! Nothing here logs, and no error carries a URL, a body, or a patient ID.
//! Every FHIR GET goes through [`FhirClient::get_json`], which sends it with
//! no-store and strips the URL from any transport error. Redirects and next
//! links stay on the origin and base path of the configured server.

use std::collections::HashSet;
use std::time::Duration;

use reqwest::Url;
use reqwest::header::{ACCEPT, CACHE_CONTROL};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, RequestBuilder};
use serde_json::Value;

use crate::error::BioMcpError;

/// The operator variable that names the one FHIR server.
pub(crate) const FHIR_BASE_ENV: &str = "BIOMCP_FHIR_BASE";
/// The source label shown in section outcomes and provenance.
pub(crate) const FHIR_SOURCE: &str = "FHIR";
/// How many conditions one page asks for.
pub(crate) const CONDITION_PAGE_SIZE: usize = 100;
/// The most pages one condition walk reads.
pub(crate) const MAX_PAGES: usize = 20;
/// The longest patient ID accepted.
pub(crate) const MAX_PATIENT_ID_CHARS: usize = 64;

const MAX_REDIRECTS: usize = 10;
const ACCEPT_FHIR_JSON: &str = "application/fhir+json, application/json";

/// What can go wrong between BioMCP and the FHIR server.
///
/// Every message is fixed text. No variant carries a URL, a body, or an ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum FhirError {
    #[error("patient commands need a FHIR server: set BIOMCP_FHIR_BASE to its base URL and retry")]
    NotConfigured,
    #[error(
        "BIOMCP_FHIR_BASE must be an http or https URL with no credentials, query, or fragment"
    )]
    BadBase,
    #[error("the FHIR request timed out")]
    Timeout,
    #[error("the FHIR server could not be reached")]
    Connect,
    #[error("the FHIR server redirected off the configured base, so the request stopped")]
    Redirect,
    #[error("the FHIR request did not complete")]
    Transport,
    #[error("the FHIR server has no record with that patient ID")]
    NotFound,
    #[error("the FHIR server answered HTTP {0}")]
    Status(u16),
    #[error("the FHIR server sent a response that is not the expected FHIR JSON")]
    Decode,
}

/// A patient ID checked against `[A-Za-z0-9\-.]{1,64}`, the FHIR id rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PatientId(String);

impl PatientId {
    /// Refuses anything outside the FHIR id rule before any request.
    /// An ID of only dots is refused too. The URL library drops `.` and `..`
    /// path segments, and the read would become a search of every patient.
    /// The error never repeats the input.
    pub(crate) fn parse(raw: &str) -> Result<Self, BioMcpError> {
        let valid = !raw.is_empty()
            && raw.chars().count() <= MAX_PATIENT_ID_CHARS
            && raw
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '.')
            && raw.chars().any(|ch| ch != '.');
        if !valid {
            return Err(BioMcpError::InvalidArgument(
                "a patient ID must be 1-64 characters of letters, digits, hyphens, or periods, and not only periods"
                    .into(),
            ));
        }
        Ok(Self(raw.to_string()))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

/// The configured base: one origin and one path prefix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FhirBase {
    url: Url,
}

impl FhirBase {
    pub(crate) fn parse(raw: &str) -> Result<Self, FhirError> {
        let mut url = Url::parse(raw.trim()).map_err(|_| FhirError::BadBase)?;
        let usable = matches!(url.scheme(), "http" | "https")
            && url.host_str().is_some()
            && url.username().is_empty()
            && url.password().is_none()
            && url.query().is_none()
            && url.fragment().is_none();
        if !usable {
            return Err(FhirError::BadBase);
        }
        let path = url.path().trim_end_matches('/').to_string();
        url.set_path(&path);
        Ok(Self { url })
    }

    /// True when `url` has the base's origin and sits at or under its path.
    pub(crate) fn contains(&self, url: &Url) -> bool {
        let base_path = self.url.path().trim_end_matches('/');
        let path = url.path();
        url.scheme() == self.url.scheme()
            && url.host_str().map(str::to_ascii_lowercase)
                == self.url.host_str().map(str::to_ascii_lowercase)
            && url.port_or_known_default() == self.url.port_or_known_default()
            && url.username().is_empty()
            && url.password().is_none()
            && (base_path.is_empty()
                || path == base_path
                || path
                    .strip_prefix(base_path)
                    .is_some_and(|rest| rest.starts_with('/')))
    }

    /// The URL of `segments` under the base, with every query value encoded.
    pub(crate) fn resource_url(&self, segments: &[&str], query: &[(&str, &str)]) -> Url {
        let mut url = self.url.clone();
        url.path_segments_mut()
            .expect("an http base URL has path segments")
            .pop_if_empty()
            .extend(segments);
        if !query.is_empty() {
            url.query_pairs_mut().extend_pairs(query);
        }
        url
    }
}

/// Why a paged walk stopped before the server said it was done.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WalkStop {
    /// A next link named a page already read.
    RepeatedLink,
    /// A next link left the configured origin or base path.
    OffBaseLink,
    /// A request was redirected off the configured origin or base path.
    OffBaseRedirect,
    /// The page cap was reached with a next link still pending.
    PageCap,
    /// A page after the first failed.
    LaterPageFailed,
}

/// Everything one paged search read.
#[derive(Debug, Clone, Default)]
pub(crate) struct Walk {
    /// Entries whose search mode is `match` (or absent), in server order.
    pub matches: Vec<Value>,
    /// True when an OperationOutcome entry reported an error or fatal issue.
    pub outcome_error: bool,
    /// Set when the walk stopped early.
    pub stop: Option<WalkStop>,
}

/// A read-only client bound to one FHIR base.
#[derive(Clone)]
pub(crate) struct FhirClient {
    client: ClientWithMiddleware,
    base: FhirBase,
}

impl FhirClient {
    /// Reads `BIOMCP_FHIR_BASE`. An unset or blank variable is `NotConfigured`.
    pub(crate) fn from_env() -> Result<Self, BioMcpError> {
        let raw = std::env::var(FHIR_BASE_ENV)
            .ok()
            .filter(|value| !value.trim().is_empty())
            .ok_or(BioMcpError::Fhir(FhirError::NotConfigured))?;
        Self::new(&raw)
    }

    pub(crate) fn is_configured() -> bool {
        std::env::var(FHIR_BASE_ENV).is_ok_and(|value| !value.trim().is_empty())
    }

    pub(crate) fn new(raw_base: &str) -> Result<Self, BioMcpError> {
        let base = FhirBase::parse(raw_base).map_err(BioMcpError::Fhir)?;
        let client = super::ca_bundle::build_client(
            reqwest::Client::builder()
                .no_proxy()
                .timeout(Duration::from_secs(30))
                .connect_timeout(Duration::from_secs(10))
                .user_agent(concat!("biomcp-cli/", env!("CARGO_PKG_VERSION")))
                .redirect(redirect_policy(base.clone())),
        )?;
        let client = ClientBuilder::new(client)
            .with(super::shared_retry_middleware())
            .with(super::RetryAfterTooManyRequestsMiddleware)
            .with(super::ResponseBodyLimitMiddleware)
            .build();
        Ok(Self { client, base })
    }

    /// Builds the one kind of request this client sends: a GET with no-store.
    fn request(&self, url: Url) -> RequestBuilder {
        super::apply_no_store(
            self.client
                .get(url)
                .header(CACHE_CONTROL, "no-store")
                .header(ACCEPT, ACCEPT_FHIR_JSON),
        )
    }

    /// The one FHIR request function. Every GET goes through it.
    async fn get_json(&self, url: Url) -> Result<Value, FhirError> {
        let response = self.request(url).send().await.map_err(transport_error)?;
        let status = response.status();
        let body = response
            .bytes()
            .await
            .map_err(|error| reqwest_error(&error.without_url()))?;
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(FhirError::NotFound);
        }
        if !status.is_success() {
            return Err(FhirError::Status(status.as_u16()));
        }
        serde_json::from_slice(&body).map_err(|_| FhirError::Decode)
    }

    /// Reads `Patient/{id}`.
    pub(crate) async fn read_patient(&self, id: &PatientId) -> Result<Value, FhirError> {
        let resource = self
            .get_json(self.base.resource_url(&["Patient", id.as_str()], &[]))
            .await?;
        if resource.get("resourceType").and_then(Value::as_str) != Some("Patient") {
            return Err(FhirError::Decode);
        }
        Ok(resource)
    }

    /// Reads `Condition?patient={id}&_count=100` and follows next links.
    pub(crate) async fn search_conditions(&self, id: &PatientId) -> Result<Walk, FhirError> {
        let count = CONDITION_PAGE_SIZE.to_string();
        let start = self.base.resource_url(
            &["Condition"],
            &[("patient", id.as_str()), ("_count", count.as_str())],
        );
        self.walk(start, MAX_PAGES).await
    }

    /// Follows next links from `start`. A failure on the first page is an
    /// error; any later stop keeps the pages read and names the reason.
    pub(crate) async fn walk(&self, start: Url, max_pages: usize) -> Result<Walk, FhirError> {
        let mut walk = Walk::default();
        let mut visited = HashSet::new();
        let mut next = Some(start);
        let mut pages_read = 0usize;
        while let Some(url) = next.take() {
            if pages_read >= max_pages {
                walk.stop = Some(WalkStop::PageCap);
                break;
            }
            visited.insert(url.as_str().to_string());
            let page = match self.get_json(url.clone()).await {
                Ok(page) => page,
                Err(FhirError::Redirect) => {
                    walk.stop = Some(WalkStop::OffBaseRedirect);
                    break;
                }
                Err(error) if pages_read == 0 => return Err(error),
                Err(_) => {
                    walk.stop = Some(WalkStop::LaterPageFailed);
                    break;
                }
            };
            if page.get("resourceType").and_then(Value::as_str) != Some("Bundle") {
                if pages_read == 0 {
                    return Err(FhirError::Decode);
                }
                walk.stop = Some(WalkStop::LaterPageFailed);
                break;
            }
            pages_read += 1;
            read_bundle_page(&page, &mut walk);
            let Some(link) = next_link(&page) else {
                break;
            };
            let Ok(resolved) = url.join(link) else {
                walk.stop = Some(WalkStop::OffBaseLink);
                break;
            };
            if !self.base.contains(&resolved) {
                walk.stop = Some(WalkStop::OffBaseLink);
                break;
            }
            if visited.contains(resolved.as_str()) {
                walk.stop = Some(WalkStop::RepeatedLink);
                break;
            }
            next = Some(resolved);
        }
        Ok(walk)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("the redirect left the configured FHIR base")]
struct RedirectRefused;

fn redirect_policy(base: FhirBase) -> reqwest::redirect::Policy {
    reqwest::redirect::Policy::custom(move |attempt| {
        if attempt.previous().len() >= MAX_REDIRECTS || !base.contains(attempt.url()) {
            return attempt.error(RedirectRefused);
        }
        attempt.follow()
    })
}

fn transport_error(error: reqwest_middleware::Error) -> FhirError {
    match error {
        reqwest_middleware::Error::Reqwest(error) => reqwest_error(&error.without_url()),
        // A middleware layer can wrap the transport error. Only its kind is read.
        reqwest_middleware::Error::Middleware(error) => {
            if error.chain().any(|cause| cause.is::<RedirectRefused>()) {
                FhirError::Redirect
            } else {
                error
                    .chain()
                    .find_map(|cause| cause.downcast_ref::<reqwest::Error>())
                    .map_or(FhirError::Transport, reqwest_error)
            }
        }
    }
}

fn reqwest_error(error: &reqwest::Error) -> FhirError {
    if error.is_redirect() {
        FhirError::Redirect
    } else if error.is_timeout() {
        FhirError::Timeout
    } else if error.is_connect() {
        FhirError::Connect
    } else {
        FhirError::Transport
    }
}

/// Adds one Bundle page to the walk. An OperationOutcome is a diagnostic
/// whatever the entry claims, so it never becomes data.
fn read_bundle_page(page: &Value, walk: &mut Walk) {
    for entry in array(page.get("entry")) {
        let Some(resource) = entry.get("resource") else {
            continue;
        };
        let mode = entry
            .get("search")
            .and_then(|search| search.get("mode"))
            .and_then(Value::as_str);
        if resource.get("resourceType").and_then(Value::as_str) == Some("OperationOutcome")
            || mode == Some("outcome")
        {
            walk.outcome_error |= array(resource.get("issue")).any(|issue| {
                matches!(
                    issue.get("severity").and_then(Value::as_str),
                    Some("error" | "fatal")
                )
            });
            continue;
        }
        if mode.is_none_or(|mode| mode == "match") {
            walk.matches.push(resource.clone());
        }
    }
}

fn next_link(page: &Value) -> Option<&str> {
    array(page.get("link")).find_map(|link| {
        (link.get("relation").and_then(Value::as_str) == Some("next"))
            .then(|| link.get("url").and_then(Value::as_str))
            .flatten()
    })
}

fn array(value: Option<&Value>) -> impl Iterator<Item = &Value> {
    value.and_then(Value::as_array).into_iter().flatten()
}

#[cfg(test)]
pub(crate) mod test_server;

#[cfg(test)]
mod tests;

#[cfg(test)]
impl FhirClient {
    pub(crate) fn request_for_test(&self, url: Url) -> RequestBuilder {
        self.request(url)
    }

    pub(crate) fn base(&self) -> &FhirBase {
        &self.base
    }
}

#[cfg(test)]
pub(crate) fn no_store_mode() -> http_cache_reqwest::CacheMode {
    http_cache_reqwest::CacheMode::NoStore
}
