pub(crate) mod cspec;

use std::collections::HashMap;
use std::fs;
use std::future::Future;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use futures::future::try_join_all;
use serde::{Deserialize, Serialize};
use tracing::{debug as warn, warn as local_warn};

use crate::entities::SearchPage;
use crate::entities::diagnostic::{DiagnosticSearchFilters, DiagnosticSearchResult};
use crate::entities::section_outcome::{SectionOutcome, SectionOutcomeState, SectionOutcomes};
use crate::entities::source_state_registry::outcome_keys;
use crate::error::BioMcpError;
use crate::sources::civic::{CivicClient, CivicContext};
use crate::sources::clingen::{ClinGenClient, GeneClinGen};
use crate::sources::dgidb::{
    DgidbClient, GeneDruggability, GeneSafetyLiability, GeneTractabilityModality,
};
use crate::sources::disgenet::{DisgenetAssociationRecord, DisgenetClient};
use crate::sources::enrichr::EnrichrClient;
use crate::sources::gnomad::{
    GNOMAD_CONSTRAINT_REFERENCE_GENOME, GNOMAD_CONSTRAINT_VERSION, GnomadClient,
};
use crate::sources::gtex::{GeneExpression, GtexClient};
use crate::sources::hpa::{GeneHpa, HpaClient};
use crate::sources::mygene::{MyGeneClient, MyGeneHit};
use crate::sources::nih_reporter::{NihReporterClient, NihReporterFundingSection};
use crate::sources::opentargets::{
    OpenTargetsClient, OpenTargetsTargetClinicalContext, OpenTargetsTargetDruggabilityContext,
};
use crate::sources::quickgo::QuickGoClient;
use crate::sources::reactome::ReactomeClient;
use crate::sources::string::StringClient;
use crate::sources::uniprot::UniProtClient;
use crate::transform;

