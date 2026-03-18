use std::borrow::Cow;
use std::time::Duration;

use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::BioMcpError;

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

    #[cfg(test)]
    fn new_for_test(base: String) -> Result<Self, BioMcpError> {
        Self::new_for_test_with_timeout(base, GPROFILER_TIMEOUT)
    }

    #[cfg(test)]
    fn new_for_test_with_timeout(base: String, timeout: Duration) -> Result<Self, BioMcpError> {
        Ok(Self {
            client: gprofiler_http_client(timeout)?,
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

    async fn post_json<T: DeserializeOwned, B: Serialize>(
        &self,
        req: reqwest::RequestBuilder,
        body: &B,
    ) -> Result<T, BioMcpError> {
        let resp = req.json(body).send().await?;
        let status = resp.status();
        let bytes = crate::sources::read_limited_body(resp, GPROFILER_API).await?;

        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: GPROFILER_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }

        serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
            api: GPROFILER_API.to_string(),
            source,
        })
    }

    pub async fn enrich_genes(
        &self,
        genes: &[String],
        limit: usize,
    ) -> Result<Vec<GProfilerTerm>, BioMcpError> {
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

        let url = self.endpoint("gost/profile/");
        let body = serde_json::json!({
            "organism": "hsapiens",
            "query": query,
        });

        let resp: GProfilerResponse = self
            .post_json(self.client.post(&url), &body)
            .await
            .map_err(remap_gprofiler_error)?;

        Ok(resp.result.into_iter().take(limit).collect())
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
mod tests {
    use super::*;
    use std::time::Duration;
    use wiremock::matchers::{body_string_contains, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn enrich_genes_posts_query_and_applies_limit() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/gost/profile/"))
            .and(body_string_contains("\"organism\":\"hsapiens\""))
            .and(body_string_contains("\"BRAF\""))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "result": [
                    {"native": "R-HSA-1", "name": "A", "source": "REAC", "p_value": 0.01},
                    {"native": "R-HSA-2", "name": "B", "source": "REAC", "p_value": 0.02}
                ]
            })))
            .mount(&server)
            .await;

        let client = GProfilerClient::new_for_test(server.uri()).unwrap();
        let rows = client
            .enrich_genes(&["BRAF".to_string(), "KRAS".to_string()], 1)
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].native.as_deref(), Some("R-HSA-1"));
    }

    #[tokio::test]
    async fn enrich_genes_rejects_empty_input() {
        let client = GProfilerClient::new_for_test("http://127.0.0.1".into()).unwrap();
        let err = client.enrich_genes(&[], 5).await.unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    }

    #[tokio::test]
    async fn enrich_genes_rejects_zero_limit() {
        let client = GProfilerClient::new_for_test("http://127.0.0.1".into()).unwrap();
        let err = client
            .enrich_genes(&["BRAF".to_string(), "KRAS".to_string()], 0)
            .await
            .unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
        assert!(err.to_string().contains("--limit must be between 1 and 50"));
    }

    #[tokio::test]
    async fn enrich_genes_rejects_limit_above_max() {
        let client = GProfilerClient::new_for_test("http://127.0.0.1".into()).unwrap();
        let err = client
            .enrich_genes(&["BRAF".to_string(), "KRAS".to_string()], 51)
            .await
            .unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
        assert!(err.to_string().contains("--limit must be between 1 and 50"));
    }

    #[tokio::test]
    async fn enrich_genes_timeout_maps_to_source_unavailable() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/gost/profile/"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_delay(Duration::from_millis(100))
                    .set_body_json(serde_json::json!({ "result": [] })),
            )
            .mount(&server)
            .await;

        let client =
            GProfilerClient::new_for_test_with_timeout(server.uri(), Duration::from_millis(50))
                .unwrap();
        let err = client
            .enrich_genes(&["BRAF".to_string(), "KRAS".to_string()], 5)
            .await
            .unwrap_err();
        assert!(matches!(err, BioMcpError::SourceUnavailable { .. }));
        let msg = err.to_string();
        assert!(msg.contains("g:Profiler"));
        assert!(msg.to_ascii_lowercase().contains("retry"));
    }

    #[tokio::test]
    async fn enrich_genes_503_maps_to_source_unavailable() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/gost/profile/"))
            .respond_with(ResponseTemplate::new(503).set_body_string("upstream failure"))
            .mount(&server)
            .await;

        let client = GProfilerClient::new_for_test(server.uri()).unwrap();
        let err = client
            .enrich_genes(&["BRAF".to_string(), "KRAS".to_string()], 5)
            .await
            .unwrap_err();
        assert!(matches!(err, BioMcpError::SourceUnavailable { .. }));
    }

    #[tokio::test]
    async fn enrich_genes_400_remains_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/gost/profile/"))
            .respond_with(ResponseTemplate::new(400).set_body_string("bad request"))
            .mount(&server)
            .await;

        let client = GProfilerClient::new_for_test(server.uri()).unwrap();
        let err = client
            .enrich_genes(&["BRAF".to_string(), "KRAS".to_string()], 5)
            .await
            .unwrap_err();
        assert!(matches!(err, BioMcpError::Api { .. }));
        assert!(err.to_string().contains("HTTP 400"));
    }
}
