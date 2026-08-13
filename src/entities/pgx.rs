use std::collections::{BTreeMap, HashSet};
use std::time::Duration;

use crate::entities::SearchPage;
use crate::entities::section_outcome::{SectionOutcome, SectionOutcomes};
use crate::entities::source_state_registry::outcome_keys;
use crate::error::BioMcpError;
use crate::sources::cpic::{
    CpicClient, CpicFrequencyRow, CpicGuidelineSummaryRow, CpicPairRow, CpicRecommendationRow,
};
use crate::sources::pharmgkb::{PharmGkbAnnotation, PharmGkbClient};
use serde::{Deserialize, Serialize};

const PGX_SECTION_INTERACTIONS: &str = "interactions";
const PGX_SECTION_RECOMMENDATIONS: &str = "recommendations";
const PGX_SECTION_FREQUENCIES: &str = "frequencies";
const PGX_SECTION_GUIDELINES: &str = "guidelines";
const PGX_SECTION_ANNOTATIONS: &str = "annotations";
const PGX_SECTION_ALL: &str = "all";

pub const PGX_SECTION_NAMES: &[&str] = &[
    PGX_SECTION_INTERACTIONS,
    PGX_SECTION_RECOMMENDATIONS,
    PGX_SECTION_FREQUENCIES,
    PGX_SECTION_GUIDELINES,
    PGX_SECTION_ANNOTATIONS,
    PGX_SECTION_ALL,
];

const OPTIONAL_ENRICHMENT_TIMEOUT: Duration = Duration::from_secs(10);

pub(crate) fn default_pgx_section_outcomes() -> SectionOutcomes {
    SectionOutcomes::with_keys(&outcome_keys("pgx"))
}

fn deserialize_pgx_section_outcomes<'de, D>(deserializer: D) -> Result<SectionOutcomes, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let outcomes = SectionOutcomes::deserialize(deserializer)?;
    outcomes
        .validate_keys(&outcome_keys("pgx"))
        .map_err(serde::de::Error::custom)?;
    Ok(outcomes)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pgx {
    #[serde(
        default = "default_pgx_section_outcomes",
        deserialize_with = "deserialize_pgx_section_outcomes"
    )]
    pub section_outcomes: SectionOutcomes,
    pub query: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub section_pagination: BTreeMap<String, PgxSectionPagination>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PgxSectionPagination {
    pub offset: usize,
    pub limit: usize,
    pub returned: usize,
    pub total: Option<usize>,
    pub has_more: bool,
    pub next_offset: Option<usize>,
}

