use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use std::time::Duration;
use tracing::warn;

use crate::entities::SearchPage;
use crate::error::BioMcpError;
use crate::sources::alphagenome::AlphaGenomeClient;
use crate::sources::cbioportal::CBioPortalClient;
use crate::sources::civic::{CivicClient, CivicContext, CivicEvidenceItem};
use crate::sources::gwas::{GwasAssociation, GwasClient};
use crate::sources::mygene::MyGeneClient;
use crate::sources::myvariant::{MyVariantClient, VariantSearchParams};
use crate::sources::oncokb::OncoKBAnnotation;
use crate::sources::oncokb::OncoKBClient;
use crate::transform;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variant {
    pub gene: String,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hgvs_p: Option<String>,
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gnomad_subpopulations: Vec<PopulationFrequency>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub population: Option<VariantPopulationSummary>,
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

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cancer_frequencies: Vec<crate::sources::cbioportal::CancerFrequency>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancer_frequency_source: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gwas: Vec<VariantGwasAssociation>,

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantPopulationSummary {
    pub dataset: String,
    pub variant_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub af: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataset_note: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub populations: Vec<PopulationFrequency>,
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
    pub significance: Option<String>,
    pub clinvar_stars: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gnomad_af: Option<f64>,
    pub revel: Option<f64>,
    pub gerp: Option<f64>,
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

#[derive(Debug, Clone, Default)]
pub struct VariantSearchFilters {
    pub gene: Option<String>,
    pub hgvsp: Option<String>,
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
}

#[derive(Debug, Clone, Default)]
pub struct GwasSearchFilters {
    pub gene: Option<String>,
    pub trait_query: Option<String>,
    pub region: Option<String>,
    pub p_value: Option<f64>,
}

const VARIANT_SECTION_PREDICT: &str = "predict";
const VARIANT_SECTION_PREDICTIONS: &str = "predictions";
const VARIANT_SECTION_CLINVAR: &str = "clinvar";
const VARIANT_SECTION_POPULATION: &str = "population";
const VARIANT_SECTION_CONSERVATION: &str = "conservation";
const VARIANT_SECTION_COSMIC: &str = "cosmic";
const VARIANT_SECTION_CGI: &str = "cgi";
const VARIANT_SECTION_CIVIC: &str = "civic";
const VARIANT_SECTION_CBIOPORTAL: &str = "cbioportal";
const VARIANT_SECTION_GWAS: &str = "gwas";
const VARIANT_SECTION_ALL: &str = "all";

pub const VARIANT_SECTION_NAMES: &[&str] = &[
    VARIANT_SECTION_PREDICT,
    VARIANT_SECTION_PREDICTIONS,
    VARIANT_SECTION_CLINVAR,
    VARIANT_SECTION_POPULATION,
    VARIANT_SECTION_CONSERVATION,
    VARIANT_SECTION_COSMIC,
    VARIANT_SECTION_CGI,
    VARIANT_SECTION_CIVIC,
    VARIANT_SECTION_CBIOPORTAL,
    VARIANT_SECTION_GWAS,
    VARIANT_SECTION_ALL,
];

const OPTIONAL_ENRICHMENT_TIMEOUT: Duration = Duration::from_secs(8);

#[derive(Debug, Clone, Copy, Default)]
struct VariantSections {
    include_prediction: bool,
    include_expanded_predictions: bool,
    include_clinvar: bool,
    include_population: bool,
    include_conservation: bool,
    include_cosmic: bool,
    include_cgi: bool,
    include_civic: bool,
    include_cbioportal: bool,
    include_gwas: bool,
}

fn parse_sections(sections: &[String]) -> Result<VariantSections, BioMcpError> {
    let mut out = VariantSections::default();
    let mut include_all = false;

    for raw in sections {
        let section = raw.trim().to_ascii_lowercase();
        if section.is_empty() {
            continue;
        }
        if section == "--json" || section == "-j" {
            continue;
        }
        match section.as_str() {
            VARIANT_SECTION_PREDICT => out.include_prediction = true,
            VARIANT_SECTION_PREDICTIONS => out.include_expanded_predictions = true,
            VARIANT_SECTION_CLINVAR => out.include_clinvar = true,
            VARIANT_SECTION_POPULATION => out.include_population = true,
            VARIANT_SECTION_CONSERVATION => out.include_conservation = true,
            VARIANT_SECTION_COSMIC => out.include_cosmic = true,
            VARIANT_SECTION_CGI => out.include_cgi = true,
            VARIANT_SECTION_CIVIC => out.include_civic = true,
            VARIANT_SECTION_CBIOPORTAL => out.include_cbioportal = true,
            VARIANT_SECTION_GWAS => out.include_gwas = true,
            VARIANT_SECTION_ALL => include_all = true,
            _ => {
                return Err(BioMcpError::InvalidArgument(format!(
                    "Unknown section \"{section}\" for variant. Available: {}",
                    VARIANT_SECTION_NAMES.join(", ")
                )));
            }
        }
    }

    if include_all {
        out.include_prediction = true;
        out.include_expanded_predictions = true;
        out.include_clinvar = true;
        out.include_population = true;
        out.include_conservation = true;
        out.include_cosmic = true;
        out.include_cgi = true;
        out.include_civic = true;
        out.include_cbioportal = true;
        out.include_gwas = true;
    }

    Ok(out)
}

pub enum VariantIdFormat {
    RsId(String),
    HgvsGenomic(String),
    GeneProteinChange { gene: String, change: String },
}

fn rsid_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)^(rs\d+)$").expect("valid regex"))
}

