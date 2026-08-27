use std::collections::HashSet;
use std::sync::OnceLock;

use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::debug as warn;

use crate::entities::section_outcome::{SectionOutcome, SectionOutcomes};
use crate::entities::source_state_registry::outcome_keys;
use crate::error::BioMcpError;
use crate::sources::gprofiler::GProfilerClient;
use crate::sources::kegg::{KeggClient, is_human_pathway_id};
use crate::sources::mygene::MyGeneClient;
use crate::sources::reactome::ReactomeClient;
use crate::sources::wikipathways::{WikiPathwaysClient, is_wikipathways_id};
use crate::transform;

fn default_pathway_section_outcomes() -> SectionOutcomes {
    SectionOutcomes::with_keys(&outcome_keys("pathway"))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pathway {
    #[serde(
        default = "default_pathway_section_outcomes",
        deserialize_with = "deserialize_pathway_section_outcomes"
    )]
    pub section_outcomes: SectionOutcomes,
    pub source: String,
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub species: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default)]
    pub genes: Vec<String>,
    #[serde(default)]
    pub events: Vec<String>,
    #[serde(default)]
    pub enrichment: Vec<PathwayEnrichment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathwayEnrichment {
    pub source: String,
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p_value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathwaySearchResult {
    pub source: String,
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Default)]
pub struct PathwaySearchFilters {
    pub query: Option<String>,
    pub pathway_type: Option<String>,
    pub top_level: bool,
}

const PATHWAY_SECTION_GENES: &str = "genes";
const PATHWAY_SECTION_EVENTS: &str = "events";
const PATHWAY_SECTION_ENRICHMENT: &str = "enrichment";
const PATHWAY_SECTION_ALL: &str = "all";
pub(crate) const PATHWAY_OUTCOME_KEYS: &[&str] = &[
    PATHWAY_SECTION_GENES,
    PATHWAY_SECTION_EVENTS,
    PATHWAY_SECTION_ENRICHMENT,
];

fn deserialize_pathway_section_outcomes<'de, D>(
    deserializer: D,
) -> Result<SectionOutcomes, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let outcomes = SectionOutcomes::deserialize(deserializer)?;
    outcomes
        .validate_keys(&outcome_keys("pathway"))
        .map_err(serde::de::Error::custom)?;
    Ok(outcomes)
}

pub const PATHWAY_SECTION_NAMES: &[&str] = &[
    PATHWAY_SECTION_GENES,
    PATHWAY_SECTION_EVENTS,
    PATHWAY_SECTION_ENRICHMENT,
    PATHWAY_SECTION_ALL,
];

const REACTOME_PATHWAY_SECTIONS: &[&str] = &[
    PATHWAY_SECTION_GENES,
    PATHWAY_SECTION_EVENTS,
    PATHWAY_SECTION_ENRICHMENT,
];
const KEGG_PATHWAY_SECTIONS: &[&str] = &[PATHWAY_SECTION_GENES];
const WIKIPATHWAYS_PATHWAY_SECTIONS: &[&str] = &[PATHWAY_SECTION_GENES];
const REACTOME_PATHWAY_ENRICHMENT_SOURCE: &str = "REAC";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PathwaySourceKind {
    Reactome,
    Kegg,
    WikiPathways,
}

#[derive(Debug, Clone, Copy, Default)]
struct PathwaySections {
    include_genes: bool,
    include_events: bool,
    include_enrichment: bool,
    include_all: bool,
}

fn parse_sections(sections: &[String]) -> Result<PathwaySections, BioMcpError> {
    let mut out = PathwaySections::default();

    for raw in sections {
        let section = raw.trim().to_ascii_lowercase();
        if section.is_empty() {
            continue;
        }
        if section == "--json" || section == "-j" {
            continue;
        }

        match section.as_str() {
            PATHWAY_SECTION_GENES => out.include_genes = true,
            PATHWAY_SECTION_EVENTS => out.include_events = true,
            PATHWAY_SECTION_ENRICHMENT => out.include_enrichment = true,
            PATHWAY_SECTION_ALL => out.include_all = true,
            _ => {
                return Err(BioMcpError::InvalidArgument(format!(
                    "Unknown section \"{section}\" for pathway. Available: {}",
                    PATHWAY_SECTION_NAMES.join(", ")
                )));
            }
        }
    }

    Ok(out)
}

fn source_kind_for_pathway_id(st_id: &str) -> PathwaySourceKind {
    if is_human_pathway_id(st_id) {
        PathwaySourceKind::Kegg
    } else if is_wikipathways_id(st_id) {
        PathwaySourceKind::WikiPathways
    } else {
        PathwaySourceKind::Reactome
    }
}

