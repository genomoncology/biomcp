use std::borrow::Cow;

use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::BioMcpError;

const COMPLEXPORTAL_BASE: &str = "https://www.ebi.ac.uk/intact/complex-ws";
const COMPLEXPORTAL_API: &str = "complexportal";
const COMPLEXPORTAL_BASE_ENV: &str = "BIOMCP_COMPLEXPORTAL_BASE";
const COMPLEXPORTAL_FILTERS_HUMAN: &str = r#"species_f:("Homo sapiens")"#;
const COMPLEXPORTAL_SEARCH_PAGE_SIZE: &str = "25";

pub struct ComplexPortalClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl ComplexPortalClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(COMPLEXPORTAL_BASE, COMPLEXPORTAL_BASE_ENV),
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
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_body(resp, COMPLEXPORTAL_API).await?;
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: COMPLEXPORTAL_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }
        crate::sources::ensure_json_content_type(COMPLEXPORTAL_API, content_type.as_ref(), &bytes)?;
        serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
            api: COMPLEXPORTAL_API.to_string(),
            source,
        })
    }

    pub async fn complexes(
        &self,
        accession: &str,
        limit: usize,
    ) -> Result<Vec<ComplexPortalComplex>, BioMcpError> {
        let accession = accession.trim();
        if accession.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "ComplexPortal requires a UniProt accession".into(),
            ));
        }
        if limit == 0 {
            return Ok(Vec::new());
        }

        let url = self.endpoint(&format!("search/{accession}"));
        let response: ComplexPortalSearchResponse = self
            .get_json(self.client.get(&url).query(&[
                ("number", COMPLEXPORTAL_SEARCH_PAGE_SIZE),
                ("filters", COMPLEXPORTAL_FILTERS_HUMAN),
            ]))
            .await?;

        let mut out = Vec::new();
        for row in response.elements {
            if !queried_accession_is_protein_participant(&row.interactors, accession) {
                continue;
            }

            let Some(complex_accession) = trim_to_option(row.complex_accession) else {
                continue;
            };
            let Some(name) = trim_to_option(row.complex_name) else {
                continue;
            };

            let participants = row
                .interactors
                .into_iter()
                .filter(is_protein_interactor)
                .filter_map(|participant| {
                    let accession = trim_to_option(participant.identifier)?;
                    let name =
                        trim_to_option(participant.name).unwrap_or_else(|| accession.clone());
                    Some(ComplexPortalParticipant {
                        accession,
                        name,
                        stoichiometry: trim_to_option(participant.stoichiometry),
                    })
                })
                .collect::<Vec<_>>();

            out.push(ComplexPortalComplex {
                accession: complex_accession,
                name,
                description: trim_to_option(row.description),
                predicted_complex: row.predicted_complex,
                participants,
            });
            if out.len() >= limit {
                break;
            }
        }

        Ok(out)
    }
}

fn trim_to_option(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn is_protein_interactor(interactor: &ComplexPortalInteractor) -> bool {
    interactor
        .interactor_type
        .as_deref()
        .map(str::trim)
        .is_some_and(|value| value.eq_ignore_ascii_case("protein"))
}

fn queried_accession_is_protein_participant(
    interactors: &[ComplexPortalInteractor],
    accession: &str,
) -> bool {
    interactors.iter().any(|interactor| {
        is_protein_interactor(interactor)
            && interactor
                .identifier
                .as_deref()
                .map(str::trim)
                .is_some_and(|value| value.eq_ignore_ascii_case(accession))
    })
}

#[derive(Debug, Clone)]
pub struct ComplexPortalComplex {
    pub accession: String,
    pub name: String,
    pub description: Option<String>,
    pub predicted_complex: bool,
    pub participants: Vec<ComplexPortalParticipant>,
}

#[derive(Debug, Clone)]
pub struct ComplexPortalParticipant {
    pub accession: String,
    pub name: String,
    pub stoichiometry: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ComplexPortalSearchResponse {
    #[serde(default)]
    elements: Vec<ComplexPortalSearchRow>,
}

#[derive(Debug, Deserialize)]
struct ComplexPortalSearchRow {
    #[serde(rename = "complexAC")]
    complex_accession: Option<String>,
    #[serde(rename = "complexName")]
    complex_name: Option<String>,
    description: Option<String>,
    #[serde(rename = "predictedComplex", default)]
    predicted_complex: bool,
    #[serde(default)]
    interactors: Vec<ComplexPortalInteractor>,
}

#[derive(Debug, Deserialize)]
struct ComplexPortalInteractor {
    identifier: Option<String>,
    name: Option<String>,
    #[serde(rename = "stochiometry")]
    stoichiometry: Option<String>,
    #[serde(rename = "interactorType")]
    interactor_type: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn complexes_requests_expected_endpoint_and_filters_false_positives() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/search/P15056"))
            .and(query_param("number", "25"))
            .and(query_param("filters", r#"species_f:("Homo sapiens")"#))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "elements": [
                    {
                        "complexAC": "CPX-1",
                        "complexName": "BRAF complex",
                        "description": "  RAF signaling complex  ",
                        "predictedComplex": false,
                        "interactors": [
                            {
                                "identifier": "P15056",
                                "name": "BRAF",
                                "stochiometry": " minValue: 1, maxValue: 1 ",
                                "interactorType": "protein"
                            },
                            {
                                "identifier": "Q02750",
                                "name": "MAP2K1",
                                "stochiometry": "",
                                "interactorType": "protein"
                            },
                            {
                                "identifier": "CHEBI:1234",
                                "name": "ATP",
                                "stochiometry": "minValue: 1, maxValue: 1",
                                "interactorType": "small molecule"
                            }
                        ]
                    },
                    {
                        "complexAC": "CPX-2",
                        "complexName": "Description-only mention",
                        "description": "Mentions P15056 but does not contain it as a participant",
                        "predictedComplex": true,
                        "interactors": [
                            {
                                "identifier": "Q9Y243",
                                "name": "AKT3",
                                "stochiometry": "minValue: 1, maxValue: 1",
                                "interactorType": "protein"
                            }
                        ]
                    }
                ]
            })))
            .mount(&server)
            .await;

        let client = ComplexPortalClient::new_for_test(server.uri()).unwrap();
        let rows = client.complexes("P15056", 10).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].accession, "CPX-1");
        assert_eq!(rows[0].name, "BRAF complex");
        assert_eq!(
            rows[0].description.as_deref(),
            Some("RAF signaling complex")
        );
        assert_eq!(rows[0].participants.len(), 2);
        assert_eq!(rows[0].participants[0].accession, "P15056");
        assert_eq!(rows[0].participants[0].name, "BRAF");
        assert_eq!(
            rows[0].participants[0].stoichiometry.as_deref(),
            Some("minValue: 1, maxValue: 1")
        );
        assert_eq!(rows[0].participants[1].accession, "Q02750");
        assert_eq!(rows[0].participants[1].stoichiometry, None);
    }

    #[tokio::test]
    async fn complexes_rejects_empty_accession() {
        let client = ComplexPortalClient::new_for_test("http://127.0.0.1".into()).unwrap();
        let err = client.complexes(" ", 10).await.unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    }

    #[tokio::test]
    async fn complexes_returns_empty_vec_for_zero_participant_matches() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/search/Q9Y243"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "elements": []
            })))
            .mount(&server)
            .await;

        let client = ComplexPortalClient::new_for_test(server.uri()).unwrap();
        let rows = client.complexes("Q9Y243", 10).await.unwrap();
        assert!(rows.is_empty());
    }
}
