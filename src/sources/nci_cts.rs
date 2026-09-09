use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;

use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::entities::trial::shared::{
    Bound, ClinicalTrialAgeBound, ClinicalTrialAgeRange, ClinicalTrialArm, ClinicalTrialArmId,
    ClinicalTrialArmInterventionAssignment, ClinicalTrialEligibility,
    ClinicalTrialEligibilityClassification, ClinicalTrialEligibilityCriterion,
    ClinicalTrialEligibilityCriterionId, ClinicalTrialIntervention, ClinicalTrialInterventionId,
    ClinicalTrialSection, ExtensibleCode, ParseOutcome, TemporalParser,
};
use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

const NCI_CTS_BASE: &str = "https://clinicaltrialsapi.cancer.gov/api/v2";
const NCI_CTS_API: &str = "nci_cts";
const NCI_CTS_BASE_ENV: &str = "BIOMCP_NCI_CTS_BASE";
const NCI_API_KEY_ENV: &str = "NCI_API_KEY";
const NCI_DETAIL_FIELDS: &[&str] = &[
    "nci_id",
    "nct_id",
    "brief_title",
    "official_title",
    "current_trial_status",
    "why_study_stopped",
    "study_protocol_type",
    "phase",
    "diseases",
    "minimum_target_accrual_number",
    "arms",
    "lead_org",
    "start_date",
    "completion_date",
    "eligibility",
    "brief_summary",
];

#[derive(Debug, Clone)]
pub(crate) struct NciCtsV2DetailPlan {
    identity: String,
    include_eligibility: bool,
}
impl NciCtsV2DetailPlan {
    pub(crate) fn new(identity: &str, include_eligibility: bool) -> Result<Self, ()> {
        let identity = identity.trim().to_ascii_uppercase();
        if identity.len() != 11
            || !identity.starts_with("NCT")
            || !identity[3..].bytes().all(|b| b.is_ascii_digit())
        {
            return Err(());
        }
        Ok(Self {
            identity,
            include_eligibility,
        })
    }
    pub(crate) fn requested_identity(&self) -> &str {
        &self.identity
    }
    pub(crate) fn relative_path(&self) -> &str {
        "trials"
    }
    pub(crate) fn query_pairs(&self) -> Vec<(&str, &str)> {
        let mut out = vec![("size", "1"), ("nct_id", self.identity.as_str())];
        out.extend(
            NCI_DETAIL_FIELDS
                .iter()
                .filter(|field| self.include_eligibility || **field != "eligibility")
                .map(|field| ("include", *field)),
        );
        out
    }
}

#[derive(Debug)]
pub(crate) struct NciCtsV2DetailResponse {
    pub(crate) title: String,
    pub(crate) status: String,
    pub(crate) stop_reason: Option<String>,
    pub(crate) phase: Option<String>,
    pub(crate) study_type: String,
    pub(crate) conditions: Vec<String>,
    pub(crate) interventions: Vec<ClinicalTrialIntervention>,
    pub(crate) arms: Vec<ClinicalTrialArm>,
    pub(crate) assignments: Vec<ClinicalTrialArmInterventionAssignment>,
    pub(crate) sponsor: String,
    pub(crate) enrollment: Option<u64>,
    pub(crate) summary: Option<String>,
    pub(crate) start_date: Option<String>,
    pub(crate) completion_date: Option<String>,
    eligibility: ClinicalTrialSection<ClinicalTrialEligibility>,
}
impl NciCtsV2DetailResponse {
    pub(crate) fn eligibility(&self) -> ClinicalTrialSection<&ClinicalTrialEligibility> {
        match &self.eligibility {
            ClinicalTrialSection::Present(v) => ClinicalTrialSection::Present(v),
            ClinicalTrialSection::Absent => ClinicalTrialSection::Absent,
            ClinicalTrialSection::NotRequested => ClinicalTrialSection::NotRequested,
            ClinicalTrialSection::Unavailable => ClinicalTrialSection::Unavailable,
        }
    }

