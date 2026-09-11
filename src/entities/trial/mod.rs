//! Trial entity models and workflows exposed through the stable trial facade.

use biodata::{
    ClinicalTrialContact, ClinicalTrialEligibility, ClinicalTrialPlannedOutcome,
    ClinicalTrialReference, ClinicalTrialSite, ClinicalTrialSiteDirectory,
};
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

use crate::error::BioMcpError;

mod design;
mod documents;
mod eligibility;
mod get;
mod search;
#[cfg(test)]
mod test_support;

pub use self::design::{TrialDesign, TrialDesignError};
pub use self::documents::{
    TrialDocumentsManifest, TrialEligibilityProvenance, trial_document_bytes,
    trial_documents_manifest,
};
pub use self::get::get;
pub use self::search::{count_all, search, search_page};

pub(crate) fn validate_search_filters(filters: &TrialSearchFilters) -> Result<(), BioMcpError> {
    search::validate_trial_search(filters).map(|_| ())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trial {
    #[serde(default)]
    pub identities: Vec<TrialIdentity>,
    pub nct_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub official_title: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub why_stopped: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
    #[serde(default)]
    pub phases: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub study_type: Option<String>,
    #[serde(default)]
    pub conditions: Vec<String>,
    #[serde(flatten)]
    pub design: TrialDesign,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enrollment: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_date: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "eligibility_wire"
    )]
    pub eligibility: Option<ClinicalTrialEligibility>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eligibility_provenance: Option<TrialEligibilityProvenance>,
    #[serde(skip)]
    pub(crate) site_directory: Option<ClinicalTrialSiteDirectory>,
    #[serde(skip)]
    pub(crate) site_offset: usize,
    #[serde(skip)]
    pub(crate) site_limit: Option<usize>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "outcome_wire::serialize",
        deserialize_with = "outcome_wire::deserialize"
    )]
    pub outcomes: Option<Vec<ClinicalTrialPlannedOutcome>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "reference_wire"
    )]
    pub references: Option<Vec<ClinicalTrialReference>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct TrialIdentity {
    pub authority: String,
    pub identifier: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrialSectionState {
    NotRequested,
    Unavailable,
    Absent,
    Present,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct TrialSectionStates {
    pub arms: TrialSectionState,
    pub eligibility: TrialSectionState,
    pub outcomes: TrialSectionState,
    pub references: TrialSectionState,
    pub contacts: TrialSectionState,
    pub locations: TrialSectionState,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TrialResponse {
    #[serde(flatten)]
    pub trial: Trial,
    pub section_states: TrialSectionStates,
}

impl Serialize for TrialResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct Wire<'a> {
            #[serde(flatten)]
            trial: &'a Trial,
            #[serde(skip_serializing_if = "Option::is_none")]
            contacts: Option<Vec<TrialContactView<'a>>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            locations: Option<Vec<TrialLocationView<'a>>>,
            section_states: &'a TrialSectionStates,
        }
        Wire {
            trial: &self.trial,
            contacts: (self.section_states.contacts == TrialSectionState::Present)
                .then(|| self.trial.contact_views()),
            locations: (self.section_states.locations == TrialSectionState::Present)
                .then(|| self.trial.location_views()),
            section_states: &self.section_states,
        }
        .serialize(serializer)
    }
}

impl Deref for TrialResponse {
    type Target = Trial;

    fn deref(&self) -> &Self::Target {
        &self.trial
    }
}

impl DerefMut for TrialResponse {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.trial
    }
}

impl Trial {
    pub(crate) fn set_site_directory(&mut self, directory: Option<ClinicalTrialSiteDirectory>) {
        self.site_directory = directory;
    }

    pub(crate) fn location_count(&self) -> usize {
        self.site_directory
            .as_ref()
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
            .site_directory
            .as_ref()
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
        if let Some(directory) = self.site_directory.as_ref() {
            views.extend(
                directory
                    .central_contacts()
                    .unwrap_or_default()
                    .iter()
                    .map(TrialContactView::central),
            );
        }
        for site in self.paged_sites() {
            views.extend(
                site.contacts()
                    .unwrap_or_default()
                    .iter()
                    .map(|contact| TrialContactView::site(contact, site)),
            );
        }
        views
    }

