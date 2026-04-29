//! Trial entity models and workflows exposed through the stable trial facade.

use serde::{Deserialize, Serialize};

use crate::error::BioMcpError;

mod get;
mod search;
#[cfg(test)]
mod test_support;

pub use self::get::get;
pub use self::search::{count_all, search, search_page};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trial {
    pub nct_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub title: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub study_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age_range: Option<String>,
    #[serde(default)]
    pub conditions: Vec<String>,
    #[serde(default)]
    pub interventions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub intervention_details: Vec<TrialIntervention>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enrollment: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eligibility_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locations: Option<Vec<TrialLocation>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcomes: Option<TrialOutcomes>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arms: Option<Vec<TrialArm>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub references: Option<Vec<TrialReference>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialIntervention {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intervention_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub other_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialLocation {
    pub facility: String,
    pub city: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    pub country: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_phone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialOutcomes {
    #[serde(default)]
    pub primary: Vec<TrialOutcome>,
    #[serde(default)]
    pub secondary: Vec<TrialOutcome>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialOutcome {
    pub measure: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_frame: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialArm {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arm_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub interventions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialReference {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pmid: Option<String>,
    pub citation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialSearchResult {
    pub nct_id: String,
    pub title: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
    #[serde(default)]
    pub conditions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_intervention_label: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct TrialSearchFilters {
    pub condition: Option<String>,
    pub intervention: Option<String>,
    pub no_alias_expand: bool,
    pub facility: Option<String>,
    pub status: Option<String>,
    pub phase: Option<String>,
    pub study_type: Option<String>,
    pub age: Option<f32>,
    pub sex: Option<String>,
    pub sponsor: Option<String>,
    pub sponsor_type: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub mutation: Option<String>,
    pub criteria: Option<String>,
    pub biomarker: Option<String>,
    pub prior_therapies: Option<String>,
    pub progression_on: Option<String>,
    pub line_of_therapy: Option<String>,
    pub results_available: bool,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub distance: Option<u32>,
    pub source: TrialSource,
}

#[derive(Debug, Clone, Default, Copy)]
pub enum TrialSource {
    #[default]
    ClinicalTrialsGov,
    NciCts,
}

impl TrialSource {
    pub fn from_flag(value: &str) -> Result<Self, BioMcpError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "" | "ctgov" | "clinicaltrials" | "clinicaltrials.gov" => Ok(Self::ClinicalTrialsGov),
            "nci" | "nci_cts" | "cts" => Ok(Self::NciCts),
            other => Err(BioMcpError::InvalidArgument(format!(
                "Unknown --source '{other}'. Expected 'ctgov' or 'nci'."
            ))),
        }
    }
}

const TRIAL_SECTION_ELIGIBILITY: &str = "eligibility";
const TRIAL_SECTION_LOCATIONS: &str = "locations";
const TRIAL_SECTION_OUTCOMES: &str = "outcomes";
const TRIAL_SECTION_ARMS: &str = "arms";
const TRIAL_SECTION_REFERENCES: &str = "references";
const TRIAL_SECTION_ALL: &str = "all";

pub const TRIAL_SECTION_NAMES: &[&str] = &[
    TRIAL_SECTION_ELIGIBILITY,
    TRIAL_SECTION_LOCATIONS,
    TRIAL_SECTION_OUTCOMES,
    TRIAL_SECTION_ARMS,
    TRIAL_SECTION_REFERENCES,
    TRIAL_SECTION_ALL,
];

/// Describes the precision of a trial `--count-only` result.
#[derive(Debug, PartialEq)]
pub enum TrialCount {
    /// Exact post-filtered count.
    Exact(usize),
    /// Upstream CTGov total before client-side age post-filtering.
    Approximate(usize),
    /// Traversal cap was hit, so the exact total is unknown.
    Unknown,
}
