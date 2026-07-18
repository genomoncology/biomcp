use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;

use reqwest::StatusCode;
use reqwest::header::HeaderValue;
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

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

    async fn get_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::COMPLEX_PORTAL,
            ))
            .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::COMPLEX_PORTAL),
        )
        .await?;
        Self::decode_json_response(status, content_type.as_ref(), &bytes).map_err(|error| {
            error.with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::COMPLEX_PORTAL,
            ))
        })
    }

    pub(crate) fn decode_json_response<T: DeserializeOwned>(
        status: StatusCode,
        content_type: Option<&HeaderValue>,
        bytes: &[u8],
    ) -> Result<T, BioMcpError> {
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(bytes);
            return Err(BioMcpError::Api {
                api: COMPLEXPORTAL_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }
        crate::sources::ensure_json_content_type(
            crate::error::SourceContext::retry(crate::error::SourceProvider::COMPLEX_PORTAL),
            content_type,
            bytes,
        )?;
        serde_json::from_slice(bytes).map_err(|source| BioMcpError::ApiJson {
            api: COMPLEXPORTAL_API.to_string(),
            source,
        })
    }

    pub(crate) fn complexes_plan(
        accession: &str,
        limit: usize,
    ) -> Result<Option<RequestPlan>, BioMcpError> {
        let accession = accession.trim();
        if accession.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "ComplexPortal requires a UniProt accession".into(),
            ));
        }
        if limit == 0 {
            return Ok(None);
        }

        Ok(Some(
            RequestPlan::get(format!("search/{accession}"))
                .query("number", COMPLEXPORTAL_SEARCH_PAGE_SIZE)
                .query("filters", COMPLEXPORTAL_FILTERS_HUMAN),
        ))
    }

    fn map_complexes(
        response: ComplexPortalSearchResponse,
        accession: &str,
        limit: usize,
    ) -> Vec<ComplexPortalComplex> {
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

        out
    }

    pub async fn complexes(
        &self,
        accession: &str,
        limit: usize,
    ) -> Result<Vec<ComplexPortalComplex>, BioMcpError> {
        let accession = accession.trim();
        let Some(plan) = Self::complexes_plan(accession, limit)? else {
            return Ok(Vec::new());
        };
        let response: ComplexPortalSearchResponse = self
            .get_json(request_from_plan(&self.client, self.base.as_ref(), &plan))
            .await?;
        Ok(Self::map_complexes(response, accession, limit))
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
mod tests;
