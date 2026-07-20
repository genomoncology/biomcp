use super::super::candidates::finalize_article_candidates;
#[allow(unused_imports)]
use super::super::test_support::*;
use super::*;

mod finalizer;
mod integration;
mod merge;
mod type_capable;

fn semantic_scholar_unavailable_status(message: &str) -> ArticleSourceStatus {
    ArticleSourceStatus {
        source: ArticleSource::SemanticScholar,
        enabled: true,
        auth_mode: None,
        status: Some(ArticleSourceAvailability::Unavailable),
        message: Some(message.to_string()),
    }
}

#[allow(clippy::too_many_arguments)]
fn merge_federated_pages(
    pubtator_leg: Result<SearchPage<ArticleSearchResult>, BioMcpError>,
    europe_leg: Result<SearchPage<ArticleSearchResult>, BioMcpError>,
    pubmed_leg: Option<Result<SearchPage<ArticleSearchResult>, BioMcpError>>,
    semantic_scholar_leg: Result<Vec<ArticleSearchResult>, BioMcpError>,
    litsense2_leg: Result<Vec<ArticleSearchResult>, BioMcpError>,
    limit: usize,
    offset: usize,
    filters: &ArticleSearchFilters,
) -> Result<SearchPage<ArticleSearchResult>, BioMcpError> {
    let semantic_scholar_leg =
        semantic_scholar_leg.map(
            |rows| super::super::backends::SemanticScholarCandidateOutcome {
                rows,
                status: ArticleSourceStatus {
                    source: ArticleSource::SemanticScholar,
                    enabled: true,
                    auth_mode: None,
                    status: Some(ArticleSourceAvailability::Ok),
                    message: None,
                },
            },
        );
    let federated = collect_federated_article_rows(
        match pubtator_leg {
            Ok(page) => FederatedSourceOutcome::Available(page),
            Err(err) => FederatedSourceOutcome::Unavailable {
                error: Some(err),
                status: source_degraded_status(
                    ArticleSource::PubTator,
                    "PubTator3 search unavailable".to_string(),
                ),
            },
        },
        match europe_leg {
            Ok(page) => FederatedSourceOutcome::Available(page),
            Err(err) => FederatedSourceOutcome::Unavailable {
                error: Some(err),
                status: source_degraded_status(
                    ArticleSource::EuropePmc,
                    "Europe PMC search unavailable".to_string(),
                ),
            },
        },
        pubmed_leg.map(|leg| match leg {
            Ok(page) => FederatedSourceOutcome::Available(page),
            Err(err) => FederatedSourceOutcome::Unavailable {
                error: Some(err),
                status: source_degraded_status(
                    ArticleSource::PubMed,
                    "PubMed search unavailable".to_string(),
                ),
            },
        }),
        match semantic_scholar_leg {
            Ok(outcome) => FederatedSourceOutcome::Available(outcome),
            Err(err) => FederatedSourceOutcome::Unavailable {
                error: Some(err),
                status: semantic_scholar_unavailable_status("Semantic Scholar search unavailable"),
            },
        },
        match litsense2_leg {
            Ok(rows) => FederatedSourceOutcome::Available(rows),
            Err(err) => FederatedSourceOutcome::Unavailable {
                error: Some(err),
                status: source_degraded_status(
                    ArticleSource::LitSense2,
                    "LitSense2 search unavailable".to_string(),
                ),
            },
        },
    )?;
    if let Some(error) = federated.primary_error {
        return Err(error);
    }
    Ok(finalize_article_candidates(
        federated.rows,
        limit,
        offset,
        None,
        filters,
    ))
}

#[test]
fn validate_search_page_request_rejects_invalid_inputs_before_backend_io() {
    let filters = empty_filters();
    let err = validate_search_page_request(&filters, 5, ArticleSourceFilter::All)
        .expect_err("queryless article search should fail prevalidation");
    assert!(err.to_string().contains("At least one filter is required"));

    let mut filters = empty_filters();
    filters.keyword = Some("BRAF".into());
    let err = validate_search_page_request(&filters, 0, ArticleSourceFilter::All)
        .expect_err("invalid limit should fail prevalidation");
    assert!(err.to_string().contains("--limit must be between 1 and 50"));
}

#[test]
fn raw_federated_acquisition_marks_known_and_unknown_route_caps() {
    let known_total = FederatedSourceOutcome::Available(SearchPage::offset(
        vec![row("1", ArticleSource::PubTator)],
        Some(2),
    ));
    assert!(page_outcome_truncated(&known_total, 100));

    let unknown_total = FederatedSourceOutcome::Available(SearchPage::offset(
        vec![row("1", ArticleSource::PubTator)],
        None,
    ));
    assert!(page_outcome_truncated(&unknown_total, 1));
    assert!(!page_outcome_truncated(&unknown_total, 2));
}

#[test]
fn raw_federated_acquisition_retains_auxiliary_rows_when_primary_sources_fail() {
    let unavailable = |source| FederatedSourceOutcome::Unavailable {
        error: Some(unavailable_source_error(source)),
        status: source_degraded_status(source, "provider unavailable".into()),
    };
    let semantic_status = ArticleSourceStatus {
        source: ArticleSource::SemanticScholar,
        enabled: true,
        auth_mode: None,
        status: Some(ArticleSourceAvailability::Ok),
        message: None,
    };

    let federated = collect_federated_article_rows(
        unavailable(ArticleSource::PubTator),
        unavailable(ArticleSource::EuropePmc),
        None,
        FederatedSourceOutcome::Available(
            super::super::backends::SemanticScholarCandidateOutcome {
                rows: vec![row("6010008", ArticleSource::SemanticScholar)],
                status: semantic_status,
            },
        ),
        FederatedSourceOutcome::Available(Vec::new()),
    )
    .expect("partial federated rows");

    assert!(federated.primary_error.is_some());
    assert_eq!(federated.rows.len(), 1);
    assert_eq!(federated.rows[0].pmid, "6010008");
}
