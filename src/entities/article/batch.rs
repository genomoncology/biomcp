//! Article batch lookup helpers and compact Semantic Scholar enrichment.

use futures::future::try_join_all;
use tracing::warn;

use crate::error::BioMcpError;
use crate::sources::europepmc::EuropePmcClient;
use crate::sources::pubtator::PubTatorClient;
use crate::sources::semantic_scholar::{SemanticScholarClient, SemanticScholarPaper};

use super::detail::get_article_base_with_clients;
use super::filters::parse_row_date;
use super::{
    ARTICLE_BATCH_MAX_IDS, AnnotationCount, Article, ArticleAnnotations, ArticleBatchEntitySummary,
    ArticleBatchItem,
};

fn trimmed_opt(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn article_batch_title(article: &Article, requested_id: &str) -> String {
    let title = article.title.trim();
    if !title.is_empty() {
        return title.to_string();
    }
    article
        .pmid
        .as_deref()
        .or(article.pmcid.as_deref())
        .or(article.doi.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(requested_id)
        .to_string()
}

fn article_batch_entity_summary(
    annotations: &ArticleAnnotations,
) -> Option<ArticleBatchEntitySummary> {
    fn top_three(rows: &[AnnotationCount]) -> Vec<AnnotationCount> {
        rows.iter().take(3).cloned().collect()
    }

    let summary = ArticleBatchEntitySummary {
        genes: top_three(&annotations.genes),
        diseases: top_three(&annotations.diseases),
        chemicals: top_three(&annotations.chemicals),
        mutations: top_three(&annotations.mutations),
    };

    if summary.genes.is_empty()
        && summary.diseases.is_empty()
        && summary.chemicals.is_empty()
        && summary.mutations.is_empty()
    {
        None
    } else {
        Some(summary)
    }
}

pub(super) fn article_batch_year(article: &Article) -> Option<u32> {
    let normalized = parse_row_date(article.date.as_deref())?;
    normalized.get(..4)?.parse::<u32>().ok()
}

fn article_batch_semantic_scholar_lookup_id(item: &ArticleBatchItem) -> Option<String> {
    item.pmid
        .as_deref()
        .map(|pmid| format!("PMID:{pmid}"))
        .or_else(|| item.doi.as_deref().map(|doi| format!("DOI:{doi}")))
}

pub(super) fn article_batch_item_from_article(
    requested_id: &str,
    article: &Article,
) -> ArticleBatchItem {
    let requested_id = requested_id.trim();
    ArticleBatchItem {
        requested_id: requested_id.to_string(),
        pmid: trimmed_opt(article.pmid.as_deref()),
        pmcid: trimmed_opt(article.pmcid.as_deref()),
        doi: trimmed_opt(article.doi.as_deref()),
        title: article_batch_title(article, requested_id),
        journal: trimmed_opt(article.journal.as_deref()),
        year: article_batch_year(article),
        entity_summary: article
            .annotations
            .as_ref()
            .and_then(article_batch_entity_summary),
        tldr: None,
        citation_count: None,
        influential_citation_count: None,
    }
}

pub(super) fn merge_semantic_scholar_compact_rows(
    items: &mut [ArticleBatchItem],
    item_positions: &[usize],
    rows: Vec<Option<SemanticScholarPaper>>,
) {
    for (idx, paper) in item_positions.iter().zip(rows) {
        let Some(paper) = paper else {
            continue;
        };
        let item = &mut items[*idx];
        item.tldr = paper
            .tldr
            .as_ref()
            .and_then(|value| value.text.as_deref())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        item.citation_count = paper.citation_count;
        item.influential_citation_count = paper.influential_citation_count;
    }
}

async fn enrich_article_batch_with_semantic_scholar(
    items: &mut [ArticleBatchItem],
) -> Result<(), BioMcpError> {
    let client = SemanticScholarClient::new()?;

    let mut lookup_ids = Vec::new();
    let mut item_positions = Vec::new();
    for (idx, item) in items.iter().enumerate() {
        if let Some(lookup_id) = article_batch_semantic_scholar_lookup_id(item) {
            item_positions.push(idx);
            lookup_ids.push(lookup_id);
        }
    }
    if lookup_ids.is_empty() {
        return Ok(());
    }

    match client.paper_batch_compact(&lookup_ids).await {
        Ok(rows) => merge_semantic_scholar_compact_rows(items, &item_positions, rows),
        Err(err) => warn!(?err, "Semantic Scholar batch enrichment failed"),
    }

    Ok(())
}

pub async fn get_batch_compact(ids: &[String]) -> Result<Vec<ArticleBatchItem>, BioMcpError> {
    if ids.len() > ARTICLE_BATCH_MAX_IDS {
        return Err(BioMcpError::InvalidArgument(format!(
            "Article batch is limited to {ARTICLE_BATCH_MAX_IDS} IDs"
        )));
    }

    let pubtator = PubTatorClient::new()?;
    let europe = EuropePmcClient::new()?;
    let articles = try_join_all(
        ids.iter()
            .map(|id| get_article_base_with_clients(id, &pubtator, &europe)),
    )
    .await?;

    let mut items = ids
        .iter()
        .zip(articles.iter())
        .map(|(requested_id, article)| article_batch_item_from_article(requested_id, article))
        .collect::<Vec<_>>();
    enrich_article_batch_with_semantic_scholar(&mut items).await?;
    Ok(items)
}

#[cfg(test)]
mod tests;
