use biodata::{Doi, EuropePmcDetailPlan, Pmcid, Pmid, PublicationIdentifier};

use super::{EUROPE_PMC_API, EuropePmcClient, EuropePmcResult, EuropePmcSearchResponse};
use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

#[derive(Debug)]
pub enum EuropePmcDetail {
    Adopted(biodata::EuropePmcDetailResponse),
    Legacy {
        requested: PublicationIdentifier,
        result: EuropePmcResult,
    },
}

enum PlannedDetail {
    Adopted(EuropePmcDetailPlan),
    UnsupportedDoi { requested: PublicationIdentifier },
}

impl EuropePmcClient {
    #[cfg(test)]
    pub fn publication_detail_plan(
        identity: PublicationIdentifier,
    ) -> Result<RequestPlan, BioMcpError> {
        let detail = EuropePmcDetailPlan::new(identity).map_err(detail_plan_error)?;
        request_plan_from_biodata(&detail)
    }

    pub async fn publication_detail(
        &self,
        identity: PublicationIdentifier,
    ) -> Result<Option<EuropePmcDetail>, BioMcpError> {
        match plan_detail(identity)? {
            PlannedDetail::Adopted(detail) => {
                let request = request_plan_from_biodata(&detail)?;
                let req = request_from_plan(&self.client, self.base.as_ref(), &request);
                let (status, content_type, bytes) = self.acquire_json_response(req).await?;
                validate_detail_transport(status, content_type.as_ref(), &bytes)?;
                parse_adopted_or_legacy(&detail, &bytes)
            }
            PlannedDetail::UnsupportedDoi { requested } => {
                let PublicationIdentifier::Doi(value) = &requested else {
                    return Err(BioMcpError::InternalProcessing);
                };
                let (executed_query, response) =
                    self.search_by_doi_with_query(value.as_str()).await?;
                guard_legacy(requested, response, &executed_query)
            }
        }
    }
}

fn plan_detail(identity: PublicationIdentifier) -> Result<PlannedDetail, BioMcpError> {
    if let PublicationIdentifier::Doi(value) = &identity
        && value.as_str().len() > 256
    {
        return Err(BioMcpError::InvalidArgument("DOI is too long.".into()));
    }
    match EuropePmcDetailPlan::new(identity.clone()) {
        Ok(plan) => Ok(PlannedDetail::Adopted(plan)),
        Err(error)
            if error.code() == "unsupported_lookup"
                && matches!(identity, PublicationIdentifier::Doi(_)) =>
        {
            Ok(PlannedDetail::UnsupportedDoi {
                requested: identity,
            })
        }
        Err(error) => Err(detail_plan_error(error)),
    }
}

fn request_plan_from_biodata(detail: &EuropePmcDetailPlan) -> Result<RequestPlan, BioMcpError> {
    if detail.method() != "GET" {
        return Err(BioMcpError::InternalProcessing);
    }
    let path = detail
        .relative_path()
        .strip_prefix("/europepmc/webservices/rest/")
        .ok_or(BioMcpError::InternalProcessing)?;
    let mut plan = RequestPlan::get(path);
    for (key, value) in detail.query_pairs() {
        plan = plan.query(key, value);
    }
    Ok(plan)
}

fn validate_detail_transport(
    status: reqwest::StatusCode,
    content_type: Option<&reqwest::header::HeaderValue>,
    bytes: &[u8],
) -> Result<(), BioMcpError> {
    let context = crate::error::SourceContext::retry(crate::error::SourceProvider::EUROPE_PMC);
    if !status.is_success() {
        return Err(BioMcpError::Api {
            api: EUROPE_PMC_API.to_string(),
            message: format!("HTTP {status}"),
        }
        .with_source_context(context));
    }
    crate::sources::ensure_json_content_type(context, content_type, bytes)
}

#[cfg(test)]
pub(crate) fn parse_publication_detail(
    identity: PublicationIdentifier,
    bytes: &[u8],
) -> Result<Option<EuropePmcDetail>, BioMcpError> {
    match plan_detail(identity)? {
        PlannedDetail::Adopted(plan) => parse_adopted_or_legacy(&plan, bytes),
        PlannedDetail::UnsupportedDoi { requested } => {
            let PublicationIdentifier::Doi(value) = &requested else {
                return Err(BioMcpError::InternalProcessing);
            };
            let expected_query = EuropePmcClient::doi_query(value.as_str())?;
            let response =
                serde_json::from_slice(bytes).map_err(|_| detail_adapter_error("legacy_decode"))?;
            guard_legacy(requested, response, &expected_query)
        }
    }
}