fn hgvs_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^(chr[0-9XYM]+:g\.\d+[ACGT]>[ACGT])$").expect("valid regex"))
}

fn hgvs_coords_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^(chr[0-9XYM]+):g\.(\d+)([ACGT])>([ACGT])$").expect("valid regex")
    })
}

fn gene_protein_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^([A-Z][A-Z0-9]+)\s+([A-Z]\d+[A-Z*])$").expect("valid regex"))
}

pub fn parse_variant_id(id: &str) -> Result<VariantIdFormat, BioMcpError> {
    let id = id.trim();
    if id.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Variant ID is required. Example: biomcp get variant rs113488022".into(),
        ));
    }

    if let Some(caps) = rsid_re().captures(id) {
        return Ok(VariantIdFormat::RsId(caps[1].to_ascii_lowercase()));
    }
    if let Some(caps) = hgvs_re().captures(id) {
        return Ok(VariantIdFormat::HgvsGenomic(caps[1].to_string()));
    }
    if let Some(caps) = gene_protein_re().captures(id) {
        return Ok(VariantIdFormat::GeneProteinChange {
            gene: caps[1].to_string(),
            change: caps[2].to_string(),
        });
    }

    Err(BioMcpError::InvalidArgument(format!(
        "Unrecognized variant format: '{id}'\n\n\
Supported formats:\n\
- rsID: rs113488022\n\
- HGVS genomic: chr7:g.140453136A>T\n\
- Gene + protein: BRAF V600E"
    )))
}

fn score_myvariant_hit(hit: &crate::sources::myvariant::MyVariantHit) -> i32 {
    let mut score = 0;
    if let Some(clinvar) = hit.clinvar.as_ref() {
        if !clinvar.rcv.is_empty() {
            score += 100;
            score += clinvar.rcv.len().min(50) as i32;
        }
        if clinvar.variant_id.is_some() {
            score += 5;
        }
    }
    if hit.dbnsfp.as_ref().and_then(|d| d.hgvsp.first()).is_some() {
        score += 10;
    }
    if hit.dbsnp.as_ref().and_then(|d| d.rsid.as_ref()).is_some() {
        score += 5;
    }
    score
}

fn best_hit(
    hits: &[crate::sources::myvariant::MyVariantHit],
) -> Option<&crate::sources::myvariant::MyVariantHit> {
    hits.iter().max_by_key(|h| score_myvariant_hit(h))
}

fn amino_acid_one_letter(token: &str) -> Option<char> {
    match token.trim().to_ascii_uppercase().as_str() {
        "A" | "ALA" => Some('A'),
        "R" | "ARG" => Some('R'),
        "N" | "ASN" => Some('N'),
        "D" | "ASP" => Some('D'),
        "C" | "CYS" => Some('C'),
        "Q" | "GLN" => Some('Q'),
        "E" | "GLU" => Some('E'),
        "G" | "GLY" => Some('G'),
        "H" | "HIS" => Some('H'),
        "I" | "ILE" => Some('I'),
        "L" | "LEU" => Some('L'),
        "K" | "LYS" => Some('K'),
        "M" | "MET" => Some('M'),
        "F" | "PHE" => Some('F'),
        "P" | "PRO" => Some('P'),
        "S" | "SER" => Some('S'),
        "T" | "THR" => Some('T'),
        "W" | "TRP" => Some('W'),
        "Y" | "TYR" => Some('Y'),
        "V" | "VAL" => Some('V'),
        "*" | "TER" | "STOP" => Some('*'),
        _ => None,
    }
}

fn normalize_oncokb_protein_change(value: &str) -> Option<String> {
    let trimmed = value
        .trim()
        .trim_start_matches("p.")
        .trim_start_matches("P.");
    if trimmed.is_empty() {
        return None;
    }

    let bytes = trimmed.as_bytes();
    let start_digits = bytes.iter().position(|b| b.is_ascii_digit())?;
    let end_digits = bytes[start_digits..]
        .iter()
        .position(|b| !b.is_ascii_digit())
        .map(|idx| start_digits + idx)
        .unwrap_or(bytes.len());
    if start_digits == 0 || end_digits <= start_digits || end_digits >= bytes.len() {
        return None;
    }

    let from = amino_acid_one_letter(&trimmed[..start_digits])?;
    let pos = trimmed[start_digits..end_digits].trim();
    let to = amino_acid_one_letter(&trimmed[end_digits..])?;
    if pos.is_empty() {
        return None;
    }

    Some(format!("{from}{pos}{to}"))
}

fn oncokb_alteration_from_variant(
    variant: &Variant,
    id_format: &VariantIdFormat,
) -> Option<String> {
    match id_format {
        VariantIdFormat::GeneProteinChange { change, .. } => {
            normalize_oncokb_protein_change(change).or_else(|| Some(change.clone()))
        }
        _ => variant
            .hgvs_p
            .as_deref()
            .and_then(normalize_oncokb_protein_change)
            .filter(|s| !s.is_empty()),
    }
}

