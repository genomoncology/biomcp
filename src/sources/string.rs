use std::borrow::Cow;

use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::BioMcpError;

const STRING_BASE: &str = "https://string-db.org/api";
const STRING_API: &str = "string";
const STRING_BASE_ENV: &str = "BIOMCP_STRING_BASE";

pub struct StringClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl StringClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(STRING_BASE, STRING_BASE_ENV),
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
        let bytes = crate::sources::read_limited_body(resp, STRING_API).await?;
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: STRING_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }
        serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
            api: STRING_API.to_string(),
            source,
        })
    }

    pub async fn interactions(
        &self,
        identifiers: &str,
        species: u32,
        limit: usize,
    ) -> Result<Vec<StringInteraction>, BioMcpError> {
        let identifiers = identifiers.trim();
        if identifiers.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "STRING identifiers are required".into(),
            ));
        }

        let url = self.endpoint("json/network");
        let species = species.to_string();
        let limit = limit.clamp(1, 25).to_string();
        self.get_json(self.client.get(&url).query(&[
            ("identifiers", identifiers),
            ("species", species.as_str()),
            ("limit", limit.as_str()),
        ]))
        .await
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct StringInteraction {
    #[serde(rename = "preferredName_A", alias = "preferredNameA")]
    pub preferred_name_a: Option<String>,
    #[serde(rename = "preferredName_B", alias = "preferredNameB")]
    pub preferred_name_b: Option<String>,
    pub score: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn interactions_sets_expected_query_params() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/json/network"))
            .and(query_param("identifiers", "BRAF"))
            .and(query_param("species", "9606"))
            .and(query_param("limit", "5"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!([{
                    "preferredNameA": "BRAF",
                    "preferredNameB": "KRAS",
                    "score": 0.91
                }])),
            )
            .mount(&server)
            .await;

        let client = StringClient::new_for_test(server.uri()).unwrap();
        let rows = client.interactions("BRAF", 9606, 5).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].preferred_name_b.as_deref(), Some("KRAS"));
    }

    #[tokio::test]
    async fn interactions_rejects_empty_identifiers() {
        let client = StringClient::new_for_test("http://127.0.0.1".into()).unwrap();
        let err = client.interactions("   ", 9606, 5).await.unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    }

    #[tokio::test]
    async fn interactions_parses_preferred_name_underscore_fields() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/json/network"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!([{
                    "preferredName_A": "BRAF",
                    "preferredName_B": "MAP2K1",
                    "score": 0.88
                }])),
            )
            .mount(&server)
            .await;

        let client = StringClient::new_for_test(server.uri()).unwrap();
        let rows = client.interactions("BRAF", 9606, 5).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].preferred_name_a.as_deref(), Some("BRAF"));
        assert_eq!(rows[0].preferred_name_b.as_deref(), Some("MAP2K1"));
    }
}
