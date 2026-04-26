use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use futures::future::join_all;
use serde::Serialize;
use serde_json::{Value, json};

use crate::cli::debug_plan::{DebugPlan, DebugPlanLeg};
use crate::error::BioMcpError;
use crate::render::markdown::shell_quote_arg as quote_arg;
use crate::utils::date::validate_since;

const MAX_SEARCH_ALL_LIMIT: usize = 50;
const EXPAND_LIMIT: usize = 20;
const SECTION_TIMEOUT: Duration = Duration::from_secs(12);
// Article fan-out can legitimately run longer than the other sections when the
// shared Semantic Scholar pool throttles unauthenticated enrichment.
const ARTICLE_SECTION_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Debug, Clone)]
pub struct SearchAllInput {
    pub gene: Option<String>,
    pub variant: Option<String>,
    pub disease: Option<String>,
    pub drug: Option<String>,
    pub keyword: Option<String>,
    pub since: Option<String>,
    pub limit: usize,
    pub counts_only: bool,
    pub debug_plan: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchAllLink {
    pub rel: String,
    pub title: String,
    pub command: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchAllSection {
    pub entity: String,
    pub label: String,
    pub count: usize,
    pub total: Option<usize>,
    pub error: Option<String>,
    pub note: Option<String>,
    pub results: Vec<Value>,
    pub links: Vec<SearchAllLink>,
}

impl SearchAllSection {
    pub fn markdown_columns(&self) -> &'static [&'static str] {
        match self.entity.as_str() {
            "gene" => &["Symbol", "Name", "Entrez"],
            "variant" => &["ID", "Gene", "Protein", "Significance"],
            "disease" => &["ID", "Name", "Synonyms"],
            "drug" => &["Name", "Target", "Mechanism"],
            "trial" => &["NCT", "Title", "Status"],
            "article" => &["PMID", "Title", "Date"],
            "pathway" => &["ID", "Name"],
            "pgx" => &["Gene", "Drug", "CPIC"],
            "gwas" => &["rsID", "Trait", "P-Value"],
            "adverse-event" => &["Reaction", "Count"],
            _ => &[],
        }
    }

    pub fn markdown_rows(&self) -> Vec<Vec<String>> {
        self.results
            .iter()
            .filter_map(|row| match self.entity.as_str() {
                "gene" => Some(vec![
                    value_str(row, "symbol"),
                    value_str(row, "name"),
                    value_str(row, "entrez_id"),
                ]),
                "variant" => {
                    let rendered = vec![
                        value_str(row, "id"),
                        value_str(row, "gene"),
                        value_str(row, "hgvs_p"),
                        value_str(row, "significance"),
                    ];
                    let uninformative = rendered.get(1).is_some_and(|cell| is_empty_cell(cell))
                        && rendered.get(2).is_some_and(|cell| is_empty_cell(cell))
                        && rendered.get(3).is_some_and(|cell| is_empty_cell(cell));
                    (!uninformative).then_some(rendered)
                }
                "disease" => Some(vec![
                    value_str(row, "id"),
                    value_str(row, "name"),
                    value_str(row, "synonyms_preview"),
                ]),
                "drug" => Some(vec![
                    value_str(row, "name"),
                    value_str(row, "target"),
                    value_str_or(row, "mechanism", value_str(row, "drug_type")),
                ]),
                "trial" => Some(vec![
                    value_str(row, "nct_id"),
                    value_str(row, "title"),
                    value_str(row, "status"),
                ]),
                "article" => Some(vec![
                    value_str(row, "pmid"),
                    value_str(row, "title"),
                    value_str(row, "date"),
                ]),
                "pathway" => Some(vec![value_str(row, "id"), value_str(row, "name")]),
                "pgx" => Some(vec![
                    value_str(row, "genesymbol"),
                    value_str(row, "drugname"),
                    value_str(row, "cpiclevel"),
                ]),
                "gwas" => Some(vec![
                    value_str(row, "rsid"),
                    value_str(row, "trait_name"),
                    value_p_value(row, "p_value"),
                ]),
                "adverse-event" => Some(vec![value_str(row, "reaction"), value_str(row, "count")]),
                _ => Some(vec![format_value(row)]),
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
struct SectionResult {
    rows: Vec<Value>,
    total: Option<usize>,
    note: Option<String>,
}

impl SectionResult {
    fn new(rows: Vec<Value>, total: Option<usize>) -> Self {
        Self {
            rows,
            total,
            note: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchAllResults {
    pub query: String,
    pub sections: Vec<SearchAllSection>,
    pub searches_dispatched: usize,
    pub searches_with_results: usize,
    pub wall_time_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) debug_plan: Option<DebugPlan>,
}

#[derive(Debug, Serialize)]
pub(crate) struct SearchAllCountsOnlyJson<'a> {
    pub query: &'a str,
    pub sections: Vec<SearchAllCountsOnlySection<'a>>,
    pub searches_dispatched: usize,
    pub searches_with_results: usize,
    pub wall_time_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_plan: Option<&'a DebugPlan>,
}

#[derive(Debug, Serialize)]
pub(crate) struct SearchAllCountsOnlySection<'a> {
    pub entity: &'a str,
    pub label: &'a str,
    pub count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<&'a str>,
}

pub(crate) fn counts_only_json(results: &SearchAllResults) -> SearchAllCountsOnlyJson<'_> {
    SearchAllCountsOnlyJson {
        query: &results.query,
        sections: results
            .sections
            .iter()
            .map(|section| SearchAllCountsOnlySection {
                entity: &section.entity,
                label: &section.label,
                count: section.total.unwrap_or(section.count),
                note: section.note.as_deref(),
                error: section.error.as_deref(),
            })
            .collect(),
        searches_dispatched: results.searches_dispatched,
        searches_with_results: results.searches_with_results,
        wall_time_ms: results.wall_time_ms,
        debug_plan: results.debug_plan.as_ref(),
    }
}

#[derive(Debug, Clone)]
pub struct DispatchSpec {
    pub entity: &'static str,
    kind: SectionKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SectionKind {
    Gene,
    Variant,
    Disease,
    Drug,
    Trial,
    Article,
    Pathway,
    Pgx,
    Gwas,
    AdverseEvent,
}

impl SectionKind {
    fn from_entity(entity: &str) -> Option<Self> {
        match entity {
            "gene" => Some(Self::Gene),
            "variant" => Some(Self::Variant),
            "disease" => Some(Self::Disease),
            "drug" => Some(Self::Drug),
            "trial" => Some(Self::Trial),
            "article" => Some(Self::Article),
            "pathway" => Some(Self::Pathway),
            "pgx" => Some(Self::Pgx),
            "gwas" => Some(Self::Gwas),
            "adverse-event" => Some(Self::AdverseEvent),
            _ => None,
        }
    }

    fn entity(self) -> &'static str {
        match self {
            Self::Gene => "gene",
            Self::Variant => "variant",
            Self::Disease => "disease",
            Self::Drug => "drug",
            Self::Trial => "trial",
            Self::Article => "article",
            Self::Pathway => "pathway",
            Self::Pgx => "pgx",
            Self::Gwas => "gwas",
            Self::AdverseEvent => "adverse-event",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Gene => "Genes",
            Self::Variant => "Variants",
            Self::Disease => "Diseases",
            Self::Drug => "Drugs",
            Self::Trial => "Trials",
            Self::Article => "Articles",
            Self::Pathway => "Pathways",
            Self::Pgx => "PGx",
            Self::Gwas => "GWAS",
            Self::AdverseEvent => "Adverse Events",
        }
    }
}

fn section_timeout(kind: SectionKind) -> Duration {
    match kind {
        SectionKind::Article => ARTICLE_SECTION_TIMEOUT,
        _ => SECTION_TIMEOUT,
    }
}

const GENE_ORDER: [SectionKind; 10] = [
    SectionKind::Gene,
    SectionKind::Variant,
    SectionKind::Disease,
    SectionKind::Drug,
    SectionKind::Trial,
    SectionKind::Article,
    SectionKind::Pathway,
    SectionKind::Pgx,
    SectionKind::Gwas,
    SectionKind::AdverseEvent,
];

const DISEASE_ORDER: [SectionKind; 10] = [
    SectionKind::Disease,
    SectionKind::Variant,
    SectionKind::Drug,
    SectionKind::Trial,
    SectionKind::Article,
    SectionKind::Gwas,
    SectionKind::Pgx,
    SectionKind::AdverseEvent,
    SectionKind::Gene,
    SectionKind::Pathway,
];

const DRUG_ORDER: [SectionKind; 10] = [
    SectionKind::Drug,
    SectionKind::Variant,
    SectionKind::Trial,
    SectionKind::Article,
    SectionKind::Pgx,
    SectionKind::AdverseEvent,
    SectionKind::Disease,
    SectionKind::Gene,
    SectionKind::Pathway,
    SectionKind::Gwas,
];

const VARIANT_ORDER: [SectionKind; 10] = [
    SectionKind::Variant,
    SectionKind::Gene,
    SectionKind::Trial,
    SectionKind::Article,
    SectionKind::Drug,
    SectionKind::Pathway,
    SectionKind::Disease,
    SectionKind::Pgx,
    SectionKind::Gwas,
    SectionKind::AdverseEvent,
];

const KEYWORD_ORDER: [SectionKind; 1] = [SectionKind::Article];

#[derive(Debug, Clone, Copy)]
enum Anchor {
    Gene,
    Disease,
    Drug,
    Variant,
    Keyword,
}

impl Anchor {
    fn as_str(self) -> &'static str {
        match self {
            Self::Gene => "gene",
            Self::Disease => "disease",
            Self::Drug => "drug",
            Self::Variant => "variant",
            Self::Keyword => "keyword",
        }
    }
}

#[derive(Debug, Clone)]
struct VariantContext {
    raw: String,
    parsed_gene: Option<String>,
    parsed_change: Option<String>,
}

#[derive(Debug, Clone)]
struct PreparedInput {
    gene: Option<String>,
    variant: Option<String>,
    disease: Option<String>,
    drug: Option<String>,
    keyword: Option<String>,
    since: Option<String>,
    limit: usize,
    counts_only: bool,
    debug_plan: bool,
    anchor: Anchor,
    variant_context: Option<VariantContext>,
}

pub fn build_dispatch_plan(input: &SearchAllInput) -> Vec<DispatchSpec> {
    let Ok(prepared) = PreparedInput::new(input) else {
        return Vec::new();
    };
    build_dispatch_plan_prepared(&prepared)
}

pub async fn dispatch(input: &SearchAllInput) -> Result<SearchAllResults, BioMcpError> {
    let prepared = PreparedInput::new(input)?;
    let plan = build_dispatch_plan_prepared(&prepared);
    let started = Instant::now();

    let sections = join_all(
        plan.iter()
            .map(|spec| dispatch_section(spec.kind, &prepared)),
    )
    .await;

    let searches_dispatched = sections.len();
    let searches_with_results = sections
        .iter()
        .filter(|section| section.error.is_none() && section.count > 0)
        .count();

    Ok(SearchAllResults {
        query: prepared.query_summary(),
        debug_plan: prepared
            .debug_plan
            .then(|| build_result_plan(&prepared, &sections)),
        sections,
        searches_dispatched,
        searches_with_results,
        wall_time_ms: u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
    })
}

fn build_result_plan(input: &PreparedInput, sections: &[SearchAllSection]) -> DebugPlan {
    let disease_leg_ungrounded = sections
        .iter()
        .find(|section| section.entity == SectionKind::Disease.entity())
        .is_some_and(|section| section.count == 0 && section.error.is_none());
    let legs = sections
        .iter()
        .filter_map(|section| {
            let kind = SectionKind::from_entity(section.entity.as_str())?;
            Some(DebugPlanLeg {
                leg: kind.entity().to_string(),
                entity: kind.entity().to_string(),
                filters: leg_filters(kind, input),
                routing: leg_routing(kind, input, section, disease_leg_ungrounded),
                sources: leg_sources(kind, input),
                matched_sources: if kind == SectionKind::Article {
                    article_matched_sources(section)
                } else {
                    Vec::new()
                },
                count: section.count,
                total: section.total,
                note: section.note.clone(),
                error: section.error.clone(),
            })
        })
        .collect();

    DebugPlan {
        surface: "search_all",
        query: input.query_summary(),
        anchor: Some(input.anchor.as_str()),
        legs,
    }
}

fn leg_filters(kind: SectionKind, input: &PreparedInput) -> Vec<String> {
    match kind {
        SectionKind::Gene => input
            .gene_anchor()
            .map(|value| vec![format!("query={value}")])
            .unwrap_or_default(),
        SectionKind::Variant => {
            if let Some(variant_id) = input
                .variant_context
                .as_ref()
                .and_then(|ctx| (ctx.parsed_gene.is_none()).then_some(ctx.raw.as_str()))
            {
                return vec![format!("variant={variant_id}")];
            }

            let mut filters = Vec::new();
            if let Some(value) = input.gene_anchor() {
                filters.push(format!("gene={value}"));
            }
            if let Some(value) = input
                .variant_context
                .as_ref()
                .and_then(|ctx| ctx.parsed_change.as_deref())
            {
                filters.push(format!("hgvsp={value}"));
            }
            if let Some(value) = input.disease.as_deref() {
                filters.push(format!("condition={value}"));
            }
            if let Some(value) = input.drug.as_deref() {
                filters.push(format!("therapy={value}"));
            }
            filters
        }
        SectionKind::Disease => input
            .disease
            .as_deref()
            .map(|value| vec![format!("query={value}")])
            .unwrap_or_default(),
        SectionKind::Drug => {
            let mut filters = Vec::new();
            if let Some(value) = input.drug_query() {
                filters.push(format!("query={value}"));
            }
            if let Some(value) = input.gene_anchor() {
                filters.push(format!("target={value}"));
            }
            if let Some(value) = input.disease.as_deref() {
                filters.push(format!("indication={value}"));
            }
            filters
        }
        SectionKind::Trial => {
            let mut filters = Vec::new();
            if let Some(value) = input.trial_condition_query() {
                filters.push(format!("condition={value}"));
            }
            if let Some(value) = input.drug.as_deref() {
                filters.push(format!("intervention={value}"));
            }
            if let Some(value) = input.gene_anchor() {
                filters.push(format!("biomarker={value}"));
            }
            if let Some(value) = input.variant_trial_query() {
                filters.push(format!("mutation={value}"));
            }
            if let Some(value) = input.since.as_deref() {
                filters.push(format!("date_from={value}"));
            }
            filters
        }
        SectionKind::Article => {
            let mut filters = Vec::new();
            if let Some(value) = input.gene_anchor() {
                filters.push(format!("gene={value}"));
            }
            if let Some(value) = input.article_disease_filter() {
                filters.push(format!("disease={value}"));
            }
            if let Some(value) = input.drug.as_deref() {
                filters.push(format!("drug={value}"));
            }
            if let Some(value) = input.article_keyword_filter() {
                filters.push(format!("keyword={value}"));
            }
            if let Some(value) = input.since.as_deref() {
                filters.push(format!("date_from={value}"));
            }
            filters
        }
        SectionKind::Pathway => input
            .gene_anchor()
            .map(|value| vec![format!("query={value}")])
            .unwrap_or_default(),
        SectionKind::Pgx => {
            let mut filters = Vec::new();
            if let Some(value) = input.gene_anchor() {
                filters.push(format!("gene={value}"));
            }
            if let Some(value) = input.drug.as_deref() {
                filters.push(format!("drug={value}"));
            }
            filters
        }
        SectionKind::Gwas => {
            let mut filters = Vec::new();
            if let Some(value) = input.gene_anchor() {
                filters.push(format!("gene={value}"));
            }
            if let Some(value) = input.disease.as_deref() {
                filters.push(format!("trait={value}"));
            }
            filters
        }
        SectionKind::AdverseEvent => {
            let mut filters = Vec::new();
            if let Some(value) = input.drug.as_deref() {
                filters.push(format!("drug={value}"));
            }
            if let Some(value) = input.since.as_deref() {
                filters.push(format!("since={value}"));
            }
            filters
        }
    }
}

fn leg_routing(
    kind: SectionKind,
    input: &PreparedInput,
    section: &SearchAllSection,
    disease_leg_ungrounded: bool,
) -> Vec<String> {
    let mut routing = vec![format!("anchor={}", input.anchor.as_str())];

    match kind {
        SectionKind::Variant => {
            if input
                .variant_context
                .as_ref()
                .is_some_and(|ctx| ctx.parsed_gene.is_none())
            {
                routing.push("routing=direct_get".to_string());
            }
            if section.note.is_some() && input.gene_anchor().is_some() && input.disease.is_some() {
                routing.push("fallback=gene_only_variant_backfill".to_string());
            }
        }
        SectionKind::Drug => {}
        SectionKind::Trial => routing.push("routing=recruiting_preference_backfill".to_string()),
        SectionKind::Article => {
            routing.push("routing=source_federation".to_string());
            if input.has_shared_disease_keyword() {
                routing.push("fallback=shared_disease_keyword_orientation".to_string());
                if disease_leg_ungrounded && section.error.is_none() {
                    routing.push("fallback=disease_leg_ungrounded_keyword_survived".to_string());
                }
            }
        }
        SectionKind::Gene
        | SectionKind::Disease
        | SectionKind::Pathway
        | SectionKind::Pgx
        | SectionKind::Gwas
        | SectionKind::AdverseEvent => {}
    }

    routing
}

fn leg_sources(kind: SectionKind, input: &PreparedInput) -> Vec<String> {
    match kind {
        SectionKind::Gene => vec!["MyGene.info".to_string()],
        SectionKind::Variant => vec!["MyVariant.info".to_string()],
        SectionKind::Disease => vec!["MyDisease.info".to_string()],
        SectionKind::Drug => vec!["MyChem.info".to_string()],
        SectionKind::Trial => vec!["ClinicalTrials.gov".to_string()],
        SectionKind::Article => {
            let filters = article_filters(input);
            let mut sources = vec![
                "PubTator3".to_string(),
                "Europe PMC".to_string(),
                "PubMed".to_string(),
            ];
            if crate::entities::article::litsense2_search_enabled(
                &filters,
                crate::entities::article::ArticleSourceFilter::All,
            ) {
                sources.push("LitSense2".to_string());
            }
            if crate::entities::article::semantic_scholar_search_enabled(
                &filters,
                crate::entities::article::ArticleSourceFilter::All,
            ) {
                sources.push("Semantic Scholar".to_string());
            }
            sources
        }
        SectionKind::Pathway => vec![
            "Reactome".to_string(),
            "KEGG".to_string(),
            "WikiPathways".to_string(),
        ],
        SectionKind::Pgx => vec!["CPIC".to_string()],
        SectionKind::Gwas => vec!["GWAS Catalog".to_string()],
        SectionKind::AdverseEvent => vec!["OpenFDA".to_string()],
    }
}

fn article_filters(input: &PreparedInput) -> crate::entities::article::ArticleSearchFilters {
    crate::entities::article::ArticleSearchFilters {
        gene: input.gene_anchor().map(str::to_string),
        gene_anchored: matches!(input.anchor, Anchor::Gene) && input.gene.is_some(),
        disease: input.article_disease_filter().map(str::to_string),
        drug: input.drug.clone(),
        author: None,
        keyword: input.article_keyword_filter().map(str::to_string),
        date_from: input.since.clone(),
        date_to: None,
        article_type: None,
        journal: None,
        open_access: false,
        no_preprints: false,
        exclude_retracted: true,
        max_per_source: None,
        sort: crate::entities::article::ArticleSort::Relevance,
        ranking: crate::entities::article::ArticleRankingOptions::default(),
    }
}

fn article_matched_sources(section: &SearchAllSection) -> Vec<String> {
    let mut matched = Vec::new();
    for source in [
        "pubtator",
        "europepmc",
        "pubmed",
        "semanticscholar",
        "litsense2",
    ] {
        let present = section.results.iter().any(|row| {
            row.get("matched_sources")
                .and_then(Value::as_array)
                .is_some_and(|values| {
                    values
                        .iter()
                        .filter_map(Value::as_str)
                        .any(|value| value == source)
                })
        });
        if present && let Some(display) = article_source_display_name(source) {
            matched.push(display.to_string());
        }
    }
    matched
}

fn article_source_display_name(source: &str) -> Option<&'static str> {
    match source {
        "pubtator" => Some("PubTator3"),
        "europepmc" => Some("Europe PMC"),
        "pubmed" => Some("PubMed"),
        "semanticscholar" => Some("Semantic Scholar"),
        "litsense2" => Some("LitSense2"),
        _ => None,
    }
}

impl PreparedInput {
    fn new(input: &SearchAllInput) -> Result<Self, BioMcpError> {
        if input.limit == 0 || input.limit > MAX_SEARCH_ALL_LIMIT {
            return Err(BioMcpError::InvalidArgument(format!(
                "--limit must be between 1 and {MAX_SEARCH_ALL_LIMIT}"
            )));
        }

        let gene = normalize_slot(input.gene.clone());
        let variant = normalize_slot(input.variant.clone());
        let disease = normalize_slot(input.disease.clone());
        let drug = normalize_slot(input.drug.clone());
        let keyword = normalize_slot(input.keyword.clone());

        if gene.is_none()
            && variant.is_none()
            && disease.is_none()
            && drug.is_none()
            && keyword.is_none()
        {
            return Err(BioMcpError::InvalidArgument(
                "at least one typed slot is required (--gene, --variant, --disease, --drug, or --keyword).".into(),
            ));
        }

        let since = input
            .since
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(validate_since)
            .transpose()?;

        let variant_context = variant.as_deref().map(parse_variant_context);

        let anchor = if gene.is_some() {
            Anchor::Gene
        } else if disease.is_some() {
            Anchor::Disease
        } else if drug.is_some() {
            Anchor::Drug
        } else if variant.is_some() {
            Anchor::Variant
        } else {
            Anchor::Keyword
        };

        Ok(Self {
            gene,
            variant,
            disease,
            drug,
            keyword,
            since,
            limit: input.limit,
            counts_only: input.counts_only,
            debug_plan: input.debug_plan,
            anchor,
            variant_context,
        })
    }

    fn query_summary(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if let Some(value) = self.gene.as_deref() {
            parts.push(format!("gene={value}"));
        }
        if let Some(value) = self.variant.as_deref() {
            parts.push(format!("variant={value}"));
        }
        if let Some(value) = self.disease.as_deref() {
            parts.push(format!("disease={value}"));
        }
        if let Some(value) = self.drug.as_deref() {
            parts.push(format!("drug={value}"));
        }
        if let Some(value) = self.keyword.as_deref() {
            parts.push(format!("keyword={value}"));
        }
        if let Some(value) = self.since.as_deref() {
            parts.push(format!("since={value}"));
        }
        parts.join(" ")
    }

    fn gene_anchor(&self) -> Option<&str> {
        self.gene.as_deref().or_else(|| {
            self.variant_context
                .as_ref()
                .and_then(|ctx| ctx.parsed_gene.as_deref())
        })
    }

    fn has_shared_disease_keyword(&self) -> bool {
        matches!(
            (self.disease.as_deref(), self.keyword.as_deref()),
            (Some(disease), Some(keyword)) if tokens_equal_normalized(disease, keyword)
        )
    }

    fn article_disease_filter(&self) -> Option<&str> {
        if self.has_shared_disease_keyword() {
            None
        } else {
            self.disease.as_deref()
        }
    }

    fn article_keyword_filter(&self) -> Option<&str> {
        self.keyword.as_deref()
    }

    fn drug_query(&self) -> Option<&str> {
        self.drug.as_deref()
    }

    fn variant_trial_query(&self) -> Option<String> {
        let context = self.variant_context.as_ref()?;
        if let (Some(gene), Some(change)) = (
            context.parsed_gene.as_deref(),
            context.parsed_change.as_deref(),
        ) {
            return Some(format!("{gene} {change}"));
        }
        Some(context.raw.clone())
    }

    fn trial_condition_query(&self) -> Option<&str> {
        self.disease.as_deref()
    }
}

fn tokens_equal_normalized(a: &str, b: &str) -> bool {
    a.trim().eq_ignore_ascii_case(b.trim())
}

fn parse_variant_context(raw: &str) -> VariantContext {
    let mut parsed_gene = None;
    let mut parsed_change = None;

    if let Ok(crate::entities::variant::VariantIdFormat::GeneProteinChange { gene, change }) =
        crate::entities::variant::parse_variant_id(raw)
    {
        parsed_gene = Some(gene);
        parsed_change = Some(change);
    }

    VariantContext {
        raw: raw.to_string(),
        parsed_gene,
        parsed_change,
    }
}

fn build_dispatch_plan_prepared(input: &PreparedInput) -> Vec<DispatchSpec> {
    let mut included: HashSet<SectionKind> = HashSet::new();

    if input.gene.is_some() {
        included.insert(SectionKind::Gene);
        included.insert(SectionKind::Variant);
        included.insert(SectionKind::Drug);
        included.insert(SectionKind::Trial);
        included.insert(SectionKind::Article);
        included.insert(SectionKind::Pathway);
        included.insert(SectionKind::Pgx);
    }

    if input.disease.is_some() {
        included.insert(SectionKind::Disease);
        included.insert(SectionKind::Variant);
        included.insert(SectionKind::Drug);
        included.insert(SectionKind::Trial);
        included.insert(SectionKind::Article);
        included.insert(SectionKind::Gwas);
    }

    if input.drug.is_some() {
        included.insert(SectionKind::Drug);
        included.insert(SectionKind::Variant);
        included.insert(SectionKind::Trial);
        included.insert(SectionKind::Article);
        included.insert(SectionKind::Pgx);
        included.insert(SectionKind::AdverseEvent);
    }

    if let Some(context) = input.variant_context.as_ref() {
        included.insert(SectionKind::Variant);
        if context.parsed_gene.is_some() {
            included.insert(SectionKind::Gene);
            included.insert(SectionKind::Trial);
            included.insert(SectionKind::Article);
            included.insert(SectionKind::Drug);
            included.insert(SectionKind::Pathway);
        }
    }

    if input.keyword.is_some() {
        included.insert(SectionKind::Article);
    }

    let ordered: &[SectionKind] = match input.anchor {
        Anchor::Gene => &GENE_ORDER,
        Anchor::Disease => &DISEASE_ORDER,
        Anchor::Drug => &DRUG_ORDER,
        Anchor::Variant => &VARIANT_ORDER,
        Anchor::Keyword => &KEYWORD_ORDER,
    };

    ordered
        .iter()
        .copied()
        .filter(|kind| included.contains(kind))
        .map(|kind| DispatchSpec {
            entity: kind.entity(),
            kind,
        })
        .collect()
}

async fn dispatch_section(kind: SectionKind, input: &PreparedInput) -> SearchAllSection {
    let search_self = canonical_search_command(kind, input, input.limit);
    let timeout = section_timeout(kind);
    let effective_limit = section_fetch_limit(kind, input);
    let section_result =
        tokio::time::timeout(timeout, run_section(kind, input, effective_limit)).await;

    match section_result {
        Ok(Ok(section_result)) => {
            let links = build_links(kind, input, &section_result.rows, &search_self);
            SearchAllSection {
                entity: kind.entity().to_string(),
                label: kind.label().to_string(),
                count: section_result.rows.len(),
                total: section_result.total,
                error: None,
                note: section_result.note,
                results: section_result.rows,
                links,
            }
        }
        Ok(Err(err)) => SearchAllSection {
            entity: kind.entity().to_string(),
            label: kind.label().to_string(),
            count: 0,
            total: None,
            error: Some(err.to_string()),
            note: None,
            results: Vec::new(),
            links: vec![SearchAllLink {
                rel: "search.retry".to_string(),
                title: format!("Retry {} search", kind.entity()),
                command: search_self,
            }],
        },
        Err(_) => SearchAllSection {
            entity: kind.entity().to_string(),
            label: kind.label().to_string(),
            count: 0,
            total: None,
            error: Some(format!(
                "{} search timed out after {}s",
                kind.entity(),
                timeout.as_secs()
            )),
            note: None,
            results: Vec::new(),
            links: vec![SearchAllLink {
                rel: "search.retry".to_string(),
                title: format!("Retry {} search", kind.entity()),
                command: search_self,
            }],
        },
    }
}

async fn run_section(
    kind: SectionKind,
    input: &PreparedInput,
    limit: usize,
) -> Result<SectionResult, BioMcpError> {
    match kind {
        SectionKind::Gene => {
            let query = input.gene_anchor().ok_or_else(|| {
                BioMcpError::InvalidArgument("No gene anchor available for gene search.".into())
            })?;
            let filters = crate::entities::gene::GeneSearchFilters {
                query: Some(query.to_string()),
                ..Default::default()
            };
            let page = crate::entities::gene::search_page(&filters, limit, 0).await?;
            Ok(SectionResult::new(to_json_array(page.results)?, page.total))
        }
        SectionKind::Variant => {
            if let Some(variant_id) = input
                .variant_context
                .as_ref()
                .and_then(|ctx| (ctx.parsed_gene.is_none()).then_some(ctx.raw.as_str()))
            {
                let row = crate::entities::variant::get(variant_id, &[]).await?;
                return Ok(SectionResult::new(to_json_array(vec![row])?, Some(1)));
            }

            let filters = crate::entities::variant::VariantSearchFilters {
                gene: input.gene_anchor().map(str::to_string),
                hgvsp: input
                    .variant_context
                    .as_ref()
                    .and_then(|ctx| ctx.parsed_change.clone()),
                condition: input.disease.clone(),
                therapy: input.drug.clone(),
                ..Default::default()
            };
            let has_filter = filters
                .gene
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
                || filters
                    .hgvsp
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
                || filters
                    .condition
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
                || filters
                    .therapy
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty());
            if !has_filter {
                return Err(BioMcpError::InvalidArgument(
                    "No filters available for variant search.".into(),
                ));
            }
            let page = crate::entities::variant::search_page(&filters, limit, 0).await?;
            let mut rows = page.results;
            let mut total = page.total;
            let mut note = None;

            if rows.is_empty() && input.gene_anchor().is_some() && input.disease.is_some() {
                let fallback_filters = crate::entities::variant::VariantSearchFilters {
                    gene: filters.gene.clone(),
                    hgvsp: filters.hgvsp.clone(),
                    therapy: filters.therapy.clone(),
                    ..Default::default()
                };
                let fallback_page =
                    crate::entities::variant::search_page(&fallback_filters, limit, 0).await?;
                rows = fallback_page.results;
                total = fallback_page.total;
                if !rows.is_empty() {
                    note = Some(
                        "No disease-filtered variants found; showing top gene variants."
                            .to_string(),
                    );
                }
            }

            if input.gene_anchor().is_some() && input.disease.is_some() {
                rows.sort_by(|a, b| {
                    variant_significance_rank(a.significance.as_deref())
                        .cmp(&variant_significance_rank(b.significance.as_deref()))
                });
            }
            Ok(SectionResult {
                rows: to_json_array(rows)?,
                total,
                note,
            })
        }
        SectionKind::Disease => {
            let query = input.disease.as_deref().ok_or_else(|| {
                BioMcpError::InvalidArgument(
                    "No disease anchor available for disease search.".into(),
                )
            })?;
            let filters = crate::entities::disease::DiseaseSearchFilters {
                query: Some(query.to_string()),
                ..Default::default()
            };
            let page = crate::entities::disease::search_page(&filters, limit, 0).await?;
            Ok(SectionResult::new(to_json_array(page.results)?, page.total))
        }
        SectionKind::Drug => {
            let filters = crate::entities::drug::DrugSearchFilters {
                query: input.drug_query().map(str::to_string),
                target: input.gene_anchor().map(str::to_string),
                indication: input.disease.clone(),
                ..Default::default()
            };
            let has_filter = filters
                .query
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
                || filters
                    .target
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
                || filters
                    .indication
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty());
            if !has_filter {
                return Err(BioMcpError::InvalidArgument(
                    "No filters available for drug search.".into(),
                ));
            }
            let page = crate::entities::drug::search_page(&filters, limit, 0).await?;
            let rows = refine_drug_results(page.results, input.drug.as_deref(), limit);
            let total = if input.drug.is_some() {
                Some(rows.len())
            } else {
                page.total
            };
            Ok(SectionResult::new(to_json_array(rows)?, total))
        }
        SectionKind::Trial => {
            let base_filters = crate::entities::trial::TrialSearchFilters {
                condition: input.trial_condition_query().map(str::to_string),
                intervention: input.drug.clone(),
                biomarker: input.gene_anchor().map(str::to_string),
                mutation: input.variant_trial_query(),
                date_from: input.since.clone(),
                source: crate::entities::trial::TrialSource::ClinicalTrialsGov,
                ..Default::default()
            };
            let preferred_filters = crate::entities::trial::TrialSearchFilters {
                status: Some(
                    "RECRUITING,ACTIVE_NOT_RECRUITING,ENROLLING_BY_INVITATION,NOT_YET_RECRUITING"
                        .to_string(),
                ),
                ..base_filters.clone()
            };

            let mut rows = Vec::new();
            let mut total = None;
            let mut preferred_error: Option<BioMcpError> = None;

            match crate::entities::trial::search_page(&preferred_filters, limit, 0, None).await {
                Ok(page) => {
                    total = page.total;
                    rows = page.results;
                }
                Err(err) => {
                    preferred_error = Some(err);
                }
            }

            if rows.len() < limit {
                let backfill_fetch = if input.counts_only {
                    limit
                } else {
                    input.limit.saturating_mul(3).min(MAX_SEARCH_ALL_LIMIT)
                };
                match crate::entities::trial::search_page(&base_filters, backfill_fetch, 0, None)
                    .await
                {
                    Ok(page) => {
                        total = total.or(page.total);
                        rows = merge_trial_backfill_rows(rows, page.results, limit);
                    }
                    Err(err) if rows.is_empty() => {
                        return Err(preferred_error.unwrap_or(err));
                    }
                    Err(_) => {}
                }
            }

            if rows.is_empty()
                && let Some(err) = preferred_error
            {
                return Err(err);
            }

            rows.truncate(limit);
            Ok(SectionResult::new(to_json_array(rows)?, total))
        }
        SectionKind::Article => {
            let filters = article_filters(input);
            let page = crate::entities::article::search_page(
                &filters,
                limit,
                0,
                crate::entities::article::ArticleSourceFilter::All,
            )
            .await?;
            Ok(SectionResult::new(to_json_array(page.results)?, page.total))
        }
        SectionKind::Pathway => {
            let query = input.gene_anchor().ok_or_else(|| {
                BioMcpError::InvalidArgument("No gene anchor available for pathway search.".into())
            })?;
            let filters = crate::entities::pathway::PathwaySearchFilters {
                query: Some(query.to_string()),
                ..Default::default()
            };
            let pathway_limit = limit.min(25);
            let (results, total) =
                crate::entities::pathway::search_with_filters(&filters, pathway_limit).await?;
            Ok(SectionResult::new(to_json_array(results)?, total))
        }
        SectionKind::Pgx => {
            let filters = crate::entities::pgx::PgxSearchFilters {
                gene: input.gene_anchor().map(str::to_string),
                drug: input.drug.clone(),
                ..Default::default()
            };
            let page = crate::entities::pgx::search_page(&filters, limit, 0).await?;
            Ok(SectionResult::new(to_json_array(page.results)?, page.total))
        }
        SectionKind::Gwas => {
            let trait_query = input.disease.as_deref().ok_or_else(|| {
                BioMcpError::InvalidArgument("No disease anchor available for GWAS search.".into())
            })?;
            let filters = crate::entities::variant::GwasSearchFilters {
                gene: input.gene_anchor().map(str::to_string),
                trait_query: Some(trait_query.to_string()),
                region: None,
                p_value: None,
            };
            let page = crate::entities::variant::search_gwas_page(&filters, limit, 0).await?;
            let mut rows = page.results;
            if let Some(disease) = input.disease.as_deref() {
                rows.retain(|row| trait_matches_disease_query(row, disease));
            }
            let mut rows = dedupe_gwas_rows(rows);
            rows.truncate(limit);
            let total = rows.len();
            Ok(SectionResult::new(to_json_array(rows)?, Some(total)))
        }
        SectionKind::AdverseEvent => {
            let drug = input.drug.as_deref().ok_or_else(|| {
                BioMcpError::InvalidArgument(
                    "No drug anchor available for adverse-event search.".into(),
                )
            })?;
            let filters = crate::entities::adverse_event::AdverseEventSearchFilters {
                drug: Some(drug.to_string()),
                since: input.since.clone(),
                ..Default::default()
            };
            let grouped = crate::entities::adverse_event::search_count(
                &filters,
                "patient.reaction.reactionmeddrapt",
                limit,
            )
            .await?;
            let total = grouped.buckets.len();
            let rows = grouped
                .buckets
                .into_iter()
                .map(|bucket| {
                    json!({
                        "reaction": bucket.value,
                        "count": bucket.count
                    })
                })
                .collect::<Vec<_>>();
            Ok(SectionResult::new(rows, Some(total)))
        }
    }
}

