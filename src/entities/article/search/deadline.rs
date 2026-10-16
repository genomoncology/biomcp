//! One wall-clock deadline for a whole article search invocation: candidate legs plus every enrichment pass share a single
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
/// measured 186-206 s commands past three minutes.
pub(super) const ARTICLE_SEARCH_DEADLINE: Duration = Duration::from_secs(60);

/// The article-search deadline budget. The `BIOMCP_TEST_` override mirrors
/// the citation-graph deadline seam: recorded fixtures also run the release
/// binary under `make verify` (no `debug_assertions`), so the hook is read
/// unconditionally, exactly like `BIOMCP_TEST_UNPACED_ORIGIN`.
pub(crate) fn article_search_deadline_budget() -> Duration {
    article_search_deadline_budget_from(
        std::env::var("BIOMCP_TEST_ARTICLE_SEARCH_DEADLINE_MS")
            .ok()
            .as_deref(),
    )
}

/// The injected-clock core of the deadline budget seam: a recorded fixture
/// narrows the deadline to milliseconds without waiting on the wall clock.
pub(crate) fn article_search_deadline_budget_from(override_value: Option<&str>) -> Duration {
    if let Some(value) = override_value
        && let Ok(millis) = value.trim().parse::<u64>()
    {
        return Duration::from_millis(millis);
    }
    ARTICLE_SEARCH_DEADLINE
}

/// The federated per-source leg timeout with the same injected-clock seam:
/// a recorded fixture that holds its reply past the leg still proves the
/// timeout, at milliseconds of wall time instead of the full budget.
/// `BIOMCP_TEST_FEDERATED_ARTICLE_SOURCE_TIMEOUT_MS` is read unconditionally,
/// exactly like the deadline seam, so release-profile lanes can narrow it too.
pub(super) fn federated_article_source_timeout() -> Duration {
    federated_article_source_timeout_from(
        std::env::var("BIOMCP_TEST_FEDERATED_ARTICLE_SOURCE_TIMEOUT_MS")
            .ok()
            .as_deref(),
    )
}

pub(super) fn federated_article_source_timeout_from(override_value: Option<&str>) -> Duration {
    if let Some(value) = override_value
        && let Ok(millis) = value.trim().parse::<u64>()
    {
        return Duration::from_millis(millis);
    }
    super::FEDERATED_ARTICLE_SOURCE_TIMEOUT
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
///.
pub(crate) const ARTICLE_SEARCH_DEADLINE_REASON_PREFIX: &str = "article search deadline exceeded";

/// Suggestion the deadline error pairs with its reason. The public error
/// projection reuses this static sentence when it carries the deadline
/// reason through to users.
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
pub(super) fn article_search_deadline_error(
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
            _ => return false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deadline_budget_reads_the_injected_clock() {
        assert_eq!(
            article_search_deadline_budget_from(Some("8000")),
            Duration::from_secs(8)
        );
        assert_eq!(
            article_search_deadline_budget_from(Some(" 250 ")),
            Duration::from_millis(250)
        );
        assert_eq!(
            article_search_deadline_budget_from(Some("not-a-number")),
            ARTICLE_SEARCH_DEADLINE
        );
        assert_eq!(
            article_search_deadline_budget_from(None),
            ARTICLE_SEARCH_DEADLINE
        );
    }

    #[test]
    fn federated_leg_timeout_reads_the_injected_clock() {
        assert_eq!(
            federated_article_source_timeout_from(Some("500")),
            Duration::from_millis(500)
        );
        assert_eq!(
            federated_article_source_timeout_from(Some("")),
            super::super::FEDERATED_ARTICLE_SOURCE_TIMEOUT
        );
        assert_eq!(
            federated_article_source_timeout_from(None),
            super::super::FEDERATED_ARTICLE_SOURCE_TIMEOUT
        );
    }
}
