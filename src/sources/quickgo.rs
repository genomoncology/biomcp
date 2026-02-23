use std::borrow::Cow;

use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::BioMcpError;

const QUICKGO_BASE: &str = "https://www.ebi.ac.uk/QuickGO/services";
const QUICKGO_API: &str = "quickgo";
const QUICKGO_BASE_ENV: &str = "BIOMCP_QUICKGO_BASE";

pub struct QuickGoClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl QuickGoClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(QUICKGO_BASE, QUICKGO_BASE_ENV),
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

    async fn get_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req).send().await?;
        let status = resp.status();
        let bytes = crate::sources::read_limited_body(resp, QUICKGO_API).await?;
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: QUICKGO_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }
        serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
            api: QUICKGO_API.to_string(),
            source,
        })
    }

    pub async fn annotations(
        &self,
        gene_product_id: &str,
        limit: usize,
    ) -> Result<Vec<QuickGoAnnotation>, BioMcpError> {
        let gene_product_id = gene_product_id.trim();
        if gene_product_id.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "QuickGO geneProductId is required".into(),
            ));
        }

        let url = self.endpoint("annotation/search");
        let page_size = limit.clamp(1, 25).to_string();
        let resp: QuickGoAnnotationResponse = self
            .get_json(self.client.get(&url).query(&[
                ("geneProductId", gene_product_id),
                ("limit", page_size.as_str()),
            ]))
            .await?;

        Ok(resp.results)
    }

    pub async fn terms(&self, go_ids: &[String]) -> Result<Vec<QuickGoTerm>, BioMcpError> {
        let mut ids = go_ids
            .iter()
            .map(|v| v.trim())
            .filter(|v| !v.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>();

        if ids.is_empty() {
            return Ok(Vec::new());
        }

        ids.sort();
        ids.dedup();
        let ids = ids.join(",");
        let url = self.endpoint(&format!("ontology/go/terms/{ids}"));
        let resp: QuickGoTermsResponse = self.get_json(self.client.get(&url)).await?;
        Ok(resp.results)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct QuickGoAnnotationResponse {
    #[serde(default)]
    pub results: Vec<QuickGoAnnotation>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickGoAnnotation {
    pub go_id: Option<String>,
    pub go_name: Option<String>,
    pub go_aspect: Option<String>,
    pub evidence_code: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QuickGoTermsResponse {
    #[serde(default)]
    pub results: Vec<QuickGoTerm>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QuickGoTerm {
    pub id: Option<String>,
    pub name: Option<String>,
    pub aspect: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn annotations_sets_expected_query_params() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/annotation/search"))
            .and(query_param("geneProductId", "P15056"))
            .and(query_param("limit", "5"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [{
                    "goId": "GO:0004672",
                    "goName": "protein kinase activity",
                    "goAspect": "molecular_function",
                    "evidenceCode": "ECO:0000269"
                }]
            })))
            .mount(&server)
            .await;

        let client = QuickGoClient::new_for_test(server.uri()).unwrap();
        let rows = client.annotations("P15056", 5).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].go_id.as_deref(), Some("GO:0004672"));
    }

    #[tokio::test]
    async fn annotations_rejects_empty_gene_product_id() {
        let client = QuickGoClient::new_for_test("http://127.0.0.1".into()).unwrap();
        let err = client.annotations("   ", 5).await.unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    }

    #[tokio::test]
    async fn terms_maps_term_metadata() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/ontology/go/terms/GO:0004672,GO:0005524"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [
                    {"id": "GO:0004672", "name": "protein kinase activity", "aspect": "molecular_function"},
                    {"id": "GO:0005524", "name": "ATP binding", "aspect": "molecular_function"}
                ]
            })))
            .mount(&server)
            .await;

        let client = QuickGoClient::new_for_test(server.uri()).unwrap();
        let rows = client
            .terms(&["GO:0004672".into(), "GO:0005524".into()])
            .await
            .unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].id.as_deref(), Some("GO:0004672"));
        assert_eq!(rows[0].name.as_deref(), Some("protein kinase activity"));
    }
}