fn pathway_lookup_error(st_id: &str, err: BioMcpError) -> BioMcpError {
    let err = match err {
        BioMcpError::WithSourceContext { context, source } => {
            let source_code = source.code();
            let remapped = pathway_lookup_error(st_id, *source);
            return if remapped.code() == source_code {
                remapped.with_source_context(context)
            } else {
                remapped
            };
        }
        other => other,
    };
    let trimmed = st_id.trim();
    let redirect = if crate::entities::protein::is_uniprot_accession(trimmed) {
        Some(format!(
            "`{trimmed}` looks like a UniProt accession — did you mean `biomcp get protein {trimmed}`?"
        ))
    } else if looks_like_ensembl_gene_or_transcript_id(trimmed) {
        Some(format!(
            "`{trimmed}` looks like an Ensembl id — did you mean `biomcp get gene {trimmed}`?"
        ))
    } else if crate::entities::variant::is_rsid(trimmed) {
        Some(format!(
            "`{trimmed}` looks like a dbSNP rsID — did you mean `biomcp get variant {trimmed}`?"
        ))
    } else if crate::entities::gene::looks_like_symbol(trimmed) {
        Some(format!(
            "`{trimmed}` looks like a gene symbol — did you mean `biomcp get gene {trimmed}`?"
        ))
    } else {
        None
    };

    if let Some(redirect) = redirect {
        BioMcpError::InvalidArgument(format!("{err}\n\n{redirect}"))
    } else {
        err
    }
}

fn looks_like_ensembl_gene_or_transcript_id(value: &str) -> bool {
    let Some(suffix) = value
        .strip_prefix("ENSG")
        .or_else(|| value.strip_prefix("ENST"))
    else {
        return false;
    };
    let (stable_part, version_part) = match suffix.split_once('.') {
        Some((stable_part, version_part)) => (stable_part, Some(version_part)),
        None => (suffix, None),
    };
    !stable_part.is_empty()
        && stable_part.chars().all(|c| c.is_ascii_digit())
        && match version_part {
            Some(part) => !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()),
            None => true,
        }
}

fn source_kind_for_pathway_source(source: &str) -> PathwaySourceKind {
    if source.trim().eq_ignore_ascii_case("KEGG") {
        PathwaySourceKind::Kegg
    } else if source.trim().eq_ignore_ascii_case("WikiPathways") {
        PathwaySourceKind::WikiPathways
    } else {
        PathwaySourceKind::Reactome
    }
}

fn source_label(kind: PathwaySourceKind) -> &'static str {
    match kind {
        PathwaySourceKind::Reactome => "Reactome",
        PathwaySourceKind::Kegg => "KEGG",
        PathwaySourceKind::WikiPathways => "WikiPathways",
    }
}

pub(crate) fn supported_pathway_sections_for_source(source: &str) -> &'static [&'static str] {
    match source_kind_for_pathway_source(source) {
        PathwaySourceKind::Reactome => REACTOME_PATHWAY_SECTIONS,
        PathwaySourceKind::Kegg => KEGG_PATHWAY_SECTIONS,
        PathwaySourceKind::WikiPathways => WIKIPATHWAYS_PATHWAY_SECTIONS,
    }
}

fn supported_pathway_sections_for_id(st_id: &str) -> &'static [&'static str] {
    match source_kind_for_pathway_id(st_id) {
        PathwaySourceKind::Reactome => REACTOME_PATHWAY_SECTIONS,
        PathwaySourceKind::Kegg => KEGG_PATHWAY_SECTIONS,
        PathwaySourceKind::WikiPathways => WIKIPATHWAYS_PATHWAY_SECTIONS,
    }
}

fn unsupported_pathway_section_error(section: &str, source: PathwaySourceKind) -> BioMcpError {
    let source = source_label(source);
    BioMcpError::InvalidArgument(format!(
        "pathway section \"{section}\" is not available for {source} pathways. \
Use a Reactome pathway ID such as R-HSA-5673001: biomcp get pathway R-HSA-5673001 {section}"
    ))
}

fn resolve_sections_for_pathway_id(
    st_id: &str,
    raw_sections: &[String],
) -> Result<PathwaySections, BioMcpError> {
    let mut resolved = parse_sections(raw_sections)?;
    let source = source_kind_for_pathway_id(st_id);
    let supported = supported_pathway_sections_for_id(st_id);

    for raw in raw_sections {
        let section = raw.trim();
        if section.is_empty()
            || section.eq_ignore_ascii_case("--json")
            || section.eq_ignore_ascii_case("-j")
        {
            continue;
        }
        if section.eq_ignore_ascii_case(PATHWAY_SECTION_ALL) {
            continue;
        }
        if !supported
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(section))
        {
            return Err(unsupported_pathway_section_error(section, source));
        }
    }

    if resolved.include_all {
        resolved.include_genes = supported
            .iter()
            .any(|section| section.eq_ignore_ascii_case(PATHWAY_SECTION_GENES));
        resolved.include_events = supported
            .iter()
            .any(|section| section.eq_ignore_ascii_case(PATHWAY_SECTION_EVENTS));
        resolved.include_enrichment = supported
            .iter()
            .any(|section| section.eq_ignore_ascii_case(PATHWAY_SECTION_ENRICHMENT));
    }

    Ok(resolved)
}

fn gene_token_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b[A-Z][A-Z0-9]{1,9}\b").expect("valid regex"))
}

fn aa_substitution_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[A-Z]\d{1,5}[A-Z*]$").expect("valid regex"))
}

fn residue_site_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[STY]\d{1,5}$").expect("valid regex"))
}

fn looks_like_gene_symbol(token: &str) -> bool {
    let token = token.trim();
    if token.len() < 2 || token.as_bytes().first().is_some_and(|b| b.is_ascii_digit()) {
        return false;
    }
    if aa_substitution_re().is_match(token) || residue_site_re().is_match(token) {
        return false;
    }
    true
}

