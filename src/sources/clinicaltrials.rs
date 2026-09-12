use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;

use biodata::{
    ClinicalTrialsGovApiV2DetailPlan, ClinicalTrialsGovApiV2Limits, ClinicalTrialsGovApiV2Response,
    ClinicalTrialsGovApiV2SearchPage, ClinicalTrialsGovApiV2SearchPlan,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

const CTGOV_BASE: &str = "https://clinicaltrials.gov/api/v2";
const CTGOV_BASE_ENV: &str = "BIOMCP_CTGOV_BASE";
const CTGOV_INTERVENTION_QUERY_ERROR_PREFIX: &str =
    "Error parsing query in Intervention / treatment:";

pub const CTGOV_ADVERSE_EVENT_SEARCH_FIELDS: &str = "protocolSection.identificationModule.nctId,protocolSection.identificationModule.briefTitle,hasResults,resultsSection.adverseEventsModule";

#[derive(Clone)]
pub struct ClinicalTrialsClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

#[derive(Clone, Default)]
pub struct CtGovAdverseEventSearchParams {
    pub intervention: Option<String>,
    pub agg_filters: Option<String>,
    pub page_token: Option<String>,
    pub page_size: usize,
}

impl std::fmt::Debug for CtGovAdverseEventSearchParams {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CtGovAdverseEventSearchParams")
            .field("page_size", &self.page_size)
            .finish_non_exhaustive()
    }
}

