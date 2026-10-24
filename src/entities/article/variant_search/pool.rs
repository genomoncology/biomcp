//! Query binding and lossless candidate storage for immutable article pools.
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::super::identity_verification::{
    VariantArticleIdentity, VariantArticleVerificationOptions,
};
use super::stored::StoredArticleRow;
use super::{VariantArticleCandidateTraceRecord, VariantArticleProvenance, VariantArticleStrategy};
use crate::entities::article::ArticleSource;
use crate::entities::article::candidates::{ArticleCandidate, ArticleSourcePosition};
use crate::entities::variant::RequestedVariantIdentity;

/// This wrapper retains the fields deliberately omitted by public article JSON.
#[derive(Clone, Serialize, Deserialize)]
pub(super) struct StoredCandidate {
    row: StoredArticleRow,
    source_positions: Vec<(ArticleSource, usize)>,
    semantic_signal: Option<f64>,
    provenance: Vec<VariantArticleProvenance>,
    trace: Vec<VariantArticleCandidateTraceRecord>,
    identity: Option<StoredIdentity>,
}

impl From<ArticleCandidate> for StoredCandidate {
    fn from(value: ArticleCandidate) -> Self {
        Self {
            row: StoredArticleRow::capture(&value.row),
            source_positions: value
                .source_positions
                .into_iter()
                .map(|p| (p.source, p.local_position))
                .collect(),
            semantic_signal: value.semantic_signal,
            provenance: value.variant_provenance,
            trace: value.candidate_trace,
            identity: value.identity.map(Into::into),
        }
    }
}
impl From<StoredCandidate> for ArticleCandidate {
    fn from(value: StoredCandidate) -> Self {
        let row = value.row.restore();
        Self {
            row,
            source_positions: value
                .source_positions
                .into_iter()
                .map(|(source, local_position)| ArticleSourcePosition {
                    source,
                    local_position,
                })
                .collect(),
            semantic_signal: value.semantic_signal,
            variant_provenance: value.provenance,
            candidate_trace: value.trace,
            identity: value.identity.map(Into::into),
        }
    }
}

/// Only semantic settings enter this binding. Credentials never enter its input.
/// Unsafe custom endpoints disable persistence instead of hashing secrets.
pub(super) fn query_key(
    literal: &str,
    requested: &RequestedVariantIdentity,
    strategy: VariantArticleStrategy,
    verification: VariantArticleVerificationOptions,
) -> Option<String> {
    let mut settings = BTreeMap::new();
    for name in [
        "BIOMCP_MYVARIANT_BASE",
        "BIOMCP_MYGENE_BASE",
        "BIOMCP_PUBMED_BASE",
        "BIOMCP_EUROPEPMC_BASE",
        "BIOMCP_S2_BASE",
        "BIOMCP_PUBTATOR_BASE",
        "BIOMCP_LITVAR_BASE",
        "BIOMCP_CLINGEN_CAR_BASE",
        "BIOMCP_CLINGEN_LDH_FIXTURE_ORIGIN",
    ] {
        if let Ok(value) = std::env::var(name) {
            let url = reqwest::Url::parse(value.trim()).ok()?;
            if !url.username().is_empty()
                || url.password().is_some()
                || url.query().is_some()
                || url.fragment().is_some()
            {
                return None;
            }
            settings.insert(name, url.to_string().trim_end_matches('/').to_owned());
        }
    }
    if let Ok(disabled) = std::env::var("BIOMCP_DISABLE_SOURCES") {
        let mut disabled = disabled
            .split([',', '|'])
            .map(|v| v.trim().to_ascii_lowercase())
            .filter(|v| !v.is_empty())
            .collect::<Vec<_>>();
        disabled.sort();
        disabled.dedup();
        settings.insert("BIOMCP_DISABLE_SOURCES", disabled.join(","));
    }
    if let Ok(assembly) = std::env::var("BIOMCP_DEFAULT_ASSEMBLY") {
        settings.insert(
            "BIOMCP_DEFAULT_ASSEMBLY",
            assembly.trim().to_ascii_lowercase(),
        );
    }
    let binding = serde_json::to_vec(&(
        1_u32,
        super::page::ORDERING_VERSION,
        super::super::identity_verification::VERIFIER_VERSION,
        literal.split_whitespace().collect::<Vec<_>>().join(" "),
        requested,
        strategy,
        verification.verify_identity,
        verification.confirmed_only,
        settings,
    ))
    .ok()?;
    Some(format!("{:x}", Sha256::digest(binding)))
}

