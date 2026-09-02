//! Disease entity models and workflows exposed through the stable disease facade.

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use futures::future::join_all;
use serde::{Deserialize, Serialize};
use tracing::debug as warn;

use crate::entities::SearchPage;
use crate::entities::diagnostic::DiagnosticSearchResult;
use crate::entities::drug::{self, DrugSearchFilters};
use crate::entities::section_outcome::SectionOutcomes;
use crate::entities::source_state_registry::outcome_keys;
use crate::entities::trial::{self, TrialSearchFilters, TrialSource};
use crate::error::BioMcpError;
use crate::sources::civic::{CivicClient, CivicContext};
use crate::sources::disgenet::{DisgenetAssociationRecord, DisgenetClient};
use crate::sources::hpo::HpoClient;
use crate::sources::monarch::{
    MonarchClient, MonarchGeneAssociation, MonarchModelAssociation, MonarchPhenotypeMatch,
};
use crate::sources::mydisease::{MyDiseaseClient, MyDiseaseHit};
use crate::sources::nih_reporter::{NihReporterClient, NihReporterFundingSection};
use crate::sources::ols4::OlsClient;
use crate::sources::opentargets::OpenTargetsClient;
use crate::sources::reactome::ReactomeClient;
use crate::sources::seer::{
    ResolvedSeerSite, SeerClient, SeerSiteCatalog, SeerSurvivalPayload, resolve_site,
};
use crate::transform;

pub(crate) fn default_disease_section_outcomes() -> SectionOutcomes {
    SectionOutcomes::with_keys(&outcome_keys("disease"))
}

fn deserialize_disease_section_outcomes<'de, D>(
    deserializer: D,
) -> Result<SectionOutcomes, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let outcomes = SectionOutcomes::deserialize(deserializer)?;
    outcomes
        .validate_keys(&outcome_keys("disease"))
        .map_err(serde::de::Error::custom)?;
    Ok(outcomes)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Disease {
    pub id: String, // e.g., MONDO:0005105
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition: Option<String>,
    #[serde(default)]
    pub synonyms: Vec<String>,
    #[serde(default)]
    pub parents: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub associated_genes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gene_associations: Vec<DiseaseGeneAssociation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub top_genes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub top_gene_scores: Vec<DiseaseTargetScore>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub treatment_landscape: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recruiting_trial_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pathways: Vec<DiseasePathway>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub phenotypes: Vec<DiseasePhenotype>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clinical_features: Vec<DiseasePhenotype>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub key_features: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<DiseaseVariantAssociation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_variant: Option<DiseaseVariantAssociation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub models: Vec<DiseaseModelAssociation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub prevalence: Vec<DiseasePrevalenceEvidence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prevalence_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub survival: Option<DiseaseSurvival>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub survival_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub funding: Option<NihReporterFundingSection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub funding_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<Vec<DiagnosticSearchResult>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub civic: Option<CivicContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disgenet: Option<DiseaseDisgenet>,
    #[serde(
        default = "default_disease_section_outcomes",
        deserialize_with = "deserialize_disease_section_outcomes"
    )]
    pub section_outcomes: SectionOutcomes,
    #[serde(default)]
    pub xrefs: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiseasePathway {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiseasePhenotype {
    pub hpo_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_qualifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub onset_qualifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sex_qualifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage_qualifier: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub qualifiers: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiseaseGeneAssociation {
    pub gene: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationship: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opentargets_score: Option<DiseaseAssociationScoreSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiseaseAssociationScoreSummary {
    pub overall_score: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gwas_score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rare_variant_score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub somatic_mutation_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiseaseTargetScore {
    pub symbol: String,
    #[serde(flatten)]
    pub summary: DiseaseAssociationScoreSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiseaseVariantAssociation {
    pub variant: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationship: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiseaseModelAssociation {
    pub model: String,
    #[serde(skip)]
    pub model_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organism: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationship: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiseasePrevalenceEvidence {
    pub estimate: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiseaseSurvival {
    pub site_code: u16,
    pub site_label: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub series: Vec<DiseaseSurvivalSeries>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiseaseSurvivalSeries {
    pub sex: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_observed: Option<DiseaseSurvivalPoint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_modeled: Option<DiseaseSurvivalPoint>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub points: Vec<DiseaseSurvivalPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiseaseSurvivalPoint {
    pub year: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relative_survival_rate: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub standard_error: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lower_ci: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upper_ci: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modeled_relative_survival_rate: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub case_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiseaseDisgenetAssociation {
    pub symbol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entrez_id: Option<u32>,
    pub score: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clinical_trial_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_index: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_level: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiseaseDisgenet {
    pub associations: Vec<DiseaseDisgenetAssociation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiseaseSearchResult {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub synonyms_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_via: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhenotypeSearchResult {
    pub disease_id: String,
    pub disease_name: String,
    pub score: f64,
}

#[derive(Debug, Clone, Default)]
pub struct DiseaseSearchFilters {
    pub query: Option<String>,
    pub source: Option<String>,
    pub inheritance: Option<String>,
    pub phenotype: Option<String>,
    pub onset: Option<String>,
}

const DISEASE_SECTION_TREATMENTS: &str = "treatments";
const DISEASE_SECTION_RECRUITING_TRIALS: &str = "recruiting_trials";
const DISEASE_SECTION_GENES: &str = "genes";
const DISEASE_SECTION_PATHWAYS: &str = "pathways";
const DISEASE_SECTION_PHENOTYPES: &str = "phenotypes";
const DISEASE_SECTION_DIAGNOSTICS: &str = "diagnostics";
const DISEASE_SECTION_VARIANTS: &str = "variants";
const DISEASE_SECTION_MODELS: &str = "models";
const DISEASE_SECTION_PREVALENCE: &str = "prevalence";
const DISEASE_SECTION_SURVIVAL: &str = "survival";
const DISEASE_SECTION_FUNDING: &str = "funding";
const DISEASE_SECTION_CIVIC: &str = "civic";
const DISEASE_SECTION_DISGENET: &str = "disgenet";
const DISEASE_SECTION_CLINICAL_FEATURES: &str = "clinical_features";
const DISEASE_SECTION_ALL: &str = "all";

pub const DISEASE_SECTION_NAMES: &[&str] = &[
    DISEASE_SECTION_GENES,
    DISEASE_SECTION_PATHWAYS,
    DISEASE_SECTION_PHENOTYPES,
    DISEASE_SECTION_DIAGNOSTICS,
    DISEASE_SECTION_VARIANTS,
    DISEASE_SECTION_MODELS,
    DISEASE_SECTION_PREVALENCE,
    DISEASE_SECTION_SURVIVAL,
    DISEASE_SECTION_FUNDING,
    DISEASE_SECTION_CIVIC,
    DISEASE_SECTION_DISGENET,
    DISEASE_SECTION_CLINICAL_FEATURES,
    DISEASE_SECTION_ALL,
];

mod associations;
mod enrichment;
mod fallback;
mod get;
mod resolution;
mod search;
#[cfg(test)]
mod test_support;
#[cfg(test)]
pub(crate) mod tests;

pub(crate) use self::fallback::fallback_search_page;
pub use self::get::get;
pub(crate) use self::get::get_with_context;
pub(crate) use self::resolution::resolve_disease_hit_by_name;
pub(crate) use self::search::validate_phenotype_search_window;
#[allow(unused_imports)]
pub use self::search::{
    PhenotypePagination, search_page, search_phenotype_page, search_query_summary,
};
