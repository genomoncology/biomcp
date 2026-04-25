//! Drug entity models and workflows exposed through the stable drug facade.

mod get;
mod interactions;
mod label;
mod metadata;
mod query;
mod search;
mod targets;
#[cfg(test)]
mod test_support;

pub use self::get::{get, get_with_region};
pub(crate) use self::get::{resolve_trial_aliases, resolve_trial_canonical_name};
pub(crate) use self::interactions::{
    DrugInteractionReport, apply_interaction_report, interaction_class_summaries,
    interaction_report, interaction_report_from_base,
};
pub use self::query::search_query_summary;
#[allow(unused_imports)]
pub use self::search::{
    search, search_name_query_with_region, search_page, search_page_with_region,
};

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::entities::SearchPage;
use crate::error::BioMcpError;
use crate::sources::civic::CivicContext;
use crate::sources::ema::EmaDrugIdentity;
use crate::sources::mychem::{MYCHEM_FIELDS_GET, MyChemClient, MyChemQueryResponse};
use crate::sources::who_pq::WhoPqIdentity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Drug {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drugbank_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chembl_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unii: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drug_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mechanism: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mechanisms: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_date_raw: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_date_display: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub brand_names: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub targets: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variant_targets: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_family_name: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub indications: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub interactions: Vec<DrugInteraction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interaction_text: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pharm_classes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub top_adverse_events: Vec<String>,

    #[serde(skip)]
    pub faers_query: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<DrugLabel>,

    #[serde(skip)]
    pub label_set_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub shortage: Option<Vec<DrugShortageEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approvals: Option<Vec<DrugApproval>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub us_safety_warnings: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ema_regulatory: Option<Vec<EmaRegulatoryRow>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ema_safety: Option<EmaSafetyInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ema_shortage: Option<Vec<EmaShortageEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub who_prequalification: Option<Vec<WhoPrequalificationEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub civic: Option<CivicContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugInteraction {
    pub drug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub partner_classes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugLabelIndication {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pivotal_trial: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugLabel {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub indication_summary: Vec<DrugLabelIndication>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indications: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dosage: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugShortageEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generic_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_info: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_posting_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugApproval {
    pub application_number: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sponsor_name: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub openfda_brand_names: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub openfda_generic_names: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub products: Vec<DrugApprovalProduct>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub submissions: Vec<DrugApprovalSubmission>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugApprovalProduct {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dosage_form: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marketing_status: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub active_ingredients: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugApprovalSubmission {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submission_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submission_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugSearchResult {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drugbank_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drug_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mechanism: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub(crate) enum WhoPrequalificationKind {
    #[default]
    FinishedPharma,
    Api,
    Vaccine,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhoPrequalificationEntry {
    #[serde(skip)]
    pub(crate) kind: WhoPrequalificationKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub who_reference_number: Option<String>,
    pub inn: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presentation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dosage_form: Option<String>,
    pub product_type: String,
    pub therapeutic_area: String,
    pub applicant: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listing_basis: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alternative_listing_basis: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prequalification_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub who_product_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grade: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirmation_document_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vaccine_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commercial_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dose_count: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manufacturer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub responsible_nra: Option<String>,
}

impl WhoPrequalificationEntry {
    pub(crate) fn is_vaccine(&self) -> bool {
        matches!(self.kind, WhoPrequalificationKind::Vaccine)
    }

    pub(crate) fn display_identifier(&self) -> &str {
        self.who_reference_number
            .as_deref()
            .or(self.who_product_id.as_deref())
            .unwrap_or("-")
    }

    pub(crate) fn stable_identifier_key(&self) -> String {
        if let Some(value) = self.who_reference_number.as_deref() {
            format!("ref:{value}")
        } else if let Some(value) = self.who_product_id.as_deref() {
            format!("product:{value}")
        } else if self.is_vaccine() {
            format!(
                "vaccine:{}|{}|{}|{}|{}|{}|{}",
                stable_who_identity_component(self.vaccine_type.as_deref()),
                stable_who_identity_component(self.commercial_name.as_deref()),
                stable_who_identity_component(self.manufacturer.as_deref()),
                stable_who_identity_component(self.presentation.as_deref()),
                stable_who_identity_component(self.dose_count.as_deref()),
                stable_who_identity_component(self.responsible_nra.as_deref()),
                stable_who_identity_component(self.prequalification_date.as_deref()),
            )
        } else {
            format!("missing:{}", self.inn.to_ascii_lowercase())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhoPrequalificationSearchResult {
    #[serde(skip)]
    pub(crate) kind: WhoPrequalificationKind,
    pub inn: String,
    pub product_type: String,
    pub therapeutic_area: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presentation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dosage_form: Option<String>,
    pub applicant: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub who_reference_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub who_product_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listing_basis: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prequalification_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vaccine_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commercial_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dose_count: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manufacturer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub responsible_nra: Option<String>,
}

impl WhoPrequalificationSearchResult {
    pub(crate) fn is_vaccine(&self) -> bool {
        matches!(self.kind, WhoPrequalificationKind::Vaccine)
    }

    pub(crate) fn display_identifier(&self) -> &str {
        self.who_reference_number
            .as_deref()
            .or(self.who_product_id.as_deref())
            .unwrap_or("-")
    }
}

fn stable_who_identity_component(value: Option<&str>) -> String {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.split_whitespace().collect::<Vec<_>>().join(" "))
        .map(|value| value.to_ascii_lowercase())
        .unwrap_or_else(|| "-".to_string())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DrugRegion {
    #[default]
    Us,
    Eu,
    Who,
    All,
}

impl DrugRegion {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Us => "us",
            Self::Eu => "eu",
            Self::Who => "who",
            Self::All => "all",
        }
    }

    pub fn includes_us(self) -> bool {
        matches!(self, Self::Us | Self::All)
    }

    pub fn includes_eu(self) -> bool {
        matches!(self, Self::Eu | Self::All)
    }

    pub fn includes_who(self) -> bool {
        matches!(self, Self::Who | Self::All)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmaDrugSearchResult {
    pub name: String,
    pub active_substance: String,
    pub ema_product_number: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmaRegulatoryRow {
    pub medicine_name: String,
    pub active_substance: String,
    pub ema_product_number: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub holder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marketing_authorisation_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub therapeutic_indication: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recent_activity: Vec<EmaRegulatoryActivity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmaRegulatoryActivity {
    pub first_published_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmaSafetyInfo {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dhpcs: Vec<EmaDhpcEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub referrals: Vec<EmaReferralEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub psusas: Vec<EmaPsusaEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmaDhpcEntry {
    pub medicine_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dhpc_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regulatory_outcome: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_published_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmaReferralEntry {
    pub referral_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_substance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub associated_medicines: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_referral: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referral_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub procedure_start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prac_recommendation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmaPsusaEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_medicines: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_substance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub procedure_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regulatory_outcome: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_published_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmaShortageEntry {
    pub medicine_affected: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability_of_alternatives: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_published_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated_date: Option<String>,
}

#[derive(Debug, Clone)]
pub enum DrugSearchPageWithRegion {
    Us(SearchPage<DrugSearchResult>),
    Eu(SearchPage<EmaDrugSearchResult>),
    Who(SearchPage<WhoPrequalificationSearchResult>),
    All {
        us: SearchPage<DrugSearchResult>,
        eu: SearchPage<EmaDrugSearchResult>,
        who: SearchPage<WhoPrequalificationSearchResult>,
    },
}

#[derive(Debug, Clone, Default)]
pub struct DrugSearchFilters {
    pub query: Option<String>,
    pub target: Option<String>,
    pub indication: Option<String>,
    pub mechanism: Option<String>,
    pub drug_type: Option<String>,
    pub atc: Option<String>,
    pub pharm_class: Option<String>,
    pub interactions: Option<String>,
}

impl DrugSearchFilters {
    pub fn has_structured_filters(&self) -> bool {
        self.target.is_some()
            || self.indication.is_some()
            || self.mechanism.is_some()
            || self.drug_type.is_some()
            || self.atc.is_some()
            || self.pharm_class.is_some()
            || self.interactions.is_some()
    }
}

const DRUG_SECTION_LABEL: &str = "label";
const DRUG_SECTION_REGULATORY: &str = "regulatory";
const DRUG_SECTION_SAFETY: &str = "safety";
const DRUG_SECTION_SHORTAGE: &str = "shortage";
const DRUG_SECTION_TARGETS: &str = "targets";
const DRUG_SECTION_INDICATIONS: &str = "indications";
const DRUG_SECTION_INTERACTIONS: &str = "interactions";
const DRUG_SECTION_CIVIC: &str = "civic";
const DRUG_SECTION_APPROVALS: &str = "approvals";
const DRUG_SECTION_ALL: &str = "all";

pub const DRUG_SECTION_NAMES: &[&str] = &[
    DRUG_SECTION_LABEL,
    DRUG_SECTION_REGULATORY,
    DRUG_SECTION_SAFETY,
    DRUG_SECTION_SHORTAGE,
    DRUG_SECTION_TARGETS,
    DRUG_SECTION_INDICATIONS,
    DRUG_SECTION_INTERACTIONS,
    DRUG_SECTION_CIVIC,
    DRUG_SECTION_APPROVALS,
    DRUG_SECTION_ALL,
];

const OPTIONAL_SAFETY_TIMEOUT: Duration = Duration::from_secs(8);

fn build_ema_identity(requested_name: &str, drug: &Drug) -> EmaDrugIdentity {
    EmaDrugIdentity::with_aliases(requested_name, Some(&drug.name), &drug.brand_names)
}

fn build_who_identity(requested_name: &str, drug: &Drug) -> WhoPqIdentity {
    WhoPqIdentity::with_aliases(requested_name, Some(&drug.name), &drug.brand_names)
}

async fn direct_drug_lookup(query: &str) -> Result<MyChemQueryResponse, BioMcpError> {
    MyChemClient::new()?
        .query_with_fields(query, 25, 0, MYCHEM_FIELDS_GET)
        .await
}
