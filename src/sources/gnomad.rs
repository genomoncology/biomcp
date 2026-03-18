use std::borrow::Cow;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::BioMcpError;

pub(crate) const GNOMAD_BASE: &str = "https://gnomad.broadinstitute.org/api";
pub(crate) const GNOMAD_API: &str = "gnomAD";
pub(crate) const GNOMAD_BASE_ENV: &str = "BIOMCP_GNOMAD_BASE";
pub(crate) const GNOMAD_CONSTRAINT_VERSION: &str = "v4";
pub(crate) const GNOMAD_CONSTRAINT_REFERENCE_GENOME: &str = "GRCh38";

pub struct GnomadClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GnomadConstraintData {
    pub pli: Option<f64>,
    pub loeuf: Option<f64>,
    pub mis_z: Option<f64>,
    pub syn_z: Option<f64>,
    pub transcript: Option<String>,
}

#[derive(Serialize)]
struct GraphQlRequest {
    query: &'static str,
    variables: serde_json::Value,
}

#[derive(Deserialize)]
struct GraphQlResponse<T> {
    data: Option<T>,
    #[serde(default)]
    errors: Option<Vec<GraphQlError>>,
}

#[derive(Deserialize)]
struct GraphQlError {
    message: Option<String>,
}

#[derive(Deserialize)]
struct GeneConstraintResponse {
    gene: Option<GeneConstraintGene>,
}

#[derive(Deserialize)]
struct GeneConstraintGene {
    canonical_transcript_id: Option<String>,
    gnomad_constraint: Option<ConstraintPayload>,
}

#[derive(Deserialize)]
struct ConstraintPayload {
    #[serde(rename = "pLI", alias = "pli")]
    pli: Option<f64>,
    #[serde(rename = "oe_lof_upper")]
    oe_lof_upper: Option<f64>,
    mis_z: Option<f64>,
    syn_z: Option<f64>,
}

