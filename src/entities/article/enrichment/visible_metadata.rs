//! Plain search metadata has no need for the generic detail resolver's annotations.

use futures::future::join_all;

use crate::error::BioMcpError;
use crate::sources::europepmc::EuropePmcClient;
use crate::sources::pubtator::PubTatorClient;

use super::{
    ArticleSearchResult, ArticleSource, ArticleSourceStatus, article_search_deadline_status,
    first_europepmc_hit, merge_article_search_row_with_article_base,
    plain_article_search_deadline_elapsed, settle_visible_row_metadata_fallback,
};

fn deadline_statuses() -> Vec<ArticleSourceStatus> {
    [ArticleSource::PubTator, ArticleSource::EuropePmc]
        .into_iter()
        .map(|source| article_search_deadline_status(source, "article metadata fallback"))
        .collect()
}

pub(super) async fn enrich(
    rows: &mut [ArticleSearchResult],
    positions: Vec<(usize, u64)>,
) -> Vec<ArticleSourceStatus> {
    if plain_article_search_deadline_elapsed(None) {
        return deadline_statuses();
    }
    let deadline = crate::sources::current_variant_article_deadline();
    let pubtator = match match deadline.as_ref() {
        Some(deadline) => PubTatorClient::new_with_deadline(deadline).await,
        None => PubTatorClient::new(),
    } {
        Ok(client) => client,
        Err(err) => {
            crate::error::warn_external_failure(
                &err,
                crate::error::SourceProvider::PUBTATOR3,
                "initialize visible article metadata fallback",
            );
            return Vec::new();
        }
    };
    let europe = match match deadline.as_ref() {
        Some(deadline) => EuropePmcClient::new_with_deadline(deadline).await,
        None => EuropePmcClient::new(),
    } {
        Ok(client) => client,
        Err(err) => {
            crate::error::warn_external_failure(
                &err,
                crate::error::SourceProvider::EUROPE_PMC,
                "initialize visible article metadata fallback",
            );
            return Vec::new();
        }
    };

    let mut collected = Vec::with_capacity(positions.len());
    for (index, pmid) in positions {
        if plain_article_search_deadline_elapsed(None) {
            break;
        }
        let article = pubtator.export_biocjson(pmid).await.and_then(|reply| {
            reply
                .documents
                .into_iter()
                .next()
                .map(|doc| crate::transform::article::from_pubtator_document(&doc))
                .ok_or_else(|| BioMcpError::NotFound {
                    entity: "article".into(),
                    id: pmid.to_string(),
                    suggestion: "Try another article identifier".into(),
                })
        });
        collected.push((index, pmid, article));
    }

    // Poll all recoveries together, rather than calling a resolver which
    // already recovered and then repeating that recovery on its error.
    let recovered = join_all(collected.iter().map(|(_, pmid, _)| async {
        if plain_article_search_deadline_elapsed(None) {
            return None;
        }
        europe
            .search_by_pmid(&pmid.to_string())
            .await
            .ok()
            .and_then(first_europepmc_hit)
    }))
    .await;
    for ((index, _, article), hit) in collected.into_iter().zip(recovered) {
        match article {
            Ok(mut article) => {
                if let Some(hit) = hit {
                    crate::transform::article::merge_europepmc_metadata(&mut article, &hit);
                }
                merge_article_search_row_with_article_base(&mut rows[index], &article);
            }
            Err(error) => {
                settle_visible_row_metadata_fallback(&mut rows[index], &error, hit.as_ref());
            }
        }
    }
    if plain_article_search_deadline_elapsed(None) {
        deadline_statuses()
    } else {
        Vec::new()
    }
}
