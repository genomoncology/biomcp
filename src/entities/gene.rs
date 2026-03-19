use std::collections::HashMap;
use std::time::Duration;

use futures::future::try_join_all;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::entities::SearchPage;
use crate::error::BioMcpError;
use crate::sources::civic::{CivicClient, CivicContext};
use crate::sources::clingen::{ClinGenClient, GeneClinGen};
use crate::sources::dgidb::{DgidbClient, GeneDruggability};
use crate::sources::enrichr::EnrichrClient;
use crate::sources::gnomad::{
    GNOMAD_CONSTRAINT_REFERENCE_GENOME, GNOMAD_CONSTRAINT_VERSION, GnomadClient,
};
use crate::sources::gtex::{GeneExpression, GtexClient};
use crate::sources::hpa::{GeneHpa, HpaClient};
use crate::sources::mygene::MyGeneClient;
use crate::sources::opentargets::OpenTargetsClient;
use crate::sources::quickgo::QuickGoClient;
use crate::sources::reactome::ReactomeClient;
use crate::sources::string::StringClient;
use crate::sources::uniprot::UniProtClient;
use crate::transform;

/// Gene entity from MyGene.info plus optional enrichment sections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gene {
    pub symbol: String,
    pub name: String,
    pub entrez_id: String,
    pub ensembl_id: Option<String>,
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genomic_coordinates: Option<String>,
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

/// Search result (lighter than full Gene)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneSearchResult {
    pub symbol: String,
    pub name: String,
    pub entrez_id: String,
    pub genomic_coordinates: Option<String>,
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
enum GeneIncludeType {
    Pathways,
    Ontology,
    Diseases,
    Protein,
    Go,
    Interactions,
    Civic,
    Expression,
    Hpa,
    Druggability,
    ClinGen,
    Constraint,
}

const GENE_SECTION_PATHWAYS: &str = "pathways";
const GENE_SECTION_ONTOLOGY: &str = "ontology";
const GENE_SECTION_DISEASES: &str = "diseases";
const GENE_SECTION_PROTEIN: &str = "protein";
const GENE_SECTION_GO: &str = "go";
const GENE_SECTION_INTERACTIONS: &str = "interactions";
const GENE_SECTION_CIVIC: &str = "civic";
const GENE_SECTION_EXPRESSION: &str = "expression";
const GENE_SECTION_HPA: &str = "hpa";
const GENE_SECTION_DRUGGABILITY: &str = "druggability";
const GENE_SECTION_CLINGEN: &str = "clingen";
const GENE_SECTION_CONSTRAINT: &str = "constraint";
const GENE_SECTION_ALL: &str = "all";

pub const GENE_SECTION_NAMES: &[&str] = &[
    GENE_SECTION_PATHWAYS,
    GENE_SECTION_ONTOLOGY,
    GENE_SECTION_DISEASES,
    GENE_SECTION_PROTEIN,
    GENE_SECTION_GO,
    GENE_SECTION_INTERACTIONS,
    GENE_SECTION_CIVIC,
    GENE_SECTION_EXPRESSION,
    GENE_SECTION_HPA,
    GENE_SECTION_DRUGGABILITY,
    GENE_SECTION_CLINGEN,
    GENE_SECTION_CONSTRAINT,
    GENE_SECTION_ALL,
];

impl GeneIncludeType {
    fn from_section(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            GENE_SECTION_PATHWAYS | "pathway" => Some(Self::Pathways),
            GENE_SECTION_ONTOLOGY => Some(Self::Ontology),
            GENE_SECTION_DISEASES | "disease" => Some(Self::Diseases),
            GENE_SECTION_PROTEIN => Some(Self::Protein),
            GENE_SECTION_GO => Some(Self::Go),
            GENE_SECTION_INTERACTIONS | "interaction" => Some(Self::Interactions),
            GENE_SECTION_CIVIC => Some(Self::Civic),
            GENE_SECTION_EXPRESSION => Some(Self::Expression),
            GENE_SECTION_HPA => Some(Self::Hpa),
            GENE_SECTION_DRUGGABILITY | "drugs" => Some(Self::Druggability),
            GENE_SECTION_CLINGEN => Some(Self::ClinGen),
            GENE_SECTION_CONSTRAINT => Some(Self::Constraint),
            _ => None,
        }
    }

    pub fn libraries(&self) -> &'static [&'static str] {
        match self {
            // Pathways come from Reactome directly, not Enrichr.
            Self::Pathways => &[],
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
            | Self::Constraint => &[],
        }
    }
}

