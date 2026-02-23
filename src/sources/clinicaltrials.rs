use std::borrow::Cow;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::BioMcpError;

const CTGOV_BASE: &str = "https://clinicaltrials.gov/api/v2";
const CTGOV_API: &str = "clinicaltrials.gov";
const CTGOV_BASE_ENV: &str = "BIOMCP_CTGOV_BASE";

const CTGOV_SEARCH_FIELDS: &str = "NCTId,BriefTitle,OverallStatus,Phase,StudyType,Condition,InterventionName,LeadSponsorName,EnrollmentCount,BriefSummary,StartDate,CompletionDate";

const CTGOV_GET_FIELDS_BASE: &[&str] = &[
    "NCTId",
    "BriefTitle",
    "OverallStatus",
    "Phase",
    "StudyType",
    "Condition",
    "InterventionName",
    "LeadSponsorName",
    "EnrollmentCount",
    "BriefSummary",
    "StartDate",
    "CompletionDate",
    "MinimumAge",
    "MaximumAge",
];

const CTGOV_GET_FIELDS_ELIGIBILITY: &[&str] = &["EligibilityCriteria"];

const CTGOV_GET_FIELDS_LOCATIONS: &[&str] = &[
    "LocationFacility",
    "LocationCity",
    "LocationState",
    "LocationZip",
    "LocationCountry",
    "LocationStatus",
    "LocationContactName",
    "LocationContactPhone",
    "LocationContactEMail",
    "CentralContactName",
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
        let bytes = crate::sources::read_limited_body(resp, CTGOV_API).await?;
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: CTGOV_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }
        serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
            api: CTGOV_API.to_string(),
            source,
        })
    }

    pub async fn search(
        &self,
        params: &CtGovSearchParams,
    ) -> Result<CtGovSearchResponse, BioMcpError> {
        let url = self.endpoint("studies");

        let mut req = self.client.get(&url);
        if let Some(v) = params
            .condition
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            req = req.query(&[("query.cond", v)]);
        }
        if let Some(v) = params
            .intervention
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            req = req.query(&[("query.intr", v)]);
        }
        if let Some(v) = params
            .facility
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            req = req.query(&[("query.locn", v)]);
        }
        if let Some(v) = params
            .status
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            req = req.query(&[("filter.overallStatus", v)]);
        }
        if let Some(v) = params
            .agg_filters
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            req = req.query(&[("aggFilters", v)]);
        }
        if let Some(v) = params
            .query_term
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            req = req.query(&[("query.term", v)]);
        }
        if params.count_total {
            req = req.query(&[("countTotal", "true")]);
        }
        if let Some(v) = params
            .page_token
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            req = req.query(&[("pageToken", v)]);
        }
        if let (Some(lat), Some(lon), Some(distance)) =
            (params.lat, params.lon, params.distance_miles)
        {
            let filter_geo = format!("distance({lat},{lon},{distance}mi)");
            req = req.query(&[("filter.geo", filter_geo.as_str())]);
        }

        let page_size = params.page_size.to_string();
        req = req.query(&[
            ("pageSize", page_size.as_str()),
            ("fields", CTGOV_SEARCH_FIELDS),
        ]);

        self.get_json(req).await
    }

    pub async fn get(&self, nct_id: &str, sections: &[String]) -> Result<CtGovStudy, BioMcpError> {
        let url = self.endpoint(&format!("studies/{nct_id}"));
        let fields = build_get_fields(sections);

        let req = self.client.get(&url).query(&[("fields", fields.as_str())]);
        let resp = crate::sources::apply_cache_mode(req).send().await?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(BioMcpError::NotFound {
                entity: "trial".into(),
                id: nct_id.to_string(),
                suggestion: format!("Try searching: biomcp search trial -c \"{nct_id}\""),
            });
        }

        let status = resp.status();
        let bytes = crate::sources::read_limited_body(resp, CTGOV_API).await?;
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: CTGOV_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }

        serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
            api: CTGOV_API.to_string(),
            source,
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
    pub minimum_age: Option<String>,
    pub maximum_age: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CtGovContactsLocationsModule {
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
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn search_builds_expected_params() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/studies"))
            .and(query_param("query.cond", "melanoma"))
            .and(query_param("query.intr", "pembrolizumab"))
            .and(query_param("filter.overallStatus", "RECRUITING"))
            .and(query_param("query.term", "AREA[Phase]PHASE2"))
            .and(query_param("countTotal", "true"))
            .and(query_param("pageSize", "3"))
            .and(query_param("fields", CTGOV_SEARCH_FIELDS))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "studies": [],
                "nextPageToken": null
            })))
            .mount(&server)
            .await;

        let client = ClinicalTrialsClient::new_for_test(server.uri()).unwrap();
        let _ = client
            .search(&CtGovSearchParams {
                condition: Some("melanoma".into()),
                intervention: Some("pembrolizumab".into()),
                facility: None,
                status: Some("RECRUITING".into()),
                agg_filters: None,
                query_term: Some("AREA[Phase]PHASE2".into()),
                count_total: true,
                page_token: None,
                page_size: 3,
                lat: None,
                lon: None,
                distance_miles: None,
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn search_includes_geo_filter_when_requested() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/studies"))
            .and(query_param("query.cond", "melanoma"))
            .and(query_param("filter.geo", "distance(41.5,-81.7,50mi)"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "studies": [],
                "nextPageToken": null
            })))
            .mount(&server)
            .await;

        let client = ClinicalTrialsClient::new_for_test(server.uri()).unwrap();
        let _ = client
            .search(&CtGovSearchParams {
                condition: Some("melanoma".into()),
                intervention: None,
                facility: None,
                status: None,
                agg_filters: None,
                query_term: None,
                count_total: false,
                page_token: None,
                page_size: 10,
                lat: Some(41.5),
                lon: Some(-81.7),
                distance_miles: Some(50),
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn search_includes_facility_and_agg_filters_when_requested() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/studies"))
            .and(query_param("query.cond", "melanoma"))
            .and(query_param("query.locn", "MD Anderson"))
            .and(query_param("aggFilters", "sex:f,funderType:nih"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "studies": [],
                "nextPageToken": null
            })))
            .mount(&server)
            .await;

        let client = ClinicalTrialsClient::new_for_test(server.uri()).unwrap();
        let _ = client
            .search(&CtGovSearchParams {
                condition: Some("melanoma".into()),
                intervention: None,
                facility: Some("MD Anderson".into()),
                status: None,
                agg_filters: Some("sex:f,funderType:nih".into()),
                query_term: None,
                count_total: false,
                page_token: None,
                page_size: 5,
                lat: None,
                lon: None,
                distance_miles: None,
            })
            .await
            .unwrap();
    }
}
