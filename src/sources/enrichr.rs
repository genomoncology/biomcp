use std::borrow::Cow;

use serde::Deserialize;
use tracing::warn;

use crate::error::BioMcpError;

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

    #[cfg(test)]
    fn new_for_test(base: String) -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            streaming_client: crate::sources::streaming_http_client()?,
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
        let resp = crate::sources::apply_cache_mode(req).send().await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_body(resp, ENRICHR_API).await?;
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
        let resp =
            crate::sources::retry_send(ENRICHR_API, 3, || async { build_request().send().await })
                .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_body(resp, ENRICHR_API).await?;
        Ok((status, content_type, bytes))
    }

    pub async fn add_list(&self, genes: &[&str]) -> Result<i64, BioMcpError> {
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

        if status.is_success() {
            let parsed: EnrichrAddListResponse =
                serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
                    api: ENRICHR_API.to_string(),
                    source,
                })?;
            return Ok(parsed.user_list_id);
        }

        let excerpt = crate::sources::body_excerpt(&bytes);
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
        let url = self.endpoint("enrich");
        let (status, content_type, bytes) = self
            .send_bytes(self.client.get(&url).query(&[
                ("userListId", user_list_id.to_string()),
                ("backgroundType", library.to_string()),
            ]))
            .await?;

        if status.is_success() {
            crate::sources::ensure_json_content_type(ENRICHR_API, content_type.as_ref(), &bytes)?;
            return serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
                api: ENRICHR_API.to_string(),
                source,
            });
        }

        // Enrichr occasionally returns HTTP 400 for otherwise-valid requests. Degrade
        // gracefully so gene lookups still succeed (terms will be empty for this library).
        if status == reqwest::StatusCode::BAD_REQUEST {
            warn!(
                source = ENRICHR_API,
                status = %status,
                body = %crate::sources::body_excerpt(&bytes),
                "Enrichr returned HTTP 400; degrading to empty enrichment payload"
            );
            return Ok(serde_json::Value::Object(serde_json::Map::new()));
        }

        let excerpt = crate::sources::body_excerpt(&bytes);
        Err(BioMcpError::Api {
            api: ENRICHR_API.to_string(),
            message: format!("HTTP {status}: {excerpt}"),
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct EnrichrAddListResponse {
    #[serde(rename = "userListId")]
    pub user_list_id: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn add_list_rejects_empty_gene_lists() {
        let client = EnrichrClient::new_for_test("http://127.0.0.1".into()).unwrap();
        let err = client.add_list(&[]).await.unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    }

    #[tokio::test]
    async fn add_list_parses_user_list_id() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/addList"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "userListId": 42
            })))
            .mount(&server)
            .await;

        let client = EnrichrClient::new_for_test(server.uri()).unwrap();
        let id = client.add_list(&["BRAF", "KRAS"]).await.unwrap();
        assert_eq!(id, 42);
    }

    #[tokio::test]
    async fn enrich_gracefully_handles_bad_request() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/enrich"))
            .and(query_param("userListId", "42"))
            .and(query_param("backgroundType", "KEGG_2021_Human"))
            .respond_with(ResponseTemplate::new(400).set_body_string("bad request"))
            .mount(&server)
            .await;

        let client = EnrichrClient::new_for_test(server.uri()).unwrap();
        let value = client.enrich(42, "KEGG_2021_Human").await.unwrap();
        assert!(value.is_object());
        assert!(value.as_object().unwrap().is_empty());
    }

    #[tokio::test]
    async fn enrich_bad_request_returns_empty_object_without_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/enrich"))
            .and(query_param("userListId", "7"))
            .and(query_param("backgroundType", "DisGeNET"))
            .respond_with(ResponseTemplate::new(400).set_body_string("invalid"))
            .expect(1)
            .mount(&server)
            .await;

        let client = EnrichrClient::new_for_test(server.uri()).unwrap();
        let value = client.enrich(7, "DisGeNET").await.unwrap();
        assert_eq!(value, serde_json::json!({}));
    }
}
