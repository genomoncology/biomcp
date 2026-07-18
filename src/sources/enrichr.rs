use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;

use serde::Deserialize;
use tracing::warn;

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

const ENRICHR_BASE: &str = "https://maayanlab.cloud/Enrichr";
const ENRICHR_API: &str = "enrichr";
const ENRICHR_BASE_ENV: &str = "BIOMCP_ENRICHR_BASE";

#[derive(Clone)]
pub struct EnrichrClient {
    client: reqwest_middleware::ClientWithMiddleware,
    streaming_client: reqwest::Client,
    base: Cow<'static, str>,
}

impl EnrichrClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            streaming_client: crate::sources::streaming_http_client()?,
            base: crate::sources::env_base(ENRICHR_BASE, ENRICHR_BASE_ENV),
        })
    }

    fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base.as_ref().trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    async fn send_bytes(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<
        (
            reqwest::StatusCode,
            Option<reqwest::header::HeaderValue>,
            Vec<u8>,
        ),
        BioMcpError,
    > {
        let resp = crate::sources::apply_cache_mode(req)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::ENRICHR,
            ))
            .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::ENRICHR),
        )
        .await?;
        Ok((status, content_type, bytes))
    }

    async fn send_bytes_streaming<F>(
        &self,
        build_request: F,
    ) -> Result<
        (
            reqwest::StatusCode,
            Option<reqwest::header::HeaderValue>,
            Vec<u8>,
        ),
        BioMcpError,
    >
    where
        F: Fn() -> reqwest::RequestBuilder,
    {
        let resp = crate::sources::retry_send(
            crate::error::SourceContext::retry(crate::error::SourceProvider::ENRICHR),
            3,
            || async { build_request().send().await },
        )
        .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::ENRICHR),
        )
        .await?;
        Ok((status, content_type, bytes))
    }

    pub(crate) fn add_list_body(genes: &[&str]) -> Result<String, BioMcpError> {
        if genes.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Gene list is required for enrichment.".into(),
            ));
        }
        if genes.len() > 1000 {
            return Err(BioMcpError::InvalidArgument(
                "Too many genes for enrichment.".into(),
            ));
        }

        let mut list = String::new();
        for g in genes {
            let g = g.trim();
            if g.is_empty() {
                continue;
            }
            if !list.is_empty() {
                list.push('\n');
            }
            list.push_str(g);
        }
        if list.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Gene list is required for enrichment.".into(),
            ));
        }
        Ok(list)
    }

    pub(crate) fn decode_add_list_response(
        status: reqwest::StatusCode,
        bytes: &[u8],
    ) -> Result<i64, BioMcpError> {
        if status.is_success() {
            let parsed: EnrichrAddListResponse =
                serde_json::from_slice(bytes).map_err(|source| BioMcpError::ApiJson {
                    api: ENRICHR_API.to_string(),
                    source,
                })?;
            return Ok(parsed.user_list_id);
        }

        let excerpt = crate::sources::body_excerpt(bytes);
        Err(BioMcpError::Api {
            api: ENRICHR_API.to_string(),
            message: format!("HTTP {status}: {excerpt}"),
        })
    }

    pub async fn add_list(&self, genes: &[&str]) -> Result<i64, BioMcpError> {
        let list = Self::add_list_body(genes)?;
        let url = self.endpoint("addList");
        crate::sources::rate_limit::wait_for_url_str(&url).await;
        let request_url = url.clone();
        let list_for_retry = list.clone();
        let (status, _content_type, bytes) = self
            // Enrichr uses a streaming multipart body for addList; bypass middleware because it
            // requires cloneable request bodies.
            .send_bytes_streaming(|| {
                let form = reqwest::multipart::Form::new()
                    .text("list", list_for_retry.clone())
                    .text("description", "biomcp-cli");
                self.streaming_client.post(&request_url).multipart(form)
            })
            .await?;

        Self::decode_add_list_response(status, &bytes).map_err(|error| {
            error.with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::ENRICHR,
            ))
        })
    }

    pub(crate) fn enrich_plan(user_list_id: i64, library: &str) -> RequestPlan {
        RequestPlan::get("enrich")
            .query("userListId", user_list_id.to_string())
            .query("backgroundType", library)
    }

    pub(crate) fn decode_enrich_response(
        status: reqwest::StatusCode,
        content_type: Option<&reqwest::header::HeaderValue>,
        bytes: &[u8],
    ) -> Result<serde_json::Value, BioMcpError> {
        if status.is_success() {
            crate::sources::ensure_json_content_type(
                crate::error::SourceContext::retry(crate::error::SourceProvider::ENRICHR),
                content_type,
                bytes,
            )?;
            return serde_json::from_slice(bytes).map_err(|source| BioMcpError::ApiJson {
                api: ENRICHR_API.to_string(),
                source,
            });
        }

        if status == reqwest::StatusCode::BAD_REQUEST {
            warn!(
                source = ENRICHR_API,
                status = %status,
                body = %crate::sources::body_excerpt(bytes),
                "Enrichr returned HTTP 400; degrading to empty enrichment payload"
            );
            return Ok(serde_json::Value::Object(serde_json::Map::new()));
        }

        let excerpt = crate::sources::body_excerpt(bytes);
        Err(BioMcpError::Api {
            api: ENRICHR_API.to_string(),
            message: format!("HTTP {status}: {excerpt}"),
        })
    }

    pub async fn enrich(
        &self,
        user_list_id: i64,
        library: &str,
    ) -> Result<serde_json::Value, BioMcpError> {
        let plan = Self::enrich_plan(user_list_id, library);
        let (status, content_type, bytes) = self
            .send_bytes(request_from_plan(&self.client, self.base.as_ref(), &plan))
            .await?;

        Self::decode_enrich_response(status, content_type.as_ref(), &bytes).map_err(|error| {
            error.with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::ENRICHR,
            ))
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct EnrichrAddListResponse {
    #[serde(rename = "userListId")]
    pub user_list_id: i64,
}

#[cfg(test)]
mod tests;
