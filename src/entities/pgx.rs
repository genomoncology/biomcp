use std::collections::HashSet;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::entities::SearchPage;
use crate::error::BioMcpError;
use crate::sources::cpic::{
    CpicClient, CpicFrequencyRow, CpicGuidelineSummaryRow, CpicPairRow, CpicRecommendationRow,
};
use crate::sources::pharmgkb::{PharmGkbAnnotation, PharmGkbClient};

const PGX_SECTION_RECOMMENDATIONS: &str = "recommendations";
const PGX_SECTION_FREQUENCIES: &str = "frequencies";
const PGX_SECTION_GUIDELINES: &str = "guidelines";
const PGX_SECTION_ANNOTATIONS: &str = "annotations";
const PGX_SECTION_ALL: &str = "all";

pub const PGX_SECTION_NAMES: &[&str] = &[
    PGX_SECTION_RECOMMENDATIONS,
    PGX_SECTION_FREQUENCIES,
    PGX_SECTION_GUIDELINES,
    PGX_SECTION_ANNOTATIONS,
    PGX_SECTION_ALL,
];

const OPTIONAL_ENRICHMENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pgx {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gene: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drug: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub interactions: Vec<PgxInteraction>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recommendations: Vec<PgxRecommendation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub frequencies: Vec<PgxFrequency>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub guidelines: Vec<PgxGuideline>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub annotations: Vec<PharmGkbAnnotation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PgxInteraction {
    pub genesymbol: String,
    pub drugname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpiclevel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pgxtesting: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guidelinename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guidelineurl: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PgxRecommendation {
    pub drugname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phenotype: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity_score: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implication: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommendation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classification: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub population: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guidelinename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guidelineurl: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PgxFrequency {
    pub genesymbol: String,
    pub allele: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub population_group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_frequency: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_frequency: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PgxGuideline {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub genes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub drugs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PgxSearchResult {
    pub genesymbol: String,
    pub drugname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpiclevel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pgxtesting: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guidelinename: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct PgxSearchFilters {
    pub gene: Option<String>,
    pub drug: Option<String>,
    pub cpic_level: Option<String>,
    pub pgx_testing: Option<String>,
    pub evidence: Option<String>,
}

#[derive(Debug, Clone, Copy, Default)]
struct PgxSections {
    include_recommendations: bool,
    include_frequencies: bool,
    include_guidelines: bool,
    include_annotations: bool,
}

fn parse_sections(sections: &[String]) -> Result<PgxSections, BioMcpError> {
    let mut out = PgxSections::default();
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
            PGX_SECTION_RECOMMENDATIONS => out.include_recommendations = true,
            PGX_SECTION_FREQUENCIES => out.include_frequencies = true,
            PGX_SECTION_GUIDELINES => out.include_guidelines = true,
            PGX_SECTION_ANNOTATIONS => out.include_annotations = true,
            PGX_SECTION_ALL => include_all = true,
            _ => {
                return Err(BioMcpError::InvalidArgument(format!(
                    "Unknown section \"{section}\" for pgx. Available: {}",
                    PGX_SECTION_NAMES.join(", ")
                )));
            }
        }
    }

    if include_all {
        out.include_recommendations = true;
        out.include_frequencies = true;
        out.include_guidelines = true;
        out.include_annotations = true;
    }

    Ok(out)
}

pub async fn get(query: &str, sections: &[String]) -> Result<Pgx, BioMcpError> {
    let parsed_sections = parse_sections(sections)?;
    let query = query.trim();
    if query.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Gene or drug is required. Example: biomcp get pgx CYP2D6".into(),
        ));
    }
    if query.len() > 256 {
        return Err(BioMcpError::InvalidArgument(
            "PGx query is too long.".into(),
        ));
    }

    let cpic = CpicClient::new()?;
    let mut source_rows: Vec<CpicPairRow> = Vec::new();
    let mut mode_gene: Option<String> = None;
    let mut mode_drug: Option<String> = None;

    if is_likely_gene(query) {
        let rows = cpic.pairs_by_gene(query, 100).await?;
        if !rows.is_empty() {
            mode_gene = Some(query.trim().to_ascii_uppercase());
            source_rows = rows;
        }
    }

    if source_rows.is_empty() {
        let rows = cpic.pairs_by_drug(query, 100).await?;
        if !rows.is_empty() {
            mode_drug = Some(query.to_string());
            source_rows = rows;
        }
    }

    if source_rows.is_empty() {
        let rows = cpic.pairs_by_gene(query, 100).await?;
        if !rows.is_empty() {
            mode_gene = Some(query.trim().to_ascii_uppercase());
            source_rows = rows;
        }
    }

    if source_rows.is_empty() {
        return Err(BioMcpError::NotFound {
            entity: "pgx".into(),
            id: query.to_string(),
            suggestion: format!("Try searching: biomcp search pgx -g {query}"),
        });
    }

    let mut interactions = map_pair_rows(&source_rows);
    interactions.sort_by(|a, b| {
        cpic_level_rank(a.cpiclevel.as_deref())
            .cmp(&cpic_level_rank(b.cpiclevel.as_deref()))
            .then_with(|| a.drugname.cmp(&b.drugname))
            .then_with(|| a.genesymbol.cmp(&b.genesymbol))
    });

    if mode_gene.is_none() {
        let genes: Vec<String> = interactions
            .iter()
            .map(|row| row.genesymbol.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        if genes.len() == 1 {
            mode_gene = genes.first().cloned();
        }
    }

    if mode_drug.is_none() {
        let drugs: Vec<String> = interactions
            .iter()
            .map(|row| row.drugname.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        if drugs.len() == 1 {
            mode_drug = drugs.first().cloned();
        }
    }

    let mut out = Pgx {
        query: query.to_string(),
        gene: mode_gene.clone(),
        drug: mode_drug.clone(),
        interactions,
        recommendations: Vec::new(),
        frequencies: Vec::new(),
        guidelines: Vec::new(),
        annotations: Vec::new(),
        annotations_note: None,
    };

    if parsed_sections.include_recommendations {
        let recommendations = if let Some(gene) = mode_gene.as_deref() {
            cpic.recommendations_by_gene(gene, 50).await?
        } else if let Some(drug) = mode_drug.as_deref() {
            cpic.recommendations_by_drug(drug, 50).await?
        } else {
            Vec::new()
        };
        out.recommendations = map_recommendations(&recommendations, mode_gene.as_deref());
    }

    if parsed_sections.include_frequencies {
        let mut rows: Vec<PgxFrequency> = Vec::new();
        if let Some(gene) = mode_gene.as_deref() {
            let frequencies = cpic.frequencies_by_gene(gene, 30).await?;
            rows.extend(map_frequencies(&frequencies));
        } else {
            let unique_genes = out
                .interactions
                .iter()
                .map(|row| row.genesymbol.clone())
                .collect::<HashSet<_>>();
            for gene in unique_genes.into_iter().take(3) {
                match cpic.frequencies_by_gene(&gene, 12).await {
                    Ok(frequencies) => rows.extend(map_frequencies(&frequencies)),
                    Err(err) => warn!(gene = %gene, "CPIC frequency lookup failed: {err}"),
                }
            }
        }
        out.frequencies = dedupe_frequencies(rows);
    }

    if parsed_sections.include_guidelines {
        let guidelines = if let Some(gene) = mode_gene.as_deref() {
            cpic.guidelines_by_gene(gene, 40).await?
        } else {
            Vec::new()
        };

        if !guidelines.is_empty() {
            out.guidelines = map_guidelines(&guidelines);
        } else {
            out.guidelines = guidelines_from_pairs(&source_rows);
        }
    }

    if parsed_sections.include_annotations {
        let pharmgkb = PharmGkbClient::new()?;
        let annotation_fut = async {
            if let Some(gene) = mode_gene.as_deref() {
                pharmgkb.annotations_by_gene(gene, 40).await
            } else if let Some(drug) = mode_drug.as_deref() {
                pharmgkb.annotations_by_drug(drug, 40).await
            } else {
                Ok(Vec::new())
            }
        };

        match tokio::time::timeout(OPTIONAL_ENRICHMENT_TIMEOUT, annotation_fut).await {
            Ok(Ok(annotations)) => out.annotations = annotations,
            Ok(Err(err)) => {
                warn!("PharmGKB enrichment unavailable: {err}");
                out.annotations_note = Some(
                    "PharmGKB annotations unavailable; returned CPIC core content.".to_string(),
                );
            }
            Err(_) => {
                warn!(
                    timeout_secs = OPTIONAL_ENRICHMENT_TIMEOUT.as_secs(),
                    "PharmGKB enrichment timed out"
                );
                out.annotations_note =
                    Some("PharmGKB annotations timed out; returned CPIC core content.".to_string());
            }
        }
    }

    Ok(out)
}

#[allow(dead_code)]
pub async fn search(
    filters: &PgxSearchFilters,
    limit: usize,
) -> Result<Vec<PgxSearchResult>, BioMcpError> {
    Ok(search_page(filters, limit, 0).await?.results)
}

pub async fn search_page(
    filters: &PgxSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<SearchPage<PgxSearchResult>, BioMcpError> {
    const MAX_SEARCH_LIMIT: usize = 50;
    if limit == 0 || limit > MAX_SEARCH_LIMIT {
        return Err(BioMcpError::InvalidArgument(format!(
            "--limit must be between 1 and {MAX_SEARCH_LIMIT}"
        )));
    }

    let cpic = CpicClient::new()?;

    let gene = filters
        .gene
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_ascii_uppercase);
    let drug = filters
        .drug
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);

    if gene.is_none() && drug.is_none() {
        return Err(BioMcpError::InvalidArgument(
            "Provide -g <gene> or -d <drug>. Example: biomcp search pgx -g CYP2D6".into(),
        ));
    }

    let fetch_limit = (limit.saturating_mul(5)).clamp(limit, 200);
    let mut total: Option<usize> = None;
    let mut rows: Vec<CpicPairRow> = if let Some(gene) = gene.as_deref() {
        let page = cpic.pairs_by_gene_page(gene, fetch_limit, offset).await?;
        total = page.total;
        page.rows
    } else if let Some(drug) = drug.as_deref() {
        let page = cpic.pairs_by_drug_page(drug, fetch_limit, offset).await?;
        total = page.total;
        page.rows
    } else {
        Vec::new()
    };

    if let (Some(gene), Some(drug)) = (gene.as_deref(), drug.as_deref()) {
        rows.retain(|row| {
            row.genesymbol.eq_ignore_ascii_case(gene)
                && row
                    .drugname
                    .to_ascii_lowercase()
                    .contains(&drug.to_ascii_lowercase())
        });
    }

    let mut out = map_search_rows(&rows);
    if let Some(expected) = filters
        .cpic_level
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        out.retain(|row| {
            row.cpiclevel
                .as_deref()
                .map(str::trim)
                .is_some_and(|v| v.eq_ignore_ascii_case(expected))
        });
    }
    if let Some(expected) = filters
        .pgx_testing
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        out.retain(|row| {
            row.pgxtesting
                .as_deref()
                .map(str::trim)
                .is_some_and(|v| v.eq_ignore_ascii_case(expected))
        });
    }
    if let Some(expected) = filters
        .evidence
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        out.retain(|row| {
            row.guidelinename
                .as_deref()
                .map(str::trim)
                .is_some_and(|v| {
                    v.to_ascii_lowercase()
                        .contains(&expected.to_ascii_lowercase())
                })
                || row
                    .cpiclevel
                    .as_deref()
                    .map(str::trim)
                    .is_some_and(|v| v.eq_ignore_ascii_case(expected))
        });
    }
    out.sort_by(|a, b| {
        cpic_level_rank(a.cpiclevel.as_deref())
            .cmp(&cpic_level_rank(b.cpiclevel.as_deref()))
            .then_with(|| a.drugname.cmp(&b.drugname))
            .then_with(|| a.genesymbol.cmp(&b.genesymbol))
    });
    out.truncate(limit);

    Ok(SearchPage::offset(out, total))
}

