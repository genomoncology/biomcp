//! Per-provider work bookkeeping shared by the article search backends.
//!
//! A provider unit ties one provider leg (search page, summary batch,
//! enrichment chunk) to the variant-article execution context so partial
//! work is recorded even when a deadline cancels the leg mid-flight.

use std::time::Instant;

use crate::error::BioMcpError;

use crate::entities::article::variant_search::VariantArticleExecutionContext;

pub(crate) struct VariantArticleProviderUnit<'a> {
    execution: &'a VariantArticleExecutionContext,
    route: String,
    source: String,
    started: Instant,
    _permit: tokio::sync::OwnedSemaphorePermit,
    completed: bool,
}

impl<'a> VariantArticleProviderUnit<'a> {
    pub(crate) fn new(
        execution: &'a VariantArticleExecutionContext,
        route: &str,
        source: &str,
        started: Instant,
        permit: tokio::sync::OwnedSemaphorePermit,
    ) -> Self {
        Self {
            execution,
            route: route.into(),
            source: source.into(),
            started,
            _permit: permit,
            completed: false,
        }
    }

    pub(crate) fn commit<T>(mut self, status: &str, pages: usize, commit: impl FnOnce() -> T) -> T {
        let output = commit();
        self.completed = true;
        self.execution
            .record(&self.route, &self.source, self.started, status, pages);
        output
    }

    pub(crate) fn commit_error<T>(mut self, error: &BioMcpError, commit: impl FnOnce() -> T) -> T {
        let output = commit();
        self.completed = true;
        self.execution
            .record_error(&self.route, &self.source, self.started, error);
        output
    }

    pub(crate) fn record(self, status: &str, pages: usize) {
        self.commit(status, pages, || ());
    }

    pub(crate) fn record_error(self, error: &BioMcpError) {
        self.commit_error(error, || ());
    }
}

impl Drop for VariantArticleProviderUnit<'_> {
    fn drop(&mut self) {
        if !self.completed {
            self.execution
                .record_cancelled(&self.route, &self.source, self.started);
        }
    }
}

pub(crate) async fn variant_article_request<T, U, F, C>(
    execution: Option<&VariantArticleExecutionContext>,
    route: &str,
    source: &str,
    first_unit: &mut Option<VariantArticleProviderUnit<'_>>,
    future: F,
    commit: C,
) -> Result<Option<U>, BioMcpError>
where
    F: std::future::Future<Output = Result<T, BioMcpError>>,
    C: FnOnce(T) -> Result<U, BioMcpError>,
{
    let Some(execution) = execution else {
        return future.await.and_then(commit).map(Some);
    };
    let unit = match first_unit.take() {
        Some(unit) => Some(unit),
        None => execution.begin_provider_unit(route, source).await,
    };
    let Some(unit) = unit else {
        return Ok(None);
    };
    let result = future.await.and_then(commit);
    match &result {
        Ok(_) => unit.record("ok", 1),
        Err(error) => unit.record_error(error),
    }
    result.map(Some)
}

pub(crate) async fn first_variant_article_unit<'a>(
    execution: Option<&'a VariantArticleExecutionContext>,
    route: &str,
    source: &str,
) -> Option<VariantArticleProviderUnit<'a>> {
    match execution {
        Some(execution) => execution.begin_provider_unit(route, source).await,
        None => None,
    }
}

pub(crate) async fn variant_article_client<'a, T, F>(
    execution: Option<&'a VariantArticleExecutionContext>,
    route: &str,
    source: &str,
    future: F,
) -> Result<(T, Option<VariantArticleProviderUnit<'a>>), BioMcpError>
where
    F: std::future::Future<Output = Result<T, BioMcpError>>,
{
    let unit = first_variant_article_unit(execution, route, source).await;
    if execution.is_some() && unit.is_none() {
        return Err(BioMcpError::Api {
            api: source.into(),
            message: "variant article provider work was not admitted".into(),
        });
    }
    match future.await {
        Ok(client) => Ok((client, unit)),
        Err(error) => {
            if let Some(unit) = unit {
                unit.record_error(&error);
            }
            Err(error)
        }
    }
}
