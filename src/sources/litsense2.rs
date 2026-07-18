use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;

use reqwest_middleware::ClientWithMiddleware;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

const LITSENSE2_BASE: &str = "https://www.ncbi.nlm.nih.gov/research/litsense2-api/api";
const LITSENSE2_BASE_ENV: &str = "BIOMCP_LITSENSE2_BASE";

// dead-code reason: litsense2::LitSense2SearchRequestPlan preserves the provider shape used by source contract fixtures
#[allow(dead_code)]
pub struct LitSense2SearchRequestPlan {
    pub method: &'static str,
    pub path: &'static str,
    pub query_params: Vec<(&'static str, String)>,
    pub cache_mode: &'static str,
    pub status_expectation: &'static str,
    pub content_type_expectation: &'static str,
}

#[derive(Clone)]
pub struct LitSense2Client {
    client: ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl LitSense2Client {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(LITSENSE2_BASE, LITSENSE2_BASE_ENV),
        })
    }

    async fn send_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::LITSENSE2,
            ))
            .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::LITSENSE2),
        )
        .await?;
        crate::sources::decode_json(
            crate::error::SourceContext::retry(crate::error::SourceProvider::LITSENSE2),
            status,
            content_type.as_ref(),
            &bytes,
            true,
        )
    }

    pub(crate) fn search_plan(path: &str, query: &str) -> Result<RequestPlan, BioMcpError> {
        let query = query.trim();
        if query.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "LitSense2 query is required".into(),
            ));
        }
        if query.len() > 4096 {
            return Err(BioMcpError::InvalidArgument(
                "LitSense2 query is too long".into(),
            ));
        }
        let path = match path.trim_start_matches('/') {
            "sentences/" => "sentences/",
            "passages/" => "passages/",
            _ => {
                return Err(BioMcpError::InvalidArgument(
                    "LitSense2 path must be /sentences/ or /passages/".into(),
                ));
            }
        };

        Ok(RequestPlan::get(path)
            .query("query", query)
            .query("rerank", "true"))
    }

    // dead-code reason: litsense2::search_request_plan preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub fn search_request_plan(
        &self,
        path: &str,
        query: &str,
    ) -> Result<LitSense2SearchRequestPlan, BioMcpError> {
        let plan = Self::search_plan(path, query)?;
        Ok(LitSense2SearchRequestPlan {
            method: "GET",
            path: if plan.path == "sentences/" {
                "/sentences/"
            } else {
                "/passages/"
            },
            query_params: plan
                .query
                .iter()
                .map(|(key, value)| {
                    let key = match key.as_str() {
                        "query" => "query",
                        "rerank" => "rerank",
                        _ => "",
                    };
                    (key, value.clone())
                })
                .collect(),
            cache_mode: "default",
            status_expectation: "non-2xx => Api",
            content_type_expectation: "json",
        })
    }

    async fn search(
        &self,
        path: &str,
        query: &str,
    ) -> Result<Vec<LitSense2SearchHit>, BioMcpError> {
        let plan = Self::search_plan(path, query)?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        self.send_json(req).await
    }

    pub async fn sentence_search(
        &self,
        query: &str,
    ) -> Result<Vec<LitSense2SearchHit>, BioMcpError> {
        self.search("sentences/", query).await
    }
}

fn deserialize_optional_trimmed_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    Ok(value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty()))
}

fn deserialize_string_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Vec<String>>::deserialize(deserializer)?;
    Ok(value.unwrap_or_default())
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct LitSense2SearchHit {
    pub pmid: u64,
    #[serde(default, deserialize_with = "deserialize_optional_trimmed_string")]
    pub pmcid: Option<String>,
    pub text: String,
    pub score: f64,
    #[serde(default, deserialize_with = "deserialize_optional_trimmed_string")]
    pub section: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_vec")]
    pub annotations: Vec<String>,
}

#[cfg(test)]
mod tests;
