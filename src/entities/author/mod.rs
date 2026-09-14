//! Public, provider-exact author identity types.

mod papers;
mod search;

#[allow(unused_imports)]
pub use papers::{
    AuthorPaper, AuthorPaperFull, AuthorPaperFullAuthor, AuthorPaperIdentifier,
    AuthorPaperOpenAccessPdf, AuthorPapersFullResult, AuthorPapersPagination, AuthorPapersResult,
    papers, papers_full,
};
pub use search::{AuthorSearchResponse, search};

use crate::error::BioMcpError;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

/// Exact static error for every malformed `orcid:` author ID. Never echoes input.
pub(crate) const ORCID_ID_ERROR: &str = "author ID must use the exact form orcid:dddd-dddd-dddd-dddC with a valid ISO/IEC 7064 MOD 11-2 checksum.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorIdProvider {
    SemanticScholar,
    Orcid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderAuthorId {
    pub provider: AuthorIdProvider,
    pub value: String,
}

impl FromStr for ProviderAuthorId {
    type Err = BioMcpError;
    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        if let Some(value) = raw.strip_prefix("orcid:") {
            return parse_orcid_value(value).map(|value| Self {
                provider: AuthorIdProvider::Orcid,
                value,
            });
        }
        if is_orcid_shaped(raw) {
            return Err(BioMcpError::InvalidArgument(ORCID_ID_ERROR.into()));
        }
        let Some(value) = raw.strip_prefix("semanticscholar:") else {
            return Err(BioMcpError::InvalidArgument("author ID must use the exact form semanticscholar:<numeric-id>; PubMed and ORCID author IDs are not supported in this release".into()));
        };
        if value.len() > 512 {
            return Err(BioMcpError::InvalidArgument("author ID is too long".into()));
        }
        if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
            return Err(BioMcpError::InvalidArgument("Semantic Scholar author ID must be a nonempty ASCII-decimal value in the form semanticscholar:<numeric-id>".into()));
        }
        Ok(Self {
            provider: AuthorIdProvider::SemanticScholar,
            value: value.to_string(),
        })
    }
}