fn therapies_from_oncokb(annotation: &OncoKBAnnotation) -> Vec<TreatmentImplication> {
    let mut implications: Vec<TreatmentImplication> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    for treatment in &annotation.treatments {
        let level = treatment
            .level
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(transform::variant::normalize_oncokb_level)
            .unwrap_or_else(|| "Unknown".to_string());
        let mut drugs = treatment
            .drugs
            .iter()
            .filter_map(|d| d.drug_name.as_deref())
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>();
        drugs.sort();
        drugs.dedup();
        let cancer_type = treatment
            .cancer_type
            .as_ref()
            .and_then(|c| c.name.as_deref())
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string);
        let dedupe_key = format!(
            "{}|{}|{}",
            level,
            drugs.join("+"),
            cancer_type.as_deref().unwrap_or("")
        );
        if !seen.insert(dedupe_key) {
            continue;
        }
        implications.push(TreatmentImplication {
            level,
            drugs,
            cancer_type,
            note: None,
        });
    }

    implications.sort_by(|a, b| a.level.cmp(&b.level));
    let total = implications.len();
    if total > 6 {
        implications.truncate(6);
        if let Some(last) = implications.last_mut() {
            last.note = Some(format!("(and {} more)", total - 6));
        }
    }
    implications
}

fn search_result_quality_score(row: &VariantSearchResult) -> i32 {
    let mut score = 0;
    if row
        .significance
        .as_deref()
        .map(str::trim)
        .is_some_and(|v| !v.is_empty())
    {
        score += 4;
    }
    if row.gnomad_af.is_some() {
        score += 4;
    }
    if row.clinvar_stars.is_some() {
        score += 3;
    }
    if row.revel.is_some() {
        score += 2;
    }
    if row.gerp.is_some() {
        score += 2;
    }
    if row
        .hgvs_p
        .as_deref()
        .map(str::trim)
        .is_some_and(|v| !v.is_empty())
    {
        score += 2;
    }
    if !row.gene.trim().is_empty() {
        score += 1;
    }
    score
}

pub fn search_query_summary(filters: &VariantSearchFilters) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(v) = filters
        .gene
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("gene={v}"));
    }
    if let Some(v) = filters
        .hgvsp
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("hgvsp={v}"));
    }
    if let Some(v) = filters
        .significance
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("significance={v}"));
    }
    if let Some(v) = filters.max_frequency {
        parts.push(format!("max_frequency={v}"));
    }
    if let Some(v) = filters.min_cadd {
        parts.push(format!("min_cadd={v}"));
    }
    if let Some(v) = filters
        .consequence
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("consequence={v}"));
    }
    if let Some(v) = filters
        .review_status
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("review_status={v}"));
    }
    if let Some(v) = filters
        .population
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("population={v}"));
    }
    if let Some(v) = filters.revel_min {
        parts.push(format!("revel_min={v}"));
    }
    if let Some(v) = filters.gerp_min {
        parts.push(format!("gerp_min={v}"));
    }
    if let Some(v) = filters
        .tumor_site
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("tumor_site={v}"));
    }
    if let Some(v) = filters
        .condition
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("condition={v}"));
    }
    if let Some(v) = filters
        .impact
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("impact={v}"));
    }
    if filters.lof {
        parts.push("lof=true".to_string());
    }
    if let Some(v) = filters
        .has
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("has={v}"));
    }
    if let Some(v) = filters
        .missing
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("missing={v}"));
    }
    if let Some(v) = filters
        .therapy
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("therapy={v}"));
    }

    parts.join(", ")
}

#[allow(dead_code)]
pub async fn search(
    filters: &VariantSearchFilters,
    limit: usize,
) -> Result<Vec<VariantSearchResult>, BioMcpError> {
    Ok(search_page(filters, limit, 0).await?.results)
}

pub async fn search_page(
    filters: &VariantSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<SearchPage<VariantSearchResult>, BioMcpError> {
    const MAX_SEARCH_LIMIT: usize = 50;
    if limit == 0 || limit > MAX_SEARCH_LIMIT {
        return Err(BioMcpError::InvalidArgument(format!(
            "--limit must be between 1 and {MAX_SEARCH_LIMIT}"
        )));
    }

    let has_precision_filter = filters
        .hgvsp
        .as_deref()
        .map(str::trim)
        .is_some_and(|v| !v.is_empty())
        || filters
            .significance
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters.max_frequency.is_some()
        || filters.min_cadd.is_some()
        || filters
            .review_status
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .population
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters.revel_min.is_some()
        || filters.gerp_min.is_some()
        || filters
            .tumor_site
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .condition
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .impact
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters.lof
        || filters
            .has
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .missing
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .therapy
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .consequence
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty());
    let fetch_limit = if has_precision_filter {
        limit
    } else {
        (limit.saturating_mul(40)).clamp(limit, 200)
    };

    let params = VariantSearchParams {
        gene: filters.gene.clone(),
        hgvsp: filters.hgvsp.clone(),
        significance: filters.significance.clone(),
        max_frequency: filters.max_frequency,
        min_cadd: filters.min_cadd,
        consequence: filters.consequence.clone(),
        review_status: filters.review_status.clone(),
        population: filters.population.clone(),
        revel_min: filters.revel_min,
        gerp_min: filters.gerp_min,
        tumor_site: filters.tumor_site.clone(),
        condition: filters.condition.clone(),
        impact: filters.impact.clone(),
        lof: filters.lof,
        has: filters.has.clone(),
        missing: filters.missing.clone(),
        therapy: filters.therapy.clone(),
        limit: fetch_limit,
        offset,
    };

    let client = MyVariantClient::new()?;
    let resp = client.search(&params).await?;
    let mut out = resp
        .hits
        .iter()
        .map(transform::variant::from_myvariant_search_hit)
        .collect::<Vec<_>>();
    out.sort_by(|a, b| {
        search_result_quality_score(b)
            .cmp(&search_result_quality_score(a))
            .then_with(|| a.id.cmp(&b.id))
    });
    out.truncate(limit);
    Ok(SearchPage::offset(out, resp.total))
}

