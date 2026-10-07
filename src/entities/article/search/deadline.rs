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
pub(crate) fn article_search_deadline_budget() -> Duration {
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

/// Prefix of the terminal deadline error's reason. The public error
/// projection keys on it to carry the deadline sentence through to users
/// (ticket 1299).
pub(crate) const ARTICLE_SEARCH_DEADLINE_REASON_PREFIX: &str = "article search deadline exceeded";

/// Suggestion the deadline error pairs with its reason. The public error
/// projection reuses this static sentence when it carries the deadline
/// reason through to users (ticket 1299).
pub(crate) const ARTICLE_SEARCH_DEADLINE_SUGGESTION: &str =
    "Retry with a narrower query or a single --source";

/// Whether an io error is the invocation deadline reporting itself: the
/// cache layer reports deadline cancellation as a `TimedOut` io error with
/// one exact message, so the kind and message are matched together.
fn is_deadline_io_error(error: &std::io::Error) -> bool {
    error.kind() == std::io::ErrorKind::TimedOut
        && error.to_string() == "variant article invocation deadline exceeded"
}

/// The terminal error for a search whose invocation deadline expired without
/// producing a page: unavailable rather than an internal error, so agents see
/// a retryable surface.
pub(in crate::entities::article) fn article_search_deadline_error(
    deadline: &crate::sources::VariantArticleDeadline,
) -> BioMcpError {
    BioMcpError::SourceUnavailable {
        source_name: "article search".into(),
        reason: format!(
            "{ARTICLE_SEARCH_DEADLINE_REASON_PREFIX} after {}s",
            deadline.limit().as_secs()
        ),
        suggestion: ARTICLE_SEARCH_DEADLINE_SUGGESTION.into(),
    }
}

/// Whether the propagated error is the invocation deadline itself, unwrapping
/// the source-context envelope provider sends add on failure.
pub(in crate::entities::article) fn is_search_deadline_error(error: &BioMcpError) -> bool {
    let mut current = error;
    loop {
        match current {
            BioMcpError::WithSourceContext { source, .. } => current = source,
            BioMcpError::HttpMiddleware(reqwest_middleware::Error::Middleware(inner)) => {
                return inner.is::<crate::sources::VariantArticleDeadlineElapsed>()
                    // The cache layer surfaces deadline cancellation as a
                    // boxed `TimedOut` io error inside a middleware error.
                    || inner.chain().any(|cause| {
                        cause
                            .downcast_ref::<std::io::Error>()
                            .is_some_and(is_deadline_io_error)
                    });
            }
            // `shared_client` fast-fails an exhausted deadline before any send.
            BioMcpError::Api { message, .. } => {
                return message == "invocation deadline exceeded";
            }
            // Cache construction reports deadline expiry directly as io.
            BioMcpError::Io(error) => return is_deadline_io_error(error),
            BioMcpError::SourceUnavailable {
                source_name,
                reason,
                ..
            } => {
                return source_name == "article search"
                    && reason.starts_with(ARTICLE_SEARCH_DEADLINE_REASON_PREFIX);
            }
            _ => return false,
        }
    }
}