    pub(crate) fn parse(plan: &NciCtsV2DetailPlan, bytes: &[u8]) -> Result<Self, &'static str> {
        crate::entities::trial::strict_json::validate(bytes).map_err(|error| match error {
            crate::entities::trial::strict_json::StrictJsonError::Malformed => "malformed_json",
            crate::entities::trial::strict_json::StrictJsonError::Unsupported => "unsupported_json",
            crate::entities::trial::strict_json::StrictJsonError::Resource => "json_resource_limit",
        })?;
        let root: serde_json::Value =
            serde_json::from_slice(bytes).map_err(|_| "malformed_json")?;
        let root = root.as_object().ok_or("unsupported_json")?;
        let total = root
            .get("total")
            .and_then(serde_json::Value::as_u64)
            .ok_or("invalid_projection")?;
        let rows = root
            .get("data")
            .and_then(serde_json::Value::as_array)
            .ok_or("invalid_projection")?;
        if total == 0 && rows.is_empty() {
            return Err("not_found");
        }
        if total != 1 || rows.len() != 1 {
            return Err("unexpected_row_count");
        }
        let row = rows[0].as_object().ok_or("unsupported_json")?;
        let required = |name: &str| {
            row.get(name)
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_owned)
                .ok_or("invalid_projection")
        };
        let nullable = |name: &str| match row.get(name) {
            None => Err("invalid_projection"),
            Some(serde_json::Value::Null) => Ok(None),
            Some(value) => value
                .as_str()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_owned)
                .map(Some)
                .ok_or("invalid_projection"),
        };
        let _nci_id = required("nci_id")?;
        let _official_title = nullable("official_title")?;
        let identity = required("nct_id")?;
        if identity != plan.requested_identity() {
            return Err("identity_mismatch");
        }
        let diseases = row
            .get("diseases")
            .and_then(serde_json::Value::as_array)
            .ok_or("invalid_projection")?
            .iter()
            .map(|value| {
                value
                    .as_object()
                    .and_then(|v| v.get("name"))
                    .and_then(serde_json::Value::as_str)
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                    .map(str::to_owned)
                    .ok_or("invalid_projection")
            })
            .collect::<Result<Vec<_>, _>>()?;
        let arm_rows = row
            .get("arms")
            .and_then(serde_json::Value::as_array)
            .ok_or("invalid_projection")?;
        let mut arms = Vec::new();
        let mut interventions = Vec::new();
        let mut assignments = Vec::new();
        for (arm_index, value) in arm_rows.iter().enumerate() {
            let arm = value.as_object().ok_or("unsupported_json")?;
            let string = |name: &str| {
                arm.get(name)
                    .and_then(serde_json::Value::as_str)
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                    .map(str::to_owned)
                    .ok_or("invalid_projection")
            };
            let arm_id = ClinicalTrialArmId::new((arm_index + 1) as u64)
                .map_err(|_| "invalid_projection")?;
            arms.push(
                ClinicalTrialArm::new(
                    arm_id,
                    string("name")?,
                    code("nci", arm.get("type").and_then(serde_json::Value::as_str))?,
                    optional_object_string(arm, "description")?,
                )
                .map_err(|_| "invalid_projection")?,
            );
            for value in arm
                .get("interventions")
                .and_then(serde_json::Value::as_array)
                .ok_or("invalid_projection")?
            {
                let value = value.as_object().ok_or("unsupported_json")?;
                let id = ClinicalTrialInterventionId::new((interventions.len() + 1) as u64)
                    .map_err(|_| "invalid_projection")?;
                let name = value
                    .get("name")
                    .and_then(serde_json::Value::as_str)
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                    .ok_or("invalid_projection")?;
                let aliases = value
                    .get("synonyms")
                    .and_then(serde_json::Value::as_array)
                    .map(|rows| {
                        rows.iter()
                            .map(|v| {
                                v.as_str()
                                    .map(str::trim)
                                    .filter(|v| !v.is_empty())
                                    .map(str::to_owned)
                                    .ok_or("invalid_projection")
                            })
                            .collect()
                    })
                    .transpose()?
                    .unwrap_or_default();
                interventions.push(
                    ClinicalTrialIntervention::new(
                        id,
                        name,
                        code("nci", value.get("type").and_then(serde_json::Value::as_str))?,
                        optional_object_string(value, "description")?,
                        Some(aliases),
                    )
                    .map_err(|_| "invalid_projection")?,
                );
                assignments.push(ClinicalTrialArmInterventionAssignment::new(arm_id, id));
            }
        }
        let eligibility = if !plan.include_eligibility {
            ClinicalTrialSection::NotRequested
        } else if row
            .get("eligibility")
            .is_some_and(serde_json::Value::is_null)
        {
            ClinicalTrialSection::Absent
        } else if let Some(value) = row.get("eligibility") {
            let value = value.as_object().ok_or("unsupported_json")?;
            let structured = value
                .get("structured")
                .and_then(serde_json::Value::as_object)
                .ok_or("unsupported_json")?;
            let minimum = nci_age(structured.get("min_age"), Bound::Minimum, false)?;
            let maximum = nci_age(structured.get("max_age"), Bound::Maximum, true)?;
            let age_range = (minimum.is_some() || maximum.is_some())
                .then(|| ClinicalTrialAgeRange::new(minimum, maximum))
                .transpose()
                .map_err(|_| "invalid_projection")?;
            let sexes = code(
                "nci",
                structured.get("sex").and_then(serde_json::Value::as_str),
            )?
            .map(|v| vec![v]);
            let healthy = match structured.get("accepts_healthy_volunteers") {
                None | Some(serde_json::Value::Null) => None,
                Some(v) => Some(v.as_bool().ok_or("invalid_projection")?),
            };
            let rows = match value.get("unstructured") {
                None | Some(serde_json::Value::Null) => None,
                Some(value) => Some(value.as_array().ok_or("unsupported_json")?),
            };
            let mut criteria = rows
                .map(|rows| {
                    rows.iter()
                        .enumerate()
                        .map(|(index, value)| {
                            let value = value.as_object().ok_or("unsupported_json")?;
                            let description = value
                                .get("description")
                                .and_then(serde_json::Value::as_str)
                                .ok_or("invalid_projection")?;
                            let inclusion = value
                                .get("inclusion_indicator")
                                .and_then(serde_json::Value::as_bool)
                                .ok_or("invalid_projection")?;
                            let order = value
                                .get("display_order")
                                .and_then(serde_json::Value::as_i64)
                                .ok_or("invalid_projection")?;
                            let criterion = ClinicalTrialEligibilityCriterion::new(
                                ClinicalTrialEligibilityCriterionId::new((index + 1) as u64)
                                    .map_err(|_| "invalid_projection")?,
                                description,
                                if inclusion {
                                    ClinicalTrialEligibilityClassification::Inclusion
                                } else {
                                    ClinicalTrialEligibilityClassification::Exclusion
                                },
                            )
                            .map_err(|_| "invalid_projection")?;
                            Ok((order, index, criterion))
                        })
                        .collect::<Result<Vec<_>, &'static str>>()
                })
                .transpose()?;
            if let Some(criteria) = &mut criteria {
                criteria.sort_by_key(|(order, index, _)| (*order, *index));
            }
            ClinicalTrialSection::Present(
                ClinicalTrialEligibility::new(
                    None,
                    age_range,
                    sexes,
                    healthy,
                    criteria.map(|rows| rows.into_iter().map(|(_, _, value)| value).collect()),
                )
                .map_err(|_| "invalid_projection")?,
            )
        } else {
            ClinicalTrialSection::Absent
        };
        let enrollment = match row.get("minimum_target_accrual_number") {
            None | Some(serde_json::Value::Null) => None,
            Some(v) => Some(v.as_u64().ok_or("invalid_projection")?),
        };
        Ok(Self {
            title: required("brief_title")?,
            status: required("current_trial_status")?,
            stop_reason: nullable("why_study_stopped")?,
            phase: nullable("phase")?,
            study_type: required("study_protocol_type")?,
            conditions: diseases,
            interventions,
            arms,
            assignments,
            sponsor: required("lead_org")?,
            enrollment,
            summary: nullable("brief_summary")?,
            start_date: nullable("start_date")?,
            completion_date: nullable("completion_date")?,
            eligibility,
        })
    }
}

