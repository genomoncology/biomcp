use std::borrow::Cow;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};

use crate::error::BioMcpError;

const PUBTATOR_BASE: &str = "https://www.ncbi.nlm.nih.gov/research/pubtator3-api";
const PUBTATOR_API: &str = "pubtator3";
const PUBTATOR_BASE_ENV: &str = "BIOMCP_PUBTATOR_BASE";

#[derive(Clone)]
pub struct PubTatorClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
    api_key: Option<String>,
}

impl PubTatorClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(PUBTATOR_BASE, PUBTATOR_BASE_ENV),
            api_key: crate::sources::ncbi_api_key(),
        })
    }

    #[cfg(test)]
    fn new_for_test(base: String, api_key: Option<String>) -> Result<Self, BioMcpError> {
        Ok(Self {
            client: Self::test_client()?,
            base: Cow::Owned(base),
            api_key: api_key
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty()),
        })
    }

    #[cfg(test)]
    fn test_client() -> Result<reqwest_middleware::ClientWithMiddleware, BioMcpError> {
        let base = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .connect_timeout(std::time::Duration::from_secs(5))
            .user_agent(concat!("biomcp-cli-test/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(BioMcpError::HttpClientInit)?;
        Ok(reqwest_middleware::ClientBuilder::new(base).build())
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
        let resp = crate::sources::apply_cache_mode_with_auth(req, self.api_key.is_some())
            .send()
            .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_body(resp, PUBTATOR_API).await?;
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: PUBTATOR_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }
        crate::sources::ensure_json_content_type(PUBTATOR_API, content_type.as_ref(), &bytes)?;
        serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
            api: PUBTATOR_API.to_string(),
            source,
        })
    }

    pub async fn export_biocjson(&self, pmid: u32) -> Result<PubTatorExportResponse, BioMcpError> {
        let url = self.endpoint("publications/export/biocjson");
        let pmids = pmid.to_string();
        let req = self.client.get(&url).query(&[("pmids", pmids.as_str())]);
        let req = crate::sources::append_ncbi_api_key(req, self.api_key.as_deref());
        self.get_json(req).await
    }

    pub async fn entity_autocomplete(
        &self,
        query: &str,
    ) -> Result<Vec<PubTatorAutocompleteResult>, BioMcpError> {
        let query = query.trim();
        if query.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Query is required for PubTator autocomplete".into(),
            ));
        }
        if query.len() > 256 {
            return Err(BioMcpError::InvalidArgument(
                "Query is too long for PubTator autocomplete".into(),
            ));
        }

        let url = self.endpoint("entity/autocomplete/");
        let req = self.client.get(&url).query(&[("query", query)]);
        let req = crate::sources::append_ncbi_api_key(req, self.api_key.as_deref());
        self.get_json(req).await
    }

    pub async fn search(
        &self,
        text: &str,
        page: usize,
        size: usize,
        sort: Option<&str>,
    ) -> Result<PubTatorSearchResponse, BioMcpError> {
        let text = text.trim();
        if text.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Text is required for PubTator search".into(),
            ));
        }
        if text.len() > 4096 {
            return Err(BioMcpError::InvalidArgument(
                "Text is too long for PubTator search".into(),
            ));
        }
        if page == 0 {
            return Err(BioMcpError::InvalidArgument(
                "PubTator page must be >= 1".into(),
            ));
        }
        if size == 0 || size > 100 {
            return Err(BioMcpError::InvalidArgument(
                "PubTator size must be between 1 and 100".into(),
            ));
        }

        let url = self.endpoint("search/");
        let page = page.to_string();
        let size = size.to_string();
        let mut req = self.client.get(&url).query(&[
            ("text", text),
            ("page", page.as_str()),
            ("size", size.as_str()),
        ]);
        if let Some(sort) = sort.map(str::trim).filter(|value| !value.is_empty()) {
            req = req.query(&[("sort", sort)]);
        }
        let req = crate::sources::append_ncbi_api_key(req, self.api_key.as_deref());
        self.get_json(req).await
    }
}

