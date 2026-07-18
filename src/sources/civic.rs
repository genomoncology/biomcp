use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;
use std::collections::HashSet;

use reqwest::StatusCode;
use reqwest::header::HeaderValue;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::BioMcpError;
use crate::sources::{RequestBody, RequestPlan, request_from_plan};

const CIVIC_BASE: &str = "https://civicdb.org/api";
const CIVIC_API: &str = "civic";
const CIVIC_BASE_ENV: &str = "BIOMCP_CIVIC_BASE";

const CIVIC_CONTEXT_QUERY: &str = r#"
query CivicContext(
  $molecularProfileName: String
  $therapyName: String
  $diseaseName: String
  $first: Int!
) {
  evidenceItems(
    molecularProfileName: $molecularProfileName
    therapyName: $therapyName
    diseaseName: $diseaseName
    status: ACCEPTED
    first: $first
  ) {
    totalCount
    nodes {
      id
      name
      status
      evidenceType
      evidenceLevel
      significance
      molecularProfile {
        name
      }
      disease {
        displayName
      }
      therapies {
        name
      }
      source {
        citation
        sourceType
        publicationYear
      }
    }
  }
  assertions(
    molecularProfileName: $molecularProfileName
    therapyName: $therapyName
    diseaseName: $diseaseName
    status: ACCEPTED
    first: $first
  ) {
    totalCount
    nodes {
      id
      name
      status
      assertionType
      assertionDirection
      ampLevel
      significance
      molecularProfile {
        name
      }
      disease {
        displayName
      }
      therapies {
        name
      }
      summary
      approvals {
        totalCount
      }
    }
  }
}
"#;