fn section_fetch_limit(kind: SectionKind, input: &PreparedInput) -> usize {
    if !input.counts_only {
        return input.limit;
    }

    // Counts-only does not skip backend fetches entirely because some sections derive
    // counts locally and markdown still needs stable follow-up commands, but it can
    // safely collapse sections backed by stable totals to a single fetched row.
    match kind {
        SectionKind::Gene | SectionKind::Disease | SectionKind::Trial | SectionKind::Pgx => 1,
        SectionKind::Article if !input.debug_plan => 1,
        SectionKind::Variant
        | SectionKind::Drug
        | SectionKind::Pathway
        | SectionKind::Gwas
        | SectionKind::AdverseEvent
        | SectionKind::Article => input.limit,
    }
}

fn merge_trial_backfill_rows(
    mut preferred: Vec<crate::entities::trial::TrialSearchResult>,
    backfill: Vec<crate::entities::trial::TrialSearchResult>,
    limit: usize,
) -> Vec<crate::entities::trial::TrialSearchResult> {
    preferred.truncate(limit);
    if preferred.len() >= limit {
        return preferred;
    }

    let mut seen = preferred
        .iter()
        .map(|row| row.nct_id.clone())
        .collect::<HashSet<_>>();
    for row in backfill {
        if preferred.len() >= limit {
            break;
        }
        if seen.insert(row.nct_id.clone()) {
            preferred.push(row);
        }
    }
    preferred
}