fn optional_object_string(
    value: &serde_json::Map<String, serde_json::Value>,
    name: &str,
) -> Result<Option<String>, &'static str> {
    match value.get(name) {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(value) => value
            .as_str()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_owned)
            .map(Some)
            .ok_or("invalid_projection"),
    }
}

fn code(authority: &str, value: Option<&str>) -> Result<Option<ExtensibleCode>, &'static str> {
    value
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|v| {
            ExtensibleCode::new(authority, v, None::<String>, None::<String>, None::<String>)
                .map_err(|_| "invalid_projection")
        })
        .transpose()
}

fn nci_age(
    value: Option<&serde_json::Value>,
    bound: Bound,
    recognize_no_limit: bool,
) -> Result<Option<ClinicalTrialAgeBound>, &'static str> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.as_str().ok_or("invalid_projection")?;
    let ParseOutcome::Parsed(duration) = TemporalParser::default().parse_duration(value, bound)
    else {
        return Err("invalid_projection");
    };
    if recognize_no_limit && value == "999 Years" {
        ClinicalTrialAgeBound::source_stated_no_limit(duration)
            .map(Some)
            .map_err(|_| "invalid_projection")
    } else {
        ClinicalTrialAgeBound::limited(duration)
            .map(Some)
            .map_err(|_| "invalid_projection")
    }
}

