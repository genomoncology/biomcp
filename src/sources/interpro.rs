use std::borrow::Cow;

use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::BioMcpError;

const INTERPRO_BASE: &str = "https://www.ebi.ac.uk/interpro/api";
const INTERPRO_API: &str = "interpro";
const INTERPRO_BASE_ENV: &str = "BIOMCP_INTERPRO_BASE";

pub struct InterProClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl InterProClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(INTERPRO_BASE, INTERPRO_BASE_ENV),
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
        let bytes = crate::sources::read_limited_body(resp, INTERPRO_API).await?;
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: INTERPRO_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }
        serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
            api: INTERPRO_API.to_string(),
            source,
        })
    }

    pub async fn domains(
        &self,
        uniprot_accession: &str,
        limit: usize,
    ) -> Result<Vec<InterProDomain>, BioMcpError> {
        let uniprot_accession = uniprot_accession.trim();
        if uniprot_accession.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "InterPro requires a UniProt accession".into(),
            ));
        }

        let page_size = limit.clamp(1, 25).to_string();
        let url = self.endpoint(&format!(
            "entry/interpro/protein/uniprot/{uniprot_accession}/"
        ));

        let resp: InterProResponse = self
            .get_json(
                self.client
                    .get(&url)
                    .query(&[("page_size", page_size.as_str())]),
            )
            .await?;

        let mut out = Vec::new();
        for row in resp.results.into_iter().take(limit.clamp(1, 25)) {
            let Some(meta) = row.metadata else { continue };
            let Some(accession) = meta.accession.map(|v| v.trim().to_string()) else {
                continue;
            };
            if accession.is_empty() {
                continue;
            }
            let name = meta
                .name
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty());
            let domain_type = meta
                .r#type
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty());
            out.push(InterProDomain {
                accession,
                name,
                domain_type,
            });
        }

        Ok(out)
    }
}

#[derive(Debug, Clone)]
pub struct InterProDomain {
    pub accession: String,
    pub name: Option<String>,
    pub domain_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct InterProResponse {
    #[serde(default)]
    results: Vec<InterProResult>,
}

#[derive(Debug, Deserialize)]
struct InterProResult {
    metadata: Option<InterProMetadata>,
}

#[derive(Debug, Deserialize)]
struct InterProMetadata {
    accession: Option<String>,
    name: Option<String>,
    #[serde(rename = "type")]
    r#type: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn domains_requests_expected_endpoint_and_maps_rows() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/entry/interpro/protein/uniprot/P15056/"))
            .and(query_param("page_size", "3"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [
                    {"metadata": {"accession": "IPR000719", "name": "Protein kinase", "type": "domain"}},
                    {"metadata": {"accession": " ", "name": "skip", "type": "domain"}}
                ]
            })))
            .mount(&server)
            .await;

        let client = InterProClient::new_for_test(server.uri()).unwrap();
        let rows = client.domains("P15056", 3).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].accession, "IPR000719");
        assert_eq!(rows[0].name.as_deref(), Some("Protein kinase"));
    }

    #[tokio::test]
    async fn domains_rejects_empty_accession() {
        let client = InterProClient::new_for_test("http://127.0.0.1".into()).unwrap();
        let err = client.domains(" ", 5).await.unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    }
}
