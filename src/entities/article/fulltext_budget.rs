//! Bound each full-text route, including the optional work around retrieval.

use std::future::Future;
use std::time::{Duration, Instant};

use crate::error::BioMcpError;

const ROUTE_TIMEOUT: Duration = Duration::from_secs(5);

pub(super) async fn route<T>(
    source: &str,
    future: impl Future<Output = Result<T, BioMcpError>>,
) -> Result<T, BioMcpError> {
    let started = Instant::now();
    let result = tokio::time::timeout(ROUTE_TIMEOUT, future)
        .await
        .unwrap_or_else(|_| {
            Err(BioMcpError::SourceUnavailable {
                source_name: source.into(),
                reason: "full-text route timed out".into(),
                suggestion: "Retry the article request".into(),
            })
        });
    tracing::debug!(source, elapsed_ms = started.elapsed().as_millis(), ok = result.is_ok(), "Article full-text route finished");
    result
}
