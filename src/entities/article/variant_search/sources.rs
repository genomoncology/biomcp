//! Membership acquisition retains identifiers before optional hydration.

use serde::{Deserialize, Serialize};

use super::*;
use crate::sources::pubmed::{PubMedClient, PubMedESearchParams, PubMedSearchSort};

const LITERAL_PUBMED_ROUTE: &str = "literal_pubmed";
const PUBMED_RETRIEVABLE_IDS: usize = 10_000;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(super) struct LiteralPubMedProgress {
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
    hydrate_literal_pubmed(&mut candidates, execution).await;
    let events = execution.events();
    let mut status = provider_statuses_for_route(LITERAL_PUBMED_ROUTE, &events);
    if !progress.complete {
        let mut unfinished = status_with_detail(
            LITERAL_PUBMED_ROUTE,
            "pubmed",
            VariantArticleSourceStatusKind::Degraded,
            Some(if progress.hard_limit {
                "PubMed's 10000-ID retrieval limit prevents a complete literal query floor"
            } else {
                "literal PubMed enumeration is unfinished; retained PMIDs remain available"
            }),
        );
        unfinished.reason_codes = vec![if progress.hard_limit {
            "query_result_cap"
        } else {
            "not_attempted"
        }];
        status.push(unfinished);
    }
    SourceAcquisition {
        candidates,
        status,
        progress: Some(progress),
    }
}

async fn enumerate_literal_pubmed(
    progress: &mut LiteralPubMedProgress,
    execution: &VariantArticleExecutionContext,
) {
    let client = match PubMedClient::new_with_deadline(execution.deadline()).await {
        Ok(client) => client,
        Err(_) => return,
    };
    let mut seen = progress.pmids.iter().cloned().collect::<BTreeSet<_>>();
    while !progress.complete && !progress.hard_limit {
        let previous_offset = progress.next_offset;
        execution.add_route_unit(LITERAL_PUBMED_ROUTE, "pubmed");
        let mut first_unit = None;
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
    execution: &VariantArticleExecutionContext,
) {
    let client = match PubMedClient::new_with_deadline(execution.deadline()).await {
        Ok(client) => client,
        Err(_) => return,
    };
    for chunk in candidates.chunks_mut(super::super::PUBMED_PAGE_SIZE) {
        let ids = chunk
            .iter()
            .map(|candidate| candidate.row.pmid.clone())
            .collect::<Vec<_>>();
        execution.add_route_unit(LITERAL_PUBMED_ROUTE, "pubmed");
        let mut first_unit = None;
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
                        candidate.row = row;
                    }
                }
                Ok(())
            },
        )
        .await;
        if !matches!(result, Ok(Some(()))) {
            break;
        }
    }
}