/// Gene entity from MyGene.info plus optional enrichment sections.
fn default_gene_section_outcomes() -> SectionOutcomes {
    SectionOutcomes::with_keys(&outcome_keys("gene"))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gene {
    #[serde(
        default = "default_gene_section_outcomes",
        deserialize_with = "deserialize_gene_section_outcomes"
    )]
    pub section_outcomes: SectionOutcomes,
    pub symbol: String,
    pub name: String,
    pub entrez_id: String,
    pub ensembl_id: Option<String>,
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genomic_coordinates: Option<super::GenomicCoordinate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub omim_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uniprot_id: Option<String>,
    pub summary: Option<String>,
    pub gene_type: Option<String>,
    pub aliases: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clinical_diseases: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clinical_drugs: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pathways: Option<Vec<GenePathway>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ontology: Option<Vec<EnrichmentResult>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diseases: Option<Vec<EnrichmentResult>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protein: Option<GeneProtein>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub go: Option<Vec<GeneGoTerm>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interactions: Option<Vec<GeneInteraction>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub civic: Option<CivicContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expression: Option<GeneExpression>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hpa: Option<GeneHpa>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub druggability: Option<GeneDruggability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clingen: Option<GeneClinGen>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraint: Option<GeneConstraint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disgenet: Option<GeneDisgenet>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub funding: Option<NihReporterFundingSection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub funding_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<Vec<DiagnosticSearchResult>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenePathway {
    pub source: String,
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneProtein {
    pub accession: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub isoforms: Vec<GeneProteinIsoform>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alternative_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneProteinIsoform {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneGoTerm {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneInteraction {
    pub partner: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneConstraint {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pli: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loeuf: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mis_z: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub syn_z: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcript: Option<String>,
    pub source: String,
    pub source_version: String,
    pub reference_genome: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneDisgenetAssociation {
    pub disease_name: String,
    pub disease_cui: String,
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
pub struct GeneDisgenet {
    pub associations: Vec<GeneDisgenetAssociation>,
}

/// Search result (lighter than full Gene)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneSearchResult {
    pub symbol: String,
    pub name: String,
    pub entrez_id: String,
    pub genomic_coordinates: Option<super::GenomicCoordinate>,
    pub uniprot_id: Option<String>,
    pub omim_id: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct GeneSearchFilters {
    pub query: Option<String>,
    pub gene_type: Option<String>,
    pub chromosome: Option<String>,
    pub region: Option<String>,
    pub pathway: Option<String>,
    pub go_term: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneIncludeType {
    Pathways,
    Ontology,
    Diseases,
    Diagnostics,
    Protein,
    Go,
    Interactions,
    Civic,
    Expression,
    Hpa,
    Druggability,
    ClinGen,
    Constraint,
    Disgenet,
    Funding,
}

pub type GeneSection = GeneIncludeType;

const GENE_SECTION_PATHWAYS: &str = "pathways";
const GENE_SECTION_ONTOLOGY: &str = "ontology";
const GENE_SECTION_DISEASES: &str = "diseases";
const GENE_SECTION_DIAGNOSTICS: &str = "diagnostics";
const GENE_SECTION_PROTEIN: &str = "protein";
const GENE_SECTION_GO: &str = "go";
const GENE_SECTION_INTERACTIONS: &str = "interactions";
const GENE_SECTION_CIVIC: &str = "civic";
const GENE_SECTION_EXPRESSION: &str = "expression";
const GENE_SECTION_HPA: &str = "hpa";
const GENE_SECTION_DRUGGABILITY: &str = "druggability";
const GENE_SECTION_CLINGEN: &str = "clingen";
const GENE_SECTION_CONSTRAINT: &str = "constraint";
const GENE_SECTION_DISGENET: &str = "disgenet";
const GENE_SECTION_FUNDING: &str = "funding";
const GENE_SECTION_ALL: &str = "all";
pub(crate) const GENE_OUTCOME_KEYS: &[&str] = &[
    GENE_SECTION_PATHWAYS,
    GENE_SECTION_ONTOLOGY,
    GENE_SECTION_DISEASES,
    GENE_SECTION_DIAGNOSTICS,
    GENE_SECTION_PROTEIN,
    GENE_SECTION_GO,
    GENE_SECTION_INTERACTIONS,
    GENE_SECTION_CIVIC,
    GENE_SECTION_EXPRESSION,
    GENE_SECTION_HPA,
    GENE_SECTION_DRUGGABILITY,
    GENE_SECTION_CLINGEN,
    GENE_SECTION_CONSTRAINT,
    GENE_SECTION_DISGENET,
    GENE_SECTION_FUNDING,
];

fn deserialize_gene_section_outcomes<'de, D>(deserializer: D) -> Result<SectionOutcomes, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let outcomes = SectionOutcomes::deserialize(deserializer)?;
    outcomes
        .validate_keys(&outcome_keys("gene"))
        .map_err(serde::de::Error::custom)?;
    Ok(outcomes)
}

pub const GENE_SECTION_NAMES: &[&str] = &[
    GENE_SECTION_PATHWAYS,
    GENE_SECTION_ONTOLOGY,
    GENE_SECTION_DISEASES,
    GENE_SECTION_DIAGNOSTICS,
    GENE_SECTION_PROTEIN,
    GENE_SECTION_GO,
    GENE_SECTION_INTERACTIONS,
    GENE_SECTION_CIVIC,
    GENE_SECTION_EXPRESSION,
    GENE_SECTION_HPA,
    GENE_SECTION_DRUGGABILITY,
    GENE_SECTION_CLINGEN,
    GENE_SECTION_CONSTRAINT,
    GENE_SECTION_DISGENET,
    GENE_SECTION_FUNDING,
    GENE_SECTION_ALL,
];

impl GeneIncludeType {
    pub fn from_section(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            GENE_SECTION_PATHWAYS | "pathway" => Some(Self::Pathways),
            GENE_SECTION_ONTOLOGY => Some(Self::Ontology),
            GENE_SECTION_DISEASES | "disease" => Some(Self::Diseases),
            GENE_SECTION_DIAGNOSTICS => Some(Self::Diagnostics),
            GENE_SECTION_PROTEIN => Some(Self::Protein),
            GENE_SECTION_GO => Some(Self::Go),
            GENE_SECTION_INTERACTIONS | "interaction" => Some(Self::Interactions),
            GENE_SECTION_CIVIC => Some(Self::Civic),
            GENE_SECTION_EXPRESSION => Some(Self::Expression),
            GENE_SECTION_HPA => Some(Self::Hpa),
            GENE_SECTION_DRUGGABILITY | "drugs" => Some(Self::Druggability),
            GENE_SECTION_CLINGEN => Some(Self::ClinGen),
            GENE_SECTION_CONSTRAINT => Some(Self::Constraint),
            GENE_SECTION_DISGENET => Some(Self::Disgenet),
            GENE_SECTION_FUNDING => Some(Self::Funding),
            _ => None,
        }
    }

    // dead-code reason: gene::as_str is exercised by native tests or binary dispatch
    #[allow(dead_code)]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pathways => GENE_SECTION_PATHWAYS,
            Self::Ontology => GENE_SECTION_ONTOLOGY,
            Self::Diseases => GENE_SECTION_DISEASES,
            Self::Diagnostics => GENE_SECTION_DIAGNOSTICS,
            Self::Protein => GENE_SECTION_PROTEIN,
            Self::Go => GENE_SECTION_GO,
            Self::Interactions => GENE_SECTION_INTERACTIONS,
            Self::Civic => GENE_SECTION_CIVIC,
            Self::Expression => GENE_SECTION_EXPRESSION,
            Self::Hpa => GENE_SECTION_HPA,
            Self::Druggability => GENE_SECTION_DRUGGABILITY,
            Self::ClinGen => GENE_SECTION_CLINGEN,
            Self::Constraint => GENE_SECTION_CONSTRAINT,
            Self::Disgenet => GENE_SECTION_DISGENET,
            Self::Funding => GENE_SECTION_FUNDING,
        }
    }

    // dead-code reason: gene::all_default is exercised by native tests or binary dispatch
    #[allow(dead_code)]
    pub fn all_default() -> Vec<Self> {
        vec![
            Self::Pathways,
            Self::Ontology,
            Self::Diseases,
            Self::Protein,
            Self::Go,
            Self::Interactions,
            Self::Civic,
            Self::Expression,
            Self::Hpa,
            Self::Druggability,
            Self::ClinGen,
            Self::Constraint,
        ]
    }

    fn libraries(&self) -> &'static [&'static str] {
        match self {
            // Pathways come from Reactome directly, not Enrichr.
            Self::Pathways => &[],
            Self::Diagnostics => &[],
            Self::Ontology => &["GO_Biological_Process_2025", "GO_Molecular_Function_2025"],
            Self::Diseases => &["DisGeNET", "OMIM_Disease"],
            Self::Protein
            | Self::Go
            | Self::Interactions
            | Self::Civic
            | Self::Expression
            | Self::Hpa
            | Self::Druggability
            | Self::ClinGen
            | Self::Constraint
            | Self::Disgenet
            | Self::Funding => &[],
        }
    }
}

const DEFAULT_OPTIONAL_ENRICHMENT_TIMEOUT_MS: u64 = 8_000;
const GENE_TIMING_PATH_ENV: &str = "BIOMCP_GENE_TIMING_PATH";
const GENE_OPTIONAL_TIMEOUT_MS_ENV: &str = "BIOMCP_GENE_OPTIONAL_TIMEOUT_MS";
const GENE_GET_STRATEGY_ENV: &str = "BIOMCP_GENE_GET_STRATEGY";
const FUNDING_NO_DATA_NOTE: &str = "No NIH funding data found for this query.";
const FUNDING_UNAVAILABLE_NOTE: &str = "NIH Reporter funding data is temporarily unavailable.";
const DIAGNOSTIC_PIVOT_LIMIT: usize = 10;
const GENE_DIAGNOSTICS_UNAVAILABLE_NOTE: &str =
    "Diagnostic local data is unavailable. Run `biomcp gtr sync` to enable gene diagnostic pivots.";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneTimingEntry {
    pub section: String,
    pub elapsed_ms: u128,
    pub outcome: SectionOutcomeState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneTimingReport {
    pub symbol: String,
    pub strategy: String,
    pub total_ms: u128,
    pub sections: Vec<GeneTimingEntry>,
}

#[derive(Debug, Clone)]
pub struct GeneGetOptions {
    pub sections: Vec<GeneSection>,
    pub strategy: GeneGetStrategy,
    pub optional_timeout: Duration,
    pub timing_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneGetResult {
    pub gene: Gene,
    pub timing: GeneTimingReport,
}

#[derive(Debug)]
struct GeneTimingCollector {
    symbol: String,
    strategy: String,
    started: Instant,
    path: Option<PathBuf>,
    sections: Vec<GeneTimingEntry>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum GeneGetStrategy {
    Baseline,
    OpenTargetsEnsembl,
    #[default]
    ParallelTop,
}

impl GeneGetStrategy {
    pub fn from_env() -> Self {
        match std::env::var(GENE_GET_STRATEGY_ENV)
            .ok()
            .map(|value| value.trim().to_ascii_lowercase())
            .as_deref()
        {
            Some("baseline") => Self::Baseline,
            Some("opentargets-ensembl") => Self::OpenTargetsEnsembl,
            Some("parallel-top") => Self::ParallelTop,
            _ => Self::ParallelTop,
        }
    }

    // dead-code reason: gene::from_name is exercised by native tests or binary dispatch
    #[allow(dead_code)]
    pub fn from_name(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "baseline" => Some(Self::Baseline),
            "opentargets-ensembl" => Some(Self::OpenTargetsEnsembl),
            "parallel-top" => Some(Self::ParallelTop),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Baseline => "baseline",
            Self::OpenTargetsEnsembl => "opentargets-ensembl",
            Self::ParallelTop => "parallel-top",
        }
    }

    fn prefers_known_opentargets_id(self) -> bool {
        matches!(self, Self::OpenTargetsEnsembl | Self::ParallelTop)
    }
}

impl Default for GeneGetOptions {
    fn default() -> Self {
        Self {
            sections: Vec::new(),
            strategy: GeneGetStrategy::default(),
            optional_timeout: Duration::from_millis(DEFAULT_OPTIONAL_ENRICHMENT_TIMEOUT_MS),
            timing_path: None,
        }
    }
}

impl GeneGetOptions {
    // dead-code reason: gene::with_sections is exercised by native tests or binary dispatch
    #[allow(dead_code)]
    pub fn with_sections(mut self, sections: Vec<GeneSection>) -> Self {
        self.sections = sections;
        self
    }

    // dead-code reason: gene::with_strategy is exercised by native tests or binary dispatch
    #[allow(dead_code)]
    pub fn with_strategy(mut self, strategy: GeneGetStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    // dead-code reason: gene::with_optional_timeout is exercised by native tests or binary dispatch
    #[allow(dead_code)]
    pub fn with_optional_timeout(mut self, timeout: Duration) -> Self {
        self.optional_timeout = timeout;
        self
    }

    // dead-code reason: gene::with_timing_path is exercised by native tests or binary dispatch
    #[allow(dead_code)]
    pub fn with_timing_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.timing_path = Some(path.into());
        self
    }

    pub fn from_env_and_sections(symbol: &str, sections: &[String]) -> Result<Self, BioMcpError> {
        Ok(Self {
            sections: parse_sections(symbol, sections)?,
            strategy: GeneGetStrategy::from_env(),
            optional_timeout: optional_enrichment_timeout_from_env(),
            timing_path: std::env::var(GENE_TIMING_PATH_ENV)
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .map(PathBuf::from),
        })
    }
}

impl GeneTimingCollector {
    fn new(symbol: &str, strategy: GeneGetStrategy, path: Option<PathBuf>) -> Self {
        Self {
            symbol: symbol.trim().to_string(),
            strategy: strategy.as_str().to_string(),
            started: Instant::now(),
            path,
            sections: Vec::new(),
        }
    }

    fn record(&mut self, section: &str, started: Instant, outcome: impl AsRef<str>) {
        self.sections.push(GeneTimingEntry {
            section: section.to_string(),
            elapsed_ms: started.elapsed().as_millis(),
            outcome: timing_outcome_state(outcome.as_ref()),
        });
    }

    fn push(&mut self, entry: GeneTimingEntry) {
        self.sections.push(entry);
    }

    fn finish(self) -> GeneTimingReport {
        let report = GeneTimingReport {
            symbol: self.symbol,
            strategy: self.strategy,
            total_ms: self.started.elapsed().as_millis(),
            sections: self.sections,
        };

        let Some(path) = self.path else {
            return report;
        };

        let bytes = match serde_json::to_vec_pretty(&report) {
            Ok(bytes) => bytes,
            Err(err) => {
                local_warn!(path = %path.display(), "Failed to serialize gene timing report: {err}");
                return report;
            }
        };

        if let Some(parent) = path.parent()
            && let Err(err) = fs::create_dir_all(parent)
        {
            local_warn!(path = %parent.display(), "Failed to create gene timing directory: {err}");
            return report;
        }

        if let Err(err) = fs::write(&path, bytes) {
            local_warn!(path = %path.display(), "Failed to write gene timing report: {err}");
        }

        report
    }
}

fn should_use_parallel_top(include: &[GeneIncludeType]) -> bool {
    include.iter().any(|section| {
        matches!(
            section,
            GeneIncludeType::Ontology
                | GeneIncludeType::Diseases
                | GeneIncludeType::Expression
                | GeneIncludeType::Hpa
                | GeneIncludeType::Druggability
                | GeneIncludeType::ClinGen
        )
    })
}

fn optional_enrichment_timeout_from_env() -> Duration {
    let millis = std::env::var(GENE_OPTIONAL_TIMEOUT_MS_ENV)
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_OPTIONAL_ENRICHMENT_TIMEOUT_MS);
    Duration::from_millis(millis)
}

fn preferred_opentargets_id(gene: &Gene, strategy: GeneGetStrategy) -> Option<&str> {
    if !strategy.prefers_known_opentargets_id() {
        return None;
    }

    gene.ensembl_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn timing_outcome_state(outcome: &str) -> SectionOutcomeState {
    match outcome {
        "data" => SectionOutcomeState::Data,
        "empty" => SectionOutcomeState::Empty,
        "degraded" => SectionOutcomeState::Degraded,
        _ => SectionOutcomeState::Unavailable,
    }
}

async fn timed_section<T, F, C>(section: &str, fut: F, classify: C) -> (T, GeneTimingEntry)
where
    F: Future<Output = T>,
    C: FnOnce(&T) -> String,
{
    let started = Instant::now();
    let value = fut.await;
    let outcome = classify(&value);
    (
        value,
        GeneTimingEntry {
            section: section.to_string(),
            elapsed_ms: started.elapsed().as_millis(),
            outcome: timing_outcome_state(&outcome),
        },
    )
}

fn complete_gene_section_outcomes(gene: &mut Gene, include: &[GeneIncludeType]) {
    for section in include {
        let key = section.as_str();
        if gene
            .section_outcomes
            .get(key)
            .is_some_and(|outcome| outcome.outcome() != SectionOutcomeState::NotRequested)
        {
            continue;
        }
        let (has_data, unavailable, source) = match section {
            GeneIncludeType::Pathways => (
                gene.pathways.as_ref().is_some_and(|rows| !rows.is_empty()),
                false,
                "Reactome",
            ),
            GeneIncludeType::Ontology => (
                gene.ontology
                    .as_ref()
                    .is_some_and(|rows| rows.iter().any(|row| !row.terms.is_empty())),
                false,
                "Enrichr",
            ),
            GeneIncludeType::Diseases => (
                gene.diseases
                    .as_ref()
                    .is_some_and(|rows| rows.iter().any(|row| !row.terms.is_empty())),
                false,
                "Enrichr",
            ),
            GeneIncludeType::Diagnostics => (
                gene.diagnostics
                    .as_ref()
                    .is_some_and(|rows| !rows.is_empty()),
                gene.diagnostics_note.as_deref() == Some(GENE_DIAGNOSTICS_UNAVAILABLE_NOTE),
                "NCBI Genetic Testing Registry",
            ),
            GeneIncludeType::Protein => (gene.protein.is_some(), false, "UniProt"),
            GeneIncludeType::Go => (
                gene.go.as_ref().is_some_and(|rows| !rows.is_empty()),
                false,
                "QuickGO",
            ),
            GeneIncludeType::Interactions => (
                gene.interactions
                    .as_ref()
                    .is_some_and(|rows| !rows.is_empty()),
                false,
                "STRING",
            ),
            GeneIncludeType::Civic => (
                gene.civic.as_ref().is_some_and(|value| {
                    !value.evidence_items.is_empty() || !value.assertions.is_empty()
                }),
                false,
                "CIViC",
            ),
            GeneIncludeType::Expression => (
                gene.expression
                    .as_ref()
                    .is_some_and(|value| !value.tissues.is_empty()),
                false,
                "GTEx",
            ),
            GeneIncludeType::Hpa => (
                gene.hpa.as_ref().is_some_and(|value| {
                    !value.tissues.is_empty()
                        || !value.subcellular_main_location.is_empty()
                        || !value.subcellular_additional_location.is_empty()
                        || value.reliability.is_some()
                        || value.protein_summary.is_some()
                        || value.rna_summary.is_some()
                }),
                false,
                "Human Protein Atlas",
            ),
            GeneIncludeType::Druggability => (
                gene.druggability.as_ref().is_some_and(|value| {
                    !value.categories.is_empty()
                        || !value.interactions.is_empty()
                        || !value.tractability.is_empty()
                        || !value.safety_liabilities.is_empty()
                }),
                false,
                "DGIdb / Open Targets",
            ),
            GeneIncludeType::ClinGen => (
                gene.clingen.as_ref().is_some_and(|value| {
                    !value.validity.is_empty()
                        || value.haploinsufficiency.is_some()
                        || value.triplosensitivity.is_some()
                }),
                false,
                "ClinGen",
            ),
            GeneIncludeType::Constraint => (
                gene.constraint.as_ref().is_some_and(|value| {
                    value.pli.is_some()
                        || value.loeuf.is_some()
                        || value.mis_z.is_some()
                        || value.syn_z.is_some()
                        || value.transcript.is_some()
                }),
                false,
                "gnomAD",
            ),
            GeneIncludeType::Disgenet => (
                gene.disgenet
                    .as_ref()
                    .is_some_and(|value| !value.associations.is_empty()),
                false,
                "DisGeNET",
            ),
            GeneIncludeType::Funding => (
                gene.funding
                    .as_ref()
                    .is_some_and(|value| !value.grants.is_empty()),
                gene.funding_note.as_deref() == Some(FUNDING_UNAVAILABLE_NOTE),
                "NIH Reporter",
            ),
        };
        let outcome = if unavailable {
            SectionOutcome::unavailable("The requested gene section is unavailable.")
        } else if key == GENE_SECTION_DRUGGABILITY {
            if has_data {
                SectionOutcome::data_sources(["DGIdb", "Open Targets"])
            } else {
                SectionOutcome::empty_sources(["DGIdb", "Open Targets"])
            }
        } else if has_data {
            SectionOutcome::data(source)
        } else {
            SectionOutcome::empty(source)
        };
        gene.section_outcomes.complete(key, outcome);
    }
}

fn sync_timing_outcomes(timing: &mut GeneTimingCollector, gene: &Gene) {
    for entry in &mut timing.sections {
        if let Some(outcome) = gene.section_outcomes.get(&entry.section)
            && outcome.outcome() != SectionOutcomeState::NotRequested
        {
            entry.outcome = outcome.outcome();
        }
    }
}

fn classify_clingen_section(section: &(GeneClinGen, SectionOutcome)) -> String {
    section.1.outcome().as_str().to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichmentResult {
    pub library: String,
    pub terms: Vec<EnrichmentTerm>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichmentTerm {
    pub name: String,
    pub p_value: f64,
    pub genes: String,
}

pub(crate) fn looks_like_symbol(query: &str) -> bool {
    if query.is_empty() {
        return false;
    }
    query
        .chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-')
        && query.chars().any(|c| c.is_ascii_uppercase())
}

pub(crate) fn mygene_query_term(query: &str) -> String {
    if looks_like_symbol(query) {
        let escaped = MyGeneClient::escape_query_value(query);
        format!("(symbol:{escaped} OR alias:{escaped})")
    } else {
        MyGeneClient::escape_query_value(query)
    }
}

fn normalized_alias_key(value: &str) -> String {
    value.trim().to_ascii_uppercase()
}

fn matching_canonical_alias_symbols(query: &str, hits: &[MyGeneHit]) -> Vec<String> {
    let query = normalized_alias_key(query);
    let mut out = Vec::new();
    for hit in hits {
        let Some(symbol) = hit
            .symbol
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            continue;
        };
        if hit.entrezgene.is_none() {
            continue;
        }
        let symbol_matches = normalized_alias_key(symbol) == query;
        let alias_matches = hit
            .alias
            .clone()
            .into_vec()
            .iter()
            .any(|alias| normalized_alias_key(alias) == query);
        if (symbol_matches || alias_matches) && !out.iter().any(|existing| existing == symbol) {
            out.push(symbol.to_string());
        }
    }
    out
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CanonicalGeneAlias {
    pub(crate) symbol: String,
    pub(crate) entrez_id: String,
}

fn matching_canonical_aliases(query: &str, hits: &[MyGeneHit]) -> Vec<CanonicalGeneAlias> {
    let symbols = matching_canonical_alias_symbols(query, hits);
    symbols
        .into_iter()
        .filter_map(|symbol| {
            let hit = hits.iter().find(|hit| {
                hit.symbol
                    .as_deref()
                    .is_some_and(|value| value.eq_ignore_ascii_case(&symbol))
            })?;
            Some(CanonicalGeneAlias {
                symbol,
                entrez_id: hit.entrezgene.as_ref()?.as_string(),
            })
        })
        .collect()
}

async fn unique_canonical_alias_symbol(
    client: &MyGeneClient,
    query: &str,
) -> Result<Option<String>, BioMcpError> {
    let resp = client
        .search(&mygene_query_term(query), 10, 0, None)
        .await?;
    let matches = matching_canonical_alias_symbols(query, &resp.hits);
    Ok((matches.len() == 1).then(|| matches[0].clone()))
}

pub(crate) async fn resolve_unique_canonical_alias(
    query: &str,
) -> Result<Option<CanonicalGeneAlias>, BioMcpError> {
    let client = MyGeneClient::new()?;
    let resp = client
        .search(&mygene_query_term(query), 10, 0, None)
        .await?;
    let mut matches = matching_canonical_aliases(query, &resp.hits);
    Ok((matches.len() == 1).then(|| matches.remove(0)))
}

fn normalize_gene_type(value: &str) -> Result<&'static str, BioMcpError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "protein-coding" => Ok("protein-coding"),
        "ncrna" => Ok("ncRNA"),
        "pseudo" => Ok("pseudo"),
        _ => Err(BioMcpError::InvalidArgument(
            "--type must be one of: protein-coding, ncrna, pseudo".into(),
        )),
    }
}

fn normalize_gene_chromosome(value: &str) -> Result<String, BioMcpError> {
    let raw = value.trim();
    let raw = raw
        .to_ascii_lowercase()
        .strip_prefix("chr")
        .map(str::to_string)
        .unwrap_or_else(|| raw.to_ascii_lowercase());

    if raw.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "--chromosome must be one of: 1-22, X, Y, MT".into(),
        ));
    }

    match raw.as_str() {
        "x" => Ok("X".into()),
        "y" => Ok("Y".into()),
        "mt" => Ok("MT".into()),
        _ => match raw.parse::<u8>() {
            Ok(chr) if (1..=22).contains(&chr) => Ok(chr.to_string()),
            _ => Err(BioMcpError::InvalidArgument(
                "--chromosome must be one of: 1-22, X, Y, MT".into(),
            )),
        },
    }
}

fn normalize_go_id(value: &str) -> Result<String, BioMcpError> {
    let raw = value.trim();
    if !raw.is_ascii() || raw.len() != 10 {
        return Err(BioMcpError::InvalidArgument(
            "--go must be a GO ID in the form GO:0000000".into(),
        ));
    }
    let (prefix, digits) = raw.split_at(3); // safe: all ASCII
    if !prefix.eq_ignore_ascii_case("GO:") || !digits.chars().all(|c| c.is_ascii_digit()) {
        return Err(BioMcpError::InvalidArgument(
            "--go must be a GO ID in the form GO:0000000".into(),
        ));
    }
    Ok(format!("GO:{digits}"))
}

fn parse_region_filter(value: &str) -> Result<(String, i64, i64), BioMcpError> {
    let raw = value.trim();
    let (raw_chr, raw_range) = raw.split_once(':').ok_or_else(|| {
        BioMcpError::InvalidArgument(
            "--region must use format chr:start-end (example: chr7:140424943-140624564)".into(),
        )
    })?;
    let chr = normalize_gene_chromosome(raw_chr)?;
    let (start_raw, end_raw) = raw_range.split_once('-').ok_or_else(|| {
        BioMcpError::InvalidArgument(
            "--region must use format chr:start-end (example: chr7:140424943-140624564)".into(),
        )
    })?;
    let start = start_raw.trim().parse::<i64>().map_err(|_| {
        BioMcpError::InvalidArgument(
            "--region start must be a positive integer (example: chr7:140424943-140624564)".into(),
        )
    })?;
    let end = end_raw.trim().parse::<i64>().map_err(|_| {
        BioMcpError::InvalidArgument(
            "--region end must be a positive integer (example: chr7:140424943-140624564)".into(),
        )
    })?;
    if start <= 0 || end <= 0 || start > end {
        return Err(BioMcpError::InvalidArgument(
            "--region requires positive coordinates with start <= end".into(),
        ));
    }
    Ok((chr, start, end))
}

fn extract_enrich_terms(
    library: &str,
    value: &serde_json::Value,
) -> Result<Vec<EnrichmentTerm>, BioMcpError> {
    let Some(rows) = value.get(library).and_then(|v| v.as_array()) else {
        return Ok(Vec::new());
    };

    let mut out: Vec<EnrichmentTerm> = Vec::new();
    for row in rows.iter().take(5) {
        let Some(row) = row.as_array() else {
            continue;
        };
        let Some(name) = row.get(1).and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(p_value) = row.get(2).and_then(|v| v.as_f64()) else {
            continue;
        };
        let genes = match row.get(5) {
            Some(serde_json::Value::Array(arr)) => arr
                .iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join(","),
            Some(v) => v.as_str().unwrap_or("").to_string(),
            None => String::new(),
        };

        out.push(EnrichmentTerm {
            name: name.to_string(),
            p_value,
            genes,
        });
    }

    Ok(out)
}

async fn enrich_gene(
    symbol: &str,
    include: &[GeneIncludeType],
) -> Result<(Option<Vec<EnrichmentResult>>, Option<Vec<EnrichmentResult>>), BioMcpError> {
    let enrichr = EnrichrClient::new()?;
    let list_id = enrichr.add_list(&[symbol]).await?;

    let mut ontology: Option<Vec<EnrichmentResult>> =
        include.contains(&GeneIncludeType::Ontology).then(Vec::new);
    let mut diseases: Option<Vec<EnrichmentResult>> =
        include.contains(&GeneIncludeType::Diseases).then(Vec::new);

    let mut futs = Vec::new();
    for kind in include {
        for &lib in kind.libraries() {
            let enrichr = enrichr.clone();
            let kind = *kind;
            futs.push(async move {
                let value = enrichr.enrich(list_id, lib).await?;
                let terms = extract_enrich_terms(lib, &value)?;
                Ok::<_, BioMcpError>((
                    kind,
                    EnrichmentResult {
                        library: lib.to_string(),
                        terms,
                    },
                ))
            });
        }
    }

    let results = try_join_all(futs).await?;
    for (kind, result) in results {
        match kind {
            GeneIncludeType::Pathways
            | GeneIncludeType::Protein
            | GeneIncludeType::Go
            | GeneIncludeType::Interactions
            | GeneIncludeType::Civic
            | GeneIncludeType::Expression
            | GeneIncludeType::Hpa
            | GeneIncludeType::Druggability
            | GeneIncludeType::ClinGen
            | GeneIncludeType::Constraint
            | GeneIncludeType::Diagnostics
            | GeneIncludeType::Disgenet
            | GeneIncludeType::Funding => {}
            GeneIncludeType::Ontology => {
                if let Some(v) = ontology.as_mut() {
                    v.push(result);
                }
            }
            GeneIncludeType::Diseases => {
                if let Some(v) = diseases.as_mut() {
                    v.push(result);
                }
            }
        }
    }

    Ok((ontology, diseases))
}

pub fn parse_sections(
    symbol: &str,
    sections: &[String],
) -> Result<Vec<GeneIncludeType>, BioMcpError> {
    let mut include: Vec<GeneIncludeType> = Vec::new();
    let mut include_all = false;
    let symbol = symbol.trim();

    for raw in sections {
        let section = raw.trim().to_ascii_lowercase();
        if section.is_empty() {
            continue;
        }
        if section == "--json" || section == "-j" {
            continue;
        }

        if section == GENE_SECTION_ALL {
            include_all = true;
            continue;
        }

        if section == "variants" {
            return Err(BioMcpError::InvalidArgument(format!(
                "Gene does not have a \"variants\" section. Use: `biomcp search variant -g {symbol}` to find variants for this gene."
            )));
        }

        let kind = GeneIncludeType::from_section(&section).ok_or_else(|| {
            BioMcpError::InvalidArgument(format!(
                "Unknown section \"{section}\" for gene. Available: {}",
                GENE_SECTION_NAMES.join(", ")
            ))
        })?;
        if !include.contains(&kind) {
            include.push(kind);
        }
    }

    if include_all {
        include = vec![
            GeneIncludeType::Pathways,
            GeneIncludeType::Ontology,
            GeneIncludeType::Diseases,
            GeneIncludeType::Protein,
            GeneIncludeType::Go,
            GeneIncludeType::Interactions,
            GeneIncludeType::Civic,
            GeneIncludeType::Expression,
            GeneIncludeType::Hpa,
            GeneIncludeType::Druggability,
            GeneIncludeType::ClinGen,
            GeneIncludeType::Constraint,
        ];
    }

    Ok(include)
}

async fn resolve_uniprot_accession(
    explicit: Option<&str>,
    symbol: &str,
) -> Result<Option<String>, BioMcpError> {
    if let Some(value) = explicit
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
    {
        return Ok(Some(value));
    }

    let page = UniProtClient::new()?.search(symbol, 1, 0, None).await?;
    Ok(page
        .results
        .into_iter()
        .next()
        .map(|r| r.primary_accession)
        .filter(|v| !v.trim().is_empty()))
}

async fn fetch_protein_section(
    uniprot_id: Option<&str>,
    symbol: &str,
) -> Result<Option<GeneProtein>, BioMcpError> {
    let accession = resolve_uniprot_accession(uniprot_id, symbol).await?;
    let Some(accession) = accession else {
        return Ok(None);
    };

    let record = UniProtClient::new()?.get_record(&accession).await?;
    let accession = record.primary_accession.clone();
    let length = record
        .sequence
        .as_ref()
        .and_then(|sequence| sequence.length);
    let isoforms = record
        .protein_isoforms()
        .into_iter()
        .map(|isoform| GeneProteinIsoform {
            name: isoform.name,
            length: isoform.is_displayed.then_some(length).flatten(),
        })
        .collect();
    let alternative_names = record.alternative_protein_names();
    Ok(Some(GeneProtein {
        accession,
        name: record.display_name(),
        function: record.function_summary(),
        length,
        isoforms,
        alternative_names,
    }))
}

async fn fetch_go_section(
    uniprot_id: Option<&str>,
    symbol: &str,
) -> Result<Vec<GeneGoTerm>, BioMcpError> {
    let accession = resolve_uniprot_accession(uniprot_id, symbol).await?;
    let Some(accession) = accession else {
        return Ok(Vec::new());
    };

    let quickgo = QuickGoClient::new()?;
    let rows = quickgo.annotations(&accession, 20).await?;
    let go_ids_missing_names = rows
        .iter()
        .filter_map(|row| {
            let id = row.go_id.as_deref()?.trim();
            if id.is_empty() {
                return None;
            }
            let has_name = row
                .go_name
                .as_deref()
                .map(str::trim)
                .is_some_and(|v| !v.is_empty());
            (!has_name).then(|| id.to_string())
        })
        .collect::<Vec<_>>();

    let mut term_map: HashMap<String, (String, Option<String>)> = HashMap::new();
    if !go_ids_missing_names.is_empty() {
        match quickgo.terms(&go_ids_missing_names).await {
            Ok(terms) => {
                for term in terms {
                    let Some(id) = term.id.as_deref().map(str::trim).filter(|v| !v.is_empty())
                    else {
                        continue;
                    };
                    let Some(name) = term
                        .name
                        .as_deref()
                        .map(str::trim)
                        .filter(|v| !v.is_empty())
                    else {
                        continue;
                    };
                    let aspect = term
                        .aspect
                        .as_deref()
                        .map(str::trim)
                        .filter(|v| !v.is_empty())
                        .map(str::to_string);
                    term_map.insert(id.to_string(), (name.to_string(), aspect));
                }
            }
            Err(err) => warn!("QuickGO term lookup unavailable: {err}"),
        }
    }

    let mut out = Vec::new();
    for row in rows {
        let Some(id) = row
            .go_id
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string)
        else {
            continue;
        };
        if out.iter().any(|v: &GeneGoTerm| v.id == id) {
            continue;
        }

        let name = row
            .go_name
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string)
            .or_else(|| term_map.get(&id).map(|(name, _)| name.clone()))
            .unwrap_or_else(|| id.clone());

        let aspect = row
            .go_aspect
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string)
            .or_else(|| term_map.get(&id).and_then(|(_, aspect)| aspect.clone()));

        out.push(GeneGoTerm {
            id,
            name,
            aspect,
            evidence: row
                .evidence_code
                .as_deref()
                .map(str::trim)
                .map(str::to_string)
                .filter(|v| !v.is_empty()),
        });
    }
    Ok(out)
}

