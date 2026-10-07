use super::*;

#[test]
fn federated_merge_keeps_non_europepmc_matches_under_default_retraction_filter() {
    let pubtator_page = PartialSearchPage {
        page: SearchPage::offset(vec![row("22663011", ArticleSource::PubTator)], Some(1)),
        degradation: None,
    };
    let europe_page = PartialSearchPage {
        page: SearchPage::offset(
            vec![row_with(
                "22663012",
                ArticleSource::EuropePmc,
                Some("2024-01-01"),
                Some(25),
                Some(false),
            )],
            Some(1),
        ),
        degradation: None,
    };
    let filters = ArticleSearchFilters {
        keyword: Some("alternative microexon splicing metastasis".into()),
        exclude_retracted: true,
        ..empty_filters()
    };

    let merged = merge_federated_pages(
        Ok(pubtator_page),
        Ok(europe_page),
        None,
        Ok(Vec::new()),
        Ok(Vec::new()),
        5,
        0,
        &filters,
    )
    .expect("federated merge should keep surviving source rows");

    assert!(!merged.results.is_empty());
    assert!(merged.results.iter().any(|row| {
        row.source == ArticleSource::PubTator
            || row.matched_sources.contains(&ArticleSource::PubTator)
    }));
}

#[test]
fn federated_merge_includes_pubmed_rows_in_matched_sources() {
    let pubtator_page = PartialSearchPage {
        page: SearchPage::offset(vec![row("22663011", ArticleSource::PubTator)], Some(1)),
        degradation: None,
    };
    let europe_page = PartialSearchPage {
        page: SearchPage::offset(vec![row("22663012", ArticleSource::EuropePmc)], Some(1)),
        degradation: None,
    };
    let pubmed_page = PartialSearchPage {
        page: SearchPage::offset(vec![row("22663013", ArticleSource::PubMed)], Some(1)),
        degradation: None,
    };
    let filters = ArticleSearchFilters {
        keyword: Some("alternative microexon splicing metastasis".into()),
        exclude_retracted: true,
        ..empty_filters()
    };

    let merged = merge_federated_pages(
        Ok(pubtator_page),
        Ok(europe_page),
        Some(Ok(pubmed_page)),
        Ok(Vec::new()),
        Ok(Vec::new()),
        5,
        0,
        &filters,
    )
    .expect("federated merge should include PubMed rows");

    assert!(merged.results.iter().any(|row| {
        row.source == ArticleSource::PubMed || row.matched_sources.contains(&ArticleSource::PubMed)
    }));
}
