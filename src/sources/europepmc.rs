use std::borrow::Cow;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::BioMcpError;

const EUROPE_PMC_BASE: &str = "https://www.ebi.ac.uk/europepmc/webservices/rest";
const EUROPE_PMC_API: &str = "europepmc";
const EUROPE_PMC_BASE_ENV: &str = "BIOMCP_EUROPEPMC_BASE";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum EuropePmcSort {
    Date,
    Citations,
    #[default]
    Relevance,
}

#[derive(Clone)]
pub struct EuropePmcClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl EuropePmcClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(EUROPE_PMC_BASE, EUROPE_PMC_BASE_ENV),
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
        let bytes = crate::sources::read_limited_body(resp, EUROPE_PMC_API).await?;
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: EUROPE_PMC_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }
        serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
            api: EUROPE_PMC_API.to_string(),
            source,
        })
    }

    pub async fn search_by_doi(&self, doi: &str) -> Result<EuropePmcSearchResponse, BioMcpError> {
        let doi = doi.trim();
        if doi.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "DOI is required. Example: biomcp get article 10.1056/NEJMoa1203421".into(),
            ));
        }
        if doi.len() > 256 {
            return Err(BioMcpError::InvalidArgument("DOI is too long.".into()));
        }

        self.search_query(&format!("DOI:{doi}"), 1, 1).await
    }

    pub async fn search_by_pmcid(
        &self,
        pmcid: &str,
    ) -> Result<EuropePmcSearchResponse, BioMcpError> {
        let pmcid = pmcid.trim();
        if pmcid.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "PMCID is required. Example: biomcp get article PMC9984800".into(),
            ));
        }
        if pmcid.len() > 64 {
            return Err(BioMcpError::InvalidArgument("PMCID is too long.".into()));
        }

        let (prefix, rest) = pmcid.split_at(3.min(pmcid.len()));
        if !prefix.eq_ignore_ascii_case("PMC")
            || rest.is_empty()
            || !rest.chars().all(|c| c.is_ascii_digit())
        {
            return Err(BioMcpError::InvalidArgument(
                "PMCID must start with PMC and contain only digits after. Example: biomcp get article PMC9984800"
                    .into(),
            ));
        }

        let normalized = format!("PMC{rest}");
        self.search_query(&format!("PMCID:{normalized}"), 1, 1)
            .await
    }

    pub async fn search_by_pmid(&self, pmid: &str) -> Result<EuropePmcSearchResponse, BioMcpError> {
        let pmid = pmid.trim();
        if pmid.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "PMID is required. Example: biomcp get article 22663011".into(),
            ));
        }
        if pmid.len() > 32 || !pmid.chars().all(|c| c.is_ascii_digit()) {
            return Err(BioMcpError::InvalidArgument(
                "PMID must be numeric. Example: biomcp get article 22663011".into(),
            ));
        }
        self.search_query(&format!("EXT_ID:{pmid} AND SRC:MED"), 1, 1)
            .await
    }

    pub async fn search_query(
        &self,
        query: &str,
        page: usize,
        page_size: usize,
    ) -> Result<EuropePmcSearchResponse, BioMcpError> {
        self.search_query_with_sort(query, page, page_size, EuropePmcSort::Relevance)
            .await
    }

    pub async fn search_query_with_sort(
        &self,
        query: &str,
        page: usize,
        page_size: usize,
        sort: EuropePmcSort,
    ) -> Result<EuropePmcSearchResponse, BioMcpError> {
        let query = query.trim();
        if query.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Query is required for Europe PMC search".into(),
            ));
        }
        if query.len() > 2048 {
            return Err(BioMcpError::InvalidArgument(
                "Query is too long for Europe PMC search".into(),
            ));
        }
        if page == 0 {
            return Err(BioMcpError::InvalidArgument(
                "Europe PMC page must be >= 1".into(),
            ));
        }
        if page_size == 0 || page_size > 100 {
            return Err(BioMcpError::InvalidArgument(
                "Europe PMC page size must be between 1 and 100".into(),
            ));
        }

        let url = self.endpoint("search");
        let page = page.to_string();
        let page_size = page_size.to_string();
        let mut req = self.client.get(&url).query(&[
            ("query", query),
            ("format", "json"),
            ("page", page.as_str()),
            ("pageSize", page_size.as_str()),
        ]);
        req = match sort {
            EuropePmcSort::Date => req.query(&[("sort", "P_PDATE_D desc")]),
            EuropePmcSort::Citations => req.query(&[("sort", "CITED desc")]),
            EuropePmcSort::Relevance => req,
        };
        self.get_json(req).await
    }

    pub async fn get_full_text_xml(
        &self,
        source: &str,
        id: &str,
    ) -> Result<Option<String>, BioMcpError> {
        let source = source.trim();
        let id = id.trim();
        if source.is_empty() || id.is_empty() {
            return Ok(None);
        }

        let url = self.endpoint(&format!("{source}/{id}/fullTextXML"));
        let resp = crate::sources::apply_cache_mode(self.client.get(&url))
            .send()
            .await?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        let status = resp.status();
        let bytes = crate::sources::read_limited_body(resp, EUROPE_PMC_API).await?;
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: EUROPE_PMC_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }

        Ok(Some(String::from_utf8_lossy(&bytes).to_string()))
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EuropePmcSearchResponse {
    #[serde(rename = "hitCount")]
    pub hit_count: Option<u64>,
    #[serde(rename = "resultList")]
    pub result_list: Option<EuropePmcResultList>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EuropePmcResultList {
    #[serde(default)]
    pub result: Vec<EuropePmcResult>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EuropePmcResult {
    pub id: Option<String>,
    pub title: Option<String>,
    pub pmid: Option<String>,
    pub pmcid: Option<String>,
    pub doi: Option<String>,
    #[serde(rename = "journalTitle")]
    pub journal_title: Option<String>,
    #[serde(rename = "firstPublicationDate")]
    pub first_publication_date: Option<String>,
    #[serde(rename = "authorString")]
    pub author_string: Option<String>,
    #[serde(rename = "pubYear")]
    pub pub_year: Option<String>,
    #[serde(rename = "citedByCount")]
    pub cited_by_count: Option<serde_json::Value>,
    #[serde(rename = "pubType")]
    pub pub_type: Option<serde_json::Value>,
    #[serde(rename = "pubTypeList")]
    pub pub_type_list: Option<serde_json::Value>,
    #[serde(rename = "isOpenAccess")]
    pub is_open_access: Option<serde_json::Value>,
    #[serde(rename = "abstractText")]
    pub abstract_text: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn search_query_sets_expected_params() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/search"))
            .and(query_param("query", "EXT_ID:22663011 AND SRC:MED"))
            .and(query_param("format", "json"))
            .and(query_param("page", "1"))
            .and(query_param("pageSize", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "hitCount": 1,
                "resultList": {"result": [{"id": "22663011"}]}
            })))
            .mount(&server)
            .await;

        let client = EuropePmcClient::new_for_test(server.uri()).unwrap();
        let resp = client.search_by_pmid("22663011").await.unwrap();
        assert_eq!(resp.hit_count, Some(1));
    }

    #[tokio::test]
    async fn search_query_with_sort_sets_sort_param() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/search"))
            .and(query_param("query", "BRAF"))
            .and(query_param("format", "json"))
            .and(query_param("page", "1"))
            .and(query_param("pageSize", "5"))
            .and(query_param("sort", "CITED desc"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "hitCount": 0,
                "resultList": {"result": []}
            })))
            .mount(&server)
            .await;

        let client = EuropePmcClient::new_for_test(server.uri()).unwrap();
        let _ = client
            .search_query_with_sort("BRAF", 1, 5, EuropePmcSort::Citations)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn search_by_pmid_rejects_non_numeric_values() {
        let client = EuropePmcClient::new_for_test("http://127.0.0.1".into()).unwrap();
        let err = client.search_by_pmid("PMID226").await.unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    }

    #[tokio::test]
    async fn get_full_text_xml_returns_none_on_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/MED/22663011/fullTextXML"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        let client = EuropePmcClient::new_for_test(server.uri()).unwrap();
        let xml = client.get_full_text_xml("MED", "22663011").await.unwrap();
        assert!(xml.is_none());
    }
}