#[allow(dead_code)]
pub async fn search_gwas(
    filters: &GwasSearchFilters,
    limit: usize,
) -> Result<Vec<VariantGwasAssociation>, BioMcpError> {
    Ok(search_gwas_page(filters, limit, 0).await?.results)
}

pub async fn search_gwas_page(
    filters: &GwasSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<SearchPage<VariantGwasAssociation>, BioMcpError> {
    const MAX_SEARCH_LIMIT: usize = 50;
    if limit == 0 || limit > MAX_SEARCH_LIMIT {
        return Err(BioMcpError::InvalidArgument(format!(
            "--limit must be between 1 and {MAX_SEARCH_LIMIT}"
        )));
    }

    let needed = limit.saturating_add(offset).max(limit);

    let gene = filters
        .gene
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);
    let trait_query = filters
        .trait_query
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);
    let region = filters
        .region
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);
    let p_value_threshold = filters.p_value;

    if gene.is_none() && trait_query.is_none() && region.is_none() {
        return Err(BioMcpError::InvalidArgument(
            "Provide -g <gene>, --trait <text>, or --region <chr:start-end>. Example: biomcp search gwas -g TCF7L2".into(),
        ));
    }

    let client = GwasClient::new()?;
    let mut rows: Vec<VariantGwasAssociation> = Vec::new();

    if let Some(gene) = gene.as_deref() {
        let snps = client
            .snps_by_gene(gene, (needed.saturating_mul(5)).clamp(needed, 200))
            .await?;
        for rsid in unique_rsids_from_snps(&snps, needed.saturating_mul(2)) {
            let associations = client.associations_by_rsid(&rsid, 3).await?;
            if associations.is_empty() {
                rows.push(VariantGwasAssociation {
                    rsid,
                    trait_name: None,
                    p_value: None,
                    effect_size: None,
                    effect_type: None,
                    confidence_interval: None,
                    risk_allele_frequency: None,
                    risk_allele: None,
                    mapped_genes: vec![gene.to_string()],
                    study_accession: None,
                    pmid: None,
                    author: None,
                    sample_description: None,
                });
                continue;
            }
            if let Some(best) = associations
                .iter()
                .filter_map(|a| map_gwas_association(a, Some(&rsid)))
                .min_by(|a, b| {
                    a.p_value
                        .unwrap_or(f64::INFINITY)
                        .total_cmp(&b.p_value.unwrap_or(f64::INFINITY))
                })
            {
                rows.push(best);
            }
        }
    }

    if let Some(trait_query) = trait_query.as_deref() {
        let snps = client
            .snps_by_trait(trait_query, (needed.saturating_mul(5)).clamp(needed, 200))
            .await?;
        for rsid in unique_rsids_from_snps(&snps, needed.saturating_mul(2)) {
            let associations = client.associations_by_rsid(&rsid, 3).await?;
            for assoc in associations {
                if let Some(row) = map_gwas_association(&assoc, Some(&rsid)) {
                    rows.push(row);
                }
            }
        }

        if rows.len() < needed {
            let studies = client
                .studies_by_trait(trait_query, needed.saturating_mul(2).clamp(needed, 50))
                .await?;
            for study in studies {
                let Some(accession) = study.accession_id.as_deref() else {
                    continue;
                };
                let associations = client
                    .associations_by_study(accession, needed.saturating_mul(3).clamp(needed, 100))
                    .await?;
                for assoc in associations {
                    if let Some(row) = map_gwas_association(&assoc, None) {
                        rows.push(row);
                    }
                }
                if rows.len() >= needed.saturating_mul(3) {
                    break;
                }
            }
        }
    }

    let mut rows = dedupe_gwas_rows(rows, needed)?;
    if let Some(threshold) = p_value_threshold {
        rows.retain(|row| row.p_value.is_some_and(|v| v <= threshold));
    }
    let results = rows.drain(..).skip(offset).take(limit).collect::<Vec<_>>();
    Ok(SearchPage::offset(results, None))
}

pub fn gwas_search_query_summary(filters: &GwasSearchFilters) -> String {
    let mut parts = Vec::new();
    if let Some(gene) = filters
        .gene
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("gene={gene}"));
    }
    if let Some(trait_query) = filters
        .trait_query
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("trait={trait_query}"));
    }
    if let Some(region) = filters
        .region
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("region={region}"));
    }
    if let Some(p_value) = filters.p_value {
        parts.push(format!("p_value={p_value}"));
    }
    parts.join(", ")
}