    pub(crate) fn location_views(&self) -> Vec<TrialLocationView<'_>> {
        self.paged_sites()
            .iter()
            .map(TrialLocationView::new)
            .collect()
    }

    pub(crate) fn contact_render_values(&self) -> serde_json::Value {
        serde_json::to_value(self.contact_views()).expect("borrowed contact views serialize")
    }

    pub(crate) fn location_render_values(&self) -> serde_json::Value {
        serde_json::to_value(self.location_views()).expect("borrowed location views serialize")
    }

    pub(crate) fn has_arms(&self) -> bool {
        self.design.arms().is_some()
    }

    pub(crate) fn has_eligibility_age(&self) -> bool {
        self.eligibility
            .as_ref()
            .is_some_and(|value| value.age_range().is_some())
    }
}

#[derive(Serialize)]
pub(crate) struct TrialSiteContactView<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    phone: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    phone_extension: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<&'a str>,
}

impl<'a> From<&'a ClinicalTrialContact> for TrialSiteContactView<'a> {
    fn from(contact: &'a ClinicalTrialContact) -> Self {
        Self {
            name: contact.name(),
            role: contact.role().map(|value| value.code()),
            phone: contact.phone(),
            phone_extension: contact.phone_extension(),
            email: contact.email(),
        }
    }
}

#[derive(Serialize)]
pub(crate) struct TrialLocationView<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    facility: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    city: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    state: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    postal_code: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    country: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    latitude: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    longitude: Option<f64>,
    contacts: Vec<TrialSiteContactView<'a>>,
}

impl<'a> TrialLocationView<'a> {
    fn new(site: &'a ClinicalTrialSite) -> Self {
        let coordinates = site.coordinates();
        Self {
            facility: site.facility(),
            city: site.city(),
            state: site.state(),
            postal_code: site.postal_code(),
            country: site.country(),
            status: site.status().map(|value| value.code()),
            latitude: coordinates.map(|value| value.latitude()),
            longitude: coordinates.map(|value| value.longitude()),
            contacts: site
                .contacts()
                .unwrap_or_default()
                .iter()
                .map(Into::into)
                .collect(),
        }
    }
}

#[derive(Serialize)]
pub(crate) struct TrialContactView<'a> {
    level: &'static str,
    #[serde(flatten)]
    contact: TrialSiteContactView<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    facility: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    city: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    state: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    country: Option<&'a str>,
}

impl<'a> TrialContactView<'a> {
    fn central(contact: &'a ClinicalTrialContact) -> Self {
        Self {
            level: "central",
            contact: contact.into(),
            facility: None,
            city: None,
            state: None,
            country: None,
        }
    }

    fn site(contact: &'a ClinicalTrialContact, site: &'a ClinicalTrialSite) -> Self {
        Self {
            level: "site",
            contact: contact.into(),
            facility: site.facility(),
            city: site.city(),
            state: site.state(),
            country: site.country(),
        }
    }
}

use eligibility as eligibility_wire;

pub(crate) mod outcome_wire {
    use biodata::{ClinicalTrialPlannedOutcome, ExtensibleCode};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    const AUTHORITY: &str = "clinicaltrials.gov";
    const INVALID_OUTCOME: &str = "invalid clinical trial planned outcome";