const OPTIONAL_ENRICHMENT_TIMEOUT: Duration = Duration::from_secs(8);

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

fn looks_like_symbol(query: &str) -> bool {
    if query.is_empty() {
        return false;
    }
    query
        .chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-')
        && query.chars().any(|c| c.is_ascii_uppercase())
}

fn mygene_query_term(query: &str) -> String {
    if looks_like_symbol(query) {
        let escaped = MyGeneClient::escape_query_value(query);
        format!("(symbol:{escaped} OR alias:{escaped})")
    } else {
        MyGeneClient::escape_query_value(query)
    }
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
            | GeneIncludeType::Constraint => {}
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

fn parse_sections(sections: &[String]) -> Result<Vec<GeneIncludeType>, BioMcpError> {
    let mut include: Vec<GeneIncludeType> = Vec::new();
    let mut include_all = false;

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
    Ok(Some(GeneProtein {
        accession,
        name: record.display_name(),
        function: record.function_summary(),
        length: record.sequence.and_then(|s| s.length),
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

async fn add_clinical_context(gene: &mut Gene) -> Result<(), BioMcpError> {
    let symbol = gene.symbol.trim();
    if symbol.is_empty() {
        return Ok(());
    }

    let context = OpenTargetsClient::new()?
        .target_clinical_context(symbol, 5)
        .await?;
    gene.clinical_diseases = context.diseases;
    gene.clinical_drugs = context.drugs;
    Ok(())
}

async fn add_civic_section(gene: &mut Gene) {
    let symbol = gene.symbol.trim();
    if symbol.is_empty() {
        return;
    }

    let civic_fut = async {
        let client = CivicClient::new()?;
        client.by_molecular_profile(symbol, 10).await
    };

    match tokio::time::timeout(OPTIONAL_ENRICHMENT_TIMEOUT, civic_fut).await {
        Ok(Ok(context)) => gene.civic = Some(context),
        Ok(Err(err)) => {
            warn!(symbol = %gene.symbol, "CIViC unavailable for gene section: {err}");
            gene.civic = Some(CivicContext::default());
        }
        Err(_) => {
            warn!(
                symbol = %gene.symbol,
                timeout_secs = OPTIONAL_ENRICHMENT_TIMEOUT.as_secs(),
                "CIViC gene section timed out"
            );
            gene.civic = Some(CivicContext::default());
        }
    }
}

async fn add_expression_section(gene: &mut Gene) {
    let Some(ensembl_id) = gene
        .ensembl_id
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    else {
        gene.expression = Some(GeneExpression::default());
        return;
    };

    let expression_fut = async {
        let client = GtexClient::new()?;
        let tissues = client.median_gene_expression(ensembl_id).await?;
        Ok::<_, BioMcpError>(GeneExpression { tissues })
    };

    match tokio::time::timeout(OPTIONAL_ENRICHMENT_TIMEOUT, expression_fut).await {
        Ok(Ok(expression)) => gene.expression = Some(expression),
        Ok(Err(err)) => {
            warn!(
                symbol = %gene.symbol,
                ensembl_id = %ensembl_id,
                "GTEx unavailable for gene expression section: {err}"
            );
            gene.expression = Some(GeneExpression::default());
        }
        Err(_) => {
            warn!(
                symbol = %gene.symbol,
                ensembl_id = %ensembl_id,
                timeout_secs = OPTIONAL_ENRICHMENT_TIMEOUT.as_secs(),
                "GTEx expression section timed out"
            );
            gene.expression = Some(GeneExpression::default());
        }
    }
}

async fn add_hpa_section(gene: &mut Gene) {
    let Some(ensembl_id) = gene
        .ensembl_id
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    else {
        gene.hpa = Some(GeneHpa::default());
        return;
    };

    let hpa_fut = async {
        let client = HpaClient::new()?;
        client.protein_data(ensembl_id).await
    };

    match tokio::time::timeout(OPTIONAL_ENRICHMENT_TIMEOUT, hpa_fut).await {
        Ok(Ok(hpa)) => gene.hpa = Some(hpa),
        Ok(Err(err)) => {
            warn!(
                symbol = %gene.symbol,
                ensembl_id = %ensembl_id,
                "HPA unavailable for gene section: {err}"
            );
            gene.hpa = Some(GeneHpa::default());
        }
        Err(_) => {
            warn!(
                symbol = %gene.symbol,
                ensembl_id = %ensembl_id,
                timeout_secs = OPTIONAL_ENRICHMENT_TIMEOUT.as_secs(),
                "HPA gene section timed out"
            );
            gene.hpa = Some(GeneHpa::default());
        }
    }
}

async fn add_druggability_section(gene: &mut Gene) {
    let symbol = gene.symbol.trim();
    if symbol.is_empty() {
        gene.druggability = Some(GeneDruggability::default());
        return;
    }

    let dgidb_fut = async {
        let client = DgidbClient::new()?;
        client.gene_interactions(symbol).await
    };

    match tokio::time::timeout(OPTIONAL_ENRICHMENT_TIMEOUT, dgidb_fut).await {
        Ok(Ok(druggability)) => gene.druggability = Some(druggability),
        Ok(Err(err)) => {
            warn!(
                symbol = %gene.symbol,
                "DGIdb unavailable for gene druggability section: {err}"
            );
            gene.druggability = Some(GeneDruggability::default());
        }
        Err(_) => {
            warn!(
                symbol = %gene.symbol,
                timeout_secs = OPTIONAL_ENRICHMENT_TIMEOUT.as_secs(),
                "DGIdb gene section timed out"
            );
            gene.druggability = Some(GeneDruggability::default());
        }
    }
}

async fn add_clingen_section(gene: &mut Gene) {
    let symbol = gene.symbol.trim();
    if symbol.is_empty() {
        gene.clingen = Some(GeneClinGen::default());
        return;
    }

    let clingen_fut = async {
        let client = ClinGenClient::new()?;
        let validity = client.gene_validity(symbol).await?;
        let (haploinsufficiency, triplosensitivity) = client.dosage_sensitivity(symbol).await?;
        Ok::<_, BioMcpError>(GeneClinGen {
            validity,
            haploinsufficiency,
            triplosensitivity,
        })
    };

    match tokio::time::timeout(OPTIONAL_ENRICHMENT_TIMEOUT, clingen_fut).await {
        Ok(Ok(clingen)) => gene.clingen = Some(clingen),
        Ok(Err(err)) => {
            warn!(
                symbol = %gene.symbol,
                "ClinGen unavailable for gene clingen section: {err}"
            );
            gene.clingen = Some(GeneClinGen::default());
        }
        Err(_) => {
            warn!(
                symbol = %gene.symbol,
                timeout_secs = OPTIONAL_ENRICHMENT_TIMEOUT.as_secs(),
                "ClinGen gene section timed out"
            );
            gene.clingen = Some(GeneClinGen::default());
        }
    }
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

async fn add_constraint_section(gene: &mut Gene) {
    let symbol = gene.symbol.trim();
    if symbol.is_empty() {
        gene.constraint = Some(gnomad_constraint_section(None, None, None, None, None));
        return;
    }

    let constraint_fut = async {
        let client = GnomadClient::new()?;
        client.gene_constraint(symbol).await
    };

    match tokio::time::timeout(OPTIONAL_ENRICHMENT_TIMEOUT, constraint_fut).await {
        Ok(Ok(Some(constraint))) => {
            gene.constraint = Some(gnomad_constraint_section(
                constraint.transcript,
                constraint.pli,
                constraint.loeuf,
                constraint.mis_z,
                constraint.syn_z,
            ));
        }
        Ok(Ok(None)) => {
            gene.constraint = Some(gnomad_constraint_section(None, None, None, None, None));
        }
        Ok(Err(err)) => {
            warn!(
                symbol = %gene.symbol,
                "gnomAD unavailable for gene constraint section: {err}"
            );
            gene.constraint = Some(gnomad_constraint_section(None, None, None, None, None));
        }
        Err(_) => {
            warn!(
                symbol = %gene.symbol,
                timeout_secs = OPTIONAL_ENRICHMENT_TIMEOUT.as_secs(),
                "gnomAD gene constraint section timed out"
            );
            gene.constraint = Some(gnomad_constraint_section(None, None, None, None, None));
        }
    }
}

pub async fn get(symbol: &str, sections: &[String]) -> Result<Gene, BioMcpError> {
    if symbol.trim().is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Gene symbol is required. Example: biomcp get gene BRAF".into(),
        ));
    }

    let include = parse_sections(sections)?;

    let client = MyGeneClient::new()?;
    let resp = client.get(symbol, false).await?;

    let mut gene = transform::gene::from_mygene_get(resp);

    if let Err(err) = add_clinical_context(&mut gene).await {
        warn!("OpenTargets unavailable for gene clinical context: {err}");
    }

    if include.contains(&GeneIncludeType::Pathways) {
        gene.pathways = match fetch_pathways_section(&gene.symbol).await {
            Ok(v) => merge_pathways(gene.pathways.take(), v),
            Err(err) => {
                warn!("Reactome unavailable for gene pathways section: {err}");
                gene.pathways
            }
        };
    } else {
        gene.pathways = None;
    }

    let enrichr_sections: Vec<GeneIncludeType> = include
        .iter()
        .copied()
        .filter(|v| matches!(v, GeneIncludeType::Ontology | GeneIncludeType::Diseases))
        .collect();

    if !enrichr_sections.is_empty() {
        let (ontology, diseases) = enrich_gene(&gene.symbol, &enrichr_sections).await?;
        gene.ontology = ontology;
        gene.diseases = diseases;
    }

    if include.contains(&GeneIncludeType::Protein) {
        gene.protein = match fetch_protein_section(gene.uniprot_id.as_deref(), &gene.symbol).await {
            Ok(v) => v,
            Err(err) => {
                warn!("UniProt unavailable for gene protein section: {err}");
                None
            }
        };
    }

    if include.contains(&GeneIncludeType::Go) {
        gene.go = match fetch_go_section(gene.uniprot_id.as_deref(), &gene.symbol).await {
            Ok(v) => Some(v),
            Err(err) => {
                warn!("QuickGO unavailable for gene GO section: {err}");
                Some(Vec::new())
            }
        };
    }

    if include.contains(&GeneIncludeType::Interactions) {
        gene.interactions = match fetch_interactions_section(&gene.symbol).await {
            Ok(v) => Some(v),
            Err(err) => {
                warn!("STRING unavailable for gene interactions section: {err}");
                Some(Vec::new())
            }
        };
    }

    if include.contains(&GeneIncludeType::Civic) {
        add_civic_section(&mut gene).await;
    }

    if include.contains(&GeneIncludeType::Expression) {
        add_expression_section(&mut gene).await;
    }

    if include.contains(&GeneIncludeType::Hpa) {
        add_hpa_section(&mut gene).await;
    }

    if include.contains(&GeneIncludeType::Druggability) {
        add_druggability_section(&mut gene).await;
    }

    if include.contains(&GeneIncludeType::ClinGen) {
        add_clingen_section(&mut gene).await;
    }

    if include.contains(&GeneIncludeType::Constraint) {
        add_constraint_section(&mut gene).await;
    }

    Ok(gene)
}

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
    }

    #[test]
    fn parse_sections_accepts_new_enrichment_sections() {
        let parsed = parse_sections(&[
            "expression".to_string(),
            "hpa".to_string(),
            "druggability".to_string(),
            "clingen".to_string(),
            "constraint".to_string(),
        ])
        .expect("new gene sections should parse");
        assert_eq!(parsed.len(), 5);
    }

    #[test]
    fn parse_sections_all_includes_new_gene_sections() {
        let parsed = parse_sections(&["all".to_string()]).expect("all should parse");
        assert_eq!(parsed.len(), 12);
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
}
