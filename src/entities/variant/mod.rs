//! Variant entity models and workflows exposed through the stable variant facade.

use crate::error::BioMcpError;
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

#[allow(unused_imports)]
pub(crate) use self::erepo::{
    ERepoAssertion, ERepoBatchInput, ERepoCriterion, ERepoGenePage, ERepoGeneResult, ERepoItem,
    ERepoResponse, ERepoSourceStatus, retrieve as retrieve_erepo, search_gene as search_erepo_gene,
};
pub use self::get::{VARIANT_SECTION_NAMES, get, get_with_workflow_signals, oncokb};
pub(crate) use self::gwas::validate_gwas_window;
#[allow(unused_imports)]
pub use self::gwas::{
    GwasPagination, GwasSearchPage, gwas_search_query_summary, search_gwas, search_gwas_page,
};
pub use self::normalization::{
    CarAliasCollection, CarNormalizationBatchResponse, CarNormalizationItem,
    CarNormalizationStatus, CarProvenance, VariantNormalizationAggregate,
    VariantNormalizationResponse, VariantNormalizationService, VariantNormalizationServiceResult,
    VariantNormalizationStatus, normalize_car, normalize_car_batch, normalize_variant,
};
pub use self::resolution::{
    classify_variant_input, parse_variant_id, parse_variant_protein_alias, variant_guidance,
};
#[allow(unused_imports)]
pub use self::search::{search, search_page, search_query_summary};
pub use self::structure::{VariantStructureResult, structure};

pub(crate) use self::gwas::validate_p_value as validate_gwas_p_value;
pub(crate) use self::normalization::{transcript_coding_hgvs_re, validate_car_hgvs_input};
#[allow(unused_imports)]
pub(crate) use self::resolution::{
    NormalizedGenomicCoordinate, NormalizedVariantAliases, RequestedVariantIdentity,
    SourceVariantIdentity, VariantArticleRequest, VariantArticleResolution,
    VariantArticleResolutionBasis, VariantArticleResolutionContext, VariantIdentityComparison,
    VariantProviderValidation, VariantProviderValidationStatus, VariantResolutionStatus,
    VariantSearchResolution, compare_variant_identity, gnomad_variant_slug, is_rsid,
    normalize_genomic_coordinate, normalize_protein_change, protein_change_segment,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GenomeBuild {
    #[serde(rename = "GRCh37")]
    Grch37,
    #[serde(rename = "GRCh38")]
    Grch38,
}

impl GenomeBuild {
    pub(crate) fn provider_value(self) -> &'static str {
        match self {
            Self::Grch37 => "hg19",
            Self::Grch38 => "hg38",
        }
    }
}

pub(crate) fn resolved_default_assembly(
    explicit: Option<GenomeBuild>,
) -> Result<GenomeBuild, BioMcpError> {
    if let Some(build) = explicit {
        return Ok(build);
    }
    match std::env::var("BIOMCP_DEFAULT_ASSEMBLY") {
        Ok(value) => value.parse().map_err(|_| {
            BioMcpError::InvalidArgument(
                "BIOMCP_DEFAULT_ASSEMBLY must be grch37, hg19, grch38, or hg38".into(),
            )
        }),
        Err(std::env::VarError::NotPresent) => Ok(GenomeBuild::Grch38),
        Err(_) => Err(BioMcpError::InvalidArgument(
            "BIOMCP_DEFAULT_ASSEMBLY must contain valid Unicode".into(),
        )),
    }
}

impl std::str::FromStr for GenomeBuild {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "hg19" | "grch37" => Ok(Self::Grch37),
            "hg38" | "grch38" => Ok(Self::Grch38),
            _ => Err("assembly must be hg19 or hg38 (GRCh37 and GRCh38 are aliases)".into()),
        }
    }
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
    pub genome_build: Option<GenomeBuild>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genome_build_provenance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_ambiguous: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub build_candidates: Vec<VariantBuildCandidate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hgvs_p: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legacy_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hgvs_c: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcript: Option<String>,
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
    pub population: Option<GnomadPopulationResult>,
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
pub struct VariantBuildCandidate {
    pub genome_build: GenomeBuild,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rsid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantGwasAssociation {
    pub rsid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trait_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p_value: Option<GwasPValue>,
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

/// A GWAS p-value that preserves the provider's exact scientific notation.
///
/// `numeric` is absent when the value cannot be represented by an `f64`
/// without underflowing to zero. Callers should use `scientific` when they
/// need a lossless display value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GwasPValue {
    pub scientific: String,
    pub mantissa: Option<i64>,
    pub exponent: Option<i32>,
    pub numeric: Option<f64>,
}

impl GwasPValue {
    const MIN_EXACT_EXPONENT: i32 = -1_000_000;