fn parse_adopted_or_legacy(
    plan: &EuropePmcDetailPlan,
    bytes: &[u8],
) -> Result<Option<EuropePmcDetail>, BioMcpError> {
    match biodata::EuropePmcDetailResponse::parse(plan, bytes, &biodata::EuropePmcLimits::default())
    {
        Ok(response) => Ok(Some(EuropePmcDetail::Adopted(response))),
        Err(error) if error.code() == "not_found" => Ok(None),
        Err(error) if error.code() == "unsupported_shape" => {
            let response =
                serde_json::from_slice(bytes).map_err(|_| detail_adapter_error("legacy_decode"))?;
            guard_legacy(plan.identity().clone(), response, plan.query())
        }
        Err(error) => Err(detail_adapter_error(error.code())),
    }
}

fn guard_legacy(
    requested: PublicationIdentifier,
    response: EuropePmcSearchResponse,
    expected_query: &str,
) -> Result<Option<EuropePmcDetail>, BioMcpError> {
    let hit_count = response
        .hit_count
        .ok_or_else(|| detail_adapter_error("legacy_incomplete_response"))?;
    let list = response
        .result_list
        .ok_or_else(|| detail_adapter_error("legacy_incomplete_response"))?;
    let row_count = u64::try_from(list.result.len())
        .map_err(|_| detail_adapter_error("legacy_incomplete_response"))?;
    if hit_count != row_count {
        return Err(detail_adapter_error("legacy_incomplete_response"));
    }
    validate_legacy_echo(response.request.as_ref(), expected_query)?;
    if hit_count == 0 {
        return Ok(None);
    }
    let [result] = <[EuropePmcResult; 1]>::try_from(list.result)
        .map_err(|_| detail_adapter_error("legacy_ambiguous_identity"))?;
    validate_legacy_identity(&result, &requested)?;
    Ok(Some(EuropePmcDetail::Legacy { requested, result }))
}

fn validate_legacy_echo(
    request: Option<&super::EuropePmcLegacyRequest>,
    expected_query: &str,
) -> Result<(), BioMcpError> {
    let Some(request) = request else {
        return Ok(());
    };
    if request.query_string.as_deref() != Some(expected_query)
        || request.page_size != Some(1)
        || request.cursor_mark.as_deref() != Some("*")
        || request.result_type.as_deref() != Some("LITE")
    {
        return Err(detail_adapter_error("legacy_request_mismatch"));
    }
    Ok(())
}

fn validate_legacy_identity(
    result: &EuropePmcResult,
    requested: &PublicationIdentifier,
) -> Result<(), BioMcpError> {
    let med_id = (result.source.as_deref() == Some("MED"))
        .then(|| result.id.as_deref().and_then(|value| Pmid::new(value).ok()))
        .flatten();
    let pmid = result
        .pmid
        .as_deref()
        .and_then(|value| Pmid::new(value).ok());
    if med_id
        .as_ref()
        .zip(pmid.as_ref())
        .is_some_and(|(left, right)| left != right)
    {
        return Err(detail_adapter_error("legacy_conflicting_identity"));
    }
    let matched = match requested {
        PublicationIdentifier::Pmid(value) => pmid.or(med_id).as_ref() == Some(value),
        PublicationIdentifier::Pmcid(value) => {
            result
                .pmcid
                .as_deref()
                .and_then(|raw| Pmcid::new(raw).ok())
                .as_ref()
                == Some(value)
        }
        PublicationIdentifier::Doi(value) => {
            result
                .doi
                .as_deref()
                .and_then(|raw| Doi::new(raw).ok())
                .as_ref()
                == Some(value)
        }
    };
    if !matched {
        return Err(detail_adapter_error("legacy_identity_mismatch"));
    }
    Ok(())
}

fn detail_plan_error(error: biodata::EuropePmcError) -> BioMcpError {
    if error.code() == "unsupported_lookup" {
        BioMcpError::InvalidArgument("Europe PMC does not support this publication lookup".into())
    } else {
        detail_adapter_error(error.code())
    }
}

fn detail_adapter_error(code: &str) -> BioMcpError {
    let context = if matches!(code, "response_too_large" | "resource_limit") {
        crate::error::SourceContext::narrow(crate::error::SourceProvider::EUROPE_PMC)
    } else {
        crate::error::SourceContext::retry(crate::error::SourceProvider::EUROPE_PMC)
    };
    BioMcpError::Api {
        api: EUROPE_PMC_API.to_string(),
        message: format!("Europe PMC detail adapter: {code}"),
    }
    .with_source_context(context)
}
