//! One wall-clock deadline for a whole article search invocation (ticket
//! 1293): candidate legs plus every enrichment pass share a single
//! `VariantArticleDeadline` propagated through the existing task-local seam.
//! These helpers wrap that deadline around source legs and translate its
//! terminal state into `source_status` rows and `SourceUnavailable` errors.

use std::future::Future;
use std::time::Duration;

use super::{
    ArticleSearchTiming, ArticleSource, BioMcpError, FederatedSourceOutcome,
    with_federated_source_timeout,
};

/// One wall-clock ceiling for a whole article search invocation: candidate
/// legs plus every enrichment pass. Matches the 60 s variant-article
/// invocation deadline (`VARIANT_ARTICLE_DEADLINE_LIMIT`), which bounds the
/// same provider mix. The federated legs consume at most their own 12 s
/// timeout, leaving ~48 s for the sequential Semantic Scholar batch and
/// per-row PubTator/Europe PMC metadata fallback — the loops that took the
/// measured 186-206 s commands past three minutes (ticket 1293).
pub(super) const ARTICLE_SEARCH_DEADLINE: Duration = Duration::from_secs(60);

/// The article-search deadline budget. The `BIOMCP_TEST_` override mirrors
/// the citation-graph deadline seam: recorded fixtures run the release-shaped
/// spec binary (no `debug_assertions`), so the hook is read unconditionally,
/// exactly like `BIOMCP_TEST_UNPACED_ORIGIN`.
pub(super) fn article_search_deadline_budget() -> Duration {
    if let Ok(value) = std::env::var("BIOMCP_TEST_ARTICLE_SEARCH_DEADLINE_MS")
        && let Ok(millis) = value.trim().parse::<u64>()
    {
        return Duration::from_millis(millis);
    }
    ARTICLE_SEARCH_DEADLINE
}

/// Run a federated leg under its per-source timeout and record how long it
/// actually took for `--full` diagnostics.
pub(super) async fn timed_source_leg<T, F>(
    source: ArticleSource,
    stage: &'static str,
    future: F,
) -> (FederatedSourceOutcome<T>, ArticleSearchTiming)
where
    F: Future<Output = Result<T, BioMcpError>>,
{
    let started = std::time::Instant::now();
    let outcome = with_federated_source_timeout(source, future).await;
    (
        outcome,
        ArticleSearchTiming {
            source: Some(source),
            stage,
            elapsed_ms: started.elapsed().as_millis() as u64,
        },
    )
}

/// Time a leg without adding a per-source timeout: single-backend plans keep
/// their previous "no leg timeout" contract and are bounded only by the
/// invocation deadline at the HTTP layer.
pub(super) async fn timed_source_call<T, F>(
    source: ArticleSource,
    stage: &'static str,
    future: F,
) -> (Result<T, BioMcpError>, ArticleSearchTiming)
where
    F: Future<Output = Result<T, BioMcpError>>,
{
    let started = std::time::Instant::now();
    let result = future.await;
    (
        result,
        ArticleSearchTiming {
            source: Some(source),
            stage,
            elapsed_ms: started.elapsed().as_millis() as u64,
        },
    )
}

/// The terminal error for a search whose invocation deadline expired without
/// producing a page: unavailable rather than an internal error, so agents see
/// a retryable surface.
pub(super) fn article_search_deadline_error(
    deadline: &crate::sources::VariantArticleDeadline,
) -> BioMcpError {
    BioMcpError::SourceUnavailable {
        source_name: "article search".into(),
        reason: format!(
            "article search deadline exceeded after {}s",
            deadline.limit().as_secs()
        ),
        suggestion: "Retry with a narrower query or a single --source".into(),
    }
}

/// Whether the propagated error is the invocation deadline itself, unwrapping
/// the source-context envelope provider sends add on failure.
pub(super) fn is_search_deadline_error(error: &BioMcpError) -> bool {
    let mut current = error;
    loop {
        match current {
            BioMcpError::WithSourceContext { source, .. } => current = source,
            BioMcpError::HttpMiddleware(reqwest_middleware::Error::Middleware(inner)) => {
                return inner.is::<crate::sources::VariantArticleDeadlineElapsed>();
            }
            // `shared_client` fast-fails an exhausted deadline before any send.
            BioMcpError::Api { message, .. } => {
                return message == "invocation deadline exceeded";
            }
            _ => return false,
        }
    }
}
