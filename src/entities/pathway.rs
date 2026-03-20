use std::collections::HashSet;
use std::sync::OnceLock;

use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::error::BioMcpError;
use crate::sources::gprofiler::GProfilerClient;
use crate::sources::kegg::{KeggClient, is_human_pathway_id};
use crate::sources::mygene::MyGeneClient;
use crate::sources::reactome::ReactomeClient;
use crate::sources::wikipathways::{WikiPathwaysClient, is_wikipathways_id};
use crate::transform;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pathway {
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
        "mitogen activated protein kinase" | "mapk pathway" | "mapk signaling" => {
            "MAPK".to_string()
        }
        _ => query.trim().to_string(),
    }
}

fn kegg_disabled() -> bool {
    matches!(
        std::env::var("BIOMCP_DISABLE_KEGG")
            .ok()
            .as_deref()
            .map(str::trim),
        Some("1") | Some("true") | Some("TRUE") | Some("yes") | Some("YES")
    )
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

async fn add_pathway_enrichment(pathway: &mut Pathway, fallback_genes: &[String]) {
    let genes = if !pathway.genes.is_empty() {
        pathway.genes.clone()
    } else {
        fallback_genes.to_vec()
    };
    if genes.is_empty() {
        return;
    }

    let client = match GProfilerClient::new() {
        Ok(client) => client,
        Err(err) => {
            warn!("g:Profiler enrichment unavailable: {err}");
            return;
        }
    };

    match client.enrich_genes(&genes, 10).await {
        Ok(rows) => {
            pathway.enrichment = rows
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
        }
        Err(err) => warn!("g:Profiler enrichment unavailable: {err}"),
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
    let (reactome_hits, reactome_total) = reactome_res?;
    let reactome_hits = reactome_hits
        .into_iter()
        .map(transform::pathway::from_reactome_hit)
        .collect::<Vec<_>>();

    let kegg_hits = match kegg_res {
        Ok(hits) => hits
            .into_iter()
            .map(transform::pathway::from_kegg_hit)
            .collect::<Vec<_>>(),
        Err(err) => {
            warn!("KEGG pathway search unavailable: {err}");
            Vec::new()
        }
    };
    let wikipathways_hits = match wikipathways_res {
        Ok(hits) => hits
            .into_iter()
            .map(transform::pathway::from_wikipathways_hit)
            .collect::<Vec<_>>(),
        Err(err) => {
            warn!("WikiPathways search unavailable: {err}");
            Vec::new()
        }
    };
    let total = if !kegg_hits.is_empty() || !wikipathways_hits.is_empty() {
        None
    } else {
        reactome_total
    };
    Ok((
        rerank_pathway_search_results(
            &effective_query,
            reactome_hits,
            kegg_hits,
            wikipathways_hits,
            limit,
        ),
        total,
    ))
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
        if !parsed_sections.include_genes {
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
            match client.pathway_entrez_gene_ids(&pathway.id).await {
                Ok(entrez_ids) => {
                    let entrez_ids = entrez_ids.into_iter().take(200).collect::<Vec<_>>();
                    if !entrez_ids.is_empty() {
                        match MyGeneClient::new() {
                            Ok(mygene) => match mygene.symbols_for_entrez_ids(&entrez_ids).await {
                                Ok(symbols) => {
                                    pathway.genes = symbols.into_iter().take(50).collect();
                                }
                                Err(err) => warn!(
                                    "WikiPathways gene symbol resolution unavailable via MyGene: {err}"
                                ),
                            },
                            Err(err) => {
                                warn!("WikiPathways gene symbol resolution unavailable: {err}")
                            }
                        }
                    }
                }
                Err(err) => warn!("WikiPathways xref retrieval unavailable: {err}"),
            }
        }
        return Ok(pathway);
    }

    let client = ReactomeClient::new()?;
    let record = client.get_pathway(st_id).await?;

    let mut pathway = transform::pathway::from_reactome_record(record);

    let mut participant_lines: Vec<String> = Vec::new();
    if parsed_sections.include_genes || parsed_sections.include_enrichment {
        participant_lines = match client.participants(&pathway.id, 200).await {
            Ok(lines) => lines,
            Err(err) => {
                warn!("Reactome participants unavailable: {err}");
                Vec::new()
            }
        };
        pathway.genes = extract_gene_symbols(&participant_lines, 50);
    }

    if parsed_sections.include_events {
        pathway.events = match client.contained_events(&pathway.id, 50).await {
            Ok(events) => events,
            Err(err) => {
                warn!("Reactome contained events unavailable: {err}");
                Vec::new()
            }
        };
    }

    if parsed_sections.include_enrichment {
        let fallback_genes = if pathway.genes.is_empty() {
            extract_gene_symbols(&participant_lines, 30)
        } else {
            Vec::new()
        };
        add_pathway_enrichment(&mut pathway, &fallback_genes).await;
    }

    Ok(pathway)
}

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, OnceLock};

    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn env_lock() -> &'static Mutex<()> {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        ENV_LOCK.get_or_init(|| Mutex::new(()))
    }

    struct EnvVarGuard {
        name: &'static str,
        previous: Option<String>,
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            // Safety: tests serialize environment mutation with `env_lock()`.
            unsafe {
                match &self.previous {
                    Some(value) => std::env::set_var(self.name, value),
                    None => std::env::remove_var(self.name),
                }
            }
        }
    }

    fn set_env_var(name: &'static str, value: Option<&str>) -> EnvVarGuard {
        let previous = std::env::var(name).ok();
        // Safety: tests serialize environment mutation with `env_lock()`.
        unsafe {
            match value {
                Some(value) => std::env::set_var(name, value),
                None => std::env::remove_var(name),
            }
        }
        EnvVarGuard { name, previous }
    }

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
    fn rerank_pathway_search_results_floats_exact_match_across_sources() {
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
        assert_eq!(ids, vec!["hsa05200", "R-HSA-9824443", "WP254"]);
    }

    #[test]
    fn rerank_pathway_search_results_uses_upstream_position_for_same_tier() {
        let ranked = rerank_pathway_search_results(
            "MAPK",
            vec![
                PathwaySearchResult {
                    source: "Reactome".to_string(),
                    id: "R-HSA-0002".to_string(),
                    name: "Cell cycle".to_string(),
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

    #[tokio::test]
    async fn search_with_filters_keeps_wikipathways_enabled_when_kegg_is_disabled() {
        let _guard = env_lock().lock().expect("env lock");
        let reactome = MockServer::start().await;
        let wikipathways = MockServer::start().await;
        let _reactome_base = set_env_var("BIOMCP_REACTOME_BASE", Some(&reactome.uri()));
        let _wikipathways_base = set_env_var("BIOMCP_WIKIPATHWAYS_BASE", Some(&wikipathways.uri()));
        let _disable_kegg = set_env_var("BIOMCP_DISABLE_KEGG", Some("1"));

        Mock::given(method("GET"))
            .and(path("/search/query"))
            .and(query_param("query", "apoptosis"))
            .and(query_param("species", "Homo sapiens"))
            .and(query_param("pageSize", "5"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(
                r#"{"results":[{"entries":[{"stId":"R-HSA-109581","name":"Apoptosis"}]}],"totalResults":1}"#,
                "application/json",
            ))
            .expect(1)
            .mount(&reactome)
            .await;

        Mock::given(method("GET"))
            .and(path("/findPathwaysByText"))
            .and(query_param("query", "apoptosis"))
            .and(query_param("organism", "Homo sapiens"))
            .and(query_param("format", "json"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(
                r#"{"result":[{"id":"WP254","name":"Apoptosis","species":"Homo sapiens"}]}"#,
                "application/json",
            ))
            .expect(1)
            .mount(&wikipathways)
            .await;

        let filters = PathwaySearchFilters {
            query: Some("apoptosis".to_string()),
            pathway_type: None,
            top_level: false,
        };
        let (results, total) = search_with_filters(&filters, 5).await.unwrap();

        let ids = results
            .iter()
            .map(|row| row.id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(ids, vec!["R-HSA-109581", "WP254"]);
        assert_eq!(results[1].source, "WikiPathways");
        assert_eq!(total, None);
    }

    #[test]
    fn kegg_disabled_flag_accepts_one() {
        let _guard = env_lock().lock().expect("env lock");
        let _env = set_env_var("BIOMCP_DISABLE_KEGG", Some("1"));
        assert!(kegg_disabled());
    }
}
