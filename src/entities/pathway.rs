use std::collections::HashSet;
use std::sync::OnceLock;

use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::error::BioMcpError;
use crate::sources::gprofiler::GProfilerClient;
use crate::sources::reactome::ReactomeClient;
use crate::transform;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pathway {
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

#[derive(Debug, Clone, Copy, Default)]
struct PathwaySections {
    include_genes: bool,
    include_events: bool,
    include_enrichment: bool,
}

fn parse_sections(sections: &[String]) -> Result<PathwaySections, BioMcpError> {
    let mut out = PathwaySections::default();
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
            PATHWAY_SECTION_GENES => out.include_genes = true,
            PATHWAY_SECTION_EVENTS => out.include_events = true,
            PATHWAY_SECTION_ENRICHMENT => out.include_enrichment = true,
            PATHWAY_SECTION_ALL => include_all = true,
            _ => {
                return Err(BioMcpError::InvalidArgument(format!(
                    "Unknown section \"{section}\" for pathway. Available: {}",
                    PATHWAY_SECTION_NAMES.join(", ")
                )));
            }
        }
    }

    if include_all {
        out.include_genes = true;
        out.include_events = true;
        out.include_enrichment = true;
    }

    Ok(out)
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
            "Query is required. Example: biomcp search pathway -q MAPK signaling".into(),
        ));
    }

    let client = ReactomeClient::new()?;
    if filters.top_level {
        let mut hits = client.top_level_pathways(limit.clamp(1, 25)).await?;
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

    let (hits, total) = client
        .search_pathways(query.unwrap_or_default(), limit.clamp(1, 25))
        .await?;
    Ok((
        hits.into_iter()
            .map(transform::pathway::from_reactome_hit)
            .collect(),
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

    let parsed_sections = parse_sections(sections)?;
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
        let genes = if !pathway.genes.is_empty() {
            pathway.genes.clone()
        } else {
            extract_gene_symbols(&participant_lines, 30)
        };

        if !genes.is_empty() {
            match GProfilerClient::new()?.enrich_genes(&genes, 10).await {
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
                        .collect();
                }
                Err(err) => warn!("g:Profiler enrichment unavailable: {err}"),
            }
        }
    }

    Ok(pathway)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sections_supports_all_and_rejects_unknown_values() {
        let flags = parse_sections(&["all".to_string()]).unwrap();
        assert!(flags.include_genes);
        assert!(flags.include_events);
        assert!(flags.include_enrichment);

        let err = parse_sections(&["bad".to_string()]).unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
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
}