pub fn search_query_summary(filters: &PgxSearchFilters) -> String {
    let mut parts = Vec::new();
    if let Some(gene) = filters
        .gene
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("gene={gene}"));
    }
    if let Some(drug) = filters
        .drug
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("drug={drug}"));
    }
    if let Some(value) = filters
        .cpic_level
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("cpic_level={value}"));
    }
    if let Some(value) = filters
        .pgx_testing
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("pgx_testing={value}"));
    }
    if let Some(value) = filters
        .evidence
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("evidence={value}"));
    }
    parts.join(", ")
}

fn is_likely_gene(value: &str) -> bool {
    let token = value.trim();
    if token.is_empty() || token.contains(char::is_whitespace) {
        return false;
    }
    let upper = token.to_ascii_uppercase();
    crate::sources::is_valid_gene_symbol(&upper)
        && upper
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-')
}

fn map_pair_rows(rows: &[CpicPairRow]) -> Vec<PgxInteraction> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for row in rows {
        let gene = row.genesymbol.trim().to_ascii_uppercase();
        let drug = row.drugname.trim().to_string();
        if gene.is_empty() || drug.is_empty() {
            continue;
        }

        let key = format!("{}|{}", gene, drug.to_ascii_lowercase());
        if !seen.insert(key) {
            continue;
        }

        out.push(PgxInteraction {
            genesymbol: gene,
            drugname: drug,
            cpiclevel: row.cpiclevel.clone(),
            pgxtesting: row.pgxtesting.clone(),
            guidelinename: row.guidelinename.clone(),
            guidelineurl: row.guidelineurl.clone(),
        });
    }
    out
}

