use std::borrow::Cow;

use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::BioMcpError;

const CHEMBL_BASE: &str = "https://www.ebi.ac.uk/chembl/api/data";
const CHEMBL_API: &str = "chembl";
const CHEMBL_BASE_ENV: &str = "BIOMCP_CHEMBL_BASE";

pub struct ChemblClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl ChemblClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(CHEMBL_BASE, CHEMBL_BASE_ENV),
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
        let bytes = crate::sources::read_limited_body(resp, CHEMBL_API).await?;
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: CHEMBL_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }
        serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
            api: CHEMBL_API.to_string(),
            source,
        })
    }

    pub async fn drug_targets(
        &self,
        chembl_id: &str,
        limit: usize,
    ) -> Result<Vec<ChemblTarget>, BioMcpError> {
        let chembl_id = chembl_id.trim();
        if chembl_id.is_empty() {
            return Err(BioMcpError::InvalidArgument("ChEMBL ID is required".into()));
        }

        let url = self.endpoint("mechanism.json");
        let limit = limit.clamp(1, 25).to_string();
        let resp: ChemblMechanismResponse = self
            .get_json(
                self.client
                    .get(&url)
                    .query(&[("molecule_chembl_id", chembl_id), ("limit", limit.as_str())]),
            )
            .await?;

        let mut out = Vec::new();
        for row in resp.mechanisms {
            let target = row
                .target_pref_name
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .unwrap_or("Unknown target");
            let action = row
                .action_type
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .unwrap_or("Mechanism");
            let mechanism = row
                .mechanism_of_action
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_string);
            out.push(ChemblTarget {
                target: target.to_string(),
                action: action.to_string(),
                mechanism,
            });
        }

        Ok(out)
    }
}

#[derive(Debug, Clone, Deserialize)]
struct ChemblMechanismResponse {
    #[serde(default)]
    mechanisms: Vec<ChemblMechanism>,
}

#[derive(Debug, Clone, Deserialize)]
struct ChemblMechanism {
    target_pref_name: Option<String>,
    action_type: Option<String>,
    mechanism_of_action: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ChemblTarget {
    pub target: String,
    pub action: String,
    pub mechanism: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn drug_targets_requests_mechanism_endpoint() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/mechanism.json"))
            .and(query_param("molecule_chembl_id", "CHEMBL25"))
            .and(query_param("limit", "3"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "mechanisms": [
                    {"target_pref_name": "BRAF", "action_type": "INHIBITOR"},
                    {"target_pref_name": null, "action_type": null}
                ]
            })))
            .mount(&server)
            .await;

        let client = ChemblClient::new_for_test(server.uri()).unwrap();
        let targets = client.drug_targets("CHEMBL25", 3).await.unwrap();
        assert_eq!(targets.len(), 2);
        assert_eq!(targets[0].target, "BRAF");
        assert_eq!(targets[0].action, "INHIBITOR");
        assert!(targets[0].mechanism.is_none());
        assert_eq!(targets[1].target, "Unknown target");
        assert_eq!(targets[1].action, "Mechanism");
    }

    #[tokio::test]
    async fn drug_targets_rejects_empty_chembl_id() {
        let client = ChemblClient::new_for_test("http://127.0.0.1".into()).unwrap();
        let err = client.drug_targets(" ", 5).await.unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    }
}
