//! Membership acquisition retains identifiers before optional hydration.

use serde::{Deserialize, Serialize};

use super::stored::StoredArticleRow;
use super::*;
use crate::sources::pubmed::{PubMedClient, PubMedESearchParams, PubMedSearchSort};

const LITERAL_PUBMED_ROUTE: &str = "literal_pubmed";
const PUBMED_RETRIEVABLE_IDS: usize = 10_000;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct ProviderProgress {
    pub rows: Vec<StoredArticleRow>,
    pub offset: usize,
    pub cursor: Option<String>,
    pub total: Option<usize>,
    pub complete: bool,
    #[serde(default)]
    pub aliases: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub lookup: Option<Vec<(String, String)>>,
}

impl ProviderProgress {
    pub(crate) fn restored_rows(&self) -> Vec<ArticleSearchResult> {
        self.rows
            .iter()
            .cloned()
            .map(StoredArticleRow::restore)
            .collect()
    }

    pub(crate) fn capture(
        rows: &[ArticleSearchResult],
        offset: usize,
        cursor: Option<String>,
        total: Option<usize>,
        complete: bool,
    ) -> Self {
        Self {
            rows: rows.iter().map(StoredArticleRow::capture).collect(),
            offset,
            cursor,
            total,
            complete,
            aliases: BTreeMap::new(),
            lookup: None,
        }
    }
}

pub(crate) fn pubmed_membership_seed(pmid: String, native_position: usize) -> ArticleSearchResult {
    let mut row = pmid_seed(pmid);
    row.source_local_position = native_position;
    row
}

fn retained_candidates(
    progress: &ProviderProgress,
    route: &str,
    source: &str,
) -> Vec<ArticleCandidate> {
    progress
        .restored_rows()
        .into_iter()
        .map(|row| {
            let aliases = progress
                .aliases
                .get(&stable_article_identifier(&row))
                .cloned()
                .unwrap_or_default();
            candidate_with_provenance(row, route, source, aliases)
        })
        .collect()
}

fn save_candidates(
    execution: &VariantArticleExecutionContext,
    key: String,
    candidates: &[ArticleCandidate],
    complete: bool,
) {
    let rows = candidates
        .iter()
        .map(|candidate| candidate.row.clone())
        .collect::<Vec<_>>();
    let mut progress = ProviderProgress::capture(&rows, 0, None, Some(rows.len()), complete);
    for candidate in candidates {
        progress.aliases.insert(
            stable_article_identifier(&candidate.row),
            candidate
                .variant_provenance
                .iter()
                .flat_map(|fact| fact.query_aliases.iter().cloned())
                .collect(),
        );
    }
    execution.set_provider_progress(key, progress);
}

pub(super) async fn requested_pubtator_candidates(
    input: &str,
    context: &VariantArticleResolutionContext,
    execution: &VariantArticleExecutionContext,
    route: &str,
) -> Result<RequestedPubTatorResult, BioMcpError> {
    let provider_execution = execution.for_provider("pubtator");
    let execution = &provider_execution;
    let query = format!("requested-entity:{input}");
    let (key, progress) = retained_provider_progress(Some(execution), route, "pubtator", &query);
    let mut retained = retained_candidates(&progress, route, "pubtator");
    if progress.complete {
        return Ok((retained, false, true, false));
    }
    let (rows, incomplete, succeeded, stopped) =
        match super::requested_pubtator_candidates(input, context, execution, route).await {
            Ok(result) => result,
            Err(error) if retained.is_empty() => return Err(error),
            Err(_) => (Vec::new(), true, false, false),
        };
    retained.extend(rows);
    let retained = super::merge::merge_candidates(retained);
    save_candidates(execution, key, &retained, succeeded && !incomplete);
    Ok((retained, incomplete, succeeded, stopped))
}