fn deserialize_option_string_or_number<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    let Some(value) = value else {
        return Ok(None);
    };
    let out = match value {
        serde_json::Value::String(v) => {
            let v = v.trim();
            if v.is_empty() {
                None
            } else {
                Some(v.to_string())
            }
        }
        serde_json::Value::Number(v) => Some(v.to_string()),
        _ => None,
    };
    Ok(out)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PubTatorExportResponse {
    #[serde(rename = "PubTator3", default)]
    pub documents: Vec<PubTatorDocument>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PubTatorDocument {
    pub pmid: Option<u32>,
    pub pmcid: Option<String>,
    pub date: Option<String>,
    pub journal: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub passages: Vec<PubTatorPassage>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PubTatorPassage {
    pub infons: Option<PubTatorInfons>,
    pub text: Option<String>,
    #[serde(default)]
    pub annotations: Vec<PubTatorAnnotation>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PubTatorInfons {
    #[serde(rename = "type")]
    pub kind: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PubTatorAnnotation {
    pub text: Option<String>,
    pub infons: Option<PubTatorAnnotationInfons>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PubTatorAnnotationInfons {
    #[serde(rename = "type")]
    pub kind: Option<String>,
    #[allow(dead_code)]
    pub identifier: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PubTatorAutocompleteResult {
    #[serde(rename = "_id")]
    pub id: Option<String>,
    pub biotype: Option<String>,
    pub db_id: Option<String>,
    pub db: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PubTatorSearchResponse {
    #[serde(default)]
    pub results: Vec<PubTatorSearchResult>,
    pub count: Option<u64>,
    pub total_pages: Option<u64>,
    pub current: Option<u64>,
    pub page_size: Option<u64>,
    #[serde(default)]
    pub facets: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PubTatorSearchResult {
    #[serde(rename = "_id")]
    pub id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_string_or_number")]
    pub pmid: Option<String>,
    pub pmcid: Option<String>,
    pub title: Option<String>,
    pub journal: Option<String>,
    pub date: Option<String>,
    pub score: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn export_biocjson_sets_pmids_query_param() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/publications/export/biocjson"))
            .and(query_param("pmids", "22663011"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "PubTator3": [{
                    "pmid": 22663011,
                    "passages": []
                }]
            })))
            .mount(&server)
            .await;

        let client = PubTatorClient::new_for_test(server.uri(), None).unwrap();
        let resp = client.export_biocjson(22663011).await.unwrap();
        assert_eq!(resp.documents.len(), 1);
        assert_eq!(resp.documents[0].pmid, Some(22663011));
    }

    #[tokio::test]
    async fn export_biocjson_includes_api_key_when_configured() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/publications/export/biocjson"))
            .and(query_param("pmids", "22663011"))
            .and(query_param("api_key", "test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "PubTator3": [{
                    "pmid": 22663011,
                    "passages": []
                }]
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = PubTatorClient::new_for_test(server.uri(), Some("test-key".into())).unwrap();
        let resp = client.export_biocjson(22663011).await.unwrap();
        assert_eq!(resp.documents[0].pmid, Some(22663011));
    }

    #[tokio::test]
    async fn export_biocjson_surfaces_http_error_context() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/publications/export/biocjson"))
            .respond_with(ResponseTemplate::new(500).set_body_string("upstream failure"))
            .expect(1)
            .mount(&server)
            .await;

        let client = PubTatorClient::new_for_test(server.uri(), None).unwrap();
        let err = client.export_biocjson(22663011).await.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("pubtator3"));
        assert!(msg.contains("500"));
    }

    #[tokio::test]
    async fn entity_autocomplete_sets_expected_params() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/entity/autocomplete/"))
            .and(query_param("query", "BRAF"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {
                    "_id": "@GENE_BRAF",
                    "biotype": "gene",
                    "db_id": "673",
                    "name": "BRAF"
                }
            ])))
            .expect(1)
            .mount(&server)
            .await;

        let client = PubTatorClient::new_for_test(server.uri(), None).unwrap();
        let resp = client.entity_autocomplete("BRAF").await.unwrap();
        assert_eq!(resp.len(), 1);
        assert_eq!(resp[0].id.as_deref(), Some("@GENE_BRAF"));
    }

    #[tokio::test]
    async fn search_sets_expected_params_and_sort() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/search/"))
            .and(query_param("text", "@GENE_BRAF"))
            .and(query_param("page", "2"))
            .and(query_param("size", "25"))
            .and(query_param("sort", "date desc"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [{
                    "_id": "123",
                    "pmid": 123,
                    "title": "BRAF",
                    "journal": "Test Journal",
                    "date": "2024-01-01T00:00:00Z",
                    "score": 42.5
                }],
                "count": 1,
                "total_pages": 1,
                "current": 1,
                "page_size": 25
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = PubTatorClient::new_for_test(server.uri(), None).unwrap();
        let resp = client
            .search("@GENE_BRAF", 2, 25, Some("date desc"))
            .await
            .unwrap();
        assert_eq!(resp.results.len(), 1);
        assert_eq!(resp.results[0].pmid.as_deref(), Some("123"));
        assert_eq!(resp.count, Some(1));
    }

    #[tokio::test]
    async fn search_includes_api_key_when_configured() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/search/"))
            .and(query_param("text", "melanoma"))
            .and(query_param("page", "1"))
            .and(query_param("size", "25"))
            .and(query_param("api_key", "test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [],
                "count": 0,
                "total_pages": 0,
                "current": 1,
                "page_size": 25
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = PubTatorClient::new_for_test(server.uri(), Some("test-key".into())).unwrap();
        let resp = client.search("melanoma", 1, 25, None).await.unwrap();
        assert_eq!(resp.count, Some(0));
    }
}