fn unique_rsids_from_snps(snps: &[crate::sources::gwas::GwasSnp], limit: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for row in snps {
        let Some(rsid) = row
            .rs_id
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_ascii_lowercase)
        else {
            continue;
        };
        if !seen.insert(rsid.clone()) {
            continue;
        }
        out.push(rsid);
        if out.len() >= limit {
            break;
        }
    }
    out
}

fn dedupe_gwas_rows(
    mut rows: Vec<VariantGwasAssociation>,
    limit: usize,
) -> Result<Vec<VariantGwasAssociation>, BioMcpError> {
    let mut seen = std::collections::HashSet::new();
    rows.retain(|row| {
        let key = format!(
            "{}|{}|{}",
            row.rsid.to_ascii_lowercase(),
            row.trait_name
                .as_deref()
                .unwrap_or_default()
                .to_ascii_lowercase(),
            row.study_accession
                .as_deref()
                .unwrap_or_default()
                .to_ascii_uppercase()
        );
        seen.insert(key)
    });

    rows.sort_by(|a, b| {
        a.p_value
            .unwrap_or(f64::INFINITY)
            .total_cmp(&b.p_value.unwrap_or(f64::INFINITY))
            .then_with(|| a.rsid.cmp(&b.rsid))
    });
    rows.truncate(limit);
    Ok(rows)
}

async fn resolve_base(id: &str) -> Result<(Variant, VariantIdFormat), BioMcpError> {
    let id = id.trim();
    if id.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Variant ID is required. Example: biomcp get variant rs113488022".into(),
        ));
    }

    let id_format = parse_variant_id(id)?;

    let myvariant = MyVariantClient::new()?;
    let hit = match &id_format {
        VariantIdFormat::HgvsGenomic(hgvs) => myvariant.get(hgvs).await?,
        VariantIdFormat::RsId(rsid) => {
            let q = format!("dbsnp.rsid:{rsid}");
            let resp = myvariant
                .query_with_fields(&q, 10, 0, crate::sources::myvariant::MYVARIANT_FIELDS_GET)
                .await?;
            best_hit(&resp.hits)
                .cloned()
                .ok_or_else(|| BioMcpError::NotFound {
                    entity: "variant".into(),
                    id: rsid.to_string(),
                    suggestion: format!("Try searching: biomcp search variant -g \"{id}\""),
                })?
        }
        VariantIdFormat::GeneProteinChange { gene, change } => {
            let q = format!(
                "dbnsfp.genename:{} AND dbnsfp.hgvsp:\"p.{}\"",
                gene,
                MyVariantClient::escape_query_value(change)
            );
            let resp = myvariant
                .query_with_fields(&q, 5, 0, crate::sources::myvariant::MYVARIANT_FIELDS_GET)
                .await?;
            resp.hits
                .into_iter()
                .next()
                .ok_or_else(|| BioMcpError::NotFound {
                    entity: "variant".into(),
                    id: id.to_string(),
                    suggestion: format!(
                        "Try searching: biomcp search variant -g {gene} --hgvsp {change}"
                    ),
                })?
        }
    };

    let variant = transform::variant::from_myvariant_hit(&hit);
    Ok((variant, id_format))
}

async fn get_base(id: &str) -> Result<Variant, BioMcpError> {
    let (variant, _) = resolve_base(id).await?;
    Ok(variant)
}

pub async fn oncokb(id: &str) -> Result<VariantOncoKbResult, BioMcpError> {
    let (variant, id_format) = resolve_base(id).await?;
    let gene = variant.gene.trim();
    if gene.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "OncoKB lookup requires a variant that resolves to a gene symbol".into(),
        ));
    }

    let alteration = oncokb_alteration_from_variant(&variant, &id_format)
        .ok_or_else(|| {
            BioMcpError::InvalidArgument(
                "OncoKB lookup requires a protein change (e.g., `BRAF V600E`)".into(),
            )
        })?
        .trim()
        .to_string();
    if alteration.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "OncoKB lookup requires a non-empty protein alteration".into(),
        ));
    }

    let client = OncoKBClient::new()?;
    let annotation = client.annotate_best_effort(gene, &alteration).await?;
    let oncogenic = annotation
        .oncogenic
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);
    let level = annotation
        .highest_sensitive_level
        .as_deref()
        .map(transform::variant::normalize_oncokb_level)
        .filter(|v| !v.is_empty())
        .or_else(|| {
            annotation
                .highest_resistance_level
                .as_deref()
                .map(transform::variant::normalize_oncokb_level)
                .filter(|v| !v.is_empty())
        });
    let effect = annotation
        .mutation_effect
        .as_ref()
        .and_then(|m| m.known_effect.as_deref())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);

    Ok(VariantOncoKbResult {
        gene: gene.to_string(),
        alteration,
        oncogenic,
        level,
        effect,
        therapies: therapies_from_oncokb(&annotation),
    })
}