fn map_search_rows(rows: &[CpicPairRow]) -> Vec<PgxSearchResult> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for row in rows {
        let gene = row.genesymbol.trim().to_ascii_uppercase();
        let drug = row.drugname.trim().to_string();
        if gene.is_empty() || drug.is_empty() {
            continue;
        }

        let key = format!("{}|{}", gene, drug.to_ascii_lowercase());
        if !seen.insert(key) {
            continue;
        }

        out.push(PgxSearchResult {
            genesymbol: gene,
            drugname: drug,
            cpiclevel: row.cpiclevel.clone(),
            pgxtesting: row.pgxtesting.clone(),
            guidelinename: row.guidelinename.clone(),
        });
    }
    out
}

fn map_recommendations(
    rows: &[CpicRecommendationRow],
    preferred_gene: Option<&str>,
) -> Vec<PgxRecommendation> {
    let mut out = Vec::new();
    for row in rows {
        let drugname = row.drugname.trim();
        if drugname.is_empty() {
            continue;
        }

        let phenotype = pick_lookup_value(&row.phenotypes, preferred_gene);
        let activity_score = pick_lookup_value(&row.activityscore, preferred_gene);
        let implication = pick_lookup_value(&row.implications, preferred_gene);

        out.push(PgxRecommendation {
            drugname: drugname.to_string(),
            phenotype,
            activity_score,
            implication,
            recommendation: row
                .drugrecommendation
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_string),
            classification: row
                .classification
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_string),
            population: row
                .population
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_string),
            guidelinename: row.guidelinename.clone(),
            guidelineurl: row.guidelineurl.clone(),
        });
    }

    out.sort_by(|a, b| a.drugname.cmp(&b.drugname));
    out.truncate(30);
    out
}