pub(super) async fn litvar_candidates(
    input: &str,
    context: &VariantArticleResolutionContext,
    execution: &VariantArticleExecutionContext,
) -> Result<(Vec<ArticleCandidate>, bool, bool), BioMcpError> {
    let provider_execution = execution.for_provider("litvar");
    let execution = &provider_execution;
    let (key, progress) =
        retained_provider_progress(Some(execution), "litvar_variant", "litvar", input);
    let mut retained = retained_candidates(&progress, "litvar_variant", "litvar");
    if progress.complete {
        return Ok((retained, false, true));
    }
    let (rows, incomplete, succeeded) = match enumerate_litvar(input, context, execution).await {
        Ok(result) => result,
        Err(error) if retained.is_empty() => return Err(error),
        Err(_) => (Vec::new(), true, false),
    };
    retained.extend(rows);
    let retained = super::merge::merge_candidates(retained);
    save_candidates(execution, key, &retained, succeeded && !incomplete);
    Ok((retained, incomplete, succeeded))
}

async fn enumerate_litvar(
    input: &str,
    context: &VariantArticleResolutionContext,
    execution: &VariantArticleExecutionContext,
) -> Result<(Vec<ArticleCandidate>, bool, bool), BioMcpError> {
    let matched = litvar_lookup(input, &context.requested, execution).await?;
    if matched.is_empty() {
        return Ok((Vec::new(), false, true));
    }
    let mut candidates = Vec::new();
    let mut incomplete = false;
    let mut succeeded = false;
    for (variant_id, matched_alias) in matched {
        execution.add_route_unit("litvar_variant", "litvar");
        let alias = context
            .requested
            .gene
            .as_deref()
            .filter(|_| !matched_alias.to_ascii_lowercase().starts_with("rs"))
            .map(|gene| format!("{gene} {matched_alias}"))
            .unwrap_or(matched_alias);
        match litvar_publications(&variant_id, &alias, execution).await {
            Ok((rows, complete)) => {
                candidates.extend(rows);
                succeeded = true;
                incomplete |= !complete;
            }
            Err(_) => {
                incomplete = true;
            }
        }
    }
    Ok((candidates, incomplete, succeeded))
}

async fn litvar_lookup(
    input: &str,
    requested: &RequestedVariantIdentity,
    execution: &VariantArticleExecutionContext,
) -> Result<Vec<(String, String)>, BioMcpError> {
    let query = format!("autocomplete:{input}");
    let (key, progress) =
        retained_provider_progress(Some(execution), "litvar_variant", "litvar", &query);
    if progress.complete
        && let Some(matched) = progress.lookup
    {
        return Ok(matched);
    }
    let (client, mut first_unit) = super::super::backends::variant_article_client(
        Some(execution),
        "litvar_variant",
        "litvar",
        crate::sources::litvar::LitVarClient::new_with_deadline(execution.deadline()),
    )
    .await?;
    let result = super::super::backends::variant_article_request(
        Some(execution),
        "litvar_variant",
        "litvar",
        &mut first_unit,
        client.autocomplete(input),
        |entries| {
            let matched = entries
                .into_iter()
                .filter_map(|entry| {
                    let identity = entry.source_identity()?;
                    match crate::entities::variant::compare_variant_identity(requested, &identity) {
                        crate::entities::variant::VariantIdentityComparison::Compatible {
                            matched_alias,
                        } => Some((identity.genomic_id.clone(), matched_alias)),
                        _ => None,
                    }
                })
                .collect::<Vec<_>>();
            let progress = ProviderProgress {
                complete: true,
                lookup: Some(matched.clone()),
                ..ProviderProgress::default()
            };
            execution.set_provider_progress(key, progress);
            Ok(matched)
        },
    )
    .await?;
    result.ok_or_else(|| BioMcpError::SourceUnavailable {
        source_name: "LitVar2".into(),
        reason: "LitVar lookup was not admitted".into(),
        suggestion: "Retry the variant article request".into(),
    })
}