fn apply_go_section_result(gene: &mut Gene, result: Result<Vec<GeneGoTerm>, BioMcpError>) {
    match result {
        Ok(rows) => {
            let outcome = if rows.is_empty() {
                SectionOutcome::empty("QuickGO")
            } else {
                SectionOutcome::data("QuickGO")
            };
            gene.go = Some(rows);
            gene.section_outcomes.complete(GENE_SECTION_GO, outcome);
        }
        Err(_) => {
            gene.go = Some(Vec::new());
            gene.section_outcomes.complete(
                GENE_SECTION_GO,
                SectionOutcome::unavailable("QuickGO gene ontology is unavailable."),
            );
        }
    }
}

async fn fetch_interactions_section(symbol: &str) -> Result<Vec<GeneInteraction>, BioMcpError> {
    let rows = StringClient::new()?.interactions(symbol, 9606, 15).await?;
    let mut out = Vec::new();
    for row in rows {
        let a = row.preferred_name_a.unwrap_or_default();
        let b = row.preferred_name_b.unwrap_or_default();
        let partner = if a.eq_ignore_ascii_case(symbol) { b } else { a };
        let partner = partner.trim().to_string();
        if partner.is_empty() {
            continue;
        }
        if out
            .iter()
            .any(|v: &GeneInteraction| v.partner.eq_ignore_ascii_case(&partner))
        {
            continue;
        }
        out.push(GeneInteraction {
            partner,
            score: row.score,
        });
    }
    out.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.partner.cmp(&b.partner))
    });
    Ok(out)
}

fn apply_gene_interactions_result(
    gene: &mut Gene,
    result: Result<Vec<GeneInteraction>, BioMcpError>,
) {
    match result {
        Ok(rows) => {
            let outcome = if rows.is_empty() {
                SectionOutcome::empty("STRING")
            } else {
                SectionOutcome::data("STRING")
            };
            gene.interactions = Some(rows);
            gene.section_outcomes
                .complete(GENE_SECTION_INTERACTIONS, outcome);
        }
        Err(_) => {
            gene.interactions = Some(Vec::new());
            gene.section_outcomes.complete(
                GENE_SECTION_INTERACTIONS,
                SectionOutcome::unavailable("STRING gene interactions are unavailable."),
            );
        }
    }
}

async fn fetch_pathways_section(symbol: &str) -> Result<Option<Vec<GenePathway>>, BioMcpError> {
    let symbol = symbol.trim();
    if symbol.is_empty() {
        return Ok(None);
    }

    let (rows, _) = ReactomeClient::new()?.search_pathways(symbol, 12).await?;
    let mut out: Vec<GenePathway> = Vec::new();
    for row in rows {
        let id = row.id.trim().to_string();
        let name = row.name.trim().to_string();
        if id.is_empty() || name.is_empty() {
            continue;
        }
        if out.iter().any(|p| p.id.eq_ignore_ascii_case(&id)) {
            continue;
        }
        out.push(GenePathway {
            source: "Reactome".to_string(),
            id,
            name,
        });
    }

    if out.is_empty() {
        Ok(None)
    } else {
        Ok(Some(out))
    }
}