#[derive(Clone)]
pub struct NciCtsClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
    api_key: String,
}

#[derive(Debug, Clone)]
pub enum NciDiseaseFilter {
    Keyword(String),
    ConceptId(String),
}

#[derive(Debug, Clone)]
pub enum NciStatusFilter {
    CurrentTrialStatus(String),
    SiteRecruitmentStatus(String),
}

#[derive(Debug, Clone)]
pub struct NciGeoFilter {
    pub lat: f64,
    pub lon: f64,
    pub distance_miles: u32,
}

#[derive(Debug, Clone, Default)]
pub struct NciSearchParams {
    pub disease: Option<NciDiseaseFilter>,
    pub interventions: Option<String>,
    pub sites_org_name: Option<String>,
    pub status: Option<NciStatusFilter>,
    pub phases: Vec<String>,
    pub geo: Option<NciGeoFilter>,
    pub biomarkers: Option<String>,
    pub size: usize,
    pub from: usize,
}

#[derive(Debug, Deserialize)]
pub struct NciSearchResponse {
    #[serde(default)]
    pub data: Vec<serde_json::Value>,
    #[serde(default)]
    pub trials: Vec<serde_json::Value>,
    #[serde(default, alias = "total", alias = "total_count", alias = "totalCount")]
    pub total: Option<usize>,
}

impl NciSearchResponse {
    pub fn hits(&self) -> &[serde_json::Value] {
        if !self.data.is_empty() {
            &self.data
        } else {
            &self.trials
        }
    }
}

fn trimmed_non_empty(value: &str) -> Option<&str> {
    let value = value.trim();
    (!value.is_empty()).then_some(value)
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

    async fn get_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<T, BioMcpError> {
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
        crate::sources::decode_json(
            crate::error::SourceContext::retry(crate::error::SourceProvider::NCI_CTS),
            status,
            None,
            &bytes,
            false,
        )
    }

    /// Build the outbound trials-search request (pure — Tier-2 testable, never sent).
    pub(crate) fn search_plan(api_key: &str, params: &NciSearchParams) -> RequestPlan {
        let mut plan = RequestPlan::get("trials").header("X-API-KEY", api_key);

        if let Some(disease) = &params.disease {
            match disease {
                NciDiseaseFilter::Keyword(v) => {
                    if let Some(v) = trimmed_non_empty(v.as_str()) {
                        plan = plan.query("keyword", v);
                    }
                }
                NciDiseaseFilter::ConceptId(v) => {
                    if let Some(v) = trimmed_non_empty(v.as_str()) {
                        plan = plan.query("diseases.nci_thesaurus_concept_id", v);
                    }
                }
            }
        }
        if let Some(v) = params
            .interventions
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            plan = plan.query("interventions", v);
        }
        if let Some(v) = params
            .sites_org_name
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            plan = plan.query("sites.org_name", v);
        }
        if let Some(status) = &params.status {
            match status {
                NciStatusFilter::CurrentTrialStatus(v) => {
                    if let Some(v) = trimmed_non_empty(v.as_str()) {
                        plan = plan.query("current_trial_status", v);
                    }
                }
                NciStatusFilter::SiteRecruitmentStatus(v) => {
                    if let Some(v) = trimmed_non_empty(v.as_str()) {
                        plan = plan.query("sites.recruitment_status", v);
                    }
                }
            }
        }
        for phase in &params.phases {
            let phase = phase.trim();
            if phase.is_empty() {
                continue;
            }
            plan = plan.query("phase", phase);
        }
        if let Some(geo) = &params.geo {
            plan = plan.query("sites.org_coordinates_lat", geo.lat.to_string());
            plan = plan.query("sites.org_coordinates_lon", geo.lon.to_string());
            plan = plan.query(
                "sites.org_coordinates_dist",
                format!("{}mi", geo.distance_miles),
            );
        }
        if let Some(v) = params
            .biomarkers
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            plan = plan.query("biomarkers", v);
        }

        plan = plan.query("size", params.size.to_string());
        plan = plan.query("from", params.from.to_string());
        plan
    }

    pub async fn search(&self, params: &NciSearchParams) -> Result<NciSearchResponse, BioMcpError> {
        let plan = Self::search_plan(&self.api_key, params);
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        self.get_json(req).await
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
        NciCtsV2DetailResponse::parse(detail, bytes).map_err(|code| match code {
            "not_found" => Self::detail_not_found(detail.requested_identity()),
            "json_resource_limit" => {
                Self::detail_api_error(format!("response validation failed: {code}"), true)
            }
            _ => Self::detail_api_error(format!("response validation failed: {code}"), false),
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