impl ClinicalTrialsClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(CTGOV_BASE, CTGOV_BASE_ENV),
        })
    }

    async fn send(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<(reqwest::StatusCode, Vec<u8>), BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::CLINICAL_TRIALS,
            ))
            .await?;
        let status = resp.status();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::CLINICAL_TRIALS),
        )
        .await?;
        Ok((status, bytes))
    }

    async fn get_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<T, BioMcpError> {
        let (status, bytes) = self.send(req).await?;
        Self::decode_json_response(status, &bytes).map_err(|error| {
            error.with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::CLINICAL_TRIALS,
            ))
        })
    }

    pub(crate) fn decode_json_response<T: DeserializeOwned>(
        status: reqwest::StatusCode,
        bytes: &[u8],
    ) -> Result<T, BioMcpError> {
        if status == reqwest::StatusCode::BAD_REQUEST
            && bytes.starts_with(CTGOV_INTERVENTION_QUERY_ERROR_PREFIX.as_bytes())
        {
            return Err(BioMcpError::CtGovInterventionQueryRejected);
        }
        crate::sources::decode_json(
            crate::error::SourceContext::retry(crate::error::SourceProvider::CLINICAL_TRIALS),
            status,
            None,
            bytes,
            false,
        )
    }

    pub(crate) fn biodata_search_plan(plan: &ClinicalTrialsGovApiV2SearchPlan) -> RequestPlan {
        let mut request = RequestPlan::get(plan.relative_path());
        for (name, value) in plan.query_pairs() {
            request = request.query(name, value);
        }
        request
    }

    fn adverse_event_search_plan(params: &CtGovAdverseEventSearchParams) -> RequestPlan {
        let mut plan = RequestPlan::get("studies");
        if let Some(v) = params
            .intervention
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            plan = plan.query("query.intr", v);
        }
        if let Some(v) = params
            .agg_filters
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            plan = plan.query("aggFilters", v);
        }
        if let Some(v) = params
            .page_token
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            plan = plan.query("pageToken", v);
        }
        plan.query("pageSize", params.page_size.to_string())
            .query("fields", CTGOV_ADVERSE_EVENT_SEARCH_FIELDS)
    }

    pub async fn search(
        &self,
        plan: &ClinicalTrialsGovApiV2SearchPlan,
    ) -> Result<ClinicalTrialsGovApiV2SearchPage, BioMcpError> {
        let request = Self::biodata_search_plan(plan);
        let req = request_from_plan(&self.client, self.base.as_ref(), &request);
        let (status, bytes) = self.send(req).await?;
        Self::decode_search_response(plan, status, &bytes)
    }

    pub(crate) async fn search_adverse_events(
        &self,
        params: &CtGovAdverseEventSearchParams,
    ) -> Result<CtGovAdverseEventSearchPage, BioMcpError> {
        let plan = Self::adverse_event_search_plan(params);
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        self.get_json(req).await
    }

    pub(crate) fn decode_search_response(
        plan: &ClinicalTrialsGovApiV2SearchPlan,
        status: reqwest::StatusCode,
        bytes: &[u8],
    ) -> Result<ClinicalTrialsGovApiV2SearchPage, BioMcpError> {
        if !status.is_success() {
            return match Self::decode_json_response::<serde_json::Value>(status, bytes) {
                Err(error) => Err(error),
                Ok(_) => Err(BioMcpError::InternalProcessing),
            };
        }
        ClinicalTrialsGovApiV2SearchPage::parse(
            plan,
            bytes,
            &ClinicalTrialsGovApiV2Limits::default(),
        )
        .map_err(Self::map_biodata_response_error)
    }

    pub(crate) fn biodata_detail_plan(
        nct_id: &str,
        sections: &[String],
    ) -> Result<ClinicalTrialsGovApiV2DetailPlan, BioMcpError> {
        let all = sections
            .iter()
            .any(|value| value.trim().eq_ignore_ascii_case("all"));
        let has = |name: &str| {
            all || sections
                .iter()
                .any(|value| value.trim().eq_ignore_ascii_case(name))
        };
        let mut plan = ClinicalTrialsGovApiV2DetailPlan::new(nct_id, has("references"))
            .map_err(|_| BioMcpError::InternalProcessing)?;
        if has("arms") {
            plan = plan.with_arms();
        }
        if sections.iter().all(|value| {
            let value = value.trim();
            value.is_empty() || value == "--json" || value == "-j"
        }) || has("eligibility")
        {
            plan = plan.with_eligibility();
        }
        if has("contacts") {
            plan = plan.with_contacts();
        }
        if has("locations") {
            plan = plan.with_locations();
        }
        if has("outcomes") {
            plan = plan.with_outcomes();
        }
        if sections
            .iter()
            .any(|value| value.trim().eq_ignore_ascii_case("documents"))
            || sections
                .iter()
                .any(|value| value.trim().eq_ignore_ascii_case("eligibility"))
        {
            plan = plan.with_documents();
        }
        Ok(plan)
    }

    #[cfg(test)]
    pub(crate) fn get_plan(nct_id: &str, sections: &[String]) -> RequestPlan {
        let plan = Self::biodata_detail_plan(nct_id, sections)
            .expect("validated trial identity reaches source planning");
        RequestPlan::get(plan.relative_path()).query("fields", plan.field_query())
    }

    pub(crate) fn decode_biodata_detail_response(
        nct_id: &str,
        sections: &[String],
        status: reqwest::StatusCode,
        bytes: &[u8],
    ) -> Result<ClinicalTrialsGovApiV2Response, BioMcpError> {
        if !status.is_success() {
            return Err(Self::detail_status_error(nct_id, status, bytes));
        }
        let plan = Self::biodata_detail_plan(nct_id, sections)?;
        ClinicalTrialsGovApiV2Response::parse(
            &plan,
            bytes,
            &ClinicalTrialsGovApiV2Limits::default(),
        )
        .map_err(Self::map_biodata_response_error)
    }

    fn detail_status_error(nct_id: &str, status: reqwest::StatusCode, bytes: &[u8]) -> BioMcpError {
        if status == reqwest::StatusCode::NOT_FOUND {
            return BioMcpError::NotFound {
                entity: "trial".into(),
                id: nct_id.to_string(),
                suggestion: format!("Try searching: biomcp search trial -c \"{nct_id}\""),
            };
        }
        match Self::decode_json_response::<serde_json::Value>(status, bytes) {
            Ok(_) => BioMcpError::InternalProcessing,
            Err(error) => error,
        }
    }

    fn map_biodata_response_error(error: biodata::ClinicalTrialsGovApiV2Error) -> BioMcpError {
        let context = if error.code() == "json_resource_limit" {
            crate::error::SourceContext::narrow(crate::error::SourceProvider::CLINICAL_TRIALS)
        } else {
            crate::error::SourceContext::retry(crate::error::SourceProvider::CLINICAL_TRIALS)
        };
        BioMcpError::Api {
            api: crate::error::SourceProvider::CLINICAL_TRIALS
                .label()
                .to_string(),
            message: format!("response validation failed: {}", error.code()),
        }
        .with_source_context(context)
    }

    pub(crate) async fn get_biodata_detail(
        &self,
        nct_id: &str,
        sections: &[String],
    ) -> Result<ClinicalTrialsGovApiV2Response, BioMcpError> {
        let biodata_plan = Self::biodata_detail_plan(nct_id, sections)?;
        let plan = RequestPlan::get(biodata_plan.relative_path())
            .query("fields", biodata_plan.field_query());
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        let (status, bytes) = self.send(req).await?;
        Self::decode_biodata_detail_response(nct_id, sections, status, &bytes).map_err(|error| {
            if matches!(error, BioMcpError::WithSourceContext { .. }) {
                error
            } else {
                error.with_source_context(crate::error::SourceContext::retry(
                    crate::error::SourceProvider::CLINICAL_TRIALS,
                ))
            }
        })
    }
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CtGovAdverseEventSearchPage {
    #[serde(default)]
    pub(crate) studies: Vec<CtGovAdverseEventStudy>,
    pub(crate) next_page_token: Option<String>,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CtGovAdverseEventStudy {
    pub(crate) protocol_section: Option<CtGovAdverseEventProtocol>,
    pub(crate) results_section: Option<CtGovResultsSection>,
}

impl CtGovAdverseEventStudy {
    pub(crate) fn nct_id(&self) -> Option<&str> {
        self.protocol_section
            .as_ref()?
            .identification_module
            .as_ref()?
            .nct_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
    }

    pub(crate) fn counted_terms(&self) -> Vec<&str> {
        let Some(module) = self
            .results_section
            .as_ref()
            .and_then(|section| section.adverse_events_module.as_ref())
        else {
            return Vec::new();
        };
        module
            .serious_events
            .iter()
            .chain(module.other_events.iter())
            .filter(|event| {
                event.stats.is_empty()
                    || event
                        .stats
                        .iter()
                        .any(|stat| stat.num_affected.unwrap_or(0) > 0)
            })
            .filter_map(|event| {
                event
                    .term
                    .as_deref()
                    .map(str::trim)
                    .filter(|term| !term.is_empty())
            })
            .collect()
    }
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CtGovAdverseEventProtocol {
    pub(crate) identification_module: Option<CtGovAdverseEventIdentity>,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CtGovAdverseEventIdentity {
    pub(crate) nct_id: Option<String>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CtGovResultsSection {
    pub(crate) adverse_events_module: Option<CtGovAdverseEventsModule>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CtGovAdverseEventsModule {
    #[serde(default)]
    pub(crate) serious_events: Vec<CtGovAdverseEvent>,
    #[serde(default)]
    pub(crate) other_events: Vec<CtGovAdverseEvent>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CtGovAdverseEvent {
    pub(crate) term: Option<String>,
    #[serde(default)]
    pub(crate) stats: Vec<CtGovAdverseEventStats>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CtGovAdverseEventStats {
    pub(crate) num_affected: Option<u32>,
    pub(crate) num_at_risk: Option<u32>,
}

macro_rules! redacted_debug {
    ($($type:ty),+ $(,)?) => {$ (
        impl std::fmt::Debug for $type {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.debug_struct(stringify!($type)).finish_non_exhaustive()
            }
        }
    )+ };
}

redacted_debug!(
    CtGovAdverseEventSearchPage,
    CtGovAdverseEventStudy,
    CtGovAdverseEventProtocol,
    CtGovAdverseEventIdentity,
    CtGovResultsSection,
    CtGovAdverseEventsModule,
    CtGovAdverseEvent,
    CtGovAdverseEventStats,
);

#[cfg(test)]
mod tests;