fn build_links(
    kind: SectionKind,
    input: &PreparedInput,
    results: &[Value],
    _search_self: &str,
) -> Vec<SearchAllLink> {
    let mut links = Vec::new();

    // get.top: inspect the top result in detail (teaches the `get` verb)
    if let Some(cmd) = top_get_command(kind, input, results) {
        links.push(SearchAllLink {
            rel: "get.top".to_string(),
            title: format!("Inspect top {}", kind.entity()),
            command: cmd,
        });
    }

    // Cross-entity links: use `search` commands (not thin wrappers like
    // `disease trials`) so users can append filters like -s, -p, etc.
    match kind {
        SectionKind::Gene => {
            if let Some(gene) = first_string(results, &["symbol"]).or(input.gene_anchor()) {
                links.push(SearchAllLink {
                    rel: "cross.trials".to_string(),
                    title: "Gene-linked trials".to_string(),
                    command: format!("biomcp search trial --biomarker {}", quote_arg(gene)),
                });
            }
        }
        SectionKind::Disease => {
            // Prefer name over ID because ClinicalTrials.gov
            // doesn't understand MONDO IDs.
            if let Some(disease) = first_string(results, &["name"]).or(input.disease.as_deref()) {
                links.push(SearchAllLink {
                    rel: "cross.trials".to_string(),
                    title: "Disease-linked trials".to_string(),
                    command: format!("biomcp search trial -c {}", quote_arg(disease)),
                });
            }
        }
        SectionKind::Drug => {
            // Prefer user's input drug name for AE cross-links: FAERS indexes by
            // generic/brand name, not salt forms. DrugBank canonical names (e.g.
            // "dabrafenib mesylate") return far fewer FAERS reports than the generic
            // name ("dabrafenib": 4K+ vs 15 reports).
            if let Some(drug) = input
                .drug
                .as_deref()
                .or_else(|| first_string(results, &["name"]))
            {
                links.push(SearchAllLink {
                    rel: "cross.adverse-events".to_string(),
                    title: "Top adverse events".to_string(),
                    command: format!("biomcp search adverse-event -d {}", quote_arg(drug)),
                });
            }
        }
        _ => {}
    }

    // Filter hints: show useful unused filters for this entity
    links.extend(filter_hints(kind, input));

    links
}