async fn litvar_publications(
    variant_id: &str,
    alias: &str,
    execution: &VariantArticleExecutionContext,
) -> Result<(Vec<ArticleCandidate>, bool), BioMcpError> {
    let query = format!("publications:{variant_id}");
    let (key, progress) =
        retained_provider_progress(Some(execution), "litvar_variant", "litvar", &query);
    if progress.complete {
        return Ok((
            retained_candidates(&progress, "litvar_variant", "litvar"),
            true,
        ));
    }
    let (client, mut first_unit) = super::super::backends::variant_article_client(
        Some(execution),
        "litvar_variant",
        "litvar",
        crate::sources::litvar::LitVarClient::new_with_deadline(execution.deadline()),
    )
    .await?;
    let result = super::super::backends::variant_article_request(
        Some(execution),
        "litvar_variant",
        "litvar",
        &mut first_unit,
        client.publications(variant_id),
        |publications| {
            let complete = publications
                .pmids_count
                .is_none_or(|count| count <= publications.pmids.len() as u64);
            let mut candidates = retained_candidates(&progress, "litvar_variant", "litvar");
            candidates.extend(
                publications
                    .pmids
                    .into_iter()
                    .enumerate()
                    .map(|(position, pmid)| {
                        candidate_with_provenance(
                            pubmed_membership_seed(pmid.to_string(), position),
                            "litvar_variant",
                            "litvar",
                            vec![alias.into()],
                        )
                    })
                    .collect::<Vec<_>>(),
            );
            let candidates = super::merge::merge_candidates(candidates);
            save_candidates(execution, key, &candidates, complete);
            Ok((candidates, complete))
        },
    )
    .await?;
    result.ok_or_else(|| BioMcpError::SourceUnavailable {
        source_name: "LitVar2".into(),
        reason: "LitVar publications were not admitted".into(),
        suggestion: "Retry the variant article request".into(),
    })
}

pub(crate) fn provider_progress_key(_route: &str, source: &str, query: &str) -> String {
    // A provider's query semantics, rather than its enclosing union strategy,
    // bind successful membership and its native continuation.
    format!("{source}\u{1f}{query}")
}

pub(crate) fn retained_provider_progress(
    execution: Option<&VariantArticleExecutionContext>,
    route: &str,
    source: &str,
    query: &str,
) -> (String, ProviderProgress) {
    let key = provider_progress_key(route, source, query);
    let progress = execution
        .and_then(|execution| execution.provider_progress(&key))
        .unwrap_or_default();
    if progress.complete
        && let Some(execution) = execution
    {
        execution.record_alias_query_attempt(route, source);
        execution.record(route, source, Instant::now(), "ok", 0);
    }
    (key, progress)
}