    #[derive(Serialize)]
    pub(crate) struct RowView<'a> {
        measure: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        time_frame: Option<&'a str>,
    }

    impl<'a> From<&'a ClinicalTrialPlannedOutcome> for RowView<'a> {
        fn from(value: &'a ClinicalTrialPlannedOutcome) -> Self {
            Self {
                measure: value.measure(),
                description: value.description(),
                time_frame: value.time_frame(),
            }
        }
    }

    #[derive(Serialize)]
    pub(crate) struct GroupedView<'a> {
        primary: Vec<RowView<'a>>,
        secondary: Vec<RowView<'a>>,
        other: Vec<RowView<'a>>,
    }

    pub(crate) fn views(
        outcomes: &Option<Vec<ClinicalTrialPlannedOutcome>>,
    ) -> Result<Option<GroupedView<'_>>, &'static str> {
        outcomes
            .as_deref()
            .map(|values| {
                let mut grouped = GroupedView {
                    primary: Vec::new(),
                    secondary: Vec::new(),
                    other: Vec::new(),
                };
                for outcome in values {
                    let classification = outcome.source_classification();
                    if classification.authority() != AUTHORITY {
                        return Err(INVALID_OUTCOME);
                    }
                    let row = RowView::from(outcome);
                    match classification.code() {
                        "primaryOutcomes" => grouped.primary.push(row),
                        "secondaryOutcomes" => grouped.secondary.push(row),
                        "otherOutcomes" => grouped.other.push(row),
                        _ => return Err(INVALID_OUTCOME),
                    }
                }
                Ok(grouped)
            })
            .transpose()
    }

    pub(super) fn serialize<S: Serializer>(
        outcomes: &Option<Vec<ClinicalTrialPlannedOutcome>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        views(outcomes)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Grouped {
        #[serde(default)]
        primary: Vec<Row>,
        #[serde(default)]
        secondary: Vec<Row>,
        #[serde(default)]
        other: Vec<Row>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Row {
        measure: String,
        description: Option<String>,
        time_frame: Option<String>,
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<Vec<ClinicalTrialPlannedOutcome>>, D::Error> {
        Option::<Grouped>::deserialize(deserializer)?
            .map(|grouped| {
                [
                    ("primaryOutcomes", grouped.primary),
                    ("secondaryOutcomes", grouped.secondary),
                    ("otherOutcomes", grouped.other),
                ]
                .into_iter()
                .flat_map(|(classification, rows)| {
                    rows.into_iter().map(move |row| (classification, row))
                })
                .map(|(classification, row)| {
                    let classification = ExtensibleCode::new(
                        AUTHORITY,
                        classification,
                        None::<String>,
                        None::<String>,
                        None::<String>,
                    )
                    .map_err(|_| serde::de::Error::custom(INVALID_OUTCOME))?;
                    ClinicalTrialPlannedOutcome::new(
                        row.measure,
                        row.description,
                        row.time_frame,
                        classification,
                    )
                    .map_err(|_| serde::de::Error::custom(INVALID_OUTCOME))
                })
                .collect()
            })
            .transpose()
    }
}

pub(crate) mod reference_wire {
    use biodata::ClinicalTrialReference;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use serde_json::value::RawValue;

    const INVALID_REFERENCE: &str = "invalid clinical trial reference";

    #[derive(Serialize)]
    pub(crate) struct View<'a> {
        pub(crate) pmid: Option<&'a str>,
        pub(crate) citation: Option<&'a str>,
        pub(crate) source_type_label: Option<&'a str>,
    }

    fn display_text(value: Option<&str>) -> Option<&str> {
        value.map(str::trim).filter(|value| !value.is_empty())
    }

    fn view(reference: &ClinicalTrialReference) -> View<'_> {
        let source_type_label = reference.source_type().and_then(|source_type| {
            display_text(source_type.display())
                .or_else(|| display_text(source_type.recognized_meaning()))
                .or_else(|| display_text(Some(source_type.code())))
        });
        View {
            pmid: display_text(reference.pmid()),
            citation: display_text(reference.citation()),
            source_type_label,
        }
    }

    pub(crate) fn views(references: &Option<Vec<ClinicalTrialReference>>) -> Option<Vec<View<'_>>> {
        references
            .as_ref()
            .map(|values| values.iter().map(view).collect())
    }

    pub(super) fn serialize<S: Serializer>(
        references: &Option<Vec<ClinicalTrialReference>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let Some(references) = references else {
            return serializer.serialize_none();
        };
        let encoded = references
            .iter()
            .map(|reference| {
                let json = reference
                    .to_json()
                    .map_err(|_| serde::ser::Error::custom(INVALID_REFERENCE))?;
                RawValue::from_string(json)
                    .map_err(|_| serde::ser::Error::custom(INVALID_REFERENCE))
            })
            .collect::<Result<Vec<_>, S::Error>>()?;
        encoded.serialize(serializer)
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<Vec<ClinicalTrialReference>>, D::Error> {
        Option::<Vec<Box<RawValue>>>::deserialize(deserializer)?
            .map(|values| {
                values
                    .into_iter()
                    .map(|value| {
                        ClinicalTrialReference::from_json_bytes(value.get().as_bytes())
                            .map_err(|_| serde::de::Error::custom(INVALID_REFERENCE))
                    })
                    .collect()
            })
            .transpose()
    }
}

