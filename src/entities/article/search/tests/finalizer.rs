use super::*;

#[test]
fn pubmed_only_rows_use_common_finalizer_for_sorting() {
    let mut filters = empty_filters();
    filters.gene = Some("BRAF".into());
    filters.sort = ArticleSort::Date;
    let rows = vec![
        row_with(
            "1",
            ArticleSource::PubMed,
            Some("2024-01-01"),
            Some(1),
            Some(false),
        ),
        row_with(
            "2",
            ArticleSource::PubMed,
            Some("2025-02-01"),
            Some(1),
            Some(false),
        ),
    ];

    let page = finalize_article_candidates(rows, 2, 0, Some(2), &filters);

    let pmids: Vec<&str> = page.results.iter().map(|row| row.pmid.as_str()).collect();
    assert_eq!(pmids, vec!["2", "1"]);
}

#[test]
fn semantic_scholar_status_tracker_keeps_batch_failure_non_fatal() {
    let mut tracker = SemanticScholarStatusTracker::default();
    tracker.record(ArticleSourceStatus {
        source: ArticleSource::SemanticScholar,
        enabled: true,
        auth_mode: None,
        status: Some(ArticleSourceAvailability::Unavailable),
        message: Some("Semantic Scholar enrichment unavailable".into()),
    });

    let status = tracker.finish();

    assert_eq!(status.len(), 1);
    assert_eq!(status[0].source, ArticleSource::SemanticScholar);
    assert_eq!(
        status[0].status,
        Some(ArticleSourceAvailability::Unavailable)
    );
    assert_eq!(
        status[0].message.as_deref(),
        Some("Semantic Scholar enrichment unavailable")
    );
}

#[test]
fn federated_collection_keeps_available_rows_when_semantic_scholar_is_unavailable() {
    let pubtator_page = PartialSearchPage {
        page: SearchPage::offset(vec![row("22663011", ArticleSource::PubTator)], Some(1)),
        degradation: None,
    };
    let europe_page = PartialSearchPage {
        page: SearchPage::offset(Vec::new(), Some(0)),
        degradation: None,
    };
    let semantic_status = ArticleSourceStatus {
        source: ArticleSource::SemanticScholar,
        enabled: true,
        auth_mode: None,
        status: Some(ArticleSourceAvailability::Unavailable),
        message: Some("Semantic Scholar search unavailable".into()),
    };

    let federated = collect_federated_article_rows(
        FederatedSourceOutcome::Available(pubtator_page),
        FederatedSourceOutcome::Available(europe_page),
        None,
        FederatedSourceOutcome::Unavailable {
            error: None,
            status: semantic_status.clone(),
        },
        FederatedSourceOutcome::Available(Vec::new()),
    )
    .expect("available primary source rows should survive semantic scholar failure");

    assert_eq!(federated.rows.len(), 1);
    assert_eq!(federated.rows[0].source, ArticleSource::PubTator);
    assert_eq!(federated.semantic_scholar_status, semantic_status);
}
