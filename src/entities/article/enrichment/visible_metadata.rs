//! Plain search metadata has no need for the generic detail resolver's annotations.

use futures::future::join_all;

use crate::error::BioMcpError;
use crate::sources::europepmc::EuropePmcClient;

use super::{
    ArticleSearchResult, ArticleSource, ArticleSourceAvailability, ArticleSourceStatus,
    article_search_deadline_status, first_europepmc_hit,
    merge_article_search_row_with_article_base, plain_article_search_deadline_elapsed,
};

fn deadline_statuses() -> Vec<ArticleSourceStatus> {
    [ArticleSource::EuropePmc]
        .into_iter()
        .map(|source| article_search_deadline_status(source, "article metadata fallback"))
        .collect()
}

fn failure_status(
    source: ArticleSource,
    error: &BioMcpError,
    operation: &'static str,
) -> ArticleSourceStatus {
    let provider = match source {
        ArticleSource::PubTator => crate::error::SourceProvider::PUBTATOR3,
        _ => crate::error::SourceProvider::EUROPE_PMC,
    };
    let failure = error.external_failure(provider, operation);
    let http_status = failure
        .status
        .map(|status| format!(" (HTTP {status})"))
        .unwrap_or_default();
    let mut message = format!("{operation}: {}{http_status}", failure.message);
    if let Some(failure) = crate::sources::literature_retry::failure(error) {
        message.push_str(&format!(": {failure}"));
    }
    ArticleSourceStatus {
        source,
        enabled: true,
        auth_mode: None,
        status: Some(ArticleSourceAvailability::Degraded),
        message: Some(message),
    }
}

pub(super) async fn enrich(
    rows: &mut [ArticleSearchResult],
    positions: Vec<(usize, u64)>,
) -> Vec<ArticleSourceStatus> {
    if plain_article_search_deadline_elapsed(None) {
        return deadline_statuses();
    }
    let deadline = crate::sources::current_variant_article_deadline();
    let europe = match match deadline.as_ref() {
        Some(deadline) => EuropePmcClient::new_with_deadline(deadline).await,
        None => EuropePmcClient::new(),
    } {
        Ok(client) => client.with_abstracts(true),
        Err(err) => {
            crate::error::warn_external_failure(
                &err,
                crate::error::SourceProvider::EUROPE_PMC,
                "initialize visible article metadata fallback",
            );
            return vec![failure_status(
                ArticleSource::EuropePmc,
                &err,
                "initialize visible article metadata fallback",
            )];
        }
    };

    // Plain result metadata needs abstracts and citation counts, not annotations.
    // Core Europe PMC records supply both without an export for unindexed PMIDs.
    let recovered = join_all(positions.iter().map(|(_, pmid)| async {
        if plain_article_search_deadline_elapsed(None) {
            return Ok(None);
        }
        europe
            .search_by_pmid(&pmid.to_string())
            .await
            .map(first_europepmc_hit)
    }))
    .await;
    let mut statuses = Vec::new();
    for ((index, _), recovery) in positions.into_iter().zip(recovered) {
        match recovery {
            Ok(Some(hit)) => merge_article_search_row_with_article_base(
                &mut rows[index],
                &crate::transform::article::from_europepmc_result(&hit),
            ),
            Ok(None) => {}
            Err(error) => statuses.push(failure_status(
                ArticleSource::EuropePmc,
                &error,
                "visible article metadata fallback",
            )),
        }
    }
    if plain_article_search_deadline_elapsed(None) {
        statuses.extend(deadline_statuses());
    }
    statuses.dedup_by(|right, left| right.source == left.source && right.message == left.message);
    statuses
}