impl PgxSectionPagination {
    fn new(
        offset: usize,
        limit: usize,
        returned: usize,
        total: Option<usize>,
        extra: bool,
    ) -> Self {
        let has_more = total
            .map(|count| offset.saturating_add(returned) < count)
            .unwrap_or(extra);
        Self {
            offset,
            limit,
            returned,
            total,
            has_more,
            next_offset: has_more.then(|| offset.saturating_add(returned)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PgxGetOptions {
    pub sections: Vec<String>,
    pub limit: usize,
    pub offset: usize,
    pub full: bool,
}

impl Default for PgxGetOptions {
    fn default() -> Self {
        Self {
            sections: Vec::new(),
            limit: 10,
            offset: 0,
            full: false,
        }
    }
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

fn normalize_cpic_level(value: &str) -> Result<String, BioMcpError> {
    match value.trim().to_ascii_uppercase().as_str() {
        "A" | "B" | "C" | "D" => Ok(value.trim().to_ascii_uppercase()),
        _ => Err(BioMcpError::InvalidArgument(
            "--cpic-level must be one of: A, B, C, D".into(),
        )),
    }
}

fn normalize_pgx_testing(value: &str) -> Result<String, BioMcpError> {
    const VALUES: &[&str] = &[
        "Actionable PGx",
        "Informative PGx",
        "No Clinical PGx",
        "Testing Recommended",
        "Testing Required",
    ];

    let value = value.trim();
    VALUES
        .iter()
        .find(|candidate| value.eq_ignore_ascii_case(candidate))
        .map(|candidate| (*candidate).to_string())
        .ok_or_else(|| {
            BioMcpError::InvalidArgument(format!(
                "--pgx-testing must be one of: {}",
                VALUES.join(", ")
            ))
        })
}

#[derive(Debug, Clone, Copy, Default)]
struct PgxSections {
    include_interactions: bool,
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
            PGX_SECTION_INTERACTIONS => out.include_interactions = true,
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
        out.include_interactions = true;
        out.include_recommendations = true;
        out.include_frequencies = true;
        out.include_guidelines = true;
        out.include_annotations = true;
    }

    Ok(out)
}

impl PgxSections {
    fn count(self) -> usize {
        [
            self.include_interactions,
            self.include_recommendations,
            self.include_frequencies,
            self.include_guidelines,
            self.include_annotations,
        ]
        .into_iter()
        .filter(|selected| *selected)
        .count()
    }

    fn all() -> Self {
        Self {
            include_interactions: true,
            include_recommendations: true,
            include_frequencies: true,
            include_guidelines: true,
            include_annotations: true,
        }
    }
}

pub async fn get(query: &str, sections: &[String]) -> Result<Pgx, BioMcpError> {
    get_with_options(
        query,
        &PgxGetOptions {
            sections: sections.to_vec(),
            ..PgxGetOptions::default()
        },
    )
    .await
}

pub async fn get_with_options(query: &str, options: &PgxGetOptions) -> Result<Pgx, BioMcpError> {
    get_with_cpic(query, options, &CpicClient::new()?).await
}

async fn get_with_cpic(
    query: &str,
    options: &PgxGetOptions,
    cpic: &CpicClient,
) -> Result<Pgx, BioMcpError> {
    if options.limit == 0 || options.limit > 50 {
        return Err(BioMcpError::InvalidArgument(
            "--limit must be between 1 and 50".into(),
        ));
    }
    let mut parsed_sections = parse_sections(&options.sections)?;
    if options.full {
        parsed_sections = PgxSections::all();
    } else if parsed_sections.count() == 0 {
        parsed_sections.include_interactions = true;
    }
    if options.offset > 0 && (options.full || parsed_sections.count() != 1) {
        return Err(BioMcpError::InvalidArgument(
            "--offset greater than zero requires exactly one named PGx section and cannot be combined with --full".into(),
        ));
    }
    let limit = if options.full { 50 } else { options.limit };
    let offset = options.offset;
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

    let mut mode_gene = is_likely_gene(query).then(|| query.to_ascii_uppercase());
    let mode_drug = mode_gene.is_none().then(|| query.to_string());

    let mut out = Pgx {
        section_outcomes: default_pgx_section_outcomes(),
        query: query.to_string(),
        section_pagination: BTreeMap::new(),
        gene: mode_gene.clone(),
        drug: mode_drug.clone(),
        interactions: Vec::new(),
        recommendations: Vec::new(),
        frequencies: Vec::new(),
        guidelines: Vec::new(),
        annotations: Vec::new(),
        annotations_note: None,
    };

    if parsed_sections.include_interactions {
        let page = if let Some(gene) = mode_gene.as_deref() {
            cpic.pairs_by_gene_page(gene, limit + 1, offset).await?
        } else {
            cpic.pairs_by_drug_page(query, limit + 1, offset).await?
        };
        let mut rows = map_pair_rows(&page.rows);
        let extra = rows.len() > limit;
        rows.truncate(limit);
        if rows.is_empty() && offset == 0 {
            return Err(BioMcpError::NotFound {
                entity: "pgx".into(),
                id: query.to_string(),
                suggestion: format!("Try searching: biomcp search pgx -g {query}"),
            });
        }
        out.section_pagination.insert(
            PGX_SECTION_INTERACTIONS.into(),
            PgxSectionPagination::new(offset, limit, rows.len(), page.total, extra),
        );
        out.interactions = rows;
    }

    async fn gene_for_drug(cpic: &CpicClient, drug: &str) -> Result<Option<String>, BioMcpError> {
        Ok(cpic
            .pairs_by_drug_page(drug, 1, 0)
            .await?
            .rows
            .first()
            .map(|row| row.genesymbol.trim().to_ascii_uppercase())
            .filter(|gene| !gene.is_empty()))
    }

    if parsed_sections.include_recommendations {
        let page = if let Some(gene) = mode_gene.as_deref() {
            cpic.recommendations_by_gene_page(gene, limit + 1, offset)
                .await?
        } else {
            cpic.recommendations_by_drug_page(query, limit + 1, offset)
                .await?
        };
        let mut rows = map_recommendations(&page.rows, mode_gene.as_deref());
        let extra = rows.len() > limit;
        rows.truncate(limit);
        out.section_pagination.insert(
            PGX_SECTION_RECOMMENDATIONS.into(),
            PgxSectionPagination::new(offset, limit, rows.len(), page.total, extra),
        );
        out.recommendations = rows;
    }

    if parsed_sections.include_frequencies {
        if mode_gene.is_none() {
            mode_gene = gene_for_drug(cpic, query).await?;
            out.gene.clone_from(&mode_gene);
        }
        let page = match mode_gene.as_deref() {
            Some(gene) => {
                cpic.frequencies_by_gene_page(gene, limit + 1, offset)
                    .await?
            }
            None => crate::sources::cpic::CpicPage {
                rows: Vec::new(),
                total: Some(0),
            },
        };
        let mut rows = dedupe_frequencies(map_frequencies(&page.rows));
        let extra = rows.len() > limit;
        rows.truncate(limit);
        out.section_pagination.insert(
            PGX_SECTION_FREQUENCIES.into(),
            PgxSectionPagination::new(offset, limit, rows.len(), page.total, extra),
        );
        out.frequencies = rows;
        let outcome = if out.frequencies.is_empty() {
            SectionOutcome::empty("CPIC")
        } else {
            SectionOutcome::data("CPIC")
        };
        out.section_outcomes.complete("frequencies", outcome);
    }

    if parsed_sections.include_guidelines {
        if mode_gene.is_none() {
            mode_gene = gene_for_drug(cpic, query).await?;
            out.gene.clone_from(&mode_gene);
        }
        let page = match mode_gene.as_deref() {
            Some(gene) => {
                cpic.guidelines_by_gene_page(gene, limit + 1, offset)
                    .await?
            }
            None => crate::sources::cpic::CpicPage {
                rows: Vec::new(),
                total: Some(0),
            },
        };
        let mut rows = map_guidelines(&page.rows);
        let extra = rows.len() > limit;
        rows.truncate(limit);
        out.section_pagination.insert(
            PGX_SECTION_GUIDELINES.into(),
            PgxSectionPagination::new(offset, limit, rows.len(), page.total, extra),
        );
        out.guidelines = rows;
    }

    if parsed_sections.include_annotations {
        let pharmgkb = match PharmGkbClient::new() {
            Ok(client) => client,
            Err(_) => {
                out.annotations_note = Some(
                    "PharmGKB annotations unavailable; returned CPIC core content.".to_string(),
                );
                out.section_outcomes.complete(
                    "annotations",
                    SectionOutcome::unavailable(
                        "PharmGKB annotations are temporarily unavailable.",
                    ),
                );
                return Ok(out);
            }
        };
        let annotation_fut = async {
            if let Some(gene) = mode_gene.as_deref() {
                pharmgkb
                    .annotations_by_gene_page(gene, limit + 1, offset)
                    .await
            } else if let Some(drug) = mode_drug.as_deref() {
                pharmgkb
                    .annotations_by_drug_page(drug, limit + 1, offset)
                    .await
            } else {
                Ok(Vec::new())
            }
        };

        match tokio::time::timeout(OPTIONAL_ENRICHMENT_TIMEOUT, annotation_fut).await {
            Ok(Ok(annotations)) => {
                let mut annotations = annotations;
                let extra = annotations.len() > limit;
                annotations.truncate(limit);
                out.section_pagination.insert(
                    PGX_SECTION_ANNOTATIONS.into(),
                    PgxSectionPagination::new(offset, limit, annotations.len(), None, extra),
                );
                out.annotations = annotations;
                let outcome = if out.annotations.is_empty() {
                    SectionOutcome::empty("PharmGKB")
                } else {
                    SectionOutcome::data("PharmGKB")
                };
                out.section_outcomes.complete("annotations", outcome);
            }
            Ok(Err(_)) => {
                out.annotations_note = Some(
                    "PharmGKB annotations unavailable; returned CPIC core content.".to_string(),
                );
                out.section_outcomes.complete(
                    "annotations",
                    SectionOutcome::unavailable(
                        "PharmGKB annotations are temporarily unavailable.",
                    ),
                );
            }
            Err(_) => {
                out.annotations_note =
                    Some("PharmGKB annotations timed out; returned CPIC core content.".to_string());
                out.section_outcomes.complete(
                    "annotations",
                    SectionOutcome::unavailable(
                        "PharmGKB annotations are temporarily unavailable.",
                    ),
                );
            }
        }
    }

    Ok(out)
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

    let pgx_testing = filters
        .pgx_testing
        .as_deref()
        .map(normalize_pgx_testing)
        .transpose()?;
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
        .map(normalize_cpic_level)
        .transpose()?
    {
        out.retain(|row| {
            row.cpiclevel
                .as_deref()
                .map(str::trim)
                .is_some_and(|v| v.eq_ignore_ascii_case(&expected))
        });
    }
    if let Some(expected) = pgx_testing.as_deref() {
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

#[cfg(test)]
fn distinct_actionable_cpic_gene_count(rows: &[CpicPairRow], threshold: usize) -> usize {
    if threshold == 0 {
        return 0;
    }

    let mut genes = HashSet::new();
    for row in rows {
        if cpic_level_rank(row.cpiclevel.as_deref()) > 1 {
            continue;
        }
        let gene = row.genesymbol.trim().to_ascii_uppercase();
        if gene.is_empty() {
            continue;
        }
        genes.insert(gene);
        if genes.len() >= threshold {
            break;
        }
    }
    genes.len()
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
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[test]
    fn parse_sections_supports_all() {
        let parsed = parse_sections(&["all".to_string()]).expect("sections");
        assert!(parsed.include_interactions);
        assert!(parsed.include_recommendations);
        assert!(parsed.include_frequencies);
        assert!(parsed.include_guidelines);
        assert!(parsed.include_annotations);
    }

    #[test]
    fn section_pagination_uses_exact_total_or_limit_plus_one() {
        assert_eq!(
            PgxSectionPagination::new(10, 5, 5, Some(16), false).next_offset,
            Some(15)
        );
        assert_eq!(
            PgxSectionPagination::new(10, 5, 5, Some(15), true).next_offset,
            None
        );
        assert_eq!(
            PgxSectionPagination::new(0, 5, 5, None, true).next_offset,
            Some(5)
        );
    }

    #[test]
    fn get_options_reject_invalid_paging_before_client_construction() {
        let runtime = tokio::runtime::Runtime::new().expect("runtime");
        let error = runtime
            .block_on(get_with_options(
                "CYP2D6",
                &PgxGetOptions {
                    sections: vec!["recommendations".into(), "guidelines".into()],
                    limit: 10,
                    offset: 1,
                    full: false,
                },
            ))
            .expect_err("multi-section offset must fail");
        assert!(error.to_string().contains("exactly one"));
    }

    #[tokio::test]
    async fn recommendations_only_uses_one_bounded_recommendation_request() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind CPIC fixture");
        let base = format!("http://{}", listener.local_addr().expect("fixture address"));
        let body = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/cpic/recommendation_cyp2d6_20260803.json"
        ));
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept CPIC request");
            let mut request = Vec::new();
            loop {
                let mut chunk = [0_u8; 4096];
                let read = stream.read(&mut chunk).await.expect("read CPIC request");
                request.extend_from_slice(&chunk[..read]);
                if read == 0 || request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
                    break;
                }
            }
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Range: 0-0/1\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream
                .write_all(response.as_bytes())
                .await
                .expect("write CPIC response");
            String::from_utf8(request).expect("request text")
        });
        let client =
            CpicClient::with_test_client(crate::sources::test_client().expect("test client"), base);
        let result = get_with_cpic(
            "CYP2D6",
            &PgxGetOptions {
                sections: vec!["recommendations".into()],
                limit: 10,
                offset: 0,
                full: false,
            },
            &client,
        )
        .await
        .expect("focused recommendations");

        assert!(result.interactions.is_empty());
        assert!(!result.recommendations.is_empty());
        let request = server.await.expect("CPIC fixture server");
        assert!(request.starts_with("GET /recommendation_view?"));
        assert!(request.contains("limit=11"));
        assert!(request.contains("offset=0"));
        assert!(!request.contains("pair_view"));
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

    #[test]
    fn normalize_cpic_level_accepts_supported_values() {
        assert_eq!(normalize_cpic_level("A").expect("A"), "A");
        assert_eq!(normalize_cpic_level("b").expect("b"), "B");
    }

    #[test]
    fn normalize_cpic_level_rejects_invalid_value() {
        let err = normalize_cpic_level("Z").expect_err("Z should fail");
        assert!(err.to_string().contains("A, B, C, D"));
    }

    #[test]
    fn normalize_pgx_testing_accepts_supported_values() {
        for value in [
            "Actionable PGx",
            "Informative PGx",
            "No Clinical PGx",
            "Testing Recommended",
            "Testing Required",
        ] {
            assert_eq!(normalize_pgx_testing(value).unwrap(), value);
        }
        assert_eq!(
            normalize_pgx_testing(" actionable pgx ").unwrap(),
            "Actionable PGx"
        );
    }

    #[test]
    fn normalize_pgx_testing_rejects_blank_and_unknown_values() {
        for value in ["", "unknown recommendation"] {
            let err = normalize_pgx_testing(value).expect_err("invalid value should fail");
            assert!(err.to_string().contains("Actionable PGx"));
        }
    }

    #[test]
    fn distinct_actionable_cpic_gene_count_counts_unique_genes_to_threshold() {
        let rows = vec![
            cpic_pair("CYP2C9", "A"),
            cpic_pair("cyp2c9", "A"),
            cpic_pair("G6PD", "C"),
            cpic_pair("VKORC1", "B"),
            cpic_pair("", "A"),
            cpic_pair("CYP4F2", "A"),
        ];

        assert_eq!(distinct_actionable_cpic_gene_count(&rows, 3), 3);
        assert_eq!(distinct_actionable_cpic_gene_count(&rows, 0), 0);
    }

    fn cpic_pair(gene: &str, level: &str) -> CpicPairRow {
        CpicPairRow {
            pairid: None,
            genesymbol: gene.into(),
            drugname: "warfarin".into(),
            cpiclevel: Some(level.into()),
            pgxtesting: None,
            guidelinename: None,
            guidelineurl: None,
            usedforrecommendation: None,
            provisional: None,
        }
    }
}
