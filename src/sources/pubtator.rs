use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

const PUBTATOR_BASE: &str = "https://www.ncbi.nlm.nih.gov/research/pubtator3-api";
const PUBTATOR_BASE_ENV: &str = "BIOMCP_PUBTATOR_BASE";

// dead-code reason: pubtator::PubTatorSearchRequestPlan preserves the provider shape used by source contract fixtures
#[allow(dead_code)]
pub struct PubTatorSearchRequestPlan {
    pub method: &'static str,
    pub path: &'static str,
    pub query_params: Vec<(&'static str, String)>,
    pub cache_mode: &'static str,
    pub status_expectation: &'static str,
    pub content_type_expectation: &'static str,
    pub auth_mode: &'static str,
}

// dead-code reason: pubtator::PubTatorExportRequestPlan preserves the provider shape used by source contract fixtures
#[allow(dead_code)]
pub struct PubTatorExportRequestPlan {
    pub method: &'static str,
    pub path: &'static str,
    pub query_params: Vec<(&'static str, String)>,
    pub cache_mode: &'static str,
    pub status_expectation: &'static str,
    pub content_type_expectation: &'static str,
    pub auth_mode: &'static str,
}

// dead-code reason: pubtator::PubTatorAutocompleteRequestPlan preserves the provider shape used by source contract fixtures
#[allow(dead_code)]
pub struct PubTatorAutocompleteRequestPlan {
    pub method: &'static str,
    pub path: &'static str,
    pub query_params: Vec<(&'static str, String)>,
    pub cache_mode: &'static str,
    pub status_expectation: &'static str,
    pub content_type_expectation: &'static str,
    pub auth_mode: &'static str,
}

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