impl fmt::Display for ProviderAuthorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.provider {
            AuthorIdProvider::SemanticScholar => write!(f, "semanticscholar:{}", self.value),
            AuthorIdProvider::Orcid => write!(f, "orcid:{}", self.value),
        }
    }
}
impl Serialize for ProviderAuthorId {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}
impl<'de> Deserialize<'de> for ProviderAuthorId {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        String::deserialize(d)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AuthorIdentity {
    ExactProvider { id: ProviderAuthorId },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthorEvidence {
    pub source: &'static str,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum TemporalAnchor {
    ObservedAt(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthorAssertion<T> {
    pub value: T,
    pub evidence: AuthorEvidence,
    pub temporal: TemporalAnchor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderStatus {
    Available,
    Degraded,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthorWarning {
    pub code: &'static str,
    pub message: &'static str,
}
impl AuthorWarning {
    pub(crate) fn unresolved_orcid() -> Self {
        Self {
            code: "orcid_link_not_established",
            message: "BioMCP has not established an ORCID link in this release",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthorConflict {
    pub field: String,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthorSourceStatus {
    pub source: &'static str,
    pub status: ProviderStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthorEvidenceUrl {
    pub source: &'static str,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthorMeta {
    pub source_status: Vec<AuthorSourceStatus>,
    pub evidence_urls: Vec<AuthorEvidenceUrl>,
    pub next_commands: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArticleAuthorRecord {
    pub identity: AuthorIdentity,
    pub display_name: String,
    pub affiliations: Vec<AuthorAssertion<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArticleAuthorsResult {
    pub article: crate::entities::article::ArticleRelatedPaper,
    pub authors: Vec<ArticleAuthorRecord>,
    pub _meta: AuthorMeta,
}

pub(crate) fn evidence_url(value: &str) -> String {
    format!("https://www.semanticscholar.org/author/{value}")
}
pub(crate) fn provider_id(value: String) -> ProviderAuthorId {
    ProviderAuthorId {
        provider: AuthorIdProvider::SemanticScholar,
        value,
    }
}

/// Validate `dddd-dddd-dddd-dddC` and return the canonical 19-byte value.
fn parse_orcid_value(raw: &str) -> Result<String, BioMcpError> {
    let invalid = || BioMcpError::InvalidArgument(ORCID_ID_ERROR.into());
    let bytes = raw.as_bytes();
    if bytes.len() != 19 || bytes[4] != b'-' || bytes[9] != b'-' || bytes[14] != b'-' {
        return Err(invalid());
    }
    let mut digits = [0u8; 15];
    let mut index = 0;
    for &byte in bytes[..18].iter().filter(|b| **b != b'-') {
        if !byte.is_ascii_digit() {
            return Err(invalid());
        }
        digits[index] = byte;
        index += 1;
    }
    let check = bytes[18];
    if index != 15 || (check != b'X' && !check.is_ascii_digit()) {
        return Err(invalid());
    }
    if check != orcid_check_digit(&digits) {
        return Err(invalid());
    }
    Ok(raw.to_string())
}

fn orcid_check_digit(digits: &[u8; 15]) -> u8 {
    let mut total: u32 = 0;
    for &digit in digits {
        total = (total + u32::from(digit - b'0')) * 2;
    }
    let remainder = (12 - (total % 11)) % 11;
    if remainder == 10 {
        b'X'
    } else {
        b'0' + remainder as u8
    }
}

/// True for every input the ticket routes to the static ORCID rejection even
/// though it lacks the exact lowercase prefix: uppercase/mixed prefixes,
/// whitespace-padded `orcid:` forms, ORCID URL spellings, and bare
/// hyphen-shaped IDs (missing prefix). Unrelated IDs stay on the
/// Semantic Scholar guidance.
fn is_orcid_shaped(raw: &str) -> bool {
    for prefix in ["ORCID:", "Orcid:", "https://orcid.org/", "www.orcid.org/"] {
        if raw.starts_with(prefix) {
            return true;
        }
    }
    let trimmed = raw.trim();
    if trimmed != raw && trimmed.starts_with("orcid:") {
        return true;
    }
    let bytes = trimmed.as_bytes();
    bytes.len() == 19 && bytes[4] == b'-' && bytes[9] == b'-' && bytes[14] == b'-'
}
pub(crate) fn valid_wire_id(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit()))
}
pub(crate) fn nonblank(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

pub(crate) fn sanitized_provider_error(err: BioMcpError) -> BioMcpError {
    sanitized_provider_error_with_message(
        err,
        "Semantic Scholar author data is unavailable; retry later",
    )
}

pub(crate) fn sanitized_provider_error_with_message(
    err: BioMcpError,
    message: &'static str,
) -> BioMcpError {
    match err {
        BioMcpError::WithSourceContext { context, source } => {
            sanitized_provider_error_with_message(*source, message).with_source_context(context)
        }
        _ => BioMcpError::Api {
            api: "semantic_scholar".into(),
            message: message.into(),
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProviderAuthorRecord {
    pub id: ProviderAuthorId,
    pub source: &'static str,
    pub status: ProviderStatus,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthorDetail {
    pub identity: AuthorIdentity,
    pub display_name: String,
    pub provider_records: Vec<ProviderAuthorRecord>,
    pub affiliations: Vec<AuthorAssertion<String>>,
    pub paper_count: Option<u64>,
    pub citation_count: Option<u64>,
    pub h_index: Option<u64>,
    pub conflicts: Vec<AuthorConflict>,
    pub warnings: Vec<AuthorWarning>,
    pub _meta: AuthorMeta,
}

pub async fn detail(raw_id: &str) -> Result<AuthorDetail, crate::error::BioMcpError> {
    let requested: ProviderAuthorId = raw_id.parse()?;
    match requested.provider {
        AuthorIdProvider::Orcid => orcid_detail(&requested).await,
        AuthorIdProvider::SemanticScholar => {
            let row = crate::sources::semantic_scholar::SemanticScholarClient::new()?
                .author_detail(&requested.value)
                .await
                .map_err(sanitized_detail_error)?;
            map_detail(row, &requested, &chrono::Utc::now().to_rfc3339())
        }
    }
}

fn orcid_unavailable() -> crate::error::BioMcpError {
    crate::error::BioMcpError::Api {
        api: "orcid".into(),
        message: "ORCID is unavailable; retry later".into(),
    }
    .with_source_context(crate::error::SourceContext::retry(
        crate::error::SourceProvider::ORCID,
    ))
}

fn orcid_command_deadline() -> std::time::Duration {
    #[cfg(debug_assertions)]
    if let Ok(value) = std::env::var("BIOMCP_TEST_AUTHOR_PAPERS_DEADLINE_MS")
        && let Ok(millis) = value.trim().parse::<u64>()
    {
        return std::time::Duration::from_millis(millis);
    }
    std::time::Duration::from_secs(35)
}

async fn orcid_detail(
    requested: &ProviderAuthorId,
) -> Result<AuthorDetail, crate::error::BioMcpError> {
    let deadline = tokio::time::Instant::now() + orcid_command_deadline();
    // The absolute deadline covers admission, attempts, body read, decode,
    // and projection; the synchronous renderer runs on bounded output
    // afterward, the same accepted pattern as the papers command.
    tokio::time::timeout_at(deadline, orcid_detail_work(requested, deadline))
        .await
        .map_err(|_| orcid_unavailable())?
}

async fn orcid_detail_work(
    requested: &ProviderAuthorId,
    deadline: tokio::time::Instant,
) -> Result<AuthorDetail, crate::error::BioMcpError> {
    let client = crate::sources::orcid::OrcidClient::new()?;
    let person = client.person(&requested.value, deadline).await?;
    let display_name = person.public_display_name()?;
    let url = format!("https://orcid.org/{}", requested.value);
    Ok(AuthorDetail {
        identity: AuthorIdentity::ExactProvider {
            id: requested.clone(),
        },
        display_name,
        provider_records: vec![ProviderAuthorRecord {
            id: requested.clone(),
            source: "orcid",
            status: ProviderStatus::Available,
        }],
        affiliations: vec![],
        paper_count: None,
        citation_count: None,
        h_index: None,
        conflicts: vec![],
        warnings: vec![],
        _meta: AuthorMeta {
            source_status: vec![AuthorSourceStatus {
                source: "orcid",
                status: ProviderStatus::Available,
            }],
            evidence_urls: vec![AuthorEvidenceUrl {
                source: "orcid",
                url,
            }],
            next_commands: vec![format!("biomcp author papers {requested}")],
        },
    })
}

fn map_detail(
    row: crate::sources::semantic_scholar::SemanticScholarAuthor,
    requested: &ProviderAuthorId,
    observed_at: &str,
) -> Result<AuthorDetail, crate::error::BioMcpError> {
    let returned = valid_wire_id(row.author_id).ok_or_else(contract_error)?;
    if returned != requested.value {
        return Err(contract_error());
    }
    let display_name = nonblank(row.name).ok_or_else(contract_error)?;
    let url = evidence_url(&returned);
    let affiliations = row
        .affiliations
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| {
            let value = v.trim().to_string();
            (!value.is_empty()).then(|| AuthorAssertion {
                value,
                evidence: AuthorEvidence {
                    source: "semantic_scholar",
                    url: url.clone(),
                },
                temporal: TemporalAnchor::ObservedAt(observed_at.to_string()),
            })
        })
        .collect();
    let id = provider_id(returned);
    Ok(AuthorDetail {
        identity: AuthorIdentity::ExactProvider { id: id.clone() },
        display_name,
        provider_records: vec![ProviderAuthorRecord {
            id,
            source: "semantic_scholar",
            status: ProviderStatus::Available,
        }],
        affiliations,
        paper_count: row.paper_count,
        citation_count: row.citation_count,
        h_index: row.h_index,
        conflicts: vec![],
        warnings: vec![AuthorWarning::unresolved_orcid()],
        _meta: AuthorMeta {
            source_status: vec![AuthorSourceStatus {
                source: "semantic_scholar",
                status: ProviderStatus::Available,
            }],
            evidence_urls: vec![AuthorEvidenceUrl {
                source: "semantic_scholar",
                url,
            }],
            next_commands: vec![format!("biomcp author papers {requested}")],
        },
    })
}
fn contract_error() -> crate::error::BioMcpError {
    crate::error::BioMcpError::Api {
        api: "semantic_scholar".into(),
        message: "author detail response did not match the requested provider record".into(),
    }
}

fn sanitized_detail_error(err: crate::error::BioMcpError) -> crate::error::BioMcpError {
    sanitized_provider_error_with_message(err, "author detail is unavailable; retry later")
}

#[cfg(test)]
mod detail_tests {
    use super::*;
    use crate::sources::semantic_scholar::SemanticScholarAuthor;

    fn row(id: Option<&str>, name: Option<&str>) -> SemanticScholarAuthor {
        SemanticScholarAuthor {
            author_id: id.map(str::to_string),
            name: name.map(str::to_string),
            affiliations: Some(vec!["Institute".into()]),
            external_ids: Some(serde_json::Map::from_iter([(
                "ORCID".into(),
                serde_json::json!("private-sentinel"),
            )])),
            paper_count: Some(548),
            citation_count: Some(50_000),
            h_index: Some(100),
        }
    }

    #[test]
    fn detail_requires_matching_decimal_id_and_nonblank_name() {
        let requested: ProviderAuthorId = "semanticscholar:1".parse().unwrap();
        for invalid in [
            row(None, Some("Name")),
            row(Some("2"), Some("Name")),
            row(Some("1"), Some(" ")),
        ] {
            assert!(map_detail(invalid, &requested, "now").is_err());
        }
    }

    #[test]
    fn provider_failure_does_not_expose_response_body() {
        let error = sanitized_detail_error(
            crate::error::BioMcpError::Api {
                api: "semantic_scholar".into(),
                message: "HTTP 500: private-author@example.invalid fixture-private-profile".into(),
            }
            .with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::SEMANTIC_SCHOLAR,
            )),
        );
        let rendered = error.to_string();
        assert_eq!(error.code(), "api");
        assert_eq!(error.public_projection().source, Some("Semantic Scholar"));
        assert!(rendered.to_ascii_lowercase().contains("retry"));
        assert!(!format!("{error:?}").contains("private-author@example.invalid"));
        assert!(!rendered.contains("fixture-private-profile"));
    }

    #[test]
    fn detail_serialization_is_allowlisted_and_has_required_metadata_arrays() {
        let requested: ProviderAuthorId = "semanticscholar:1".parse().unwrap();
        let detail = map_detail(row(Some("1"), Some("Name")), &requested, "now").unwrap();
        let value = serde_json::to_value(detail).unwrap();
        assert_eq!(value["identity"]["id"], "semanticscholar:1");
        assert_eq!(value["provider_records"][0]["source"], "semantic_scholar");
        assert_eq!(value["paper_count"], 548);
        assert_eq!(value["citation_count"], 50_000);
        assert_eq!(value["h_index"], 100);
        assert_eq!(value["warnings"][0]["code"], "orcid_link_not_established");
        assert_eq!(
            value["_meta"]["next_commands"],
            serde_json::json!(["biomcp author papers semanticscholar:1"])
        );
        assert_eq!(
            value["affiliations"][0]["evidence"]["source"],
            "semantic_scholar"
        );
        assert_eq!(value["affiliations"][0]["temporal"]["kind"], "observed_at");
        let json = value.to_string();
        assert!(!json.contains("external_ids"));
        assert!(!json.contains("private-sentinel"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn provider_ids_are_strict_and_round_trip() {
        let id: ProviderAuthorId = "semanticscholar:1716151".parse().unwrap();
        assert_eq!(id.to_string(), "semanticscholar:1716151");
        assert!(
            format!("semanticscholar:{}", "1".repeat(512))
                .parse::<ProviderAuthorId>()
                .is_ok()
        );
        assert!(
            format!("semanticscholar:{}", "1".repeat(513))
                .parse::<ProviderAuthorId>()
                .is_err()
        );
        for invalid in [
            "1716151",
            "pubmed:1716151",
            "SemanticScholar:1",
            "semanticscholar:",
            "semanticscholar:..",
            "semanticscholar:1/2",
        ] {
            let error = invalid
                .parse::<ProviderAuthorId>()
                .expect_err("unsupported author ID should fail");
            assert!(
                error.to_string().contains("semanticscholar:<numeric-id>"),
                "error was not actionable for {invalid}: {error}"
            );
        }
    }
    #[test]
    fn public_serialization_is_allowlisted() {
        let result = search::map_row(
            crate::sources::semantic_scholar::SemanticScholarAuthor {
                author_id: Some("1".into()),
                name: Some("A Name".into()),
                affiliations: Some(vec!["Lab".into()]),
                external_ids: Some(serde_json::Map::from_iter([(
                    "ORCID".into(),
                    serde_json::json!("private"),
                )])),
                paper_count: None,
                citation_count: None,
                h_index: None,
            },
            "now",
        )
        .unwrap();
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("external_ids"));
        assert!(!json.contains("private"));
    }
    #[test]
    fn orcid_ids_accept_exact_checksummed_forms_and_round_trip() {
        for valid in ["orcid:0000-0002-1825-0097", "orcid:0000-0002-1694-233X"] {
            let id: ProviderAuthorId = valid.parse().unwrap_or_else(|e| panic!("{valid}: {e}"));
            assert_eq!(id.to_string(), valid);
            assert_eq!(id.provider, AuthorIdProvider::Orcid);
        }
    }
    #[test]
    fn orcid_ids_reject_every_malformed_category_with_one_static_message() {
        for invalid in [
            "orcid:0000-0002-1825-0098",  // wrong checksum
            "orcid:0000-0002-1694-233x",  // lowercase x
            "orcid:0000-0002-1694-233",   // too short
            "orcid:00000-002-1694-233X",  // wrong hyphen position
            "orcid:00000002-1694-233X",   // missing hyphen
            "orcid:0000-0002-1694--233X", // extra hyphen
            " orcid:0000-0002-1825-0097", // leading space
            "orcid:0000-0002-1825-0097 ", // trailing space
            "0000-0002-1825-0097",        // bare ID
            "ORCID:0000-0002-1825-0097",  // uppercase prefix
            "Orcid:0000-0002-1825-0097",
            "orcid:0000-0002-1825-0097/works",       // path suffix
            "orcid:0000-0002-1825-0097?x=1",         // query
            "orcid:0000-0002-1825-0097\u{1}",        // control
            "orcid:0000-0002-1825-00977",            // overlong
            "https://orcid.org/0000-0002-1825-0097", // URL form
            "orcid:0000-0002-1825-009７",            // Unicode digit
        ] {
            let error = invalid
                .parse::<ProviderAuthorId>()
                .expect_err("malformed ORCID ID should fail");
            let BioMcpError::InvalidArgument(message) = &error else {
                panic!("wrong variant for {invalid:?}: {error}");
            };
            assert_eq!(message, ORCID_ID_ERROR, "must not echo {invalid:?}");
            assert!(!format!("{error}").contains(invalid));
        }
        assert_eq!(
            "orcid:0000-0002-1825-0097"
                .parse::<ProviderAuthorId>()
                .unwrap()
                .to_string(),
            "orcid:0000-0002-1825-0097"
        );
    }
}