async fn add_prediction(variant: &mut Variant) -> Result<(), BioMcpError> {
    let Some(caps) = hgvs_coords_re().captures(&variant.id) else {
        warn!(
            variant_id = %variant.id,
            "AlphaGenome prediction skipped (unsupported HGVS format)"
        );
        return Ok(());
    };

    let chr = caps[1].to_string();
    let pos: i64 = caps[2]
        .parse()
        .map_err(|_| BioMcpError::InvalidArgument("Invalid HGVS position for prediction".into()))?;
    let reference = caps[3].to_string();
    let alternate = caps[4].to_string();

    let client = AlphaGenomeClient::new().await?;
    match client
        .score_variant(&chr, pos, &reference, &alternate)
        .await
    {
        Ok(mut pred) => {
            if let Some(top_gene) = pred.top_gene.as_deref()
                && top_gene.trim().starts_with("ENSG")
            {
                let query = format!("ensembl.gene:\"{}\"", top_gene.trim());
                match MyGeneClient::new() {
                    Ok(client) => {
                        if let Ok(resp) = client.search(&query, 1, 0, None).await
                            && let Some(symbol) = resp
                                .hits
                                .first()
                                .and_then(|h| h.symbol.as_deref())
                                .map(str::trim)
                                .filter(|s| !s.is_empty())
                        {
                            pred.top_gene = Some(symbol.to_string());
                        }
                    }
                    Err(err) => {
                        warn!("MyGene unavailable for AlphaGenome gene resolution: {err}")
                    }
                }
            }
            transform::variant::merge_prediction(variant, pred)
        }
        Err(err) => warn!(variant_id = %variant.id, "AlphaGenome unavailable: {err}"),
    }

    Ok(())
}

async fn add_cbioportal(variant: &mut Variant) {
    let gene = variant.gene.trim();
    if gene.is_empty() {
        return;
    }

    let cbio_fut = async {
        let client = CBioPortalClient::new()?;
        let summary = client.get_mutation_summary(gene).await?;
        Ok::<_, BioMcpError>(summary)
    };

    match tokio::time::timeout(OPTIONAL_ENRICHMENT_TIMEOUT, cbio_fut).await {
        Ok(Ok(summary)) => transform::variant::merge_cbioportal(variant, &summary),
        Ok(Err(err)) => warn!(gene = %variant.gene, "cBioPortal unavailable: {err}"),
        Err(_) => warn!(
            gene = %variant.gene,
            timeout_secs = OPTIONAL_ENRICHMENT_TIMEOUT.as_secs(),
            "cBioPortal enrichment timed out"
        ),
    }
}

fn civic_molecular_profile_name(variant: &Variant) -> Option<String> {
    let gene = variant.gene.trim();
    if gene.is_empty() {
        return None;
    }

    if let Some(hgvs_p) = variant
        .hgvs_p
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let normalized = hgvs_p.strip_prefix("p.").unwrap_or(hgvs_p).trim();
        if !normalized.is_empty() {
            return Some(format!("{gene} {normalized}"));
        }
    }

    None
}

async fn add_civic(variant: &mut Variant) {
    let Some(molecular_profile_name) = civic_molecular_profile_name(variant) else {
        return;
    };

    let civic_fut = async {
        let client = CivicClient::new()?;
        client
            .by_molecular_profile(&molecular_profile_name, 10)
            .await
    };

    match tokio::time::timeout(OPTIONAL_ENRICHMENT_TIMEOUT, civic_fut).await {
        Ok(Ok(context)) => {
            let section = variant
                .civic
                .get_or_insert_with(VariantCivicSection::default);
            section.graphql = Some(context);
        }
        Ok(Err(err)) => warn!(
            molecular_profile = %molecular_profile_name,
            "CIViC enrichment unavailable: {err}"
        ),
        Err(_) => warn!(
            molecular_profile = %molecular_profile_name,
            timeout_secs = OPTIONAL_ENRICHMENT_TIMEOUT.as_secs(),
            "CIViC enrichment timed out"
        ),
    }
}

fn rsid_from_risk_allele(value: &str) -> Option<String> {
    let token = value.trim();
    if token.is_empty() {
        return None;
    }
    let prefix = token.split('-').next().unwrap_or(token).trim();
    if prefix.len() < 3 || !prefix.to_ascii_lowercase().starts_with("rs") {
        return None;
    }
    Some(prefix.to_ascii_lowercase())
}

fn association_rsid(association: &GwasAssociation, fallback: Option<&str>) -> Option<String> {
    if let Some(rsid) = association
        .snps
        .iter()
        .filter_map(|snp| snp.rs_id.as_deref())
        .map(str::trim)
        .find(|v| !v.is_empty())
        .map(str::to_ascii_lowercase)
    {
        return Some(rsid);
    }

    if let Some(rsid) = association
        .loci
        .iter()
        .flat_map(|locus| locus.strongest_risk_alleles.iter())
        .filter_map(|allele| allele.risk_allele_name.as_deref())
        .find_map(rsid_from_risk_allele)
    {
        return Some(rsid);
    }

    fallback
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_ascii_lowercase)
}

fn association_trait_name(association: &GwasAssociation) -> Option<String> {
    association
        .efo_traits
        .iter()
        .filter_map(|row| row.trait_field.as_deref())
        .map(str::trim)
        .find(|v| !v.is_empty())
        .map(str::to_string)
        .or_else(|| {
            association
                .study
                .as_ref()
                .and_then(|study| study.disease_trait.as_ref())
                .and_then(|trait_row| trait_row.trait_field.as_deref())
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_string)
        })
}

fn association_risk_allele(association: &GwasAssociation) -> Option<String> {
    association
        .loci
        .iter()
        .flat_map(|locus| locus.strongest_risk_alleles.iter())
        .filter_map(|allele| allele.risk_allele_name.as_deref())
        .map(str::trim)
        .find(|v| !v.is_empty())
        .map(str::to_string)
}

