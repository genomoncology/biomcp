//! One patient's record on the operator's FHIR server: demographics and the
//! `conditions` section.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::entities::section_outcome::{SectionOutcome, SectionOutcomes};
use crate::entities::source_state_registry::outcome_keys;
use crate::error::BioMcpError;
use crate::sources::fhir::{FHIR_SOURCE, FhirClient, PatientId, Walk, WalkStop};

pub(crate) const PATIENT_SECTION_CONDITIONS: &str = "conditions";
const PATIENT_SECTION_ALL: &str = "all";

pub const PATIENT_SECTION_NAMES: &[&str] = &[PATIENT_SECTION_CONDITIONS, PATIENT_SECTION_ALL];

const CONDITIONS_UNAVAILABLE: &str = "The FHIR server did not return the condition list.";
const CONDITIONS_REPEATED_LINK: &str =
    "The condition list stopped at a repeated page link and may be incomplete.";
const CONDITIONS_OFF_BASE: &str = "The condition list stopped at a link or redirect off the configured server and may be incomplete.";
const CONDITIONS_PAGE_CAP: &str =
    "The condition list stopped at the 20-page limit and may be incomplete.";
const CONDITIONS_LATER_PAGE: &str =
    "The condition list stopped when a later page failed and may be incomplete.";
const CONDITIONS_MISSING_STATUS: &str = "At least one condition has no clinical status.";
const CONDITIONS_OUTCOME_ERROR: &str = "The FHIR server reported an error with the condition list.";

fn default_patient_section_outcomes() -> SectionOutcomes {
    SectionOutcomes::with_keys(&outcome_keys("patient"))
}

fn deserialize_patient_section_outcomes<'de, D>(
    deserializer: D,
) -> Result<SectionOutcomes, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let outcomes = SectionOutcomes::deserialize(deserializer)?;
    outcomes
        .validate_keys(&outcome_keys("patient"))
        .map_err(serde::de::Error::custom)?;
    Ok(outcomes)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patient {
    #[serde(
        default = "default_patient_section_outcomes",
        deserialize_with = "deserialize_patient_section_outcomes"
    )]
    pub section_outcomes: SectionOutcomes,
    pub source: String,
    pub id: String,
    pub gender: Option<String>,
    pub birth_date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conditions: Option<Vec<PatientCondition>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatientCondition {
    pub id: Option<String>,
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub codings: Vec<ConditionCoding>,
    pub clinical_status: Option<String>,
    pub verification_status: Option<String>,
    pub onset: Option<String>,
    pub recorded_date: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConditionCoding {
    pub system: Option<String>,
    pub code: Option<String>,
    pub display: Option<String>,
}

/// Reads one patient from the server in `BIOMCP_FHIR_BASE`.
///
/// The ID and the sections are checked before any request.
pub async fn get(id: &str, sections: &[String]) -> Result<Patient, BioMcpError> {
    let id = PatientId::parse(id.trim())?;
    let include_conditions = include_conditions(sections)?;
    let client = FhirClient::from_env()?;
    get_with_client(&client, &id, include_conditions).await
}

pub(crate) async fn get_with_client(
    client: &FhirClient,
    id: &PatientId,
    include_conditions: bool,
) -> Result<Patient, BioMcpError> {
    let resource = client.read_patient(id).await.map_err(BioMcpError::Fhir)?;
    let mut patient = Patient {
        section_outcomes: default_patient_section_outcomes(),
        source: FHIR_SOURCE.to_string(),
        id: id.as_str().to_string(),
        gender: text_at(&resource, "gender"),
        birth_date: text_at(&resource, "birthDate"),
        conditions: None,
    };
    if include_conditions {
        let (conditions, outcome) = match client.search_conditions(id).await {
            Ok(walk) => conditions_from_walk(&walk),
            Err(_) => (
                Vec::new(),
                SectionOutcome::unavailable(CONDITIONS_UNAVAILABLE),
            ),
        };
        patient.conditions = Some(conditions);
        patient
            .section_outcomes
            .complete(PATIENT_SECTION_CONDITIONS, outcome);
    }
    Ok(patient)
}

fn include_conditions(sections: &[String]) -> Result<bool, BioMcpError> {
    let mut include = false;
    for section in sections {
        let section = section.trim().to_ascii_lowercase();
        match section.as_str() {
            "" => {}
            PATIENT_SECTION_CONDITIONS | PATIENT_SECTION_ALL => include = true,
            _ => {
                return Err(BioMcpError::InvalidArgument(format!(
                    "unknown patient section: {section}. Use conditions or all."
                )));
            }
        }
    }
    Ok(include)
}

/// Settles the conditions section from one walk.
pub(crate) fn conditions_from_walk(walk: &Walk) -> (Vec<PatientCondition>, SectionOutcome) {
    let conditions = walk
        .matches
        .iter()
        .filter(|resource| {
            resource.get("resourceType").and_then(Value::as_str) == Some("Condition")
        })
        .map(condition_from_resource)
        .collect::<Vec<_>>();
    let degraded = match walk.stop {
        Some(WalkStop::RepeatedLink) => Some(CONDITIONS_REPEATED_LINK),
        Some(WalkStop::OffBaseLink | WalkStop::OffBaseRedirect) => Some(CONDITIONS_OFF_BASE),
        Some(WalkStop::PageCap) => Some(CONDITIONS_PAGE_CAP),
        Some(WalkStop::LaterPageFailed) => Some(CONDITIONS_LATER_PAGE),
        None if walk.outcome_error => Some(CONDITIONS_OUTCOME_ERROR),
        None if conditions.iter().any(|row| row.clinical_status.is_none()) => {
            Some(CONDITIONS_MISSING_STATUS)
        }
        None => None,
    };
    let outcome = match degraded {
        Some(message) => SectionOutcome::degraded([FHIR_SOURCE], message),
        None if conditions.is_empty() => SectionOutcome::empty(FHIR_SOURCE),
        None => SectionOutcome::data(FHIR_SOURCE),
    };
    (conditions, outcome)
}

fn condition_from_resource(resource: &Value) -> PatientCondition {
    let code = resource.get("code");
    let codings = code
        .and_then(|code| code.get("coding"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|coding| ConditionCoding {
            system: text_at(coding, "system"),
            code: text_at(coding, "code"),
            display: text_at(coding, "display"),
        })
        .collect::<Vec<_>>();
    let text = code
        .and_then(|code| text_at(code, "text"))
        .or_else(|| codings.iter().find_map(|coding| coding.display.clone()));
    let onset = text_at(resource, "onsetDateTime")
        .or_else(|| {
            resource
                .get("onsetPeriod")
                .and_then(|period| text_at(period, "start"))
        })
        .or_else(|| text_at(resource, "onsetString"));
    PatientCondition {
        id: text_at(resource, "id"),
        text,
        codings,
        clinical_status: first_code(resource.get("clinicalStatus")),
        verification_status: first_code(resource.get("verificationStatus")),
        onset,
        recorded_date: text_at(resource, "recordedDate"),
    }
}

fn first_code(concept: Option<&Value>) -> Option<String> {
    concept?
        .get("coding")?
        .as_array()?
        .iter()
        .find_map(|coding| text_at(coding, "code"))
}

fn text_at(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(ToOwned::to_owned)
}

mod search;
pub use self::search::{
    PATIENT_SEARCH_MAX_LIMIT, PatientSearchFilters, PatientSearchRow, count, search,
};

#[cfg(test)]
mod tests;