fn family_gene_expansion(token: &str) -> Option<&'static [&'static str]> {
    match token {
        "RAS" => Some(&["HRAS", "KRAS", "NRAS"]),
        "RAF" | "RAFS" => Some(&["ARAF", "BRAF", "RAF1"]),
        "MAP2K" => Some(&["MAP2K1", "MAP2K2"]),
        "MAPK" => Some(&["MAPK1", "MAPK3", "MAPK8", "MAPK9", "MAPK14"]),
        "SPRED" => Some(&["SPRED1", "SPRED2", "SPRED3"]),
        "GAP" => Some(&["NF1", "RASA1", "RASA2"]),
        "PP1" => Some(&["PPP1CA", "PPP1CB", "PPP1CC"]),
        _ => None,
    }
}

fn is_generic_family_token(token: &str) -> bool {
    matches!(
        token,
        "RAS" | "RAF" | "RAFS" | "MAP2K" | "MAPK" | "SPRED" | "GAP" | "PP1"
    )
}

fn extract_gene_symbols(lines: &[String], limit: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();

    for line in lines {
        for cap in gene_token_re().find_iter(line) {
            let gene = cap.as_str().trim();
            if gene.is_empty() || !looks_like_gene_symbol(gene) {
                continue;
            }
            if ["ATP", "ADP", "GDP", "GTP", "DNA", "RNA", "H2O", "PI"]
                .iter()
                .any(|v| v == &gene)
            {
                continue;
            }

            if let Some(expanded) = family_gene_expansion(gene) {
                for mapped in expanded {
                    if !seen.insert((*mapped).to_string()) {
                        continue;
                    }
                    out.push((*mapped).to_string());
                    if out.len() >= limit {
                        return out;
                    }
                }
                continue;
            }
            if is_generic_family_token(gene) {
                continue;
            }

            if !seen.insert(gene.to_string()) {
                continue;
            }
            out.push(gene.to_string());
            if out.len() >= limit {
                return out;
            }
        }
    }

    out
}

fn normalize_pathway_query(query: &str) -> String {
    let normalized = query.trim().to_ascii_lowercase().replace(['-', '_'], " ");

    match normalized.as_str() {
        "mitogen activated protein kinase signaling pathway" => {
            "MAPK signaling pathway".to_string()
        }
        "mitogen activated protein kinase" | "mapk pathway" | "mapk signaling" => {
            "MAPK".to_string()
        }
        _ => query.trim().to_string(),
    }
}

fn kegg_disabled_from_env_value(value: Option<&str>) -> bool {
    matches!(
        value.map(str::trim),
        Some("1") | Some("true") | Some("TRUE") | Some("yes") | Some("YES")
    )
}

fn kegg_disabled() -> bool {
    kegg_disabled_from_env_value(std::env::var("BIOMCP_DISABLE_KEGG").ok().as_deref())
}

fn kegg_disabled_error(pathway_id: &str) -> BioMcpError {
    BioMcpError::SourceUnavailable {
        source_name: "kegg".to_string(),
        reason: format!(
            "KEGG pathway access for {pathway_id} is disabled by BIOMCP_DISABLE_KEGG=1."
        ),
        suggestion:
            "Unset BIOMCP_DISABLE_KEGG or query a Reactome pathway ID such as R-HSA-5673001."
                .to_string(),
    }
}

fn normalize_pathway_match_text(value: &str) -> String {
    value
        .split_ascii_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

fn pathway_title_match_tier(name: &str, query: &str) -> u8 {
    let normalized_name = normalize_pathway_match_text(name);
    let normalized_query = normalize_pathway_match_text(query);
    if normalized_name.is_empty() || normalized_query.is_empty() {
        return 0;
    }
    if normalized_name == normalized_query {
        return 3;
    }
    if normalized_name.starts_with(&normalized_query) {
        return 2;
    }
    if normalized_name.contains(&normalized_query) {
        return 1;
    }
    0
}

fn rerank_pathway_search_results(
    query: &str,
    reactome_hits: Vec<PathwaySearchResult>,
    kegg_hits: Vec<PathwaySearchResult>,
    wikipathways_hits: Vec<PathwaySearchResult>,
    limit: usize,
) -> Vec<PathwaySearchResult> {
    let mut seen = HashSet::new();
    let mut ranked = Vec::new();

    push_ranked_hits(query, reactome_hits, &mut seen, &mut ranked);
    push_ranked_hits(query, kegg_hits, &mut seen, &mut ranked);
    push_ranked_hits(query, wikipathways_hits, &mut seen, &mut ranked);

    ranked.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| a.1.cmp(&b.1))
            .then_with(|| a.2.cmp(&b.2))
    });
    ranked.truncate(limit);
    ranked.into_iter().map(|(_, _, _, row)| row).collect()
}

fn push_ranked_hits(
    query: &str,
    hits: Vec<PathwaySearchResult>,
    seen: &mut HashSet<String>,
    ranked: &mut Vec<(u8, usize, String, PathwaySearchResult)>,
) {
    for (upstream_idx, row) in hits.into_iter().enumerate() {
        let source = row.source.trim().to_string();
        let id = row.id.trim().to_string();
        let name = row.name.trim().to_string();
        if source.is_empty() || id.is_empty() || name.is_empty() {
            continue;
        }

        let dedupe_key = format!(
            "{}:{}",
            source.to_ascii_lowercase(),
            id.to_ascii_lowercase()
        );
        if !seen.insert(dedupe_key) {
            continue;
        }

        ranked.push((
            pathway_title_match_tier(&name, query),
            upstream_idx,
            id.clone(),
            PathwaySearchResult { source, id, name },
        ));
    }
}

