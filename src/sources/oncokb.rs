use std::borrow::Cow;

use serde::Deserialize;
use serde::de::DeserializeOwned;
use tracing::debug;

use crate::error::BioMcpError;

const ONCOKB_PROD_BASE: &str = "https://www.oncokb.org/api/v1";
const ONCOKB_API: &str = "oncokb";
const ONCOKB_TOKEN_ENV: &str = "ONCOKB_TOKEN";
const ONCOKB_BASE_ENV: &str = "BIOMCP_ONCOKB_BASE";

pub struct OncoKBClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
    token: Option<String>,
}

impl OncoKBClient {
    pub fn new() -> Result<Self, BioMcpError> {
        let token = std::env::var(ONCOKB_TOKEN_ENV)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let base = std::env::var(ONCOKB_BASE_ENV)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .map(Cow::Owned)
            .unwrap_or_else(|| Cow::Borrowed(ONCOKB_PROD_BASE));

        Ok(Self {
            client: crate::sources::shared_client()?,
            base,
            token,
        })
    }

    #[cfg(test)]
    fn new_for_test(base: String, token: Option<String>) -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: Cow::Owned(base),
            token,
        })
    }

    fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base.as_ref().trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    fn require_token(&self) -> Result<&str, BioMcpError> {
        self.token
            .as_deref()
            .filter(|t| !t.trim().is_empty())
            .ok_or_else(|| BioMcpError::ApiKeyRequired {
                api: ONCOKB_API.to_string(),
                env_var: ONCOKB_TOKEN_ENV.to_string(),
                docs_url: "https://www.oncokb.org/".to_string(),
            })
    }

    async fn get_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
        authenticated: bool,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode_with_auth(req, authenticated)
            .send()
            .await?;
        let status = resp.status();
        let bytes = crate::sources::read_limited_body(resp, ONCOKB_API).await?;
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: ONCOKB_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }
        serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
            api: ONCOKB_API.to_string(),
            source,
        })
    }

    pub async fn annotate_by_protein_change(
        &self,
        gene: &str,
        alteration: &str,
    ) -> Result<OncoKBAnnotation, BioMcpError> {
        let gene = gene.trim();
        let alteration = alteration.trim();
        if gene.is_empty() || alteration.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "OncoKB annotation requires gene and alteration".into(),
            ));
        }
        let token = self.require_token()?;

        let url = self.endpoint("annotate/mutations/byProteinChange");
        let req = self
            .client
            .get(&url)
            .query(&[("hugoSymbol", gene), ("alteration", alteration)])
            .header("Authorization", format!("Bearer {token}"));

        self.get_json(req, true).await
    }

    pub async fn annotate_best_effort(
        &self,
        gene: &str,
        alteration: &str,
    ) -> Result<OncoKBAnnotation, BioMcpError> {
        let gene = gene.trim();
        let alteration = alteration.trim();
        if gene.is_empty() || alteration.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "OncoKB annotation requires gene and alteration".into(),
            ));
        }

        let mut attempts: Vec<String> = Vec::new();
        let mut push_attempt = |value: String| {
            let v = value.trim().to_string();
            if v.is_empty() {
                return;
            }
            if attempts.iter().any(|a| a.eq_ignore_ascii_case(&v)) {
                return;
            }
            attempts.push(v);
        };

        push_attempt(alteration.to_string());
        if alteration.starts_with("p.") || alteration.starts_with("P.") {
            push_attempt(alteration[2..].trim().to_string());
        } else {
            push_attempt(format!("p.{alteration}"));
        }

        let mut last_err: Option<BioMcpError> = None;
        for alt in attempts {
            debug!(gene = %gene, alteration = %alt, "OncoKB annotate attempt");
            match self.annotate_by_protein_change(gene, &alt).await {
                Ok(ann) => return Ok(ann),
                Err(err) => last_err = Some(err),
            }
        }

        Err(last_err.unwrap_or_else(|| BioMcpError::Api {
            api: ONCOKB_API.to_string(),
            message: "No OncoKB annotation available".into(),
        }))
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OncoKBAnnotation {
    pub oncogenic: Option<String>,
    pub mutation_effect: Option<OncoKBMutationEffect>,
    pub highest_sensitive_level: Option<String>,
    pub highest_resistance_level: Option<String>,
    #[serde(default)]
    pub treatments: Vec<OncoKBTreatment>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OncoKBMutationEffect {
    pub known_effect: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OncoKBTreatment {
    pub level: Option<String>,
    #[serde(default)]
    pub drugs: Vec<OncoKBDrug>,
    pub cancer_type: Option<OncoKBCancerType>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OncoKBDrug {
    pub drug_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OncoKBCancerType {
    pub name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn annotate_includes_auth_header_when_token_provided() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/annotate/mutations/byProteinChange"))
            .and(query_param("hugoSymbol", "BRAF"))
            .and(query_param("alteration", "V600E"))
            .and(header("Authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "oncogenic": "Oncogenic",
                "mutationEffect": { "knownEffect": "Gain-of-function" },
                "highestSensitiveLevel": "LEVEL_1"
            })))
            .mount(&server)
            .await;

        let client = OncoKBClient::new_for_test(server.uri(), Some("test-token".into())).unwrap();
        let ann = client
            .annotate_by_protein_change("BRAF", "V600E")
            .await
            .unwrap();
        assert_eq!(ann.oncogenic.as_deref(), Some("Oncogenic"));
        assert_eq!(
            ann.mutation_effect
                .as_ref()
                .and_then(|m| m.known_effect.as_deref()),
            Some("Gain-of-function")
        );
        assert_eq!(ann.highest_sensitive_level.as_deref(), Some("LEVEL_1"));
    }

    #[tokio::test]
    async fn annotate_requires_gene_and_alteration() {
        let server = MockServer::start().await;
        let client = OncoKBClient::new_for_test(server.uri(), None).unwrap();

        let err = client
            .annotate_by_protein_change("", "V600E")
            .await
            .unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));

        let err = client
            .annotate_by_protein_change("BRAF", "")
            .await
            .unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    }

    #[tokio::test]
    async fn annotate_requires_api_key() {
        let server = MockServer::start().await;
        let client = OncoKBClient::new_for_test(server.uri(), None).unwrap();

        let err = client
            .annotate_by_protein_change("BRAF", "V600E")
            .await
            .unwrap_err();
        assert!(matches!(err, BioMcpError::ApiKeyRequired { .. }));
    }

    #[tokio::test]
    async fn annotate_surfaces_http_errors() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/annotate/mutations/byProteinChange"))
            .and(query_param("hugoSymbol", "BRAF"))
            .and(query_param("alteration", "V600E"))
            .respond_with(ResponseTemplate::new(500).set_body_string("upstream failed"))
            .mount(&server)
            .await;

        let client = OncoKBClient::new_for_test(server.uri(), Some("test-token".into())).unwrap();
        let err = client
            .annotate_by_protein_change("BRAF", "V600E")
            .await
            .unwrap_err();
        assert!(matches!(err, BioMcpError::Api { .. }));
        let msg = err.to_string();
        assert!(msg.contains("oncokb"));
        assert!(msg.contains("500"));
    }
}
