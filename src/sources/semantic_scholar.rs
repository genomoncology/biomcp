use std::borrow::Cow;

use reqwest::Url;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::BioMcpError;

const SEMANTIC_SCHOLAR_BASE: &str = "https://api.semanticscholar.org";
const SEMANTIC_SCHOLAR_API: &str = "semantic_scholar";
const SEMANTIC_SCHOLAR_BASE_ENV: &str = "BIOMCP_S2_BASE";
const SEMANTIC_SCHOLAR_DOCS_URL: &str = "https://www.semanticscholar.org/product/api";
const GRAPH_PAPER_FIELDS: &str = "paperId,externalIds,title,venue,year,tldr,citationCount,influentialCitationCount,referenceCount,isOpenAccess,openAccessPdf";
const BATCH_PAPER_FIELDS: &str = "paperId,externalIds,title,venue,year";
const SEARCH_PAPER_FIELDS: &str =
    "paperId,externalIds,title,venue,year,citationCount,influentialCitationCount,abstract";
const CITATION_EDGE_FIELDS: &str = "contexts,intents,isInfluential,citingPaper.paperId,citingPaper.externalIds,citingPaper.title,citingPaper.venue,citingPaper.year";
const REFERENCE_EDGE_FIELDS: &str = "contexts,intents,isInfluential,citedPaper.paperId,citedPaper.externalIds,citedPaper.title,citedPaper.venue,citedPaper.year";
const RECOMMENDATION_FIELDS: &str = "paperId,externalIds,title,venue,year";

#[derive(Clone)]
pub struct SemanticScholarClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
    api_key: Option<String>,
}

