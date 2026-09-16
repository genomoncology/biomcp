use super::{PubTatorClient, PubTatorDocument, PubTatorExportResponse, clean_api_key};
use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

#[derive(Debug)]
pub enum PubTatorDetail {
    Adopted(biodata::PubTator3DetailResponse),
    Legacy {
        requested_pmid: biodata::Pmid,
        document: PubTatorDocument,
    },
}

impl PubTatorClient {
    #[cfg(test)]
    pub fn publication_detail_plan(
        pmid: u32,
        api_key: Option<&str>,
    ) -> Result<RequestPlan, BioMcpError> {
        let detail_plan = biodata_detail_plan(pmid)?;
        request_plan_from_biodata(&detail_plan, api_key)
    }

    pub async fn publication_detail(
        &self,
        pmid: u32,
    ) -> Result<Option<PubTatorDetail>, BioMcpError> {
        let detail_plan = biodata_detail_plan(pmid)?;
        let plan = request_plan_from_biodata(&detail_plan, self.api_key.as_deref())?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        let (status, content_type, bytes) = self
            .acquire_json_response(req, self.api_key.is_some())
            .await?;
        validate_detail_transport(status, content_type.as_ref(), &bytes)?;
        parse_publication_detail_with_plan(&detail_plan, &bytes)
    }
}

pub(crate) fn validate_detail_transport(
    status: reqwest::StatusCode,
    content_type: Option<&reqwest::header::HeaderValue>,
    bytes: &[u8],
) -> Result<(), BioMcpError> {
    let context = crate::error::SourceContext::retry(crate::error::SourceProvider::PUBTATOR3);
    if !status.is_success() {
        return Err(BioMcpError::Api {
            api: crate::error::SourceProvider::PUBTATOR3.label().to_string(),
            message: format!("HTTP {status}"),
        }
        .with_source_context(context));
    }
    crate::sources::ensure_json_content_type(context, content_type, bytes)
}

fn biodata_detail_plan(pmid: u32) -> Result<biodata::PubTator3DetailPlan, BioMcpError> {
    let pmid = biodata::Pmid::new(&pmid.to_string()).map_err(|_| {
        BioMcpError::InvalidArgument("PMID must be a nonzero canonical identifier".into())
    })?;
    Ok(biodata::PubTator3DetailPlan::new(pmid))
}

fn request_plan_from_biodata(
    detail: &biodata::PubTator3DetailPlan,
    api_key: Option<&str>,
) -> Result<RequestPlan, BioMcpError> {
    let path = detail
        .relative_path()
        .strip_prefix("/research/pubtator3-api/")
        .ok_or(BioMcpError::InternalProcessing)?;
    let mut plan = RequestPlan::get(path);
    for (key, value) in detail.query_pairs() {
        plan = plan.query(key, value);
    }
    if let Some(key) = clean_api_key(api_key) {
        plan = plan.query("api_key", key);
    }
    Ok(plan)
}

#[cfg(test)]
pub(crate) fn parse_publication_detail(
    pmid: u32,
    bytes: &[u8],
) -> Result<Option<PubTatorDetail>, BioMcpError> {
    let plan = biodata_detail_plan(pmid)?;
    parse_publication_detail_with_plan(&plan, bytes)
}

fn parse_publication_detail_with_plan(
    plan: &biodata::PubTator3DetailPlan,
    bytes: &[u8],
) -> Result<Option<PubTatorDetail>, BioMcpError> {
    match biodata::PubTator3DetailResponse::parse(plan, bytes, &biodata::PubTator3Limits::default())
    {
        Ok(response) => Ok(Some(PubTatorDetail::Adopted(response))),
        Err(error) if error.code() == "not_found" => Ok(None),
        Err(error) if error.code() == "unsupported_shape" => {
            parse_legacy_detail(plan.requested_pmid(), bytes).map(Some)
        }
        Err(error) => Err(detail_adapter_error(error.code())),
    }
}

fn parse_legacy_detail(
    requested: &biodata::Pmid,
    bytes: &[u8],
) -> Result<PubTatorDetail, BioMcpError> {
    let response: PubTatorExportResponse =
        serde_json::from_slice(bytes).map_err(|_| detail_adapter_error("legacy_decode"))?;
    let [document] = response.documents.as_slice() else {
        return Err(detail_adapter_error("legacy_ambiguous_identity"));
    };
    let id = document
        .id
        .as_deref()
        .and_then(|value| biodata::Pmid::new(value).ok());
    let pmid = document
        .pmid
        .and_then(|value| biodata::Pmid::new(&value.to_string()).ok());
    if id
        .as_ref()
        .zip(pmid.as_ref())
        .is_some_and(|(left, right)| left != right)
    {
        return Err(detail_adapter_error("legacy_conflicting_identity"));
    }
    let Some(identity) = id.or(pmid) else {
        return Err(detail_adapter_error("legacy_unusable_identity"));
    };
    if &identity != requested {
        return Err(detail_adapter_error("legacy_identity_mismatch"));
    }
    let document = response
        .documents
        .into_iter()
        .next()
        .ok_or_else(|| detail_adapter_error("legacy_ambiguous_identity"))?;
    Ok(PubTatorDetail::Legacy {
        requested_pmid: requested.clone(),
        document,
    })
}

fn detail_adapter_error(code: &'static str) -> BioMcpError {
    let context = if matches!(code, "response_too_large" | "resource_limit") {
        crate::error::SourceContext::narrow(crate::error::SourceProvider::PUBTATOR3)
    } else {
        crate::error::SourceContext::retry(crate::error::SourceProvider::PUBTATOR3)
    };
    BioMcpError::Api {
        api: crate::error::SourceProvider::PUBTATOR3.label().to_string(),
        message: format!("PubTator detail adapter: {code}"),
    }
    .with_source_context(context)
}
