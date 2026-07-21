//! Variant-specific article route union, provenance, ranking, and pagination.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering as AtomicOrdering},
};
use std::time::Instant;

use clap::ValueEnum;
use futures::{StreamExt, stream};
use serde::Serialize;

use crate::entities::variant::{
    NormalizedVariantAliases, RequestedVariantIdentity, VariantArticleResolution,
    VariantArticleResolutionContext, VariantProviderValidationStatus, VariantResolutionStatus,
};
use crate::error::BioMcpError;

use super::backends::search_pubtator_page_with_context;
use super::candidates::{
    ArticleCandidate, article_candidate_from_row, merge_article_candidate_pool,
    stable_article_identifier,
};
use super::enrichment::{
    enrich_article_search_rows_with_semantic_scholar_context,
    enrich_visible_article_search_rows_with_article_base_context,
};
use super::query::resolve_variant_entity_tokens;
use super::search::{
    VARIANT_ENTITY_RETRIEVAL_PATH, VARIANT_FALLBACK_RETRIEVAL_PATH,
    acquire_federated_article_rows_with_context,
};
use super::{
    ArticleRankingOptions, ArticleSearchFilters, ArticleSearchResult, ArticleSort, ArticleSource,
    ArticleSourceAvailability, ArticleSourceStatus, ArticleVariantIntent,
    MAX_FEDERATED_FETCH_RESULTS,
};