/// Return contextual filter hints — filters the user *could* add to narrow or
/// pivot the search. Each hint teaches a different capability of the entity
/// search rather than repeating the query the user already ran.
fn filter_hints(kind: SectionKind, input: &PreparedInput) -> Vec<SearchAllLink> {
    let mut hints = Vec::new();

    match kind {
        SectionKind::Variant => {
            let variant_base = variant_base_args(input);
            let significance_command = if variant_base.is_empty() {
                "biomcp search variant --significance pathogenic".to_string()
            } else {
                format!("biomcp search variant {variant_base} --significance pathogenic")
            };
            if input.disease.is_none() {
                hints.push(SearchAllLink {
                    rel: "filter.hint".to_string(),
                    title: "Filter by significance".to_string(),
                    command: significance_command,
                });
            }
            // Population frequency is always useful context
            let rarity_command = if variant_base.is_empty() {
                "biomcp search variant --max-frequency 0.01".to_string()
            } else {
                format!("biomcp search variant {variant_base} --max-frequency 0.01")
            };
            hints.push(SearchAllLink {
                rel: "filter.hint".to_string(),
                title: "Rare variants only".to_string(),
                command: rarity_command,
            });
        }
        SectionKind::Trial => {
            if input.since.is_none() {
                hints.push(SearchAllLink {
                    rel: "filter.hint".to_string(),
                    title: "Recruiting only".to_string(),
                    command: format!(
                        "{} -s recruiting",
                        canonical_search_command(kind, input, input.limit)
                    ),
                });
            }
            hints.push(SearchAllLink {
                rel: "filter.hint".to_string(),
                title: "Phase 3 trials".to_string(),
                command: format!(
                    "{} -p phase3",
                    canonical_search_command(kind, input, input.limit)
                ),
            });
        }
        SectionKind::Article => {
            hints.push(SearchAllLink {
                rel: "filter.hint".to_string(),
                title: "Clinical trials only".to_string(),
                command: format!(
                    "{} --type research-article",
                    canonical_search_command(kind, input, input.limit)
                ),
            });
            hints.push(SearchAllLink {
                rel: "filter.hint".to_string(),
                title: "Reviews & meta-analyses".to_string(),
                command: format!(
                    "{} --type review",
                    canonical_search_command(kind, input, input.limit)
                ),
            });
        }
        SectionKind::Drug => {}
        SectionKind::AdverseEvent => {
            if let Some(drug) = input.drug.as_deref() {
                hints.push(SearchAllLink {
                    rel: "filter.hint".to_string(),
                    title: "Top reactions by frequency".to_string(),
                    command: adverse_event_count_command(input),
                });
                hints.push(SearchAllLink {
                    rel: "filter.hint".to_string(),
                    title: "Serious reports only".to_string(),
                    command: format!(
                        "biomcp search adverse-event --drug {} --serious",
                        quote_arg(drug)
                    ),
                });
            }
        }
        SectionKind::Gene
        | SectionKind::Disease
        | SectionKind::Pathway
        | SectionKind::Pgx
        | SectionKind::Gwas => {}
    }

    hints
}

/// Build the base args for a variant search command from the current input.
fn variant_base_args(input: &PreparedInput) -> String {
    let mut args = Vec::new();
    if let Some(gene) = input.gene_anchor() {
        args.push(format!("--gene {}", quote_arg(gene)));
    }
    if let Some(condition) = input.disease.as_deref() {
        args.push(format!("--condition {}", quote_arg(condition)));
    }
    if let Some(therapy) = input.drug.as_deref() {
        args.push(format!("--therapy {}", quote_arg(therapy)));
    }
    args.join(" ")
}

