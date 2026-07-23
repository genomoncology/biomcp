//! Variant entity models and workflows exposed through the stable variant facade.

use serde::{Deserialize, Serialize};

use crate::entities::section_outcome::SectionOutcomes;
use crate::entities::source_state_registry::outcome_keys;
use crate::sources::civic::{CivicContext, CivicEvidenceItem};

mod erepo;
mod get;
mod gwas;
mod normalization;
mod resolution;
mod search;
mod structure;
#[cfg(test)]
mod test_support;

pub(crate) use self::erepo::{ERepoBatchInput, retrieve as retrieve_erepo};
pub use self::get::{VARIANT_SECTION_NAMES, get, get_with_workflow_signals, oncokb};
#[allow(unused_imports)]
pub use self::gwas::{gwas_search_query_summary, search_gwas, search_gwas_page};
pub use self::normalization::{
    VariantNormalizationResponse, VariantNormalizationService, VariantNormalizationServiceResult,
    VariantNormalizationStatus, normalize_variant,
};
pub use self::resolution::{
    classify_variant_input, parse_variant_id, parse_variant_protein_alias, variant_guidance,
};
#[allow(unused_imports)]
pub use self::search::{search, search_page, search_query_summary};
pub use self::structure::{VariantStructureResult, structure};

pub(crate) use self::gwas::validate_p_value as validate_gwas_p_value;
pub(crate) use self::normalization::transcript_coding_hgvs_re;
#[allow(unused_imports)]
pub(crate) use self::resolution::{
    NormalizedVariantAliases, RequestedVariantIdentity, SourceVariantIdentity,
    VariantArticleRequest, VariantArticleResolution, VariantArticleResolutionBasis,
    VariantArticleResolutionContext, VariantIdentityComparison, VariantProviderValidation,
    VariantProviderValidationStatus, VariantResolutionStatus, VariantSearchResolution,
    compare_variant_identity, gnomad_variant_slug, is_rsid, normalize_protein_change,
    protein_change_segment,
};
pub(crate) use self::search::resolve_article_variant_identity;

pub(crate) fn default_variant_section_outcomes() -> SectionOutcomes {
    SectionOutcomes::with_keys(&outcome_keys("variant"))
}

fn deserialize_variant_section_outcomes<'de, D>(
    deserializer: D,
) -> Result<SectionOutcomes, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let outcomes = SectionOutcomes::deserialize(deserializer)?;
    outcomes
        .validate_keys(&outcome_keys("variant"))
        .map_err(serde::de::Error::custom)?;
    Ok(outcomes)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variant {
    #[serde(
        default = "default_variant_section_outcomes",
        deserialize_with = "deserialize_variant_section_outcomes"
    )]
    pub section_outcomes: SectionOutcomes,
    pub gene: String,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hgvs_p: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legacy_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hgvs_c: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rsid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cosmic_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub significance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clinvar_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clinvar_review_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clinvar_review_stars: Option<u8>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conditions: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub gnomad_af: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allele_frequency_raw: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allele_frequency_percent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consequence: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cadd_score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sift_pred: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub polyphen_pred: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conservation: Option<VariantConservationScores>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expanded_predictions: Vec<VariantPredictionScore>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub population_breakdown: Option<VariantPopulationBreakdown>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cosmic_context: Option<VariantCosmicContext>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cgi_associations: Vec<VariantCgiAssociation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub civic: Option<VariantCivicSection>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clinvar_conditions: Vec<ConditionReportCount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clinvar_condition_reports: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_disease: Option<ConditionReportCount>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancerhotspots: Option<crate::sources::cancerhotspots::CancerHotspotRecurrence>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cancer_frequencies: Vec<crate::sources::cbioportal::CancerFrequency>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancer_frequency_source: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gwas: Vec<VariantGwasAssociation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gwas_unavailable_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supporting_pmids: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub prediction: Option<VariantPrediction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantGwasAssociation {
    pub rsid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trait_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p_value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect_size: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence_interval: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_allele_frequency: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_allele: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mapped_genes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub study_accession: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pmid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationFrequency {
    pub population: String,
    pub af: f64,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_subgroup: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantPopulationBreakdown {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub populations: Vec<PopulationFrequency>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exac_af: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exac_nontcga_af: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantConservationScores {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phylop_100way_vertebrate: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phylop_470way_mammalian: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phastcons_100way_vertebrate: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phastcons_470way_mammalian: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gerp_rs: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantPredictionScore {
    pub tool: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prediction: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantCosmicContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mut_freq: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tumor_site: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mut_nt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantCgiAssociation {
    pub drug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub association: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tumor_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VariantCivicSection {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cached_evidence: Vec<CivicEvidenceItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graphql: Option<CivicContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreatmentImplication {
    pub level: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub drugs: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancer_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionReportCount {
    pub condition: String,
    pub reports: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantPrediction {
    /// Gene expression log fold change (RNA-seq)
    pub expression_lfc: Option<f64>,
    /// Splice site disruption score
    pub splice_score: Option<f64>,
    /// Chromatin accessibility score (DNase)
    pub chromatin_score: Option<f64>,
    /// Top affected gene
    pub top_gene: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantSearchResult {
    pub id: String,
    pub gene: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hgvs_p: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legacy_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub significance: Option<String>,
    pub clinvar_stars: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gnomad_af: Option<f64>,
    pub revel: Option<f64>,
    pub gerp: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_identity: Option<SourceVariantIdentity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_alias: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantOncoKbResult {
    pub gene: String,
    pub alteration: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oncogenic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub therapies: Vec<TreatmentImplication>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VariantProteinAlias {
    pub position: u32,
    pub residue: char,
}

impl VariantProteinAlias {
    pub fn label(&self) -> String {
        format!("{}{}", self.position, self.residue)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariantShorthand {
    GeneResidueAlias {
        gene: String,
        alias: String,
        position: u32,
        residue: char,
    },
    ProteinChangeOnly {
        change: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariantInputKind {
    Exact(VariantIdFormat),
    TranscriptCodingHgvs(String),
    Shorthand(VariantShorthand),
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VariantGuidanceKind {
    GeneResidueAlias { gene: String, alias: String },
    ProteinChangeOnly { change: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VariantGuidance {
    pub query: String,
    pub kind: VariantGuidanceKind,
    pub next_commands: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct VariantSearchFilters {
    pub gene: Option<String>,
    pub hgvsp: Option<String>,
    pub hgvsc: Option<String>,
    pub rsid: Option<String>,
    pub protein_alias: Option<VariantProteinAlias>,
    pub significance: Option<String>,
    pub max_frequency: Option<f64>,
    pub min_cadd: Option<f64>,
    pub consequence: Option<String>,
    pub review_status: Option<String>,
    pub population: Option<String>,
    pub revel_min: Option<f64>,
    pub gerp_min: Option<f64>,
    pub tumor_site: Option<String>,
    pub condition: Option<String>,
    pub impact: Option<String>,
    pub lof: bool,
    pub has: Option<String>,
    pub missing: Option<String>,
    pub therapy: Option<String>,
    pub(crate) requested_identity: Option<RequestedVariantIdentity>,
}

#[derive(Debug, Clone, Default)]
pub struct GwasSearchFilters {
    pub gene: Option<String>,
    pub trait_query: Option<String>,
    pub region: Option<String>,
    pub p_value: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariantIdFormat {
    RsId(String),
    HgvsGenomic(String),
    GeneProteinChange { gene: String, change: String },
}