impl SemanticScholarClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(SEMANTIC_SCHOLAR_BASE, SEMANTIC_SCHOLAR_BASE_ENV),
            api_key: std::env::var("S2_API_KEY")
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
        })
    }

    #[cfg(test)]
    fn new_for_test(base: String, api_key: Option<String>) -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: Cow::Owned(base),
            api_key: api_key
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
        })
    }

    pub fn is_configured(&self) -> bool {
        self.api_key.is_some()
    }

    pub fn api_key_required() -> BioMcpError {
        BioMcpError::ApiKeyRequired {
            api: "Semantic Scholar".to_string(),
            env_var: "S2_API_KEY".to_string(),
            docs_url: SEMANTIC_SCHOLAR_DOCS_URL.to_string(),
        }
    }

    fn require_api_key(&self) -> Result<&str, BioMcpError> {
        self.api_key.as_deref().ok_or_else(Self::api_key_required)
    }

    fn endpoint_url(&self, path: &str) -> Result<Url, BioMcpError> {
        Url::parse(&format!(
            "{}/{}",
            self.base.as_ref().trim_end_matches('/'),
            path.trim_start_matches('/')
        ))
        .map_err(|err| BioMcpError::Api {
            api: SEMANTIC_SCHOLAR_API.to_string(),
            message: format!("invalid Semantic Scholar base URL: {err}"),
        })
    }

    fn paper_url(&self, id: &str) -> Result<Url, BioMcpError> {
        let id = validate_paper_id(id)?;
        let mut url = self.endpoint_url("graph/v1/paper")?;
        {
            let mut segments = url.path_segments_mut().map_err(|_| BioMcpError::Api {
                api: SEMANTIC_SCHOLAR_API.to_string(),
                message: "invalid Semantic Scholar graph URL".to_string(),
            })?;
            segments.push(id);
        }
        Ok(url)
    }

    fn paper_subresource_url(&self, id: &str, subresource: &str) -> Result<Url, BioMcpError> {
        let id = validate_paper_id(id)?;
        let mut url = self.endpoint_url("graph/v1/paper")?;
        {
            let mut segments = url.path_segments_mut().map_err(|_| BioMcpError::Api {
                api: SEMANTIC_SCHOLAR_API.to_string(),
                message: "invalid Semantic Scholar graph URL".to_string(),
            })?;
            segments.push(id);
            segments.push(subresource);
        }
        Ok(url)
    }

    async fn send_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<T, BioMcpError> {
        self.require_api_key()?;
        let resp = crate::sources::apply_cache_mode_with_auth(req, true)
            .send()
            .await?;
        let status = resp.status();
        let bytes = crate::sources::read_limited_body(resp, SEMANTIC_SCHOLAR_API).await?;
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: SEMANTIC_SCHOLAR_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }
        serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
            api: SEMANTIC_SCHOLAR_API.to_string(),
            source,
        })
    }

    fn with_auth(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<reqwest_middleware::RequestBuilder, BioMcpError> {
        let api_key = self.require_api_key()?;
        Ok(req.header("x-api-key", api_key))
    }

    pub async fn paper_detail(&self, id: &str) -> Result<SemanticScholarPaper, BioMcpError> {
        let url = self.paper_url(id)?;
        let req = self.with_auth(
            self.client
                .get(url)
                .query(&[("fields", GRAPH_PAPER_FIELDS)]),
        )?;
        self.send_json(req).await
    }

    pub async fn paper_batch(
        &self,
        ids: &[String],
    ) -> Result<Vec<Option<SemanticScholarPaper>>, BioMcpError> {
        if ids.is_empty() || ids.len() > 500 {
            return Err(BioMcpError::InvalidArgument(
                "Semantic Scholar batch lookup requires 1-500 paper IDs".into(),
            ));
        }
        let url = self.endpoint_url("graph/v1/paper/batch")?;
        let req = self.with_auth(
            self.client
                .post(url)
                .query(&[("fields", BATCH_PAPER_FIELDS)])
                .json(&SemanticScholarBatchRequest { ids }),
        )?;
        self.send_json(req).await
    }

    pub async fn paper_search(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<SemanticScholarSearchResponse, BioMcpError> {
        let query = query.trim();
        if query.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Semantic Scholar paper search query is required".into(),
            ));
        }
        let limit = validate_limit(limit)?;
        let url = self.endpoint_url("graph/v1/paper/search")?;
        let req = self.with_auth(self.client.get(url).query(&[
            ("query", query),
            ("fields", SEARCH_PAPER_FIELDS),
            ("limit", &limit.to_string()),
        ]))?;
        self.send_json(req).await
    }

    pub async fn paper_citations(
        &self,
        id: &str,
        limit: usize,
    ) -> Result<SemanticScholarGraphResponse<SemanticScholarCitationEdge>, BioMcpError> {
        let limit = validate_limit(limit)?;
        let url = self.paper_subresource_url(id, "citations")?;
        let req = self.with_auth(self.client.get(url).query(&[
            ("fields", CITATION_EDGE_FIELDS),
            ("limit", &limit.to_string()),
        ]))?;
        self.send_json(req).await
    }

    pub async fn paper_references(
        &self,
        id: &str,
        limit: usize,
    ) -> Result<SemanticScholarGraphResponse<SemanticScholarReferenceEdge>, BioMcpError> {
        let limit = validate_limit(limit)?;
        let url = self.paper_subresource_url(id, "references")?;
        let req = self.with_auth(self.client.get(url).query(&[
            ("fields", REFERENCE_EDGE_FIELDS),
            ("limit", &limit.to_string()),
        ]))?;
        self.send_json(req).await
    }

    pub async fn recommendations_for_paper(
        &self,
        paper_id: &str,
        limit: usize,
    ) -> Result<SemanticScholarRecommendationsResponse, BioMcpError> {
        let paper_id = validate_paper_id(paper_id)?;
        let limit = validate_limit(limit)?;
        let mut url = self.endpoint_url("recommendations/v1/papers/forpaper")?;
        {
            let mut segments = url.path_segments_mut().map_err(|_| BioMcpError::Api {
                api: SEMANTIC_SCHOLAR_API.to_string(),
                message: "invalid Semantic Scholar recommendations URL".to_string(),
            })?;
            segments.push(paper_id);
        }
        let req = self.with_auth(self.client.get(url).query(&[
            ("fields", RECOMMENDATION_FIELDS),
            ("limit", &limit.to_string()),
        ]))?;
        self.send_json(req).await
    }

    pub async fn recommendations(
        &self,
        positive_paper_ids: &[String],
        negative_paper_ids: &[String],
        limit: usize,
    ) -> Result<SemanticScholarRecommendationsResponse, BioMcpError> {
        if positive_paper_ids.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Semantic Scholar recommendations require at least one positive paper".into(),
            ));
        }
        let limit = validate_limit(limit)?;
        let url = self.endpoint_url("recommendations/v1/papers/")?;
        let req = self.with_auth(
            self.client
                .post(url)
                .query(&[
                    ("fields", RECOMMENDATION_FIELDS),
                    ("limit", &limit.to_string()),
                ])
                .json(&SemanticScholarRecommendationsRequest {
                    positive_paper_ids,
                    negative_paper_ids,
                }),
        )?;
        self.send_json(req).await
    }
}