const LEXICAL_ALIAS_FETCH_LIMIT: usize = 25;
const MAX_EXACT_ALIASES: usize = 10;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum VariantArticleStrategy {
    #[default]
    Union,
    Annotation,
    Lexical,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct VariantArticleProvenance {
    pub route: String,
    pub source: String,
    pub matched_alias: Option<String>,
    pub native_position: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariantArticleSourceStatus {
    pub route: String,
    pub source: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariantArticlePagination {
    pub offset: usize,
    pub limit: usize,
    pub returned: usize,
    pub total: Option<usize>,
    pub has_more: bool,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariantArticleRow {
    #[serde(flatten)]
    pub article: ArticleSearchResult,
    pub requested_variant: RequestedVariantIdentity,
    pub matched_aliases: Vec<String>,
    pub retrieval_routes: Vec<String>,
    pub sources: Vec<String>,
    pub rank: usize,
    pub provenance: Vec<VariantArticleProvenance>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariantArticleResponse {
    pub requested_variant: RequestedVariantIdentity,
    pub resolution: VariantArticleResolution,
    pub strategy: VariantArticleStrategy,
    pub complete: bool,
    pub truncated: bool,
    pub pagination: VariantArticlePagination,
    pub source_status: Vec<VariantArticleSourceStatus>,
    pub retrieval_path: &'static str,
    pub results: Vec<VariantArticleRow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_plan: Option<VariantArticleDebugPlan>,
}

#[derive(Debug)]
pub struct VariantArticleOutcome {
    pub response: VariantArticleResponse,
    pub hard_error: bool,
}

const ITEM_WORK_LIMIT: usize = 50;
const ITEM_CONCURRENCY_LIMIT: usize = 2;

#[derive(Debug)]
struct SharedWorkBudget {
    limit: usize,
    consumed: AtomicUsize,
}

#[derive(Debug, Clone)]
struct VariantArticleCallEvent {
    route: String,
    source: String,
    status: String,
    latency_ms: u64,
    pages: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct VariantArticleExecutionContext {
    item: Arc<SharedWorkBudget>,
    request: Arc<SharedWorkBudget>,
    events: Arc<Mutex<Vec<VariantArticleCallEvent>>>,
    stopped_routes: Arc<Mutex<BTreeSet<String>>>,
}

impl VariantArticleExecutionContext {
    fn with_request(request: Arc<SharedWorkBudget>) -> Self {
        Self {
            item: Arc::new(SharedWorkBudget {
                limit: ITEM_WORK_LIMIT,
                consumed: AtomicUsize::new(0),
            }),
            request,
            events: Arc::new(Mutex::new(Vec::new())),
            stopped_routes: Arc::new(Mutex::new(BTreeSet::new())),
        }
    }

    pub(crate) fn single() -> Self {
        Self::with_request(Arc::new(SharedWorkBudget {
            limit: ITEM_WORK_LIMIT,
            consumed: AtomicUsize::new(0),
        }))
    }

    fn batch(item_count: usize) -> Vec<Self> {
        let request = Arc::new(SharedWorkBudget {
            limit: ITEM_WORK_LIMIT.saturating_mul(item_count),
            consumed: AtomicUsize::new(0),
        });
        (0..item_count)
            .map(|_| Self::with_request(request.clone()))
            .collect()
    }

    pub(crate) fn reserve(&self, route: &str) -> Option<Instant> {
        let reserve = |budget: &SharedWorkBudget| {
            budget
                .consumed
                .fetch_update(AtomicOrdering::SeqCst, AtomicOrdering::SeqCst, |current| {
                    (current < budget.limit).then(|| current + 1)
                })
                .is_ok()
        };
        if !reserve(&self.item) {
            self.stop(route);
            return None;
        }
        if !reserve(&self.request) {
            self.item.consumed.fetch_sub(1, AtomicOrdering::SeqCst);
            self.stop(route);
            return None;
        }
        Some(Instant::now())
    }

    fn stop(&self, route: &str) {
        self.stopped_routes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(route.to_string());
    }

    pub(crate) fn record(
        &self,
        route: &str,
        source: &str,
        started: Instant,
        status: &str,
        pages: usize,
    ) {
        self.events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(VariantArticleCallEvent {
                route: route.into(),
                source: source.into(),
                status: status.into(),
                latency_ms: started.elapsed().as_millis().try_into().unwrap_or(u64::MAX),
                pages,
            });
    }

    fn item_work(&self) -> VariantArticleWork {
        VariantArticleWork::new(
            self.item.limit,
            self.item.consumed.load(AtomicOrdering::SeqCst),
        )
    }

    fn request_work(&self) -> VariantArticleWork {
        VariantArticleWork::new(
            self.request.limit,
            self.request.consumed.load(AtomicOrdering::SeqCst),
        )
    }

    fn events(&self) -> Vec<VariantArticleCallEvent> {
        self.events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    fn stopped_routes(&self) -> Vec<String> {
        self.stopped_routes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .cloned()
            .collect()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct VariantArticleWork {
    pub limit: usize,
    pub consumed: usize,
    pub remaining: usize,
    pub exhausted: bool,
}

impl VariantArticleWork {
    fn new(limit: usize, consumed: usize) -> Self {
        let consumed = consumed.min(limit);
        Self {
            limit,
            consumed,
            remaining: limit.saturating_sub(consumed),
            exhausted: consumed >= limit,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct VariantArticleProviderPlan {
    pub source: String,
    pub status: String,
    pub latency_ms: u64,
    pub calls: usize,
    pub pages: usize,
    pub cache: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariantArticleRoutePlan {
    pub route: String,
    pub queries: Vec<String>,
    pub providers: Vec<VariantArticleProviderPlan>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariantArticleCountsPlan {
    pub pre_dedup: usize,
    pub post_dedup: usize,
    pub returned: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariantArticleRankingPlan {
    pub method: &'static str,
    pub inputs: [&'static str; 3],
}

#[derive(Debug, Clone, Serialize)]
pub struct VariantArticleBudgetsPlan {
    pub item: VariantArticleWork,
    pub request: VariantArticleWork,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariantArticleNextPlan {
    pub offset: usize,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariantArticleDebugPlan {
    pub normalized_aliases: NormalizedVariantAliases,
    pub routes: Vec<VariantArticleRoutePlan>,
    pub counts: VariantArticleCountsPlan,
    pub ranking: VariantArticleRankingPlan,
    pub budgets: VariantArticleBudgetsPlan,
    pub truncated: bool,
    pub stopped_routes: Vec<String>,
    pub next: VariantArticleNextPlan,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariantArticleBatchDebugPlan {
    pub item_concurrency_limit: usize,
    pub work: VariantArticleWork,
    pub items_planned: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompactVariantArticleRow {
    pub pmid: Option<String>,
    pub pmcid: Option<String>,
    pub doi: Option<String>,
    pub arxiv_id: Option<String>,
    pub semantic_scholar_id: Option<String>,
    pub title: String,
    pub date: Option<String>,
    pub match_reason: &'static str,
    pub matched_aliases: Vec<String>,
    pub routes: Vec<String>,
    pub sources: Vec<String>,
    pub rank: usize,
    pub is_retracted: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariantArticleItemError {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariantArticleBatchItem {
    pub request_id: String,
    pub requested_variant: RequestedVariantIdentity,
    pub resolution: Option<VariantArticleResolution>,
    pub complete: bool,
    pub truncated: bool,
    pub pagination: VariantArticlePagination,
    pub source_status: Vec<VariantArticleSourceStatus>,
    pub error: Option<VariantArticleItemError>,
    pub results: Vec<CompactVariantArticleRow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_plan: Option<VariantArticleDebugPlan>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariantArticleBatchMeta {
    pub next_commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariantArticleBatchResponse {
    pub items: Vec<VariantArticleBatchItem>,
    pub complete: bool,
    pub truncated: bool,
    pub _meta: VariantArticleBatchMeta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_plan: Option<VariantArticleBatchDebugPlan>,
}

#[derive(Debug)]
pub struct VariantArticleBatchOutcome {
    pub response: VariantArticleBatchResponse,
    pub hard_error: bool,
}

fn article_filters() -> ArticleSearchFilters {
    ArticleSearchFilters {
        gene: None,
        gene_anchored: false,
        disease: None,
        drug: None,
        variant: None,
        author: None,
        keyword: None,
        date_from: None,
        date_to: None,
        article_type: None,
        journal: None,
        open_access: false,
        no_preprints: false,
        exclude_retracted: true,
        max_per_source: None,
        sort: ArticleSort::Relevance,
        ranking: ArticleRankingOptions::default(),
    }
}

fn source_name(source: ArticleSource) -> &'static str {
    match source {
        ArticleSource::PubTator => "pubtator",
        ArticleSource::EuropePmc => "europepmc",
        ArticleSource::SemanticScholar => "semanticscholar",
        ArticleSource::PubMed => "pubmed",
        ArticleSource::LitSense2 => "litsense2",
    }
}

fn status(route: &str, source: &str, state: &str) -> VariantArticleSourceStatus {
    status_with_detail(route, source, state, None)
}

fn status_with_detail(
    route: &str,
    source: &str,
    state: &str,
    detail: Option<&str>,
) -> VariantArticleSourceStatus {
    VariantArticleSourceStatus {
        route: route.to_string(),
        source: source.to_string(),
        status: state.to_string(),
        detail: detail.map(str::to_string),
    }
}

fn availability_severity(status: Option<ArticleSourceAvailability>) -> u8 {
    match status {
        Some(ArticleSourceAvailability::Unavailable) => 2,
        Some(ArticleSourceAvailability::Degraded) => 1,
        Some(ArticleSourceAvailability::Ok | ArticleSourceAvailability::Skipped) | None => 0,
    }
}

fn record_provider_status(
    statuses: &mut BTreeMap<String, u8>,
    source: ArticleSource,
    availability: Option<ArticleSourceAvailability>,
) {
    let severity = availability_severity(availability);
    statuses
        .entry(source_name(source).to_string())
        .and_modify(|current| *current = (*current).max(severity))
        .or_insert(severity);
}

fn record_federated_statuses(
    statuses: &mut BTreeMap<String, u8>,
    source_status: &[ArticleSourceStatus],
    semantic_scholar_status: &ArticleSourceStatus,
) {
    for source in [
        ArticleSource::PubTator,
        ArticleSource::EuropePmc,
        ArticleSource::PubMed,
    ] {
        let availability = source_status
            .iter()
            .find(|status| status.source == source)
            .and_then(|status| status.status)
            .or(Some(ArticleSourceAvailability::Ok));
        record_provider_status(statuses, source, availability);
    }
    record_provider_status(
        statuses,
        ArticleSource::SemanticScholar,
        semantic_scholar_status.status,
    );
}

fn candidate_with_provenance(
    row: ArticleSearchResult,
    route: &str,
    source: &str,
    matched_alias: Option<String>,
) -> ArticleCandidate {
    let native_position = row.source_local_position.saturating_add(1);
    let mut candidate = article_candidate_from_row(row);
    candidate.variant_provenance.push(VariantArticleProvenance {
        route: route.to_string(),
        source: source.to_string(),
        matched_alias,
        native_position,
    });
    candidate
}

fn exact_aliases(context: &VariantArticleResolutionContext) -> (Vec<String>, bool) {
    if context.requested.is_authoritative_refseq() {
        let mut aliases = BTreeSet::new();
        if let Some(coding) = context.requested.coding_change.as_deref() {
            if let Some(gene) = context.requested.gene.as_deref() {
                aliases.insert(format!("{gene} {coding}"));
            }
            if let Some(transcript) = context.requested.transcript.as_deref() {
                aliases.insert(format!("{transcript}:{coding}"));
            }
        }
        if let (Some(accession), Some(position), Some(reference), Some(alternate)) = (
            context.requested.genomic_accession.as_deref(),
            context.requested.position,
            context.requested.reference.as_deref(),
            context.requested.alternate.as_deref(),
        ) {
            aliases.insert(format!("{accession}:g.{position}{reference}>{alternate}"));
        }
        return (aliases.into_iter().take(MAX_EXACT_ALIASES).collect(), false);
    }
    let mut aliases = BTreeSet::new();
    let gene = context
        .source_identity
        .as_ref()
        .and_then(|source| source.genes.first())
        .or(context.requested.gene.as_ref());
    let mut insert_change = |change: &str| {
        let change = change.trim();
        if change.is_empty() {
            return;
        }
        if let Some(gene) = gene {
            aliases.insert(format!("{gene} {change}"));
        } else {
            aliases.insert(change.to_string());
        }
    };
    if let Some(change) = context.requested.protein_change.as_deref() {
        insert_change(change);
    }
    if let Some(change) = context.requested.coding_change.as_deref() {
        insert_change(change);
    }
    if context.requested.protein_change.is_none() {
        for change in &context.resolution.normalized_aliases.protein_changes {
            insert_change(change);
        }
    }
    if let Some(source) = context.source_identity.as_ref() {
        for change in source.protein_changes.iter().chain(&source.coding_changes) {
            insert_change(change);
        }
        if !source.genomic_id.trim().is_empty() {
            aliases.insert(source.genomic_id.trim().to_string());
        }
        aliases.extend(source.rsids.iter().map(|value| value.trim().to_string()));
    }
    aliases.retain(|value| !value.is_empty());
    let primary = primary_exact_alias(context);
    let mut ordered = Vec::with_capacity(aliases.len().min(MAX_EXACT_ALIASES));
    if let Some(primary) = primary
        && aliases.remove(&primary)
    {
        ordered.push(primary);
    }
    let truncated = ordered.len().saturating_add(aliases.len()) > MAX_EXACT_ALIASES;
    ordered.extend(
        aliases
            .into_iter()
            .take(MAX_EXACT_ALIASES.saturating_sub(ordered.len())),
    );
    (ordered, truncated)
}

fn combined_normalized_aliases(
    context: &VariantArticleResolutionContext,
) -> NormalizedVariantAliases {
    let mut aliases = context.requested.normalized_aliases();
    if context.requested.is_authoritative_refseq() {
        return aliases;
    }
    if let Some(source) = context.source_identity.as_ref() {
        aliases.protein_changes.extend(
            source
                .protein_changes
                .iter()
                .filter_map(|value| crate::entities::variant::normalize_protein_change(value)),
        );
        aliases.coding_changes.extend(source.coding_changes.clone());
        if !source.genomic_id.trim().is_empty() {
            aliases.genomic_ids.push(source.genomic_id.clone());
        }
        aliases.rsids.extend(source.rsids.clone());
    }
    aliases.protein_changes.sort();
    aliases.protein_changes.dedup();
    aliases.coding_changes.sort();
    aliases.coding_changes.dedup();
    aliases.genomic_ids.sort();
    aliases.genomic_ids.dedup();
    aliases.rsids.sort();
    aliases.rsids.dedup();
    aliases
}

fn primary_exact_alias(context: &VariantArticleResolutionContext) -> Option<String> {
    if context.requested.is_authoritative_refseq() {
        return exact_aliases(context).0.into_iter().next();
    }
    let gene = context.requested.gene.as_deref();
    if let Some(change) = context.requested.protein_change.as_deref() {
        return Some(
            gene.map(|gene| format!("{gene} {change}"))
                .unwrap_or_else(|| change.to_string()),
        );
    }
    if let Some(change) = context.requested.coding_change.as_deref() {
        return Some(
            gene.map(|gene| format!("{gene} {change}"))
                .unwrap_or_else(|| change.to_string()),
        );
    }
    context.requested.rsid.clone().or_else(|| {
        context
            .source_identity
            .as_ref()
            .map(|source| source.genomic_id.clone())
            .filter(|value| !value.is_empty())
    })
}

async fn annotation_candidates(
    input: &str,
    context: &VariantArticleResolutionContext,
    execution: &VariantArticleExecutionContext,
) -> Result<(Vec<ArticleCandidate>, bool, bool), BioMcpError> {
    let pubtator = crate::sources::pubtator::PubTatorClient::new()?;
    let Some(started) = execution.reserve("pubtator_variant") else {
        return Ok((Vec::new(), true, false));
    };
    let token_result = resolve_variant_entity_tokens(&pubtator, input, &context.requested).await;
    execution.record(
        "pubtator_variant",
        "pubtator",
        started,
        if token_result.is_ok() {
            "ok"
        } else {
            "unavailable"
        },
        usize::from(token_result.is_ok()),
    );
    let tokens = token_result?;
    let mut candidates = Vec::new();
    let mut incomplete = false;
    let mut succeeded = tokens.is_empty();
    for token in tokens {
        let mut filters = article_filters();
        filters.variant = Some(ArticleVariantIntent {
            original: input.to_string(),
            gene: context.requested.gene.clone(),
            change: context.requested.protein_change.clone(),
            entity_id: Some(token.entity_id),
        });
        let page_result = search_pubtator_page_with_context(
            &filters,
            MAX_FEDERATED_FETCH_RESULTS,
            0,
            Some(execution),
            "pubtator_variant",
        )
        .await;
        let page = match page_result {
            Ok(page) => page,
            Err(_) => {
                incomplete = true;
                continue;
            }
        };
        succeeded = true;
        incomplete |= page.total.is_some_and(|total| total > page.results.len());
        for row in page.results {
            if candidates.len() >= MAX_FEDERATED_FETCH_RESULTS {
                return Ok((candidates, true, succeeded));
            }
            candidates.push(candidate_with_provenance(
                row,
                "pubtator_variant",
                "pubtator",
                Some(if context.requested.is_authoritative_refseq() {
                    input.trim().to_string()
                } else {
                    token.matched_alias.clone()
                }),
            ));
        }
    }
    Ok((candidates, incomplete, succeeded))
}

async fn federated_alias_candidates(
    aliases: Vec<String>,
    alias_budget_stopped: bool,
    route: &str,
    expose_matched_alias: bool,
    execution: &VariantArticleExecutionContext,
) -> (
    Vec<ArticleCandidate>,
    bool,
    bool,
    Vec<VariantArticleSourceStatus>,
) {
    let searches = aliases.into_iter().map(|alias| async move {
        let mut filters = article_filters();
        filters.keyword = Some(alias.clone());
        acquire_federated_article_rows_with_context(
            &filters,
            LEXICAL_ALIAS_FETCH_LIMIT,
            Some(execution),
            route,
        )
        .await
        .map(|rows| (alias, rows))
    });
    let mut candidates = Vec::new();
    let mut incomplete = alias_budget_stopped;
    let mut succeeded = false;
    let mut alias_failed = false;
    let mut provider_statuses = BTreeMap::new();
    for result in futures::future::join_all(searches).await {
        let (alias, federated) = match result {
            Ok(result) => result,
            Err(_) => {
                incomplete = true;
                alias_failed = true;
                continue;
            }
        };
        let alias_succeeded = federated.primary_error.is_none()
            || matches!(
                federated.semantic_scholar_status.status,
                Some(ArticleSourceAvailability::Ok | ArticleSourceAvailability::Degraded)
            );
        succeeded |= alias_succeeded;
        alias_failed |= !alias_succeeded;
        incomplete |= !alias_succeeded;
        record_federated_statuses(
            &mut provider_statuses,
            &federated.source_status,
            &federated.semantic_scholar_status,
        );
        for source in federated.truncated_sources {
            record_provider_status(
                &mut provider_statuses,
                source,
                Some(ArticleSourceAvailability::Degraded),
            );
        }
        incomplete |= provider_statuses.values().any(|severity| *severity > 0);
        for row in federated.rows {
            if candidates.len() >= MAX_FEDERATED_FETCH_RESULTS {
                incomplete = true;
                break;
            }
            let sources = if row.matched_sources.is_empty() {
                vec![row.source]
            } else {
                row.matched_sources.clone()
            };
            let mut candidate = article_candidate_from_row(row);
            for source in sources {
                candidate.variant_provenance.push(VariantArticleProvenance {
                    route: route.to_string(),
                    source: source_name(source).to_string(),
                    matched_alias: expose_matched_alias.then(|| alias.clone()),
                    native_position: candidate.row.source_local_position.saturating_add(1),
                });
            }
            candidates.push(candidate);
        }
    }
    if !succeeded {
        incomplete = true;
    }
    if alias_failed || !succeeded {
        let route_severity = if succeeded { 1 } else { 2 };
        for source in ["pubtator", "europepmc", "pubmed", "semanticscholar"] {
            provider_statuses
                .entry(source.to_string())
                .and_modify(|current| *current = (*current).max(route_severity))
                .or_insert(route_severity);
        }
    }
    if alias_budget_stopped {
        for source in ["pubtator", "europepmc", "pubmed", "semanticscholar"] {
            provider_statuses
                .entry(source.to_string())
                .and_modify(|current| *current = (*current).max(1))
                .or_insert(1);
        }
    }
    let statuses = provider_statuses
        .into_iter()
        .map(|(source, severity)| {
            let state = match severity {
                0 => "ok",
                1 => "degraded",
                _ => "unavailable",
            };
            status_with_detail(
                route,
                &source,
                state,
                (severity > 0)
                    .then_some("one or more providers or aliases stopped before the route bound"),
            )
        })
        .collect();
    (candidates, incomplete, succeeded, statuses)
}

async fn lexical_candidates(
    context: &VariantArticleResolutionContext,
    execution: &VariantArticleExecutionContext,
) -> (
    Vec<ArticleCandidate>,
    bool,
    bool,
    Vec<VariantArticleSourceStatus>,
) {
    let (aliases, alias_budget_stopped) = exact_aliases(context);
    federated_alias_candidates(
        aliases,
        alias_budget_stopped,
        "exact_lexical",
        true,
        execution,
    )
    .await
}

fn pmid_seed(pmid: String) -> ArticleSearchResult {
    ArticleSearchResult {
        pmid,
        pmcid: None,
        doi: None,
        arxiv_id: None,
        semantic_scholar_id: None,
        title: String::new(),
        journal: None,
        date: None,
        first_index_date: None,
        citation_count: None,
        influential_citation_count: None,
        source: ArticleSource::PubMed,
        matched_sources: vec![ArticleSource::PubMed],
        score: None,
        is_retracted: None,
        abstract_snippet: None,
        ranking: None,
        normalized_title: String::new(),
        normalized_abstract: String::new(),
        publication_type: None,
        source_local_position: 0,
    }
}

async fn citation_candidates(
    context: &VariantArticleResolutionContext,
    execution: &VariantArticleExecutionContext,
) -> Result<Vec<ArticleCandidate>, BioMcpError> {
    let Some(retained_hit) = context.source_hit.as_ref() else {
        return Ok(Vec::new());
    };
    let hydrated_hit = if retained_hit.civic.is_none() {
        let Some(started) = execution.reserve("source_citation") else {
            return Ok(Vec::new());
        };
        let result = crate::sources::myvariant::MyVariantClient::new()?
            .get_all(&retained_hit.id)
            .await;
        execution.record(
            "source_citation",
            "myvariant",
            started,
            if result.is_ok() { "ok" } else { "unavailable" },
            usize::from(result.is_ok()),
        );
        let source_key = context
            .source_identity
            .as_ref()
            .map(crate::entities::variant::SourceVariantIdentity::normalized_key);
        result?
            .into_iter()
            .filter(|hit| {
                source_key.as_ref().is_none_or(|key| {
                    crate::entities::variant::SourceVariantIdentity::from_myvariant_hit(hit)
                        .normalized_key()
                        == *key
                })
            })
            .min_by_key(|hit| serde_json::to_string(hit).unwrap_or_default())
    } else {
        None
    };
    let hit = hydrated_hit.as_ref().unwrap_or(retained_hit);
    let matched_alias = primary_exact_alias(context);
    Ok(crate::sources::myvariant::civic_pubmed_ids(hit)
        .into_iter()
        .enumerate()
        .map(|(position, pmid)| {
            let mut row = pmid_seed(pmid);
            row.source_local_position = position;
            candidate_with_provenance(row, "source_citation", "civic", matched_alias.clone())
        })
        .collect())
}

fn fallback_aliases(input: &str, context: &VariantArticleResolutionContext) -> (Vec<String>, bool) {
    let mut aliases = BTreeSet::from([input.trim().to_string()]);
    let gene = context.requested.gene.as_deref();
    let mut insert_change = |change: &str| {
        let change = change.trim();
        if !change.is_empty() {
            aliases.insert(
                gene.map(|gene| format!("{gene} {change}"))
                    .unwrap_or_else(|| change.to_string()),
            );
        }
    };
    for change in &context.resolution.normalized_aliases.protein_changes {
        insert_change(change);
    }
    for change in &context.resolution.normalized_aliases.coding_changes {
        insert_change(change);
    }

    if let Some(first) = context.fallback_source_identities.first() {
        let requested_proteins = context
            .requested
            .normalized_aliases()
            .protein_changes
            .into_iter()
            .collect::<BTreeSet<_>>();
        for change in &first.protein_changes {
            let Some(normalized) = crate::entities::variant::normalize_protein_change(change)
            else {
                continue;
            };
            if requested_proteins.contains(&normalized)
                && context.fallback_source_identities.iter().all(|identity| {
                    identity.protein_changes.iter().any(|candidate| {
                        crate::entities::variant::normalize_protein_change(candidate).as_ref()
                            == Some(&normalized)
                    })
                })
            {
                insert_change(change);
            }
        }
        for change in &first.coding_changes {
            if context.fallback_source_identities.iter().all(|identity| {
                identity
                    .coding_changes
                    .iter()
                    .any(|candidate| candidate.eq_ignore_ascii_case(change))
            }) {
                insert_change(change);
            }
        }
    }

    aliases.retain(|value| !value.is_empty());
    let primary = input.trim().to_string();
    aliases.remove(&primary);
    let mut ordered = vec![primary];
    let truncated = ordered.len().saturating_add(aliases.len()) > MAX_EXACT_ALIASES;
    ordered.extend(
        aliases
            .into_iter()
            .take(MAX_EXACT_ALIASES.saturating_sub(ordered.len())),
    );
    (ordered, truncated)
}

async fn fallback_candidates(
    input: &str,
    context: &VariantArticleResolutionContext,
    execution: &VariantArticleExecutionContext,
) -> (
    Vec<ArticleCandidate>,
    bool,
    bool,
    Vec<VariantArticleSourceStatus>,
) {
    let (aliases, alias_budget_stopped) = fallback_aliases(input, context);
    federated_alias_candidates(
        aliases,
        alias_budget_stopped,
        "best_effort_free_text",
        false,
        execution,
    )
    .await
}

fn route_score(candidate: &ArticleCandidate) -> f64 {
    let best = candidate.variant_provenance.iter().fold(
        BTreeMap::<(&str, &str), usize>::new(),
        |mut best, fact| {
            best.entry((&fact.route, &fact.source))
                .and_modify(|position| *position = (*position).min(fact.native_position))
                .or_insert(fact.native_position);
            best
        },
    );
    best.values()
        .map(|position| 1.0 / (60.0 + *position as f64))
        .sum()
}

fn rank_candidates(candidates: &mut [ArticleCandidate]) {
    candidates.sort_by(|left, right| {
        let left_exact = left
            .variant_provenance
            .iter()
            .any(|fact| fact.route != "best_effort_free_text");
        let right_exact = right
            .variant_provenance
            .iter()
            .any(|fact| fact.route != "best_effort_free_text");
        right_exact
            .cmp(&left_exact)
            .then_with(|| {
                route_score(right)
                    .partial_cmp(&route_score(left))
                    .unwrap_or(Ordering::Equal)
            })
            .then_with(|| {
                left.variant_provenance
                    .iter()
                    .map(|fact| fact.native_position)
                    .min()
                    .cmp(
                        &right
                            .variant_provenance
                            .iter()
                            .map(|fact| fact.native_position)
                            .min(),
                    )
            })
            .then_with(|| {
                stable_article_identifier(&left.row).cmp(&stable_article_identifier(&right.row))
            })
    });
}

async fn enrich_candidates(
    candidates: &mut [ArticleCandidate],
    execution: &VariantArticleExecutionContext,
) {
    let mut rows = candidates
        .iter()
        .map(|candidate| candidate.row.clone())
        .collect::<Vec<_>>();
    let _ =
        enrich_article_search_rows_with_semantic_scholar_context(&mut rows, Some(execution)).await;
    enrich_visible_article_search_rows_with_article_base_context(&mut rows, Some(execution)).await;
    for (candidate, row) in candidates.iter_mut().zip(rows) {
        candidate.row = row;
    }
}

fn plan_queries(
    input: &str,
    context: &VariantArticleResolutionContext,
    route: &str,
) -> Vec<String> {
    match route {
        "exact_lexical" => exact_aliases(context).0,
        "best_effort_free_text" => fallback_aliases(input, context).0,
        "source_citation" => context
            .source_id
            .clone()
            .or_else(|| Some(input.to_string()))
            .into_iter()
            .collect(),
        "pubtator_variant" if context.requested.is_authoritative_refseq() => {
            exact_aliases(context).0
        }
        _ => {
            let aliases = combined_normalized_aliases(context);
            aliases
                .genomic_ids
                .into_iter()
                .chain(aliases.rsids)
                .chain(aliases.protein_changes)
                .chain(aliases.coding_changes)
                .chain(std::iter::once(input.to_string()))
                .filter(|query| !query.trim().is_empty())
                .take(MAX_EXACT_ALIASES)
                .collect()
        }
    }
}

fn build_debug_plan(
    input: &str,
    context: &VariantArticleResolutionContext,
    strategy: VariantArticleStrategy,
    execution: &VariantArticleExecutionContext,
    counts: VariantArticleCountsPlan,
    truncated: bool,
    next: VariantArticleNextPlan,
) -> VariantArticleDebugPlan {
    let resolved = matches!(context.resolution.status, VariantResolutionStatus::Resolved);
    let contradictory = matches!(
        context.resolution.provider_validation.status,
        VariantProviderValidationStatus::Contradictory
    );
    let route_names: Vec<&str> = if contradictory {
        match strategy {
            VariantArticleStrategy::Union => vec!["best_effort_free_text"],
            VariantArticleStrategy::Annotation | VariantArticleStrategy::Lexical => Vec::new(),
        }
    } else if !resolved {
        match strategy {
            VariantArticleStrategy::Union => vec!["best_effort_free_text"],
            VariantArticleStrategy::Annotation => vec!["pubtator_variant"],
            VariantArticleStrategy::Lexical => vec!["exact_lexical"],
        }
    } else {
        match strategy {
            VariantArticleStrategy::Union => {
                vec!["pubtator_variant", "exact_lexical", "source_citation"]
            }
            VariantArticleStrategy::Annotation => vec!["pubtator_variant"],
            VariantArticleStrategy::Lexical => vec!["exact_lexical"],
        }
    };
    let events = execution.events();
    let routes = route_names
        .into_iter()
        .map(|route| {
            let queries = plan_queries(input, context, route);
            let mut grouped = BTreeMap::<String, Vec<&VariantArticleCallEvent>>::new();
            for event in events.iter().filter(|event| event.route == route) {
                grouped.entry(event.source.clone()).or_default().push(event);
            }
            let providers = if grouped.is_empty() {
                vec![VariantArticleProviderPlan {
                    source: "federated".into(),
                    status: "skipped".into(),
                    latency_ms: 0,
                    calls: 0,
                    pages: 0,
                    cache: "not_applicable",
                }]
            } else {
                grouped
                    .into_iter()
                    .map(|(source, events)| {
                        let successful = events.iter().filter(|event| event.status == "ok").count();
                        let status = match successful {
                            0 => "unavailable",
                            count if count == events.len() => "ok",
                            _ => "degraded",
                        };
                        VariantArticleProviderPlan {
                            source,
                            status: status.into(),
                            latency_ms: events.iter().map(|event| event.latency_ms).sum(),
                            calls: events.len(),
                            pages: events.iter().map(|event| event.pages).sum(),
                            cache: "unavailable",
                        }
                    })
                    .collect()
            };
            VariantArticleRoutePlan {
                route: route.into(),
                queries,
                providers,
            }
        })
        .collect::<Vec<_>>();
    let item_work = execution.item_work();
    let request_work = execution.request_work();
    VariantArticleDebugPlan {
        normalized_aliases: context.resolution.normalized_aliases.clone(),
        routes,
        counts,
        ranking: VariantArticleRankingPlan {
            method: "exact route union with deterministic native-position ranking",
            inputs: ["exactness", "route_source_position", "stable_identifier"],
        },
        budgets: VariantArticleBudgetsPlan {
            item: item_work,
            request: request_work,
        },
        truncated,
        stopped_routes: execution.stopped_routes(),
        next,
    }
}

pub async fn search_variant_articles(
    input: &str,
    strategy: VariantArticleStrategy,
    limit: usize,
    offset: usize,
) -> Result<VariantArticleOutcome, BioMcpError> {
    search_variant_articles_with_plan(input, strategy, limit, offset, false).await
}

pub async fn search_variant_articles_with_plan(
    input: &str,
    strategy: VariantArticleStrategy,
    limit: usize,
    offset: usize,
    debug_plan: bool,
) -> Result<VariantArticleOutcome, BioMcpError> {
    let requested = RequestedVariantIdentity::from_variant_input(input)?;
    search_variant_articles_identity(
        input,
        requested,
        strategy,
        limit,
        offset,
        debug_plan,
        VariantArticleExecutionContext::single(),
    )
    .await
}

async fn search_variant_articles_identity(
    input: &str,
    requested: RequestedVariantIdentity,
    strategy: VariantArticleStrategy,
    limit: usize,
    offset: usize,
    include_debug_plan: bool,
    execution: VariantArticleExecutionContext,
) -> Result<VariantArticleOutcome, BioMcpError> {
    if limit == 0 || limit > 50 {
        return Err(BioMcpError::InvalidArgument(
            "--limit must be between 1 and 50".to_string(),
        ));
    }
    let mut context =
        crate::entities::variant::resolve_article_variant_identity(requested, input, &execution)
            .await?;
    context.resolution.normalized_aliases = combined_normalized_aliases(&context);
    if !context.available {
        let debug_plan = include_debug_plan.then(|| {
            let mut plan =
                empty_debug_plan(&context.requested, true, vec!["resolution".into()], offset);
            plan.budgets.item = execution.item_work();
            plan.budgets.request = execution.request_work();
            plan
        });
        return Ok(VariantArticleOutcome {
            response: VariantArticleResponse {
                requested_variant: context.requested,
                resolution: context.resolution,
                strategy,
                complete: false,
                truncated: true,
                pagination: VariantArticlePagination {
                    offset,
                    limit,
                    returned: 0,
                    total: None,
                    has_more: false,
                    next_page_token: None,
                },
                source_status: vec![status("resolution", "myvariant", "unavailable")],
                retrieval_path: "variant resolution unavailable",
                results: Vec::new(),
                debug_plan,
            },
            hard_error: true,
        });
    }
    let resolved = matches!(context.resolution.status, VariantResolutionStatus::Resolved);
    let mut candidates = Vec::new();
    let mut statuses = Vec::new();
    let mut succeeded_routes = 0usize;
    let mut failed_routes = 0usize;

    if !resolved {
        match strategy {
            VariantArticleStrategy::Union => {
                match context.resolution.provider_validation.status {
                    VariantProviderValidationStatus::Contradictory => {
                        statuses.push(status_with_detail(
                            "source_citation",
                            "myvariant",
                            "skipped",
                            Some("provider identity contradicted request"),
                        ))
                    }
                    VariantProviderValidationStatus::NotFound => statuses.push(status_with_detail(
                        "source_citation",
                        "myvariant",
                        "skipped",
                        Some("no compatible MyVariant record"),
                    )),
                    VariantProviderValidationStatus::Indeterminate => {
                        statuses.push(status_with_detail(
                            "source_citation",
                            "myvariant",
                            "skipped",
                            Some("provider identity was not confirmable"),
                        ))
                    }
                    VariantProviderValidationStatus::Unavailable => {
                        statuses.push(status_with_detail(
                            "source_citation",
                            "myvariant",
                            "skipped",
                            Some("provider validation unavailable"),
                        ))
                    }
                    VariantProviderValidationStatus::Confirmed => {}
                }
                let (rows, incomplete, succeeded, route_statuses) =
                    fallback_candidates(input, &context, &execution).await;
                candidates.extend(rows);
                statuses.extend(route_statuses);
                succeeded_routes += usize::from(succeeded);
                failed_routes += usize::from(incomplete || !succeeded);
            }
            VariantArticleStrategy::Annotation => {
                if matches!(
                    context.resolution.provider_validation.status,
                    VariantProviderValidationStatus::Contradictory
                ) {
                    statuses.push(status_with_detail(
                        "pubtator_variant",
                        "pubtator",
                        "skipped",
                        Some("provider identity contradicted request"),
                    ));
                } else {
                    statuses.push(status("pubtator_variant", "pubtator", "ok"));
                }
                succeeded_routes += 1;
            }
            VariantArticleStrategy::Lexical => {
                if matches!(
                    context.resolution.provider_validation.status,
                    VariantProviderValidationStatus::Contradictory
                ) {
                    statuses.push(status_with_detail(
                        "exact_lexical",
                        "federated",
                        "skipped",
                        Some("provider identity contradicted request"),
                    ));
                } else {
                    statuses.push(status("exact_lexical", "federated", "ok"));
                }
                succeeded_routes += 1;
            }
        }
    } else {
        if matches!(
            strategy,
            VariantArticleStrategy::Union | VariantArticleStrategy::Annotation
        ) {
            match annotation_candidates(input, &context, &execution).await {
                Ok((rows, incomplete, succeeded)) => {
                    candidates.extend(rows);
                    let state = if !succeeded {
                        "unavailable"
                    } else if incomplete {
                        "degraded"
                    } else {
                        "ok"
                    };
                    statuses.push(status_with_detail(
                        "pubtator_variant",
                        "pubtator",
                        state,
                        incomplete.then_some(
                            "one or more annotation tokens stopped before the route bound",
                        ),
                    ));
                    succeeded_routes += usize::from(succeeded);
                    failed_routes += usize::from(incomplete || !succeeded);
                }
                Err(_) => {
                    statuses.push(status("pubtator_variant", "pubtator", "unavailable"));
                    failed_routes += 1;
                }
            }
        }
        if matches!(
            strategy,
            VariantArticleStrategy::Union | VariantArticleStrategy::Lexical
        ) {
            let (rows, incomplete, succeeded, route_statuses) =
                lexical_candidates(&context, &execution).await;
            candidates.extend(rows);
            statuses.extend(route_statuses);
            succeeded_routes += usize::from(succeeded);
            failed_routes += usize::from(incomplete || !succeeded);
        }
        if strategy == VariantArticleStrategy::Union {
            match context.resolution.provider_validation.status {
                VariantProviderValidationStatus::Confirmed => {
                    match citation_candidates(&context, &execution).await {
                        Ok(rows) => {
                            candidates.extend(rows);
                            statuses.push(status("source_citation", "myvariant", "ok"));
                            succeeded_routes += 1;
                        }
                        Err(_) => {
                            statuses.push(status("source_citation", "myvariant", "unavailable"));
                            failed_routes += 1;
                        }
                    }
                }
                VariantProviderValidationStatus::NotFound => statuses.push(status_with_detail(
                    "source_citation",
                    "myvariant",
                    "skipped",
                    Some("no compatible MyVariant record"),
                )),
                VariantProviderValidationStatus::Indeterminate => {
                    statuses.push(status_with_detail(
                        "source_citation",
                        "myvariant",
                        "skipped",
                        Some("provider identity was not confirmable"),
                    ));
                    failed_routes += 1;
                }
                VariantProviderValidationStatus::Unavailable => {
                    statuses.push(status_with_detail(
                        "source_citation",
                        "myvariant",
                        "skipped",
                        Some("provider validation unavailable"),
                    ));
                    failed_routes += 1;
                }
                VariantProviderValidationStatus::Contradictory => {}
            }
        }
    }

    let pre_dedup = candidates.len();
    let mut candidates = merge_article_candidate_pool(candidates);
    rank_candidates(&mut candidates);
    let total_candidates = candidates.len();
    let hard_error = succeeded_routes == 0 && failed_routes > 0;
    let has_more = offset.saturating_add(limit) < total_candidates;
    let mut visible_candidates = candidates
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect::<Vec<_>>();
    enrich_candidates(&mut visible_candidates, &execution).await;
    let budget_stopped = !execution.stopped_routes().is_empty();
    let provider_incomplete = matches!(
        context.resolution.provider_validation.status,
        VariantProviderValidationStatus::Indeterminate
            | VariantProviderValidationStatus::Unavailable
    );
    let complete = failed_routes == 0 && !budget_stopped && !provider_incomplete;
    let truncated = !complete || offset > 0 || has_more;
    let rows = visible_candidates
        .into_iter()
        .enumerate()
        .map(|(index, candidate)| {
            let matched_aliases = candidate
                .variant_provenance
                .iter()
                .filter_map(|fact| fact.matched_alias.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            let retrieval_routes = candidate
                .variant_provenance
                .iter()
                .map(|fact| fact.route.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            let sources = candidate
                .variant_provenance
                .iter()
                .map(|fact| fact.source.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            VariantArticleRow {
                article: candidate.row,
                requested_variant: context.requested.clone(),
                matched_aliases,
                retrieval_routes,
                sources,
                rank: offset.saturating_add(index).saturating_add(1),
                provenance: candidate.variant_provenance,
            }
        })
        .collect::<Vec<_>>();
    statuses.sort_by(|left, right| (&left.route, &left.source).cmp(&(&right.route, &right.source)));
    let retrieval_path = if !resolved {
        VARIANT_FALLBACK_RETRIEVAL_PATH
    } else if strategy == VariantArticleStrategy::Annotation {
        VARIANT_ENTITY_RETRIEVAL_PATH
    } else if strategy == VariantArticleStrategy::Lexical {
        "resolved exact-alias article search"
    } else {
        "resolved exact-variant route union"
    };
    let debug_plan = include_debug_plan.then(|| {
        build_debug_plan(
            input,
            &context,
            strategy,
            &execution,
            VariantArticleCountsPlan {
                pre_dedup,
                post_dedup: total_candidates,
                returned: rows.len(),
            },
            truncated,
            VariantArticleNextPlan {
                offset: offset.saturating_add(rows.len()),
                cursor: None,
            },
        )
    });
    Ok(VariantArticleOutcome {
        response: VariantArticleResponse {
            requested_variant: context.requested,
            resolution: context.resolution,
            strategy,
            complete,
            truncated,
            pagination: VariantArticlePagination {
                offset,
                limit,
                returned: rows.len(),
                total: complete.then_some(total_candidates),
                has_more,
                next_page_token: None,
            },
            source_status: statuses,
            retrieval_path,
            results: rows,
            debug_plan,
        },
        hard_error,
    })
}

pub(crate) fn parse_variant_article_batch(
    bytes: &[u8],
) -> Result<Vec<crate::entities::variant::VariantArticleRequest>, BioMcpError> {
    const MAX_INPUT_BYTES: usize = 64 * 1024;
    if bytes.len() > MAX_INPUT_BYTES {
        return Err(BioMcpError::InvalidArgument(format!(
            "variant article input must not exceed {MAX_INPUT_BYTES} bytes"
        )));
    }
    serde_json::from_slice::<Vec<crate::entities::variant::VariantArticleRequest>>(bytes).map_err(
        |error| {
            BioMcpError::InvalidArgument(format!("invalid variant article input JSON: {error}"))
        },
    )
}

struct ValidatedBatchItem {
    request_id: String,
    requested: RequestedVariantIdentity,
    input: String,
    error: Option<VariantArticleItemError>,
}

fn item_error(error: BioMcpError) -> VariantArticleItemError {
    let code = error.code();
    let message = match error {
        BioMcpError::InvalidArgument(message) => message,
        other => crate::render::human::sanitize_inline(&other.public_projection().message),
    };
    VariantArticleItemError { code, message }
}

fn empty_debug_plan(
    requested: &RequestedVariantIdentity,
    truncated: bool,
    stopped_routes: Vec<String>,
    offset: usize,
) -> VariantArticleDebugPlan {
    let work = VariantArticleWork::new(ITEM_WORK_LIMIT, 0);
    VariantArticleDebugPlan {
        normalized_aliases: requested.normalized_aliases(),
        routes: Vec::new(),
        counts: VariantArticleCountsPlan {
            pre_dedup: 0,
            post_dedup: 0,
            returned: 0,
        },
        ranking: VariantArticleRankingPlan {
            method: "exact route union with deterministic native-position ranking",
            inputs: ["exactness", "route_source_position", "stable_identifier"],
        },
        budgets: VariantArticleBudgetsPlan {
            item: work.clone(),
            request: work,
        },
        truncated,
        stopped_routes,
        next: VariantArticleNextPlan {
            offset,
            cursor: None,
        },
    }
}

fn validate_batch_requests(
    requests: Vec<crate::entities::variant::VariantArticleRequest>,
) -> Result<Vec<ValidatedBatchItem>, BioMcpError> {
    if requests.is_empty() || requests.len() > 10 {
        return Err(BioMcpError::InvalidArgument(
            "variant article input must contain between 1 and 10 items".into(),
        ));
    }
    let mut ids = BTreeSet::new();
    let mut validated = Vec::with_capacity(requests.len());
    for (index, request) in requests.into_iter().enumerate() {
        let request_id = request
            .request_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| format!("item-{}", index.saturating_add(1)));
        if !ids.insert(request_id.clone()) {
            return Err(BioMcpError::InvalidArgument(format!(
                "duplicate variant article request_id: {request_id}"
            )));
        }
        let requested = request.requested_identity();
        let input = request.display_input(&requested);
        let error = request.validate_identity().err().map(item_error);
        validated.push(ValidatedBatchItem {
            request_id,
            requested,
            input,
            error,
        });
    }
    Ok(validated)
}

fn compact_row(row: VariantArticleRow) -> CompactVariantArticleRow {
    let exact = row
        .retrieval_routes
        .iter()
        .any(|route| route != "best_effort_free_text");
    CompactVariantArticleRow {
        pmid: (!row.article.pmid.trim().is_empty()).then_some(row.article.pmid),
        pmcid: row.article.pmcid,
        doi: row.article.doi,
        arxiv_id: row.article.arxiv_id,
        semantic_scholar_id: row.article.semantic_scholar_id,
        title: row.article.title,
        date: row.article.date,
        match_reason: if exact {
            "exact_variant"
        } else {
            "best_effort_free_text"
        },
        matched_aliases: row.matched_aliases,
        routes: row.retrieval_routes,
        sources: row.sources,
        rank: row.rank,
        is_retracted: row.article.is_retracted,
    }
}

fn empty_item(
    request: ValidatedBatchItem,
    limit: usize,
    offset: usize,
    error: VariantArticleItemError,
    include_debug_plan: bool,
) -> VariantArticleBatchItem {
    let debug_plan =
        include_debug_plan.then(|| empty_debug_plan(&request.requested, false, Vec::new(), offset));
    VariantArticleBatchItem {
        request_id: request.request_id,
        requested_variant: request.requested,
        resolution: None,
        complete: false,
        truncated: false,
        pagination: VariantArticlePagination {
            offset,
            limit,
            returned: 0,
            total: None,
            has_more: false,
            next_page_token: None,
        },
        source_status: Vec::new(),
        error: Some(error),
        results: Vec::new(),
        debug_plan,
    }
}

async fn execute_batch_item(
    request: ValidatedBatchItem,
    strategy: VariantArticleStrategy,
    limit: usize,
    offset: usize,
    debug_plan: bool,
    execution: VariantArticleExecutionContext,
) -> VariantArticleBatchItem {
    if let Some(error) = request.error.clone() {
        return empty_item(request, limit, offset, error, debug_plan);
    }
    let fallback = ValidatedBatchItem {
        request_id: request.request_id.clone(),
        requested: request.requested.clone(),
        input: request.input.clone(),
        error: None,
    };
    let outcome = match search_variant_articles_identity(
        &request.input,
        request.requested,
        strategy,
        limit,
        offset,
        debug_plan,
        execution,
    )
    .await
    {
        Ok(outcome) => outcome,
        Err(error) => {
            return empty_item(fallback, limit, offset, item_error(error), debug_plan);
        }
    };
    let error = outcome.hard_error.then(|| VariantArticleItemError {
        code: "source_unavailable",
        message: "all required variant article routes were unavailable".into(),
    });
    let response = outcome.response;
    VariantArticleBatchItem {
        request_id: request.request_id,
        requested_variant: response.requested_variant,
        resolution: Some(response.resolution),
        complete: response.complete && error.is_none(),
        truncated: response.truncated,
        pagination: response.pagination,
        source_status: response.source_status,
        error,
        results: response.results.into_iter().map(compact_row).collect(),
        debug_plan: response.debug_plan,
    }
}

fn stable_row_id(row: &CompactVariantArticleRow) -> Option<String> {
    row.pmid
        .as_ref()
        .or(row.pmcid.as_ref())
        .or(row.doi.as_ref())
        .or(row.arxiv_id.as_ref())
        .or(row.semantic_scholar_id.as_ref())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn batch_next_commands(items: &[VariantArticleBatchItem]) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let ids = items
        .iter()
        .flat_map(|item| &item.results)
        .filter_map(stable_row_id)
        .filter(|id| seen.insert(id.clone()))
        .collect::<Vec<_>>();
    let mut commands = Vec::new();
    if !ids.is_empty() {
        commands.push(
            crate::next_command::NextCommand::biomcp()
                .args(["article", "batch"])
                .args(ids.iter().take(10).cloned())
                .render_shell(),
        );
    }
    for id in ids.iter().take(3) {
        for section in [None, Some("fulltext"), Some("assets")] {
            let mut command = crate::next_command::NextCommand::biomcp()
                .args(["get", "article"])
                .arg(id);
            if let Some(section) = section {
                command = command.arg(section);
            }
            commands.push(command.render_shell());
        }
        commands.push(
            crate::next_command::NextCommand::biomcp()
                .args(["article", "citations"])
                .arg(id)
                .render_shell(),
        );
    }
    commands
}

async fn collect_bounded_ordered<F, T>(futures: Vec<F>) -> Vec<T>
where
    F: std::future::Future<Output = T>,
{
    stream::iter(futures)
        .buffered(ITEM_CONCURRENCY_LIMIT)
        .collect()
        .await
}

pub async fn search_variant_article_batch(
    requests: Vec<crate::entities::variant::VariantArticleRequest>,
    strategy: VariantArticleStrategy,
    limit: usize,
    offset: usize,
    debug_plan: bool,
) -> Result<VariantArticleBatchOutcome, BioMcpError> {
    if limit == 0 || limit > 50 {
        return Err(BioMcpError::InvalidArgument(
            "--limit must be between 1 and 50".into(),
        ));
    }
    let validated = validate_batch_requests(requests)?;
    let item_count = validated.len();
    let contexts = VariantArticleExecutionContext::batch(item_count);
    let request_context = contexts.first().cloned();
    let futures = validated
        .into_iter()
        .zip(contexts)
        .map(|(request, execution)| {
            execute_batch_item(request, strategy, limit, offset, debug_plan, execution)
        })
        .collect();
    let mut items = collect_bounded_ordered(futures).await;
    let request_work = request_context
        .map(|context| context.request_work())
        .unwrap_or_else(|| VariantArticleWork::new(0, 0));
    if debug_plan {
        for item in &mut items {
            if let Some(plan) = &mut item.debug_plan {
                plan.budgets.request = request_work.clone();
            }
        }
    }
    let hard_error = items.iter().any(|item| item.error.is_some());
    let complete = items
        .iter()
        .all(|item| item.complete && item.error.is_none());
    let truncated = items.iter().any(|item| item.truncated);
    let next_commands = batch_next_commands(&items);
    Ok(VariantArticleBatchOutcome {
        response: VariantArticleBatchResponse {
            items,
            complete,
            truncated,
            _meta: VariantArticleBatchMeta { next_commands },
            debug_plan: debug_plan.then_some(VariantArticleBatchDebugPlan {
                item_concurrency_limit: ITEM_CONCURRENCY_LIMIT,
                work: request_work,
                items_planned: item_count,
            }),
        },
        hard_error,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering as AtomicOrdering},
    };

    use crate::entities::article::test_support::row;
    use crate::entities::variant::{
        SourceVariantIdentity, VariantArticleResolutionBasis, VariantProviderValidation,
    };

    fn resolved_context() -> VariantArticleResolutionContext {
        let requested = RequestedVariantIdentity::from_variant_input("BRAF p.V600E")
            .expect("requested identity");
        VariantArticleResolutionContext {
            resolution: VariantArticleResolution {
                status: VariantResolutionStatus::Resolved,
                normalized_aliases: requested.normalized_aliases(),
                exhaustive: true,
                basis: Some(VariantArticleResolutionBasis::ProviderConfirmed),
                provider_validation: VariantProviderValidation {
                    source: "myvariant".into(),
                    status: VariantProviderValidationStatus::Confirmed,
                    matched_alias: Some("p.V600E".into()),
                    contradictory_field: None,
                },
            },
            requested,
            source_id: Some("chr7:g.140453136A>T".into()),
            source_identity: Some(SourceVariantIdentity {
                genomic_id: "chr7:g.140453136A>T".into(),
                genes: vec!["BRAF".into()],
                protein_changes: vec!["p.V600E".into(), "p.Val600Glu".into()],
                coding_changes: vec!["c.1799T>A".into()],
                rsids: vec!["rs113488022".into()],
            }),
            source_hit: None,
            fallback_source_identities: Vec::new(),
            available: true,
        }
    }

    fn lexical_candidate(pmid: &str, positions: &[usize]) -> ArticleCandidate {
        let mut candidate = article_candidate_from_row(row(pmid, ArticleSource::PubTator));
        candidate.variant_provenance = positions
            .iter()
            .map(|position| VariantArticleProvenance {
                route: "exact_lexical".into(),
                source: "pubtator".into(),
                matched_alias: Some(format!("alias-{position}")),
                native_position: *position,
            })
            .collect();
        candidate
    }

    #[test]
    fn resolved_exact_aliases_include_validated_source_forms_once() {
        let (aliases, truncated) = exact_aliases(&resolved_context());
        assert!(!truncated);
        assert_eq!(
            aliases,
            vec![
                "BRAF p.V600E",
                "BRAF c.1799T>A",
                "BRAF p.Val600Glu",
                "chr7:g.140453136A>T",
                "rs113488022",
            ]
        );
    }

    #[test]
    fn refseq_exact_aliases_are_only_caller_literal_forms() {
        let requested = RequestedVariantIdentity {
            gene: Some("ATM".into()),
            coding_change: Some("c.1066-6T>G".into()),
            transcript: Some("NM_000051.4".into()),
            genomic_accession: Some("NC_000011.10".into()),
            genome_build: Some("GRCh38".into()),
            position: Some(108248927),
            reference: Some("T".into()),
            alternate: Some("G".into()),
            ..Default::default()
        };
        let mut context = VariantArticleResolutionContext {
            resolution: VariantArticleResolution {
                status: VariantResolutionStatus::Resolved,
                normalized_aliases: requested.normalized_aliases(),
                exhaustive: true,
                basis: Some(VariantArticleResolutionBasis::CallerSupplied),
                provider_validation: VariantProviderValidation {
                    source: "myvariant".into(),
                    status: VariantProviderValidationStatus::NotFound,
                    matched_alias: None,
                    contradictory_field: None,
                },
            },
            requested,
            source_id: None,
            source_identity: Some(SourceVariantIdentity {
                genomic_id: "chr11:g.108248927T>G".into(),
                genes: vec!["ATM".into()],
                protein_changes: vec!["p.?".into()],
                coding_changes: vec!["c.1A>T".into()],
                rsids: vec!["rs1".into()],
            }),
            source_hit: None,
            fallback_source_identities: Vec::new(),
            available: true,
        };
        assert_eq!(
            exact_aliases(&context).0,
            vec![
                "ATM c.1066-6T>G",
                "NC_000011.10:g.108248927T>G",
                "NM_000051.4:c.1066-6T>G",
            ]
        );
        assert_eq!(
            combined_normalized_aliases(&context),
            context.requested.normalized_aliases()
        );
        assert_eq!(
            primary_exact_alias(&context).as_deref(),
            Some("ATM c.1066-6T>G")
        );
        assert_eq!(
            plan_queries("NC_000011.10:g.108248927T>G", &context, "pubtator_variant"),
            exact_aliases(&context).0
        );

        context.resolution.status = VariantResolutionStatus::Unresolved;
        context.resolution.basis = None;
        context.resolution.provider_validation.status =
            VariantProviderValidationStatus::Contradictory;
        let plan = build_debug_plan(
            "NC_000011.10:g.108248927T>G",
            &context,
            VariantArticleStrategy::Annotation,
            &VariantArticleExecutionContext::single(),
            VariantArticleCountsPlan {
                pre_dedup: 0,
                post_dedup: 0,
                returned: 0,
            },
            false,
            VariantArticleNextPlan {
                offset: 0,
                cursor: None,
            },
        );
        assert!(plan.routes.is_empty());
    }

    #[test]
    fn lexical_alias_budget_preserves_the_requested_identity_and_reports_truncation() {
        let mut context = resolved_context();
        context
            .source_identity
            .as_mut()
            .expect("source identity")
            .coding_changes
            .extend((0..20).map(|index| format!("c.{index}A>T")));

        let (aliases, truncated) = exact_aliases(&context);

        assert!(truncated);
        assert_eq!(aliases.first().map(String::as_str), Some("BRAF p.V600E"));
    }

    #[test]
    fn ambiguous_fallback_uses_only_request_compatible_shared_source_aliases() {
        let requested = RequestedVariantIdentity::from_variant_input("MSH2 p.L341P")
            .expect("requested identity");
        let mut context = VariantArticleResolutionContext {
            resolution: VariantArticleResolution {
                status: VariantResolutionStatus::Ambiguous,
                normalized_aliases: requested.normalized_aliases(),
                exhaustive: true,
                basis: None,
                provider_validation: VariantProviderValidation {
                    source: "myvariant".into(),
                    status: VariantProviderValidationStatus::Indeterminate,
                    matched_alias: None,
                    contradictory_field: None,
                },
            },
            requested,
            source_id: None,
            source_identity: None,
            source_hit: None,
            fallback_source_identities: vec![
                SourceVariantIdentity {
                    genomic_id: "first".into(),
                    genes: vec!["MSH2".into()],
                    protein_changes: vec!["p.L341P".into(), "p.Leu341Pro".into()],
                    coding_changes: vec!["c.1022T>C".into(), "c.824T>C".into()],
                    rsids: Vec::new(),
                },
                SourceVariantIdentity {
                    genomic_id: "second".into(),
                    genes: vec!["MSH2".into()],
                    protein_changes: vec!["p.Leu341Pro".into(), "p.L341P".into()],
                    coding_changes: vec!["c.1022T>C".into(), "c.1220T>C".into()],
                    rsids: Vec::new(),
                },
            ],
            available: true,
        };
        context.resolution.normalized_aliases = combined_normalized_aliases(&context);

        let (aliases, truncated) = fallback_aliases("MSH2 p.L341P", &context);

        assert!(!truncated);
        assert!(aliases.iter().any(|alias| alias == "MSH2 L341P"));
        assert!(aliases.iter().any(|alias| alias == "MSH2 p.Leu341Pro"));
        assert!(aliases.iter().any(|alias| alias == "MSH2 c.1022T>C"));
        assert!(!aliases.iter().any(|alias| alias == "MSH2 c.824T>C"));
        assert!(!aliases.iter().any(|alias| alias == "MSH2 c.1220T>C"));
    }

    #[test]
    fn ranking_counts_only_the_best_alias_position_per_route_and_provider() {
        let mut candidates = vec![
            lexical_candidate("2", &[1, 2, 3]),
            lexical_candidate("1", &[1]),
        ];

        rank_candidates(&mut candidates);

        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.row.pmid.as_str())
                .collect::<Vec<_>>(),
            vec!["1", "2"]
        );
    }

    #[test]
    fn structured_input_enforces_byte_count_item_count_and_duplicate_ids() {
        let mut exact = br#"[{"gene":"BRAF","protein":"V600E"}]"#.to_vec();
        exact.resize(64 * 1024, b' ');
        assert_eq!(
            parse_variant_article_batch(&exact)
                .expect("64 KiB input")
                .len(),
            1
        );
        exact.push(b' ');
        assert!(matches!(
            parse_variant_article_batch(&exact),
            Err(BioMcpError::InvalidArgument(_))
        ));

        assert!(validate_batch_requests(Vec::new()).is_err());
        let eleven =
            serde_json::from_value::<Vec<crate::entities::variant::VariantArticleRequest>>(
                serde_json::Value::Array(
                    (0..11)
                        .map(|_| serde_json::json!({"gene":"BRAF","protein":"V600E"}))
                        .collect(),
                ),
            )
            .expect("typed requests");
        assert!(validate_batch_requests(eleven).is_err());
        let duplicates = parse_variant_article_batch(
            br#"[{"request_id":"same","rsid":"rs1"},{"request_id":"same","rsid":"rs2"}]"#,
        )
        .expect("typed duplicate request");
        assert!(validate_batch_requests(duplicates).is_err());
    }

    #[test]
    fn structured_identity_accepts_only_documented_anchors() {
        let rows = [
            (serde_json::json!({"rsid":"rs113488022"}), true),
            (serde_json::json!({"genomic":"chr7:g.140453136A>T"}), true),
            (
                serde_json::json!({"accession":"chr7","build":"GRCh38","position":140453136,"ref":"A","alt":"T"}),
                true,
            ),
            (serde_json::json!({"gene":"BRAF","protein":"p.V600E"}), true),
            (
                serde_json::json!({"transcript":"NM_004333.6","coding":"c.1799T>A"}),
                true,
            ),
            (serde_json::json!({"gene":"BRAF"}), false),
            (serde_json::json!({"protein":"V600E"}), false),
            (
                serde_json::json!({"transcript":"NM_004333.6","coding":"c."}),
                false,
            ),
            (
                serde_json::json!({"gene":"BRAF","coding":"c.not-hgvs"}),
                false,
            ),
            (
                serde_json::json!({"rsid":"rs113488022","protein":"not-a-change"}),
                false,
            ),
            (
                serde_json::json!({"rsid":"rs113488022","transcript":"not-a-transcript"}),
                false,
            ),
            (serde_json::json!({"accession":"chr7","position":1}), false),
            (
                serde_json::json!({"accession":"not-an-accession","position":0,"ref":"garbage","alt":"also-garbage"}),
                false,
            ),
            (
                serde_json::json!({"accession":"chr7","build":"unknown","position":140453136,"ref":"A","alt":"T"}),
                false,
            ),
            (serde_json::json!({"genomic":"chr7:g.0A>T"}), false),
            (
                serde_json::json!({"genomic":"chr7:g.140453136A>T","accession":"chr7"}),
                false,
            ),
        ];
        for (value, accepted) in rows {
            let request =
                serde_json::from_value::<crate::entities::variant::VariantArticleRequest>(
                    value.clone(),
                )
                .expect("typed request");
            assert_eq!(
                request.validate_identity().is_ok(),
                accepted,
                "request={value}"
            );
        }
    }

    #[test]
    fn item_and_request_work_budgets_stop_at_fifty_and_five_hundred() {
        let contexts = VariantArticleExecutionContext::batch(10);
        for context in &contexts {
            for _ in 0..ITEM_WORK_LIMIT {
                assert!(context.reserve("exact_lexical").is_some());
            }
            assert!(context.reserve("exact_lexical").is_none());
            assert_eq!(context.item_work().consumed, ITEM_WORK_LIMIT);
            assert_eq!(context.stopped_routes(), vec!["exact_lexical"]);
        }
        assert_eq!(contexts[0].request_work().consumed, 500);
        assert!(contexts[0].request_work().exhausted);
    }

    #[test]
    fn structured_genomic_identity_is_the_resolution_lookup_key() {
        let request = serde_json::from_value::<crate::entities::variant::VariantArticleRequest>(
            serde_json::json!({
                "gene":"BRAF",
                "protein":"p.V600E",
                "accession":"chr7",
                "position":140453136,
                "ref":"A",
                "alt":"T"
            }),
        )
        .expect("typed request");
        let identity = request
            .validate_identity()
            .expect("valid conjunctive identity");

        assert_eq!(request.display_input(&identity), "chr7:g.140453136A>T");
    }

    #[tokio::test]
    async fn ordered_executor_never_runs_more_than_two_items() {
        let active = Arc::new(AtomicUsize::new(0));
        let maximum = Arc::new(AtomicUsize::new(0));
        let futures = (0..6)
            .map(|index| {
                let active = active.clone();
                let maximum = maximum.clone();
                async move {
                    let now = active.fetch_add(1, AtomicOrdering::SeqCst) + 1;
                    maximum.fetch_max(now, AtomicOrdering::SeqCst);
                    tokio::task::yield_now().await;
                    tokio::task::yield_now().await;
                    active.fetch_sub(1, AtomicOrdering::SeqCst);
                    index
                }
            })
            .collect();

        let completed = collect_bounded_ordered(futures).await;

        assert_eq!(maximum.load(AtomicOrdering::SeqCst), 2);
        assert_eq!(completed, (0..6).collect::<Vec<_>>());
    }

    #[tokio::test]
    async fn invalid_items_are_retained_without_provider_execution() {
        let requests = parse_variant_article_batch(
            br#"[{"request_id":"first","gene":"BRAF"},{"protein":"V600E"}]"#,
        )
        .expect("typed requests");
        let outcome =
            search_variant_article_batch(requests, VariantArticleStrategy::Union, 3, 0, true)
                .await
                .expect("invalid items are in-band");

        assert!(outcome.hard_error);
        assert_eq!(outcome.response.items.len(), 2);
        assert_eq!(outcome.response.items[0].request_id, "first");
        assert_eq!(outcome.response.items[1].request_id, "item-2");
        assert!(outcome.response.items.iter().all(|item| {
            item.resolution.is_none()
                && item.error.is_some()
                && item.results.is_empty()
                && item
                    .debug_plan
                    .as_ref()
                    .is_some_and(|plan| plan.budgets.item.consumed == 0 && plan.routes.is_empty())
        }));
    }

    #[test]
    fn item_errors_preserve_runtime_error_classification() {
        let error = item_error(BioMcpError::SourceUnavailable {
            source_name: "fixture".into(),
            reason: "offline".into(),
            suggestion: "retry".into(),
        });

        assert_eq!(error.code, "source_unavailable");
        assert!(!error.message.is_empty());
    }

    #[test]
    fn provider_plan_reports_mixed_outcomes_and_unknown_cache_truthfully() {
        let context = resolved_context();
        let execution = VariantArticleExecutionContext::single();
        for status in ["ok", "unavailable"] {
            execution.record(
                "exact_lexical",
                "pubmed",
                Instant::now(),
                status,
                usize::from(status == "ok"),
            );
        }
        let plan = build_debug_plan(
            "BRAF p.V600E",
            &context,
            VariantArticleStrategy::Union,
            &execution,
            VariantArticleCountsPlan {
                pre_dedup: 0,
                post_dedup: 0,
                returned: 0,
            },
            true,
            VariantArticleNextPlan {
                offset: 0,
                cursor: None,
            },
        );
        let provider = plan
            .routes
            .iter()
            .find(|route| route.route == "exact_lexical")
            .and_then(|route| route.providers.first())
            .expect("lexical provider plan");

        assert_eq!(provider.status, "degraded");
        assert_eq!(provider.cache, "unavailable");
    }

    #[test]
    fn compact_projection_omits_verbose_fields_and_preserves_unknown_retraction() {
        let mut candidate = lexical_candidate("6010003", &[1]);
        candidate.row.is_retracted = None;
        candidate.row.abstract_snippet = Some("verbose".into());
        let compact = compact_row(VariantArticleRow {
            article: candidate.row,
            requested_variant: RequestedVariantIdentity::default(),
            matched_aliases: vec!["BRAF V600E".into()],
            retrieval_routes: vec!["exact_lexical".into()],
            sources: vec!["pubtator".into()],
            rank: 1,
            provenance: candidate.variant_provenance,
        });
        let value = serde_json::to_value(compact).expect("compact JSON");

        assert!(value["is_retracted"].is_null());
        for forbidden in ["abstract", "abstract_snippet", "provenance", "ranking"] {
            assert!(
                value.get(forbidden).is_none(),
                "forbidden field {forbidden}"
            );
        }
    }

    #[test]
    fn transitive_merge_retains_associated_variant_provenance() {
        let mut annotation = lexical_candidate("6010003", &[2]);
        annotation.variant_provenance[0].route = "pubtator_variant".into();
        let lexical = lexical_candidate("6010003", &[1]);

        let merged = merge_article_candidate_pool(vec![annotation, lexical]);

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].variant_provenance.len(), 2);
        assert_eq!(
            merged[0]
                .variant_provenance
                .iter()
                .map(|fact| fact.route.as_str())
                .collect::<Vec<_>>(),
            vec!["exact_lexical", "pubtator_variant"]
        );
    }
}
