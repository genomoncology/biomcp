use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;

use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

// NCBI PMC ID Converter API
// Docs: https://pmc.ncbi.nlm.nih.gov/tools/id-converter-api/
const NCBI_IDCONV_BASE: &str = "https://pmc.ncbi.nlm.nih.gov/tools/idconv/api/v1/articles";
const NCBI_IDCONV_BASE_ENV: &str = "BIOMCP_NCBI_IDCONV_BASE";

#[derive(Clone)]
pub struct NcbiIdConverterClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
    api_key: Option<String>,
}

impl NcbiIdConverterClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(NCBI_IDCONV_BASE, NCBI_IDCONV_BASE_ENV),
            api_key: crate::sources::ncbi_api_key(),
        })
    }

    async fn get_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode_with_auth(req, self.api_key.is_some())
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::NCBI_ID_CONVERTER,
            ))
            .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::NCBI_ID_CONVERTER),
        )
        .await?;
        crate::sources::decode_json(
            crate::error::SourceContext::retry(crate::error::SourceProvider::NCBI_ID_CONVERTER),
            status,
            content_type.as_ref(),
            &bytes,
            false,
        )
    }

    pub(crate) fn lookup_plan(idtype: &str, id: &str, api_key: Option<&str>) -> RequestPlan {
        let mut plan = RequestPlan::get("")
            .query("format", "json")
            .query("idtype", idtype)
            .query("ids", id);
        if let Some(key) = api_key.map(str::trim).filter(|key| !key.is_empty()) {
            plan = plan.query("api_key", key);
        }
        plan
    }

    async fn lookup(&self, plan: &RequestPlan) -> Result<NcbiIdConvResponse, BioMcpError> {
        let req = request_from_plan(&self.client, self.base.as_ref(), plan);
        self.get_json(req).await
    }

    pub(crate) fn pmid_to_pmcid_plan(
        pmid: &str,
        api_key: Option<&str>,
    ) -> Result<Option<RequestPlan>, BioMcpError> {
        let pmid = pmid.trim();
        if pmid.is_empty() {
            return Ok(None);
        }
        if pmid.len() > 32 {
            return Err(BioMcpError::InvalidArgument("PMID is too long.".into()));
        }
        if !pmid.chars().all(|c| c.is_ascii_digit()) {
            return Err(BioMcpError::InvalidArgument(
                "PMID must contain only digits.".into(),
            ));
        }
        Ok(Some(Self::lookup_plan("pmid", pmid, api_key)))
    }

    pub(crate) fn extract_first_pmcid(resp: NcbiIdConvResponse) -> Option<String> {
        resp.records
            .into_iter()
            .next()
            .and_then(|r| r.pmcid)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    pub async fn pmid_to_pmcid(&self, pmid: &str) -> Result<Option<String>, BioMcpError> {
        let Some(plan) = Self::pmid_to_pmcid_plan(pmid, self.api_key.as_deref())? else {
            return Ok(None);
        };
        let resp = self.lookup(&plan).await?;
        Ok(Self::extract_first_pmcid(resp))
    }

    pub(crate) fn doi_to_pmcid_plan(
        doi: &str,
        api_key: Option<&str>,
    ) -> Result<Option<RequestPlan>, BioMcpError> {
        let doi = doi.trim();
        if doi.is_empty() {
            return Ok(None);
        }
        if doi.len() > 256 {
            return Err(BioMcpError::InvalidArgument("DOI is too long.".into()));
        }
        if !doi.starts_with("10.") || !doi.contains('/') {
            return Err(BioMcpError::InvalidArgument(
                "DOI must start with 10. and include a slash.".into(),
            ));
        }

        Ok(Some(Self::lookup_plan("doi", doi, api_key)))
    }

    pub async fn doi_to_pmcid(&self, doi: &str) -> Result<Option<String>, BioMcpError> {
        let Some(plan) = Self::doi_to_pmcid_plan(doi, self.api_key.as_deref())? else {
            return Ok(None);
        };
        let resp = self.lookup(&plan).await?;
        Ok(resp
            .records
            .into_iter()
            .next()
            .and_then(|r| r.pmcid)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()))
    }
}

#[derive(Debug, Deserialize)]
pub struct NcbiIdConvResponse {
    // dead-code reason: ncbi_idconv::status preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub status: Option<String>,
    #[serde(default)]
    pub records: Vec<NcbiIdConvRecord>,
}

#[derive(Debug, Deserialize)]
pub struct NcbiIdConvRecord {
    pub pmcid: Option<String>,
    // dead-code reason: ncbi_idconv::pmid preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub pmid: Option<u64>,
    // dead-code reason: ncbi_idconv::doi preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub doi: Option<String>,
    // dead-code reason: ncbi_idconv::status preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub status: Option<String>,
    // dead-code reason: ncbi_idconv::errmsg preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub errmsg: Option<String>,
    // dead-code reason: ncbi_idconv::requested_id preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    #[serde(rename = "requested-id")]
    pub requested_id: Option<String>,
}

#[cfg(test)]
mod tests;