/// Identity observations have a compact public serializer. Store their full
/// internal graph here so restoration cannot discard typed linkage fields.
#[derive(Clone, Serialize, Deserialize)]
struct StoredIdentity {
    #[serde(deserialize_with = "crate::entities::article::variant_search::atoms::atom")]
    status: crate::entities::article::variant_search::atoms::Atom,
    #[serde(deserialize_with = "crate::entities::article::variant_search::atoms::atom")]
    basis: crate::entities::article::variant_search::atoms::Atom,
    #[serde(deserialize_with = "crate::entities::article::variant_search::atoms::optional")]
    reason: Option<crate::entities::article::variant_search::atoms::Atom>,
    requested_gene: Option<String>,
    requested_allele: Option<String>,
    observations: Vec<StoredObservation>,
    contradictions: Vec<StoredObservation>,
    incomplete: bool,
}

#[derive(Clone, Serialize, Deserialize)]
struct StoredObservation {
    #[serde(deserialize_with = "crate::entities::article::variant_search::atoms::atom")]
    source: crate::entities::article::variant_search::atoms::Atom,
    section: String,
    locator: String,
    linked_gene: String,
    observed_alias: String,
    gene_annotation_id: String,
    allele_annotation_id: String,
    provider_relation: Option<String>,
    provider_linkage: Option<super::super::identity_verification::ProviderLinkage>,
    canonical_content_hash: String,
}