    pub(crate) fn from_numeric(value: f64) -> Option<Self> {
        if !value.is_finite() || value <= 0.0 || value > 1.0 {
            return None;
        }
        Some(Self {
            scientific: normalize_scientific(format!("{value:e}")),
            mantissa: None,
            exponent: None,
            numeric: Some(value),
        })
    }

    pub(crate) fn from_provider_parts(
        numeric: Option<f64>,
        mantissa: Option<i64>,
        exponent: Option<i32>,
    ) -> Option<Self> {
        match (mantissa, exponent) {
            (Some(mantissa), Some(exponent))
                if mantissa > 0
                    && (Self::MIN_EXACT_EXPONENT..=0).contains(&exponent)
                    && compare_positive_scientific(&format!("{mantissa}e{exponent}"), "1e0")
                        .is_le() =>
            {
                let representable = (mantissa as f64) * 10_f64.powi(exponent);
                Some(Self {
                    scientific: format!("{mantissa}e{exponent}"),
                    mantissa: Some(mantissa),
                    exponent: Some(exponent),
                    numeric: (representable.is_finite()
                        && representable > 0.0
                        && representable <= 1.0)
                        .then_some(representable),
                })
            }
            _ => numeric.and_then(Self::from_numeric),
        }
    }

    pub(crate) fn is_at_most(&self, threshold: f64) -> bool {
        Self::from_numeric(threshold).is_some_and(|limit| self.total_cmp(&limit).is_le())
    }

    pub(crate) fn total_cmp(&self, other: &Self) -> std::cmp::Ordering {
        compare_positive_scientific(&self.scientific, &other.scientific)
    }
}

fn normalize_scientific(value: String) -> String {
    let Some((coefficient, exponent)) = value.split_once('e') else {
        return value;
    };
    let coefficient = coefficient
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string();
    let exponent = exponent.parse::<i32>().unwrap_or_default();
    format!("{coefficient}e{exponent}")
}

fn compare_positive_scientific(left: &str, right: &str) -> std::cmp::Ordering {
    let Some((left_digits, left_exponent)) = scientific_parts(left) else {
        return left.cmp(right);
    };
    let Some((right_digits, right_exponent)) = scientific_parts(right) else {
        return left.cmp(right);
    };
    let left_magnitude = left_exponent + i64::try_from(left_digits.len()).unwrap_or(i64::MAX);
    let right_magnitude = right_exponent + i64::try_from(right_digits.len()).unwrap_or(i64::MAX);
    left_magnitude.cmp(&right_magnitude).then_with(|| {
        let width = left_digits.len().max(right_digits.len());
        let mut left_scaled = left_digits.clone();
        let mut right_scaled = right_digits.clone();
        left_scaled.extend(std::iter::repeat_n('0', width - left_scaled.len()));
        right_scaled.extend(std::iter::repeat_n('0', width - right_scaled.len()));
        left_scaled.cmp(&right_scaled)
    })
}

fn scientific_parts(value: &str) -> Option<(String, i64)> {
    let (coefficient, exponent) = value.split_once('e')?;
    let exponent = exponent.parse::<i64>().ok()?;
    let decimal_places = coefficient
        .split_once('.')
        .map_or(0, |(_, fraction)| fraction.len());
    let mut digits = coefficient.replace('.', "");
    if !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let leading = digits.len() - digits.trim_start_matches('0').len();
    digits.drain(..leading);
    if digits.is_empty() {
        return None;
    }
    let trailing = digits.len() - digits.trim_end_matches('0').len();
    digits.truncate(digits.len() - trailing);
    let exponent = exponent
        .checked_sub(i64::try_from(decimal_places).ok()?)?
        .checked_add(i64::try_from(trailing).ok()?)?;
    Some((digits, exponent))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GnomadPopulationResult {
    pub status: GnomadPopulationStatus,
    pub dataset: String,
    pub release: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub exome: Option<crate::sources::gnomad::GnomadSequencingPopulation>,
    pub genome: Option<crate::sources::gnomad::GnomadSequencingPopulation>,
    pub faf_caveat: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GnomadPopulationStatus {
    Data,
    Missing,
    Absent,
    ProviderFailure,
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
    pub genome_build: GenomeBuild,
    pub genome_build_provenance: String,
    pub gene: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hgvs_p: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hgvs_c: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcript: Option<String>,
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
    pub p_value: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariantIdFormat {
    RsId(String),
    HgvsGenomic(String),
    GeneProteinChange { gene: String, change: String },
}
