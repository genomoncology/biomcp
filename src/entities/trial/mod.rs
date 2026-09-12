//! Trial entity models and workflows exposed through the stable trial facade.

pub use biodata::{ClinicalTrialSearchTotal, ClinicalTrialSearchUnknownReason};
use biodata::{ClinicalTrialSite, ClinicalTrialSiteDirectory};
use serde::Serialize;

use crate::error::BioMcpError;

mod documents;
mod get;
mod search;
mod search_hit;
#[cfg(test)]
mod test_support;

pub use self::documents::{
    TrialDocumentsManifest, TrialEligibilityProvenance, trial_document_bytes,
    trial_documents_manifest,
};
pub use self::get::get;
pub use self::search::{count_all, search, search_page};
pub use self::search_hit::TrialSearchHit;
use crate::render::trial_projection as wire;
pub(crate) use crate::render::trial_projection::{
    TrialContactView, TrialLocationView, outcome_wire, reference_wire,
};

pub(crate) fn validate_search_filters(filters: &TrialSearchFilters) -> Result<(), BioMcpError> {
    search::validate_trial_search(filters).map(|_| ())
}

pub(crate) fn section_state<T>(section: biodata::ClinicalTrialSection<T>) -> TrialSectionState {
    match section {
        biodata::ClinicalTrialSection::NotRequested => TrialSectionState::NotRequested,
        biodata::ClinicalTrialSection::Unavailable => TrialSectionState::Unavailable,
        biodata::ClinicalTrialSection::Absent => TrialSectionState::Absent,
        biodata::ClinicalTrialSection::Present(_) => TrialSectionState::Present,
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TrialSectionState {
    NotRequested,
    Unavailable,
    Absent,
    Present,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct TrialSectionStates {
    pub arms: TrialSectionState,
    pub eligibility: TrialSectionState,
    pub outcomes: TrialSectionState,
    pub references: TrialSectionState,
    pub contacts: TrialSectionState,
    pub locations: TrialSectionState,
}

#[derive(Clone)]
pub struct TrialResponse {
    projection: biodata::ClinicalTrialProjection<biodata::Capture>,
    source: String,
    eligibility_provenance_metadata: Option<TrialEligibilityProvenance>,
    site_offset: usize,
    site_limit: Option<usize>,
    pub section_states: TrialSectionStates,
}

impl std::fmt::Debug for TrialResponse {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TrialResponse")
            .finish_non_exhaustive()
    }
}

impl serde::Serialize for TrialResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.output_value()
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

impl TrialResponse {
    pub(crate) fn new(
        projection: biodata::ClinicalTrialProjection<biodata::Capture>,
        source: impl Into<String>,
        eligibility_provenance: Option<TrialEligibilityProvenance>,
        section_states: TrialSectionStates,
    ) -> Self {
        Self {
            projection,
            source: source.into(),
            eligibility_provenance_metadata: eligibility_provenance,
            site_offset: 0,
            site_limit: None,
            section_states,
        }
    }

    pub(crate) fn trial(&self) -> &biodata::ClinicalTrial {
        self.projection.trial()
    }

    pub(crate) fn source(&self) -> &str {
        &self.source
    }

    pub(crate) fn nct_id(&self) -> &str {
        self.trial()
            .identities()
            .iter()
            .find(|value| value.authority() == "clinicaltrials.gov")
            .map(|value| value.identifier())
            .unwrap_or_default()
    }

    pub(crate) fn eligibility_provenance(
        &self,
    ) -> Option<documents::TrialEligibilityProvenanceView<'_>> {
        self.eligibility_provenance_metadata
            .as_ref()
            .map(|value| value.view(self.projection.capture()))
    }

    pub(crate) fn location_count(&self) -> usize {
        self.trial()
            .site_directory()
            .and_then(ClinicalTrialSiteDirectory::sites)
            .map_or(0, <[_]>::len)
    }

    pub(crate) fn set_site_page(&mut self, offset: usize, limit: usize) {
        self.site_offset = offset;
        self.site_limit = Some(limit);
    }

    pub(crate) fn returned_location_count(&self) -> usize {
        self.paged_sites().len()
    }

    fn paged_sites(&self) -> &[ClinicalTrialSite] {
        let sites = self
            .trial()
            .site_directory()
            .and_then(ClinicalTrialSiteDirectory::sites)
            .unwrap_or_default();
        let start = self.site_offset.min(sites.len());
        let end = self.site_limit.map_or(sites.len(), |limit| {
            start.saturating_add(limit).min(sites.len())
        });
        &sites[start..end]
    }

    pub(crate) fn contact_views(&self) -> Vec<TrialContactView<'_>> {
        let mut views = Vec::new();
        if let Some(directory) = self.trial().site_directory() {
            views.extend(
                directory
                    .central_contacts()
                    .unwrap_or_default()
                    .iter()
                    .map(wire::TrialContactView::central),
            );
        }
        for site in self.paged_sites() {
            views.extend(
                site.contacts()
                    .unwrap_or_default()
                    .iter()
                    .map(|contact| wire::TrialContactView::site(contact, site)),
            );
        }
        views
    }

    pub(crate) fn location_views(&self) -> Vec<TrialLocationView<'_>> {
        self.paged_sites()
            .iter()
            .map(wire::TrialLocationView::new)
            .collect()
    }

    pub(crate) fn contact_render_values(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::to_value(self.contact_views())
    }

    pub(crate) fn location_render_values(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::to_value(self.location_views())
    }

    pub(crate) fn has_arms(&self) -> bool {
        self.trial().arms().is_some()
    }

    pub(crate) fn has_eligibility_age(&self) -> bool {
        self.trial()
            .eligibility()
            .is_some_and(|value| value.age_range().is_some())
    }

    #[cfg(test)]
    pub(crate) fn test_ctgov(
        nct_id: &str,
        title: &str,
        status: &str,
        condition: &str,
        intervention: Option<&str>,
    ) -> Self {
        let intervention_module = intervention.map_or_else(
            String::new,
            |name| {
                format!(
                    r#","armsInterventionsModule":{{"interventions":[{{"name":"{name}","type":"DRUG"}}]}}"#
                )
            },
        );
        let input = format!(
            r#"{{"protocolSection":{{"identificationModule":{{"nctId":"{nct_id}","briefTitle":"{title}"}},"statusModule":{{"overallStatus":"{status}"}},"sponsorCollaboratorsModule":{{"leadSponsor":{{"name":"Test sponsor"}}}},"conditionsModule":{{"conditions":["{condition}"]}},"designModule":{{"studyType":"INTERVENTIONAL"}}{intervention_module}}}}}"#
        );
        let plan =
            biodata::ClinicalTrialsGovApiV2DetailPlan::new(nct_id, false).expect("test plan");
        let response = biodata::ClinicalTrialsGovApiV2Response::parse(
            &plan,
            input.as_bytes(),
            &Default::default(),
        )
        .expect("test response");
        let projection = response.into_projection().expect("test projection");
        Self::new(
            projection,
            "ClinicalTrials.gov",
            None,
            TrialSectionStates {
                arms: TrialSectionState::NotRequested,
                eligibility: TrialSectionState::NotRequested,
                outcomes: TrialSectionState::NotRequested,
                references: TrialSectionState::NotRequested,
                contacts: TrialSectionState::NotRequested,
                locations: TrialSectionState::NotRequested,
            },
        )
    }

    #[cfg(test)]
    pub(crate) fn test_ctgov_json(nct_id: &str, input: &str) -> Self {
        let plan =
            biodata::ClinicalTrialsGovApiV2DetailPlan::new(nct_id, false).expect("test plan");
        let response = biodata::ClinicalTrialsGovApiV2Response::parse(
            &plan,
            input.as_bytes(),
            &Default::default(),
        )
        .expect("test response");
        Self::new(
            response.into_projection().expect("test projection"),
            "ClinicalTrials.gov",
            None,
            TrialSectionStates {
                arms: TrialSectionState::NotRequested,
                eligibility: TrialSectionState::NotRequested,
                outcomes: TrialSectionState::NotRequested,
                references: TrialSectionState::NotRequested,
                contacts: TrialSectionState::NotRequested,
                locations: TrialSectionState::NotRequested,
            },
        )
    }

    fn output_value(&self) -> Result<serde_json::Value, &'static str> {
        const INVALID: &str = "clinical trial projection could not be rendered";
        let shared = self.trial();
        let mut output = serde_json::Map::new();
        output.insert("identities".into(), wire::identities(shared.identities()));
        output.insert("nct_id".into(), self.nct_id().into());
        output.insert("source".into(), self.source.clone().into());
        output.insert("title".into(), shared.brief_title().into());
        output.insert("status".into(), shared.overall_status().code().into());
        if let Some(value) = shared.official_title() {
            output.insert("official_title".into(), value.into());
        }
        output.insert(
            "why_stopped".into(),
            shared.stop_reason().map(serde_json::Value::from).into(),
        );
        let phases: Vec<_> = shared.phases().iter().map(|value| value.code()).collect();
        if !phases.is_empty() {
            output.insert("phase".into(), phases.join("/").into());
        }
        output.insert("phases".into(), phases.into());
        output.insert("study_type".into(), shared.study_type().code().into());
        output.insert("conditions".into(), shared.conditions().into());
        output.insert(
            "interventions".into(),
            wire::interventions(shared.interventions()),
        );
        if self.section_states.arms == TrialSectionState::Present {
            output.insert("arms".into(), wire::arms(shared.arms()));
            output.insert(
                "arm_intervention_assignments".into(),
                wire::assignments(shared.arm_intervention_assignments()),
            );
        }
        output.insert("sponsor".into(), shared.lead_sponsor_name().into());
        if let Some(value) = shared.enrollment_count() {
            output.insert("enrollment".into(), value.into());
        }
        if let Some(value) = shared.brief_summary() {
            output.insert("summary".into(), value.into());
        }
        if let Some(value) = shared.start_date() {
            output.insert("start_date".into(), value.into());
        }
        if let Some(value) = shared.completion_date() {
            output.insert("completion_date".into(), value.into());
        }
        if let Some(value) = shared.eligibility() {
            output.insert("eligibility".into(), wire::eligibility(value));
        }
        if let Some(value) = self.eligibility_provenance() {
            output.insert(
                "eligibility_provenance".into(),
                serde_json::to_value(value).map_err(|_| INVALID)?,
            );
        }
        if let Some(outcomes) =
            outcome_wire::views(self.trial().planned_outcomes()).map_err(|_| INVALID)?
        {
            output.insert(
                "outcomes".into(),
                serde_json::to_value(outcomes).map_err(|_| INVALID)?,
            );
        }
        if let Some(references) = reference_wire::views(shared.references()) {
            output.insert(
                "references".into(),
                serde_json::to_value(references).map_err(|_| INVALID)?,
            );
        }
        if self.section_states.contacts == TrialSectionState::Present {
            output.insert(
                "contacts".into(),
                self.contact_render_values().map_err(|_| INVALID)?,
            );
        }
        if self.section_states.locations == TrialSectionState::Present {
            output.insert(
                "locations".into(),
                self.location_render_values().map_err(|_| INVALID)?,
            );
        }
        output.insert(
            "section_states".into(),
            serde_json::to_value(&self.section_states).map_err(|_| INVALID)?,
        );
        output.insert("capture".into(), wire::capture(self.projection.capture()));
        output.insert(
            "conversion_report".into(),
            wire::report(self.projection.report()),
        );
        Ok(output.into())
    }
}

#[derive(Debug, Clone, Default)]
pub struct TrialSearchFilters {
    pub condition: Option<String>,
    pub intervention: Option<String>,
    pub no_alias_expand: bool,
    pub no_count_total: bool,
    pub facility: Option<String>,
    pub status: Option<String>,
    pub phase: Option<String>,
    pub study_type: Option<String>,
    pub age: Option<f64>,
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
const TRIAL_SECTION_CONTACTS: &str = "contacts";
const TRIAL_SECTION_LOCATIONS: &str = "locations";
const TRIAL_SECTION_OUTCOMES: &str = "outcomes";
const TRIAL_SECTION_ARMS: &str = "arms";
const TRIAL_SECTION_REFERENCES: &str = "references";
const TRIAL_SECTION_ALL: &str = "all";

pub const TRIAL_SECTION_NAMES: &[&str] = &[
    TRIAL_SECTION_ELIGIBILITY,
    TRIAL_SECTION_CONTACTS,
    TRIAL_SECTION_LOCATIONS,
    TRIAL_SECTION_OUTCOMES,
    TRIAL_SECTION_ARMS,
    TRIAL_SECTION_REFERENCES,
    TRIAL_SECTION_ALL,
];
