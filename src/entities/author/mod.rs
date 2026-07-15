//! Public, provider-exact author identity types.

mod detail;
mod search;

#[cfg(test)]
pub(crate) use detail::ProviderAuthorRecord;
pub use detail::{AuthorDetail, detail};
pub use search::{AuthorSearchResponse, search};

use crate::error::BioMcpError;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorIdProvider {
    SemanticScholar,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderAuthorId {
    pub provider: AuthorIdProvider,
    pub value: String,
}

impl FromStr for ProviderAuthorId {
    type Err = BioMcpError;
    fn from_str(raw: &str) -> Result<Self, Self::Err> {
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
        write!(f, "semanticscholar:{}", self.value)
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

pub(crate) fn evidence_url(value: &str) -> String {
    format!("https://www.semanticscholar.org/author/{value}")
}
pub(crate) fn provider_id(value: String) -> ProviderAuthorId {
    ProviderAuthorId {
        provider: AuthorIdProvider::SemanticScholar,
        value,
    }
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
            "orcid:0000-0000",
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
}