#[derive(Clone, Serialize, Deserialize)]
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

impl std::fmt::Debug for TrialSearchResult {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TrialSearchResult")
            .finish_non_exhaustive()
    }
}

impl TrialSearchResult {
    pub(crate) fn from_biodata(
        value: &biodata::ClinicalTrialSearchSummary,
    ) -> Result<Self, BioMcpError> {
        let nct_id = value
            .identities()
            .first()
            .map(|identity| identity.identifier().to_owned())
            .ok_or(BioMcpError::InternalProcessing)?;
        let phase = (!value.phases().is_empty()).then(|| {
            value
                .phases()
                .iter()
                .map(|phase| phase.code())
                .collect::<Vec<_>>()
                .join("/")
        });
        Ok(Self {
            nct_id,
            title: value.brief_title().to_owned(),
            status: value.overall_status().code().to_owned(),
            phase,
            conditions: value.conditions().to_vec(),
            sponsor: value.lead_sponsor_name().map(str::to_owned),
            matched_intervention_label: None,
        })
    }
}

#[cfg(test)]
mod search_result_tests {
    use super::TrialSearchResult;

    #[test]
    fn product_search_debug_redacts_untrusted_values() {
        const SENTINEL: &str = "TRIAL-SEARCH-PRIVATE-SENTINEL-0117";
        let result = TrialSearchResult {
            nct_id: "NCT00000001".into(),
            title: SENTINEL.into(),
            status: SENTINEL.into(),
            phase: Some(SENTINEL.into()),
            conditions: vec![SENTINEL.into()],
            sponsor: Some(SENTINEL.into()),
            matched_intervention_label: Some(SENTINEL.into()),
        };
        assert!(!format!("{result:?}").contains(SENTINEL));
        assert!(serde_json::to_string(&result).unwrap().contains(SENTINEL));
    }

    #[test]
    fn shared_summary_maps_to_the_stable_product_keys_without_trimming() {
        let page = biodata::ClinicalTrialsGovApiV2SearchPage::parse(
            br#"{"studies":[{"protocolSection":{"identificationModule":{"nctId":"NCT00000001","briefTitle":" title "},"statusModule":{"overallStatus":" status "},"designModule":{"phases":["PHASE1","PHASE2"]},"conditionsModule":{"conditions":[" A "," A "]},"sponsorCollaboratorsModule":{"leadSponsor":{"name":" sponsor "}}}}],"totalCount":1}"#,
            &Default::default(),
        )
        .unwrap();
        let result =
            TrialSearchResult::from_biodata(page.results().unwrap()[0].projection().value())
                .unwrap();
        assert_eq!(
            serde_json::to_value(result).unwrap(),
            serde_json::json!({
                "nct_id": "NCT00000001",
                "title": " title ",
                "status": " status ",
                "phase": "PHASE1/PHASE2",
                "conditions": [" A ", " A "],
                "sponsor": " sponsor "
            })
        );
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

/// Describes the precision of a trial `--count-only` result.
#[derive(Debug, PartialEq)]
pub enum TrialCount {
    /// Exact post-filtered count.
    Exact(usize),
    /// Upstream CTGov total before client-side age post-filtering.
    Approximate(usize),
    /// The total is unknown for the stated reason.
    Unknown(TrialCountUnknownReason),
}

/// Explains why a trial count could not be stated numerically.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrialCountUnknownReason {
    /// ClinicalTrials.gov omitted the requested total from its response.
    ProviderOmittedTotal,
    /// Bounded post-filter traversal reached its page limit.
    TraversalLimitReached,
    /// An expanded ClinicalTrials.gov worker failed, leaving coverage incomplete.
    IncompleteCoverage,
}
