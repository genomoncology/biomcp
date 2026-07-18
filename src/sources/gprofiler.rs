use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;
use std::time::Duration;

use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::BioMcpError;
use crate::sources::{RequestBody, RequestPlan};

const GPROFILER_BASE: &str = "https://biit.cs.ut.ee/gprofiler/api";
const GPROFILER_API: &str = "gprofiler";
const GPROFILER_BASE_ENV: &str = "BIOMCP_GPROFILER_BASE";
const GPROFILER_MAX_ENRICH_LIMIT: usize = 50;
const GPROFILER_TIMEOUT: Duration = Duration::from_secs(15);
const GPROFILER_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const GPROFILER_RETRY_SUGGESTION: &str = "Retry shortly. If the problem persists, probe https://biit.cs.ut.ee/gprofiler/api/gost/profile/ directly.";

pub struct GProfilerClient {
    client: reqwest::Client,
    base: Cow<'static, str>,
}

impl GProfilerClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: gprofiler_http_client(GPROFILER_TIMEOUT)?,
            base: crate::sources::env_base(GPROFILER_BASE, GPROFILER_BASE_ENV),
        })
    }

    fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base.as_ref().trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    async fn post_json<T: DeserializeOwned, B: Serialize>(
        &self,
        req: reqwest::RequestBuilder,
        body: &B,
    ) -> Result<T, BioMcpError> {
        let resp = req
            .json(body)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::GPROFILER,
            ))
            .await?;
        let status = resp.status();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::GPROFILER),
        )
        .await?;
        Self::decode_json_response(status, &bytes).map_err(|error| {
            error.with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::GPROFILER,
            ))
        })
    }

    pub(crate) fn decode_json_response<T: DeserializeOwned>(
        status: StatusCode,
        bytes: &[u8],
    ) -> Result<T, BioMcpError> {
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(bytes);
            return Err(BioMcpError::Api {
                api: GPROFILER_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }

        serde_json::from_slice(bytes).map_err(|source| BioMcpError::ApiJson {
            api: GPROFILER_API.to_string(),
            source,
        })
    }

    pub(crate) fn enrich_genes_plan(
        genes: &[String],
        limit: usize,
    ) -> Result<(RequestPlan, usize), BioMcpError> {
        let query = genes
            .iter()
            .map(|g| g.trim().to_string())
            .filter(|g| !g.is_empty())
            .collect::<Vec<_>>();
        if query.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "g:Profiler requires at least one gene".into(),
            ));
        }
        if limit == 0 || limit > GPROFILER_MAX_ENRICH_LIMIT {
            return Err(BioMcpError::InvalidArgument(format!(
                "--limit must be between 1 and {GPROFILER_MAX_ENRICH_LIMIT}"
            )));
        }

        let mut plan = RequestPlan::post("gost/profile/");
        plan.body = RequestBody::Json(serde_json::json!({
            "organism": "hsapiens",
            "query": query,
        }));
        Ok((plan, limit))
    }

    fn map_enrich_response(resp: GProfilerResponse, limit: usize) -> Vec<GProfilerTerm> {
        resp.result.into_iter().take(limit).collect()
    }

    pub async fn enrich_genes(
        &self,
        genes: &[String],
        limit: usize,
    ) -> Result<Vec<GProfilerTerm>, BioMcpError> {
        let (plan, limit) = Self::enrich_genes_plan(genes, limit)?;
        let RequestBody::Json(body) = plan.body else {
            unreachable!("g:Profiler enrich uses a JSON body")
        };
        let url = self.endpoint(&plan.path);
        let resp: GProfilerResponse = self
            .post_json(self.client.post(&url), &body)
            .await
            .map_err(remap_gprofiler_error)?;

        Ok(Self::map_enrich_response(resp, limit))
    }
}

fn gprofiler_http_client(timeout: Duration) -> Result<reqwest::Client, BioMcpError> {
    reqwest::Client::builder()
        .timeout(timeout)
        .connect_timeout(GPROFILER_CONNECT_TIMEOUT)
        .user_agent(concat!("biomcp-cli/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(BioMcpError::HttpClientInit)
}

fn remap_gprofiler_error(err: BioMcpError) -> BioMcpError {
    match err {
        BioMcpError::WithSourceContext { context, source } => {
            remap_gprofiler_error(*source).with_source_context(context)
        }
        BioMcpError::Http(source) if source.is_timeout() || source.is_connect() => {
            gprofiler_source_unavailable(
                "The upstream is temporarily unavailable or too slow to respond.".to_string(),
            )
        }
        BioMcpError::Api { api, message } if api == GPROFILER_API => {
            if let Some(status) = transient_status_from_api_message(&message) {
                return gprofiler_source_unavailable(format!(
                    "The upstream returned HTTP {status} and is temporarily unavailable."
                ));
            }
            BioMcpError::Api { api, message }
        }
        other => other,
    }
}

/// Parse the HTTP status code from an `BioMcpError::Api` message produced by `post_json`,
/// which formats errors as `"HTTP {status}: {excerpt}"` where `{status}` includes the
/// canonical reason phrase (e.g. `"503 Service Unavailable"`).
fn transient_status_from_api_message(message: &str) -> Option<StatusCode> {
    let code = message
        .strip_prefix("HTTP ")?
        .split_whitespace()
        .next()?
        .parse::<u16>()
        .ok()?;
    let status = StatusCode::from_u16(code).ok()?;
    if status == StatusCode::REQUEST_TIMEOUT
        || status == StatusCode::TOO_MANY_REQUESTS
        || status.is_server_error()
    {
        Some(status)
    } else {
        None
    }
}

fn gprofiler_source_unavailable(reason: String) -> BioMcpError {
    BioMcpError::SourceUnavailable {
        source_name: "g:Profiler".to_string(),
        reason,
        suggestion: GPROFILER_RETRY_SUGGESTION.to_string(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GProfilerResponse {
    #[serde(default)]
    pub result: Vec<GProfilerTerm>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GProfilerTerm {
    pub native: Option<String>,
    pub name: Option<String>,
    pub source: Option<String>,
    pub p_value: Option<f64>,
}

#[cfg(test)]
mod tests;