fn top_get_command(kind: SectionKind, input: &PreparedInput, results: &[Value]) -> Option<String> {
    match kind {
        SectionKind::Gene => first_string(results, &["symbol"])
            .or(input.gene_anchor())
            .map(|id| format!("biomcp get gene {}", quote_arg(id))),
        SectionKind::Variant => first_gettable_variant_id(results)
            .or_else(|| {
                input
                    .variant
                    .as_deref()
                    .filter(|id| !is_civic_variant_id(id))
                    .map(str::to_string)
            })
            .map(|id| format!("biomcp get variant {}", quote_arg(&id))),
        SectionKind::Disease => first_string(results, &["id", "name"])
            .or(input.disease.as_deref())
            .map(|id| format!("biomcp get disease {}", quote_arg(id))),
        SectionKind::Drug => input
            .drug
            .as_deref()
            .map(str::trim)
            .filter(|id| !id.is_empty())
            .map(str::to_string)
            .or_else(|| {
                crate::render::markdown::preferred_drug_name(
                    results
                        .iter()
                        .filter_map(|row| row.as_object())
                        .filter_map(|obj| obj.get("name"))
                        .filter_map(Value::as_str),
                    input.drug.as_deref(),
                )
            })
            .or_else(|| first_string(results, &["name"]).map(str::to_string))
            .map(|id| format!("biomcp get drug {}", quote_arg(&id))),
        SectionKind::Pathway => {
            first_string(results, &["id"]).map(|id| format!("biomcp get pathway {}", quote_arg(id)))
        }
        SectionKind::Trial
        | SectionKind::Article
        | SectionKind::Pgx
        | SectionKind::Gwas
        | SectionKind::AdverseEvent => None,
    }
}

fn canonical_search_command(kind: SectionKind, input: &PreparedInput, limit: usize) -> String {
    let mut args: Vec<String> = vec!["biomcp".into(), "search".into(), kind.entity().into()];

    match kind {
        SectionKind::Gene => {
            push_opt(&mut args, "--query", input.gene_anchor());
        }
        SectionKind::Variant => {
            let gene = input.gene_anchor();
            let hgvsp = input
                .variant_context
                .as_ref()
                .and_then(|ctx| ctx.parsed_change.as_deref());
            let condition = input.disease.as_deref();
            let therapy = input.drug.as_deref();

            if gene.is_none()
                && hgvsp.is_none()
                && condition.is_none()
                && therapy.is_none()
                && let Some(raw_variant) = input.variant.as_deref()
            {
                // Keep the anchor in search.self even when we cannot parse gene/change.
                args.push(quote_arg(raw_variant));
            }

            push_opt(&mut args, "--gene", gene);
            push_opt(&mut args, "--hgvsp", hgvsp);
            push_opt(&mut args, "--condition", condition);
            push_opt(&mut args, "--therapy", therapy);
        }
        SectionKind::Disease => {
            push_opt(&mut args, "--query", input.disease.as_deref());
        }
        SectionKind::Drug => {
            push_opt(&mut args, "--query", input.drug_query());
            push_opt(&mut args, "--target", input.gene_anchor());
            push_opt(&mut args, "--indication", input.disease.as_deref());
        }
        SectionKind::Trial => {
            push_opt(&mut args, "--condition", input.trial_condition_query());
            push_opt(&mut args, "--intervention", input.drug.as_deref());
            push_opt(&mut args, "--biomarker", input.gene_anchor());
            push_opt_owned(&mut args, "--mutation", input.variant_trial_query());
            push_opt(&mut args, "--since", input.since.as_deref());
        }
        SectionKind::Article => {
            push_opt(&mut args, "--gene", input.gene_anchor());
            push_opt(&mut args, "--disease", input.article_disease_filter());
            push_opt(&mut args, "--drug", input.drug.as_deref());
            push_opt(&mut args, "--keyword", input.article_keyword_filter());
            push_opt(&mut args, "--since", input.since.as_deref());
        }
        SectionKind::Pathway => {
            push_opt(&mut args, "--query", input.gene_anchor());
        }
        SectionKind::Pgx => {
            push_opt(&mut args, "--gene", input.gene_anchor());
            push_opt(&mut args, "--drug", input.drug.as_deref());
        }
        SectionKind::Gwas => {
            push_opt(&mut args, "--gene", input.gene_anchor());
            push_opt(&mut args, "--trait", input.disease.as_deref());
        }
        SectionKind::AdverseEvent => {
            push_opt(&mut args, "--drug", input.drug.as_deref());
            push_opt(&mut args, "--since", input.since.as_deref());
        }
    }

    // Clamp to entity-specific maximums so generated commands are always runnable.
    let clamped = match kind {
        SectionKind::Pathway => limit.min(25),
        _ => limit,
    };
    args.push("--limit".into());
    args.push(clamped.to_string());
    args.join(" ")
}

fn adverse_event_count_command(input: &PreparedInput) -> String {
    let mut args = vec![
        "biomcp".to_string(),
        "search".to_string(),
        "adverse-event".to_string(),
    ];
    push_opt(&mut args, "--drug", input.drug.as_deref());
    push_opt(&mut args, "--since", input.since.as_deref());
    args.push("--count".to_string());
    args.push("patient.reaction.reactionmeddrapt".to_string());
    args.push("--limit".to_string());
    args.push(EXPAND_LIMIT.to_string());
    args.join(" ")
}

fn normalize_slot(value: Option<String>) -> Option<String> {
    value.and_then(|raw| {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn to_json_array<T: Serialize>(rows: Vec<T>) -> Result<Vec<Value>, BioMcpError> {
    let value = serde_json::to_value(rows)?;
    Ok(value.as_array().cloned().unwrap_or_default())
}

fn first_string<'a>(results: &'a [Value], fields: &[&str]) -> Option<&'a str> {
    let row = results.first()?.as_object()?;
    for field in fields {
        if let Some(value) = row
            .get(*field)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return Some(value);
        }
    }
    None
}

fn push_opt(args: &mut Vec<String>, flag: &str, value: Option<&str>) {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return;
    };
    args.push(flag.to_string());
    args.push(quote_arg(value));
}

fn push_opt_owned(args: &mut Vec<String>, flag: &str, value: Option<String>) {
    let Some(value) = value else {
        return;
    };
    push_opt(args, flag, Some(value.as_str()));
}

fn value_str(row: &Value, key: &str) -> String {
    let Some(obj) = row.as_object() else {
        return "-".to_string();
    };
    let Some(value) = obj.get(key) else {
        return "-".to_string();
    };
    format_value(value)
}

fn value_str_or(row: &Value, primary: &str, fallback: String) -> String {
    let value = value_str(row, primary);
    if value == "-" { fallback } else { value }
}

fn value_p_value(row: &Value, key: &str) -> String {
    let Some(obj) = row.as_object() else {
        return "-".to_string();
    };
    let Some(value) = obj.get(key) else {
        return "-".to_string();
    };
    format_search_all_p_value(value)
}

fn format_value(value: &Value) -> String {
    match value {
        Value::Null => "-".to_string(),
        Value::Bool(v) => v.to_string(),
        Value::Number(v) => v.to_string(),
        Value::String(v) => {
            let trimmed = v.trim();
            if trimmed.is_empty() {
                "-".to_string()
            } else {
                trimmed.to_string()
            }
        }
        Value::Array(values) => {
            let joined = values
                .iter()
                .take(3)
                .map(format_value)
                .filter(|v| v != "-")
                .collect::<Vec<_>>()
                .join("; ");
            if joined.is_empty() {
                "-".to_string()
            } else {
                joined
            }
        }
        Value::Object(_) => value.to_string(),
    }
}

fn is_empty_cell(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.is_empty() || trimmed == "-"
}

fn format_search_all_p_value(value: &Value) -> String {
    let parsed = match value {
        Value::Number(v) => v.as_f64(),
        Value::String(v) => v.trim().parse::<f64>().ok(),
        _ => None,
    };
    let Some(mut parsed) = parsed else {
        return format_value(value);
    };
    if !parsed.is_finite() {
        return format_value(value);
    }
    if parsed == -0.0 {
        parsed = 0.0;
    }
    if parsed == 0.0 {
        return "0".to_string();
    }
    if parsed.abs() < 0.001 {
        return trim_scientific_notation(parsed);
    }
    if parsed.abs() < 0.01 {
        return trim_trailing_decimal_zeros(format!("{parsed:.4}"));
    }
    trim_trailing_decimal_zeros(format!("{parsed:.3}"))
}

fn trim_scientific_notation(value: f64) -> String {
    let rendered = format!("{value:.2e}");
    let Some((mantissa, exponent)) = rendered.split_once('e') else {
        return rendered;
    };
    let mantissa = trim_trailing_decimal_zeros(mantissa.to_string());
    format!("{mantissa}e{exponent}")
}

fn trim_trailing_decimal_zeros(mut rendered: String) -> String {
    if rendered.contains('.') {
        while rendered.ends_with('0') {
            rendered.pop();
        }
        if rendered.ends_with('.') {
            rendered.pop();
        }
    }
    if rendered.is_empty() {
        "0".to_string()
    } else {
        rendered
    }
}

fn is_civic_variant_id(id: &str) -> bool {
    id.trim().to_ascii_uppercase().starts_with("CIVIC_VARIANT:")
}

fn first_gettable_variant_id(results: &[Value]) -> Option<String> {
    results
        .iter()
        .filter_map(|row| row.as_object())
        .filter_map(|obj| obj.get("id"))
        .filter_map(Value::as_str)
        .map(str::trim)
        .find(|id| !id.is_empty() && !is_civic_variant_id(id))
        .map(str::to_string)
}

fn refine_drug_results(
    mut rows: Vec<crate::entities::drug::DrugSearchResult>,
    preferred: Option<&str>,
    limit: usize,
) -> Vec<crate::entities::drug::DrugSearchResult> {
    let Some(preferred_lower) = preferred
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_ascii_lowercase)
    else {
        rows.truncate(limit);
        return rows;
    };

    rows.sort_by_key(|row| {
        crate::render::markdown::drug_parent_match_rank(&row.name, &preferred_lower)
            .unwrap_or(u8::MAX)
    });

    let has_parent_like = rows.iter().any(|row| {
        crate::render::markdown::drug_parent_match_rank(&row.name, &preferred_lower)
            .is_some_and(|rank| rank < 3)
    });

    if has_parent_like {
        rows.retain(|row| {
            let normalized = row.name.trim().to_ascii_lowercase();
            if !normalized.contains(&preferred_lower) {
                return true;
            }
            crate::render::markdown::drug_parent_match_rank(&row.name, &preferred_lower)
                .is_some_and(|rank| rank < 3)
        });
    }

    rows.truncate(limit);
    rows
}

fn variant_significance_rank(significance: Option<&str>) -> u8 {
    let Some(significance) = significance.map(str::trim).filter(|v| !v.is_empty()) else {
        return 50;
    };
    let normalized = significance.to_ascii_lowercase();
    if normalized == "pathogenic"
        || (normalized.contains("pathogenic") && !normalized.contains("likely"))
    {
        return 0;
    }
    if normalized.contains("likely pathogenic") {
        return 1;
    }
    if normalized == "vus"
        || normalized.contains("uncertain")
        || normalized.contains("unknown significance")
    {
        return 2;
    }
    if normalized.contains("likely benign") {
        return 3;
    }
    if normalized == "benign" || normalized.contains("benign") {
        return 4;
    }
    5
}