impl From<VariantArticleIdentity> for StoredIdentity {
    fn from(value: VariantArticleIdentity) -> Self {
        Self {
            status: value.status,
            basis: value.basis,
            reason: value.reason,
            requested_gene: value.requested_gene,
            requested_allele: value.requested_allele,
            observations: value.observations.into_iter().map(Into::into).collect(),
            contradictions: value.contradictions.into_iter().map(Into::into).collect(),
            incomplete: value.incomplete,
        }
    }
}
impl From<StoredIdentity> for VariantArticleIdentity {
    fn from(value: StoredIdentity) -> Self {
        Self {
            status: value.status,
            basis: value.basis,
            reason: value.reason,
            requested_gene: value.requested_gene,
            requested_allele: value.requested_allele,
            observations: value.observations.into_iter().map(Into::into).collect(),
            contradictions: value.contradictions.into_iter().map(Into::into).collect(),
            incomplete: value.incomplete,
        }
    }
}
impl From<super::super::identity_verification::VariantArticleIdentityObservation>
    for StoredObservation
{
    fn from(value: super::super::identity_verification::VariantArticleIdentityObservation) -> Self {
        Self {
            source: value.source,
            section: value.section,
            locator: value.locator,
            linked_gene: value.linked_gene,
            observed_alias: value.observed_alias,
            gene_annotation_id: value.gene_annotation_id,
            allele_annotation_id: value.allele_annotation_id,
            provider_relation: value.provider_relation,
            provider_linkage: value.provider_linkage,
            canonical_content_hash: value.canonical_content_hash,
        }
    }
}
impl From<StoredObservation>
    for super::super::identity_verification::VariantArticleIdentityObservation
{
    fn from(value: StoredObservation) -> Self {
        Self {
            source: value.source,
            section: value.section,
            locator: value.locator,
            linked_gene: value.linked_gene,
            observed_alias: value.observed_alias,
            gene_annotation_id: value.gene_annotation_id,
            allele_annotation_id: value.allele_annotation_id,
            provider_relation: value.provider_relation,
            provider_linkage: value.provider_linkage,
            canonical_content_hash: value.canonical_content_hash,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct StoredPool {
    context: StoredContext,
    ordered: Vec<StoredRow>,
}

#[derive(Clone, Serialize, Deserialize)]
struct StoredContext {
    gene_symbol_coverage: super::VariantArticleGeneSymbolCoverage,
    alias_coverage: Vec<super::VariantArticleAliasCoverage>,
    requested_variant: RequestedVariantIdentity,
    resolution: crate::entities::variant::VariantArticleResolution,
    canonical_equivalence: Option<super::CanonicalEquivalence>,
    canonical_aliases: Vec<String>,
    debug_coverage_aliases: Vec<Vec<String>>,
    strategy: VariantArticleStrategy,
    complete: bool,
    truncated: bool,
    source_status: Vec<super::VariantArticleSourceStatus>,
    #[serde(deserialize_with = "crate::entities::article::variant_search::atoms::atom")]
    retrieval_path: crate::entities::article::variant_search::atoms::Atom,
    error: Option<super::VariantArticleItemError>,
    debug_plan: Option<super::VariantArticleDebugPlan>,
}

#[derive(Clone, Serialize, Deserialize)]
struct StoredRow {
    candidate: StoredCandidate,
    matched_aliases: Vec<String>,
    #[serde(deserialize_with = "crate::entities::article::variant_search::atoms::atom")]
    match_reason: crate::entities::article::variant_search::atoms::Atom,
    alias_matches: Vec<super::VariantArticleAliasMatch>,
    retrieval_routes: Vec<String>,
    sources: Vec<String>,
    rank: usize,
    metadata_status: Option<crate::entities::article::ArticleMetadataStatus>,
}

impl StoredPool {
    pub(super) fn capture(
        response: super::VariantArticleResponse,
        candidates: Vec<ArticleCandidate>,
    ) -> Self {
        let canonical_aliases = response
            .canonical_equivalence
            .as_ref()
            .map(|value| value.aliases.clone())
            .unwrap_or_default();
        let debug_coverage_aliases = response
            .debug_plan
            .as_ref()
            .map(|plan| {
                plan.provider_queries
                    .iter()
                    .map(|query| query.coverage_aliases.clone())
                    .collect()
            })
            .unwrap_or_default();
        let context = StoredContext {
            gene_symbol_coverage: response.gene_symbol_coverage,
            alias_coverage: response.alias_coverage,
            requested_variant: response.requested_variant,
            resolution: response.resolution,
            canonical_equivalence: response.canonical_equivalence,
            canonical_aliases,
            debug_coverage_aliases,
            strategy: response.strategy,
            complete: response.complete,
            truncated: response.truncated,
            source_status: response.source_status,
            retrieval_path: response.retrieval_path,
            error: response.error,
            debug_plan: response.debug_plan,
        };
        let ordered = candidates
            .into_iter()
            .zip(response.results)
            .map(|(mut candidate, row)| {
                // Visible enrichment may have filled display metadata after ranking.
                candidate.row = row.article;
                candidate.identity = row.identity;
                StoredRow {
                    candidate: candidate.into(),
                    matched_aliases: row.matched_aliases,
                    match_reason: row.match_reason,
                    alias_matches: row.alias_matches,
                    retrieval_routes: row.retrieval_routes,
                    sources: row.sources,
                    rank: row.rank,
                    metadata_status: row.metadata_status,
                }
            })
            .collect();
        Self { context, ordered }
    }

    fn validate(
        &self,
        requested: &RequestedVariantIdentity,
        strategy: VariantArticleStrategy,
    ) -> bool {
        if self.context.requested_variant != *requested
            || self.context.strategy != strategy
            || self.context.error.as_ref().is_some_and(|error| {
                !matches!(error.code, "deadline_exceeded" | "source_unavailable")
            })
        {
            return false;
        }
        let expected = super::verify_captured_abstract(requested, "");
        let mut identifiers = std::collections::BTreeSet::new();
        self.ordered.iter().enumerate().all(|(index, stored)| {
            let id =
                super::super::candidates::stable_article_identifier(stored.candidate.row.article());
            stored.rank == index + 1
                && !id.is_empty()
                && identifiers.insert(id)
                && stored.candidate.identity.as_ref().is_none_or(|identity| {
                    identity.requested_gene == expected.requested_gene
                        && identity.requested_allele == expected.requested_allele
                        && identity.status
                            == super::super::identity_verification::status_for_presence(
                                !identity.observations.is_empty(),
                                !identity.contradictions.is_empty(),
                            )
                        && (matches!(identity.basis, "none" | "structured_annotation") || identity.basis == super::super::identity_verification::PAPER_TRANSCRIPT_ANCHOR_BASIS)
                })
        })
    }

    pub(super) fn complete(&self) -> bool {
        self.context.complete
    }

    pub(super) fn into_response(
        self,
        offset: usize,
        limit: usize,
        token: Option<String>,
        debug: bool,
    ) -> Result<super::VariantArticleOutcome, crate::error::BioMcpError> {
        if !self.validate(&self.context.requested_variant, self.context.strategy) {
            return Err(crate::error::BioMcpError::InvalidArgument(
                "Invalid stored article pool; restart at offset 0 without a token".into(),
            ));
        }
        let total = self.ordered.len();
        let mut context = self.context;
        if let Some(canonical) = context.canonical_equivalence.as_mut() {
            canonical.aliases = context.canonical_aliases;
        }
        if let Some(plan) = context.debug_plan.as_mut() {
            for (query, aliases) in plan
                .provider_queries
                .iter_mut()
                .zip(context.debug_coverage_aliases)
            {
                query.coverage_aliases = aliases;
            }
        }

        let results = self
            .ordered
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(|stored| {
                let candidate: ArticleCandidate = stored.candidate.into();
                super::VariantArticleRow {
                    article: candidate.row,
                    requested_variant: context.requested_variant.clone(),
                    genomic_coordinates: context
                        .resolution
                        .normalized_aliases
                        .genomic_coordinates
                        .clone(),
                    matched_aliases: stored.matched_aliases,
                    match_reason: stored.match_reason,
                    alias_matches: stored.alias_matches,
                    retrieval_routes: stored.retrieval_routes,
                    sources: stored.sources,
                    rank: stored.rank,
                    provenance: candidate.variant_provenance,
                    identity: candidate.identity,
                    metadata_status: stored.metadata_status,
                }
            })
            .collect::<Vec<_>>();
        if let Some(plan) = context.debug_plan.as_mut() {
            plan.counts.returned = results.len();
            plan.next.offset = offset.saturating_add(results.len());
            plan.next.cursor.clone_from(&token);
            for trace in &mut plan.candidate_trace.candidates {
                trace.pagination_disposition = if trace
                    .rank_position
                    .is_some_and(|rank| rank > offset && rank <= offset.saturating_add(limit))
                {
                    "visible"
                } else {
                    "not_visible"
                }
                .into();
            }
        }
        let hard_error = results.is_empty() && context.error.is_some();
        Ok(super::VariantArticleOutcome {
            response: super::VariantArticleResponse {
                gene_symbol_coverage: context.gene_symbol_coverage,
                alias_coverage: context.alias_coverage,
                requested_variant: context.requested_variant,
                resolution: context.resolution,
                canonical_equivalence: context.canonical_equivalence,
                strategy: context.strategy,
                complete: context.complete,
                truncated: context.truncated,
                pagination: super::VariantArticlePagination {
                    offset,
                    limit,
                    returned: results.len(),
                    total: Some(total),
                    has_more: offset.saturating_add(results.len()) < total,
                    next_page_token: token,
                },
                source_status: context.source_status,
                retrieval_path: context.retrieval_path,
                results,
                error: context.error,
                debug_plan: if debug { context.debug_plan } else { None },
            },
            hard_error,
        })
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn execute(
    input: &str,
    requested: RequestedVariantIdentity,
    strategy: VariantArticleStrategy,
    limit: usize,
    offset: usize,
    debug: bool,
    verification: VariantArticleVerificationOptions,
    execution: super::VariantArticleExecutionContext,
    page_token: Option<&str>,
) -> Result<super::VariantArticleOutcome, crate::error::BioMcpError> {
    use crate::cache::variant_article_pool::PoolStore;
    use crate::error::BioMcpError;
    if limit == 0 || limit > 50 {
        return Err(BioMcpError::InvalidArgument(
            "--limit must be between 1 and 50".into(),
        ));
    }
    let key = query_key(input, &requested, strategy, verification);
    let persistent = key.is_some() && !crate::sources::cache_is_bypassed();
    if !persistent && (offset != 0 || page_token.is_some()) {
        return Err(super::page::unavailable());
    }
    let key = key.unwrap_or_default();
    let acquisition_key = query_key(
        input,
        &requested,
        VariantArticleStrategy::Union,
        VariantArticleVerificationOptions::default(),
    )
    .unwrap_or_default();
    let store = PoolStore::new(crate::cache::resolve_cache_config()?);
    let continuation = page_token
        .map(|token| super::page::Continuation::decode(token, &key, offset))
        .transpose()?;
    let page_offset = continuation.as_ref().map_or(offset, |token| token.offset);
    let existing = if !persistent {
        None
    } else {
        let read = if let Some(token) = &continuation {
            store.read(&key, &token.snapshot)
        } else {
            store.selected(&key, offset != 0)
        };
        match read {
            Ok(value) => value,
            Err(error) if offset == 0 && continuation.is_none() => {
                crate::cache::warn_local_http_cache("article-pool-read", &error);
                None
            }
            Err(_) => return Err(super::page::unavailable()),
        }
    };
    if existing.is_none() && (offset != 0 || continuation.is_some()) {
        return Err(super::page::unavailable());
    }
    let decoded = existing
        .as_ref()
        .map(|snapshot| serde_json::from_value::<StoredPool>(snapshot.payload.clone()))
        .transpose();
    let mut loaded = match decoded {
        Ok(value) => value,
        Err(_) if continuation.is_some() || offset != 0 => return Err(super::page::unavailable()),
        Err(_) => None,
    };
    if loaded
        .as_ref()
        .is_some_and(|pool| !pool.validate(&requested, strategy))
    {
        if continuation.is_some() || offset != 0 {
            return Err(super::page::unavailable());
        }
        loaded = None;
    }
    if loaded.as_ref().is_some_and(StoredPool::complete) {
        let snapshot = existing.as_ref().expect("decoded stored pool");
        if offset == 0
            && continuation.is_none()
            && let Err(error) = store.activate(snapshot)
        {
            crate::cache::warn_local_http_cache("article-pool-activate", &error);
        }
        let pool = loaded.take().expect("complete stored pool");
        let next =
            page_offset.saturating_add(limit.min(pool.ordered.len().saturating_sub(page_offset)));
        let token = Some(super::page::Continuation::encode(&snapshot.id, &key, next));
        let outcome = pool.into_response(page_offset, limit, token, debug)?;
        return Ok(super::page::enrich_visible(outcome, &execution, verification).await);
    }
    let retained = loaded
        .as_ref()
        .map(|pool| {
            pool.ordered
                .iter()
                .map(|row| row.candidate.clone().into())
                .collect::<Vec<ArticleCandidate>>()
        })
        .unwrap_or_default();
    if persistent
        && let Ok(Some(value)) = store.progress(&acquisition_key, "literal_pubmed")
        && let Ok(progress) = serde_json::from_value(value)
    {
        execution.set_literal_pubmed_progress(Some(progress));
    }
    if persistent && let Ok(entries) = store.all_progress(&acquisition_key) {
        let mut progress = BTreeMap::new();
        for (key, value) in entries {
            if let Some(provider_key) = key.strip_prefix("provider:")
                && let Ok(value) = serde_json::from_value(value)
            {
                progress.insert(provider_key.to_owned(), value);
            }
        }
        execution.restore_provider_progress(progress);
    }
    let mut acquisition_execution = execution.clone();
    if existing.is_some() {
        acquisition_execution.deadline =
            execution.deadline.capped(std::time::Duration::from_secs(2));
    }
    let acquisition_verification = if existing.is_some() && !verification.confirmed_only {
        VariantArticleVerificationOptions::default()
    } else {
        verification
    };
    let acquisition_deadline = acquisition_execution.deadline().clone();
    let (acquired, candidates) = crate::sources::with_variant_article_deadline(
        acquisition_deadline,
        super::acquire_variant_article_pool(
            input,
            requested,
            strategy,
            limit,
            page_offset,
            true,
            acquisition_verification,
            acquisition_execution,
            retained,
            existing.is_none(),
        ),
    )
    .await?;
    if persistent
        && let Some(progress) = execution.literal_pubmed_progress()
        && let Ok(value) = serde_json::to_value(progress)
        && let Err(error) = store.save_progress(&acquisition_key, "literal_pubmed", value)
    {
        crate::cache::warn_local_http_cache("article-progress-write", &error);
    }
    if persistent {
        for (provider_key, progress) in execution.all_provider_progress() {
            if let Ok(value) = serde_json::to_value(progress)
                && let Err(error) = store.save_progress(
                    &acquisition_key,
                    &format!("provider:{provider_key}"),
                    value,
                )
            {
                crate::cache::warn_local_http_cache("article-provider-progress-write", &error);
            }
        }
    }
    let pool = StoredPool::capture(acquired.response, candidates);
    let payload = serde_json::to_value(&pool).map_err(BioMcpError::Json)?;
    let snapshot = store.snapshot(&key, payload);
    let published = persistent
        && match store.publish(&snapshot) {
            Ok(()) => true,
            Err(error) => {
                crate::cache::warn_local_http_cache("article-pool-write", &error);
                false
            }
        };
    // A bound or legacy continuation keeps its old membership while a retry
    // publishes a successor. Only a new offset-zero session activates it.
    let (selected_pool, selected_id) =
        if let (Some(old), Some(old_pool)) = (existing.as_ref(), loaded) {
            if continuation.is_some() || offset != 0 {
                (old_pool, Some(old.id.clone()))
            } else {
                (pool, published.then(|| snapshot.id.clone()))
            }
        } else {
            (pool, published.then(|| snapshot.id.clone()))
        };
    if offset == 0
        && continuation.is_none()
        && published
        && let Err(error) = store.activate(&snapshot)
    {
        crate::cache::warn_local_http_cache("article-pool-activate", &error);
    }
    let next = page_offset
        .saturating_add(limit.min(selected_pool.ordered.len().saturating_sub(page_offset)));
    let token = selected_id.map(|id| super::page::Continuation::encode(&id, &key, next));
    let outcome = selected_pool.into_response(page_offset, limit, token, debug)?;
    if existing.is_some() {
        Ok(super::page::enrich_visible(outcome, &execution, verification).await)
    } else {
        Ok(outcome)
    }
}
