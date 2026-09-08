use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::entities::trial::shared::{
    Bound, ClinicalTrialAgeBound, ClinicalTrialAgeRange, ClinicalTrialArm, ClinicalTrialArmId,
    ClinicalTrialArmInterventionAssignment, ClinicalTrialArms, ClinicalTrialEligibility,
    ClinicalTrialIntervention, ClinicalTrialInterventionId, ClinicalTrialReference,
    ClinicalTrialSection, ExtensibleCode, ParseOutcome, TemporalParser,
};
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
pub(crate) struct CtGovDetailResponse {
    pub(crate) study: CtGovStudy,
    pub(crate) shared: CtGovProjection,
}

#[derive(Debug)]
pub(crate) struct CtGovProjection {
    interventions: ClinicalTrialSection<Vec<ClinicalTrialIntervention>>,
    arms: ClinicalTrialSection<ClinicalTrialArms>,
    eligibility: ClinicalTrialSection<ClinicalTrialEligibility>,
    references: ClinicalTrialSection<Vec<ClinicalTrialReference>>,
}

impl CtGovProjection {
    pub(crate) fn interventions(&self) -> &ClinicalTrialSection<Vec<ClinicalTrialIntervention>> {
        &self.interventions
    }
    pub(crate) fn arms(&self) -> &ClinicalTrialSection<ClinicalTrialArms> {
        &self.arms
    }
    pub(crate) fn eligibility(&self) -> &ClinicalTrialSection<ClinicalTrialEligibility> {
        &self.eligibility
    }
    pub(crate) fn references(&self) -> &ClinicalTrialSection<Vec<ClinicalTrialReference>> {
        &self.references
    }
}

#[derive(Debug)]
pub(crate) struct CtGovDetailPlan {
    identity: String,
    fields: String,
}
impl CtGovDetailPlan {
    pub(crate) fn relative_path(&self) -> String {
        format!("studies/{}", self.identity)
    }
    pub(crate) fn field_query(&self) -> &str {
        &self.fields
    }
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