    async fn get_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
        authenticated: bool,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode_with_auth(req, authenticated)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::PUBTATOR3,
            ))
            .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::PUBTATOR3),
        )
        .await?;
        crate::sources::decode_json(
            crate::error::SourceContext::retry(crate::error::SourceProvider::PUBTATOR3),
            status,
            content_type.as_ref(),
            &bytes,
            true,
        )
    }

    pub fn export_biocjson_plan(pmid: u32, api_key: Option<&str>) -> RequestPlan {
        let mut plan =
            RequestPlan::get("publications/export/biocjson").query("pmids", pmid.to_string());
        if let Some(key) = clean_api_key(api_key) {
            plan = plan.query("api_key", key);
        }
        plan
    }

    // dead-code reason: pubtator::export_biocjson_request_plan preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub fn export_biocjson_request_plan(&self, pmid: u32) -> PubTatorExportRequestPlan {
        PubTatorExportRequestPlan {
            method: "GET",
            path: "/publications/export/biocjson",
            query_params: vec![("pmids", pmid.to_string())],
            cache_mode: if self.api_key.is_some() {
                "auth"
            } else {
                "default"
            },
            status_expectation: "non-2xx => Api",
            content_type_expectation: "json",
            auth_mode: if self.api_key.is_some() {
                "authenticated"
            } else {
                "keyless"
            },
        }
    }

    pub async fn export_biocjson(&self, pmid: u32) -> Result<PubTatorExportResponse, BioMcpError> {
        let authenticated = self.api_key.is_some();
        let plan = Self::export_biocjson_plan(pmid, self.api_key.as_deref());
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        self.get_json(req, authenticated).await
    }

    pub fn entity_autocomplete_plan(
        query: &str,
        api_key: Option<&str>,
    ) -> Result<RequestPlan, BioMcpError> {
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

        let mut plan = RequestPlan::get("entity/autocomplete/").query("query", query);
        if let Some(key) = clean_api_key(api_key) {
            plan = plan.query("api_key", key);
        }
        Ok(plan)
    }

    // dead-code reason: pubtator::entity_autocomplete_request_plan preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub fn entity_autocomplete_request_plan(
        &self,
        query: &str,
    ) -> Result<PubTatorAutocompleteRequestPlan, BioMcpError> {
        let plan = Self::entity_autocomplete_plan(query, self.api_key.as_deref())?;
        Ok(PubTatorAutocompleteRequestPlan {
            method: "GET",
            path: "/entity/autocomplete/",
            query_params: plan
                .query
                .into_iter()
                .filter(|(key, _)| key != "api_key")
                .map(|(key, value)| (pubtator_query_key(&key), value))
                .collect(),
            cache_mode: if self.api_key.is_some() {
                "auth"
            } else {
                "default"
            },
            status_expectation: "non-2xx => Api",
            content_type_expectation: "json",
            auth_mode: if self.api_key.is_some() {
                "authenticated"
            } else {
                "keyless"
            },
        })
    }

    pub async fn entity_autocomplete(
        &self,
        query: &str,
    ) -> Result<Vec<PubTatorAutocompleteResult>, BioMcpError> {
        let authenticated = self.api_key.is_some();
        let plan = Self::entity_autocomplete_plan(query, self.api_key.as_deref())?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        self.get_json(req, authenticated).await
    }

    pub fn search_plan(
        text: &str,
        page: usize,
        size: usize,
        sort: Option<&str>,
        api_key: Option<&str>,
    ) -> Result<RequestPlan, BioMcpError> {
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

        let mut query_params = vec![
            ("text", text.to_string()),
            ("page", page.to_string()),
            ("size", size.to_string()),
        ];
        if let Some(sort) = sort.map(str::trim).filter(|value| !value.is_empty()) {
            query_params.push(("sort", sort.to_string()));
        }
        if let Some(key) = clean_api_key(api_key) {
            query_params.push(("api_key", key.to_string()));
        }
        let mut plan = RequestPlan::get("search/");
        for (key, value) in query_params {
            plan = plan.query(key, value);
        }
        Ok(plan)
    }

    // dead-code reason: pubtator::search_request_plan preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub fn search_request_plan(
        &self,
        text: &str,
        page: usize,
        size: usize,
        sort: Option<&str>,
    ) -> Result<PubTatorSearchRequestPlan, BioMcpError> {
        let plan = Self::search_plan(text, page, size, sort, self.api_key.as_deref())?;
        Ok(PubTatorSearchRequestPlan {
            method: "GET",
            path: "/search/",
            query_params: plan
                .query
                .into_iter()
                .filter(|(key, _)| key != "api_key")
                .map(|(key, value)| (pubtator_query_key(&key), value))
                .collect(),
            cache_mode: if self.api_key.is_some() {
                "auth"
            } else {
                "default"
            },
            status_expectation: "non-2xx => Api",
            content_type_expectation: "json",
            auth_mode: if self.api_key.is_some() {
                "authenticated"
            } else {
                "keyless"
            },
        })
    }

    pub async fn search(
        &self,
        text: &str,
        page: usize,
        size: usize,
        sort: Option<&str>,
    ) -> Result<PubTatorSearchResponse, BioMcpError> {
        let authenticated = self.api_key.is_some();
        let plan = Self::search_plan(text, page, size, sort, self.api_key.as_deref())?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        self.get_json(req, authenticated).await
    }
}

fn clean_api_key(api_key: Option<&str>) -> Option<&str> {
    api_key.map(str::trim).filter(|key| !key.is_empty())
}

// dead-code reason: pubtator::pubtator_query_key preserves the provider shape used by source contract fixtures
#[allow(dead_code)]
fn pubtator_query_key(key: &str) -> &'static str {
    match key {
        "pmids" => "pmids",
        "query" => "query",
        "text" => "text",
        "page" => "page",
        "size" => "size",
        "sort" => "sort",
        "api_key" => "api_key",
        _ => unreachable!("unexpected PubTator query key: {key}"),
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
    pub id: Option<String>,
    pub pmid: Option<u32>,
    pub pmcid: Option<String>,
    pub date: Option<String>,
    pub journal: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub passages: Vec<PubTatorPassage>,
    #[serde(default)]
    pub relations: Vec<PubTatorRelation>,
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
    pub id: Option<String>,
    pub text: Option<String>,
    pub infons: Option<PubTatorAnnotationInfons>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PubTatorRelation {
    pub id: Option<String>,
    pub infons: Option<serde_json::Value>,
    #[serde(default)]
    pub nodes: Vec<PubTatorRelationNode>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PubTatorRelationNode {
    pub refid: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PubTatorNormalizedId {
    String(String),
    Number(u64),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PubTatorAnnotationInfons {
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub name: Option<String>,
    pub identifier: Option<String>,
    pub normalized_id: Option<PubTatorNormalizedId>,
    pub hgvs: Option<String>,
    pub gene_id: Option<u64>,
    pub gene_ids: Option<Vec<u64>>,
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
mod tests;