pub(crate) fn save_provider_progress(
    execution: Option<&VariantArticleExecutionContext>,
    key: &str,
    rows: &[ArticleSearchResult],
    offset: usize,
    cursor: Option<String>,
    total: Option<usize>,
    complete: bool,
) {
    if let Some(execution) = execution {
        execution.set_provider_progress(
            key.into(),
            ProviderProgress::capture(rows, offset, cursor, total, complete),
        );
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct LiteralPubMedProgress {
    pub query: String,
    pub total: Option<u64>,
    pub next_offset: usize,
    pub pmids: Vec<String>,
    pub complete: bool,
    pub hard_limit: bool,
}

pub(super) struct SourceAcquisition {
    pub candidates: Vec<ArticleCandidate>,
    pub status: Vec<VariantArticleSourceStatus>,
    pub progress: Option<LiteralPubMedProgress>,
}

pub(super) type ComponentAcquisition = (
    Vec<ArticleCandidate>,
    bool,
    bool,
    Vec<VariantArticleSourceStatus>,
);

fn unused_component() -> ComponentAcquisition {
    (Vec::new(), false, false, Vec::new())
}

/// Run each applicable component with its own reserved provider allowance.
/// A slow annotation or citation source cannot delay lexical admission.
pub(super) async fn acquire_components(
    input: &str,
    context: &VariantArticleResolutionContext,
    strategy: VariantArticleStrategy,
    aliases: &[String],
    aliases_truncated: bool,
    execution: &VariantArticleExecutionContext,
) -> ComponentAcquisition {
    let resolved = context.resolution.status == VariantResolutionStatus::Resolved;
    let compatible = context.resolution.provider_validation.status
        != VariantProviderValidationStatus::Contradictory;
    let union = strategy == VariantArticleStrategy::Union;
    let planned = async {
        if compatible && (union || (strategy == VariantArticleStrategy::Lexical && resolved)) {
            planned_candidates(input, context, strategy, aliases, execution).await
        } else {
            unused_component()
        }
    };
    let lexical = async {
        if compatible
            && resolved
            && matches!(
                strategy,
                VariantArticleStrategy::Union | VariantArticleStrategy::Lexical
            )
        {
            lexical_candidates(
                execution.lexical_aliases(aliases),
                aliases_truncated,
                execution,
            )
            .await
        } else {
            unused_component()
        }
    };
    let annotation = acquire_annotation(input, context, strategy, execution);
    let citations = acquire_citations(context, strategy, execution);
    let litvar = async {
        let mut candidates = Vec::new();
        let mut status = Vec::new();
        let (succeeded, failed) = litvar_route_outcome(
            input,
            context,
            strategy,
            execution,
            &mut candidates,
            &mut status,
        )
        .await;
        (candidates, failed, succeeded, status)
    };
    let fallback = async {
        if union && !resolved {
            fallback_candidates(input, context, execution).await
        } else {
            unused_component()
        }
    };
    let exact = async {
        if strategy == VariantArticleStrategy::Exact {
            exact_name_candidates(input, execution).await
        } else {
            unused_component()
        }
    };
    let (planned, lexical, annotation, citations, litvar, fallback, exact) = tokio::join!(
        planned, lexical, annotation, citations, litvar, fallback, exact
    );
    let mut acquired = unused_component();
    for (rows, incomplete, succeeded, statuses) in [
        planned, lexical, annotation, citations, litvar, fallback, exact,
    ] {
        acquired.0.extend(rows);
        acquired.1 |= incomplete;
        acquired.2 |= succeeded;
        acquired.3.extend(statuses);
    }
    acquired
}

async fn acquire_annotation(
    input: &str,
    context: &VariantArticleResolutionContext,
    strategy: VariantArticleStrategy,
    execution: &VariantArticleExecutionContext,
) -> ComponentAcquisition {
    if context.resolution.status != VariantResolutionStatus::Resolved
        || context.resolution.provider_validation.status
            == VariantProviderValidationStatus::Contradictory
        || !matches!(
            strategy,
            VariantArticleStrategy::Union | VariantArticleStrategy::Annotation
        )
    {
        return unused_component();
    }
    match annotation_candidates(input, context, execution).await {
        Ok((rows, incomplete, succeeded, _)) => {
            let statuses = provider_statuses_for_route("pubtator_variant", &execution.events());
            (rows, incomplete || !succeeded, succeeded, statuses)
        }
        Err(_) => (
            Vec::new(),
            true,
            false,
            vec![status(
                "pubtator_variant",
                "pubtator",
                VariantArticleSourceStatusKind::Unavailable,
            )],
        ),
    }
}

async fn acquire_citations(
    context: &VariantArticleResolutionContext,
    strategy: VariantArticleStrategy,
    execution: &VariantArticleExecutionContext,
) -> ComponentAcquisition {
    if strategy != VariantArticleStrategy::Union {
        return unused_component();
    }
    if context.resolution.provider_validation.status != VariantProviderValidationStatus::Confirmed {
        return (
            Vec::new(),
            false,
            false,
            vec![status_with_detail(
                "source_citation",
                "myvariant",
                VariantArticleSourceStatusKind::Skipped,
                Some("provider identity was not confirmed"),
            )],
        );
    }
    match citation_candidates(context, execution).await {
        Ok((rows, stopped)) => (
            rows,
            stopped,
            !stopped,
            provider_statuses_for_route("source_citation", &execution.events()),
        ),
        Err(_) => (
            Vec::new(),
            true,
            false,
            vec![status(
                "source_citation",
                "myvariant",
                VariantArticleSourceStatusKind::Unavailable,
            )],
        ),
    }
}

impl SourceAcquisition {
    fn empty() -> Self {
        Self {
            candidates: Vec::new(),
            status: Vec::new(),
            progress: None,
        }
    }
}

/// This query uses the caller's words and never depends on provider identity.
pub(super) fn literal_query(input: &str, requested: &RequestedVariantIdentity) -> Option<String> {
    if requested.gene.is_none()
        || requested.protein_change.is_none()
        || requested.is_authoritative_refseq()
    {
        return None;
    }
    Some(input.split_whitespace().collect::<Vec<_>>().join(" "))
}

pub(super) async fn acquire_literal_pubmed(
    query: Option<&str>,
    prior: Option<LiteralPubMedProgress>,
    execution: &VariantArticleExecutionContext,
) -> SourceAcquisition {
    let provider_execution = execution.for_provider("pubmed");
    let execution = &provider_execution;
    let Some(query) = query else {
        return SourceAcquisition::empty();
    };
    let mut progress = prior
        .filter(|prior| prior.query == query)
        .unwrap_or_else(|| LiteralPubMedProgress {
            query: query.into(),
            ..Default::default()
        });
    execution.set_route_unit_plan(LITERAL_PUBMED_ROUTE, "pubmed", Some(0));
    if !progress.complete && !progress.hard_limit {
        enumerate_literal_pubmed(&mut progress, execution).await;
    }
    let mut candidates = progress
        .pmids
        .iter()
        .enumerate()
        .map(|(position, pmid)| {
            let mut row = pmid_seed(pmid.clone());
            row.source_local_position = position;
            candidate_with_provenance(row, LITERAL_PUBMED_ROUTE, "pubmed", vec![query.into()])
        })
        .collect::<Vec<_>>();
    hydrate_literal_pubmed(&mut candidates, query, execution).await;
    let events = execution.events();
    let mut status = provider_statuses_for_route(LITERAL_PUBMED_ROUTE, &events);
    if !progress.complete {
        let mut unfinished = status.pop().unwrap_or_else(|| {
            status_with_detail(
                LITERAL_PUBMED_ROUTE,
                "pubmed",
                VariantArticleSourceStatusKind::Degraded,
                None,
            )
        });
        unfinished.status = VariantArticleSourceStatusKind::Degraded;
        unfinished.detail = Some(
            if progress.hard_limit {
                "PubMed's 10000-ID retrieval limit prevents a complete literal query floor"
            } else {
                "literal PubMed enumeration is unfinished; retained PMIDs remain available"
            }
            .into(),
        );
        unfinished.reason_codes.push(if progress.hard_limit {
            "query_result_cap"
        } else {
            "logical_work_cap"
        });
        status.push(unfinished);
    }
    SourceAcquisition {
        candidates,
        status,
        progress: Some(progress),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Atomic resource admission needs direct boundary coverage in addition to
    // the caller-facing provider replay.
    #[test]
    fn provider_reservations_respect_both_shared_ceilings_without_leaking_units() {
        for item_full in [true, false] {
            let execution = VariantArticleExecutionContext::single();
            let (full, unchanged) = if item_full {
                (&execution.item, &execution.request)
            } else {
                (&execution.request, &execution.item)
            };
            full.consumed.store(full.limit, AtomicOrdering::SeqCst);
            assert!(
                execution
                    .reserve_provider("literal_pubmed", "pubmed")
                    .is_none()
            );
            assert_eq!(full.consumed.load(AtomicOrdering::SeqCst), full.limit);
            assert_eq!(unchanged.consumed.load(AtomicOrdering::SeqCst), 0);
            assert!(
                execution
                    .provider_work
                    .lock()
                    .unwrap()
                    .values()
                    .all(|count| *count == 0)
            );
        }
    }
}

async fn enumerate_literal_pubmed(
    progress: &mut LiteralPubMedProgress,
    execution: &VariantArticleExecutionContext,
) {
    let (client, mut first_unit) = match super::super::backends::variant_article_client(
        Some(execution),
        LITERAL_PUBMED_ROUTE,
        "pubmed",
        PubMedClient::new_with_deadline(execution.deadline()),
    )
    .await
    {
        Ok(client) => client,
        Err(_) => return,
    };
    let mut seen = progress.pmids.iter().cloned().collect::<BTreeSet<_>>();
    while !progress.complete && !progress.hard_limit {
        let previous_offset = progress.next_offset;
        execution.add_route_unit(LITERAL_PUBMED_ROUTE, "pubmed");
        let result = super::super::backends::variant_article_request(
            Some(execution),
            LITERAL_PUBMED_ROUTE,
            "pubmed",
            &mut first_unit,
            client.esearch(&PubMedESearchParams {
                term: progress.query.clone(),
                sort: Some(PubMedSearchSort::Relevance),
                retstart: progress.next_offset,
                retmax: super::super::PUBMED_PAGE_SIZE,
                date_from: None,
                date_to: None,
            }),
            |response| {
                progress.total = Some(response.count);
                progress.next_offset += response.idlist.len();
                for pmid in response.idlist {
                    if seen.insert(pmid.clone()) {
                        progress.pmids.push(pmid);
                    }
                }
                progress.hard_limit = response.count > PUBMED_RETRIEVABLE_IDS as u64;
                progress.complete =
                    !progress.hard_limit && progress.next_offset as u64 >= response.count;
                Ok(())
            },
        )
        .await;
        if !matches!(result, Ok(Some(()))) {
            break;
        }
        // An empty page before the reported count is not a completed floor.
        if progress.next_offset == previous_offset || progress.next_offset >= PUBMED_RETRIEVABLE_IDS
        {
            break;
        }
    }
}

async fn hydrate_literal_pubmed(
    candidates: &mut [ArticleCandidate],
    query: &str,
    execution: &VariantArticleExecutionContext,
) {
    if candidates.is_empty() {
        return;
    }
    let query_identity = format!("literal-metadata:{query}");
    let (key, retained) = retained_provider_progress(
        Some(execution),
        LITERAL_PUBMED_ROUTE,
        "pubmed",
        &query_identity,
    );
    let mut hydrated = retained
        .restored_rows()
        .into_iter()
        .map(|row| (row.pmid.clone(), row))
        .collect::<BTreeMap<_, _>>();
    for candidate in candidates.iter_mut() {
        if let Some(row) = hydrated.get(&candidate.row.pmid) {
            candidate.row = row.clone();
        }
    }
    let expected_rows = candidates.len();
    if candidates
        .iter()
        .all(|candidate| !candidate.row.title.is_empty())
    {
        return;
    }
    let (client, mut first_unit) = match super::super::backends::variant_article_client(
        Some(execution),
        LITERAL_PUBMED_ROUTE,
        "pubmed",
        PubMedClient::new_with_deadline(execution.deadline()),
    )
    .await
    {
        Ok(client) => client,
        Err(_) => return,
    };
    for chunk in candidates.chunks_mut(super::super::PUBMED_PAGE_SIZE) {
        let ids = chunk
            .iter()
            .filter(|candidate| candidate.row.title.is_empty())
            .map(|candidate| candidate.row.pmid.clone())
            .collect::<Vec<_>>();
        if ids.is_empty() {
            continue;
        }
        execution.add_route_unit(LITERAL_PUBMED_ROUTE, "pubmed");
        let result = super::super::backends::variant_article_request(
            Some(execution),
            LITERAL_PUBMED_ROUTE,
            "pubmed",
            &mut first_unit,
            client.esummary(&ids),
            |entries| {
                let mut rows = entries
                    .into_iter()
                    .filter_map(|entry| {
                        crate::transform::article::from_pubmed_esummary_entry(&entry)
                    })
                    .map(|row| (row.pmid.clone(), row))
                    .collect::<BTreeMap<_, _>>();
                for candidate in chunk {
                    if let Some(mut row) = rows.remove(&candidate.row.pmid) {
                        row.source_local_position = candidate.row.source_local_position;
                        hydrated.insert(row.pmid.clone(), row.clone());
                        candidate.row = row;
                    }
                }
                Ok(())
            },
        )
        .await;
        let rows = hydrated.values().cloned().collect::<Vec<_>>();
        save_provider_progress(
            Some(execution),
            &key,
            &rows,
            0,
            None,
            Some(expected_rows),
            hydrated.len() >= expected_rows,
        );
        if !matches!(result, Ok(Some(()))) {
            break;
        }
    }
}
