use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;

use biodata::{
    NciCtsV2DetailPlan, NciCtsV2DetailResponse, NciCtsV2Limits, NciCtsV2SearchPage,
    NciCtsV2SearchPlan,
};

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

const NCI_CTS_BASE: &str = "https://clinicaltrialsapi.cancer.gov/api/v2";
const NCI_CTS_API: &str = "nci_cts";
const NCI_CTS_BASE_ENV: &str = "BIOMCP_NCI_CTS_BASE";
const NCI_API_KEY_ENV: &str = "NCI_API_KEY";

#[derive(Clone)]
pub struct NciCtsClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
    api_key: String,
}

impl NciCtsClient {
    pub fn new() -> Result<Self, BioMcpError> {
        let api_key = std::env::var(NCI_API_KEY_ENV)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| BioMcpError::ApiKeyRequired {
                api: NCI_CTS_API.to_string(),
                env_var: NCI_API_KEY_ENV.to_string(),
                docs_url: "https://clinicaltrialsapi.cancer.gov/".to_string(),
            })?;

        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(NCI_CTS_BASE, NCI_CTS_BASE_ENV),
            api_key,
        })
    }

    async fn get_search_page(
        &self,
        plan: &NciCtsV2SearchPlan,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<NciCtsV2SearchPage, BioMcpError> {
        let resp = crate::sources::apply_cache_mode_with_auth(req, true)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::NCI_CTS,
            ))
            .await?;
        let status = resp.status();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::NCI_CTS),
        )
        .await?;
        Self::decode_search_response(plan, status, &bytes)
    }

    pub(crate) fn biodata_search_plan(api_key: &str, source: &NciCtsV2SearchPlan) -> RequestPlan {
        let mut plan = RequestPlan::get(source.relative_path());
        for (name, value) in source.query_pairs() {
            plan = plan.query(name, value);
        }
        plan.header("X-API-KEY", api_key)
    }

    pub(crate) fn decode_search_response(
        plan: &NciCtsV2SearchPlan,
        status: reqwest::StatusCode,
        bytes: &[u8],
    ) -> Result<NciCtsV2SearchPage, BioMcpError> {
        if !status.is_success() {
            return match crate::sources::decode_json::<serde_json::Value>(
                crate::error::SourceContext::retry(crate::error::SourceProvider::NCI_CTS),
                status,
                None,
                bytes,
                false,
            ) {
                Err(error) => Err(error),
                Ok(_) => Err(BioMcpError::InternalProcessing),
            };
        }
        NciCtsV2SearchPage::parse(plan, bytes, &NciCtsV2Limits::default()).map_err(|error| {
            let narrow = error.code() == "json_resource_limit";
            Self::detail_api_error(
                format!("response validation failed: {}", error.code()),
                narrow,
            )
        })
    }

    pub async fn search(
        &self,
        plan: &NciCtsV2SearchPlan,
    ) -> Result<NciCtsV2SearchPage, BioMcpError> {
        let request = Self::biodata_search_plan(&self.api_key, plan);
        let req = request_from_plan(&self.client, self.base.as_ref(), &request);
        self.get_search_page(plan, req).await
    }

    /// Build the outbound single-trial request (pure — Tier-2 testable).
    pub(crate) fn get_plan(api_key: &str, detail: &NciCtsV2DetailPlan) -> RequestPlan {
        let mut plan = RequestPlan::get(detail.relative_path());
        for (name, value) in detail.query_pairs() {
            plan = plan.query(name, value);
        }
        plan.header("X-API-KEY", api_key)
    }

    fn detail_not_found(nct_id: &str) -> BioMcpError {
        BioMcpError::NotFound {
            entity: "trial".to_string(),
            id: nct_id.to_string(),
            suggestion: format!("Try searching: biomcp search trial -c \"{nct_id}\""),
        }
    }

    fn detail_api_error(message: String, narrow: bool) -> BioMcpError {
        let context = if narrow {
            crate::error::SourceContext::narrow(crate::error::SourceProvider::NCI_CTS)
        } else {
            crate::error::SourceContext::retry(crate::error::SourceProvider::NCI_CTS)
        };
        BioMcpError::Api {
            api: NCI_CTS_API.to_string(),
            message,
        }
        .with_source_context(context)
    }

    fn detail_status_error(
        detail: &NciCtsV2DetailPlan,
        status: reqwest::StatusCode,
    ) -> Option<BioMcpError> {
        if status == reqwest::StatusCode::NOT_FOUND {
            return Some(Self::detail_not_found(detail.requested_identity()));
        }
        (!status.is_success())
            .then(|| Self::detail_api_error(format!("HTTP {}", status.as_u16()), false))
    }

    pub(crate) fn decode_detail_response(
        detail: &NciCtsV2DetailPlan,
        status: reqwest::StatusCode,
        bytes: &[u8],
    ) -> Result<NciCtsV2DetailResponse, BioMcpError> {
        if let Some(error) = Self::detail_status_error(detail, status) {
            return Err(error);
        }
        NciCtsV2DetailResponse::parse(detail, bytes, &NciCtsV2Limits::default()).map_err(|error| {
            match error.code() {
                "not_found" => Self::detail_not_found(detail.requested_identity()),
                "json_resource_limit" => Self::detail_api_error(
                    format!("response validation failed: {}", error.code()),
                    true,
                ),
                _ => Self::detail_api_error(
                    format!("response validation failed: {}", error.code()),
                    false,
                ),
            }
        })
    }

    pub async fn get(
        &self,
        detail: &NciCtsV2DetailPlan,
    ) -> Result<NciCtsV2DetailResponse, BioMcpError> {
        let plan = Self::get_plan(&self.api_key, detail);
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan)
            .with_extension(crate::sources::StatusBeforeBodyLimit);
        let resp = crate::sources::apply_cache_mode_with_auth(req, true)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::NCI_CTS,
            ))
            .await?;
        let status = resp.status();
        if let Some(error) = Self::detail_status_error(detail, status) {
            return Err(error);
        }
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::NCI_CTS),
        )
        .await?;
        Self::decode_detail_response(detail, status, &bytes)
    }
}

#[cfg(test)]
mod tests;