fn association_genes(association: &GwasAssociation) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for gene in association
        .loci
        .iter()
        .flat_map(|locus| locus.author_reported_genes.iter())
        .filter_map(|gene| gene.gene_name.as_deref())
    {
        let symbol = gene.trim();
        if symbol.is_empty() {
            continue;
        }
        let key = symbol.to_ascii_uppercase();
        if !seen.insert(key) {
            continue;
        }
        out.push(symbol.to_string());
    }
    out
}

fn association_sample_description(association: &GwasAssociation) -> Option<String> {
    let study = association.study.as_ref()?;
    let mut parts = Vec::new();
    if let Some(v) = study
        .initial_sample_size
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("initial: {v}"));
    }
    if let Some(v) = study
        .replication_sample_size
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty() && !v.eq_ignore_ascii_case("na"))
    {
        parts.push(format!("replication: {v}"));
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("; "))
    }
}

fn map_gwas_association(
    association: &GwasAssociation,
    fallback_rsid: Option<&str>,
) -> Option<VariantGwasAssociation> {
    let rsid = association_rsid(association, fallback_rsid)?;
    let (effect_size, effect_type) = if let Some(v) = association.or_per_copy_num {
        (Some(v), Some("OR".to_string()))
    } else if let Some(v) = association.beta_num {
        (Some(v), Some("beta".to_string()))
    } else {
        (None, None)
    };

    let study_accession = association
        .study
        .as_ref()
        .and_then(|study| study.accession_id.as_deref())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);
    let pmid = association
        .study
        .as_ref()
        .and_then(|study| study.publication_info.as_ref())
        .and_then(|pubinfo| pubinfo.pubmed_id.as_deref())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);
    let author = association
        .study
        .as_ref()
        .and_then(|study| study.publication_info.as_ref())
        .and_then(|pubinfo| pubinfo.author.as_ref())
        .and_then(|author| author.fullname.as_deref())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);

    Some(VariantGwasAssociation {
        rsid,
        trait_name: association_trait_name(association),
        p_value: association.pvalue,
        effect_size,
        effect_type,
        confidence_interval: association
            .range
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string),
        risk_allele_frequency: association.risk_frequency,
        risk_allele: association_risk_allele(association),
        mapped_genes: association_genes(association),
        study_accession,
        pmid,
        author,
        sample_description: association_sample_description(association),
    })
}

async fn add_gwas_section(variant: &mut Variant, query_id: &str) -> Result<(), BioMcpError> {
    let fallback_rsid = parse_variant_id(query_id)
        .ok()
        .and_then(|parsed| match parsed {
            VariantIdFormat::RsId(rsid) => Some(rsid),
            _ => None,
        });

    let rsid = variant
        .rsid
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_ascii_lowercase)
        .or(fallback_rsid);

    let Some(rsid) = rsid else {
        return Ok(());
    };

    let client = GwasClient::new()?;
    let associations = client.associations_by_rsid(&rsid, 30).await?;
    let mut rows: Vec<VariantGwasAssociation> = associations
        .iter()
        .filter_map(|assoc| map_gwas_association(assoc, Some(&rsid)))
        .collect();
    rows = dedupe_gwas_rows(rows, 10)?;
    variant.gwas = rows;
    Ok(())
}

fn strip_clinvar_details(variant: &mut Variant) {
    variant.conditions.clear();
    variant.clinvar_conditions.clear();
    variant.clinvar_condition_reports = None;
    variant.clinvar_id = None;
    variant.clinvar_review_status = None;
    variant.clinvar_review_stars = None;
}

