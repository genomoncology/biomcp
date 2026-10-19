//! Recover missing visible abstracts from one bounded PubMed citation batch.

use crate::sources::pubmed::{PUBMED_CITATION_BATCH_MAX_IDS, PubMedClient};

use super::{
    ArticleSearchResult, ArticleSource, ArticleSourceAvailability, ArticleSourceStatus,
    article_search_deadline_status, merge_article_search_row_abstract_text,
    plain_article_search_deadline_elapsed,
};

fn unavailable(message: &str) -> Vec<ArticleSourceStatus> {
    vec![ArticleSourceStatus {
        source: ArticleSource::PubMed,
        enabled: true,
        auth_mode: None,
        status: Some(ArticleSourceAvailability::Degraded),
        message: Some(message.to_string()),
    }]
}

pub(super) async fn enrich(rows: &mut [ArticleSearchResult]) -> Vec<ArticleSourceStatus> {
    let pmids: Vec<String> = rows
        .iter()
        .filter(|row| {
            row.normalized_abstract.trim().is_empty()
                && (row.source == ArticleSource::PubMed
                    || row.matched_sources.contains(&ArticleSource::PubMed))
        })
        .map(|row| row.pmid.clone())
        .collect();
    if pmids.is_empty() {
        return Vec::new();
    }
    if plain_article_search_deadline_elapsed(None) {
        return vec![article_search_deadline_status(
            ArticleSource::PubMed,
            "abstract recovery",
        )];
    }
    let deadline = crate::sources::current_variant_article_deadline();
    let client = match match deadline.as_ref() {
        Some(deadline) => PubMedClient::new_with_deadline(deadline).await,
        None => PubMedClient::new(),
    } {
        Ok(client) => client,
        Err(_) => return unavailable("PubMed abstract recovery unavailable"),
    };
    for chunk in pmids.chunks(PUBMED_CITATION_BATCH_MAX_IDS) {
        if plain_article_search_deadline_elapsed(None) {
            return vec![article_search_deadline_status(
                ArticleSource::PubMed,
                "abstract recovery",
            )];
        }
        let citations = match client.citations(chunk).await {
            Ok(citations) => citations,
            Err(_) => return unavailable("PubMed abstract recovery unavailable"),
        };
        for (pmid, citation) in citations {
            if let Some(text) = citation.abstract_text.as_deref() {
                for row in rows.iter_mut().filter(|row| row.pmid == pmid) {
                    merge_article_search_row_abstract_text(row, text);
                }
            }
        }
    }
    Vec::new()
}
