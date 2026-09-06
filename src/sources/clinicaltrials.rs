use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;

use biodata::{
    ClinicalTrialsGovApiV2DetailPlan, ClinicalTrialsGovApiV2Limits, ClinicalTrialsGovApiV2Response,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

const CTGOV_BASE: &str = "https://clinicaltrials.gov/api/v2";
const CTGOV_BASE_ENV: &str = "BIOMCP_CTGOV_BASE";
const CTGOV_INTERVENTION_QUERY_ERROR_PREFIX: &str =
    "Error parsing query in Intervention / treatment:";

const CTGOV_SEARCH_FIELDS: &str = "NCTId,BriefTitle,OverallStatus,Phase,StudyType,Condition,InterventionName,LeadSponsorName,EnrollmentCount,BriefSummary,StartDate,CompletionDate,MinimumAge,MaximumAge";
pub const CTGOV_ADVERSE_EVENT_SEARCH_FIELDS: &str = "protocolSection.identificationModule.nctId,protocolSection.identificationModule.briefTitle,hasResults,resultsSection.adverseEventsModule";

#[derive(Clone)]
pub struct ClinicalTrialsClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

#[derive(Debug)]
pub(crate) struct CtGovBiodataDetailResponse {
    pub(crate) study: CtGovStudy,
    pub(crate) shared: ClinicalTrialsGovApiV2Response,
}

#[derive(Debug, Clone, Default)]
pub struct CtGovSearchParams {
    pub condition: Option<String>,
    pub intervention: Option<String>,
    pub facility: Option<String>,
    pub status: Option<String>,
    pub agg_filters: Option<String>,
    /// ClinicalTrials.gov advanced query syntax. Multiple terms should be joined by ` AND `.
    pub query_term: Option<String>,
    pub fields_override: Option<String>,
    pub count_total: bool,
    pub page_token: Option<String>,
    pub page_size: usize,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub distance_miles: Option<u32>,
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
            let reason = String::from_utf8_lossy(&bytes[..bytes.len().min(256)]).into_owned();
            return Err(BioMcpError::CtGovInterventionQueryRejected { reason });
        }
        crate::sources::decode_json(
            crate::error::SourceContext::retry(crate::error::SourceProvider::CLINICAL_TRIALS),
            status,
            None,
            bytes,
            false,
        )
    }

    pub(crate) fn search_plan(params: &CtGovSearchParams) -> RequestPlan {
        let mut plan = RequestPlan::get("studies");
        if let Some(v) = params
            .condition
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            plan = plan.query("query.cond", v);
        }
        if let Some(v) = params
            .intervention
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            plan = plan.query("query.intr", v);
        }
        if let Some(v) = params
            .facility
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            plan = plan.query("query.locn", v);
        }
        if let Some(v) = params
            .status
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            plan = plan.query("filter.overallStatus", v);
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
            .query_term
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            plan = plan.query("query.term", v);
        }
        if params.count_total {
            plan = plan.query("countTotal", "true");
        }
        if let Some(v) = params
            .page_token
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            plan = plan.query("pageToken", v);
        }
        if let (Some(lat), Some(lon), Some(distance)) =
            (params.lat, params.lon, params.distance_miles)
        {
            plan = plan.query("filter.geo", format!("distance({lat},{lon},{distance}mi)"));
        }

        let fields = params
            .fields_override
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(CTGOV_SEARCH_FIELDS);
        plan.query("pageSize", params.page_size.to_string())
            .query("fields", fields)
    }

    pub async fn search(
        &self,
        params: &CtGovSearchParams,
    ) -> Result<CtGovSearchResponse, BioMcpError> {
        let plan = Self::search_plan(params);
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        self.get_json(req).await
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
        if has("eligibility") {
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
    ) -> Result<CtGovBiodataDetailResponse, BioMcpError> {
        if !status.is_success() {
            return Self::decode_get_response(nct_id, status, bytes)
                .and(Err(BioMcpError::InternalProcessing));
        }
        let plan = Self::biodata_detail_plan(nct_id, sections)?;
        let shared = ClinicalTrialsGovApiV2Response::parse(
            &plan,
            bytes,
            &ClinicalTrialsGovApiV2Limits::default(),
        )
        .map_err(Self::map_biodata_response_error)?;
        let study = Self::decode_get_response(nct_id, status, bytes)?;
        Ok(CtGovBiodataDetailResponse { study, shared })
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
            message: format!("BioData response validation failed: {}", error.code()),
        }
        .with_source_context(context)
    }

    pub(crate) fn decode_get_response(
        nct_id: &str,
        status: reqwest::StatusCode,
        bytes: &[u8],
    ) -> Result<CtGovStudy, BioMcpError> {
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(BioMcpError::NotFound {
                entity: "trial".into(),
                id: nct_id.to_string(),
                suggestion: format!("Try searching: biomcp search trial -c \"{nct_id}\""),
            });
        }

        Self::decode_json_response(status, bytes)
    }

    pub async fn get(&self, nct_id: &str, sections: &[String]) -> Result<CtGovStudy, BioMcpError> {
        self.get_biodata_detail(nct_id, sections)
            .await
            .map(|value| value.study)
    }

    pub(crate) async fn get_biodata_detail(
        &self,
        nct_id: &str,
        sections: &[String],
    ) -> Result<CtGovBiodataDetailResponse, BioMcpError> {
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

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CtGovSearchResponse {
    #[serde(default)]
    pub studies: Vec<CtGovStudy>,
    pub next_page_token: Option<String>,
    pub total_count: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CtGovStudy {
    pub protocol_section: Option<CtGovProtocolSection>,
    pub document_section: Option<CtGovDocumentSection>,
    pub has_results: Option<bool>,
    pub results_section: Option<CtGovResultsSection>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CtGovProtocolSection {
    pub identification_module: Option<CtGovIdentificationModule>,
    pub status_module: Option<CtGovStatusModule>,
    pub sponsor_collaborators_module: Option<CtGovSponsorCollaboratorsModule>,
    pub description_module: Option<CtGovDescriptionModule>,
    pub conditions_module: Option<CtGovConditionsModule>,
    pub design_module: Option<CtGovDesignModule>,
    pub arms_interventions_module: Option<CtGovArmsInterventionsModule>,
    pub eligibility_module: Option<CtGovEligibilityModule>,
    pub contacts_locations_module: Option<CtGovContactsLocationsModule>,
    pub outcomes_module: Option<CtGovOutcomesModule>,
    pub references_module: Option<CtGovReferencesModule>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CtGovResultsSection {
    pub adverse_events_module: Option<CtGovAdverseEventsModule>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CtGovAdverseEventsModule {
    #[serde(default)]
    pub serious_events: Vec<CtGovAdverseEvent>,
    #[serde(default)]
    pub other_events: Vec<CtGovAdverseEvent>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CtGovAdverseEvent {
    pub term: Option<String>,
    #[serde(default)]
    pub stats: Vec<CtGovAdverseEventStats>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CtGovAdverseEventStats {
    pub group_id: Option<String>,
    pub num_affected: Option<u32>,
    pub num_at_risk: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CtGovIdentificationModule {
    pub nct_id: Option<String>,
    pub brief_title: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CtGovStatusModule {
    pub overall_status: Option<String>,
    pub why_stopped: Option<String>,
    pub start_date_struct: Option<CtGovDateStruct>,
    pub completion_date_struct: Option<CtGovDateStruct>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CtGovDateStruct {
    pub date: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CtGovSponsorCollaboratorsModule {
    pub lead_sponsor: Option<CtGovSponsor>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CtGovSponsor {
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CtGovDescriptionModule {
    pub brief_summary: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CtGovConditionsModule {
    #[serde(default)]
    pub conditions: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CtGovDesignModule {
    pub phases: Option<Vec<String>>,
    pub study_type: Option<String>,
    pub enrollment_info: Option<CtGovEnrollmentInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CtGovEnrollmentInfo {
    pub count: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CtGovArmsInterventionsModule {
    pub interventions: Option<Vec<CtGovIntervention>>,
    pub arm_groups: Option<Vec<CtGovArmGroup>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CtGovIntervention {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub intervention_type: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub other_names: Vec<String>,
    #[serde(default)]
    pub arm_group_labels: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CtGovArmGroup {
    pub label: Option<String>,
    #[serde(rename = "type")]
    pub arm_group_type: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub intervention_names: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CtGovEligibilityModule {
    pub eligibility_criteria: Option<String>,
    pub sex: Option<String>,
    pub minimum_age: Option<NormalizedTimeWire>,
    pub maximum_age: Option<NormalizedTimeWire>,
}

#[derive(Debug, Clone)]
pub struct NormalizedTimeWire(String, Option<crate::entities::trial::TrialAge>);

impl NormalizedTimeWire {
    pub(crate) fn parsed(&self) -> Option<&crate::entities::trial::TrialAge> {
        self.1.as_ref()
    }

    pub(crate) fn original(&self) -> &str {
        &self.0
    }
}

impl Serialize for NormalizedTimeWire {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.original())
    }
}

impl<'de> Deserialize<'de> for NormalizedTimeWire {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let original = String::deserialize(deserializer)?;
        let parsed = crate::entities::trial::TrialAge::from_provider(&original);
        Ok(Self(original, parsed))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CtGovDocumentSection {
    pub large_document_module: Option<CtGovLargeDocumentModule>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CtGovLargeDocumentModule {
    #[serde(default)]
    pub large_docs: Vec<CtGovLargeDocument>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CtGovLargeDocument {
    pub type_abbrev: Option<String>,
    pub label: Option<String>,
    pub date: Option<String>,
    pub upload_date: Option<String>,
    pub filename: Option<String>,
    pub size: Option<u64>,
    pub has_protocol: Option<bool>,
    pub has_sap: Option<bool>,
    pub has_icf: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CtGovContactsLocationsModule {
    #[serde(default)]
    pub central_contacts: Vec<CtGovContact>,
    #[serde(default)]
    pub locations: Vec<CtGovLocation>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CtGovLocation {
    pub facility: Option<String>,
    pub status: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
    pub country: Option<String>,
    #[serde(default)]
    pub contacts: Vec<CtGovContact>,
    pub geo_point: Option<CtGovGeoPoint>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CtGovContact {
    pub name: Option<String>,
    pub role: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CtGovGeoPoint {
    pub lat: Option<f64>,
    pub lon: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CtGovOutcome {
    pub measure: Option<String>,
    pub description: Option<String>,
    pub time_frame: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CtGovOutcomesModule {
    #[serde(default)]
    pub primary_outcomes: Vec<CtGovOutcome>,
    #[serde(default)]
    pub secondary_outcomes: Vec<CtGovOutcome>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CtGovReference {
    pub pmid: Option<String>,
    #[serde(rename = "type")]
    pub reference_type: Option<String>,
    pub citation: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CtGovReferencesModule {
    #[serde(default)]
    pub references: Vec<CtGovReference>,
}

#[cfg(test)]
mod tests;