fn pathway_outcome(pathways: Option<&[GenePathway]>, reactome_available: bool) -> SectionOutcome {
    let mut sources = pathways
        .unwrap_or_default()
        .iter()
        .map(|row| row.source.trim())
        .filter(|source| !source.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    if reactome_available
        && !sources
            .iter()
            .any(|source| source.eq_ignore_ascii_case("Reactome"))
    {
        sources.push("Reactome".to_string());
    }
    sources.sort_by_key(|source| source.to_ascii_lowercase());
    sources.dedup_by(|left, right| left.eq_ignore_ascii_case(right));

    if !reactome_available {
        return if sources.is_empty() {
            SectionOutcome::unavailable("Reactome gene pathways are unavailable.")
        } else {
            SectionOutcome::degraded(
                sources,
                "Reactome pathways are unavailable; retained pathway evidence may be incomplete.",
            )
        };
    }
    if pathways.is_some_and(|rows| !rows.is_empty()) {
        SectionOutcome::data_sources(sources)
    } else {
        SectionOutcome::empty("Reactome")
    }
}

fn merge_pathways(
    existing: Option<Vec<GenePathway>>,
    additional: Option<Vec<GenePathway>>,
) -> Option<Vec<GenePathway>> {
    let mut out = Vec::new();
    let mut push_rows = |rows: Vec<GenePathway>| {
        for row in rows {
            let source = row.source.trim().to_string();
            let id = row.id.trim().to_string();
            let name = row.name.trim().to_string();
            if source.is_empty() || id.is_empty() || name.is_empty() {
                continue;
            }
            if out.iter().any(|existing: &GenePathway| {
                existing.source.eq_ignore_ascii_case(&source)
                    && existing.id.eq_ignore_ascii_case(&id)
            }) {
                continue;
            }
            out.push(GenePathway { source, id, name });
        }
    };

    if let Some(rows) = existing {
        push_rows(rows);
    }
    if let Some(rows) = additional {
        push_rows(rows);
    }

    (!out.is_empty()).then_some(out)
}

async fn fetch_clinical_context(
    symbol: &str,
    target_id: Option<&str>,
) -> Result<OpenTargetsTargetClinicalContext, BioMcpError> {
    let symbol = symbol.trim();
    if symbol.is_empty() {
        return Ok(OpenTargetsTargetClinicalContext::default());
    }

    let client = OpenTargetsClient::new()?;
    if let Some(target_id) = target_id {
        Ok(client
            .target_clinical_context_for_target_id(target_id, 5)
            .await?)
    } else {
        Ok(client.target_clinical_context(symbol, 5).await?)
    }
}

async fn add_clinical_context(gene: &mut Gene, target_id: Option<&str>) -> Result<(), BioMcpError> {
    let context = fetch_clinical_context(&gene.symbol, target_id).await?;
    gene.clinical_diseases = context.diseases;
    gene.clinical_drugs = context.drugs;
    Ok(())
}

async fn fetch_civic_section(symbol: &str, timeout: Duration) -> (CivicContext, SectionOutcome) {
    let symbol = symbol.trim();
    if symbol.is_empty() {
        return (CivicContext::default(), SectionOutcome::empty("CIViC"));
    }

    let civic_fut = async {
        let client = CivicClient::new()?;
        client.by_molecular_profile(symbol, 10).await
    };

    match tokio::time::timeout(timeout, civic_fut).await {
        Ok(Ok(context)) => {
            let outcome = if context.evidence_items.is_empty() && context.assertions.is_empty() {
                SectionOutcome::empty("CIViC")
            } else {
                SectionOutcome::data("CIViC")
            };
            (context, outcome)
        }
        Ok(Err(err)) => {
            warn!(symbol = %symbol, "CIViC unavailable for gene section: {err}");
            (
                CivicContext::default(),
                SectionOutcome::unavailable("CIViC gene evidence is unavailable."),
            )
        }
        Err(_) => {
            warn!(
                symbol = %symbol,
                timeout_secs = timeout.as_secs(),
                "CIViC gene section timed out"
            );
            (
                CivicContext::default(),
                SectionOutcome::unavailable("CIViC gene evidence is unavailable."),
            )
        }
    }
}

async fn add_civic_section(gene: &mut Gene, timeout: Duration) {
    let (context, outcome) = fetch_civic_section(&gene.symbol, timeout).await;
    gene.civic = Some(context);
    gene.section_outcomes.complete(GENE_SECTION_CIVIC, outcome);
}

async fn fetch_expression_section(
    ensembl_id: Option<&str>,
    symbol: &str,
    timeout: Duration,
) -> (GeneExpression, SectionOutcome) {
    let Some(ensembl_id) = ensembl_id.map(str::trim).filter(|v| !v.is_empty()) else {
        return (GeneExpression::default(), SectionOutcome::empty("GTEx"));
    };

    let expression_fut = async {
        let client = GtexClient::new()?;
        let tissues = client.median_gene_expression(ensembl_id).await?;
        Ok::<_, BioMcpError>(GeneExpression { tissues })
    };

    match tokio::time::timeout(timeout, expression_fut).await {
        Ok(Ok(expression)) => {
            let outcome = if expression.tissues.is_empty() {
                SectionOutcome::empty("GTEx")
            } else {
                SectionOutcome::data("GTEx")
            };
            (expression, outcome)
        }
        Ok(Err(err)) => {
            warn!(
                symbol = %symbol,
                ensembl_id = %ensembl_id,
                "GTEx unavailable for gene expression section: {err}"
            );
            (
                GeneExpression::default(),
                SectionOutcome::unavailable("GTEx gene expression is unavailable."),
            )
        }
        Err(_) => {
            warn!(
                symbol = %symbol,
                ensembl_id = %ensembl_id,
                timeout_secs = timeout.as_secs(),
                "GTEx expression section timed out"
            );
            (
                GeneExpression::default(),
                SectionOutcome::unavailable("GTEx gene expression is unavailable."),
            )
        }
    }
}

async fn add_expression_section(gene: &mut Gene, timeout: Duration) {
    let (expression, outcome) =
        fetch_expression_section(gene.ensembl_id.as_deref(), &gene.symbol, timeout).await;
    gene.expression = Some(expression);
    gene.section_outcomes
        .complete(GENE_SECTION_EXPRESSION, outcome);
}

async fn fetch_hpa_section(
    ensembl_id: Option<&str>,
    symbol: &str,
    timeout: Duration,
) -> (GeneHpa, SectionOutcome) {
    let Some(ensembl_id) = ensembl_id.map(str::trim).filter(|v| !v.is_empty()) else {
        return (
            GeneHpa::default(),
            SectionOutcome::empty("Human Protein Atlas"),
        );
    };

    let hpa_fut = async {
        let client = HpaClient::new()?;
        client.protein_data(ensembl_id).await
    };

    match tokio::time::timeout(timeout, hpa_fut).await {
        Ok(Ok(hpa)) => {
            let has_data = !hpa.tissues.is_empty()
                || !hpa.subcellular_main_location.is_empty()
                || !hpa.subcellular_additional_location.is_empty()
                || hpa.reliability.is_some()
                || hpa.protein_summary.is_some()
                || hpa.rna_summary.is_some();
            let outcome = if has_data {
                SectionOutcome::data("Human Protein Atlas")
            } else {
                SectionOutcome::empty("Human Protein Atlas")
            };
            (hpa, outcome)
        }
        Ok(Err(err)) => {
            warn!(
                symbol = %symbol,
                ensembl_id = %ensembl_id,
                "HPA unavailable for gene section: {err}"
            );
            (
                GeneHpa::default(),
                SectionOutcome::unavailable("Human Protein Atlas data is unavailable."),
            )
        }
        Err(_) => {
            warn!(
                symbol = %symbol,
                ensembl_id = %ensembl_id,
                timeout_secs = timeout.as_secs(),
                "HPA gene section timed out"
            );
            (
                GeneHpa::default(),
                SectionOutcome::unavailable("Human Protein Atlas data is unavailable."),
            )
        }
    }
}

async fn add_hpa_section(gene: &mut Gene, timeout: Duration) {
    let (hpa, outcome) = fetch_hpa_section(gene.ensembl_id.as_deref(), &gene.symbol, timeout).await;
    gene.hpa = Some(hpa);
    gene.section_outcomes.complete(GENE_SECTION_HPA, outcome);
}

async fn fetch_druggability_section(
    symbol: &str,
    target_id: Option<&str>,
    timeout: Duration,
) -> (GeneDruggability, SectionOutcome) {
    let symbol = symbol.trim();
    if symbol.is_empty() {
        return (
            GeneDruggability::default(),
            SectionOutcome::empty_sources(["DGIdb", "Open Targets"]),
        );
    }
    let target_id = target_id.map(str::to_string);

    let dgidb_fut = tokio::time::timeout(timeout, async {
        let client = DgidbClient::new()?;
        client.gene_interactions(symbol).await
    });
    let opentargets_fut = tokio::time::timeout(timeout, async {
        let client = OpenTargetsClient::new()?;
        if let Some(target_id) = target_id.as_deref() {
            client
                .target_druggability_context_for_target_id(target_id)
                .await
        } else {
            client.target_druggability_context(symbol).await
        }
    });

    let (dgidb_result, opentargets_result) = tokio::join!(dgidb_fut, opentargets_fut);

    let dgidb_result = match dgidb_result {
        Ok(Ok(druggability)) => Ok(druggability),
        Ok(Err(err)) => {
            warn!(
                symbol = %symbol,
                "DGIdb unavailable for gene druggability section: {err}"
            );
            Err(err)
        }
        Err(_) => {
            warn!(
                symbol = %symbol,
                timeout_secs = timeout.as_secs(),
                "DGIdb gene section timed out"
            );
            Err(BioMcpError::Api {
                api: "dgidb".to_string(),
                message: "timed out".to_string(),
            })
        }
    };

    let opentargets_result = match opentargets_result {
        Ok(Ok(context)) => Ok(context),
        Ok(Err(err)) => {
            warn!(
                symbol = %symbol,
                "OpenTargets unavailable for gene druggability section: {err}"
            );
            Err(err)
        }
        Err(_) => {
            warn!(
                symbol = %symbol,
                timeout_secs = timeout.as_secs(),
                "OpenTargets gene druggability section timed out"
            );
            Err(BioMcpError::Api {
                api: "opentargets".to_string(),
                message: "timed out".to_string(),
            })
        }
    };

    let successful_sources = [
        dgidb_result.is_ok().then_some("DGIdb"),
        opentargets_result.is_ok().then_some("Open Targets"),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    let failed = 2 - successful_sources.len();
    let merged = merge_druggability_results(dgidb_result, opentargets_result);
    let has_data = !merged.categories.is_empty()
        || !merged.interactions.is_empty()
        || !merged.tractability.is_empty()
        || !merged.safety_liabilities.is_empty();
    let outcome = match (successful_sources.is_empty(), failed, has_data) {
        (true, _, _) => SectionOutcome::unavailable("Gene druggability is unavailable."),
        (false, 0, true) => SectionOutcome::data_sources(successful_sources),
        (false, 0, false) => SectionOutcome::empty_sources(successful_sources),
        (false, _, _) => SectionOutcome::degraded(
            successful_sources,
            "Gene druggability is incomplete because one provider is unavailable.",
        ),
    };
    (merged, outcome)
}

async fn add_druggability_section(gene: &mut Gene, target_id: Option<&str>, timeout: Duration) {
    let (section, outcome) = fetch_druggability_section(&gene.symbol, target_id, timeout).await;
    gene.druggability = Some(section);
    gene.section_outcomes
        .complete(GENE_SECTION_DRUGGABILITY, outcome);
}

fn merge_druggability_results(
    dgidb_result: Result<GeneDruggability, BioMcpError>,
    opentargets_result: Result<OpenTargetsTargetDruggabilityContext, BioMcpError>,
) -> GeneDruggability {
    let mut merged = GeneDruggability::default();

    if let Ok(dgidb) = dgidb_result {
        merged.categories = dgidb.categories;
        merged.interactions = dgidb.interactions;
    }

    if let Ok(context) = opentargets_result {
        merged.tractability = context
            .tractability
            .into_iter()
            .map(|row| GeneTractabilityModality {
                modality: row.modality,
                tractable: row.tractable,
                evidence_labels: row.evidence_labels,
            })
            .collect();
        merged.safety_liabilities = context
            .safety_liabilities
            .into_iter()
            .map(|row| GeneSafetyLiability {
                event: row.event,
                datasource: row.datasource,
                effect_direction: row.effect_direction,
                biosample: row.biosample,
            })
            .collect();
    }

    merged
}

async fn fetch_clingen_section(symbol: &str, timeout: Duration) -> (GeneClinGen, SectionOutcome) {
    let symbol = symbol.trim();
    if symbol.is_empty() {
        return (GeneClinGen::default(), SectionOutcome::empty("ClinGen"));
    }

    let clingen_fut = async {
        let client = ClinGenClient::new()?;
        client.gene_context(symbol).await
    };

    match tokio::time::timeout(timeout, clingen_fut).await {
        Ok(Ok(clingen)) => {
            let outcome = if clingen.validity.is_empty()
                && clingen.haploinsufficiency.is_none()
                && clingen.triplosensitivity.is_none()
            {
                SectionOutcome::empty("ClinGen")
            } else {
                SectionOutcome::data("ClinGen")
            };
            (clingen, outcome)
        }
        Ok(Err(err)) => {
            warn!(
                symbol = %symbol,
                "ClinGen unavailable for gene clingen section: {err}"
            );
            (
                GeneClinGen::default(),
                SectionOutcome::unavailable("ClinGen gene evidence is unavailable."),
            )
        }
        Err(_) => {
            warn!(
                symbol = %symbol,
                timeout_secs = timeout.as_secs(),
                "ClinGen gene section timed out"
            );
            (
                GeneClinGen::default(),
                SectionOutcome::unavailable("ClinGen gene evidence is unavailable."),
            )
        }
    }
}

async fn add_clingen_section(gene: &mut Gene, timeout: Duration) {
    let (clingen, outcome) = fetch_clingen_section(&gene.symbol, timeout).await;
    gene.clingen = Some(clingen);
    gene.section_outcomes
        .complete(GENE_SECTION_CLINGEN, outcome);
}

fn gnomad_constraint_section(
    transcript: Option<String>,
    pli: Option<f64>,
    loeuf: Option<f64>,
    mis_z: Option<f64>,
    syn_z: Option<f64>,
) -> GeneConstraint {
    GeneConstraint {
        pli,
        loeuf,
        mis_z,
        syn_z,
        transcript,
        source: "gnomAD".to_string(),
        source_version: GNOMAD_CONSTRAINT_VERSION.to_string(),
        reference_genome: GNOMAD_CONSTRAINT_REFERENCE_GENOME.to_string(),
    }
}

fn gnomad_constraint_outcome(constraint: &GeneConstraint) -> SectionOutcome {
    if constraint.transcript.is_some()
        || constraint.pli.is_some()
        || constraint.loeuf.is_some()
        || constraint.mis_z.is_some()
        || constraint.syn_z.is_some()
    {
        SectionOutcome::data("gnomAD")
    } else {
        SectionOutcome::empty("gnomAD")
    }
}

async fn fetch_constraint_section(
    symbol: &str,
    timeout: Duration,
) -> (GeneConstraint, SectionOutcome) {
    let symbol = symbol.trim();
    if symbol.is_empty() {
        return (
            gnomad_constraint_section(None, None, None, None, None),
            SectionOutcome::empty("gnomAD"),
        );
    }

    let constraint_fut = async {
        let client = GnomadClient::new()?;
        client.gene_constraint(symbol).await
    };

    match tokio::time::timeout(timeout, constraint_fut).await {
        Ok(Ok(Some(constraint))) => {
            let section = gnomad_constraint_section(
                constraint.transcript,
                constraint.pli,
                constraint.loeuf,
                constraint.mis_z,
                constraint.syn_z,
            );
            let outcome = gnomad_constraint_outcome(&section);
            (section, outcome)
        }
        Ok(Ok(None)) => (
            gnomad_constraint_section(None, None, None, None, None),
            SectionOutcome::empty("gnomAD"),
        ),
        Ok(Err(err)) => {
            warn!(
                symbol = %symbol,
                "gnomAD unavailable for gene constraint section: {err}"
            );
            (
                gnomad_constraint_section(None, None, None, None, None),
                SectionOutcome::unavailable("gnomAD gene constraint is unavailable."),
            )
        }
        Err(_) => {
            warn!(
                symbol = %symbol,
                timeout_secs = timeout.as_secs(),
                "gnomAD gene constraint section timed out"
            );
            (
                gnomad_constraint_section(None, None, None, None, None),
                SectionOutcome::unavailable("gnomAD gene constraint is unavailable."),
            )
        }
    }
}

async fn add_constraint_section(gene: &mut Gene, timeout: Duration) {
    let (constraint, outcome) = fetch_constraint_section(&gene.symbol, timeout).await;
    gene.constraint = Some(constraint);
    gene.section_outcomes
        .complete(GENE_SECTION_CONSTRAINT, outcome);
}

fn map_disgenet_gene_association(row: DisgenetAssociationRecord) -> GeneDisgenetAssociation {
    GeneDisgenetAssociation {
        disease_name: row.disease_name,
        disease_cui: row.disease_umls_cui,
        score: row.score,
        publication_count: row.publication_count,
        clinical_trial_count: row.clinical_trial_count,
        evidence_index: row.evidence_index,
        evidence_level: row.evidence_level,
    }
}

async fn add_disgenet_section(gene: &mut Gene) -> Result<(), BioMcpError> {
    let client = DisgenetClient::new()?;
    let associations = client
        .fetch_gene_associations(gene, 10)
        .await?
        .into_iter()
        .map(map_disgenet_gene_association)
        .collect();
    gene.disgenet = Some(GeneDisgenet { associations });
    Ok(())
}

async fn add_funding_section(gene: &mut Gene, timeout: Duration) {
    let symbol = gene.symbol.trim();
    if symbol.is_empty() {
        gene.funding = Some(NihReporterFundingSection {
            query: String::new(),
            fiscal_years: Vec::new(),
            matching_project_years: 0,
            grants: Vec::new(),
        });
        gene.funding_note = Some(FUNDING_NO_DATA_NOTE.into());
        return;
    }

    let funding_fut = async {
        let client = NihReporterClient::new()?;
        client.funding(symbol).await
    };

    match tokio::time::timeout(timeout, funding_fut).await {
        Ok(Ok(section)) => {
            let no_hits = section.matching_project_years == 0 && section.grants.is_empty();
            gene.funding = Some(section);
            gene.funding_note = if no_hits {
                Some(FUNDING_NO_DATA_NOTE.into())
            } else {
                None
            };
        }
        Ok(Err(err)) => {
            warn!(symbol = %gene.symbol, "NIH Reporter unavailable for gene funding section: {err}");
            gene.funding = None;
            gene.funding_note = Some(FUNDING_UNAVAILABLE_NOTE.into());
        }
        Err(_) => {
            warn!(
                symbol = %gene.symbol,
                timeout_secs = timeout.as_secs(),
                "NIH Reporter gene funding section timed out"
            );
            gene.funding = None;
            gene.funding_note = Some(FUNDING_UNAVAILABLE_NOTE.into());
        }
    }
}

async fn add_diagnostics_section(gene: &mut Gene) {
    let query = gene.symbol.trim().to_string();
    if query.is_empty() {
        gene.diagnostics = Some(Vec::new());
        gene.diagnostics_note = None;
        return;
    }

    let filters = DiagnosticSearchFilters {
        gene: Some(query.clone()),
        ..Default::default()
    };

    match crate::entities::diagnostic::search_page(&filters, DIAGNOSTIC_PIVOT_LIMIT, 0).await {
        Ok(page) => {
            apply_diagnostics_section_result(gene, &query, Ok(page.results));
        }
        Err(err) => {
            apply_diagnostics_section_result(gene, &query, Err(err));
        }
    }
}

fn apply_diagnostics_section_result(
    gene: &mut Gene,
    query: &str,
    result: Result<Vec<DiagnosticSearchResult>, BioMcpError>,
) {
    match result {
        Ok(rows) => {
            gene.diagnostics = Some(rows);
            gene.diagnostics_note = None;
        }
        Err(err) => {
            warn!(gene = %query, "Diagnostic local data unavailable for gene diagnostics section: {err}");
            gene.diagnostics = None;
            gene.diagnostics_note = Some(GENE_DIAGNOSTICS_UNAVAILABLE_NOTE.into());
        }
    }
}

async fn populate_sections_parallel_top(
    gene: &mut Gene,
    include: &[GeneIncludeType],
    timing: &mut GeneTimingCollector,
    opentargets_id: Option<&str>,
    optional_timeout: Duration,
    prefetched_clingen: Option<
        tokio::task::JoinHandle<((GeneClinGen, SectionOutcome), GeneTimingEntry)>,
    >,
) -> Result<(), BioMcpError> {
    let symbol = gene.symbol.clone();
    let ensembl_id = gene.ensembl_id.clone();
    let uniprot_id = gene.uniprot_id.clone();
    let enrichr_sections: Vec<GeneIncludeType> = include
        .iter()
        .copied()
        .filter(|value| matches!(value, GeneIncludeType::Ontology | GeneIncludeType::Diseases))
        .collect();

    let clinical_target_id = opentargets_id.map(str::to_string);
    let druggability_target_id = clinical_target_id.clone();

    let clinical_context_fut = timed_section(
        "clinical_context",
        fetch_clinical_context(&symbol, clinical_target_id.as_deref()),
        |result| match result {
            Ok(context) if !context.diseases.is_empty() || !context.drugs.is_empty() => {
                "data".to_string()
            }
            Ok(_) => "empty".to_string(),
            Err(_) => "error".to_string(),
        },
    );

    let enrichr_fut = async {
        if enrichr_sections.is_empty() {
            None
        } else {
            Some(
                timed_section(
                    "enrichr",
                    enrich_gene(&symbol, &enrichr_sections),
                    |result| match result {
                        Ok((ontology, diseases))
                            if ontology.as_ref().is_some_and(|rows| {
                                rows.iter().any(|row| !row.terms.is_empty())
                            }) || diseases.as_ref().is_some_and(|rows| {
                                rows.iter().any(|row| !row.terms.is_empty())
                            }) =>
                        {
                            "data".to_string()
                        }
                        Ok(_) => "empty".to_string(),
                        Err(_) => "error".to_string(),
                    },
                )
                .await,
            )
        }
    };

    let expression_fut = async {
        if !include.contains(&GeneIncludeType::Expression) {
            None
        } else {
            Some(
                timed_section(
                    "expression",
                    fetch_expression_section(ensembl_id.as_deref(), &symbol, optional_timeout),
                    |(_, outcome)| outcome.outcome().as_str().to_string(),
                )
                .await,
            )
        }
    };

    let hpa_fut = async {
        if !include.contains(&GeneIncludeType::Hpa) {
            None
        } else {
            Some(
                timed_section(
                    "hpa",
                    fetch_hpa_section(ensembl_id.as_deref(), &symbol, optional_timeout),
                    |(_, outcome)| outcome.outcome().as_str().to_string(),
                )
                .await,
            )
        }
    };

    let druggability_fut = async {
        if !include.contains(&GeneIncludeType::Druggability) {
            None
        } else {
            Some(
                timed_section(
                    "druggability",
                    fetch_druggability_section(
                        &symbol,
                        druggability_target_id.as_deref(),
                        optional_timeout,
                    ),
                    |(_, outcome)| outcome.outcome().as_str().to_string(),
                )
                .await,
            )
        }
    };

    let clingen_fut = async {
        if !include.contains(&GeneIncludeType::ClinGen) {
            None
        } else if let Some(prefetched) = prefetched_clingen {
            match prefetched.await {
                Ok(value) => Some(value),
                Err(err) => {
                    warn!("ClinGen prefetch task failed: {err}");
                    Some((
                        (
                            GeneClinGen::default(),
                            SectionOutcome::unavailable("ClinGen gene evidence is unavailable."),
                        ),
                        GeneTimingEntry {
                            section: "clingen".to_string(),
                            elapsed_ms: 0,
                            outcome: SectionOutcomeState::Unavailable,
                        },
                    ))
                }
            }
        } else {
            Some(
                timed_section(
                    "clingen",
                    fetch_clingen_section(&symbol, optional_timeout),
                    classify_clingen_section,
                )
                .await,
            )
        }
    };

    let pathways_fut = async {
        if !include.contains(&GeneIncludeType::Pathways) {
            None
        } else {
            Some(
                timed_section(
                    "pathways",
                    fetch_pathways_section(&symbol),
                    |result| match result {
                        Ok(Some(rows)) if !rows.is_empty() => "data".to_string(),
                        Ok(_) => "empty".to_string(),
                        Err(_) => "error".to_string(),
                    },
                )
                .await,
            )
        }
    };

    let protein_fut = async {
        if !include.contains(&GeneIncludeType::Protein) {
            None
        } else {
            Some(
                timed_section(
                    "protein",
                    fetch_protein_section(uniprot_id.as_deref(), &symbol),
                    |result| match result {
                        Ok(Some(_)) => "data".to_string(),
                        Ok(None) => "empty".to_string(),
                        Err(_) => "error".to_string(),
                    },
                )
                .await,
            )
        }
    };

    let go_fut = async {
        if !include.contains(&GeneIncludeType::Go) {
            None
        } else {
            Some(
                timed_section(
                    "go",
                    fetch_go_section(uniprot_id.as_deref(), &symbol),
                    |result| match result {
                        Ok(rows) if !rows.is_empty() => "data".to_string(),
                        Ok(_) => "empty".to_string(),
                        Err(_) => "error".to_string(),
                    },
                )
                .await,
            )
        }
    };

    let interactions_fut = async {
        if !include.contains(&GeneIncludeType::Interactions) {
            None
        } else {
            Some(
                timed_section(
                    "interactions",
                    fetch_interactions_section(&symbol),
                    |result| match result {
                        Ok(rows) if !rows.is_empty() => "data".to_string(),
                        Ok(_) => "empty".to_string(),
                        Err(_) => "error".to_string(),
                    },
                )
                .await,
            )
        }
    };

    let civic_fut = async {
        if !include.contains(&GeneIncludeType::Civic) {
            None
        } else {
            Some(
                timed_section(
                    "civic",
                    fetch_civic_section(&symbol, optional_timeout),
                    |(_, outcome)| outcome.outcome().as_str().to_string(),
                )
                .await,
            )
        }
    };

    let constraint_fut = async {
        if !include.contains(&GeneIncludeType::Constraint) {
            None
        } else {
            Some(
                timed_section(
                    "constraint",
                    fetch_constraint_section(&symbol, optional_timeout),
                    |(_, outcome)| outcome.outcome().as_str().to_string(),
                )
                .await,
            )
        }
    };

    let (
        (clinical_context_result, clinical_context_entry),
        enrichr_result,
        expression_result,
        hpa_result,
        druggability_result,
        clingen_result,
        pathways_result,
        protein_result,
        go_result,
        interactions_result,
        civic_result,
        constraint_result,
    ) = tokio::join!(
        Box::pin(clinical_context_fut),
        Box::pin(enrichr_fut),
        Box::pin(expression_fut),
        Box::pin(hpa_fut),
        Box::pin(druggability_fut),
        Box::pin(clingen_fut),
        Box::pin(pathways_fut),
        Box::pin(protein_fut),
        Box::pin(go_fut),
        Box::pin(interactions_fut),
        Box::pin(civic_fut),
        Box::pin(constraint_fut),
    );

    timing.push(clinical_context_entry);
    match clinical_context_result {
        Ok(context) => {
            gene.clinical_diseases = context.diseases;
            gene.clinical_drugs = context.drugs;
        }
        Err(err) => warn!("OpenTargets unavailable for gene clinical context: {err}"),
    }

    if let Some((result, entry)) = enrichr_result {
        timing.push(entry);
        let (ontology, diseases) = match result {
            Ok(value) => value,
            Err(_) => {
                for section in &enrichr_sections {
                    gene.section_outcomes.complete(
                        section.as_str(),
                        SectionOutcome::unavailable("Enrichr gene enrichment is unavailable."),
                    );
                }
                (None, None)
            }
        };
        gene.ontology = ontology;
        gene.diseases = diseases;
    }

    if let Some(((expression, outcome), entry)) = expression_result {
        timing.push(entry);
        gene.expression = Some(expression);
        gene.section_outcomes
            .complete(GENE_SECTION_EXPRESSION, outcome);
    }

    if let Some(((hpa, outcome), entry)) = hpa_result {
        timing.push(entry);
        gene.hpa = Some(hpa);
        gene.section_outcomes.complete(GENE_SECTION_HPA, outcome);
    }

    if let Some(((druggability, outcome), entry)) = druggability_result {
        timing.push(entry);
        gene.druggability = Some(druggability);
        gene.section_outcomes
            .complete(GENE_SECTION_DRUGGABILITY, outcome);
    }

    if let Some(((clingen, outcome), entry)) = clingen_result {
        timing.push(entry);
        gene.clingen = Some(clingen);
        gene.section_outcomes
            .complete(GENE_SECTION_CLINGEN, outcome);
    }

    if let Some((result, entry)) = pathways_result {
        timing.push(entry);
        gene.pathways = match result {
            Ok(value) => {
                let pathways = merge_pathways(gene.pathways.take(), value);
                gene.section_outcomes.complete(
                    GENE_SECTION_PATHWAYS,
                    pathway_outcome(pathways.as_deref(), true),
                );
                pathways
            }
            Err(_) => {
                gene.section_outcomes.complete(
                    GENE_SECTION_PATHWAYS,
                    pathway_outcome(gene.pathways.as_deref(), false),
                );
                gene.pathways.clone()
            }
        };
    } else {
        gene.pathways = None;
    }

    if let Some((result, entry)) = protein_result {
        timing.push(entry);
        gene.protein = match result {
            Ok(value) => value,
            Err(_) => {
                gene.section_outcomes.complete(
                    GENE_SECTION_PROTEIN,
                    SectionOutcome::unavailable("UniProt gene protein data is unavailable."),
                );
                None
            }
        };
    }

    if let Some((result, entry)) = go_result {
        timing.push(entry);
        apply_go_section_result(gene, result);
    }

    if let Some((result, entry)) = interactions_result {
        timing.push(entry);
        apply_gene_interactions_result(gene, result);
    }

    if let Some(((civic, outcome), entry)) = civic_result {
        timing.push(entry);
        gene.civic = Some(civic);
        gene.section_outcomes.complete(GENE_SECTION_CIVIC, outcome);
    }

    if let Some(((constraint, outcome), entry)) = constraint_result {
        timing.push(entry);
        gene.constraint = Some(constraint);
        gene.section_outcomes
            .complete(GENE_SECTION_CONSTRAINT, outcome);
    }

    if include.contains(&GeneIncludeType::Disgenet) {
        let started = Instant::now();
        let timing_outcome = if add_disgenet_section(gene).await.is_err() {
            gene.section_outcomes.complete(
                GENE_SECTION_DISGENET,
                SectionOutcome::unavailable("DisGeNET gene associations are unavailable."),
            );
            "error"
        } else if gene
            .disgenet
            .as_ref()
            .is_some_and(|section| !section.associations.is_empty())
        {
            "data"
        } else {
            "empty"
        };
        timing.record("disgenet", started, timing_outcome);
    }

    if include.contains(&GeneIncludeType::Diagnostics) {
        let started = Instant::now();
        add_diagnostics_section(gene).await;
        timing.record(
            "diagnostics",
            started,
            if gene.diagnostics_note.is_some() {
                "error"
            } else if gene
                .diagnostics
                .as_ref()
                .is_some_and(|section| !section.is_empty())
            {
                "data"
            } else {
                "empty"
            },
        );
    }

    if include.contains(&GeneIncludeType::Funding) {
        let started = Instant::now();
        add_funding_section(gene, optional_timeout).await;
        timing.record(
            "funding",
            started,
            if gene
                .funding
                .as_ref()
                .is_some_and(|section| !section.grants.is_empty())
            {
                "data"
            } else {
                "empty"
            },
        );
    }

    complete_gene_section_outcomes(gene, include);
    sync_timing_outcomes(timing, gene);
    Ok(())
}

pub async fn get(symbol: &str, sections: &[String]) -> Result<Gene, BioMcpError> {
    let options = GeneGetOptions::from_env_and_sections(symbol, sections)?;
    get_with_options(symbol, &options).await
}

pub async fn get_with_options(symbol: &str, options: &GeneGetOptions) -> Result<Gene, BioMcpError> {
    Ok(get_with_report(symbol, options).await?.gene)
}

pub async fn get_with_report(
    symbol: &str,
    options: &GeneGetOptions,
) -> Result<GeneGetResult, BioMcpError> {
    if symbol.trim().is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Gene symbol is required. Example: biomcp get gene BRAF".into(),
        ));
    }

    let strategy = options.strategy;
    let optional_timeout = if options.optional_timeout.is_zero() {
        Duration::from_millis(DEFAULT_OPTIONAL_ENRICHMENT_TIMEOUT_MS)
    } else {
        options.optional_timeout
    };
    let mut timing = GeneTimingCollector::new(symbol, strategy, options.timing_path.clone());
    let include = options.sections.clone();
    let use_parallel_top =
        strategy == GeneGetStrategy::ParallelTop && should_use_parallel_top(&include);
    let mut clingen_prefetch = if use_parallel_top && include.contains(&GeneIncludeType::ClinGen) {
        let symbol = symbol.trim().to_string();
        Some(tokio::spawn(async move {
            timed_section(
                "clingen",
                fetch_clingen_section(&symbol, optional_timeout),
                classify_clingen_section,
            )
            .await
        }))
    } else {
        None
    };

    let client = MyGeneClient::new()?;
    let started = Instant::now();
    let resp = client.get(symbol, false).await;
    timing.record(
        "mygene",
        started,
        if resp.is_ok() { "data" } else { "error" },
    );
    let resp = match resp {
        Ok(resp) => resp,
        Err(err @ BioMcpError::NotFound { .. }) => {
            if let Some(handle) = clingen_prefetch.take() {
                handle.abort();
            }
            if let Some(canonical_symbol) = unique_canonical_alias_symbol(&client, symbol).await? {
                return Box::pin(get_with_report(&canonical_symbol, options)).await;
            }
            return Err(err);
        }
        Err(err) => {
            if let Some(handle) = clingen_prefetch.take() {
                handle.abort();
            }
            return Err(err);
        }
    };

    let mut gene = transform::gene::from_mygene_get(resp);
    let opentargets_id = preferred_opentargets_id(&gene, strategy).map(str::to_string);

    if use_parallel_top {
        populate_sections_parallel_top(
            &mut gene,
            &include,
            &mut timing,
            opentargets_id.as_deref(),
            optional_timeout,
            clingen_prefetch,
        )
        .await?;
        let timing = timing.finish();
        return Ok(GeneGetResult { gene, timing });
    }

    let started = Instant::now();
    match add_clinical_context(&mut gene, opentargets_id.as_deref()).await {
        Ok(()) => timing.record(
            "clinical_context",
            started,
            if !gene.clinical_diseases.is_empty() || !gene.clinical_drugs.is_empty() {
                "data"
            } else {
                "empty"
            },
        ),
        Err(err) => {
            timing.record("clinical_context", started, "error");
            warn!("OpenTargets unavailable for gene clinical context: {err}");
        }
    }

    if include.contains(&GeneIncludeType::Pathways) {
        let started = Instant::now();
        gene.pathways = match fetch_pathways_section(&gene.symbol).await {
            Ok(value) => {
                let pathways = merge_pathways(gene.pathways.take(), value);
                gene.section_outcomes.complete(
                    GENE_SECTION_PATHWAYS,
                    pathway_outcome(pathways.as_deref(), true),
                );
                pathways
            }
            Err(_) => {
                gene.section_outcomes.complete(
                    GENE_SECTION_PATHWAYS,
                    pathway_outcome(gene.pathways.as_deref(), false),
                );
                gene.pathways
            }
        };
        timing.record(
            "pathways",
            started,
            if gene.pathways.as_ref().is_some_and(|rows| !rows.is_empty()) {
                "data"
            } else {
                "empty"
            },
        );
    } else {
        gene.pathways = None;
    }

    let enrichr_sections: Vec<GeneIncludeType> = include
        .iter()
        .copied()
        .filter(|v| matches!(v, GeneIncludeType::Ontology | GeneIncludeType::Diseases))
        .collect();

    if !enrichr_sections.is_empty() {
        let started = Instant::now();
        let enrichr = enrich_gene(&gene.symbol, &enrichr_sections).await;
        let (ontology, diseases) = match enrichr {
            Ok(value) => value,
            Err(_) => {
                timing.record("enrichr", started, "error");
                for section in &enrichr_sections {
                    gene.section_outcomes.complete(
                        section.as_str(),
                        SectionOutcome::unavailable("Enrichr gene enrichment is unavailable."),
                    );
                }
                (None, None)
            }
        };
        gene.ontology = ontology;
        gene.diseases = diseases;
        timing.record(
            "enrichr",
            started,
            if gene
                .ontology
                .as_ref()
                .is_some_and(|rows| rows.iter().any(|row| !row.terms.is_empty()))
                || gene
                    .diseases
                    .as_ref()
                    .is_some_and(|rows| rows.iter().any(|row| !row.terms.is_empty()))
            {
                "data"
            } else {
                "empty"
            },
        );
    }

    if include.contains(&GeneIncludeType::Protein) {
        let started = Instant::now();
        gene.protein = match fetch_protein_section(gene.uniprot_id.as_deref(), &gene.symbol).await {
            Ok(v) => v,
            Err(_) => {
                gene.section_outcomes.complete(
                    GENE_SECTION_PROTEIN,
                    SectionOutcome::unavailable("UniProt gene protein data is unavailable."),
                );
                None
            }
        };
        timing.record(
            "protein",
            started,
            if gene.protein.is_some() {
                "data"
            } else {
                "empty"
            },
        );
    }

    if include.contains(&GeneIncludeType::Go) {
        let started = Instant::now();
        let result = fetch_go_section(gene.uniprot_id.as_deref(), &gene.symbol).await;
        apply_go_section_result(&mut gene, result);
        timing.record(
            "go",
            started,
            if gene.go.as_ref().is_some_and(|rows| !rows.is_empty()) {
                "data"
            } else {
                "empty"
            },
        );
    }

    if include.contains(&GeneIncludeType::Interactions) {
        let started = Instant::now();
        let result = fetch_interactions_section(&gene.symbol).await;
        apply_gene_interactions_result(&mut gene, result);
        timing.record(
            "interactions",
            started,
            if gene
                .interactions
                .as_ref()
                .is_some_and(|rows| !rows.is_empty())
            {
                "data"
            } else {
                "empty"
            },
        );
    }

    if include.contains(&GeneIncludeType::Civic) {
        let started = Instant::now();
        add_civic_section(&mut gene, optional_timeout).await;
        timing.record(
            "civic",
            started,
            if gene
                .civic
                .as_ref()
                .is_some_and(|ctx| !ctx.evidence_items.is_empty() || !ctx.assertions.is_empty())
            {
                "data"
            } else {
                "empty"
            },
        );
    }

    if include.contains(&GeneIncludeType::Expression) {
        let started = Instant::now();
        add_expression_section(&mut gene, optional_timeout).await;
        timing.record(
            "expression",
            started,
            if gene
                .expression
                .as_ref()
                .is_some_and(|expression| !expression.tissues.is_empty())
            {
                "data"
            } else {
                "empty"
            },
        );
    }

    if include.contains(&GeneIncludeType::Hpa) {
        let started = Instant::now();
        add_hpa_section(&mut gene, optional_timeout).await;
        timing.record(
            "hpa",
            started,
            if gene.hpa.as_ref().is_some_and(|hpa| {
                !hpa.tissues.is_empty()
                    || !hpa.subcellular_main_location.is_empty()
                    || !hpa.subcellular_additional_location.is_empty()
                    || hpa.reliability.is_some()
                    || hpa.protein_summary.is_some()
                    || hpa.rna_summary.is_some()
            }) {
                "data"
            } else {
                "empty"
            },
        );
    }

    if include.contains(&GeneIncludeType::Druggability) {
        let started = Instant::now();
        add_druggability_section(&mut gene, opentargets_id.as_deref(), optional_timeout).await;
        timing.record(
            "druggability",
            started,
            if gene.druggability.as_ref().is_some_and(|section| {
                !section.categories.is_empty()
                    || !section.interactions.is_empty()
                    || !section.tractability.is_empty()
                    || !section.safety_liabilities.is_empty()
            }) {
                "data"
            } else {
                "empty"
            },
        );
    }

    if include.contains(&GeneIncludeType::ClinGen) {
        let started = Instant::now();
        add_clingen_section(&mut gene, optional_timeout).await;
        timing.record(
            "clingen",
            started,
            if gene.clingen.as_ref().is_some_and(|section| {
                !section.validity.is_empty()
                    || section.haploinsufficiency.is_some()
                    || section.triplosensitivity.is_some()
            }) {
                "data"
            } else {
                "empty"
            },
        );
    }

    if include.contains(&GeneIncludeType::Constraint) {
        let started = Instant::now();
        add_constraint_section(&mut gene, optional_timeout).await;
        timing.record(
            "constraint",
            started,
            if gene.constraint.as_ref().is_some_and(|section| {
                section.pli.is_some()
                    || section.loeuf.is_some()
                    || section.mis_z.is_some()
                    || section.syn_z.is_some()
                    || section.transcript.is_some()
            }) {
                "data"
            } else {
                "empty"
            },
        );
    }

    if include.contains(&GeneIncludeType::Disgenet) {
        let started = Instant::now();
        let timing_outcome = if let Err(err) = add_disgenet_section(&mut gene).await {
            warn!("DisGeNET unavailable for gene disease associations: {err}");
            gene.section_outcomes.complete(
                GENE_SECTION_DISGENET,
                SectionOutcome::unavailable("DisGeNET gene associations are unavailable."),
            );
            "error"
        } else if gene
            .disgenet
            .as_ref()
            .is_some_and(|section| !section.associations.is_empty())
        {
            "data"
        } else {
            "empty"
        };
        timing.record("disgenet", started, timing_outcome);
    }

    if include.contains(&GeneIncludeType::Diagnostics) {
        let started = Instant::now();
        add_diagnostics_section(&mut gene).await;
        timing.record(
            "diagnostics",
            started,
            if gene.diagnostics_note.is_some() {
                "error"
            } else if gene
                .diagnostics
                .as_ref()
                .is_some_and(|section| !section.is_empty())
            {
                "data"
            } else {
                "empty"
            },
        );
    }

    if include.contains(&GeneIncludeType::Funding) {
        let started = Instant::now();
        add_funding_section(&mut gene, optional_timeout).await;
        timing.record(
            "funding",
            started,
            if gene
                .funding
                .as_ref()
                .is_some_and(|section| !section.grants.is_empty())
            {
                "data"
            } else {
                "empty"
            },
        );
    }

    complete_gene_section_outcomes(&mut gene, &include);
    sync_timing_outcomes(&mut timing, &gene);
    let timing = timing.finish();
    Ok(GeneGetResult { gene, timing })
}