#[derive(Debug, Default)]
struct PathwaySearchSourceResults {
    reactome_hits: Vec<PathwaySearchResult>,
    reactome_total: Option<usize>,
    reactome_error: Option<BioMcpError>,
    kegg_hits: Vec<PathwaySearchResult>,
    kegg_error: Option<BioMcpError>,
    wikipathways_hits: Vec<PathwaySearchResult>,
    wikipathways_error: Option<BioMcpError>,
}

fn finalize_pathway_search_results(
    query: &str,
    limit: usize,
    source_results: PathwaySearchSourceResults,
) -> Result<(Vec<PathwaySearchResult>, Option<usize>), BioMcpError> {
    let PathwaySearchSourceResults {
        reactome_hits,
        reactome_total,
        reactome_error,
        kegg_hits,
        kegg_error,
        wikipathways_hits,
        wikipathways_error,
    } = source_results;

    if reactome_hits.is_empty()
        && kegg_hits.is_empty()
        && wikipathways_hits.is_empty()
        && let Some(err) = reactome_error.or(kegg_error).or(wikipathways_error)
    {
        return Err(err);
    }

    let total = if !kegg_hits.is_empty() || !wikipathways_hits.is_empty() {
        None
    } else {
        reactome_total
    };

    Ok((
        rerank_pathway_search_results(query, reactome_hits, kegg_hits, wikipathways_hits, limit),
        total,
    ))
}

fn wikipathways_gene_outcome(symbols: &[String]) -> SectionOutcome {
    if symbols.is_empty() {
        SectionOutcome::empty_sources(["WikiPathways", "MyGene.info"])
    } else {
        SectionOutcome::data_sources(["WikiPathways", "MyGene.info"])
    }
}

async fn add_pathway_enrichment(
    pathway: &mut Pathway,
    fallback_genes: &[String],
) -> SectionOutcome {
    let genes = if !pathway.genes.is_empty() {
        pathway.genes.clone()
    } else {
        fallback_genes.to_vec()
    };
    if genes.is_empty() {
        return SectionOutcome::empty("Reactome");
    }

    let client = match GProfilerClient::new() {
        Ok(client) => client,
        Err(_) => {
            return SectionOutcome::unavailable("Pathway enrichment is unavailable.");
        }
    };

    match client.enrich_genes(&genes, 10).await {
        Ok(enrichment) => {
            pathway.enrichment = enrichment
                .terms
                .into_iter()
                .filter_map(|r| {
                    Some(PathwayEnrichment {
                        source: r.source?.trim().to_string(),
                        id: r.native?.trim().to_string(),
                        name: r.name?.trim().to_string(),
                        p_value: r.p_value,
                    })
                })
                .filter(|r| !r.source.is_empty() && !r.id.is_empty() && !r.name.is_empty())
                .filter(|r| {
                    r.source
                        .eq_ignore_ascii_case(REACTOME_PATHWAY_ENRICHMENT_SOURCE)
                })
                .collect();
            if pathway.enrichment.is_empty() {
                SectionOutcome::empty("g:Profiler")
            } else {
                SectionOutcome::data("g:Profiler")
            }
        }
        Err(_) => SectionOutcome::unavailable("Pathway enrichment is unavailable."),
    }
}

pub fn search_query_summary(filters: &PathwaySearchFilters) -> String {
    let mut parts = Vec::new();
    if let Some(query) = filters
        .query
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(query.to_string());
    }
    if let Some(pathway_type) = filters
        .pathway_type
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("type={pathway_type}"));
    }
    if filters.top_level {
        parts.push("top_level=true".to_string());
    }
    parts.join(", ")
}