fn trait_matches_disease_query(
    row: &crate::entities::variant::VariantGwasAssociation,
    disease: &str,
) -> bool {
    let disease = disease.trim();
    if disease.is_empty() {
        return true;
    }
    row.trait_name
        .as_deref()
        .map(str::trim)
        .filter(|trait_name| !trait_name.is_empty())
        .is_some_and(|trait_name| {
            trait_name
                .to_ascii_lowercase()
                .contains(&disease.to_ascii_lowercase())
        })
}

fn dedupe_gwas_rows(
    rows: Vec<crate::entities::variant::VariantGwasAssociation>,
) -> Vec<crate::entities::variant::VariantGwasAssociation> {
    let mut out: Vec<crate::entities::variant::VariantGwasAssociation> = Vec::new();
    let mut index_by_key: HashMap<(String, String), usize> = HashMap::new();

    for row in rows {
        let rsid_key = row.rsid.trim().to_ascii_lowercase();
        let trait_key = row
            .trait_name
            .as_deref()
            .map(str::trim)
            .unwrap_or("")
            .to_ascii_lowercase();

        if rsid_key.is_empty() || trait_key.is_empty() {
            out.push(row);
            continue;
        }

        let key = (rsid_key, trait_key);
        if let Some(existing_idx) = index_by_key.get(&key).copied() {
            let existing_p = out[existing_idx].p_value.unwrap_or(f64::INFINITY);
            let candidate_p = row.p_value.unwrap_or(f64::INFINITY);
            if candidate_p < existing_p {
                out[existing_idx] = row;
            }
            continue;
        }

        index_by_key.insert(key, out.len());
        out.push(row);
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::article::{
        ArticleRankingMetadata, ArticleRankingMode, ArticleSearchResult, ArticleSource,
    };
    use crate::entities::drug::DrugSearchResult;
    use crate::entities::trial::TrialSearchResult;
    use crate::entities::variant::VariantGwasAssociation;
    use crate::test_support::set_env_var;
    use serde_json::json;
    use tokio::sync::MutexGuard;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn env_lock_async() -> MutexGuard<'static, ()> {
        crate::test_support::env_lock().lock().await
    }

    fn input_with_gene() -> SearchAllInput {
        SearchAllInput {
            gene: Some("BRAF".to_string()),
            variant: None,
            disease: None,
            drug: None,
            keyword: None,
            since: None,
            limit: 3,
            counts_only: false,
            debug_plan: false,
        }
    }

    #[test]
    fn build_dispatch_plan_gene_only_matches_contract() {
        let plan = build_dispatch_plan(&input_with_gene());
        let entities = plan.iter().map(|spec| spec.entity).collect::<Vec<_>>();
        assert_eq!(
            entities,
            vec![
                "gene", "variant", "drug", "trial", "article", "pathway", "pgx"
            ]
        );
    }

    #[test]
    fn build_dispatch_plan_keyword_only_routes_to_article() {
        let plan = build_dispatch_plan(&SearchAllInput {
            gene: None,
            variant: None,
            disease: None,
            drug: None,
            keyword: Some("resistance".to_string()),
            since: None,
            limit: 3,
            counts_only: false,
            debug_plan: false,
        });
        let entities = plan.iter().map(|spec| spec.entity).collect::<Vec<_>>();
        assert_eq!(entities, vec!["article"]);
    }

    #[test]
    fn article_filters_follow_keyword_dependent_ranking_defaults() {
        let keyword_prepared = PreparedInput::new(&SearchAllInput {
            gene: None,
            variant: None,
            disease: None,
            drug: None,
            keyword: Some("checkpoint inhibitor".to_string()),
            since: None,
            limit: 3,
            counts_only: false,
            debug_plan: false,
        })
        .expect("valid prepared input");
        let keyword_filters = article_filters(&keyword_prepared);
        assert_eq!(
            crate::entities::article::article_effective_ranking_mode(&keyword_filters),
            Some(ArticleRankingMode::Hybrid)
        );

        let gene_prepared = PreparedInput::new(&input_with_gene()).expect("valid prepared input");
        let gene_filters = article_filters(&gene_prepared);
        assert_eq!(
            crate::entities::article::article_effective_ranking_mode(&gene_filters),
            Some(ArticleRankingMode::Lexical)
        );
    }

    #[test]
    fn build_dispatch_plan_variant_with_gene_fanout() {
        let plan = build_dispatch_plan(&SearchAllInput {
            gene: None,
            variant: Some("BRAF V600E".to_string()),
            disease: None,
            drug: None,
            keyword: None,
            since: None,
            limit: 3,
            counts_only: false,
            debug_plan: false,
        });
        let entities = plan.iter().map(|spec| spec.entity).collect::<Vec<_>>();
        assert_eq!(
            entities,
            vec!["variant", "gene", "trial", "article", "drug", "pathway"]
        );
    }

    #[test]
    fn prepared_input_rejects_empty_typed_slots() {
        let err = PreparedInput::new(&SearchAllInput {
            gene: None,
            variant: None,
            disease: None,
            drug: None,
            keyword: None,
            since: None,
            limit: 3,
            counts_only: false,
            debug_plan: false,
        })
        .expect_err("expected validation error");
        assert!(err.to_string().contains("at least one typed slot"));
    }

    #[test]
    fn canonical_drug_command_stays_typed_only_with_gene_anchor() {
        let prepared = PreparedInput::new(&SearchAllInput {
            gene: Some("BRAF".to_string()),
            variant: None,
            disease: None,
            drug: None,
            keyword: Some("resistance".to_string()),
            since: None,
            limit: 3,
            counts_only: false,
            debug_plan: false,
        })
        .expect("valid prepared input");

        let command = canonical_search_command(SectionKind::Drug, &prepared, 3);
        assert!(command.contains("search drug"));
        assert!(command.contains("--target BRAF"));
        assert!(!command.contains("--query resistance"));
    }

    #[test]
    fn canonical_article_command_keeps_keyword_only_search() {
        let prepared = PreparedInput::new(&SearchAllInput {
            gene: None,
            variant: None,
            disease: None,
            drug: None,
            keyword: Some("resistance".to_string()),
            since: None,
            limit: 3,
            counts_only: false,
            debug_plan: false,
        })
        .expect("valid prepared input");

        let command = canonical_search_command(SectionKind::Article, &prepared, 3);
        assert!(command.contains("search article"));
        assert!(command.contains("--keyword resistance"));
    }

    #[test]
    fn canonical_article_command_dedupes_shared_disease_keyword_token() {
        let prepared = PreparedInput::new(&SearchAllInput {
            gene: None,
            variant: None,
            disease: Some("cancer".to_string()),
            drug: None,
            keyword: Some("Cancer".to_string()),
            since: None,
            limit: 3,
            counts_only: false,
            debug_plan: false,
        })
        .expect("valid prepared input");

        let command = canonical_search_command(SectionKind::Article, &prepared, 3);
        assert!(command.contains("search article"));
        assert!(!command.contains("--disease cancer"));
        assert!(command.contains("--keyword Cancer"));
    }

    #[test]
    fn canonical_article_command_keeps_distinct_disease_and_keyword_filters() {
        let prepared = PreparedInput::new(&SearchAllInput {
            gene: None,
            variant: None,
            disease: Some("melanoma".to_string()),
            drug: None,
            keyword: Some("BRAF".to_string()),
            since: None,
            limit: 3,
            counts_only: false,
            debug_plan: false,
        })
        .expect("valid prepared input");

        let command = canonical_search_command(SectionKind::Article, &prepared, 3);
        assert!(command.contains("--disease melanoma"));
        assert!(command.contains("--keyword BRAF"));
    }

    #[test]
    fn canonical_trial_command_stays_typed_only_with_distinct_keyword() {
        let prepared = PreparedInput::new(&SearchAllInput {
            gene: None,
            variant: None,
            disease: Some("melanoma".to_string()),
            drug: None,
            keyword: Some("BRAF".to_string()),
            since: None,
            limit: 3,
            counts_only: false,
            debug_plan: false,
        })
        .expect("valid prepared input");

        let command = canonical_search_command(SectionKind::Trial, &prepared, 3);
        assert!(command.contains("--condition melanoma"));
        assert!(!command.contains("melanoma BRAF"));
        assert!(!command.contains("--condition BRAF"));
    }

    #[test]
    fn canonical_variant_command_preserves_unparsed_anchor() {
        let prepared = PreparedInput::new(&SearchAllInput {
            gene: None,
            variant: Some("rs121913529".to_string()),
            disease: None,
            drug: None,
            keyword: None,
            since: None,
            limit: 3,
            counts_only: false,
            debug_plan: false,
        })
        .expect("valid prepared input");

        let command = canonical_search_command(SectionKind::Variant, &prepared, 3);
        assert_eq!(command, "biomcp search variant rs121913529 --limit 3");
    }

    #[test]
    fn canonical_article_command_quotes_apostrophe_keyword_for_shell_safety() {
        let prepared = PreparedInput::new(&SearchAllInput {
            gene: None,
            variant: None,
            disease: None,
            drug: None,
            keyword: Some("Graves'".to_string()),
            since: None,
            limit: 3,
            counts_only: false,
            debug_plan: false,
        })
        .expect("valid prepared input");

        let command = canonical_search_command(SectionKind::Article, &prepared, 3);
        assert_eq!(
            command,
            "biomcp search article --keyword \"Graves'\" --limit 3"
        );
    }

    #[test]
    fn to_json_array_preserves_article_source_and_ranking_metadata() {
        let rows = to_json_array(vec![ArticleSearchResult {
            pmid: "22663011".into(),
            pmcid: Some("PMC9984800".into()),
            doi: Some("10.1056/NEJMoa1203421".into()),
            title: "BRAF melanoma review".into(),
            journal: Some("Journal".into()),
            date: Some("2025-01-01".into()),
            first_index_date: None,
            citation_count: Some(12),
            influential_citation_count: Some(4),
            source: ArticleSource::EuropePmc,
            matched_sources: vec![ArticleSource::EuropePmc, ArticleSource::SemanticScholar],
            score: None,
            is_retracted: Some(false),
            abstract_snippet: Some("Abstract".into()),
            ranking: Some(ArticleRankingMetadata {
                directness_tier: 3,
                anchor_count: 2,
                title_anchor_hits: 2,
                abstract_anchor_hits: 0,
                combined_anchor_hits: 2,
                all_anchors_in_title: true,
                all_anchors_in_text: true,
                study_or_review_cue: true,
                pubmed_rescue: false,
                pubmed_rescue_kind: None,
                pubmed_source_position: None,
                mode: Some(ArticleRankingMode::Lexical),
                semantic_score: None,
                lexical_score: None,
                citation_score: None,
                position_score: None,
                composite_score: None,
                avg_source_rank: None,
            }),
            normalized_title: "braf melanoma review".into(),
            normalized_abstract: "abstract".into(),
            publication_type: Some("Review".into()),
            source_local_position: 0,
        }])
        .expect("article rows should serialize");

        assert_eq!(
            rows[0]["source"],
            serde_json::Value::String("europepmc".into())
        );
        assert_eq!(
            rows[0]["matched_sources"][1],
            serde_json::Value::String("semanticscholar".into())
        );
        assert_eq!(rows[0]["ranking"]["study_or_review_cue"], true);
        assert_eq!(rows[0]["ranking"]["pubmed_rescue"], false);
        assert!(rows[0]["ranking"]["pubmed_rescue_kind"].is_null());
        assert!(rows[0]["ranking"]["pubmed_source_position"].is_null());
    }

    #[test]
    fn counts_only_json_projection_omits_results_links_and_total() {
        let results = SearchAllResults {
            query: "gene=BRAF".to_string(),
            sections: vec![SearchAllSection {
                entity: "gene".to_string(),
                label: "Genes".to_string(),
                count: 1,
                total: Some(12),
                error: None,
                note: Some("Counts-only projection".to_string()),
                results: vec![json!({"symbol":"BRAF"})],
                links: vec![SearchAllLink {
                    rel: "get.top".to_string(),
                    title: "Inspect BRAF".to_string(),
                    command: "biomcp get gene BRAF".to_string(),
                }],
            }],
            searches_dispatched: 1,
            searches_with_results: 1,
            wall_time_ms: 42,
            debug_plan: None,
        };

        let value = serde_json::to_value(counts_only_json(&results)).expect("counts-only json");
        let section = &value["sections"][0];

        assert_eq!(section["entity"], "gene");
        assert_eq!(section["label"], "Genes");
        assert_eq!(section["count"], 12);
        assert_eq!(section["note"], "Counts-only projection");
        assert!(section.get("results").is_none());
        assert!(section.get("links").is_none());
        assert!(section.get("total").is_none());
    }

    #[test]
    fn counts_only_json_projection_preserves_debug_plan() {
        let results = SearchAllResults {
            query: "gene=BRAF".to_string(),
            sections: vec![SearchAllSection {
                entity: "article".to_string(),
                label: "Articles".to_string(),
                count: 1,
                total: Some(5),
                error: None,
                note: None,
                results: vec![json!({"pmid":"22663011"})],
                links: Vec::new(),
            }],
            searches_dispatched: 1,
            searches_with_results: 1,
            wall_time_ms: 42,
            debug_plan: Some(DebugPlan {
                surface: "search_all",
                query: "gene=BRAF".to_string(),
                anchor: Some("gene"),
                legs: vec![DebugPlanLeg {
                    leg: "article".to_string(),
                    entity: "article".to_string(),
                    filters: vec!["gene=BRAF".to_string()],
                    routing: vec!["anchor=gene".to_string()],
                    sources: vec!["PubMed".to_string()],
                    matched_sources: vec!["PubMed".to_string()],
                    count: 1,
                    total: Some(5),
                    note: None,
                    error: None,
                }],
            }),
        };

        let value = serde_json::to_value(counts_only_json(&results)).expect("counts-only json");
        assert!(value.get("debug_plan").is_some());
        assert_eq!(value["debug_plan"]["anchor"], "gene");
        assert_eq!(value["debug_plan"]["legs"][0]["leg"], "article");
    }

    #[test]
    fn section_fetch_limit_reduces_only_safe_counts_only_sections() {
        let counts_only = PreparedInput::new(&SearchAllInput {
            gene: Some("BRAF".to_string()),
            variant: None,
            disease: Some("melanoma".to_string()),
            drug: None,
            keyword: None,
            since: None,
            limit: 7,
            counts_only: true,
            debug_plan: false,
        })
        .expect("valid prepared input");
        let debug_plan = PreparedInput::new(&SearchAllInput {
            gene: Some("BRAF".to_string()),
            variant: None,
            disease: Some("melanoma".to_string()),
            drug: None,
            keyword: None,
            since: None,
            limit: 7,
            counts_only: true,
            debug_plan: true,
        })
        .expect("valid prepared input");
        let full_fetch = PreparedInput::new(&SearchAllInput {
            gene: Some("BRAF".to_string()),
            variant: None,
            disease: Some("melanoma".to_string()),
            drug: None,
            keyword: None,
            since: None,
            limit: 7,
            counts_only: false,
            debug_plan: false,
        })
        .expect("valid prepared input");

        for kind in [
            SectionKind::Gene,
            SectionKind::Disease,
            SectionKind::Trial,
            SectionKind::Pgx,
        ] {
            assert_eq!(section_fetch_limit(kind, &counts_only), 1, "{kind:?}");
        }
        assert_eq!(section_fetch_limit(SectionKind::Article, &counts_only), 1);
        assert_eq!(section_fetch_limit(SectionKind::Article, &debug_plan), 7);

        for kind in [
            SectionKind::Variant,
            SectionKind::Drug,
            SectionKind::Pathway,
            SectionKind::Gwas,
            SectionKind::AdverseEvent,
        ] {
            assert_eq!(section_fetch_limit(kind, &counts_only), 7, "{kind:?}");
            assert_eq!(section_fetch_limit(kind, &full_fetch), 7, "{kind:?}");
        }
    }

    #[test]
    fn build_result_plan_includes_fallback_and_article_matched_sources() {
        let prepared = PreparedInput::new(&SearchAllInput {
            gene: Some("BRAF".to_string()),
            variant: None,
            disease: Some("melanoma".to_string()),
            drug: None,
            keyword: None,
            since: None,
            limit: 3,
            counts_only: false,
            debug_plan: true,
        })
        .expect("valid prepared input");
        let sections = vec![
            SearchAllSection {
                entity: "variant".to_string(),
                label: "Variants".to_string(),
                count: 1,
                total: Some(5),
                error: None,
                note: Some(
                    "No disease-filtered variants found; showing top gene variants.".to_string(),
                ),
                results: vec![json!({"id":"rs113488022","gene":"BRAF"})],
                links: Vec::new(),
            },
            SearchAllSection {
                entity: "article".to_string(),
                label: "Articles".to_string(),
                count: 1,
                total: Some(10),
                error: None,
                note: None,
                results: vec![json!({
                    "pmid": "22663011",
                    "matched_sources": ["pubtator", "pubmed", "semanticscholar"]
                })],
                links: Vec::new(),
            },
        ];

        let plan = build_result_plan(&prepared, &sections);
        let article_leg = plan
            .legs
            .iter()
            .find(|leg| leg.leg == "article")
            .expect("article leg");

        assert_eq!(plan.surface, "search_all");
        assert_eq!(plan.anchor, Some("gene"));
        assert_eq!(plan.query, "gene=BRAF disease=melanoma");
        assert!(plan.legs[0].routing.contains(&"anchor=gene".to_string()));
        assert!(
            plan.legs[0]
                .routing
                .contains(&"fallback=gene_only_variant_backfill".to_string())
        );
        assert_eq!(
            article_leg.sources,
            vec![
                "PubTator3".to_string(),
                "Europe PMC".to_string(),
                "PubMed".to_string(),
                "Semantic Scholar".to_string()
            ]
        );
        assert_eq!(
            article_leg.matched_sources,
            vec![
                "PubTator3".to_string(),
                "PubMed".to_string(),
                "Semantic Scholar".to_string()
            ]
        );
    }

    #[test]
    fn build_result_plan_keyword_article_leg_includes_litsense2_source_and_match() {
        let prepared = PreparedInput::new(&SearchAllInput {
            gene: None,
            variant: None,
            disease: None,
            drug: None,
            keyword: Some("Hirschsprung disease".to_string()),
            since: None,
            limit: 3,
            counts_only: false,
            debug_plan: true,
        })
        .expect("valid prepared input");
        let sections = vec![SearchAllSection {
            entity: "article".to_string(),
            label: "Articles".to_string(),
            count: 1,
            total: Some(10),
            error: None,
            note: None,
            results: vec![json!({
                "pmid": "36741595",
                "matched_sources": ["litsense2", "pubmed"]
            })],
            links: Vec::new(),
        }];

        let plan = build_result_plan(&prepared, &sections);
        let article_leg = plan
            .legs
            .iter()
            .find(|leg| leg.leg == "article")
            .expect("article leg");

        assert_eq!(plan.surface, "search_all");
        assert_eq!(plan.anchor, Some("keyword"));
        assert_eq!(plan.query, "keyword=Hirschsprung disease");
        assert!(article_leg.sources.contains(&"LitSense2".to_string()));
        assert!(article_leg.sources.contains(&"PubMed".to_string()));
        assert_eq!(
            article_leg.matched_sources,
            vec!["PubMed".to_string(), "LitSense2".to_string()]
        );
    }

    #[test]
    fn build_result_plan_marks_shared_disease_keyword_orientation_fallback() {
        let prepared = PreparedInput::new(&SearchAllInput {
            gene: None,
            variant: None,
            disease: Some("cancer".to_string()),
            drug: None,
            keyword: Some("Cancer".to_string()),
            since: None,
            limit: 3,
            counts_only: false,
            debug_plan: true,
        })
        .expect("valid prepared input");
        let sections = vec![SearchAllSection {
            entity: "article".to_string(),
            label: "Articles".to_string(),
            count: 1,
            total: None,
            error: None,
            note: None,
            results: vec![json!({"pmid":"1"})],
            links: Vec::new(),
        }];

        let plan = build_result_plan(&prepared, &sections);

        let article_leg = plan
            .legs
            .iter()
            .find(|l| l.leg == "article")
            .expect("article leg");
        assert!(
            article_leg
                .routing
                .contains(&"fallback=shared_disease_keyword_orientation".to_string()),
            "article leg routing should include shared-token fallback marker: {:?}",
            article_leg.routing
        );
        assert!(
            !article_leg.filters.contains(&"disease=cancer".to_string()),
            "article leg filters should drop the duplicate disease token: {:?}",
            article_leg.filters
        );
    }

    #[test]
    fn build_result_plan_marks_ungrounded_disease_fallback_on_article_leg() {
        let prepared = PreparedInput::new(&SearchAllInput {
            gene: None,
            variant: None,
            disease: Some("cancer".to_string()),
            drug: None,
            keyword: Some("cancer".to_string()),
            since: None,
            limit: 3,
            counts_only: false,
            debug_plan: true,
        })
        .expect("valid prepared input");
        let sections = vec![
            SearchAllSection {
                entity: "disease".to_string(),
                label: "Diseases".to_string(),
                count: 0,
                total: Some(0),
                error: None,
                note: None,
                results: vec![],
                links: Vec::new(),
            },
            SearchAllSection {
                entity: "article".to_string(),
                label: "Articles".to_string(),
                count: 1,
                total: Some(1),
                error: None,
                note: None,
                results: vec![json!({"pmid":"1"})],
                links: Vec::new(),
            },
        ];

        let plan = build_result_plan(&prepared, &sections);
        let article_leg = plan
            .legs
            .iter()
            .find(|l| l.leg == "article")
            .expect("article leg");
        assert!(
            article_leg
                .routing
                .contains(&"fallback=disease_leg_ungrounded_keyword_survived".to_string()),
            "article leg routing should note the ungrounded disease fallback: {:?}",
            article_leg.routing
        );
    }

    #[test]
    fn build_result_plan_skips_ungrounded_marker_when_disease_leg_errors() {
        let prepared = PreparedInput::new(&SearchAllInput {
            gene: None,
            variant: None,
            disease: Some("cancer".to_string()),
            drug: None,
            keyword: Some("cancer".to_string()),
            since: None,
            limit: 3,
            counts_only: false,
            debug_plan: true,
        })
        .expect("valid prepared input");
        let sections = vec![
            SearchAllSection {
                entity: "disease".to_string(),
                label: "Diseases".to_string(),
                count: 0,
                total: None,
                error: Some("upstream timeout".to_string()),
                note: None,
                results: vec![],
                links: Vec::new(),
            },
            SearchAllSection {
                entity: "article".to_string(),
                label: "Articles".to_string(),
                count: 1,
                total: Some(1),
                error: None,
                note: None,
                results: vec![json!({"pmid":"1"})],
                links: Vec::new(),
            },
        ];

        let plan = build_result_plan(&prepared, &sections);
        let article_leg = plan
            .legs
            .iter()
            .find(|l| l.leg == "article")
            .expect("article leg");
        assert!(
            !article_leg
                .routing
                .contains(&"fallback=disease_leg_ungrounded_keyword_survived".to_string()),
            "transport errors must not masquerade as ungrounded disease fallback: {:?}",
            article_leg.routing
        );
    }

    fn gwas_row(
        rsid: &str,
        trait_name: Option<&str>,
        p_value: Option<f64>,
    ) -> VariantGwasAssociation {
        VariantGwasAssociation {
            rsid: rsid.to_string(),
            trait_name: trait_name.map(str::to_string),
            p_value,
            effect_size: None,
            effect_type: None,
            confidence_interval: None,
            risk_allele_frequency: None,
            risk_allele: None,
            mapped_genes: Vec::new(),
            study_accession: None,
            pmid: None,
            author: None,
            sample_description: None,
        }
    }

    fn drug_row(name: &str) -> DrugSearchResult {
        DrugSearchResult {
            name: name.to_string(),
            drugbank_id: None,
            drug_type: None,
            mechanism: None,
            target: None,
        }
    }

    fn trial_row(nct_id: &str, status: &str) -> TrialSearchResult {
        TrialSearchResult {
            nct_id: nct_id.to_string(),
            title: format!("Trial {nct_id}"),
            status: status.to_string(),
            phase: None,
            conditions: Vec::new(),
            sponsor: None,
            matched_intervention_label: None,
        }
    }

    #[test]
    fn top_get_command_skips_civic_variant_ids() {
        let input = PreparedInput::new(&SearchAllInput {
            gene: None,
            variant: None,
            disease: None,
            drug: Some("dabrafenib".to_string()),
            keyword: None,
            since: None,
            limit: 3,
            counts_only: false,
            debug_plan: false,
        })
        .expect("valid input");
        let results = vec![
            json!({"id":"CIVIC_VARIANT:147"}),
            json!({"id":"rs113488022"}),
        ];
        let cmd = top_get_command(SectionKind::Variant, &input, &results).expect("command");
        assert_eq!(cmd, "biomcp get variant rs113488022");
    }

    #[test]
    fn top_get_command_prefers_parent_drug_name() {
        let input = PreparedInput::new(&SearchAllInput {
            gene: None,
            variant: None,
            disease: None,
            drug: Some("dabrafenib".to_string()),
            keyword: None,
            since: None,
            limit: 3,
            counts_only: false,
            debug_plan: false,
        })
        .expect("valid input");
        let results = vec![
            json!({"name":"desmethyl dabrafenib"}),
            json!({"name":"dabrafenib"}),
        ];
        let cmd = top_get_command(SectionKind::Drug, &input, &results).expect("command");
        assert_eq!(cmd, "biomcp get drug dabrafenib");
    }

    #[test]
    fn top_get_command_prefers_parent_like_salt_name_over_metabolites() {
        let input = PreparedInput::new(&SearchAllInput {
            gene: None,
            variant: None,
            disease: None,
            drug: Some("dabrafenib".to_string()),
            keyword: None,
            since: None,
            limit: 3,
            counts_only: false,
            debug_plan: false,
        })
        .expect("valid input");
        let results = vec![
            json!({"name":"desmethyl dabrafenib"}),
            json!({"name":"dabrafenib mesylate"}),
            json!({"name":"hydroxy dabrafenib"}),
        ];
        let cmd = top_get_command(SectionKind::Drug, &input, &results).expect("command");
        assert_eq!(cmd, "biomcp get drug dabrafenib");
    }

    #[test]
    fn refine_drug_results_filters_metabolites_when_parent_like_match_exists() {
        let rows = vec![
            drug_row("desmethyl dabrafenib"),
            drug_row("dabrafenib mesylate"),
            drug_row("hydroxy dabrafenib"),
        ];
        let refined = refine_drug_results(rows, Some("dabrafenib"), 3);
        let names = refined.into_iter().map(|row| row.name).collect::<Vec<_>>();
        assert_eq!(names, vec!["dabrafenib mesylate"]);
    }

    #[test]
    fn refine_drug_results_keeps_metabolites_when_no_parent_like_match() {
        let rows = vec![
            drug_row("desmethyl dabrafenib"),
            drug_row("hydroxy dabrafenib"),
        ];
        let refined = refine_drug_results(rows, Some("dabrafenib"), 3);
        assert_eq!(refined.len(), 2);
    }

    #[test]
    fn variant_significance_rank_matches_clinical_priority() {
        assert!(
            variant_significance_rank(Some("Pathogenic"))
                < variant_significance_rank(Some("Likely pathogenic"))
        );
        assert!(
            variant_significance_rank(Some("Likely pathogenic"))
                < variant_significance_rank(Some("VUS"))
        );
        assert!(
            variant_significance_rank(Some("VUS"))
                < variant_significance_rank(Some("Likely benign"))
        );
        assert!(
            variant_significance_rank(Some("Likely benign"))
                < variant_significance_rank(Some("Benign"))
        );
    }

    #[test]
    fn dedupe_gwas_rows_keeps_lowest_p_value() {
        let rows = vec![
            gwas_row("rs1", Some("melanoma"), Some(1e-5)),
            gwas_row("rs1", Some("melanoma"), Some(1e-7)),
            gwas_row("rs2", Some("melanoma"), Some(2e-6)),
        ];
        let deduped = dedupe_gwas_rows(rows);
        assert_eq!(deduped.len(), 2);
        let rs1 = deduped
            .iter()
            .find(|row| row.rsid == "rs1")
            .expect("rs1 row should remain");
        assert_eq!(rs1.p_value, Some(1e-7));
    }

    #[test]
    fn format_search_all_p_value_removes_float_artifacts() {
        assert_eq!(
            format_search_all_p_value(&json!(6.000000000000001e-22)),
            "6e-22"
        );
        assert_eq!(format_search_all_p_value(&json!(0.005)), "0.005");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn dispatch_section_pathway_surfaces_sanitized_wikipathways_404_without_timeout() {
        // Serialize the shared BIOMCP_* mock-base env mutation this warning-path test relies on.
        let _guard = env_lock_async().await;
        let reactome = MockServer::start().await;
        let kegg = MockServer::start().await;
        let wikipathways = MockServer::start().await;
        let _reactome_base = set_env_var("BIOMCP_REACTOME_BASE", Some(&reactome.uri()));
        let _kegg_base = set_env_var("BIOMCP_KEGG_BASE", Some(&kegg.uri()));
        let _wikipathways_base = set_env_var("BIOMCP_WIKIPATHWAYS_BASE", Some(&wikipathways.uri()));
        let _disable_kegg = set_env_var("BIOMCP_DISABLE_KEGG", None);

        Mock::given(method("GET"))
            .and(path("/search/query"))
            .and(query_param("query", "BRAF"))
            .and(query_param("species", "Homo sapiens"))
            .and(query_param("pageSize", "3"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw(r#"{"results":[],"totalResults":0}"#, "application/json"),
            )
            .expect(1)
            .mount(&reactome)
            .await;

        Mock::given(method("GET"))
            .and(path("/find/pathway/BRAF"))
            .respond_with(ResponseTemplate::new(200).set_body_string(""))
            .expect(1)
            .mount(&kegg)
            .await;

        Mock::given(method("GET"))
            .and(path("/findPathwaysByText.json"))
            .respond_with(ResponseTemplate::new(404).set_body_raw(
                "<!DOCTYPE html><html><head><title>404</title></head><body>File not found</body></html>",
                "text/html; charset=utf-8",
            ))
            .expect(1)
            .mount(&wikipathways)
            .await;

        let prepared = PreparedInput::new(&SearchAllInput {
            gene: Some("BRAF".to_string()),
            variant: None,
            disease: None,
            drug: None,
            keyword: None,
            since: None,
            limit: 3,
            counts_only: true,
            debug_plan: false,
        })
        .expect("valid prepared input");

        let section = dispatch_section(SectionKind::Pathway, &prepared).await;
        let error = section.error.clone().expect("pathway section should fail");
        assert_eq!(section.entity, "pathway");
        assert_eq!(section.count, 0);
        assert!(error.contains("wikipathways"));
        assert!(error.contains("HTTP 404"));
        assert!(error.contains("HTML error page"));
        assert!(!error.contains("timed out"));
        assert!(!error.contains("<!DOCTYPE"));
        assert!(!error.contains("<html"));
        assert!(!error.contains("<head"));

        let markdown = crate::render::markdown::search_all_markdown(
            &SearchAllResults {
                query: prepared.query_summary(),
                sections: vec![section],
                searches_dispatched: 1,
                searches_with_results: 0,
                wall_time_ms: 0,
                debug_plan: None,
            },
            true,
        )
        .expect("counts-only markdown should render");
        assert!(markdown.contains("## Pathways (0)"));
        assert!(markdown.contains("Error: API error from wikipathways: HTTP 404"));
        assert!(markdown.contains("HTML error page"));
        assert!(!markdown.contains("<!DOCTYPE"));
        assert!(!markdown.contains("<html"));
        assert!(!markdown.contains("<head"));
    }

    #[test]
    fn section_timeout_uses_article_specific_budget() {
        assert_eq!(section_timeout(SectionKind::Article).as_secs(), 60);
        assert_eq!(section_timeout(SectionKind::Trial).as_secs(), 12);
    }

    #[test]
    fn merge_trial_backfill_rows_preserves_preferred_order_and_dedupes() {
        let preferred = vec![
            trial_row("NCT00000001", "RECRUITING"),
            trial_row("NCT00000002", "ACTIVE_NOT_RECRUITING"),
        ];
        let backfill = vec![
            trial_row("NCT00000002", "UNKNOWN"),
            trial_row("NCT00000003", "UNKNOWN"),
            trial_row("NCT00000004", "COMPLETED"),
        ];

        let merged = merge_trial_backfill_rows(preferred, backfill, 3);
        let ids = merged
            .iter()
            .map(|row| row.nct_id.clone())
            .collect::<Vec<_>>();
        assert_eq!(ids, vec!["NCT00000001", "NCT00000002", "NCT00000003"]);
    }

    #[test]
    fn merge_trial_backfill_rows_respects_limit_with_preferred_only() {
        let preferred = vec![
            trial_row("NCT00000001", "RECRUITING"),
            trial_row("NCT00000002", "ACTIVE_NOT_RECRUITING"),
            trial_row("NCT00000003", "NOT_YET_RECRUITING"),
        ];

        let merged = merge_trial_backfill_rows(preferred, vec![], 2);
        let ids = merged
            .iter()
            .map(|row| row.nct_id.clone())
            .collect::<Vec<_>>();
        assert_eq!(ids, vec!["NCT00000001", "NCT00000002"]);
    }
}