fn pick_lookup_value(
    map: &std::collections::HashMap<String, String>,
    preferred_gene: Option<&str>,
) -> Option<String> {
    if let Some(gene) = preferred_gene
        && let Some(value) = map
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(gene))
            .map(|(_, v)| v)
            .map(String::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
    {
        return Some(value.to_string());
    }

    map.values()
        .find(|v| !v.trim().is_empty())
        .map(|v| v.trim().to_string())
}

fn map_frequencies(rows: &[CpicFrequencyRow]) -> Vec<PgxFrequency> {
    rows.iter()
        .filter_map(|row| {
            let gene = row.genesymbol.trim();
            let allele = row.name.trim();
            if gene.is_empty() || allele.is_empty() {
                return None;
            }

            Some(PgxFrequency {
                genesymbol: gene.to_string(),
                allele: allele.to_string(),
                population_group: row.population_group.clone(),
                subject_count: row.subjectcount,
                frequency: row
                    .freq_weighted_avg
                    .or(row.freq_avg)
                    .or(row.freq_max)
                    .or(row.freq_min),
                min_frequency: row.freq_min,
                max_frequency: row.freq_max,
            })
        })
        .collect()
}

fn dedupe_frequencies(rows: Vec<PgxFrequency>) -> Vec<PgxFrequency> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for row in rows {
        let key = format!(
            "{}|{}|{}",
            row.genesymbol.to_ascii_uppercase(),
            row.allele.to_ascii_uppercase(),
            row.population_group
                .as_deref()
                .unwrap_or_default()
                .to_ascii_lowercase()
        );
        if !seen.insert(key) {
            continue;
        }
        out.push(row);
    }
    out.sort_by(|a, b| {
        a.genesymbol
            .cmp(&b.genesymbol)
            .then_with(|| a.allele.cmp(&b.allele))
            .then_with(|| {
                a.population_group
                    .as_deref()
                    .unwrap_or_default()
                    .cmp(b.population_group.as_deref().unwrap_or_default())
            })
    });
    out.truncate(30);
    out
}