    pub(crate) fn detail_plan(
        nct_id: &str,
        sections: &[String],
    ) -> Result<CtGovDetailPlan, BioMcpError> {
        let all = sections
            .iter()
            .any(|value| value.trim().eq_ignore_ascii_case("all"));
        let has = |name: &str| {
            all || sections
                .iter()
                .any(|value| value.trim().eq_ignore_ascii_case(name))
        };
        let mut fields = [
            "BriefSummary",
            "BriefTitle",
            "CompletionDate",
            "Condition",
            "EnrollmentCount",
            "InterventionDescription",
            "InterventionName",
            "InterventionOtherName",
            "InterventionType",
            "LeadSponsorName",
            "NCTId",
            "OverallStatus",
            "Phase",
            "StartDate",
            "StudyType",
            "WhyStopped",
        ]
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
        let mut add = |values: &[&'static str]| fields.extend(values.iter().copied());
        if has("references") {
            add(&["ReferenceCitation", "ReferencePMID", "ReferenceType"]);
        }
        if has("arms") {
            add(&[
                "ArmGroupDescription",
                "ArmGroupInterventionName",
                "ArmGroupLabel",
                "ArmGroupType",
                "InterventionArmGroupLabel",
            ]);
        }
        if sections.iter().all(|value| {
            let value = value.trim();
            value.is_empty() || value == "--json" || value == "-j"
        }) || has("eligibility")
        {
            add(&[
                "EligibilityCriteria",
                "HealthyVolunteers",
                "MaximumAge",
                "MinimumAge",
                "Sex",
            ]);
        }
        if has("contacts") {
            add(&[
                "CentralContactEMail",
                "CentralContactName",
                "CentralContactPhone",
                "CentralContactRole",
                "LocationCity",
                "LocationContactEMail",
                "LocationContactName",
                "LocationContactPhone",
                "LocationContactRole",
                "LocationCountry",
                "LocationFacility",
                "LocationState",
            ]);
        }
        if has("locations") {
            add(&[
                "LocationCity",
                "LocationContactEMail",
                "LocationContactName",
                "LocationContactPhone",
                "LocationContactRole",
                "LocationCountry",
                "LocationFacility",
                "LocationGeoPoint",
                "LocationState",
                "LocationStatus",
                "LocationZip",
            ]);
        }
        if has("outcomes") {
            add(&[
                "PrimaryOutcomeDescription",
                "PrimaryOutcomeMeasure",
                "PrimaryOutcomeTimeFrame",
                "SecondaryOutcomeDescription",
                "SecondaryOutcomeMeasure",
                "SecondaryOutcomeTimeFrame",
            ]);
        }
        if sections
            .iter()
            .any(|value| value.trim().eq_ignore_ascii_case("documents"))
            || sections
                .iter()
                .any(|value| value.trim().eq_ignore_ascii_case("eligibility"))
        {
            add(&["LargeDocumentModule"]);
        }
        Ok(CtGovDetailPlan {
            identity: nct_id.to_string(),
            fields: fields.into_iter().collect::<Vec<_>>().join(","),
        })
    }

    #[cfg(test)]
    pub(crate) fn get_plan(nct_id: &str, sections: &[String]) -> RequestPlan {
        let plan = Self::detail_plan(nct_id, sections)
            .expect("validated trial identity reaches source planning");
        RequestPlan::get(plan.relative_path()).query("fields", plan.field_query())
    }

    pub(crate) fn decode_detail_response(
        nct_id: &str,
        sections: &[String],
        status: reqwest::StatusCode,
        bytes: &[u8],
    ) -> Result<CtGovDetailResponse, BioMcpError> {
        if !status.is_success() {
            return Self::decode_get_response(nct_id, status, bytes)
                .and(Err(BioMcpError::InternalProcessing));
        }
        crate::entities::trial::strict_json::validate(bytes).map_err(|error| {
            Self::map_detail_response_error(match error {
                crate::entities::trial::strict_json::StrictJsonError::Malformed => "malformed_json",
                crate::entities::trial::strict_json::StrictJsonError::Unsupported => {
                    "unsupported_json"
                }
                crate::entities::trial::strict_json::StrictJsonError::Resource => {
                    "json_resource_limit"
                }
            })
        })?;
        let value: serde_json::Value = serde_json::from_slice(bytes)
            .map_err(|_| Self::map_detail_response_error("malformed_json"))?;
        if !value.is_object() {
            return Err(Self::map_detail_response_error("unsupported_json"));
        }
        let study: CtGovStudy = serde_json::from_value(value)
            .map_err(|_| Self::map_detail_response_error("invalid_projection"))?;
        let actual = study
            .protocol_section
            .as_ref()
            .and_then(|p| p.identification_module.as_ref())
            .and_then(|m| m.nct_id.as_deref())
            .ok_or_else(|| Self::map_detail_response_error("invalid_projection"))?;
        if actual != nct_id {
            return Err(Self::map_detail_response_error("identity_mismatch"));
        }
        let shared = CtGovProjection::from_study(&study, sections)
            .map_err(Self::map_detail_response_error)?;
        Ok(CtGovDetailResponse { study, shared })
    }

    fn map_detail_response_error(code: &str) -> BioMcpError {
        let context = if code == "json_resource_limit" {
            crate::error::SourceContext::narrow(crate::error::SourceProvider::CLINICAL_TRIALS)
        } else {
            crate::error::SourceContext::retry(crate::error::SourceProvider::CLINICAL_TRIALS)
        };
        BioMcpError::Api {
            api: crate::error::SourceProvider::CLINICAL_TRIALS
                .label()
                .to_string(),
            message: format!("response validation failed: {code}"),
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
        self.get_detail(nct_id, sections)
            .await
            .map(|value| value.study)
    }

    pub(crate) async fn get_detail(
        &self,
        nct_id: &str,
        sections: &[String],
    ) -> Result<CtGovDetailResponse, BioMcpError> {
        let detail = Self::detail_plan(nct_id, sections)?;
        let plan = RequestPlan::get(detail.relative_path()).query("fields", detail.field_query());
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        let (status, bytes) = self.send(req).await?;
        Self::decode_detail_response(nct_id, sections, status, &bytes).map_err(|error| {
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

fn requested(sections: &[String], name: &str) -> bool {
    sections.iter().any(|value| {
        let value = value.trim();
        value.eq_ignore_ascii_case("all") || value.eq_ignore_ascii_case(name)
    })
}

fn clean(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn code(authority: &str, value: Option<&str>) -> Result<Option<ExtensibleCode>, &'static str> {
    clean(value)
        .map(|value| {
            ExtensibleCode::new(
                authority,
                value,
                None::<String>,
                None::<String>,
                None::<String>,
            )
            .map_err(|_| "invalid_projection")
        })
        .transpose()
}

fn age(
    value: Option<&NormalizedTimeWire>,
    bound: Bound,
) -> Result<Option<ClinicalTrialAgeBound>, &'static str> {
    let Some(value) = value else {
        return Ok(None);
    };
    match TemporalParser::default().parse_duration(value.original().trim(), bound) {
        ParseOutcome::Parsed(value) => ClinicalTrialAgeBound::limited(value)
            .map(Some)
            .map_err(|_| "invalid_projection"),
        ParseOutcome::Unparsed => Err("invalid_projection"),
    }
}

impl CtGovProjection {
    fn from_study(study: &CtGovStudy, sections: &[String]) -> Result<Self, &'static str> {
        let protocol = study
            .protocol_section
            .as_ref()
            .ok_or("invalid_projection")?;
        let arms_requested = requested(sections, "arms");
        let eligibility_requested = sections
            .iter()
            .all(|value| matches!(value.trim(), "" | "--json" | "-j"))
            || requested(sections, "eligibility");
        let references_requested = requested(sections, "references");

        let module = protocol.arms_interventions_module.as_ref();
        let intervention_rows = module.and_then(|value| value.interventions.as_ref());
        let mut interventions = Vec::new();
        if let Some(rows) = intervention_rows {
            for (index, row) in rows.iter().enumerate() {
                interventions.push(
                    ClinicalTrialIntervention::new(
                        ClinicalTrialInterventionId::new((index + 1) as u64)
                            .map_err(|_| "invalid_projection")?,
                        clean(row.name.as_deref()).ok_or("invalid_projection")?,
                        code("clinicaltrials.gov", row.intervention_type.as_deref())?,
                        clean(row.description.as_deref()),
                        Some(
                            row.other_names
                                .iter()
                                .filter_map(|value| clean(Some(value)))
                                .collect(),
                        ),
                    )
                    .map_err(|_| "invalid_projection")?,
                );
            }
        }
        let interventions_section = intervention_rows.map_or(ClinicalTrialSection::Absent, |_| {
            ClinicalTrialSection::Present(interventions.clone())
        });

        let arms = if !arms_requested {
            ClinicalTrialSection::NotRequested
        } else if let Some(rows) = module.and_then(|value| value.arm_groups.as_ref()) {
            let mut values = Vec::new();
            let mut labels = std::collections::HashMap::new();
            for (index, row) in rows.iter().enumerate() {
                let label = clean(row.label.as_deref()).ok_or("invalid_projection")?;
                if labels.insert(label.clone(), index).is_some() {
                    return Err("invalid_projection");
                }
                values.push(
                    ClinicalTrialArm::new(
                        ClinicalTrialArmId::new((index + 1) as u64)
                            .map_err(|_| "invalid_projection")?,
                        label,
                        code("clinicaltrials.gov", row.arm_group_type.as_deref())?,
                        clean(row.description.as_deref()),
                    )
                    .map_err(|_| "invalid_projection")?,
                );
            }
            let mut assignments = Vec::new();
            for (intervention_index, row) in module
                .and_then(|value| value.interventions.as_ref())
                .into_iter()
                .flatten()
                .enumerate()
            {
                for label in &row.arm_group_labels {
                    let arm_index = labels.get(label.trim()).ok_or("invalid_projection")?;
                    assignments.push(ClinicalTrialArmInterventionAssignment::new(
                        ClinicalTrialArmId::new((*arm_index + 1) as u64)
                            .map_err(|_| "invalid_projection")?,
                        ClinicalTrialInterventionId::new((intervention_index + 1) as u64)
                            .map_err(|_| "invalid_projection")?,
                    ));
                }
            }
            ClinicalTrialSection::Present(
                ClinicalTrialArms::new(values, &interventions, assignments)
                    .map_err(|_| "invalid_projection")?,
            )
        } else {
            ClinicalTrialSection::Absent
        };

        let eligibility = if !eligibility_requested {
            ClinicalTrialSection::NotRequested
        } else if let Some(value) = protocol.eligibility_module.as_ref() {
            let minimum = age(value.minimum_age.as_ref(), Bound::Minimum)?;
            let maximum = age(value.maximum_age.as_ref(), Bound::Maximum)?;
            let age_range = (minimum.is_some() || maximum.is_some())
                .then(|| ClinicalTrialAgeRange::new(minimum, maximum))
                .transpose()
                .map_err(|_| "invalid_projection")?;
            let sexes = code("clinicaltrials.gov", value.sex.as_deref())?.map(|value| vec![value]);
            ClinicalTrialSection::Present(
                ClinicalTrialEligibility::new(
                    clean(value.eligibility_criteria.as_deref()),
                    age_range,
                    sexes,
                    value.healthy_volunteers,
                    None,
                )
                .map_err(|_| "invalid_projection")?,
            )
        } else {
            ClinicalTrialSection::Absent
        };

        let references = if !references_requested {
            ClinicalTrialSection::NotRequested
        } else if let Some(value) = protocol.references_module.as_ref() {
            let rows = value
                .references
                .iter()
                .map(|row| {
                    let source_type = row
                        .reference_type
                        .as_ref()
                        .map(|value| {
                            ExtensibleCode::new(
                                "clinicaltrials.gov",
                                value,
                                None::<String>,
                                None::<String>,
                                None::<String>,
                            )
                        })
                        .transpose()
                        .map_err(|_| "invalid_projection")?;
                    ClinicalTrialReference::new(row.pmid.clone(), row.citation.clone(), source_type)
                        .map_err(|_| "invalid_projection")
                })
                .collect::<Result<Vec<_>, _>>()?;
            ClinicalTrialSection::Present(rows)
        } else {
            ClinicalTrialSection::Absent
        };
        Ok(Self {
            interventions: interventions_section,
            arms,
            eligibility,
            references,
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CtGovEligibilityModule {
    pub eligibility_criteria: Option<String>,
    pub minimum_age: Option<NormalizedTimeWire>,
    pub maximum_age: Option<NormalizedTimeWire>,
    pub sex: Option<String>,
    pub healthy_volunteers: Option<bool>,
}

impl Serialize for CtGovEligibilityModule {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(3))?;
        map.serialize_entry("eligibilityCriteria", &self.eligibility_criteria)?;
        map.serialize_entry("minimumAge", &self.minimum_age)?;
        map.serialize_entry("maximumAge", &self.maximum_age)?;
        map.end()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CtGovReferencesModule {
    #[serde(default)]
    pub references: Vec<CtGovReference>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CtGovReference {
    pub pmid: Option<String>,
    pub citation: Option<String>,
    #[serde(rename = "type")]
    pub reference_type: Option<String>,
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

#[cfg(test)]
mod tests;