pub async fn search_with_filters(
    filters: &PathwaySearchFilters,
    limit: usize,
) -> Result<(Vec<PathwaySearchResult>, Option<usize>), BioMcpError> {
    let limit = limit.clamp(1, 25);
    let query = filters
        .query
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());
    let pathway_type = filters
        .pathway_type
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());
    if let Some(pathway_type) = pathway_type
        && !pathway_type.eq_ignore_ascii_case("pathway")
    {
        return Err(BioMcpError::InvalidArgument(
            "--type currently supports only: pathway".into(),
        ));
    }
    if !filters.top_level && query.is_none() {
        return Err(BioMcpError::InvalidArgument(
            "Query is required. Example: biomcp search pathway -q \"MAPK signaling\"".into(),
        ));
    }

    let client = ReactomeClient::new()?;
    if filters.top_level {
        let mut hits = client.top_level_pathways(limit).await?;
        if let Some(query) = query {
            let query_lower = query.to_ascii_lowercase();
            hits.retain(|row| row.name.to_ascii_lowercase().contains(&query_lower));
        }
        return Ok((
            hits.into_iter()
                .map(transform::pathway::from_reactome_hit)
                .collect(),
            None,
        ));
    }

    let effective_query = normalize_pathway_query(query.unwrap_or_default());
    let wikipathways = WikiPathwaysClient::new()?;

    let (reactome_res, kegg_res, wikipathways_res) = if kegg_disabled() {
        warn!("KEGG pathway search disabled by BIOMCP_DISABLE_KEGG=1");
        let (reactome_res, wikipathways_res) = tokio::join!(
            client.search_pathways(&effective_query, limit),
            wikipathways.search_pathways(&effective_query, limit)
        );
        (reactome_res, Ok(Vec::new()), wikipathways_res)
    } else {
        let kegg = KeggClient::new()?;
        tokio::join!(
            client.search_pathways(&effective_query, limit),
            kegg.search_pathways(&effective_query, limit),
            wikipathways.search_pathways(&effective_query, limit)
        )
    };
    let (reactome_hits, reactome_total, reactome_error) = match reactome_res {
        Ok((hits, total)) => (
            hits.into_iter()
                .map(transform::pathway::from_reactome_hit)
                .collect::<Vec<_>>(),
            total,
            None,
        ),
        Err(err) => {
            warn!("Reactome pathway search unavailable: {err}");
            (Vec::new(), None, Some(err))
        }
    };

    let (kegg_hits, kegg_error) = match kegg_res {
        Ok(hits) => (
            hits.into_iter()
                .map(transform::pathway::from_kegg_hit)
                .collect::<Vec<_>>(),
            None,
        ),
        Err(err) => {
            warn!("KEGG pathway search unavailable: {err}");
            (Vec::new(), Some(err))
        }
    };
    let (wikipathways_hits, wikipathways_error) = match wikipathways_res {
        Ok(hits) => (
            hits.into_iter()
                .map(transform::pathway::from_wikipathways_hit)
                .collect::<Vec<_>>(),
            None,
        ),
        Err(err) => {
            warn!("WikiPathways search unavailable: {err}");
            (Vec::new(), Some(err))
        }
    };
    finalize_pathway_search_results(
        &effective_query,
        limit,
        PathwaySearchSourceResults {
            reactome_hits,
            reactome_total,
            reactome_error,
            kegg_hits,
            kegg_error,
            wikipathways_hits,
            wikipathways_error,
        },
    )
}