// dead-code reason: gene::search is exercised by native tests or binary dispatch
#[allow(dead_code)]
pub async fn search(
    filters: &GeneSearchFilters,
    limit: usize,
) -> Result<Vec<GeneSearchResult>, BioMcpError> {
    Ok(search_page(filters, limit, 0).await?.results)
}

pub async fn search_page(
    filters: &GeneSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<SearchPage<GeneSearchResult>, BioMcpError> {
    const MAX_SEARCH_LIMIT: usize = 50;

    let query = filters
        .query
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| {
            BioMcpError::InvalidArgument(
                "Query is required. Example: biomcp search gene -q BRAF".into(),
            )
        })?;

    if query.len() > 256 {
        return Err(BioMcpError::InvalidArgument(
            "Query is too long. Example: biomcp search gene -q BRAF".into(),
        ));
    }

    let gene_type = filters
        .gene_type
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());
    let chromosome = filters
        .chromosome
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());
    let region = filters
        .region
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());
    let pathway = filters
        .pathway
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());
    let go_term = filters
        .go_term
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());

    if gene_type.is_some_and(|v| v.len() > 64) {
        return Err(BioMcpError::InvalidArgument(
            "--type is too long. Example: --type protein-coding".into(),
        ));
    }

    if chromosome.is_some_and(|v| v.len() > 16) {
        return Err(BioMcpError::InvalidArgument(
            "--chromosome is too long. Example: --chromosome 7".into(),
        ));
    }
    if pathway.is_some_and(|v| v.len() > 128) {
        return Err(BioMcpError::InvalidArgument(
            "--pathway is too long. Example: --pathway R-HSA-5673001".into(),
        ));
    }
    if go_term.is_some_and(|v| v.len() > 128) {
        return Err(BioMcpError::InvalidArgument(
            "--go is too long. Example: --go GO:0004672".into(),
        ));
    }

    let normalized_gene_type = gene_type.map(normalize_gene_type).transpose()?;
    let mut normalized_chromosome = chromosome.map(normalize_gene_chromosome).transpose()?;
    let normalized_region = region.map(parse_region_filter).transpose()?;
    if let Some((region_chr, _, _)) = normalized_region.as_ref() {
        normalized_chromosome.get_or_insert_with(|| region_chr.clone());
    }

    if limit == 0 || limit > MAX_SEARCH_LIMIT {
        return Err(BioMcpError::InvalidArgument(format!(
            "--limit must be between 1 and {MAX_SEARCH_LIMIT}"
        )));
    }

    let mut terms: Vec<String> = vec![mygene_query_term(query)];

    if let Some(v) = normalized_gene_type {
        let escaped = MyGeneClient::escape_query_value(v);
        let value = format!("\"{escaped}\"");
        terms.push(format!("type_of_gene:{value}"));
    }

    if let Some(pathway) = pathway {
        let escaped = MyGeneClient::escape_query_value(pathway);
        terms.push(format!(
            "(pathway.kegg.id:\"{escaped}\" OR pathway.reactome.id:\"{escaped}\" OR pathway.kegg.name:*{escaped}*)"
        ));
    }

    if let Some(go_term) = go_term {
        let normalized_go = normalize_go_id(go_term)?;
        let escaped = MyGeneClient::escape_query_value(&normalized_go);
        terms.push(format!(
            "(go.BP.id:\"{escaped}\" OR go.CC.id:\"{escaped}\" OR go.MF.id:\"{escaped}\")"
        ));
    }

    if let Some((chr, start, end)) = normalized_region.as_ref() {
        terms.push(format!(
            "(genomic_pos.chr:{chr} AND genomic_pos.start:[{start} TO {end}])"
        ));
    }

    let q = terms.join(" AND ");

    let client = MyGeneClient::new()?;
    let fetch_limit = if normalized_chromosome.is_some() || normalized_gene_type.is_some() {
        (limit.saturating_add(offset)).clamp(limit, MAX_SEARCH_LIMIT)
    } else {
        limit
    };
    let resp = client
        .search(&q, fetch_limit, offset, normalized_chromosome.as_deref())
        .await?;
    let expected_gene_type = normalized_gene_type.map(str::to_ascii_lowercase);
    let expected_chr = normalized_chromosome.map(|v| v.to_ascii_uppercase());

    let mut out = resp
        .hits
        .iter()
        .filter(|hit| {
            if let Some(expected) = expected_gene_type.as_deref() {
                let actual = hit
                    .type_of_gene
                    .as_deref()
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                    .map(str::to_ascii_lowercase);
                if actual.as_deref() != Some(expected) {
                    return false;
                }
            }

            if let Some(expected) = expected_chr.as_deref() {
                let actual = hit
                    .genomic_pos
                    .as_ref()
                    .and_then(|g| g.chr())
                    .map(|v| v.trim_start_matches("chr").to_ascii_uppercase());
                if actual.as_deref() != Some(expected) {
                    return false;
                }
            }

            if let Some((region_chr, region_start, region_end)) = normalized_region.as_ref() {
                let Some(pos) = hit.genomic_pos.as_ref() else {
                    return false;
                };
                let actual_chr = pos
                    .chr()
                    .map(|v| v.trim_start_matches("chr").to_ascii_uppercase());
                if actual_chr.as_deref() != Some(region_chr.as_str()) {
                    return false;
                }
                let Some(actual_start) = pos.start() else {
                    return false;
                };
                let Some(actual_end) = pos.end() else {
                    return false;
                };
                if actual_start > *region_end || actual_end < *region_start {
                    return false;
                }
            }

            true
        })
        .map(transform::gene::from_mygene_hit)
        .collect::<Vec<_>>();
    out.truncate(limit);
    Ok(SearchPage::offset(out, Some(resp.total)))
}

