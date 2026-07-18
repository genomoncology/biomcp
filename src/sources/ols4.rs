use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;

use reqwest::StatusCode;
use reqwest::header::HeaderValue;
use serde::Deserialize;

use crate::error::BioMcpError;

const OLS4_BASE: &str = "https://www.ebi.ac.uk/ols4";
const OLS4_API: &str = "ols4";
const OLS4_BASE_ENV: &str = "BIOMCP_OLS4_BASE";
const OLS4_ONTOLOGIES: &str = "hgnc,mesh,mondo,doid,hp,go,chebi,dron,ncit,ordo,wikipathways,so";

// dead-code reason: ols4::OlsSearchRequestPlan preserves the provider shape used by source contract fixtures
#[allow(dead_code)]
pub struct OlsSearchRequestPlan {
    pub method: &'static str,
    pub path: Option<&'static str>,
    pub query_params: Vec<(&'static str, String)>,
    pub source_label: &'static str,
    pub base_url: String,
    pub cache_mode: &'static str,
    pub status_expectation: &'static str,
    pub content_type_expectation: &'static str,
}

pub struct OlsClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl OlsClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(OLS4_BASE, OLS4_BASE_ENV),
        })
    }

    #[cfg(test)]
    pub(crate) fn new_for_test(base: String) -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::test_client()?,
            base: Cow::Owned(base),
        })
    }

    fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base.as_ref().trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    pub fn search_request_plan(&self, query: &str) -> OlsSearchRequestPlan {
        let query = query.trim();
        OlsSearchRequestPlan {
            method: "GET",
            path: (!query.is_empty()).then_some("/api/search"),
            query_params: if query.is_empty() {
                Vec::new()
            } else {
                vec![
                    ("q", query.to_string()),
                    ("rows", "10".to_string()),
                    ("groupField", "iri".to_string()),
                    ("ontology", OLS4_ONTOLOGIES.to_string()),
                ]
            },
            source_label: "ols4",
            base_url: self.base.to_string(),
            cache_mode: "default",
            status_expectation: "non-2xx => Api",
            content_type_expectation: "json",
        }
    }

    pub async fn search(&self, query: &str) -> Result<Vec<OlsDoc>, BioMcpError> {
        let plan = self.search_request_plan(query);
        let Some(path) = plan.path else {
            return Ok(Vec::new());
        };

        let resp = crate::sources::apply_cache_mode(self.client.get(self.endpoint(path)))
            .query(&plan.query_params)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::OLS4,
            ))
            .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::OLS4),
        )
        .await?;

        Self::decode_search_response(status, content_type.as_ref(), &bytes).map_err(|error| {
            error.with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::OLS4,
            ))
        })
    }

    pub(crate) fn decode_search_response(
        status: StatusCode,
        content_type: Option<&HeaderValue>,
        bytes: &[u8],
    ) -> Result<Vec<OlsDoc>, BioMcpError> {
        if !status.is_success() {
            return Err(BioMcpError::Api {
                api: OLS4_API.to_string(),
                message: format!("HTTP {status}: {}", crate::sources::body_excerpt(bytes)),
            });
        }

        crate::sources::ensure_json_content_type(
            crate::error::SourceContext::retry(crate::error::SourceProvider::OLS4),
            content_type,
            bytes,
        )?;
        let response: OlsSearchEnvelope =
            serde_json::from_slice(bytes).map_err(|source| BioMcpError::ApiJson {
                api: OLS4_API.to_string(),
                source,
            })?;
        Ok(response.response.docs)
    }
}

#[derive(Debug, Clone, Deserialize)]
struct OlsSearchEnvelope {
    response: OlsSearchResponse,
}

#[derive(Debug, Clone, Deserialize)]
struct OlsSearchResponse {
    #[serde(default)]
    docs: Vec<OlsDoc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OlsDoc {
    pub iri: String,
    // dead-code reason: ols4::ontology_name preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    #[serde(default)]
    pub ontology_name: String,
    #[serde(default)]
    pub ontology_prefix: String,
    #[serde(default)]
    pub short_form: Option<String>,
    #[serde(default)]
    pub obo_id: Option<String>,
    #[serde(default)]
    pub label: String,
    // dead-code reason: ols4::description preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    #[serde(default)]
    pub description: Vec<String>,
    #[serde(default)]
    pub exact_synonyms: Vec<String>,
    // dead-code reason: ols4::is_defining_ontology preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    #[serde(default)]
    pub is_defining_ontology: bool,
    // dead-code reason: ols4::doc_type preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    #[serde(default, rename = "type")]
    pub doc_type: Option<String>,
}

#[cfg(test)]
mod tests;