pub async fn get(st_id: &str, sections: &[String]) -> Result<Pathway, BioMcpError> {
    let st_id = st_id.trim();
    if st_id.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Pathway ID is required. Example: biomcp get pathway R-HSA-5673001".into(),
        ));
    }

    let parsed_sections = resolve_sections_for_pathway_id(st_id, sections)?;
    if matches!(source_kind_for_pathway_id(st_id), PathwaySourceKind::Kegg) {
        if kegg_disabled() {
            return Err(kegg_disabled_error(st_id));
        }

        let record = KeggClient::new()?.get_pathway(st_id).await?;
        let mut pathway = transform::pathway::from_kegg_record(record);
        if parsed_sections.include_genes {
            pathway.section_outcomes.complete(
                PATHWAY_SECTION_GENES,
                if pathway.genes.is_empty() {
                    SectionOutcome::empty("KEGG")
                } else {
                    SectionOutcome::data("KEGG")
                },
            );
        } else {
            pathway.genes.clear();
        }
        return Ok(pathway);
    }

    if matches!(
        source_kind_for_pathway_id(st_id),
        PathwaySourceKind::WikiPathways
    ) {
        let client = WikiPathwaysClient::new()?;
        let record = client.get_pathway(st_id).await?;
        let mut pathway = transform::pathway::from_wikipathways_record(record);
        if parsed_sections.include_genes {
            let outcome = match client.pathway_entrez_gene_ids(&pathway.id).await {
                Ok(entrez_ids) => {
                    let entrez_ids = entrez_ids.into_iter().take(200).collect::<Vec<_>>();
                    if entrez_ids.is_empty() {
                        SectionOutcome::empty("WikiPathways")
                    } else {
                        match MyGeneClient::new() {
                            Ok(mygene) => match mygene.symbols_for_entrez_ids(&entrez_ids).await {
                                Ok(symbols) => {
                                    pathway.genes = symbols.into_iter().take(50).collect();
                                    wikipathways_gene_outcome(&pathway.genes)
                                }
                                Err(_) => SectionOutcome::unavailable(
                                    "WikiPathways genes are unavailable.",
                                ),
                            },
                            Err(_) => {
                                SectionOutcome::unavailable("WikiPathways genes are unavailable.")
                            }
                        }
                    }
                }
                Err(_) => SectionOutcome::unavailable("WikiPathways genes are unavailable."),
            };
            pathway
                .section_outcomes
                .complete(PATHWAY_SECTION_GENES, outcome);
        }
        return Ok(pathway);
    }

    let client = ReactomeClient::new()?;
    let record = client
        .get_pathway(st_id)
        .await
        .map_err(|err| pathway_lookup_error(st_id, err))?;

    let mut pathway = transform::pathway::from_reactome_record(record);

    let mut participant_lines: Vec<String> = Vec::new();
    let mut participants_available = true;
    if parsed_sections.include_genes || parsed_sections.include_enrichment {
        match client.participants(&pathway.id, 200).await {
            Ok(lines) => participant_lines = lines,
            Err(_) => participants_available = false,
        }
        pathway.genes = extract_gene_symbols(&participant_lines, 50);
        if parsed_sections.include_genes || parsed_sections.include_enrichment {
            pathway.section_outcomes.complete(
                PATHWAY_SECTION_GENES,
                if !participants_available {
                    SectionOutcome::unavailable("Reactome pathway genes are unavailable.")
                } else if pathway.genes.is_empty() {
                    SectionOutcome::empty("Reactome")
                } else {
                    SectionOutcome::data("Reactome")
                },
            );
        }
    }

    if parsed_sections.include_events {
        match client.contained_events(&pathway.id, 50).await {
            Ok(events) => {
                pathway.section_outcomes.complete(
                    PATHWAY_SECTION_EVENTS,
                    if events.is_empty() {
                        SectionOutcome::empty("Reactome")
                    } else {
                        SectionOutcome::data("Reactome")
                    },
                );
                pathway.events = events;
            }
            Err(_) => {
                pathway.section_outcomes.complete(
                    PATHWAY_SECTION_EVENTS,
                    SectionOutcome::unavailable("Reactome contained events are unavailable."),
                );
            }
        }
    }

    if parsed_sections.include_enrichment {
        let outcome = if participants_available {
            let fallback_genes = if pathway.genes.is_empty() {
                extract_gene_symbols(&participant_lines, 30)
            } else {
                Vec::new()
            };
            add_pathway_enrichment(&mut pathway, &fallback_genes).await
        } else {
            SectionOutcome::unavailable("Pathway enrichment is unavailable.")
        };
        pathway
            .section_outcomes
            .complete(PATHWAY_SECTION_ENRICHMENT, outcome);
    }

    Ok(pathway)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sections_supports_all_and_rejects_unknown_values() {
        let flags = parse_sections(&["all".to_string()]).unwrap();
        assert!(flags.include_all);
        assert!(!flags.include_genes);
        assert!(!flags.include_events);
        assert!(!flags.include_enrichment);

        let err = parse_sections(&["bad".to_string()]).unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    }

    fn pathway_lookup_error_message(st_id: &str) -> String {
        pathway_lookup_error(
            st_id,
            BioMcpError::NotFound {
                entity: "pathway".to_string(),
                id: st_id.to_string(),
                suggestion: format!("Try searching: biomcp search pathway -q {st_id}"),
            },
        )
        .to_string()
    }

    #[test]
    fn pathway_lookup_error_adds_protein_redirect_for_uniprot_accession() {
        let message = pathway_lookup_error_message("P21964-2");
        assert!(message.contains("looks like a UniProt accession"));
        assert!(message.contains("biomcp get protein P21964-2"));
    }

    #[test]
    fn pathway_lookup_error_adds_gene_redirect_for_ensembl_gene_id() {
        let message = pathway_lookup_error_message("ENSG00000157764");
        assert!(message.contains("looks like an Ensembl id"));
        assert!(message.contains("biomcp get gene ENSG00000157764"));
    }

    #[test]
    fn pathway_lookup_error_adds_gene_redirect_for_ensembl_transcript_id() {
        let message = pathway_lookup_error_message("ENST00000646891");
        assert!(message.contains("looks like an Ensembl id"));
        assert!(message.contains("biomcp get gene ENST00000646891"));
    }

    #[test]
    fn pathway_lookup_error_adds_gene_redirect_for_versioned_ensembl_id() {
        let message = pathway_lookup_error_message("ENSG00000157764.13");
        assert!(message.contains("looks like an Ensembl id"));
        assert!(message.contains("biomcp get gene ENSG00000157764.13"));
    }

    #[test]
    fn ensembl_redirect_matcher_rejects_malformed_version_suffix() {
        assert!(!looks_like_ensembl_gene_or_transcript_id(
            "ENSG00000157764."
        ));
        assert!(!looks_like_ensembl_gene_or_transcript_id(
            "ENSG00000157764.x"
        ));
    }

    #[test]
    fn pathway_lookup_error_adds_gene_redirect_for_gene_symbol() {
        let message = pathway_lookup_error_message("BRAF");
        assert!(message.contains("looks like a gene symbol"));
        assert!(message.contains("biomcp get gene BRAF"));
    }

    #[test]
    fn pathway_lookup_error_adds_variant_redirect_for_rsid() {
        let message = pathway_lookup_error_message("rs113488022");
        assert!(message.contains("looks like a dbSNP rsID"));
        assert!(message.contains("biomcp get variant rs113488022"));
    }

    #[test]
    fn pathway_lookup_error_preserves_redirect_without_duplicate_source_recovery() {
        let error = pathway_lookup_error(
            "BRAF",
            BioMcpError::NotFound {
                entity: "pathway".to_string(),
                id: "BRAF".to_string(),
                suggestion: "search pathway".to_string(),
            }
            .with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::REACTOME,
            )),
        );

        assert_eq!(error.code(), "invalid_argument");
        assert_eq!(error.exit_code(), 2);
        assert_eq!(error.public_projection().source, None);
        let message = error.to_string();
        assert!(message.contains("biomcp get gene BRAF"));
        assert!(!message.contains("Retry the remote source"));
    }

    #[test]
    fn kegg_explicit_events_section_is_rejected() {
        let err = resolve_sections_for_pathway_id("hsa05200", &["events".to_string()])
            .expect_err("KEGG events should fail fast");
        let message = err.to_string();
        assert!(message.contains("events"));
        assert!(message.contains("KEGG"));
        assert!(message.contains("Reactome"));
        assert!(message.contains("R-HSA-5673001"));
    }

    #[test]
    fn kegg_explicit_enrichment_section_is_rejected() {
        let err = resolve_sections_for_pathway_id("hsa05200", &["enrichment".to_string()])
            .expect_err("KEGG enrichment should fail fast");
        let message = err.to_string();
        assert!(message.contains("enrichment"));
        assert!(message.contains("KEGG"));
        assert!(message.contains("Reactome"));
        assert!(message.contains("R-HSA-5673001"));
    }

    #[test]
    fn kegg_all_expands_to_supported_sections_only() {
        let flags = resolve_sections_for_pathway_id("hsa05200", &["all".to_string()])
            .expect("KEGG all should remain valid");
        assert!(flags.include_genes);
        assert!(!flags.include_events);
        assert!(!flags.include_enrichment);
    }

    #[test]
    fn wikipathways_explicit_events_section_is_rejected() {
        let err = resolve_sections_for_pathway_id("WP254", &["events".to_string()])
            .expect_err("WikiPathways events should fail fast");
        let message = err.to_string();
        assert!(message.contains("events"));
        assert!(message.contains("WikiPathways"));
        assert!(message.contains("Reactome"));
        assert!(message.contains("R-HSA-5673001"));
    }

    #[test]
    fn wikipathways_explicit_enrichment_section_is_rejected() {
        let err = resolve_sections_for_pathway_id("WP254", &["enrichment".to_string()])
            .expect_err("WikiPathways enrichment should fail fast");
        let message = err.to_string();
        assert!(message.contains("enrichment"));
        assert!(message.contains("WikiPathways"));
        assert!(message.contains("Reactome"));
        assert!(message.contains("R-HSA-5673001"));
    }

    #[test]
    fn wikipathways_all_expands_to_supported_sections_only() {
        let flags = resolve_sections_for_pathway_id("WP254", &["all".to_string()])
            .expect("WikiPathways all should remain valid");
        assert!(flags.include_genes);
        assert!(!flags.include_events);
        assert!(!flags.include_enrichment);
    }

    #[test]
    fn pathway_json_compatibility_defaults_missing_outcomes_and_rejects_foreign_keys() {
        let legacy = serde_json::from_str::<Pathway>(
            r#"{"source":"Reactome","id":"R-HSA-1","name":"Example"}"#,
        )
        .expect("legacy pathway JSON should deserialize");
        assert_eq!(legacy.section_outcomes.iter().count(), 3);
        assert!(legacy.section_outcomes.iter().all(|(_, outcome)| {
            outcome.outcome() == crate::entities::section_outcome::SectionOutcomeState::NotRequested
        }));

        let foreign = serde_json::from_str::<Pathway>(
            r#"{"source":"Reactome","id":"R-HSA-1","name":"Example","section_outcomes":{"approvals":{"outcome":"empty","sources":["OpenFDA"]}}}"#,
        );
        assert!(foreign.is_err());
    }

    #[test]
    fn wikipathways_gene_outcome_credits_symbol_resolution_provider() {
        let data = wikipathways_gene_outcome(&["BRAF".to_string()]);
        assert_eq!(
            data.outcome(),
            crate::entities::section_outcome::SectionOutcomeState::Data
        );
        assert_eq!(data.sources(), &["WikiPathways", "MyGene.info"]);

        let empty = wikipathways_gene_outcome(&[]);
        assert_eq!(
            empty.outcome(),
            crate::entities::section_outcome::SectionOutcomeState::Empty
        );
        assert_eq!(empty.sources(), &["WikiPathways", "MyGene.info"]);
    }

    #[tokio::test]
    async fn search_requires_query_with_quoted_example() {
        let filters = PathwaySearchFilters {
            query: None,
            pathway_type: None,
            top_level: false,
        };
        let err = search_with_filters(&filters, 5)
            .await
            .expect_err("missing query should fail before any source call");
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
        assert!(
            err.to_string().contains(
                "Query is required. Example: biomcp search pathway -q \"MAPK signaling\""
            )
        );
    }

    #[test]
    fn extract_gene_symbols_dedupes_and_filters_non_gene_tokens() {
        let lines = vec![
            "BRAF and KRAS activate MAPK".to_string(),
            "ATP GDP BRAF V600E S338".to_string(),
            "EGFR".to_string(),
        ];
        let genes = extract_gene_symbols(&lines, 10);
        assert_eq!(
            genes,
            vec![
                "BRAF".to_string(),
                "KRAS".to_string(),
                "MAPK1".to_string(),
                "MAPK3".to_string(),
                "MAPK8".to_string(),
                "MAPK9".to_string(),
                "MAPK14".to_string(),
                "EGFR".to_string()
            ]
        );
    }

    #[test]
    fn looks_like_gene_symbol_rejects_mutation_notation() {
        assert!(!looks_like_gene_symbol("V600E"));
        assert!(!looks_like_gene_symbol("S338"));
        assert!(looks_like_gene_symbol("MAP2K1"));
    }

    #[test]
    fn normalize_pathway_query_maps_confirmed_mapk_aliases() {
        assert_eq!(
            normalize_pathway_query("mitogen activated protein kinase signaling pathway"),
            "MAPK signaling pathway"
        );
        assert_eq!(
            normalize_pathway_query("mitogen activated protein kinase"),
            "MAPK"
        );
        assert_eq!(normalize_pathway_query("mapk signaling"), "MAPK");
        assert_eq!(
            normalize_pathway_query("oxidative phosphorylation"),
            "oxidative phosphorylation"
        );
    }

    #[test]
    fn pathway_title_match_tier_prefers_exact_then_prefix_then_contains() {
        assert!(
            pathway_title_match_tier("Pathways in cancer", "Pathways in cancer")
                > pathway_title_match_tier("Pathways in cancer and immunity", "Pathways in cancer")
        );
        assert!(
            pathway_title_match_tier("Pathways in cancer and immunity", "Pathways in cancer")
                > pathway_title_match_tier(
                    "Human Pathways in cancer overview",
                    "Pathways in cancer"
                )
        );
        assert!(
            pathway_title_match_tier("Human Pathways in cancer overview", "Pathways in cancer")
                > pathway_title_match_tier("Cell cycle", "Pathways in cancer")
        );
    }

    #[test]
    fn rerank_pathway_search_results_drops_rows_unrelated_to_query() {
        let ranked = rerank_pathway_search_results(
            "Pathways in cancer",
            vec![PathwaySearchResult {
                source: "Reactome".to_string(),
                id: "R-HSA-9824443".to_string(),
                name: "Parasitic Infection Pathways".to_string(),
            }],
            vec![PathwaySearchResult {
                source: "KEGG".to_string(),
                id: "hsa05200".to_string(),
                name: "Pathways in cancer".to_string(),
            }],
            vec![PathwaySearchResult {
                source: "WikiPathways".to_string(),
                id: "WP254".to_string(),
                name: "Pathway Commons".to_string(),
            }],
            5,
        );

        let ids = ranked.iter().map(|row| row.id.as_str()).collect::<Vec<_>>();
        assert_eq!(ids, vec!["hsa05200"]);
    }

    #[test]
    fn rerank_pathway_search_results_uses_upstream_position_for_same_tier() {
        let ranked = rerank_pathway_search_results(
            "MAPK",
            vec![
                PathwaySearchResult {
                    source: "Reactome".to_string(),
                    id: "R-HSA-0002".to_string(),
                    name: "Cell cycle MAPK".to_string(),
                },
                PathwaySearchResult {
                    source: "Reactome".to_string(),
                    id: "R-HSA-0003".to_string(),
                    name: "MAPK adaptor proteins".to_string(),
                },
            ],
            vec![PathwaySearchResult {
                source: "KEGG".to_string(),
                id: "hsa04010".to_string(),
                name: "MAPK signaling pathway".to_string(),
            }],
            vec![PathwaySearchResult {
                source: "WikiPathways".to_string(),
                id: "WP382".to_string(),
                name: "MAPK cascade".to_string(),
            }],
            5,
        );

        let ids = ranked.iter().map(|row| row.id.as_str()).collect::<Vec<_>>();
        assert_eq!(ids, vec!["WP382", "hsa04010", "R-HSA-0003", "R-HSA-0002"]);
    }

    #[test]
    fn finalize_pathway_search_results_keeps_wikipathways_when_kegg_is_disabled() {
        let (results, total) = finalize_pathway_search_results(
            "apoptosis",
            5,
            PathwaySearchSourceResults {
                reactome_hits: vec![PathwaySearchResult {
                    source: "Reactome".to_string(),
                    id: "R-HSA-109581".to_string(),
                    name: "Apoptosis".to_string(),
                }],
                reactome_total: Some(1),
                wikipathways_hits: vec![PathwaySearchResult {
                    source: "WikiPathways".to_string(),
                    id: "WP254".to_string(),
                    name: "Apoptosis".to_string(),
                }],
                ..Default::default()
            },
        )
        .unwrap();

        let ids = results
            .iter()
            .map(|row| row.id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(ids, vec!["R-HSA-109581", "WP254"]);
        assert_eq!(results[1].source, "WikiPathways");
        assert_eq!(total, None);
    }

    #[test]
    fn finalize_pathway_search_results_surfaces_partial_source_failure() {
        let result = finalize_pathway_search_results(
            "apoptosis",
            5,
            PathwaySearchSourceResults {
                reactome_error: Some(BioMcpError::Api {
                    api: "reactome".to_string(),
                    message: "HTTP 504".to_string(),
                }),
                wikipathways_hits: vec![PathwaySearchResult {
                    source: "WikiPathways".to_string(),
                    id: "WP254".to_string(),
                    name: "Apoptosis".to_string(),
                }],
                ..Default::default()
            },
        );

        let error = result.expect_err("a partial answer must report its missing source");
        assert!(
            error.to_string().contains("HTTP 504"),
            "partial failure should preserve the source reason"
        );
    }

    #[test]
    fn kegg_disabled_flag_parsing_accepts_expected_values() {
        assert!(kegg_disabled_from_env_value(Some("1")));
        assert!(kegg_disabled_from_env_value(Some("true")));
        assert!(kegg_disabled_from_env_value(Some("TRUE")));
        assert!(kegg_disabled_from_env_value(Some("yes")));
        assert!(kegg_disabled_from_env_value(Some("YES")));
        assert!(!kegg_disabled_from_env_value(Some("0")));
        assert!(!kegg_disabled_from_env_value(None));
    }

    #[test]
    fn kegg_disabled_error_is_actionable() {
        let err = kegg_disabled_error("hsa05200");
        let msg = err.to_string();
        assert!(msg.contains("KEGG"));
        assert!(msg.to_ascii_lowercase().contains("retry"));
        assert!(!msg.contains("BIOMCP_DISABLE_KEGG=1"));
        assert!(!msg.contains("R-HSA-5673001"));
    }
}
