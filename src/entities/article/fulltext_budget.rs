//! Bound each full-text route, including the optional work around retrieval.

use std::future::Future;
use std::time::{Duration, Instant};

use crate::error::BioMcpError;

pub(crate) const FAILURE_PREFIX: &str = "Article full-text retrieval:";

const ROUTE_TIMEOUT: Duration = Duration::from_secs(5);

pub(super) async fn route<T>(
    source: &str,
    future: impl Future<Output = Result<T, BioMcpError>>,
) -> Result<T, BioMcpError> {
    let started = Instant::now();
    let observation = crate::sources::fulltext_diagnostics::RequestObservation::default();
    let result = crate::sources::fulltext_diagnostics::CURRENT_REQUEST
        .scope(
            observation.clone(),
            tokio::time::timeout(ROUTE_TIMEOUT, future),
        )
        .await
        .map(|result| result.map_err(|error| observation.annotate(error)))
        .unwrap_or_else(|_| {
            Err(BioMcpError::SourceUnavailable {
                source_name: source.into(),
                reason: format!(
                    "{FAILURE_PREFIX} {source} timed out.{}",
                    observation.timeout_detail()
                ),
                suggestion: "Retry the article request".into(),
            })
        });
    tracing::debug!(
        source,
        elapsed_ms = started.elapsed().as_millis(),
        ok = result.is_ok(),
        "Article full-text route finished"
    );
    result
}