pub fn search_query_summary(filters: &GeneSearchFilters) -> String {
    let mut parts: Vec<String> = Vec::new();

    if let Some(v) = filters
        .query
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(v.to_string());
    }

    if let Some(v) = filters
        .gene_type
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("type={v}"));
    }

    if let Some(v) = filters
        .chromosome
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("chromosome={v}"));
    }
    if let Some(v) = filters
        .region
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("region={v}"));
    }
    if let Some(v) = filters
        .pathway
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("pathway={v}"));
    }
    if let Some(v) = filters
        .go_term
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("go={v}"));
    }

    parts.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_gene(symbol: &str) -> Gene {
        Gene {
            section_outcomes: SectionOutcomes::with_keys(GENE_OUTCOME_KEYS),
            symbol: symbol.to_string(),
            name: format!("{symbol} gene"),
            entrez_id: "0".to_string(),
            ensembl_id: None,
            location: None,
            genomic_coordinates: None,
            omim_id: None,
            uniprot_id: None,
            summary: None,
            gene_type: None,
            aliases: Vec::new(),
            clinical_diseases: Vec::new(),
            clinical_drugs: Vec::new(),
            pathways: None,
            ontology: None,
            diseases: None,
            protein: None,
            go: None,
            interactions: None,
            civic: None,
            expression: None,
            hpa: None,
            druggability: None,
            clingen: None,
            constraint: None,
            disgenet: None,
            funding: None,
            funding_note: None,
            diagnostics: None,
            diagnostics_note: None,
        }
    }

    #[test]
    fn gnomad_constraint_without_metrics_is_healthy_empty() {
        let empty = gnomad_constraint_section(None, None, None, None, None);
        assert_eq!(
            gnomad_constraint_outcome(&empty).outcome(),
            SectionOutcomeState::Empty
        );

        let data = gnomad_constraint_section(Some("ENST0001".to_string()), None, None, None, None);
        assert_eq!(
            gnomad_constraint_outcome(&data).outcome(),
            SectionOutcomeState::Data
        );
    }

    #[test]
    fn outcome_inventory_matches_parser_visible_sections() {
        let registry = SectionOutcomes::with_keys(GENE_OUTCOME_KEYS);
        let keys = registry.iter().map(|(key, _)| key).collect::<Vec<_>>();
        let mut visible = GENE_SECTION_NAMES[..GENE_SECTION_NAMES.len() - 1].to_vec();
        visible.sort_unstable();
        assert_eq!(keys, visible);
        assert!(
            registry
                .iter()
                .all(|(_, value)| value.outcome() == SectionOutcomeState::NotRequested)
        );
    }

    #[test]
    fn search_query_summary_includes_new_filters() {
        let summary = search_query_summary(&GeneSearchFilters {
            query: Some("kinase".into()),
            gene_type: Some("protein-coding".into()),
            chromosome: Some("7".into()),
            region: None,
            pathway: None,
            go_term: None,
        });
        assert_eq!(summary, "kinase, type=protein-coding, chromosome=7");
    }

    #[test]
    fn mygene_query_term_escapes_free_text_special_chars() {
        assert_eq!(mygene_query_term("BRAF:V600E"), r"BRAF\:V600E");
        assert_eq!(mygene_query_term("ALK (fusion)"), r"ALK \(fusion\)");
    }

    #[test]
    fn mygene_query_term_searches_aliases_for_symbol_like_input() {
        assert_eq!(mygene_query_term("ERBB1"), "(symbol:ERBB1 OR alias:ERBB1)");
        assert_eq!(mygene_query_term("P53"), "(symbol:P53 OR alias:P53)");
    }

    fn mygene_hit(symbol: &str, aliases: &[&str]) -> MyGeneHit {
        MyGeneHit {
            symbol: Some(symbol.to_string()),
            name: Some(format!("{symbol} gene")),
            entrezgene: Some(crate::sources::mygene::StringOrU64::Number(1)),
            alias: crate::utils::serde::StringOrVec::Multiple(
                aliases.iter().map(|alias| alias.to_string()).collect(),
            ),
            type_of_gene: Some("protein-coding".to_string()),
            genomic_pos: None,
            mim: None,
            uniprot: None,
        }
    }

    #[test]
    fn canonical_alias_matches_cover_common_gene_aliases() {
        let hits = vec![
            mygene_hit("CD274", &["PD-L1", "PDCD1L1"]),
            mygene_hit("ERBB2", &["HER2", "NEU"]),
            mygene_hit("TP53", &["P53"]),
        ];

        assert_eq!(
            matching_canonical_alias_symbols("PD-L1", &hits),
            vec!["CD274"]
        );
        assert_eq!(
            matching_canonical_alias_symbols("HER2", &hits),
            vec!["ERBB2"]
        );
        assert_eq!(matching_canonical_alias_symbols("P53", &hits), vec!["TP53"]);
    }

    #[test]
    fn canonical_alias_matches_keep_ambiguous_aliases_ambiguous() {
        let hits = vec![
            mygene_hit("GENE1", &["SHARED"]),
            mygene_hit("GENE2", &["SHARED"]),
        ];

        assert_eq!(
            matching_canonical_alias_symbols("SHARED", &hits),
            vec!["GENE1", "GENE2"]
        );
    }

    #[test]
    fn canonical_alias_identity_keeps_the_entrez_identifier() {
        let aliases = matching_canonical_aliases("ERBB1", &[mygene_hit("EGFR", &["ERBB1"])]);
        assert_eq!(
            aliases,
            vec![CanonicalGeneAlias {
                symbol: "EGFR".into(),
                entrez_id: "1".into(),
            }]
        );
    }

    #[test]
    fn search_query_includes_chromosome_filter() {
        let summary = search_query_summary(&GeneSearchFilters {
            query: Some("BRCA1".into()),
            gene_type: None,
            chromosome: Some("17".into()),
            region: None,
            pathway: None,
            go_term: None,
        });
        assert_eq!(summary, "BRCA1, chromosome=17");
    }

    #[test]
    fn normalize_gene_type_accepts_supported_aliases() {
        assert_eq!(
            normalize_gene_type("protein-coding").expect("protein-coding should parse"),
            "protein-coding"
        );
        assert_eq!(
            normalize_gene_type("ncRNA").expect("ncRNA should parse"),
            "ncRNA"
        );
        assert_eq!(
            normalize_gene_type("ncrna").expect("ncrna alias should parse"),
            "ncRNA"
        );
        assert_eq!(
            normalize_gene_type("pseudo").expect("pseudo should parse"),
            "pseudo"
        );
    }

    #[test]
    fn normalize_gene_type_rejects_invalid_value() {
        let err = normalize_gene_type("invalid").expect_err("invalid gene type should fail");
        assert!(err.to_string().contains("protein-coding"));
    }

    #[test]
    fn normalize_gene_chromosome_accepts_chr_prefix_and_special_values() {
        assert_eq!(
            normalize_gene_chromosome("chr7").expect("chr7 should parse"),
            "7"
        );
        assert_eq!(normalize_gene_chromosome("X").expect("X should parse"), "X");
        assert_eq!(
            normalize_gene_chromosome("chrmt").expect("chrmt should parse"),
            "MT"
        );
    }

    #[test]
    fn normalize_gene_chromosome_rejects_invalid_values() {
        let err = normalize_gene_chromosome("99").expect_err("99 should fail");
        assert!(err.to_string().contains("1-22"));
    }

    #[test]
    fn normalize_go_id_accepts_canonical_and_lowercase_prefix() {
        assert_eq!(
            normalize_go_id("GO:0004672").expect("valid GO ID"),
            "GO:0004672"
        );
        assert_eq!(
            normalize_go_id("go:0008150").expect("lowercase GO ID"),
            "GO:0008150"
        );
    }

    #[test]
    fn normalize_go_id_rejects_free_text() {
        let err = normalize_go_id("DNA repair").expect_err("free text should fail");
        assert!(err.to_string().contains("GO:0000000"));
    }

    #[test]
    fn gene_section_names_include_new_enrichment_sections() {
        assert!(GENE_SECTION_NAMES.contains(&"expression"));
        assert!(GENE_SECTION_NAMES.contains(&"hpa"));
        assert!(GENE_SECTION_NAMES.contains(&"druggability"));
        assert!(GENE_SECTION_NAMES.contains(&"clingen"));
        assert!(GENE_SECTION_NAMES.contains(&"constraint"));
        assert!(GENE_SECTION_NAMES.contains(&"disgenet"));
        assert!(GENE_SECTION_NAMES.contains(&"funding"));
        assert!(GENE_SECTION_NAMES.contains(&"diagnostics"));
    }

    #[test]
    fn parse_sections_accepts_new_enrichment_sections() {
        let parsed = parse_sections(
            "BRAF",
            &[
                "expression".to_string(),
                "hpa".to_string(),
                "druggability".to_string(),
                "clingen".to_string(),
                "constraint".to_string(),
                "disgenet".to_string(),
                "funding".to_string(),
                "diagnostics".to_string(),
            ],
        )
        .expect("new gene sections should parse");
        assert_eq!(parsed.len(), 8);
        assert!(parsed.contains(&GeneIncludeType::Diagnostics));
    }

    #[test]
    fn parse_sections_accepts_diagnostics() {
        let parsed =
            parse_sections("BRAF", &["diagnostics".to_string()]).expect("diagnostics should parse");
        assert_eq!(parsed.len(), 1);
        assert!(parsed.contains(&GeneIncludeType::Diagnostics));
    }

    #[test]
    fn parse_sections_all_keeps_optional_sections_opt_in() {
        let parsed = parse_sections("BRAF", &["all".to_string()]).expect("all should parse");
        assert_eq!(parsed.len(), 12);
        assert!(!parsed.contains(&GeneIncludeType::Diagnostics));
        assert!(!parsed.contains(&GeneIncludeType::Disgenet));
        assert!(!parsed.contains(&GeneIncludeType::Funding));
    }

    #[test]
    fn parse_sections_all_keeps_optional_diagnostics_opt_in() {
        let parsed = parse_sections("BRAF", &["all".to_string()]).expect("all should parse");
        assert!(!parsed.contains(&GeneIncludeType::Diagnostics));
    }

    #[test]
    fn gene_diagnostics_section_populates_from_rows() {
        let mut gene = test_gene("BRCA1");
        apply_diagnostics_section_result(
            &mut gene,
            "BRCA1",
            Ok(vec![DiagnosticSearchResult {
                source: crate::entities::diagnostic::DIAGNOSTIC_SOURCE_GTR.to_string(),
                accession: "GTR000000001.1".to_string(),
                name: "BRCA1 Hereditary Cancer Panel".to_string(),
                test_type: Some("Clinical".to_string()),
                manufacturer_or_lab: Some("Example Lab".to_string()),
                genes: vec!["BRCA1".to_string()],
                conditions: vec!["Hereditary breast ovarian cancer".to_string()],
            }]),
        );

        let rows = gene.diagnostics.as_ref().expect("diagnostics rows");
        assert!(gene.diagnostics_note.is_none());
        assert!(rows.iter().any(|row| {
            row.source == crate::entities::diagnostic::DIAGNOSTIC_SOURCE_GTR
                && row.accession == "GTR000000001.1"
                && row.name == "BRCA1 Hereditary Cancer Panel"
                && row.genes.iter().any(|gene| gene == "BRCA1")
        }));
    }

    #[test]
    fn gene_diagnostics_unavailable_sets_note() {
        let mut gene = test_gene("BRCA1");
        apply_diagnostics_section_result(
            &mut gene,
            "BRCA1",
            Err(BioMcpError::SourceUnavailable {
                source_name: "gtr".to_string(),
                reason: "fixture directory is unavailable".to_string(),
                suggestion: "Run `biomcp gtr sync`".to_string(),
            }),
        );

        assert!(gene.diagnostics.is_none());
        assert_eq!(
            gene.diagnostics_note.as_deref(),
            Some(GENE_DIAGNOSTICS_UNAVAILABLE_NOTE)
        );
    }

    #[test]
    fn parse_sections_redirects_variants_to_variant_search() {
        let err = parse_sections("SCN5A", &["variants".to_string()])
            .expect_err("variants should redirect");

        let message = err.to_string();
        assert!(message.contains("Gene does not have a \"variants\" section."));
        assert!(message.contains("`biomcp search variant -g SCN5A`"));
        assert!(!message.contains("Available:"));
    }

    #[test]
    fn merge_pathways_keeps_kegg_then_appends_reactome_without_duplicates() {
        let merged = merge_pathways(
            Some(vec![GenePathway {
                source: "KEGG".to_string(),
                id: "hsa04010".to_string(),
                name: "MAPK signaling pathway".to_string(),
            }]),
            Some(vec![
                GenePathway {
                    source: "Reactome".to_string(),
                    id: "R-HSA-5673001".to_string(),
                    name: "RAF/MAP kinase cascade".to_string(),
                },
                GenePathway {
                    source: "KEGG".to_string(),
                    id: "HSA04010".to_string(),
                    name: "duplicate".to_string(),
                },
            ]),
        )
        .expect("merged");

        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].source, "KEGG");
        assert_eq!(merged[1].source, "Reactome");
    }

    #[test]
    fn pathway_outcome_credits_merged_sources_and_only_retained_sources_on_failure() {
        let pathways = vec![
            GenePathway {
                source: "KEGG".to_string(),
                id: "hsa04010".to_string(),
                name: "MAPK signaling pathway".to_string(),
            },
            GenePathway {
                source: "Reactome".to_string(),
                id: "R-HSA-5673001".to_string(),
                name: "RAF/MAP kinase cascade".to_string(),
            },
        ];

        let healthy = pathway_outcome(Some(&pathways), true);
        assert_eq!(healthy.outcome(), SectionOutcomeState::Data);
        assert_eq!(healthy.sources(), &["KEGG", "Reactome"]);

        let degraded = pathway_outcome(Some(&pathways[..1]), false);
        assert_eq!(degraded.outcome(), SectionOutcomeState::Degraded);
        assert_eq!(degraded.sources(), &["KEGG"]);

        let unavailable = pathway_outcome(None, false);
        assert_eq!(unavailable.outcome(), SectionOutcomeState::Unavailable);
        assert!(unavailable.sources().is_empty());
    }

    #[test]
    fn merge_druggability_keeps_successful_source_data_when_other_source_fails() {
        let merged = merge_druggability_results(
            Err(BioMcpError::Api {
                api: "dgidb".to_string(),
                message: "down".to_string(),
            }),
            Ok(
                crate::sources::opentargets::OpenTargetsTargetDruggabilityContext {
                    tractability: vec![
                        crate::sources::opentargets::OpenTargetsTractabilityModality {
                            modality: "small molecule".to_string(),
                            tractable: true,
                            evidence_labels: vec!["Approved Drug".to_string()],
                        },
                    ],
                    safety_liabilities: vec![
                        crate::sources::opentargets::OpenTargetsSafetyLiability {
                            event: "Skin rash".to_string(),
                            datasource: Some("ForceGenetics".to_string()),
                            effect_direction: Some("activation".to_string()),
                            biosample: Some("Skin".to_string()),
                        },
                    ],
                },
            ),
        );

        assert!(merged.categories.is_empty());
        assert!(merged.interactions.is_empty());
        assert_eq!(merged.tractability.len(), 1);
        assert_eq!(merged.safety_liabilities.len(), 1);

        let merged = merge_druggability_results(
            Ok(GeneDruggability {
                categories: vec!["Kinase".to_string()],
                interactions: Vec::new(),
                tractability: Vec::new(),
                safety_liabilities: Vec::new(),
            }),
            Err(BioMcpError::Api {
                api: "opentargets".to_string(),
                message: "down".to_string(),
            }),
        );

        assert_eq!(merged.categories, vec!["Kinase"]);
        assert!(merged.tractability.is_empty());
        assert!(merged.safety_liabilities.is_empty());
    }

    fn injected_section_failure(source: &str, kind: &str) -> BioMcpError {
        match kind {
            "connection-refused" => BioMcpError::Api {
                api: source.to_string(),
                message: "connection refused".to_string(),
            },
            "timeout" => BioMcpError::SourceUnavailable {
                source_name: source.to_string(),
                reason: "request timed out".to_string(),
                suggestion: "retry".to_string(),
            },
            "malformed-body" => BioMcpError::ApiJson {
                api: source.to_string(),
                source: serde_json::from_str::<serde_json::Value>("{")
                    .expect_err("fixture body must be malformed"),
            },
            other => panic!("unknown injected failure: {other}"),
        }
    }

    fn assert_gene_section_outcome(
        gene: &Gene,
        key: &str,
        expected: SectionOutcomeState,
        successful_source: &str,
    ) {
        let outcome = gene.section_outcomes.get(key).expect("registered outcome");
        assert_eq!(outcome.outcome(), expected, "wrong outcome for {key}");
        if expected == SectionOutcomeState::Unavailable {
            assert!(
                outcome.sources().is_empty(),
                "failed {key} source received successful credit"
            );
        } else {
            assert_eq!(outcome.sources(), &[successful_source.to_string()]);
        }
    }

    #[test]
    fn quickgo_and_string_failure_state_matrix() {
        for failure in ["connection-refused", "timeout", "malformed-body"] {
            let mut gene = test_gene("BRAF");
            let error = injected_section_failure("QuickGO", failure);
            let private_detail = error.to_string();
            apply_go_section_result(&mut gene, Err::<Vec<GeneGoTerm>, _>(error));
            assert!(gene.go.as_ref().is_some_and(Vec::is_empty));
            assert!(
                !serde_json::to_string(&gene)
                    .expect("failed GO state serializes")
                    .contains(&private_detail)
            );
            assert_gene_section_outcome(
                &gene,
                GENE_SECTION_GO,
                SectionOutcomeState::Unavailable,
                "QuickGO",
            );

            let mut gene = test_gene("BRAF");
            let error = injected_section_failure("STRING", failure);
            let private_detail = error.to_string();
            apply_gene_interactions_result(&mut gene, Err::<Vec<GeneInteraction>, _>(error));
            assert!(gene.interactions.as_ref().is_some_and(Vec::is_empty));
            assert!(
                !serde_json::to_string(&gene)
                    .expect("failed interaction state serializes")
                    .contains(&private_detail)
            );
            assert_gene_section_outcome(
                &gene,
                GENE_SECTION_INTERACTIONS,
                SectionOutcomeState::Unavailable,
                "STRING",
            );
        }

        let mut empty = test_gene("BRAF");
        apply_go_section_result(&mut empty, Ok(Vec::new()));
        assert!(empty.go.as_ref().is_some_and(Vec::is_empty));
        assert_gene_section_outcome(
            &empty,
            GENE_SECTION_GO,
            SectionOutcomeState::Empty,
            "QuickGO",
        );
        let mut data = test_gene("BRAF");
        apply_go_section_result(
            &mut data,
            Ok(vec![GeneGoTerm {
                id: "GO:0004672".to_string(),
                name: "protein kinase activity".to_string(),
                aspect: Some("molecular_function".to_string()),
                evidence: Some("IDA".to_string()),
            }]),
        );
        assert_eq!(data.go.as_ref().expect("GO payload")[0].id, "GO:0004672");
        assert_gene_section_outcome(&data, GENE_SECTION_GO, SectionOutcomeState::Data, "QuickGO");

        let mut empty = test_gene("BRAF");
        apply_gene_interactions_result(&mut empty, Ok(Vec::new()));
        assert!(empty.interactions.as_ref().is_some_and(Vec::is_empty));
        assert_gene_section_outcome(
            &empty,
            GENE_SECTION_INTERACTIONS,
            SectionOutcomeState::Empty,
            "STRING",
        );
        let mut data = test_gene("BRAF");
        apply_gene_interactions_result(
            &mut data,
            Ok(vec![GeneInteraction {
                partner: "MAP2K1".to_string(),
                score: Some(0.99),
            }]),
        );
        assert_eq!(
            data.interactions.as_ref().expect("interaction payload")[0].partner,
            "MAP2K1"
        );
        assert_gene_section_outcome(
            &data,
            GENE_SECTION_INTERACTIONS,
            SectionOutcomeState::Data,
            "STRING",
        );
    }

    fn normalized_timing(timing: &GeneTimingCollector) -> Vec<(String, SectionOutcomeState)> {
        let mut entries = timing
            .sections
            .iter()
            .map(|entry| (entry.section.clone(), entry.outcome))
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| left.0.cmp(&right.0));
        entries
    }

    fn parity_go_result(case: &str) -> Result<Vec<GeneGoTerm>, BioMcpError> {
        match case {
            "healthy-empty" => Ok(Vec::new()),
            "data" => Ok(vec![GeneGoTerm {
                id: "GO:0004672".to_string(),
                name: "protein kinase activity".to_string(),
                aspect: None,
                evidence: None,
            }]),
            failure => Err(injected_section_failure("QuickGO", failure)),
        }
    }

    fn parity_interactions_result(case: &str) -> Result<Vec<GeneInteraction>, BioMcpError> {
        match case {
            "healthy-empty" => Ok(Vec::new()),
            "data" => Ok(vec![GeneInteraction {
                partner: "MAP2K1".to_string(),
                score: Some(0.99),
            }]),
            failure => Err(injected_section_failure("STRING", failure)),
        }
    }

    #[test]
    fn gene_section_result_application_is_strategy_order_invariant() {
        for case in [
            "connection-refused",
            "timeout",
            "malformed-body",
            "healthy-empty",
            "data",
        ] {
            let mut baseline = test_gene("BRAF");
            apply_go_section_result(&mut baseline, parity_go_result(case));
            apply_gene_interactions_result(&mut baseline, parity_interactions_result(case));

            let mut parallel_top = test_gene("BRAF");
            apply_gene_interactions_result(&mut parallel_top, parity_interactions_result(case));
            apply_go_section_result(&mut parallel_top, parity_go_result(case));

            assert_eq!(
                serde_json::to_value(&baseline).expect("baseline serializes"),
                serde_json::to_value(&parallel_top).expect("parallel-top serializes"),
                "entity parity failed for {case}"
            );
            assert_eq!(
                crate::render::provenance::gene_section_sources(&baseline),
                crate::render::provenance::gene_section_sources(&parallel_top),
                "provenance parity failed for {case}"
            );

            let mut baseline_timing =
                GeneTimingCollector::new("BRAF", GeneGetStrategy::Baseline, None);
            baseline_timing.push(GeneTimingEntry {
                section: GENE_SECTION_GO.to_string(),
                elapsed_ms: 1,
                outcome: SectionOutcomeState::Unavailable,
            });
            baseline_timing.push(GeneTimingEntry {
                section: GENE_SECTION_INTERACTIONS.to_string(),
                elapsed_ms: 2,
                outcome: SectionOutcomeState::Unavailable,
            });
            sync_timing_outcomes(&mut baseline_timing, &baseline);

            let mut parallel_timing =
                GeneTimingCollector::new("BRAF", GeneGetStrategy::ParallelTop, None);
            parallel_timing.push(GeneTimingEntry {
                section: GENE_SECTION_INTERACTIONS.to_string(),
                elapsed_ms: 9,
                outcome: SectionOutcomeState::Unavailable,
            });
            parallel_timing.push(GeneTimingEntry {
                section: GENE_SECTION_GO.to_string(),
                elapsed_ms: 7,
                outcome: SectionOutcomeState::Unavailable,
            });
            sync_timing_outcomes(&mut parallel_timing, &parallel_top);

            let expected_outcome = match case {
                "healthy-empty" => SectionOutcomeState::Empty,
                "data" => SectionOutcomeState::Data,
                _ => SectionOutcomeState::Unavailable,
            };
            assert!(
                normalized_timing(&baseline_timing)
                    .iter()
                    .all(|(_, outcome)| *outcome == expected_outcome),
                "baseline timing classification failed for {case}"
            );
            assert!(
                normalized_timing(&parallel_timing)
                    .iter()
                    .all(|(_, outcome)| *outcome == expected_outcome),
                "parallel-top timing classification failed for {case}"
            );
            assert_eq!(
                normalized_timing(&baseline_timing),
                normalized_timing(&parallel_timing),
                "timing parity failed for {case}"
            );
        }
    }
}
