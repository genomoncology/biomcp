use std::collections::HashSet;
use std::time::{Duration, Instant};

use futures::future::join_all;
use serde::Serialize;
use serde_json::Value;

use crate::error::BioMcpError;
use crate::utils::date::validate_since;

const MAX_SEARCH_ALL_LIMIT: usize = 50;
const EXPAND_LIMIT: usize = 20;
const SECTION_TIMEOUT: Duration = Duration::from_secs(12);

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
            "adverse-event" => &["Report", "Drug", "Serious"],
            _ => &[],
        }
    }

    pub fn markdown_rows(&self) -> Vec<Vec<String>> {
        self.results
            .iter()
            .map(|row| match self.entity.as_str() {
                "gene" => vec![
                    value_str(row, "symbol"),
                    value_str(row, "name"),
                    value_str(row, "entrez_id"),
                ],
                "variant" => vec![
                    value_str(row, "id"),
                    value_str(row, "gene"),
                    value_str(row, "hgvs_p"),
                    value_str(row, "significance"),
                ],
                "disease" => vec![
                    value_str(row, "id"),
                    value_str(row, "name"),
                    value_str(row, "synonyms_preview"),
                ],
                "drug" => vec![
                    value_str(row, "name"),
                    value_str(row, "target"),
                    value_str_or(row, "mechanism", value_str(row, "drug_type")),
                ],
                "trial" => vec![
                    value_str(row, "nct_id"),
                    value_str(row, "title"),
                    value_str(row, "status"),
                ],
                "article" => vec![
                    value_str(row, "pmid"),
                    value_str(row, "title"),
                    value_str(row, "date"),
                ],
                "pathway" => vec![value_str(row, "id"), value_str(row, "name")],
                "pgx" => vec![
                    value_str(row, "genesymbol"),
                    value_str(row, "drugname"),
                    value_str(row, "cpiclevel"),
                ],
                "gwas" => vec![
                    value_str(row, "rsid"),
                    value_str(row, "trait_name"),
                    value_str(row, "p_value"),
                ],
                "adverse-event" => vec![
                    value_str(row, "report_id"),
                    value_str(row, "drug"),
                    value_str(row, "serious"),
                ],
                _ => vec![format_value(row)],
            })
            .collect()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchAllResults {
    pub query: String,
    pub sections: Vec<SearchAllSection>,
    pub searches_dispatched: usize,
    pub searches_with_results: usize,
    pub wall_time_ms: u64,
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
        sections,
        searches_dispatched,
        searches_with_results,
        wall_time_ms: u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
    })
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

    fn keyword_pushdown(&self) -> Option<&str> {
        if self.keyword.is_some() && (self.gene.is_some() || self.disease.is_some()) {
            self.keyword.as_deref()
        } else {
            None
        }
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
    let section_result = tokio::time::timeout(SECTION_TIMEOUT, run_section(kind, input)).await;

    match section_result {
        Ok(Ok((results, total))) => {
            let links = build_links(kind, input, &results, &search_self);
            SearchAllSection {
                entity: kind.entity().to_string(),
                label: kind.label().to_string(),
                count: results.len(),
                total,
                error: None,
                results,
                links,
            }
        }
        Ok(Err(err)) => SearchAllSection {
            entity: kind.entity().to_string(),
            label: kind.label().to_string(),
            count: 0,
            total: None,
            error: Some(err.to_string()),
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
                SECTION_TIMEOUT.as_secs()
            )),
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
) -> Result<(Vec<Value>, Option<usize>), BioMcpError> {
    match kind {
        SectionKind::Gene => {
            let query = input.gene_anchor().ok_or_else(|| {
                BioMcpError::InvalidArgument("No gene anchor available for gene search.".into())
            })?;
            let filters = crate::entities::gene::GeneSearchFilters {
                query: Some(query.to_string()),
                ..Default::default()
            };
            let page = crate::entities::gene::search_page(&filters, input.limit, 0).await?;
            Ok((to_json_array(page.results)?, page.total))
        }
        SectionKind::Variant => {
            if let Some(variant_id) = input
                .variant_context
                .as_ref()
                .and_then(|ctx| (ctx.parsed_gene.is_none()).then_some(ctx.raw.as_str()))
            {
                let row = crate::entities::variant::get(variant_id, &[]).await?;
                return Ok((to_json_array(vec![row])?, Some(1)));
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
            let page = crate::entities::variant::search_page(&filters, input.limit, 0).await?;
            Ok((to_json_array(page.results)?, page.total))
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
            let page = crate::entities::disease::search_page(&filters, input.limit, 0).await?;
            Ok((to_json_array(page.results)?, page.total))
        }
        SectionKind::Drug => {
            // When --drug is explicit, use it as-is. Keyword pushdown only
            // applies for drug *discovery* (no --drug) with gene/disease anchors.
            let query = match (input.drug.as_deref(), input.keyword_pushdown()) {
                (Some(drug), _) => Some(drug.to_string()),
                (None, Some(keyword)) => Some(keyword.to_string()),
                (None, None) => None,
            };

            let filters = crate::entities::drug::DrugSearchFilters {
                query,
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
            let page = crate::entities::drug::search_page(&filters, input.limit, 0).await?;
            Ok((to_json_array(page.results)?, page.total))
        }
        SectionKind::Trial => {
            let filters = crate::entities::trial::TrialSearchFilters {
                condition: input.disease.clone(),
                intervention: input.drug.clone(),
                biomarker: input.gene_anchor().map(str::to_string),
                mutation: input.variant_trial_query(),
                date_from: input.since.clone(),
                source: crate::entities::trial::TrialSource::ClinicalTrialsGov,
                ..Default::default()
            };
            let page = crate::entities::trial::search_page(&filters, input.limit, 0, None).await?;
            Ok((to_json_array(page.results)?, page.total))
        }
        SectionKind::Article => {
            let filters = crate::entities::article::ArticleSearchFilters {
                gene: input.gene_anchor().map(str::to_string),
                disease: input.disease.clone(),
                drug: input.drug.clone(),
                author: None,
                keyword: input.keyword.clone(),
                date_from: input.since.clone(),
                date_to: None,
                article_type: None,
                journal: None,
                open_access: false,
                no_preprints: false,
                exclude_retracted: true,
                sort: crate::entities::article::ArticleSort::Date,
            };
            let page = crate::entities::article::search_page(&filters, input.limit, 0).await?;
            Ok((to_json_array(page.results)?, page.total))
        }
        SectionKind::Pathway => {
            let query = input.gene_anchor().ok_or_else(|| {
                BioMcpError::InvalidArgument("No gene anchor available for pathway search.".into())
            })?;
            let filters = crate::entities::pathway::PathwaySearchFilters {
                query: Some(query.to_string()),
                ..Default::default()
            };
            let pathway_limit = input.limit.min(25);
            let (results, total) =
                crate::entities::pathway::search_with_filters(&filters, pathway_limit).await?;
            Ok((to_json_array(results)?, total))
        }
        SectionKind::Pgx => {
            let filters = crate::entities::pgx::PgxSearchFilters {
                gene: input.gene_anchor().map(str::to_string),
                drug: input.drug.clone(),
                ..Default::default()
            };
            let page = crate::entities::pgx::search_page(&filters, input.limit, 0).await?;
            Ok((to_json_array(page.results)?, page.total))
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
            let page = crate::entities::variant::search_gwas_page(&filters, input.limit, 0).await?;
            Ok((to_json_array(page.results)?, page.total))
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
            let page =
                crate::entities::adverse_event::search_page(&filters, input.limit, 0).await?;
            Ok((to_json_array(page.results)?, page.total))
        }
    }
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
            if let Some(drug) = first_string(results, &["name"]).or(input.drug.as_deref()) {
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
            if input.disease.is_none() {
                hints.push(SearchAllLink {
                    rel: "filter.hint".to_string(),
                    title: "Filter by significance".to_string(),
                    command: format!(
                        "biomcp search variant {} --significance pathogenic",
                        variant_base_args(input)
                    ),
                });
            }
            // Population frequency is always useful context
            hints.push(SearchAllLink {
                rel: "filter.hint".to_string(),
                title: "Rare variants only".to_string(),
                command: format!(
                    "biomcp search variant {} --max-frequency 0.01",
                    variant_base_args(input)
                ),
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
        SectionKind::Drug => {
            // No placeholder hints — every link must be directly runnable.
            // Drug filter hints only appear when we have concrete values.
        }
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
        SectionKind::Variant => first_string(results, &["id"])
            .or(input.variant.as_deref())
            .map(|id| format!("biomcp get variant {}", quote_arg(id))),
        SectionKind::Disease => first_string(results, &["id", "name"])
            .or(input.disease.as_deref())
            .map(|id| format!("biomcp get disease {}", quote_arg(id))),
        SectionKind::Drug => first_string(results, &["name"])
            .or(input.drug.as_deref())
            .map(|id| format!("biomcp get drug {}", quote_arg(id))),
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
            // Mirror run_section: explicit --drug wins; keyword only for discovery.
            let query = match (input.drug.as_deref(), input.keyword_pushdown()) {
                (Some(drug), _) => Some(drug.to_string()),
                (None, Some(keyword)) => Some(keyword.to_string()),
                (None, None) => None,
            };
            push_opt_owned(&mut args, "--query", query);
            push_opt(&mut args, "--target", input.gene_anchor());
            push_opt(&mut args, "--indication", input.disease.as_deref());
        }
        SectionKind::Trial => {
            push_opt(&mut args, "--condition", input.disease.as_deref());
            push_opt(&mut args, "--intervention", input.drug.as_deref());
            push_opt(&mut args, "--biomarker", input.gene_anchor());
            push_opt_owned(&mut args, "--mutation", input.variant_trial_query());
            push_opt(&mut args, "--since", input.since.as_deref());
        }
        SectionKind::Article => {
            push_opt(&mut args, "--gene", input.gene_anchor());
            push_opt(&mut args, "--disease", input.disease.as_deref());
            push_opt(&mut args, "--drug", input.drug.as_deref());
            push_opt(&mut args, "--keyword", input.keyword.as_deref());
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

    args.push("--limit".into());
    args.push(limit.to_string());
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

fn quote_arg(value: &str) -> String {
    // Quote when the value contains whitespace, shell metacharacters,
    // or characters that need escaping. Variant IDs like chr7:g.55223568G>C
    // contain `>` which shells interpret as redirect.
    if value
        .chars()
        .any(|ch| ch.is_whitespace() || ">|&;()$`!#\"\\".contains(ch))
    {
        format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        value.to_string()
    }
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

#[cfg(test)]
mod tests {
    use super::*;

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
        });
        let entities = plan.iter().map(|spec| spec.entity).collect::<Vec<_>>();
        assert_eq!(entities, vec!["article"]);
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
        })
        .expect_err("expected validation error");
        assert!(err.to_string().contains("at least one typed slot"));
    }

    #[test]
    fn canonical_drug_command_pushes_keyword_with_gene_anchor() {
        let prepared = PreparedInput::new(&SearchAllInput {
            gene: Some("BRAF".to_string()),
            variant: None,
            disease: None,
            drug: None,
            keyword: Some("resistance".to_string()),
            since: None,
            limit: 3,
            counts_only: false,
        })
        .expect("valid prepared input");

        let command = canonical_search_command(SectionKind::Drug, &prepared, 3);
        assert!(command.contains("search drug"));
        assert!(command.contains("--target BRAF"));
        assert!(command.contains("--query resistance"));
    }

    #[test]
    fn canonical_drug_command_does_not_push_unanchored_keyword() {
        let prepared = PreparedInput::new(&SearchAllInput {
            gene: None,
            variant: None,
            disease: None,
            drug: None,
            keyword: Some("resistance".to_string()),
            since: None,
            limit: 3,
            counts_only: false,
        })
        .expect("valid prepared input");

        let command = canonical_search_command(SectionKind::Article, &prepared, 3);
        assert!(command.contains("search article"));
        assert!(command.contains("--keyword resistance"));
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
        })
        .expect("valid prepared input");

        let command = canonical_search_command(SectionKind::Variant, &prepared, 3);
        assert_eq!(command, "biomcp search variant rs121913529 --limit 3");
    }

    #[test]
    fn quote_arg_wraps_spaces_and_quotes() {
        assert_eq!(quote_arg("BRAF"), "BRAF");
        assert_eq!(quote_arg("BRAF V600E"), "\"BRAF V600E\"");
        assert_eq!(quote_arg("BRAF \"V600E\""), "\"BRAF \\\"V600E\\\"\"");
    }
}
