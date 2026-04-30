//! Search-all input preparation, dispatch planning, and debug-plan metadata.

use std::collections::HashSet;

use serde_json::Value;

use crate::cli::debug_plan::{DebugPlan, DebugPlanLeg};
use crate::error::BioMcpError;
use crate::utils::date::validate_since;

use super::{DispatchSpec, MAX_SEARCH_ALL_LIMIT, SearchAllInput, SearchAllSection, SectionKind};

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
pub(super) enum Anchor {
    Gene,
    Disease,
    Drug,
    Variant,
    Keyword,
}

impl Anchor {
    pub(super) fn as_str(self) -> &'static str {
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
pub(super) struct VariantContext {
    pub(super) raw: String,
    pub(super) parsed_gene: Option<String>,
    pub(super) parsed_change: Option<String>,
}

#[derive(Debug, Clone)]
pub(super) struct PreparedInput {
    pub(super) gene: Option<String>,
    pub(super) variant: Option<String>,
    pub(super) disease: Option<String>,
    pub(super) drug: Option<String>,
    pub(super) keyword: Option<String>,
    pub(super) since: Option<String>,
    pub(super) limit: usize,
    pub(super) counts_only: bool,
    pub(super) debug_plan: bool,
    pub(super) anchor: Anchor,
    pub(super) variant_context: Option<VariantContext>,
}

pub(super) fn build_result_plan(input: &PreparedInput, sections: &[SearchAllSection]) -> DebugPlan {
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
                source_status: Vec::new(),
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

pub(super) fn article_filters(
    input: &PreparedInput,
) -> crate::entities::article::ArticleSearchFilters {
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
    pub(super) fn new(input: &SearchAllInput) -> Result<Self, BioMcpError> {
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

    pub(super) fn query_summary(&self) -> String {
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

    pub(super) fn gene_anchor(&self) -> Option<&str> {
        self.gene.as_deref().or_else(|| {
            self.variant_context
                .as_ref()
                .and_then(|ctx| ctx.parsed_gene.as_deref())
        })
    }

    pub(super) fn has_shared_disease_keyword(&self) -> bool {
        matches!(
            (self.disease.as_deref(), self.keyword.as_deref()),
            (Some(disease), Some(keyword)) if tokens_equal_normalized(disease, keyword)
        )
    }

    pub(super) fn article_disease_filter(&self) -> Option<&str> {
        if self.has_shared_disease_keyword() {
            None
        } else {
            self.disease.as_deref()
        }
    }

    pub(super) fn article_keyword_filter(&self) -> Option<&str> {
        self.keyword.as_deref()
    }

    pub(super) fn drug_query(&self) -> Option<&str> {
        self.drug.as_deref()
    }

    pub(super) fn variant_trial_query(&self) -> Option<String> {
        let context = self.variant_context.as_ref()?;
        if let (Some(gene), Some(change)) = (
            context.parsed_gene.as_deref(),
            context.parsed_change.as_deref(),
        ) {
            return Some(format!("{gene} {change}"));
        }
        Some(context.raw.clone())
    }

    pub(super) fn trial_condition_query(&self) -> Option<&str> {
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

pub(super) fn build_dispatch_plan_prepared(input: &PreparedInput) -> Vec<DispatchSpec> {
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