pub struct CivicClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl CivicClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(CIVIC_BASE, CIVIC_BASE_ENV),
        })
    }

    pub(crate) fn context_plan(
        filter: CivicFilter<'_>,
        limit: usize,
    ) -> Result<RequestPlan, BioMcpError> {
        let (variable_name, variable_value) = match filter {
            CivicFilter::MolecularProfile(value) => (
                "molecularProfileName",
                required_query_value("molecular profile name", value)?,
            ),
            CivicFilter::Therapy(value) => {
                ("therapyName", required_query_value("therapy name", value)?)
            }
            CivicFilter::Disease(value) => {
                ("diseaseName", required_query_value("disease name", value)?)
            }
        };
        let first = limit.clamp(1, 25);
        let mut variables = serde_json::Map::new();
        variables.insert("first".to_string(), serde_json::json!(first));
        variables.insert(
            variable_name.to_string(),
            serde_json::Value::String(variable_value),
        );

        let mut plan = RequestPlan::post("graphql");
        plan.body = RequestBody::Json(serde_json::json!({
            "query": CIVIC_CONTEXT_QUERY,
            "variables": serde_json::Value::Object(variables),
        }));
        Ok(plan)
    }

    pub(crate) fn decode_json_response<T: DeserializeOwned>(
        status: StatusCode,
        content_type: Option<&HeaderValue>,
        bytes: &[u8],
    ) -> Result<T, BioMcpError> {
        crate::sources::decode_json(
            crate::error::SourceContext::retry(crate::error::SourceProvider::CIVIC),
            status,
            content_type,
            bytes,
            true,
        )
    }

    async fn post_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::CIVIC,
            ))
            .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::CIVIC),
        )
        .await?;
        Self::decode_json_response(status, content_type.as_ref(), &bytes).map_err(|error| {
            error.with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::CIVIC,
            ))
        })
    }

    pub async fn by_molecular_profile(
        &self,
        molecular_profile_name: &str,
        limit: usize,
    ) -> Result<CivicContext, BioMcpError> {
        self.fetch_context(CivicFilter::MolecularProfile(molecular_profile_name), limit)
            .await
    }

    pub async fn by_therapy(
        &self,
        therapy_name: &str,
        limit: usize,
    ) -> Result<CivicContext, BioMcpError> {
        self.fetch_context(CivicFilter::Therapy(therapy_name), limit)
            .await
    }

    pub async fn by_disease(
        &self,
        disease_name: &str,
        limit: usize,
    ) -> Result<CivicContext, BioMcpError> {
        self.fetch_context(CivicFilter::Disease(disease_name), limit)
            .await
    }

    async fn fetch_context(
        &self,
        filter: CivicFilter<'_>,
        limit: usize,
    ) -> Result<CivicContext, BioMcpError> {
        let plan = Self::context_plan(filter, limit)?;
        let resp: GraphQlResponse<CivicContextData> = self
            .post_json(request_from_plan(&self.client, self.base.as_ref(), &plan))
            .await?;
        Self::context_from_response(resp)
    }

    fn context_from_response(
        resp: GraphQlResponse<CivicContextData>,
    ) -> Result<CivicContext, BioMcpError> {
        if let Some(errors) = resp.errors {
            let message = errors
                .into_iter()
                .filter_map(|row| row.message)
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
                .collect::<Vec<_>>()
                .join("; ");
            if !message.is_empty() {
                return Err(BioMcpError::Api {
                    api: CIVIC_API.to_string(),
                    message,
                });
            }
        }

        let data = resp.data.unwrap_or_default();
        Ok(CivicContext {
            evidence_total_count: data.evidence_items.total_count,
            assertion_total_count: data.assertions.total_count,
            evidence_items: data
                .evidence_items
                .nodes
                .into_iter()
                .map(CivicEvidenceItem::from_node)
                .collect(),
            assertions: data
                .assertions
                .nodes
                .into_iter()
                .map(CivicAssertion::from_node)
                .collect(),
        })
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CivicContext {
    pub evidence_total_count: usize,
    pub assertion_total_count: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_items: Vec<CivicEvidenceItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub assertions: Vec<CivicAssertion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CivicEvidenceItem {
    pub id: i64,
    pub name: String,
    pub molecular_profile: String,
    pub evidence_type: String,
    pub evidence_level: String,
    pub significance: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disease: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub therapies: Vec<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_year: Option<i32>,
}

impl CivicEvidenceItem {
    fn from_node(node: CivicEvidenceNode) -> Self {
        Self {
            id: node.id,
            name: clean_required(node.name),
            molecular_profile: clean_required(node.molecular_profile.name),
            evidence_type: clean_required(node.evidence_type),
            evidence_level: clean_required(node.evidence_level),
            significance: clean_required(node.significance),
            disease: node
                .disease
                .and_then(|row| clean_optional(Some(row.display_name))),
            therapies: clean_names(node.therapies),
            status: clean_required(node.status),
            citation: node
                .source
                .as_ref()
                .and_then(|src| clean_optional(src.citation.clone())),
            source_type: node
                .source
                .as_ref()
                .and_then(|src| clean_optional(src.source_type.clone())),
            publication_year: node.source.and_then(|src| src.publication_year),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CivicAssertion {
    pub id: i64,
    pub name: String,
    pub molecular_profile: String,
    pub assertion_type: String,
    pub assertion_direction: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amp_level: Option<String>,
    pub significance: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disease: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub therapies: Vec<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    pub approvals_count: usize,
}

impl CivicAssertion {
    fn from_node(node: CivicAssertionNode) -> Self {
        Self {
            id: node.id,
            name: clean_required(node.name),
            molecular_profile: clean_required(node.molecular_profile.name),
            assertion_type: clean_required(node.assertion_type),
            assertion_direction: clean_required(node.assertion_direction),
            amp_level: clean_optional(node.amp_level),
            significance: clean_required(node.significance),
            disease: node
                .disease
                .and_then(|row| clean_optional(Some(row.display_name))),
            therapies: clean_names(node.therapies),
            status: clean_required(node.status),
            summary: clean_optional(node.summary),
            approvals_count: node.approvals.map_or(0, |v| v.total_count),
        }
    }
}

fn clean_required(value: String) -> String {
    value.trim().to_string()
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn clean_names(rows: Vec<CivicNameNode>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for row in rows {
        let Some(name) = clean_optional(Some(row.name)) else {
            continue;
        };
        let key = name.to_ascii_lowercase();
        if !seen.insert(key) {
            continue;
        }
        out.push(name);
    }
    out
}

fn required_query_value(label: &str, value: &str) -> Result<String, BioMcpError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(BioMcpError::InvalidArgument(format!(
            "CIViC {label} is required."
        )));
    }
    if trimmed.len() > 256 {
        return Err(BioMcpError::InvalidArgument(format!(
            "CIViC {label} is too long."
        )));
    }
    Ok(trimmed.to_string())
}

pub(crate) enum CivicFilter<'a> {
    MolecularProfile(&'a str),
    Therapy(&'a str),
    Disease(&'a str),
}

#[derive(Debug, Deserialize)]
struct GraphQlResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphQlError>>,
}

#[derive(Debug, Deserialize)]
struct GraphQlError {
    message: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct CivicContextData {
    #[serde(rename = "evidenceItems", default)]
    evidence_items: CivicEvidenceConnection,
    #[serde(default)]
    assertions: CivicAssertionConnection,
}

#[derive(Debug, Default, Deserialize)]
struct CivicEvidenceConnection {
    #[serde(rename = "totalCount", default)]
    total_count: usize,
    #[serde(default)]
    nodes: Vec<CivicEvidenceNode>,
}

#[derive(Debug, Default, Deserialize)]
struct CivicAssertionConnection {
    #[serde(rename = "totalCount", default)]
    total_count: usize,
    #[serde(default)]
    nodes: Vec<CivicAssertionNode>,
}

#[derive(Debug, Deserialize)]
struct CivicEvidenceNode {
    id: i64,
    name: String,
    status: String,
    #[serde(rename = "evidenceType")]
    evidence_type: String,
    #[serde(rename = "evidenceLevel")]
    evidence_level: String,
    significance: String,
    #[serde(rename = "molecularProfile")]
    molecular_profile: CivicNameNode,
    disease: Option<CivicDiseaseNode>,
    #[serde(default)]
    therapies: Vec<CivicNameNode>,
    source: Option<CivicSourceNode>,
}

#[derive(Debug, Deserialize)]
struct CivicAssertionNode {
    id: i64,
    name: String,
    status: String,
    #[serde(rename = "assertionType")]
    assertion_type: String,
    #[serde(rename = "assertionDirection")]
    assertion_direction: String,
    #[serde(rename = "ampLevel")]
    amp_level: Option<String>,
    significance: String,
    #[serde(rename = "molecularProfile")]
    molecular_profile: CivicNameNode,
    disease: Option<CivicDiseaseNode>,
    #[serde(default)]
    therapies: Vec<CivicNameNode>,
    summary: Option<String>,
    approvals: Option<CivicCountConnection>,
}

#[derive(Debug, Deserialize)]
struct CivicNameNode {
    name: String,
}

#[derive(Debug, Deserialize)]
struct CivicDiseaseNode {
    #[serde(rename = "displayName")]
    display_name: String,
}

#[derive(Debug, Deserialize)]
struct CivicSourceNode {
    citation: Option<String>,
    #[serde(rename = "sourceType")]
    source_type: Option<String>,
    #[serde(rename = "publicationYear")]
    publication_year: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct CivicCountConnection {
    #[serde(rename = "totalCount", default)]
    total_count: usize,
}

#[cfg(test)]
mod tests;