fn deserialize_vec_or_default<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Option::<Vec<T>>::deserialize(deserializer)?.unwrap_or_default())
}

fn validate_paper_id(id: &str) -> Result<&str, BioMcpError> {
    let id = id.trim();
    if id.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Semantic Scholar paper ID is required".into(),
        ));
    }
    if id.len() > 512 {
        return Err(BioMcpError::InvalidArgument(
            "Semantic Scholar paper ID is too long".into(),
        ));
    }
    Ok(id)
}

fn validate_limit(limit: usize) -> Result<usize, BioMcpError> {
    if limit == 0 || limit > 100 {
        return Err(BioMcpError::InvalidArgument(
            "Semantic Scholar --limit must be between 1 and 100".into(),
        ));
    }
    Ok(limit)
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SemanticScholarPaper {
    #[serde(rename = "paperId")]
    pub paper_id: Option<String>,
    #[serde(rename = "externalIds")]
    pub external_ids: Option<SemanticScholarExternalIds>,
    pub title: Option<String>,
    pub venue: Option<String>,
    pub year: Option<u32>,
    #[serde(rename = "citationCount")]
    pub citation_count: Option<u64>,
    #[serde(rename = "influentialCitationCount")]
    pub influential_citation_count: Option<u64>,
    #[serde(rename = "abstract", skip_serializing_if = "Option::is_none")]
    pub abstract_text: Option<String>,
    #[serde(rename = "referenceCount")]
    pub reference_count: Option<u64>,
    #[serde(rename = "isOpenAccess")]
    pub is_open_access: Option<bool>,
    #[serde(rename = "openAccessPdf")]
    pub open_access_pdf: Option<SemanticScholarOpenAccessPdf>,
    pub tldr: Option<SemanticScholarTldr>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SemanticScholarSearchResponse {
    pub total: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_vec_or_default")]
    pub data: Vec<SemanticScholarPaper>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SemanticScholarExternalIds {
    #[serde(rename = "PubMed")]
    pub pubmed: Option<String>,
    #[serde(rename = "PubMedCentral")]
    pub pmcid: Option<String>,
    #[serde(rename = "DOI")]
    pub doi: Option<String>,
    #[serde(rename = "ArXiv")]
    pub arxiv: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SemanticScholarOpenAccessPdf {
    pub url: Option<String>,
    pub status: Option<String>,
    pub license: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SemanticScholarTldr {
    pub text: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Serialize)]
struct SemanticScholarBatchRequest<'a> {
    ids: &'a [String],
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
pub struct SemanticScholarGraphResponse<T> {
    #[serde(default, deserialize_with = "deserialize_vec_or_default")]
    pub data: Vec<T>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SemanticScholarCitationEdge {
    #[serde(default, deserialize_with = "deserialize_vec_or_default")]
    pub contexts: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_vec_or_default")]
    pub intents: Vec<String>,
    #[serde(rename = "isInfluential")]
    pub is_influential: Option<bool>,
    #[serde(rename = "citingPaper")]
    pub citing_paper: SemanticScholarPaper,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SemanticScholarReferenceEdge {
    #[serde(default, deserialize_with = "deserialize_vec_or_default")]
    pub contexts: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_vec_or_default")]
    pub intents: Vec<String>,
    #[serde(rename = "isInfluential")]
    pub is_influential: Option<bool>,
    #[serde(rename = "citedPaper")]
    pub cited_paper: SemanticScholarPaper,
}

#[derive(Debug, Serialize)]
struct SemanticScholarRecommendationsRequest<'a> {
    #[serde(rename = "positivePaperIds")]
    positive_paper_ids: &'a [String],
    #[serde(rename = "negativePaperIds")]
    negative_paper_ids: &'a [String],
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SemanticScholarRecommendationsResponse {
    #[serde(rename = "recommendedPapers", default)]
    pub recommended_papers: Vec<SemanticScholarPaper>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{body_string_contains, header, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn paper_detail_sends_api_key_header_and_fields() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/graph/v1/paper/PMID:22663011"))
            .and(query_param("fields", GRAPH_PAPER_FIELDS))
            .and(header("x-api-key", "test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "paperId": "paper-1",
                "title": "Example"
            })))
            .mount(&server)
            .await;

        let client =
            SemanticScholarClient::new_for_test(server.uri(), Some("test-key".to_string()))
                .unwrap();
        let paper = client.paper_detail("PMID:22663011").await.unwrap();
        assert_eq!(paper.paper_id.as_deref(), Some("paper-1"));
    }

    #[tokio::test]
    async fn paper_batch_posts_ids_and_parses_rows() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/graph/v1/paper/batch"))
            .and(query_param("fields", BATCH_PAPER_FIELDS))
            .and(header("x-api-key", "test-key"))
            .and(body_string_contains("\"PMID:22663011\""))
            .and(body_string_contains("\"PMID:24200969\""))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {"paperId": "paper-1", "title": "One"},
                {"paperId": "paper-2", "title": "Two"}
            ])))
            .mount(&server)
            .await;

        let client =
            SemanticScholarClient::new_for_test(server.uri(), Some("test-key".to_string()))
                .unwrap();
        let rows = client
            .paper_batch(&["PMID:22663011".to_string(), "PMID:24200969".to_string()])
            .await
            .unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[0].as_ref().and_then(|row| row.paper_id.as_deref()),
            Some("paper-1")
        );
    }

    #[tokio::test]
    async fn recommendations_post_uses_expected_shape() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/recommendations/v1/papers/"))
            .and(query_param("fields", RECOMMENDATION_FIELDS))
            .and(query_param("limit", "2"))
            .and(header("x-api-key", "test-key"))
            .and(body_string_contains("\"positivePaperIds\":[\"paper-1\"]"))
            .and(body_string_contains("\"negativePaperIds\":[\"paper-2\"]"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "recommendedPapers": [{"paperId": "paper-3", "title": "Three"}]
            })))
            .mount(&server)
            .await;

        let client =
            SemanticScholarClient::new_for_test(server.uri(), Some("test-key".to_string()))
                .unwrap();
        let response = client
            .recommendations(&["paper-1".to_string()], &["paper-2".to_string()], 2)
            .await
            .unwrap();
        assert_eq!(response.recommended_papers.len(), 1);
        assert_eq!(
            response.recommended_papers[0].paper_id.as_deref(),
            Some("paper-3")
        );
    }

    #[tokio::test]
    async fn paper_detail_requires_api_key() {
        let server = MockServer::start().await;
        let client = SemanticScholarClient::new_for_test(server.uri(), None).unwrap();

        let err = client.paper_detail("PMID:22663011").await.unwrap_err();
        assert!(matches!(err, BioMcpError::ApiKeyRequired { .. }));
    }

    #[tokio::test]
    async fn paper_search_sends_query_limit_and_parses_abstract_metadata() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/graph/v1/paper/search"))
            .and(query_param("query", "braf melanoma"))
            .and(query_param("fields", SEARCH_PAPER_FIELDS))
            .and(query_param("limit", "3"))
            .and(header("x-api-key", "test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "total": 1,
                "data": [{
                    "paperId": "paper-1",
                    "externalIds": {
                        "PubMed": "22663011",
                        "DOI": "10.1056/NEJMoa1203421"
                    },
                    "title": "BRAF melanoma response",
                    "citationCount": 12,
                    "influentialCitationCount": 4,
                    "abstract": "Direct answer abstract."
                }]
            })))
            .mount(&server)
            .await;

        let client =
            SemanticScholarClient::new_for_test(server.uri(), Some("test-key".to_string()))
                .unwrap();
        let response = client.paper_search("braf melanoma", 3).await.unwrap();
        assert_eq!(response.total, Some(1));
        assert_eq!(response.data.len(), 1);
        let paper = &response.data[0];
        assert_eq!(paper.paper_id.as_deref(), Some("paper-1"));
        assert_eq!(paper.influential_citation_count, Some(4));
        assert_eq!(
            paper.abstract_text.as_deref(),
            Some("Direct answer abstract.")
        );
    }
}
