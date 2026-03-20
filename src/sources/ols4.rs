use std::borrow::Cow;

use serde::Deserialize;

use crate::error::BioMcpError;

const OLS4_BASE: &str = "https://www.ebi.ac.uk/ols4";
const OLS4_API: &str = "ols4";
const OLS4_BASE_ENV: &str = "BIOMCP_OLS4_BASE";
const OLS4_ONTOLOGIES: &str = "hgnc,mesh,mondo,doid,hp,go,chebi,dron,ncit,ordo,wikipathways,so";

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
    fn new_for_test(base: String) -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
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

    pub async fn search(&self, query: &str) -> Result<Vec<OlsDoc>, BioMcpError> {
        let query = query.trim();
        if query.is_empty() {
            return Ok(Vec::new());
        }

        let resp = crate::sources::apply_cache_mode(self.client.get(self.endpoint("api/search")))
            .query(&[
                ("q", query),
                ("rows", "10"),
                ("groupField", "iri"),
                ("ontology", OLS4_ONTOLOGIES),
            ])
            .send()
            .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_body(resp, OLS4_API).await?;

        if !status.is_success() {
            return Err(BioMcpError::Api {
                api: OLS4_API.to_string(),
                message: format!("HTTP {status}: {}", crate::sources::body_excerpt(&bytes)),
            });
        }

        crate::sources::ensure_json_content_type(OLS4_API, content_type.as_ref(), &bytes)?;
        let response: OlsSearchEnvelope =
            serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
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
    #[allow(dead_code)]
    #[serde(default)]
    pub description: Vec<String>,
    #[serde(default)]
    pub exact_synonyms: Vec<String>,
    #[allow(dead_code)]
    #[serde(default)]
    pub is_defining_ontology: bool,
    #[allow(dead_code)]
    #[serde(default, rename = "type")]
    pub doc_type: Option<String>,
}

#[cfg(test)]
mod tests {
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::OlsClient;

    #[tokio::test]
    async fn search_uses_required_query_contract() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/search"))
            .and(query_param("q", "ERBB1"))
            .and(query_param("rows", "10"))
            .and(query_param("groupField", "iri"))
            .and(query_param(
                "ontology",
                "hgnc,mesh,mondo,doid,hp,go,chebi,dron,ncit,ordo,wikipathways,so",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "response": {
                    "docs": [
                        {
                            "iri": "http://example.org/hgnc/3236",
                            "ontology_name": "hgnc",
                            "ontology_prefix": "hgnc",
                            "short_form": "hgnc:3236",
                            "obo_id": "HGNC:3236",
                            "label": "EGFR",
                            "description": [],
                            "exact_synonyms": ["ERBB1"],
                            "type": "class"
                        }
                    ]
                }
            })))
            .mount(&server)
            .await;

        let client = OlsClient::new_for_test(server.uri()).expect("client");
        let rows = client.search("ERBB1").await.expect("search");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].label, "EGFR");
    }
}
