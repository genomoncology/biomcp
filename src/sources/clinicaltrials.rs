use std::borrow::Cow;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

const CTGOV_BASE: &str = "https://clinicaltrials.gov/api/v2";
const CTGOV_API: &str = "clinicaltrials.gov";
const CTGOV_BASE_ENV: &str = "BIOMCP_CTGOV_BASE";
const CTGOV_INTERVENTION_QUERY_ERROR_PREFIX: &str =
    "Error parsing query in Intervention / treatment:";

const CTGOV_SEARCH_FIELDS: &str = "NCTId,BriefTitle,OverallStatus,Phase,StudyType,Condition,InterventionName,LeadSponsorName,EnrollmentCount,BriefSummary,StartDate,CompletionDate,MinimumAge,MaximumAge";
pub const CTGOV_ADVERSE_EVENT_SEARCH_FIELDS: &str = "protocolSection.identificationModule.nctId,protocolSection.identificationModule.briefTitle,hasResults,resultsSection.adverseEventsModule";

const CTGOV_GET_FIELDS_BASE: &[&str] = &[
    "NCTId",
    "BriefTitle",
    "OverallStatus",
    "Phase",
    "StudyType",
    "Condition",
    "InterventionName",
    "InterventionOtherName",
    "LeadSponsorName",
    "EnrollmentCount",
    "BriefSummary",
    "StartDate",
    "CompletionDate",
    "MinimumAge",
    "MaximumAge",
];

const CTGOV_GET_FIELDS_ELIGIBILITY: &[&str] =
    &["EligibilityCriteria", "MinimumAge", "MaximumAge", "Sex"];

const CTGOV_GET_FIELDS_CONTACTS: &[&str] = &[
    "CentralContactName",
    "CentralContactRole",
    "CentralContactPhone",
    "CentralContactEMail",
    "LocationFacility",
    "LocationCity",
    "LocationState",
    "LocationCountry",
    "LocationContactName",
    "LocationContactRole",
    "LocationContactPhone",
    "LocationContactEMail",
];

const CTGOV_GET_FIELDS_LOCATIONS: &[&str] = &[
    "LocationFacility",
    "LocationCity",
    "LocationState",
    "LocationZip",
    "LocationCountry",
    "LocationStatus",
    "LocationContactName",
    "LocationContactRole",
    "LocationContactPhone",
    "LocationContactEMail",
    "CentralContactName",
    "CentralContactRole",
    "CentralContactPhone",
    "CentralContactEMail",
    "LocationGeoPoint",
];

const CTGOV_GET_FIELDS_OUTCOMES: &[&str] = &[
    "PrimaryOutcomeMeasure",
    "PrimaryOutcomeDescription",
    "PrimaryOutcomeTimeFrame",
    "SecondaryOutcomeMeasure",
    "SecondaryOutcomeDescription",
    "SecondaryOutcomeTimeFrame",
];

const CTGOV_GET_FIELDS_ARMS: &[&str] = &[
    "ArmGroupLabel",
    "ArmGroupType",
    "ArmGroupDescription",
    "ArmGroupInterventionName",
    "InterventionType",
    "InterventionName",
    "InterventionOtherName",
    "InterventionDescription",
    "InterventionArmGroupLabel",
];

const CTGOV_GET_FIELDS_REFERENCES: &[&str] =
    &["ReferencePMID", "ReferenceType", "ReferenceCitation"];

#[derive(Clone)]
pub struct ClinicalTrialsClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
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

fn build_get_fields(sections: &[String]) -> String {
    let mut fields: Vec<&str> = CTGOV_GET_FIELDS_BASE.to_vec();
    let mut add_all_sections = false;

    for section in sections {
        match section.trim().to_ascii_lowercase().as_str() {
            "eligibility" => fields.extend_from_slice(CTGOV_GET_FIELDS_ELIGIBILITY),
            "contacts" => fields.extend_from_slice(CTGOV_GET_FIELDS_CONTACTS),
            "locations" => fields.extend_from_slice(CTGOV_GET_FIELDS_LOCATIONS),
            "outcomes" => fields.extend_from_slice(CTGOV_GET_FIELDS_OUTCOMES),
            "arms" => fields.extend_from_slice(CTGOV_GET_FIELDS_ARMS),
            "references" => fields.extend_from_slice(CTGOV_GET_FIELDS_REFERENCES),
            "all" => add_all_sections = true,
            _ => {}
        }
    }

    if add_all_sections {
        fields.extend_from_slice(CTGOV_GET_FIELDS_ELIGIBILITY);
        fields.extend_from_slice(CTGOV_GET_FIELDS_CONTACTS);
        fields.extend_from_slice(CTGOV_GET_FIELDS_LOCATIONS);
        fields.extend_from_slice(CTGOV_GET_FIELDS_OUTCOMES);
        fields.extend_from_slice(CTGOV_GET_FIELDS_ARMS);
        fields.extend_from_slice(CTGOV_GET_FIELDS_REFERENCES);
    }

    fields.sort_unstable();
    fields.dedup();
    fields.join(",")
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
        let resp = crate::sources::apply_cache_mode(req).send().await?;
        let status = resp.status();
        let bytes = crate::sources::read_limited_body(resp, CTGOV_API).await?;
        Ok((status, bytes.to_vec()))
    }

    async fn get_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<T, BioMcpError> {
        let (status, bytes) = self.send(req).await?;
        Self::decode_json_response(status, &bytes)
    }

    pub(crate) fn decode_json_response<T: DeserializeOwned>(
        status: reqwest::StatusCode,
        bytes: &[u8],
    ) -> Result<T, BioMcpError> {
        let body = String::from_utf8_lossy(bytes);
        if status == reqwest::StatusCode::BAD_REQUEST
            && body.starts_with(CTGOV_INTERVENTION_QUERY_ERROR_PREFIX)
        {
            return Err(BioMcpError::CtGovInterventionQueryRejected {
                reason: body.chars().take(256).collect(),
            });
        }
        crate::sources::decode_json(CTGOV_API, status, None, bytes, false)
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

    pub(crate) fn get_plan(nct_id: &str, sections: &[String]) -> RequestPlan {
        RequestPlan::get(format!("studies/{nct_id}")).query("fields", build_get_fields(sections))
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
        let plan = Self::get_plan(nct_id, sections);
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        let (status, bytes) = self.send(req).await?;
        Self::decode_get_response(nct_id, status, &bytes)
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
    #[serde(default)]
    pub interventions: Vec<CtGovIntervention>,
    #[serde(default)]
    pub arm_groups: Vec<CtGovArmGroup>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CtGovIntervention {
    pub name: Option<String>,
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
    pub minimum_age: Option<String>,
    pub maximum_age: Option<String>,
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
    #[serde(default)]
    pub central_contacts: Vec<CtGovContact>,
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
