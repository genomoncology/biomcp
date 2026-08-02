//! Variant-specific article route union, provenance, ranking, and pagination.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering as AtomicOrdering},
};
use std::time::Instant;

use crate::entities::variant::{
    CarNormalizationItem, CarNormalizationStatus, NormalizedVariantAliases,
    RequestedVariantIdentity, VariantArticleResolution, VariantArticleResolutionContext,
    VariantProviderValidationStatus, VariantResolutionStatus,
};
use crate::error::BioMcpError;
use clap::ValueEnum;
use futures::{StreamExt, stream};
use serde::Serialize;

use super::backends::{
    search_europepmc_page_with_context, search_pubmed_page_with_context,
    search_pubtator_page_with_context, search_semantic_scholar_candidates,
};
use super::candidates::{
    ArticleCandidate, article_candidate_from_row, merge_article_candidate_pool,
    stable_article_identifier,
};
use super::enrichment::{
    enrich_article_search_rows_with_semantic_scholar_context,
    enrich_visible_article_search_rows_with_article_base_context,
};
use super::identity_verification::{
    PUBTATOR_EXPORT_TEMPLATE_VERSION, VariantArticleIdentity, VariantArticleVerificationOptions,
    VariantArticleVerificationPlan, canonical_content_subset, canonical_response_subset,
    combine_identities, verification_plan, verify_captured_abstract, verify_ldh_annotation,
    verify_pubtator,
};
use super::query::{
    build_europepmc_variant_strict_query, build_pubmed_variant_strict_query,
    resolve_variant_entity_tokens,
};
use super::search::{
    VARIANT_ENTITY_RETRIEVAL_PATH, VARIANT_FALLBACK_RETRIEVAL_PATH,
    acquire_federated_article_rows_with_context,
};
use super::{
    ArticleRankingOptions, ArticleSearchFilters, ArticleSearchResult, ArticleSort, ArticleSource,
    ArticleSourceAvailability, ArticleVariantIntent, MAX_FEDERATED_FETCH_RESULTS,
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
    pub query_aliases: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_alias: Option<String>,
    pub native_position: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VariantArticleSourceStatusKind {
    Ok,
    Degraded,
    Unavailable,
    Skipped,
    NotAttempted,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariantArticleSourceStatus {
    pub route: String,
    pub source: String,
    pub status: VariantArticleSourceStatusKind,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<VariantArticleIdentity>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CanonicalEquivalenceObservation {
    pub basis: String,
    pub query: String,
    pub status: String,
    pub caid: Option<String>,
    pub provider_exhaustive: bool,
    pub comparison_complete: bool,
    pub source: String,
    pub request_template_version: String,
    pub car_version: Option<String>,
    pub provider_response_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CanonicalEquivalence {
    pub status: String,
    pub caid: Option<String>,
    pub exhaustive: bool,
    pub complete: bool,
    pub applicable_identity_count: usize,
    pub observations: Vec<CanonicalEquivalenceObservation>,
    pub message: String,
    #[serde(skip)]
    aliases: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariantArticleResponse {
    pub requested_variant: RequestedVariantIdentity,
    pub resolution: VariantArticleResolution,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canonical_equivalence: Option<CanonicalEquivalence>,
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
const EXACT_WORK_LIMIT: usize = MAX_EXACT_ALIASES * 5;
const ITEM_CONCURRENCY_LIMIT: usize = 2;
const LDH_MEDIUM_LIMIT: usize = 1;
const LDH_DIRECT_LIMIT: usize = 10;

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

#[derive(Debug)]
struct VariantArticleWorkAllocation {
    identity_verification_reserved: AtomicUsize,
    identity_verification_consumed: AtomicUsize,
}

#[derive(Debug, Clone)]
pub(crate) struct VariantArticleExecutionContext {
    item: Arc<SharedWorkBudget>,
    request: Arc<SharedWorkBudget>,
    exact_item: Arc<SharedWorkBudget>,
    exact_request: Arc<SharedWorkBudget>,
    identity_item: Arc<SharedWorkBudget>,
    identity_request: Arc<SharedWorkBudget>,
    allocation: Arc<VariantArticleWorkAllocation>,
    events: Arc<Mutex<Vec<VariantArticleCallEvent>>>,
    stopped_routes: Arc<Mutex<BTreeSet<String>>>,
    strict_pubtator_queries: Arc<Mutex<BTreeMap<String, String>>>,
    ldh_medium: Arc<AtomicUsize>,
    ldh_direct: Arc<AtomicUsize>,
}

impl VariantArticleExecutionContext {
    fn with_request(request: Arc<SharedWorkBudget>) -> Self {
        Self {
            item: Arc::new(SharedWorkBudget {
                limit: ITEM_WORK_LIMIT,
                consumed: AtomicUsize::new(0),
            }),
            exact_item: Arc::new(SharedWorkBudget {
                limit: EXACT_WORK_LIMIT,
                consumed: AtomicUsize::new(0),
            }),
            exact_request: Arc::new(SharedWorkBudget {
                limit: EXACT_WORK_LIMIT,
                consumed: AtomicUsize::new(0),
            }),
            identity_item: Arc::new(SharedWorkBudget {
                limit: ITEM_WORK_LIMIT,
                consumed: AtomicUsize::new(0),
            }),
            identity_request: Arc::new(SharedWorkBudget {
                limit: ITEM_WORK_LIMIT,
                consumed: AtomicUsize::new(0),
            }),
            request,
            allocation: Arc::new(VariantArticleWorkAllocation {
                identity_verification_reserved: AtomicUsize::new(0),
                identity_verification_consumed: AtomicUsize::new(0),
            }),
            events: Arc::new(Mutex::new(Vec::new())),
            stopped_routes: Arc::new(Mutex::new(BTreeSet::new())),
            strict_pubtator_queries: Arc::new(Mutex::new(BTreeMap::new())),
            ldh_medium: Arc::new(AtomicUsize::new(0)),
            ldh_direct: Arc::new(AtomicUsize::new(0)),
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
        let contexts = (0..item_count)
            .map(|_| Self::with_request(request.clone()))
            .collect::<Vec<_>>();
        let exact_request = Arc::new(SharedWorkBudget {
            limit: EXACT_WORK_LIMIT.saturating_mul(item_count),
            consumed: AtomicUsize::new(0),
        });
        let identity_request = Arc::new(SharedWorkBudget {
            limit: ITEM_WORK_LIMIT.saturating_mul(item_count),
            consumed: AtomicUsize::new(0),
        });
        contexts
            .into_iter()
            .map(|mut context| {
                context.exact_request = exact_request.clone();
                context.identity_request = identity_request.clone();
                context
            })
            .collect()
    }

    fn reserve_identity_verification(&self, count: usize) {
        self.allocation
            .identity_verification_reserved
            .fetch_add(count, AtomicOrdering::SeqCst);
    }

    pub(crate) fn reserve_identity_verification_through(&self, count: usize) {
        let verification_consumed = self
            .allocation
            .identity_verification_consumed
            .load(AtomicOrdering::SeqCst);
        let available_for_verification = self
            .identity_item
            .limit
            .saturating_sub(verification_consumed);
        let target = count.min(available_for_verification);
        let _ = self.allocation.identity_verification_reserved.fetch_update(
            AtomicOrdering::SeqCst,
            AtomicOrdering::SeqCst,
            |current| (current < target).then_some(target),
        );
    }

    pub(crate) fn reserve(&self, route: &str) -> Option<Instant> {
        // ClinGen LDH runs last, after every retrieval route and after
        // per-candidate PubTator verification. Live requests exhaust the shared
        // item budget before it is reached, so drawing from that pool means the
        // LDH ladder never runs at all outside a fixture. Its work is separately
        // and tightly bounded -- one medium lookup and at most ten direct
        // fetches per item -- so it carries its own allowance rather than
        // competing for the general pool.
        if let Some((consumed, cap)) = match route {
            "clingen_ldh_medium" => Some((&self.ldh_medium, LDH_MEDIUM_LIMIT)),
            "clingen_ldh_direct" => Some((&self.ldh_direct, LDH_DIRECT_LIMIT)),
            _ => None,
        } {
            if consumed.fetch_add(1, AtomicOrdering::SeqCst) >= cap {
                consumed.fetch_sub(1, AtomicOrdering::SeqCst);
                self.stop(route);
                return None;
            }
            return Some(Instant::now());
        }
        let identity_verification = route == "identity_verification";
        if matches!(route, "exact_lexical" | "identity_verification") {
            let (item, request) = if identity_verification {
                (&self.identity_item, &self.identity_request)
            } else {
                (&self.exact_item, &self.exact_request)
            };
            let reserve = |budget: &SharedWorkBudget| {
                budget
                    .consumed
                    .fetch_update(AtomicOrdering::SeqCst, AtomicOrdering::SeqCst, |current| {
                        (current < budget.limit).then(|| current + 1)
                    })
                    .is_ok()
            };
            if !reserve(item) {
                self.stop(route);
                return None;
            }
            if !reserve(request) {
                item.consumed.fetch_sub(1, AtomicOrdering::SeqCst);
                self.stop(route);
                return None;
            }
            if identity_verification {
                self.allocation
                    .identity_verification_consumed
                    .fetch_add(1, AtomicOrdering::SeqCst);
            }
            return Some(Instant::now());
        }
        if identity_verification
            && self
                .allocation
                .identity_verification_consumed
                .fetch_update(AtomicOrdering::SeqCst, AtomicOrdering::SeqCst, |current| {
                    (current
                        < self
                            .allocation
                            .identity_verification_reserved
                            .load(AtomicOrdering::SeqCst))
                    .then(|| current + 1)
                })
                .is_err()
        {
            self.stop(route);
            return None;
        }
        let reserved = self
            .allocation
            .identity_verification_reserved
            .load(AtomicOrdering::SeqCst);
        let reserve = |budget: &SharedWorkBudget| {
            budget
                .consumed
                .fetch_update(AtomicOrdering::SeqCst, AtomicOrdering::SeqCst, |current| {
                    ((identity_verification || current.saturating_add(reserved) < budget.limit)
                        && current < budget.limit)
                        .then(|| current + 1)
                })
                .is_ok()
        };
        let protected_discovery = !identity_verification
            && reserved > 0
            && self
                .item
                .consumed
                .load(AtomicOrdering::SeqCst)
                .saturating_add(reserved)
                >= self.item.limit;
        if !reserve(&self.item) {
            if identity_verification {
                self.allocation
                    .identity_verification_consumed
                    .fetch_sub(1, AtomicOrdering::SeqCst);
            }
            if !protected_discovery {
                self.stop(route);
            }
            return None;
        }
        if !reserve(&self.request) {
            self.item.consumed.fetch_sub(1, AtomicOrdering::SeqCst);
            if identity_verification {
                self.allocation
                    .identity_verification_consumed
                    .fetch_sub(1, AtomicOrdering::SeqCst);
            }
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
            self.item.limit + self.exact_item.limit + self.identity_item.limit,
            self.item.consumed.load(AtomicOrdering::SeqCst)
                + self.exact_item.consumed.load(AtomicOrdering::SeqCst)
                + self.identity_item.consumed.load(AtomicOrdering::SeqCst),
        )
    }

    fn request_work(&self) -> VariantArticleWork {
        VariantArticleWork::new(
            self.request.limit,
            self.request.consumed.load(AtomicOrdering::SeqCst),
        )
    }

    fn work_allocation(&self) -> VariantArticleWorkAllocationPlan {
        let reserved = self
            .allocation
            .identity_verification_reserved
            .load(AtomicOrdering::SeqCst);
        let consumed = self
            .allocation
            .identity_verification_consumed
            .load(AtomicOrdering::SeqCst);
        VariantArticleWorkAllocationPlan {
            discovery: VariantArticleWork::new(
                self.item.limit,
                self.item.consumed.load(AtomicOrdering::SeqCst),
            ),
            exact_lexical: VariantArticleWorkAllocationScope {
                item: VariantArticleWork::new(
                    self.exact_item.limit,
                    self.exact_item.consumed.load(AtomicOrdering::SeqCst),
                ),
                request: VariantArticleWork::new(
                    self.exact_request.limit,
                    self.exact_request.consumed.load(AtomicOrdering::SeqCst),
                ),
            },
            identity_verification: VariantArticleIdentityVerificationAllocation {
                reserved,
                consumed,
                item: VariantArticleWork::new(
                    self.identity_item.limit,
                    self.identity_item.consumed.load(AtomicOrdering::SeqCst),
                ),
                request: VariantArticleWork::new(
                    self.identity_request.limit,
                    self.identity_request.consumed.load(AtomicOrdering::SeqCst),
                ),
            },
        }
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

    fn route_stopped(&self, route: &str) -> bool {
        self.stopped_routes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .contains(route)
    }

    fn record_strict_pubtator_query(&self, query_alias: &str, entity_id: &str) {
        self.strict_pubtator_queries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(query_alias.to_string(), entity_id.to_string());
    }

    fn strict_pubtator_query(&self, query_alias: &str) -> Option<String> {
        self.strict_pubtator_queries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(query_alias)
            .cloned()
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
pub struct VariantArticleWorkAllocationScope {
    pub item: VariantArticleWork,
    pub request: VariantArticleWork,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariantArticleIdentityVerificationAllocation {
    pub reserved: usize,
    pub consumed: usize,
    pub item: VariantArticleWork,
    pub request: VariantArticleWork,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariantArticleWorkAllocationPlan {
    pub discovery: VariantArticleWork,
    pub exact_lexical: VariantArticleWorkAllocationScope,
    pub identity_verification: VariantArticleIdentityVerificationAllocation,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariantArticleNextPlan {
    pub offset: usize,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderVariantQueryPlan {
    pub provider: String,
    pub route: String,
    pub query_alias: String,
    pub query: String,
    pub query_template_version: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariantArticleCandidateTraceRecord {
    pub identifier: String,
    pub route: String,
    pub provider_terminal_state: String,
    pub received: bool,
    pub after_union: bool,
    pub after_dedup: bool,
    pub rank_position: Option<usize>,
    pub verification_disposition: String,
    pub pagination_disposition: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariantArticleCandidateTrace {
    pub schema_version: &'static str,
    pub bounded: bool,
    pub candidates: Vec<VariantArticleCandidateTraceRecord>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariantArticleDebugPlan {
    pub normalized_aliases: NormalizedVariantAliases,
    pub provider_queries: Vec<ProviderVariantQueryPlan>,
    pub routes: Vec<VariantArticleRoutePlan>,
    pub counts: VariantArticleCountsPlan,
    pub ranking: VariantArticleRankingPlan,
    pub budgets: VariantArticleBudgetsPlan,
    pub work_allocation: VariantArticleWorkAllocationPlan,
    pub truncated: bool,
    pub stopped_routes: Vec<String>,
    pub next: VariantArticleNextPlan,
    pub candidate_trace: VariantArticleCandidateTrace,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification: Option<VariantArticleVerificationPlan>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<VariantArticleIdentity>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canonical_equivalence: Option<CanonicalEquivalence>,
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

fn status(
    route: &str,
    source: &str,
    status: VariantArticleSourceStatusKind,
) -> VariantArticleSourceStatus {
    status_with_detail(route, source, status, None)
}

fn status_with_detail(
    route: &str,
    source: &str,
    status: VariantArticleSourceStatusKind,
    detail: Option<&str>,
) -> VariantArticleSourceStatus {
    VariantArticleSourceStatus {
        route: route.to_string(),
        source: source.to_string(),
        status,
        detail: detail.map(str::to_string),
    }
}

fn provider_terminal_status(events: &[&VariantArticleCallEvent]) -> VariantArticleSourceStatusKind {
    let successful = events.iter().filter(|event| event.status == "ok").count();
    match successful {
        0 => VariantArticleSourceStatusKind::Unavailable,
        count if count == events.len() => VariantArticleSourceStatusKind::Ok,
        _ => VariantArticleSourceStatusKind::Degraded,
    }
}

fn provider_status_name(status: VariantArticleSourceStatusKind) -> &'static str {
    match status {
        VariantArticleSourceStatusKind::Ok => "ok",
        VariantArticleSourceStatusKind::Degraded => "degraded",
        VariantArticleSourceStatusKind::Unavailable => "unavailable",
        VariantArticleSourceStatusKind::Skipped => "skipped",
        VariantArticleSourceStatusKind::NotAttempted => "not_attempted",
    }
}

fn provider_statuses_for_route(
    route: &str,
    events: &[VariantArticleCallEvent],
) -> Vec<VariantArticleSourceStatus> {
    let mut grouped = BTreeMap::<String, Vec<&VariantArticleCallEvent>>::new();
    for event in events.iter().filter(|event| event.route == route) {
        grouped.entry(event.source.clone()).or_default().push(event);
    }
    grouped
        .into_iter()
        .map(|(source, events)| status(route, &source, provider_terminal_status(&events)))
        .collect()
}

fn route_stop_detail(route_stopped: bool) -> Option<&'static str> {
    route_stopped.then_some("internal work or configuration stopped before a provider call")
}

fn candidate_with_provenance(
    row: ArticleSearchResult,
    route: &str,
    source: &str,
    query_aliases: Vec<String>,
) -> ArticleCandidate {
    let native_position = row.source_local_position.saturating_add(1);
    let mut candidate = article_candidate_from_row(row);
    candidate.variant_provenance.push(VariantArticleProvenance {
        route: route.to_string(),
        source: source.to_string(),
        matched_alias: query_aliases.first().cloned(),
        query_aliases,
        native_position,
    });
    candidate
        .candidate_trace
        .push(VariantArticleCandidateTraceRecord {
            identifier: stable_article_identifier(&candidate.row),
            route: route.to_string(),
            provider_terminal_state: "received".into(),
            received: true,
            after_union: false,
            after_dedup: false,
            rank_position: None,
            verification_disposition: "not_requested".into(),
            pagination_disposition: "not_visible".into(),
        });
    candidate
}

fn canonical_equivalence_queries(requested: &RequestedVariantIdentity) -> Vec<(String, String)> {
    let mut queries = Vec::new();
    if let (Some(transcript), Some(coding)) = (
        requested.transcript.as_deref(),
        requested.coding_change.as_deref(),
    ) {
        let query = format!("{transcript}:{coding}");
        if crate::entities::variant::validate_car_hgvs_input(&query).is_ok() {
            queries.push(("transcript_coding".to_string(), query));
        }
    }
    if requested.is_authoritative_refseq() {
        let query = format!(
            "{}:g.{}{}>{}",
            requested.genomic_accession.as_deref().unwrap_or_default(),
            requested.position.unwrap_or_default(),
            requested.reference.as_deref().unwrap_or_default(),
            requested.alternate.as_deref().unwrap_or_default(),
        );
        if crate::entities::variant::validate_car_hgvs_input(&query).is_ok() {
            queries.push(("genomic".to_string(), query));
        }
    }
    queries.dedup();
    queries
}

fn canonical_observation(
    basis: String,
    query: String,
    item: CarNormalizationItem,
) -> CanonicalEquivalenceObservation {
    let comparison_complete = matches!(
        item.status,
        CarNormalizationStatus::Resolved | CarNormalizationStatus::NotFound
    );
    CanonicalEquivalenceObservation {
        basis,
        query,
        status: match item.status {
            CarNormalizationStatus::Resolved => "resolved",
            CarNormalizationStatus::NotFound => "not_found",
            CarNormalizationStatus::Invalid => "invalid",
            CarNormalizationStatus::Indeterminate => "indeterminate",
            CarNormalizationStatus::Unavailable => "unavailable",
        }
        .into(),
        caid: item.caid,
        provider_exhaustive: item.exhaustive,
        comparison_complete,
        source: item.source,
        request_template_version: item.provenance.request_template_version,
        car_version: item.provenance.car_version,
        provider_response_sha256: item.provenance.response_sha256,
    }
}

fn canonical_equivalence(
    observations: Vec<CanonicalEquivalenceObservation>,
    aliases: Vec<String>,
) -> CanonicalEquivalence {
    let count = observations.len();
    if count == 0 {
        return CanonicalEquivalence {
            status: "inapplicable".into(),
            caid: None,
            exhaustive: true,
            complete: true,
            applicable_identity_count: 0,
            observations,
            aliases: Vec::new(),
            message: "no independently supplied CAR HGVS identities".into(),
        };
    }
    if count == 1 {
        let observation = &observations[0];
        return CanonicalEquivalence {
            status: "single_identity".into(),
            caid: observation.caid.clone(),
            exhaustive: observation.provider_exhaustive,
            complete: observation.comparison_complete,
            applicable_identity_count: count,
            observations,
            // One identity cannot establish equivalence, but its CAR aliases are
            // still the requester's own identity and downstream verification
            // matches provider strings against them. Dropping them here left the
            // LDH ladder -- which explicitly accepts a lone resolved identity --
            // comparing every annotation against an empty alias list, so it
            // could never confirm one outside a two-identity fixture.
            aliases,
            message: "one independently supplied CAR HGVS identity cannot establish equivalence"
                .into(),
        };
    }
    let caids = observations
        .iter()
        .filter_map(|observation| observation.caid.clone())
        .collect::<BTreeSet<_>>();
    let provider_exhaustive = observations
        .iter()
        .all(|observation| observation.provider_exhaustive);
    let complete = observations
        .iter()
        .all(|observation| observation.comparison_complete);
    let statuses = observations
        .iter()
        .map(|observation| observation.status.as_str())
        .collect::<BTreeSet<_>>();
    let (status, caid, message) = if caids.len() >= 2 {
        (
            "contradictory",
            None,
            "independently supplied CAR identities resolved to different CAids",
        )
    } else if statuses.contains("indeterminate") || statuses.contains("invalid") {
        (
            "indeterminate",
            None,
            "CAR could not complete the identity comparison",
        )
    } else if statuses.contains("unavailable") {
        (
            "unavailable",
            None,
            "CAR was unavailable before the identity comparison completed",
        )
    } else if !provider_exhaustive {
        (
            "indeterminate",
            None,
            "CAR did not exhaustively resolve every supplied identity",
        )
    } else if caids.is_empty() {
        (
            "not_found",
            None,
            "CAR found no canonical allele for the supplied identities",
        )
    } else if statuses.contains("not_found") {
        (
            "indeterminate",
            None,
            "one supplied identity resolved while another was not found",
        )
    } else {
        (
            "confirmed",
            caids.first().cloned(),
            "all independently supplied CAR identities resolved to one CAid",
        )
    };
    CanonicalEquivalence {
        status: status.into(),
        caid,
        exhaustive: provider_exhaustive
            && !statuses.contains("invalid")
            && !statuses.contains("indeterminate")
            && !statuses.contains("unavailable"),
        complete,
        applicable_identity_count: count,
        observations,
        aliases: if status == "confirmed" || count == 1 {
            aliases
        } else {
            Vec::new()
        },
        message: message.into(),
    }
}

async fn resolve_canonical_equivalence(
    requested: &RequestedVariantIdentity,
    execution: &VariantArticleExecutionContext,
) -> CanonicalEquivalence {
    let queries = canonical_equivalence_queries(requested);
    let client = crate::sources::clingen_allele_registry::ClinGenAlleleRegistryClient::new().ok();
    let mut observations = Vec::with_capacity(queries.len());
    let mut items = Vec::new();
    for (basis, query) in queries {
        let item = match (execution.reserve("canonical_equivalence"), client.as_ref()) {
            (Some(started), Some(client)) => {
                let item =
                    client
                        .normalize(&query)
                        .await
                        .unwrap_or_else(|_| CarNormalizationItem {
                            input: query.clone(),
                            status: CarNormalizationStatus::Unavailable,
                            exhaustive: false,
                            caid: None,
                            canonical_title: None,
                            genomic_aliases: Default::default(),
                            transcript_aliases: Default::default(),
                            protein_aliases: Default::default(),
                            external_ids: Default::default(),
                            source: "clingen_car".into(),
                            query: query.clone(),
                            warnings: Vec::new(),
                            error: None,
                            provenance: crate::entities::variant::CarProvenance {
                                request_template_version: "1".into(),
                                car_version: None,
                                response_sha256: None,
                            },
                        });
                execution.record(
                    "canonical_equivalence",
                    "clingen_car",
                    started,
                    if matches!(item.status, CarNormalizationStatus::Unavailable) {
                        "unavailable"
                    } else {
                        "ok"
                    },
                    1,
                );
                item
            }
            _ => CarNormalizationItem {
                input: query.clone(),
                status: CarNormalizationStatus::Unavailable,
                exhaustive: false,
                caid: None,
                canonical_title: None,
                genomic_aliases: Default::default(),
                transcript_aliases: Default::default(),
                protein_aliases: Default::default(),
                external_ids: Default::default(),
                source: "clingen_car".into(),
                query: query.clone(),
                warnings: Vec::new(),
                error: None,
                provenance: crate::entities::variant::CarProvenance {
                    request_template_version: "1".into(),
                    car_version: None,
                    response_sha256: None,
                },
            },
        };
        items.push(item.clone());
        observations.push(canonical_observation(basis, query, item));
    }
    let mut aliases = Vec::new();
    for item in &items {
        aliases.extend(item.transcript_aliases.values.iter().cloned());
    }
    for item in &items {
        aliases.extend(item.protein_aliases.values.iter().cloned());
    }
    for item in &items {
        aliases.extend(item.genomic_aliases.values.iter().cloned());
    }
    for item in &items {
        aliases.extend(item.external_ids.values.iter().cloned());
    }
    let mut seen = BTreeSet::new();
    aliases.retain(|alias| seen.insert(alias.clone()));
    canonical_equivalence(observations, aliases)
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
        if let (Some(gene), Some(protein)) = (
            context.requested.gene.as_deref(),
            context
                .requested
                .protein_change
                .as_deref()
                .and_then(crate::entities::variant::normalize_protein_change),
        ) {
            aliases.insert(format!("{gene} {protein}"));
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
    for change in &context.resolution.normalized_aliases.protein_changes {
        insert_change(change);
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

fn exact_aliases_with_equivalence(
    context: &VariantArticleResolutionContext,
    equivalence: &CanonicalEquivalence,
) -> (Vec<String>, bool) {
    let (mut aliases, truncated) = exact_aliases(context);
    if context.requested.is_authoritative_refseq()
        || equivalence.status != "confirmed"
        || aliases.len() >= MAX_EXACT_ALIASES
    {
        return (aliases, truncated);
    }
    for alias in &equivalence.aliases {
        if !aliases.contains(alias) && aliases.len() < MAX_EXACT_ALIASES {
            aliases.push(alias.clone());
        }
    }
    let aliases_truncated = equivalence
        .aliases
        .iter()
        .any(|alias| !aliases.contains(alias));
    (aliases, truncated || aliases_truncated)
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
) -> Result<(Vec<ArticleCandidate>, bool, bool, bool), BioMcpError> {
    let Some(started) = execution.reserve("pubtator_variant") else {
        return Ok((Vec::new(), true, false, true));
    };
    let pubtator = match crate::sources::pubtator::PubTatorClient::new() {
        Ok(client) => client,
        Err(_) => return Ok((Vec::new(), true, false, true)),
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
            None,
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
        for row in page.results {
            if candidates.len() >= MAX_FEDERATED_FETCH_RESULTS {
                return Ok((candidates, true, succeeded, false));
            }
            candidates.push(candidate_with_provenance(
                row,
                "pubtator_variant",
                "pubtator",
                vec![if context.requested.is_authoritative_refseq() {
                    input.trim().to_string()
                } else {
                    token.matched_alias.clone()
                }],
            ));
        }
    }
    Ok((candidates, incomplete, succeeded, false))
}

async fn federated_alias_candidates(
    aliases: Vec<String>,
    _alias_budget_stopped: bool,
    route: &str,
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
    let mut incomplete = false;
    let mut succeeded = false;
    let mut alias_failed = false;
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
        for row in federated.rows {
            if candidates.len() >= MAX_FEDERATED_FETCH_RESULTS {
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
                    matched_alias: Some(alias.clone()),
                    query_aliases: vec![alias.clone()],
                    native_position: candidate.row.source_local_position.saturating_add(1),
                });
                candidate
                    .candidate_trace
                    .push(VariantArticleCandidateTraceRecord {
                        identifier: stable_article_identifier(&candidate.row),
                        route: route.to_string(),
                        provider_terminal_state: "received".into(),
                        received: true,
                        after_union: false,
                        after_dedup: false,
                        rank_position: None,
                        verification_disposition: "not_requested".into(),
                        pagination_disposition: "not_visible".into(),
                    });
            }
            candidates.push(candidate);
        }
    }
    incomplete |= alias_failed || !succeeded;
    let calls = execution.events();
    incomplete |= calls
        .iter()
        .any(|event| event.route == route && event.status != "ok");
    let mut statuses = provider_statuses_for_route(route, &calls);
    if let Some(detail) = route_stop_detail(execution.route_stopped(route)) {
        statuses.push(status_with_detail(
            route,
            "internal",
            VariantArticleSourceStatusKind::NotAttempted,
            Some(detail),
        ));
    }
    (candidates, incomplete, succeeded, statuses)
}

async fn strict_provider_candidates(
    input: &str,
    context: &VariantArticleResolutionContext,
    strategy: VariantArticleStrategy,
    exact_aliases: &[String],
    execution: &VariantArticleExecutionContext,
) -> (
    Vec<ArticleCandidate>,
    bool,
    bool,
    Vec<VariantArticleSourceStatus>,
) {
    let filters = article_filters();
    let plans = provider_variant_query_plan_with_aliases(input, context, strategy, exact_aliases);
    let mut candidates = Vec::new();
    let mut incomplete = false;
    let mut succeeded = false;

    for plan in plans.into_iter().filter(|plan| plan.route == "strict") {
        if execution.route_stopped("strict") {
            break;
        }
        let stopped_before = execution.route_stopped("strict");
        let rows = match plan.provider.as_str() {
            "pubmed" => search_pubmed_page_with_context(
                &filters,
                LEXICAL_ALIAS_FETCH_LIMIT,
                0,
                Some(execution),
                "strict",
                Some(&plan.query),
            )
            .await
            .map(|page| page.results),
            "europepmc" => search_europepmc_page_with_context(
                &filters,
                LEXICAL_ALIAS_FETCH_LIMIT,
                0,
                Some(execution),
                "strict",
                Some(&plan.query),
            )
            .await
            .map(|page| page.results),
            "semanticscholar" => search_semantic_scholar_candidates(
                &filters,
                LEXICAL_ALIAS_FETCH_LIMIT,
                Some(execution),
                "strict",
                Some(&plan.query),
            )
            .await
            .and_then(|outcome| {
                if matches!(
                    outcome.status.status,
                    Some(ArticleSourceAvailability::Unavailable)
                ) {
                    Err(BioMcpError::Api {
                        api: "semantic-scholar".into(),
                        message: "strict search unavailable".into(),
                    })
                } else {
                    Ok(outcome.rows)
                }
            }),
            "pubtator" => {
                let Some(started) = execution.reserve("strict") else {
                    incomplete = true;
                    break;
                };
                let pubtator = crate::sources::pubtator::PubTatorClient::new();
                let tokens = match pubtator {
                    Ok(pubtator) => {
                        resolve_variant_entity_tokens(&pubtator, input, &context.requested).await
                    }
                    Err(error) => Err(error),
                };
                execution.record(
                    "strict",
                    "pubtator",
                    started,
                    if tokens.is_ok() { "ok" } else { "unavailable" },
                    usize::from(tokens.is_ok()),
                );
                match tokens {
                    Ok(tokens) => match tokens.first() {
                        Some(token) => {
                            execution
                                .record_strict_pubtator_query(&plan.query_alias, &token.entity_id);
                            search_pubtator_page_with_context(
                                &filters,
                                LEXICAL_ALIAS_FETCH_LIMIT,
                                0,
                                Some(execution),
                                "strict",
                                Some(&token.entity_id),
                            )
                        }
                        .await
                        .map(|page| page.results),
                        None => Ok(Vec::new()),
                    },
                    Err(error) => Err(error),
                }
            }
            _ => continue,
        };
        match rows {
            Ok(rows) if !execution.route_stopped("strict") || stopped_before => {
                succeeded = true;
                candidates.extend(rows.into_iter().map(|row| {
                    candidate_with_provenance(
                        row,
                        "strict",
                        &plan.provider,
                        vec![plan.query_alias.clone()],
                    )
                }));
            }
            Ok(_) | Err(_) if execution.route_stopped("strict") => {
                incomplete = true;
                break;
            }
            Ok(_) | Err(_) => {
                incomplete = true;
            }
        }
    }
    let calls = execution.events();
    let mut statuses = provider_statuses_for_route("strict", &calls);
    if let Some(detail) = route_stop_detail(execution.route_stopped("strict")) {
        statuses.push(status_with_detail(
            "strict",
            "internal",
            VariantArticleSourceStatusKind::NotAttempted,
            Some(detail),
        ));
    }
    (candidates, incomplete, succeeded, statuses)
}

async fn lexical_candidates(
    aliases: Vec<String>,
    alias_budget_stopped: bool,
    execution: &VariantArticleExecutionContext,
) -> (
    Vec<ArticleCandidate>,
    bool,
    bool,
    Vec<VariantArticleSourceStatus>,
) {
    federated_alias_candidates(aliases, alias_budget_stopped, "exact_lexical", execution).await
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

fn select_hydrated_source_hit(
    hits: Vec<crate::sources::myvariant::MyVariantHit>,
    source_key: Option<&str>,
) -> Result<crate::sources::myvariant::MyVariantHit, BioMcpError> {
    hits.into_iter()
        .filter(|hit| {
            source_key.is_none_or(|key| {
                crate::entities::variant::SourceVariantIdentity::from_myvariant_hit(hit)
                    .normalized_key()
                    == key
            })
        })
        .min_by_key(|hit| serde_json::to_string(hit).unwrap_or_default())
        .ok_or_else(|| BioMcpError::SourceUnavailable {
            source_name: "MyVariant".into(),
            reason: "the confirmed variant record was absent during citation hydration".into(),
            suggestion: "Retry the variant article request".into(),
        })
}

async fn citation_candidates(
    context: &VariantArticleResolutionContext,
    execution: &VariantArticleExecutionContext,
) -> Result<(Vec<ArticleCandidate>, bool), BioMcpError> {
    let Some(retained_hit) = context.source_hit.as_ref() else {
        return Ok((Vec::new(), false));
    };
    let hydrated_hit = if retained_hit.civic.is_none() {
        let Some(started) = execution.reserve("source_citation") else {
            return Ok((Vec::new(), true));
        };
        let source_key = context
            .source_identity
            .as_ref()
            .map(crate::entities::variant::SourceVariantIdentity::normalized_key);
        let client = match crate::sources::myvariant::MyVariantClient::new() {
            Ok(client) => client,
            Err(_) => return Ok((Vec::new(), true)),
        };
        let result = client
            .get_all(&retained_hit.id)
            .await
            .and_then(|hits| select_hydrated_source_hit(hits, source_key.as_deref()));
        execution.record(
            "source_citation",
            "myvariant",
            started,
            if result.is_ok() { "ok" } else { "unavailable" },
            usize::from(result.is_ok()),
        );
        Some(result?)
    } else {
        None
    };
    let hit = hydrated_hit.as_ref().unwrap_or(retained_hit);
    let query_aliases: Vec<String> = primary_exact_alias(context).into_iter().collect();
    Ok((
        crate::sources::myvariant::civic_pubmed_ids(hit)
            .into_iter()
            .enumerate()
            .map(|(position, pmid)| {
                let mut row = pmid_seed(pmid);
                row.source_local_position = position;
                candidate_with_provenance(row, "source_citation", "civic", query_aliases.clone())
            })
            .collect(),
        false,
    ))
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
    exact_aliases: &[String],
) -> Vec<String> {
    match route {
        "strict" | "exact_lexical" => exact_aliases.to_vec(),
        "best_effort_free_text" => fallback_aliases(input, context).0,
        "source_citation" => context
            .source_id
            .clone()
            .or_else(|| Some(input.to_string()))
            .into_iter()
            .collect(),
        "pubtator_variant" if context.requested.is_authoritative_refseq() => {
            vec![input.trim().to_string()]
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

fn provider_variant_query_plan_with_aliases(
    input: &str,
    context: &VariantArticleResolutionContext,
    strategy: VariantArticleStrategy,
    exact_aliases: &[String],
) -> Vec<ProviderVariantQueryPlan> {
    let gene = context
        .requested
        .gene
        .as_deref()
        .or_else(|| {
            context
                .source_identity
                .as_ref()
                .and_then(|identity| identity.genes.first().map(String::as_str))
        })
        .unwrap_or_default();
    let aliases = exact_aliases.to_vec();
    let mut queries = Vec::new();
    if strategy != VariantArticleStrategy::Annotation && !gene.is_empty() {
        for alias in aliases {
            let alias = alias
                .strip_prefix(gene)
                .and_then(|remainder| remainder.strip_prefix(' '))
                .unwrap_or(&alias);
            let query_alias = format!("{gene} {alias}");
            queries.extend([
                ProviderVariantQueryPlan {
                    provider: "pubmed".into(),
                    route: "strict".into(),
                    query_alias: query_alias.clone(),
                    query: build_pubmed_variant_strict_query(gene, alias),
                    query_template_version: "pubmed-title-abstract-v1".into(),
                },
                ProviderVariantQueryPlan {
                    provider: "europepmc".into(),
                    route: "strict".into(),
                    query_alias: query_alias.clone(),
                    query: build_europepmc_variant_strict_query(gene, alias),
                    query_template_version: "europepmc-title-abstract-v1".into(),
                },
                ProviderVariantQueryPlan {
                    provider: "semanticscholar".into(),
                    route: "strict".into(),
                    query_alias: query_alias.clone(),
                    query: query_alias.clone(),
                    query_template_version: "semantic-scholar-bulk-phrase-v1".into(),
                },
                ProviderVariantQueryPlan {
                    provider: "pubtator".into(),
                    route: "strict".into(),
                    query_alias: query_alias.clone(),
                    query: format!("@VARIANT_{query_alias}"),
                    query_template_version: "pubtator-entity-v1".into(),
                },
            ]);
        }
    }
    if strategy == VariantArticleStrategy::Union {
        queries.push(ProviderVariantQueryPlan {
            provider: "federated".into(),
            route: "discovery".into(),
            query_alias: input.trim().to_string(),
            query: input.trim().to_string(),
            query_template_version: "federated-free-text-v1".into(),
        });
    }
    queries
}

struct VariantArticleDebugPlanState {
    counts: VariantArticleCountsPlan,
    truncated: bool,
    next: VariantArticleNextPlan,
    candidate_trace: Vec<VariantArticleCandidateTraceRecord>,
}

fn build_debug_plan(
    input: &str,
    context: &VariantArticleResolutionContext,
    strategy: VariantArticleStrategy,
    canonical_aliases: &[String],
    execution: &VariantArticleExecutionContext,
    state: VariantArticleDebugPlanState,
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
            VariantArticleStrategy::Union
                if !matches!(
                    context.resolution.provider_validation.status,
                    VariantProviderValidationStatus::Contradictory
                ) =>
            {
                vec!["strict", "best_effort_free_text"]
            }
            VariantArticleStrategy::Union => vec!["best_effort_free_text"],
            VariantArticleStrategy::Annotation => vec!["pubtator_variant"],
            VariantArticleStrategy::Lexical => vec!["exact_lexical"],
        }
    } else {
        match strategy {
            VariantArticleStrategy::Union => {
                vec![
                    "strict",
                    "pubtator_variant",
                    "exact_lexical",
                    "source_citation",
                ]
            }
            VariantArticleStrategy::Annotation => vec!["pubtator_variant"],
            VariantArticleStrategy::Lexical => vec!["strict", "exact_lexical"],
        }
    };
    let events = execution.events();
    let routes = route_names
        .into_iter()
        .map(|route| {
            let queries = plan_queries(input, context, route, canonical_aliases);
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
                        let status = provider_terminal_status(&events);
                        VariantArticleProviderPlan {
                            source,
                            status: provider_status_name(status).into(),
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
    let mut provider_queries =
        provider_variant_query_plan_with_aliases(input, context, strategy, canonical_aliases);
    for plan in &mut provider_queries {
        if plan.provider == "pubtator"
            && plan.route == "strict"
            && let Some(query) = execution.strict_pubtator_query(&plan.query_alias)
        {
            plan.query = query;
        }
    }
    VariantArticleDebugPlan {
        normalized_aliases: context.resolution.normalized_aliases.clone(),
        provider_queries,
        routes,
        counts: state.counts,
        ranking: VariantArticleRankingPlan {
            method: "exact route union with deterministic native-position ranking",
            inputs: ["exactness", "route_source_position", "stable_identifier"],
        },
        budgets: VariantArticleBudgetsPlan {
            item: item_work,
            request: request_work,
        },
        work_allocation: execution.work_allocation(),
        truncated: state.truncated,
        stopped_routes: execution.stopped_routes(),
        next: state.next,
        candidate_trace: VariantArticleCandidateTrace {
            schema_version: "variant-article-candidate-trace-v1",
            bounded: true,
            candidates: {
                let mut visible_identifiers = BTreeSet::new();
                let selected_indices = state
                    .candidate_trace
                    .iter()
                    .enumerate()
                    .filter(|(_, record)| {
                        record.received
                            && record.after_union
                            && record.after_dedup
                            && record.pagination_disposition == "visible"
                            && visible_identifiers.insert(&record.identifier)
                    })
                    .take(ITEM_WORK_LIMIT)
                    .map(|(index, _)| index)
                    .collect::<BTreeSet<_>>();
                let (selected, remaining): (Vec<_>, Vec<_>) = state
                    .candidate_trace
                    .into_iter()
                    .enumerate()
                    .partition(|(index, _)| selected_indices.contains(index));

                selected
                    .into_iter()
                    .chain(remaining)
                    .map(|(_, record)| record)
                    .take(ITEM_WORK_LIMIT)
                    .collect()
            },
        },
        verification: None,
    }
}

// dead-code reason: search_variant_articles preserves the article facade's default-verification entry point.
#[allow(dead_code)]
pub async fn search_variant_articles(
    input: &str,
    strategy: VariantArticleStrategy,
    limit: usize,
    offset: usize,
) -> Result<VariantArticleOutcome, BioMcpError> {
    search_variant_articles_with_plan(input, strategy, limit, offset, false).await
}

// dead-code reason: search_variant_articles_with_plan preserves the article facade's debug-plan entry point.
#[allow(dead_code)]
pub async fn search_variant_articles_with_plan(
    input: &str,
    strategy: VariantArticleStrategy,
    limit: usize,
    offset: usize,
    debug_plan: bool,
) -> Result<VariantArticleOutcome, BioMcpError> {
    search_variant_articles_with_options(
        input,
        strategy,
        limit,
        offset,
        debug_plan,
        VariantArticleVerificationOptions::default(),
    )
    .await
}

pub(crate) async fn search_variant_articles_with_options(
    input: &str,
    strategy: VariantArticleStrategy,
    limit: usize,
    offset: usize,
    debug_plan: bool,
    verification: VariantArticleVerificationOptions,
) -> Result<VariantArticleOutcome, BioMcpError> {
    let requested = RequestedVariantIdentity::from_variant_input(input)?;
    search_variant_articles_identity(
        input,
        requested,
        strategy,
        limit,
        offset,
        debug_plan,
        verification,
        VariantArticleExecutionContext::single(),
    )
    .await
}

#[allow(clippy::too_many_arguments)]
fn variant_article_terminal_state(
    failed_routes: usize,
    provider_incomplete: bool,
    verification_incomplete: bool,
    canonical_equivalence_complete: bool,
    offset: usize,
    has_more: bool,
    total_candidates: usize,
    returned_candidates: usize,
) -> (bool, bool) {
    let complete = failed_routes == 0
        && !provider_incomplete
        && !verification_incomplete
        && canonical_equivalence_complete;
    let truncated = !complete || offset > 0 || has_more || total_candidates != returned_candidates;
    (complete, truncated)
}

#[allow(clippy::too_many_arguments)]
async fn search_variant_articles_identity(
    input: &str,
    requested: RequestedVariantIdentity,
    strategy: VariantArticleStrategy,
    limit: usize,
    offset: usize,
    include_debug_plan: bool,
    verification: VariantArticleVerificationOptions,
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
    let canonical_equivalence = resolve_canonical_equivalence(&context.requested, &execution).await;
    let (selected_exact_aliases, selected_exact_aliases_truncated) =
        exact_aliases_with_equivalence(&context, &canonical_equivalence);
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
                canonical_equivalence: Some(canonical_equivalence),
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
                source_status: vec![status(
                    "resolution",
                    "myvariant",
                    VariantArticleSourceStatusKind::Unavailable,
                )],
                retrieval_path: "variant resolution unavailable",
                results: Vec::new(),
                debug_plan,
            },
            hard_error: true,
        });
    }
    if verification.verify_identity {
        // Keep one verification unit before discovery. Each discovery page then
        // expands this reservation for candidates that could occupy the visible page.
        execution.reserve_identity_verification(1);
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
                            VariantArticleSourceStatusKind::Skipped,
                            Some("provider identity contradicted request"),
                        ))
                    }
                    VariantProviderValidationStatus::NotFound => statuses.push(status_with_detail(
                        "source_citation",
                        "myvariant",
                        VariantArticleSourceStatusKind::Skipped,
                        Some("no compatible MyVariant record"),
                    )),
                    VariantProviderValidationStatus::Indeterminate => {
                        statuses.push(status_with_detail(
                            "source_citation",
                            "myvariant",
                            VariantArticleSourceStatusKind::Skipped,
                            Some("provider identity was not confirmable"),
                        ))
                    }
                    VariantProviderValidationStatus::Unavailable => {
                        statuses.push(status_with_detail(
                            "source_citation",
                            "myvariant",
                            VariantArticleSourceStatusKind::Skipped,
                            Some("provider validation unavailable"),
                        ))
                    }
                    VariantProviderValidationStatus::Confirmed => {}
                }
                if !matches!(
                    context.resolution.provider_validation.status,
                    VariantProviderValidationStatus::Contradictory
                ) {
                    let (rows, incomplete, succeeded, route_statuses) = strict_provider_candidates(
                        input,
                        &context,
                        strategy,
                        &selected_exact_aliases,
                        &execution,
                    )
                    .await;
                    candidates.extend(rows);
                    statuses.extend(route_statuses);
                    succeeded_routes += usize::from(succeeded);
                    failed_routes += usize::from(incomplete || !succeeded);
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
                        VariantArticleSourceStatusKind::Skipped,
                        Some("provider identity contradicted request"),
                    ));
                } else {
                    statuses.push(status(
                        "pubtator_variant",
                        "pubtator",
                        VariantArticleSourceStatusKind::Ok,
                    ));
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
                        VariantArticleSourceStatusKind::Skipped,
                        Some("provider identity contradicted request"),
                    ));
                } else {
                    statuses.push(status(
                        "exact_lexical",
                        "federated",
                        VariantArticleSourceStatusKind::Ok,
                    ));
                }
                succeeded_routes += 1;
            }
        }
    } else {
        if matches!(
            strategy,
            VariantArticleStrategy::Union | VariantArticleStrategy::Lexical
        ) {
            let (rows, incomplete, succeeded, route_statuses) = strict_provider_candidates(
                input,
                &context,
                strategy,
                &selected_exact_aliases,
                &execution,
            )
            .await;
            candidates.extend(rows);
            statuses.extend(route_statuses);
            succeeded_routes += usize::from(succeeded);
            failed_routes += usize::from(incomplete || !succeeded);
        }
        if matches!(
            strategy,
            VariantArticleStrategy::Union | VariantArticleStrategy::Annotation
        ) {
            match annotation_candidates(input, &context, &execution).await {
                Ok((rows, incomplete, succeeded, pre_call_stopped)) => {
                    candidates.extend(rows);
                    let calls = execution.events();
                    statuses.extend(provider_statuses_for_route("pubtator_variant", &calls));
                    if pre_call_stopped
                        && let Some(detail) =
                            route_stop_detail(execution.route_stopped("pubtator_variant"))
                    {
                        statuses.push(status_with_detail(
                            "pubtator_variant",
                            "internal",
                            VariantArticleSourceStatusKind::NotAttempted,
                            Some(detail),
                        ));
                    }
                    succeeded_routes += usize::from(succeeded);
                    failed_routes += usize::from(incomplete || !succeeded);
                }
                Err(_) => {
                    statuses.push(status(
                        "pubtator_variant",
                        "pubtator",
                        VariantArticleSourceStatusKind::Unavailable,
                    ));
                    failed_routes += 1;
                }
            }
        }
        if matches!(
            strategy,
            VariantArticleStrategy::Union | VariantArticleStrategy::Lexical
        ) {
            let (rows, incomplete, succeeded, route_statuses) = lexical_candidates(
                selected_exact_aliases.clone(),
                selected_exact_aliases_truncated,
                &execution,
            )
            .await;
            candidates.extend(rows);
            statuses.extend(route_statuses);
            succeeded_routes += usize::from(succeeded);
            failed_routes += usize::from(incomplete || !succeeded);
        }
        if strategy == VariantArticleStrategy::Union {
            match context.resolution.provider_validation.status {
                VariantProviderValidationStatus::Confirmed => {
                    match citation_candidates(&context, &execution).await {
                        Ok((rows, pre_call_stopped)) => {
                            candidates.extend(rows);
                            if pre_call_stopped {
                                if let Some(detail) =
                                    route_stop_detail(execution.route_stopped("source_citation"))
                                {
                                    statuses.push(status_with_detail(
                                        "source_citation",
                                        "internal",
                                        VariantArticleSourceStatusKind::NotAttempted,
                                        Some(detail),
                                    ));
                                }
                                failed_routes += 1;
                            } else {
                                statuses.push(status(
                                    "source_citation",
                                    "myvariant",
                                    VariantArticleSourceStatusKind::Ok,
                                ));
                                succeeded_routes += 1;
                            }
                        }
                        Err(_) => {
                            statuses.push(status(
                                "source_citation",
                                "myvariant",
                                VariantArticleSourceStatusKind::Unavailable,
                            ));
                            failed_routes += 1;
                        }
                    }
                }
                VariantProviderValidationStatus::NotFound => statuses.push(status_with_detail(
                    "source_citation",
                    "myvariant",
                    VariantArticleSourceStatusKind::Skipped,
                    Some("no compatible MyVariant record"),
                )),
                VariantProviderValidationStatus::Indeterminate => {
                    statuses.push(status_with_detail(
                        "source_citation",
                        "myvariant",
                        VariantArticleSourceStatusKind::Skipped,
                        Some("provider identity was not confirmable"),
                    ));
                    failed_routes += 1;
                }
                VariantProviderValidationStatus::Unavailable => {
                    statuses.push(status_with_detail(
                        "source_citation",
                        "myvariant",
                        VariantArticleSourceStatusKind::Skipped,
                        Some("provider validation unavailable"),
                    ));
                    failed_routes += 1;
                }
                VariantProviderValidationStatus::Contradictory => {}
            }
        }
    }

    let pre_dedup = candidates.len();
    for candidate in &mut candidates {
        for trace in &mut candidate.candidate_trace {
            trace.after_union = true;
        }
    }
    let mut candidates = merge_article_candidate_pool(candidates);
    for candidate in &mut candidates {
        for trace in &mut candidate.candidate_trace {
            trace.after_dedup = true;
        }
    }
    let mut filtered_candidate_trace = Vec::new();
    let mut verification_response_subsets = Vec::new();
    let mut verification_content_subsets = Vec::new();
    let mut verification_incomplete = false;
    if verification.verify_identity {
        rank_candidates(&mut candidates);
        let (verification_start, verification_count) = if verification.confirmed_only {
            (0, candidates.len().min(ITEM_WORK_LIMIT))
        } else {
            (offset, candidates.len().saturating_sub(offset).min(limit))
        };
        execution.reserve_identity_verification_through(verification_count);
        for candidate in candidates
            .iter_mut()
            .skip(verification_start)
            .take(verification_count)
        {
            let captured = verify_captured_abstract(
                &context.requested,
                candidate
                    .row
                    .abstract_snippet
                    .as_deref()
                    .unwrap_or(&candidate.row.normalized_abstract),
            );
            if captured.status != "unverified" {
                verification_content_subsets.push(canonical_content_subset(&captured));
                candidate.identity = Some(captured);
                continue;
            }
            let incomplete = match candidate.row.pmid.parse::<u32>() {
                Ok(pmid) => {
                    let Some(started) = execution.reserve("identity_verification") else {
                        verification_incomplete = true;
                        candidate.identity = Some(combine_identities(
                            captured.clone(),
                            verify_pubtator(
                                &context.requested,
                                &pmid.to_string(),
                                &crate::sources::pubtator::PubTatorExportResponse {
                                    documents: Vec::new(),
                                },
                                true,
                            ),
                        ));
                        continue;
                    };
                    let response = match crate::sources::pubtator::PubTatorClient::new() {
                        Ok(client) => client.export_biocjson(pmid).await,
                        Err(error) => Err(error),
                    };
                    execution.record(
                        "identity_verification",
                        "pubtator",
                        started,
                        if response.is_ok() {
                            "ok"
                        } else {
                            "unavailable"
                        },
                        1,
                    );
                    match response {
                        Ok(response) => {
                            verification_response_subsets
                                .push(canonical_response_subset(&response));
                            let fetched = verify_pubtator(
                                &context.requested,
                                &pmid.to_string(),
                                &response,
                                false,
                            );
                            let identity = combine_identities(captured.clone(), fetched);
                            verification_content_subsets.push(canonical_content_subset(&identity));
                            candidate.identity = Some(identity);
                            false
                        }
                        Err(_) => true,
                    }
                }
                Err(_) => true,
            };
            verification_incomplete |= incomplete;
            if candidate.identity.is_none() {
                candidate.identity = Some(combine_identities(
                    captured.clone(),
                    verify_pubtator(
                        &context.requested,
                        &candidate.row.pmid,
                        &crate::sources::pubtator::PubTatorExportResponse {
                            documents: Vec::new(),
                        },
                        incomplete,
                    ),
                ));
            }
        }
        verification_incomplete |=
            verification.confirmed_only && candidates.len() > verification_count;
        let _ldh_incomplete = add_ldh_observations(
            &mut candidates,
            &context.requested,
            &canonical_equivalence,
            &execution,
        )
        .await;
        for candidate in &mut candidates {
            let disposition = candidate
                .identity
                .as_ref()
                .map_or("unverified", |identity| identity.status);
            for trace in &mut candidate.candidate_trace {
                trace.verification_disposition = disposition.into();
            }
        }
        if verification.confirmed_only {
            candidates.retain_mut(|candidate| {
                let confirmed = candidate
                    .identity
                    .as_ref()
                    .is_some_and(|identity| identity.status == "confirmed");
                if !confirmed {
                    for trace in &mut candidate.candidate_trace {
                        trace.verification_disposition = "filtered_confirmed_only".into();
                    }
                    filtered_candidate_trace.append(&mut candidate.candidate_trace);
                }
                confirmed
            });
        }
    }
    rank_candidates(&mut candidates);
    for (index, candidate) in candidates.iter_mut().enumerate() {
        for trace in &mut candidate.candidate_trace {
            trace.rank_position = Some(index.saturating_add(1));
            trace.pagination_disposition =
                if index >= offset && index < offset.saturating_add(limit) {
                    "visible".into()
                } else {
                    "not_visible".into()
                };
        }
    }
    let candidate_trace = candidates
        .iter()
        .flat_map(|candidate| candidate.candidate_trace.iter().cloned())
        .chain(filtered_candidate_trace)
        .collect();
    let total_candidates = candidates.len();
    let hard_error = succeeded_routes == 0 && failed_routes > 0;
    let has_more = offset.saturating_add(limit) < total_candidates;
    let mut visible_candidates = candidates
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect::<Vec<_>>();
    enrich_candidates(&mut visible_candidates, &execution).await;
    let provider_incomplete = matches!(
        context.resolution.provider_validation.status,
        VariantProviderValidationStatus::Indeterminate
            | VariantProviderValidationStatus::Unavailable
    );
    let (complete, truncated) = variant_article_terminal_state(
        failed_routes,
        provider_incomplete,
        verification_incomplete,
        canonical_equivalence.applicable_identity_count < 2 || canonical_equivalence.complete,
        offset,
        has_more,
        total_candidates,
        visible_candidates.len(),
    );
    let rows = visible_candidates
        .into_iter()
        .enumerate()
        .map(|(index, candidate)| {
            let matched_aliases = candidate
                .variant_provenance
                .iter()
                .flat_map(|fact| fact.query_aliases.iter().cloned())
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
                identity: candidate.identity,
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
        let mut plan = build_debug_plan(
            input,
            &context,
            strategy,
            &selected_exact_aliases,
            &execution,
            VariantArticleDebugPlanState {
                counts: VariantArticleCountsPlan {
                    pre_dedup,
                    post_dedup: total_candidates,
                    returned: rows.len(),
                },
                truncated,
                next: VariantArticleNextPlan {
                    offset: offset.saturating_add(rows.len()),
                    cursor: None,
                },
                candidate_trace,
            },
        );
        if verification.verify_identity {
            plan.verification = Some(verification_plan(
                &context.requested,
                PUBTATOR_EXPORT_TEMPLATE_VERSION,
                &verification_response_subsets,
                &verification_content_subsets,
            ));
        }
        plan
    });
    Ok(VariantArticleOutcome {
        response: VariantArticleResponse {
            requested_variant: context.requested,
            resolution: context.resolution,
            canonical_equivalence: Some(canonical_equivalence),
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

/// Providers disagree on how to spell a PMC identifier: Europe PMC and PubTator
/// return `PMC8710334`, Semantic Scholar returns the bare `8710334` for the same
/// article. ClinGen LDH keys its literature set on the prefixed form, so the join
/// has to happen on one spelling or it silently never matches.
fn canonical_pmcid(value: &str) -> Option<String> {
    let trimmed = value.trim();
    let digits = trimmed
        .strip_prefix("PMC")
        .or_else(|| trimmed.strip_prefix("pmc"))
        .unwrap_or(trimmed);
    (!digits.is_empty() && digits.chars().all(|character| character.is_ascii_digit()))
        .then(|| format!("PMC{digits}"))
}

async fn add_ldh_observations(
    candidates: &mut [ArticleCandidate],
    requested: &RequestedVariantIdentity,
    equivalence: &CanonicalEquivalence,
    execution: &VariantArticleExecutionContext,
) -> bool {
    let caid = if equivalence.applicable_identity_count == 1 {
        equivalence
            .observations
            .iter()
            .find(|observation| observation.status == "resolved" && observation.provider_exhaustive)
            .and_then(|observation| observation.caid.as_deref())
    } else if equivalence.applicable_identity_count >= 2 && equivalence.status == "confirmed" {
        equivalence.caid.as_deref()
    } else {
        None
    };
    let Some(caid) = caid else { return false };
    let Some(started) = execution.reserve("clingen_ldh_medium") else {
        return true;
    };
    let client = match crate::sources::ClinGenLdhClient::new() {
        Ok(client) => client,
        Err(_) => return true,
    };
    let medium = client.medium(caid).await;
    execution.record(
        "clingen_ldh_medium",
        "clingen_ldh",
        started,
        if medium.is_ok() { "ok" } else { "unavailable" },
        1,
    );
    let Ok(medium) = medium else { return true };
    let rows = medium
        .get("data")
        .and_then(|data| data.get("VariantsInLiterature"))
        .and_then(serde_json::Value::as_array);
    if medium
        .pointer("/status/code")
        .and_then(serde_json::Value::as_i64)
        != Some(200)
        || medium.get("metadata").is_none()
        || rows.is_none()
    {
        return true;
    }
    let rows = rows.unwrap();
    let mut incomplete = false;
    let mut direct_bytes = 0;
    for candidate in candidates
        .iter_mut()
        .filter(|candidate| {
            candidate
                .row
                .pmcid
                .as_deref()
                .and_then(canonical_pmcid)
                .is_some_and(|pmcid| {
                    rows.iter().any(|row| {
                        row.get("entId").and_then(serde_json::Value::as_str) == Some(pmcid.as_str())
                    })
                })
        })
        .take(5)
    {
        let Some(pmcid) = candidate.row.pmcid.as_deref().and_then(canonical_pmcid) else {
            continue;
        };
        let pmcid = pmcid.as_str();
        let iris = rows
            .iter()
            .filter_map(|row| {
                let id = row.get("entId").and_then(serde_json::Value::as_str)?;
                let iri = row.get("entIri").and_then(serde_json::Value::as_str)?;
                (id == pmcid
                    && row
                        .get("entDisposition")
                        .and_then(serde_json::Value::as_str)
                        == Some("external")
                    && row.get("entType").and_then(serde_json::Value::as_str)
                        == Some("VariantsInLiterature")
                    && ldh_iri_matches(iri, id))
                .then_some(iri)
            })
            .take(2)
            .collect::<Vec<_>>();
        for iri in iris {
            let Some(started) = execution.reserve("clingen_ldh_direct") else {
                return true;
            };
            let Some(body_limit) =
                crate::sources::clingen_ldh::remaining_direct_body_limit(direct_bytes)
            else {
                return true;
            };
            let direct = client.direct(iri, body_limit).await;
            execution.record(
                "clingen_ldh_direct",
                "clingen_ldh",
                started,
                if direct.is_ok() { "ok" } else { "unavailable" },
                1,
            );
            match direct {
                Ok((direct, body_bytes)) => {
                    direct_bytes += body_bytes;
                    let ldh = verify_ldh_annotation(
                        requested,
                        caid,
                        &equivalence.aliases,
                        pmcid,
                        iri,
                        &direct,
                    );
                    incomplete |= ldh.incomplete;
                    let existing = candidate.identity.take().unwrap_or_else(|| {
                        verify_captured_abstract(
                            requested,
                            candidate
                                .row
                                .abstract_snippet
                                .as_deref()
                                .unwrap_or(&candidate.row.normalized_abstract),
                        )
                    });
                    candidate.identity = Some(combine_identities(existing, ldh));
                }
                Err(_) => incomplete = true,
            }
        }
    }
    incomplete
}

fn ldh_iri_matches(iri: &str, pmcid: &str) -> bool {
    let Ok(url) = reqwest::Url::parse(iri) else {
        return false;
    };
    url.scheme() == "https"
        && url.host_str() == Some("ldh.genome.network")
        && url.username().is_empty()
        && url.password().is_none()
        && url.query().is_none()
        && url.fragment().is_none()
        && url.path() == format!("/ldh/dss/cg/ns/ldh/set/variants_in_literature/id/{pmcid}/data")
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
    let item_work =
        VariantArticleWork::new(ITEM_WORK_LIMIT + EXACT_WORK_LIMIT + ITEM_WORK_LIMIT, 0);
    VariantArticleDebugPlan {
        normalized_aliases: requested.normalized_aliases(),
        provider_queries: Vec::new(),
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
            item: item_work,
            request: VariantArticleWork::new(ITEM_WORK_LIMIT, 0),
        },
        work_allocation: VariantArticleWorkAllocationPlan {
            discovery: VariantArticleWork::new(ITEM_WORK_LIMIT, 0),
            exact_lexical: VariantArticleWorkAllocationScope {
                item: VariantArticleWork::new(EXACT_WORK_LIMIT, 0),
                request: VariantArticleWork::new(EXACT_WORK_LIMIT, 0),
            },
            identity_verification: VariantArticleIdentityVerificationAllocation {
                reserved: 0,
                consumed: 0,
                item: VariantArticleWork::new(ITEM_WORK_LIMIT, 0),
                request: VariantArticleWork::new(ITEM_WORK_LIMIT, 0),
            },
        },
        truncated,
        stopped_routes,
        next: VariantArticleNextPlan {
            offset,
            cursor: None,
        },
        candidate_trace: VariantArticleCandidateTrace {
            schema_version: "variant-article-candidate-trace-v1",
            bounded: true,
            candidates: Vec::new(),
        },
        verification: None,
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
        identity: row.identity,
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
        canonical_equivalence: None,
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
    verification: VariantArticleVerificationOptions,
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
        verification,
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
        canonical_equivalence: response.canonical_equivalence,
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

// dead-code reason: search_variant_article_batch preserves the batch facade's default-verification entry point.
#[allow(dead_code)]
pub async fn search_variant_article_batch(
    requests: Vec<crate::entities::variant::VariantArticleRequest>,
    strategy: VariantArticleStrategy,
    limit: usize,
    offset: usize,
    debug_plan: bool,
) -> Result<VariantArticleBatchOutcome, BioMcpError> {
    search_variant_article_batch_with_options(
        requests,
        strategy,
        limit,
        offset,
        debug_plan,
        VariantArticleVerificationOptions::default(),
    )
    .await
}

pub(crate) async fn search_variant_article_batch_with_options(
    requests: Vec<crate::entities::variant::VariantArticleRequest>,
    strategy: VariantArticleStrategy,
    limit: usize,
    offset: usize,
    debug_plan: bool,
    verification: VariantArticleVerificationOptions,
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
            execute_batch_item(
                request,
                strategy,
                limit,
                offset,
                debug_plan,
                verification,
                execution,
            )
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
                query_aliases: vec![format!("alias-{position}")],
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
                "BRAF V600E",
                "BRAF c.1799T>A",
                "BRAF p.Val600Glu",
                "chr7:g.140453136A>T",
                "rs113488022",
            ]
        );
    }

    #[test]
    fn strict_provider_queries_keep_coding_collisions_distinct() {
        let mut left = resolved_context();
        left.requested.gene = Some("BRCA1".into());
        left.requested.protein_change = None;
        left.requested.coding_change = Some("c.788G>T".into());
        left.resolution.normalized_aliases = NormalizedVariantAliases::default();
        let mut right = left.clone();
        right.requested.coding_change = Some("c.2428A>T".into());

        let left_query = provider_variant_query_plan_with_aliases(
            "BRCA1 c.788G>T",
            &left,
            VariantArticleStrategy::Union,
            &exact_aliases(&left).0,
        );
        let right_query = provider_variant_query_plan_with_aliases(
            "BRCA1 c.2428A>T",
            &right,
            VariantArticleStrategy::Union,
            &exact_aliases(&right).0,
        );

        assert_eq!(left_query.len(), 25);
        assert_eq!(
            left_query[0].query_template_version,
            "pubmed-title-abstract-v1"
        );
        assert!(
            left_query
                .iter()
                .any(|query| query.query.contains("c.788G>T"))
        );
        assert!(
            right_query
                .iter()
                .any(|query| query.query.contains("c.2428A>T"))
        );
        let discovery = left_query
            .iter()
            .position(|query| query.route == "discovery")
            .expect("union retains discovery");
        assert!(
            left_query[..discovery]
                .iter()
                .all(|query| query.route == "strict")
        );
    }

    #[test]
    fn debug_plan_reports_strict_route_for_resolved_and_ambiguous_union_requests() {
        let execution = VariantArticleExecutionContext::single();
        let counts = VariantArticleCountsPlan {
            pre_dedup: 0,
            post_dedup: 0,
            returned: 0,
        };
        let next = VariantArticleNextPlan {
            offset: 0,
            cursor: None,
        };
        let resolved = resolved_context();
        let mut ambiguous = resolved.clone();
        ambiguous.resolution.status = VariantResolutionStatus::Ambiguous;
        ambiguous.resolution.provider_validation.status =
            VariantProviderValidationStatus::Indeterminate;

        for context in [&resolved, &ambiguous] {
            let plan = build_debug_plan(
                "BRAF p.V600E",
                context,
                VariantArticleStrategy::Union,
                &[],
                &execution,
                VariantArticleDebugPlanState {
                    counts: counts.clone(),
                    truncated: false,
                    next: next.clone(),
                    candidate_trace: Vec::new(),
                },
            );
            assert!(plan.routes.iter().any(|route| route.route == "strict"));
        }
    }

    #[test]
    fn candidate_trace_is_capped_and_serializes_only_stage_facts() {
        let plan = build_debug_plan(
            "BRAF p.V600E",
            &resolved_context(),
            VariantArticleStrategy::Union,
            &[],
            &VariantArticleExecutionContext::single(),
            VariantArticleDebugPlanState {
                counts: VariantArticleCountsPlan {
                    pre_dedup: 0,
                    post_dedup: 0,
                    returned: 0,
                },
                truncated: false,
                next: VariantArticleNextPlan {
                    offset: 0,
                    cursor: None,
                },
                candidate_trace: (0..=ITEM_WORK_LIMIT)
                    .map(|index| VariantArticleCandidateTraceRecord {
                        identifier: index.to_string(),
                        route: "strict".into(),
                        provider_terminal_state: "received".into(),
                        received: true,
                        after_union: true,
                        after_dedup: true,
                        rank_position: Some(index.saturating_add(1)),
                        verification_disposition: "confirmed".into(),
                        pagination_disposition: "visible".into(),
                    })
                    .collect(),
            },
        );

        assert_eq!(plan.candidate_trace.candidates.len(), ITEM_WORK_LIMIT);
        assert_eq!(
            serde_json::to_value(&plan.candidate_trace.candidates[0])
                .expect("candidate trace serializes")
                .as_object()
                .expect("candidate trace is an object")
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            vec![
                "after_dedup",
                "after_union",
                "identifier",
                "pagination_disposition",
                "provider_terminal_state",
                "rank_position",
                "received",
                "route",
                "verification_disposition",
            ]
        );
    }

    #[test]
    fn visible_candidate_trace_retains_returned_pmid_when_prefix_is_full() {
        let plan = build_debug_plan(
            "ATM p.C2464R",
            &resolved_context(),
            VariantArticleStrategy::Union,
            &[],
            &VariantArticleExecutionContext::single(),
            VariantArticleDebugPlanState {
                counts: VariantArticleCountsPlan {
                    pre_dedup: ITEM_WORK_LIMIT.saturating_add(1),
                    post_dedup: ITEM_WORK_LIMIT.saturating_add(1),
                    returned: 1,
                },
                truncated: true,
                next: VariantArticleNextPlan {
                    offset: 1,
                    cursor: None,
                },
                candidate_trace: (0..ITEM_WORK_LIMIT)
                    .map(|index| VariantArticleCandidateTraceRecord {
                        identifier: format!("earlier-{index}"),
                        route: "strict".into(),
                        provider_terminal_state: "received".into(),
                        received: true,
                        after_union: true,
                        after_dedup: true,
                        rank_position: Some(index.saturating_add(2)),
                        verification_disposition: "not_requested".into(),
                        pagination_disposition: "not_visible".into(),
                    })
                    .chain(std::iter::once(VariantArticleCandidateTraceRecord {
                        identifier: "11805335".into(),
                        route: "exact_lexical".into(),
                        provider_terminal_state: "received".into(),
                        received: true,
                        after_union: true,
                        after_dedup: true,
                        rank_position: Some(1),
                        verification_disposition: "not_requested".into(),
                        pagination_disposition: "visible".into(),
                    }))
                    .collect(),
            },
        );

        assert!(plan.candidate_trace.bounded);
        assert_eq!(plan.candidate_trace.candidates.len(), ITEM_WORK_LIMIT);
        assert!(
            plan.candidate_trace.candidates.iter().any(|trace| {
                trace.identifier == "11805335"
                    && trace.received
                    && trace.after_union
                    && trace.after_dedup
                    && trace.pagination_disposition == "visible"
            }),
            "every returned article must retain a visible route receipt"
        );
    }

    #[test]
    fn missing_confirmed_record_during_citation_hydration_is_unavailable() {
        let error = select_hydrated_source_hit(Vec::new(), Some("missing"))
            .expect_err("a stale confirmed record must not become healthy empty coverage");
        assert!(matches!(error, BioMcpError::SourceUnavailable { .. }));
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
            plan_queries(
                "NC_000011.10:g.108248927T>G",
                &context,
                "pubtator_variant",
                &[],
            ),
            vec!["NC_000011.10:g.108248927T>G"]
        );

        context.resolution.status = VariantResolutionStatus::Unresolved;
        context.resolution.basis = None;
        context.resolution.provider_validation.status =
            VariantProviderValidationStatus::Contradictory;
        let plan = build_debug_plan(
            "NC_000011.10:g.108248927T>G",
            &context,
            VariantArticleStrategy::Annotation,
            &[],
            &VariantArticleExecutionContext::single(),
            VariantArticleDebugPlanState {
                counts: VariantArticleCountsPlan {
                    pre_dedup: 0,
                    post_dedup: 0,
                    returned: 0,
                },
                truncated: false,
                next: VariantArticleNextPlan {
                    offset: 0,
                    cursor: None,
                },
                candidate_trace: Vec::new(),
            },
        );
        assert!(plan.routes.is_empty());
    }

    #[test]
    fn authoritative_refseq_strict_plans_include_only_known_supplied_proteins() {
        let mut context = resolved_context();
        context.requested = RequestedVariantIdentity {
            gene: Some("ATM".into()),
            protein_change: Some("p.Met16Ile".into()),
            coding_change: Some("c.47G>A".into()),
            transcript: Some("NM_000051.4".into()),
            genomic_accession: Some("NC_000011.10".into()),
            genome_build: Some("GRCh38".into()),
            position: Some(108248927),
            reference: Some("T".into()),
            alternate: Some("G".into()),
            ..Default::default()
        };

        assert!(exact_aliases(&context).0.contains(&"ATM M16I".to_string()));
        assert!(
            provider_variant_query_plan_with_aliases(
                "ATM p.Met16Ile",
                &context,
                VariantArticleStrategy::Union,
                &exact_aliases(&context).0,
            )
            .iter()
            .any(|plan| plan.query_alias == "ATM M16I")
        );

        context.requested.protein_change = Some("p.?".into());
        assert!(
            !exact_aliases(&context)
                .0
                .iter()
                .any(|alias| alias.contains("p.?"))
        );
        assert!(
            !provider_variant_query_plan_with_aliases(
                "ATM p.?",
                &context,
                VariantArticleStrategy::Union,
                &exact_aliases(&context).0,
            )
            .iter()
            .any(|plan| plan.route == "strict" && plan.query_alias.contains("p.?"))
        );
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
        let reserved = VariantArticleExecutionContext::single();
        reserved.reserve_identity_verification(3);
        for _ in 0..ITEM_WORK_LIMIT - 3 {
            assert!(reserved.reserve("strict").is_some());
        }
        assert!(reserved.reserve("strict").is_none());
        for _ in 0..3 {
            assert!(reserved.reserve("identity_verification").is_some());
        }
        assert_eq!(reserved.work_allocation().discovery.limit, ITEM_WORK_LIMIT);
        assert_eq!(reserved.work_allocation().discovery.consumed, 47);
        assert_eq!(reserved.work_allocation().identity_verification.consumed, 3);
        assert_eq!(
            reserved
                .work_allocation()
                .identity_verification
                .item
                .consumed,
            3
        );

        let contexts = VariantArticleExecutionContext::batch(10);
        for context in &contexts {
            for _ in 0..ITEM_WORK_LIMIT {
                assert!(context.reserve("strict").is_some());
            }
            assert!(context.reserve("strict").is_none());
            assert_eq!(context.item_work().consumed, ITEM_WORK_LIMIT);
            assert_eq!(context.stopped_routes(), vec!["strict"]);
        }
        assert_eq!(contexts[0].request_work().consumed, 500);
        assert!(contexts[0].request_work().exhausted);
    }

    #[test]
    fn debug_plan_work_allocation_reconciles_parent_budgets_and_recorded_routes() {
        let execution = VariantArticleExecutionContext::single();
        for _ in 0..44 {
            let started = execution.reserve("strict").expect("strict work");
            execution.record("strict", "pubmed", started, "ok", 1);
        }
        for _ in 0..EXACT_WORK_LIMIT {
            let started = execution
                .reserve("exact_lexical")
                .expect("exact lexical work");
            execution.record("exact_lexical", "pubmed", started, "ok", 1);
        }
        let started = execution
            .reserve("identity_verification")
            .expect("identity verification work");
        execution.record("identity_verification", "pubtator", started, "ok", 1);

        let allocation = execution.work_allocation();
        let item = execution.item_work();
        let recorded_calls = execution.events();
        let allocated = allocation.discovery.consumed
            + allocation.exact_lexical.item.consumed
            + allocation.identity_verification.item.consumed;

        assert!(
            allocation.discovery.consumed <= item.consumed
                && allocation.exact_lexical.item.consumed <= item.consumed
                && allocation.identity_verification.item.consumed <= item.consumed,
            "no work-allocation child may consume more than budgets.item"
        );
        assert_eq!(
            allocated, item.consumed,
            "work allocations must reconcile to budgets.item"
        );
        assert!(
            recorded_calls
                .iter()
                .any(|call| call.route == "strict" && call.source == "pubmed")
                && allocation.discovery.consumed > 0
                && recorded_calls
                    .iter()
                    .any(|call| call.route == "exact_lexical" && call.source == "pubmed")
                && allocation.exact_lexical.item.consumed > 0
                && recorded_calls
                    .iter()
                    .any(|call| call.route == "identity_verification" && call.source == "pubtator")
                && allocation.identity_verification.item.consumed > 0,
            "each recorded strict, exact, and identity provider call requires matching allocation"
        );
    }

    #[test]
    fn exact_and_identity_request_allowances_are_shared_and_bounded() {
        let contexts = VariantArticleExecutionContext::batch(2);
        for context in &contexts {
            for _ in 0..EXACT_WORK_LIMIT {
                assert!(context.reserve("exact_lexical").is_some());
            }
            for _ in 0..ITEM_WORK_LIMIT {
                assert!(context.reserve("identity_verification").is_some());
            }
        }

        let allocation = contexts[0].work_allocation();
        assert!(allocation.exact_lexical.request.exhausted);
        assert!(allocation.identity_verification.request.exhausted);
    }

    #[test]
    fn recorded_provider_calls_drive_public_and_debug_terminal_statuses() {
        let ok = VariantArticleCallEvent {
            route: "exact_lexical".into(),
            source: "pubmed".into(),
            status: "ok".into(),
            latency_ms: 0,
            pages: 1,
        };
        let unavailable = VariantArticleCallEvent {
            status: "unavailable".into(),
            ..ok.clone()
        };

        assert_eq!(
            provider_terminal_status(&[&ok]),
            VariantArticleSourceStatusKind::Ok
        );
        assert_eq!(
            provider_terminal_status(&[&ok, &unavailable]),
            VariantArticleSourceStatusKind::Degraded
        );
        assert_eq!(
            provider_terminal_status(&[&unavailable]),
            VariantArticleSourceStatusKind::Unavailable
        );
        assert_eq!(
            provider_statuses_for_route("exact_lexical", &[ok, unavailable])
                .into_iter()
                .map(|status| status.status)
                .collect::<Vec<_>>(),
            vec![VariantArticleSourceStatusKind::Degraded]
        );
    }

    #[test]
    fn only_a_recorded_route_stop_produces_a_stop_detail() {
        assert_eq!(route_stop_detail(false), None);
        assert_eq!(
            route_stop_detail(true),
            Some("internal work or configuration stopped before a provider call")
        );
    }

    #[test]
    fn terminal_state_is_complete_and_untruncated_after_an_auxiliary_budget_stop() {
        assert_eq!(
            variant_article_terminal_state(0, false, false, true, 0, false, 11, 11),
            (true, false),
        );
    }

    #[test]
    fn terminal_state_is_complete_when_pagination_withholds_candidates() {
        assert_eq!(
            variant_article_terminal_state(0, false, false, true, 0, true, 11, 5),
            (true, true),
        );
    }

    #[test]
    fn terminal_state_is_incomplete_and_truncated_when_requested_work_is_unperformed() {
        assert_eq!(
            variant_article_terminal_state(1, false, false, true, 0, false, 11, 11),
            (false, true),
        );
    }

    #[test]
    fn terminal_state_does_not_invent_an_internal_stop_for_a_provider_outage() {
        assert_eq!(
            variant_article_terminal_state(0, true, false, true, 0, false, 11, 11),
            (false, true),
        );
    }

    #[tokio::test]
    async fn annotation_pre_call_stop_is_reported_as_internal_work() {
        let execution = VariantArticleExecutionContext::single();
        for _ in 0..ITEM_WORK_LIMIT {
            assert!(execution.reserve("pubtator_variant").is_some());
        }

        let (rows, incomplete, succeeded, pre_call_stopped) =
            annotation_candidates("BRAF p.V600E", &resolved_context(), &execution)
                .await
                .expect("budget stop is not a provider failure");
        assert!(rows.is_empty());
        assert!(incomplete);
        assert!(!succeeded);
        assert!(pre_call_stopped);
    }

    #[tokio::test]
    async fn citation_pre_call_stop_is_reported_as_internal_work() {
        let execution = VariantArticleExecutionContext::single();
        for _ in 0..ITEM_WORK_LIMIT {
            assert!(execution.reserve("source_citation").is_some());
        }
        let mut context = resolved_context();
        context.source_hit = Some(crate::sources::myvariant::MyVariantHit {
            id: "fixture".into(),
            cadd: None,
            clinvar: None,
            dbnsfp: None,
            dbsnp: None,
            gnomad_exome: None,
            gnomad: None,
            exac: None,
            exac_nontcga: None,
            cosmic: None,
            cgi: None,
            civic: None,
        });

        let (rows, pre_call_stopped) = citation_candidates(&context, &execution)
            .await
            .expect("budget stop is not a provider failure");
        assert!(rows.is_empty());
        assert!(pre_call_stopped);
    }

    #[test]
    fn exact_lexical_allowance_survives_strict_budget_exhaustion() {
        let execution = VariantArticleExecutionContext::single();
        for _ in 0..ITEM_WORK_LIMIT {
            assert!(execution.reserve("strict").is_some());
        }

        assert!(
            execution.reserve("exact_lexical").is_some(),
            "strict retrieval must not consume exact lexical work"
        );
    }

    #[test]
    fn identity_verification_allowance_survives_strict_budget_exhaustion() {
        let execution = VariantArticleExecutionContext::single();
        for _ in 0..ITEM_WORK_LIMIT {
            assert!(execution.reserve("strict").is_some());
        }
        execution.reserve_identity_verification(1);

        assert!(
            execution.reserve("identity_verification").is_some(),
            "requested identity verification must not be stopped by retrieval work"
        );
    }

    #[test]
    fn ldh_direct_annotation_fetches_stop_after_ten_per_item() {
        let execution = VariantArticleExecutionContext::single();
        for _ in 0..10 {
            assert!(execution.reserve("clingen_ldh_direct").is_some());
        }
        assert!(execution.reserve("clingen_ldh_direct").is_none());
        // The LDH allowance is separate from the shared item budget, so a
        // request that spent its retrieval budget can still run the ladder.
        assert_eq!(execution.item_work().consumed, 0);
        assert_eq!(execution.stopped_routes(), vec!["clingen_ldh_direct"]);
    }

    #[test]
    fn ldh_medium_lookup_is_one_per_item_and_survives_an_exhausted_item_budget() {
        let execution = VariantArticleExecutionContext::single();
        for _ in 0..ITEM_WORK_LIMIT {
            assert!(execution.reserve("strict").is_some());
        }
        assert!(execution.reserve("strict").is_none());
        assert!(execution.reserve("clingen_ldh_medium").is_some());
        assert!(execution.reserve("clingen_ldh_medium").is_none());
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
            &[],
            &execution,
            VariantArticleDebugPlanState {
                counts: VariantArticleCountsPlan {
                    pre_dedup: 0,
                    post_dedup: 0,
                    returned: 0,
                },
                truncated: true,
                next: VariantArticleNextPlan {
                    offset: 0,
                    cursor: None,
                },
                candidate_trace: Vec::new(),
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
            identity: None,
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

    fn equivalence_observation(
        status: &str,
        caid: Option<&str>,
        exhaustive: bool,
    ) -> CanonicalEquivalenceObservation {
        CanonicalEquivalenceObservation {
            basis: "genomic".into(),
            query: "NC_000017.11:g.1A>G".into(),
            status: status.into(),
            caid: caid.map(str::to_string),
            provider_exhaustive: exhaustive,
            comparison_complete: matches!(status, "resolved" | "not_found"),
            source: "clingen_car".into(),
            request_template_version: "1".into(),
            car_version: None,
            provider_response_sha256: None,
        }
    }

    #[test]
    fn canonical_aliases_fill_only_unused_strict_alias_slots() {
        let mut context = resolved_context();
        context.requested = RequestedVariantIdentity {
            gene: Some("ATM".into()),
            coding_change: Some("c.1066-6T>G".into()),
            ..Default::default()
        };
        context.source_identity = None;
        context.resolution.normalized_aliases = NormalizedVariantAliases::default();
        let equivalence = CanonicalEquivalence {
            status: "confirmed".into(),
            caid: None,
            exhaustive: true,
            complete: true,
            applicable_identity_count: 2,
            observations: Vec::new(),
            message: String::new(),
            aliases: (0..10).map(|index| format!("alias-{index}")).collect(),
        };
        let (aliases, _) = exact_aliases_with_equivalence(&context, &equivalence);
        let plan = provider_variant_query_plan_with_aliases(
            "ATM c.1066-6T>G",
            &context,
            VariantArticleStrategy::Union,
            &aliases,
        );
        let sent = plan
            .iter()
            .filter(|query| query.route == "strict" && query.provider == "pubmed")
            .map(|query| query.query_alias.clone())
            .collect::<Vec<_>>();
        assert_eq!(sent.len(), MAX_EXACT_ALIASES);
        assert_eq!(
            &sent[..3],
            ["ATM c.1066-6T>G", "ATM alias-0", "ATM alias-1"]
        );
        assert_eq!(sent[3], "ATM alias-2");

        let debug_plan = build_debug_plan(
            "ATM c.1066-6T>G",
            &context,
            VariantArticleStrategy::Union,
            &aliases,
            &VariantArticleExecutionContext::single(),
            VariantArticleDebugPlanState {
                counts: VariantArticleCountsPlan {
                    pre_dedup: 0,
                    post_dedup: 0,
                    returned: 0,
                },
                truncated: false,
                next: VariantArticleNextPlan {
                    offset: 0,
                    cursor: None,
                },
                candidate_trace: Vec::new(),
            },
        );
        assert!(debug_plan.provider_queries.iter().any(|query| {
            query.route == "strict"
                && query.provider == "pubmed"
                && query.query_alias == "ATM alias-0"
        }));
    }

    #[test]
    fn canonical_equivalence_aggregation_uses_set_based_state_precedence() {
        let inapplicable = canonical_equivalence(Vec::new(), Vec::new());
        assert_eq!(inapplicable.status, "inapplicable");
        assert!(inapplicable.complete);

        let single = canonical_equivalence(
            vec![equivalence_observation("unavailable", None, false)],
            Vec::new(),
        );
        assert_eq!(single.status, "single_identity");
        assert!(!single.complete);

        let confirmed = canonical_equivalence(
            vec![
                equivalence_observation("resolved", Some("CA1"), true),
                equivalence_observation("resolved", Some("CA1"), true),
            ],
            Vec::new(),
        );
        assert_eq!(confirmed.status, "confirmed");
        assert!(confirmed.complete);

        let non_exhaustive_resolution = canonical_equivalence(
            vec![
                equivalence_observation("resolved", Some("CA1"), true),
                equivalence_observation("resolved", Some("CA1"), false),
            ],
            vec!["NM_000007.14:c.1A>G".into()],
        );
        assert_eq!(non_exhaustive_resolution.status, "indeterminate");
        assert!(!non_exhaustive_resolution.exhaustive);
        assert!(non_exhaustive_resolution.complete);
        assert!(non_exhaustive_resolution.aliases.is_empty());

        let contradictory = canonical_equivalence(
            vec![
                equivalence_observation("unavailable", None, false),
                equivalence_observation("resolved", Some("CA1"), true),
                equivalence_observation("resolved", Some("CA2"), true),
            ],
            Vec::new(),
        );
        assert_eq!(contradictory.status, "contradictory");
        assert!(!contradictory.complete);

        let indeterminate = canonical_equivalence(
            vec![
                equivalence_observation("not_found", None, true),
                equivalence_observation("resolved", Some("CA1"), true),
            ],
            Vec::new(),
        );
        assert_eq!(indeterminate.status, "indeterminate");
        assert!(indeterminate.complete);

        let not_found = canonical_equivalence(
            vec![
                equivalence_observation("not_found", None, true),
                equivalence_observation("not_found", None, true),
            ],
            Vec::new(),
        );
        assert_eq!(not_found.status, "not_found");
        assert!(not_found.complete);

        let unavailable = canonical_equivalence(
            vec![
                equivalence_observation("unavailable", None, false),
                equivalence_observation("not_found", None, true),
            ],
            Vec::new(),
        );
        assert_eq!(unavailable.status, "unavailable");
        assert!(!unavailable.complete);

        let provider_indeterminate = canonical_equivalence(
            vec![
                equivalence_observation("indeterminate", None, false),
                equivalence_observation("not_found", None, true),
            ],
            Vec::new(),
        );
        assert_eq!(provider_indeterminate.status, "indeterminate");
        assert!(!provider_indeterminate.complete);

        let invalid = canonical_equivalence(
            vec![
                equivalence_observation("invalid", None, true),
                equivalence_observation("not_found", None, true),
            ],
            Vec::new(),
        );
        assert_eq!(invalid.status, "indeterminate");
        assert!(!invalid.exhaustive);
        assert!(!invalid.complete);
    }

    #[test]
    fn canonical_equivalence_queries_require_explicit_versioned_refseq_inputs() {
        let requested = RequestedVariantIdentity {
            transcript: Some("NM_000051.4".into()),
            coding_change: Some("c.1066-6T>G".into()),
            genomic_accession: Some("NC_000011.10".into()),
            genome_build: Some("GRCh38".into()),
            position: Some(108248927),
            reference: Some("T".into()),
            alternate: Some("G".into()),
            ..Default::default()
        };
        assert_eq!(canonical_equivalence_queries(&requested).len(), 2);
        let only_gene = RequestedVariantIdentity {
            gene: Some("ATM".into()),
            ..Default::default()
        };
        assert!(canonical_equivalence_queries(&only_gene).is_empty());
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