fn map_guidelines(rows: &[CpicGuidelineSummaryRow]) -> Vec<PgxGuideline> {
    let mut out: Vec<PgxGuideline> = rows
        .iter()
        .filter_map(|row| {
            let name = row.guideline_name.trim();
            if name.is_empty() {
                return None;
            }

            Some(PgxGuideline {
                name: name.to_string(),
                url: row.guideline_url.clone(),
                genes: row
                    .genes
                    .iter()
                    .filter_map(|g| {
                        let symbol = g.symbol.trim();
                        if symbol.is_empty() {
                            None
                        } else {
                            Some(symbol.to_string())
                        }
                    })
                    .collect(),
                drugs: row
                    .drugs
                    .iter()
                    .filter_map(|d| {
                        let value = d.trim();
                        if value.is_empty() {
                            None
                        } else {
                            Some(value.to_string())
                        }
                    })
                    .collect(),
            })
        })
        .collect();

    out.sort_by(|a, b| a.name.cmp(&b.name));
    out.truncate(20);
    out
}

fn guidelines_from_pairs(rows: &[CpicPairRow]) -> Vec<PgxGuideline> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for row in rows {
        let Some(name) = row
            .guidelinename
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        else {
            continue;
        };

        let key = name.to_ascii_lowercase();
        if !seen.insert(key) {
            continue;
        }

        out.push(PgxGuideline {
            name: name.to_string(),
            url: row.guidelineurl.clone(),
            genes: Vec::new(),
            drugs: Vec::new(),
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

fn cpic_level_rank(level: Option<&str>) -> i32 {
    let value = level
        .map(str::trim)
        .unwrap_or_default()
        .to_ascii_uppercase();

    if value.starts_with('A') {
        0
    } else if value.starts_with('B') {
        1
    } else if value.starts_with('C') {
        2
    } else if value.starts_with('D') {
        3
    } else {
        4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sections_supports_all() {
        let parsed = parse_sections(&["all".to_string()]).expect("sections");
        assert!(parsed.include_recommendations);
        assert!(parsed.include_frequencies);
        assert!(parsed.include_guidelines);
        assert!(parsed.include_annotations);
    }

    #[test]
    fn search_summary_formats_filters() {
        let summary = search_query_summary(&PgxSearchFilters {
            gene: Some("CYP2D6".into()),
            drug: Some("codeine".into()),
            cpic_level: None,
            pgx_testing: None,
            evidence: None,
        });
        assert!(summary.contains("gene=CYP2D6"));
        assert!(summary.contains("drug=codeine"));
    }

    #[test]
    fn likely_gene_recognizes_hgnc_style_symbol() {
        assert!(is_likely_gene("CYP2D6"));
        assert!(!is_likely_gene("type 2 diabetes"));
    }
}