pub async fn get(id: &str, sections: &[String]) -> Result<Variant, BioMcpError> {
    let section_flags = parse_sections(sections)?;
    let mut variant = get_base(id).await?;

    if !section_flags.include_clinvar {
        strip_clinvar_details(&mut variant);
    }
    if !section_flags.include_conservation {
        variant.conservation = None;
    }
    if !section_flags.include_expanded_predictions {
        variant.expanded_predictions.clear();
    }
    if !section_flags.include_population {
        variant.population_breakdown = None;
    }
    if !section_flags.include_cosmic {
        variant.cosmic_context = None;
    }
    if !section_flags.include_cgi {
        variant.cgi_associations.clear();
    }
    if !section_flags.include_civic {
        variant.civic = None;
    }
    if !section_flags.include_cbioportal {
        variant.cancer_frequencies.clear();
    }
    if !section_flags.include_gwas {
        variant.gwas.clear();
    }
    if section_flags.include_prediction {
        add_prediction(&mut variant).await?;
    }
    if section_flags.include_cbioportal {
        add_cbioportal(&mut variant).await;
    }
    if section_flags.include_civic {
        add_civic(&mut variant).await;
    }
    if section_flags.include_gwas {
        add_gwas_section(&mut variant, id).await?;
    }

    Ok(variant)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_variant_id_examples() {
        match parse_variant_id("rs113488022").unwrap() {
            VariantIdFormat::RsId(v) => assert_eq!(v, "rs113488022"),
            _ => panic!("expected rsid"),
        }
        match parse_variant_id("chr7:g.140453136A>T").unwrap() {
            VariantIdFormat::HgvsGenomic(v) => assert_eq!(v, "chr7:g.140453136A>T"),
            _ => panic!("expected hgvs"),
        }
        match parse_variant_id("BRAF V600E").unwrap() {
            VariantIdFormat::GeneProteinChange { gene, change } => {
                assert_eq!(gene, "BRAF");
                assert_eq!(change, "V600E");
            }
            _ => panic!("expected gene+protein"),
        }
    }

    #[test]
    fn parse_variant_id_egfr_l858r() {
        match parse_variant_id("EGFR L858R").unwrap() {
            VariantIdFormat::GeneProteinChange { gene, change } => {
                assert_eq!(gene, "EGFR");
                assert_eq!(change, "L858R");
            }
            _ => panic!("expected gene+protein"),
        }
    }

    #[test]
    fn parse_variant_id_kras_g12c() {
        match parse_variant_id("KRAS G12C").unwrap() {
            VariantIdFormat::GeneProteinChange { gene, change } => {
                assert_eq!(gene, "KRAS");
                assert_eq!(change, "G12C");
            }
            _ => panic!("expected gene+protein"),
        }
    }

    #[test]
    fn parse_variant_id_normalizes_uppercase_rsid_prefix() {
        match parse_variant_id("RS113488022").unwrap() {
            VariantIdFormat::RsId(v) => assert_eq!(v, "rs113488022"),
            _ => panic!("expected rsid"),
        }
    }

    #[test]
    fn quality_score_prioritizes_significance_and_frequency() {
        let rich = VariantSearchResult {
            id: "chr1:g.1A>T".into(),
            gene: "TP53".into(),
            hgvs_p: Some("p.V1A".into()),
            significance: Some("Pathogenic".into()),
            clinvar_stars: None,
            gnomad_af: Some(0.001),
            revel: None,
            gerp: None,
        };
        let sparse = VariantSearchResult {
            id: "chr1:g.2A>T".into(),
            gene: "TP53".into(),
            hgvs_p: Some("p.V2A".into()),
            significance: None,
            clinvar_stars: None,
            gnomad_af: None,
            revel: None,
            gerp: None,
        };

        assert!(search_result_quality_score(&rich) > search_result_quality_score(&sparse));
    }

    #[test]
    fn parse_sections_supports_new_variant_sections() {
        let flags = parse_sections(&[
            "conservation".to_string(),
            "predictions".to_string(),
            "cosmic".to_string(),
            "cgi".to_string(),
            "civic".to_string(),
            "cbioportal".to_string(),
            "gwas".to_string(),
        ])
        .expect("sections should parse");

        assert!(flags.include_conservation);
        assert!(flags.include_expanded_predictions);
        assert!(flags.include_cosmic);
        assert!(flags.include_cgi);
        assert!(flags.include_civic);
        assert!(flags.include_cbioportal);
        assert!(flags.include_gwas);
    }

    #[test]
    fn civic_molecular_profile_name_prefers_gene_and_hgvs_p() {
        let variant = Variant {
            gene: "BRAF".into(),
            id: "chr7:g.140453136A>T".into(),
            hgvs_p: Some("p.V600E".into()),
            hgvs_c: None,
            rsid: None,
            cosmic_id: None,
            significance: None,
            clinvar_id: None,
            clinvar_review_status: None,
            clinvar_review_stars: None,
            conditions: Vec::new(),
            gnomad_af: None,
            gnomad_subpopulations: Vec::new(),
            population: None,
            consequence: None,
            cadd_score: None,
            sift_pred: None,
            polyphen_pred: None,
            conservation: None,
            expanded_predictions: Vec::new(),
            population_breakdown: None,
            cosmic_context: None,
            cgi_associations: Vec::new(),
            civic: None,
            clinvar_conditions: Vec::new(),
            clinvar_condition_reports: None,
            cancer_frequencies: Vec::new(),
            cancer_frequency_source: None,
            gwas: Vec::new(),
            prediction: None,
        };

        assert_eq!(
            civic_molecular_profile_name(&variant).as_deref(),
            Some("BRAF V600E")
        );
    }

    #[test]
    fn therapies_from_oncokb_truncation_shows_count() {
        let annotation: OncoKBAnnotation = serde_json::from_value(serde_json::json!({
            "treatments": [
                {"level": "LEVEL_1", "drugs": [{"drugName": "osimertinib"}], "cancerType": {"name": "Lung"}},
                {"level": "LEVEL_2", "drugs": [{"drugName": "afatinib"}], "cancerType": {"name": "Lung"}},
                {"level": "LEVEL_3A", "drugs": [{"drugName": "erlotinib"}], "cancerType": {"name": "Lung"}},
                {"level": "LEVEL_3B", "drugs": [{"drugName": "gefitinib"}], "cancerType": {"name": "Lung"}},
                {"level": "LEVEL_4", "drugs": [{"drugName": "dacomitinib"}], "cancerType": {"name": "Lung"}},
                {"level": "LEVEL_R1", "drugs": [{"drugName": "poziotinib"}], "cancerType": {"name": "Lung"}},
                {"level": "LEVEL_R2", "drugs": [{"drugName": "mobocertinib"}], "cancerType": {"name": "Lung"}}
            ]
        }))
        .expect("valid OncoKB annotation");

        let therapies = therapies_from_oncokb(&annotation);
        assert_eq!(therapies.len(), 6);
        assert!(
            therapies
                .last()
                .and_then(|row| row.note.as_deref())
                .is_some_and(|note| note.contains("(and 1 more)"))
        );
    }
}