impl GnomadClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(GNOMAD_BASE, GNOMAD_BASE_ENV),
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

    async fn post_json<T: DeserializeOwned, B: Serialize>(
        &self,
        req: reqwest_middleware::RequestBuilder,
        body: &B,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req.json(body))
            .send()
            .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_body(resp, GNOMAD_API).await?;

        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: GNOMAD_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }

        crate::sources::ensure_json_content_type(GNOMAD_API, content_type.as_ref(), &bytes)?;
        serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
            api: GNOMAD_API.to_string(),
            source,
        })
    }

    pub async fn gene_constraint(
        &self,
        symbol: &str,
    ) -> Result<Option<GnomadConstraintData>, BioMcpError> {
        let symbol = symbol.trim();
        if !crate::sources::is_valid_gene_symbol(symbol) {
            return Err(BioMcpError::InvalidArgument(
                "gnomAD requires a valid gene symbol".into(),
            ));
        }

        let body = GraphQlRequest {
            query: r#"
query GeneConstraint($symbol: String!) {
  gene(gene_symbol: $symbol, reference_genome: GRCh38) {
    canonical_transcript_id
    gnomad_constraint {
      pLI
      oe_lof_upper
      mis_z
      syn_z
    }
  }
}
"#,
            variables: serde_json::json!({ "symbol": symbol }),
        };

        let resp: GraphQlResponse<GeneConstraintResponse> = self
            .post_json(self.client.post(self.endpoint("")), &body)
            .await?;

        let errors = resp.errors.unwrap_or_default();
        let gene = resp.data.and_then(|data| data.gene);

        if !errors.is_empty() {
            let messages = errors
                .iter()
                .filter_map(|error| error.message.as_deref())
                .map(str::trim)
                .filter(|message| !message.is_empty())
                .collect::<Vec<_>>();

            if gene.is_none()
                && !messages.is_empty()
                && messages
                    .iter()
                    .all(|message| message.eq_ignore_ascii_case("Gene not found"))
            {
                return Ok(None);
            }

            let message = if messages.is_empty() {
                "GraphQL request failed".to_string()
            } else {
                messages.join("; ")
            };

            return Err(BioMcpError::Api {
                api: GNOMAD_API.to_string(),
                message,
            });
        }

        let Some(gene) = gene else {
            return Ok(None);
        };

        let transcript = gene
            .canonical_transcript_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);

        let Some(metrics) = gene.gnomad_constraint else {
            return Ok(Some(GnomadConstraintData {
                pli: None,
                loeuf: None,
                mis_z: None,
                syn_z: None,
                transcript,
            }));
        };

        Ok(Some(GnomadConstraintData {
            pli: metrics.pli,
            loeuf: metrics.oe_lof_upper,
            mis_z: metrics.mis_z,
            syn_z: metrics.syn_z,
            transcript,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{body_string_contains, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn gene_constraint_maps_metrics_and_transcript() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/"))
            .and(body_string_contains("GeneConstraint"))
            .and(body_string_contains("\"symbol\":\"TP53\""))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "gene": {
                        "canonical_transcript_id": "ENST00000269305",
                        "gnomad_constraint": {
                            "pLI": 0.9979,
                            "oe_lof_upper": 0.449,
                            "mis_z": 1.1539,
                            "syn_z": 0.9583
                        }
                    }
                }
            })))
            .mount(&server)
            .await;

        let client = GnomadClient::new_for_test(server.uri()).expect("client");
        let constraint = client
            .gene_constraint("TP53")
            .await
            .expect("constraint")
            .expect("gene result");

        assert_eq!(constraint.transcript.as_deref(), Some("ENST00000269305"));
        assert_eq!(constraint.pli, Some(0.9979));
        assert_eq!(constraint.loeuf, Some(0.449));
        assert_eq!(constraint.mis_z, Some(1.1539));
        assert_eq!(constraint.syn_z, Some(0.9583));
    }

    #[tokio::test]
    async fn gene_constraint_returns_some_with_transcript_when_constraint_is_null() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/"))
            .and(body_string_contains("\"symbol\":\"DDX3X\""))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "gene": {
                        "canonical_transcript_id": "ENST00000644876",
                        "gnomad_constraint": null
                    }
                }
            })))
            .mount(&server)
            .await;

        let client = GnomadClient::new_for_test(server.uri()).expect("client");
        let constraint = client
            .gene_constraint("DDX3X")
            .await
            .expect("constraint")
            .expect("gene result");

        assert_eq!(constraint.transcript.as_deref(), Some("ENST00000644876"));
        assert_eq!(constraint.pli, None);
        assert_eq!(constraint.loeuf, None);
        assert_eq!(constraint.mis_z, None);
        assert_eq!(constraint.syn_z, None);
    }

    #[tokio::test]
    async fn gene_constraint_returns_none_for_gene_not_found() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/"))
            .and(body_string_contains("\"symbol\":\"NOTAREALGENE123\""))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "errors": [{"message": "Gene not found"}],
                "data": {"gene": null}
            })))
            .mount(&server)
            .await;

        let client = GnomadClient::new_for_test(server.uri()).expect("client");
        let constraint = client
            .gene_constraint("NOTAREALGENE123")
            .await
            .expect("not found should degrade");

        assert!(constraint.is_none());
    }

    #[tokio::test]
    async fn gene_constraint_propagates_non_not_found_graphql_errors() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/"))
            .and(body_string_contains("\"symbol\":\"TP53\""))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "errors": [{"message": "upstream exploded"}],
                "data": {"gene": null}
            })))
            .mount(&server)
            .await;

        let client = GnomadClient::new_for_test(server.uri()).expect("client");
        let err = client
            .gene_constraint("TP53")
            .await
            .expect_err("non-not-found graphql errors should surface");

        assert!(matches!(err, BioMcpError::Api { .. }));
        assert!(err.to_string().contains("upstream exploded"));
    }
}
