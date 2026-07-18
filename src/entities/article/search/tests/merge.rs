use super::*;

#[test]
fn merge_federated_pages_dedups_with_pubtator_priority() {
    let pubtator_page = SearchPage::offset(
        vec![
            row("100", ArticleSource::PubTator),
            row("200", ArticleSource::PubTator),
        ],
        Some(2),
    );
    let europe_page = SearchPage::offset(
        vec![
            row("200", ArticleSource::EuropePmc),
            row("300", ArticleSource::EuropePmc),
        ],
        Some(2),
    );

    let merged = merge_federated_pages(
        Ok(pubtator_page),
        Ok(europe_page),
        None,
        Ok(Vec::new()),
        Ok(Vec::new()),
        3,
        0,
        &empty_filters(),
    )
    .expect("federated merge should succeed");
    assert_eq!(merged.results.len(), 3);
    assert_eq!(merged.results[0].pmid, "100");
    assert_eq!(merged.results[1].pmid, "200");
    assert_eq!(merged.results[2].pmid, "300");
    assert_eq!(merged.results[1].source, ArticleSource::PubTator);
    assert_eq!(merged.total, None);
}

#[test]
fn merge_federated_pages_records_litsense2_in_matched_sources() {
    let pubtator_page = SearchPage::offset(vec![row("100", ArticleSource::PubTator)], Some(1));
    let europe_page = SearchPage::offset(Vec::new(), Some(0));
    let litsense2_rows = vec![row("100", ArticleSource::LitSense2)];

    let merged = merge_federated_pages(
        Ok(pubtator_page),
        Ok(europe_page),
        None,
        Ok(Vec::new()),
        Ok(litsense2_rows),
        10,
        0,
        &empty_filters(),
    )
    .expect("federated merge should succeed");

    assert_eq!(merged.results.len(), 1);
    assert_eq!(merged.results[0].source, ArticleSource::PubTator);
    assert_eq!(
        merged.results[0].matched_sources,
        vec![ArticleSource::PubTator, ArticleSource::LitSense2]
    );
}

#[test]
fn merge_federated_pages_returns_surviving_pubtator_leg() {
    let pubtator_page = SearchPage::offset(
        vec![
            row("100", ArticleSource::PubTator),
            row("200", ArticleSource::PubTator),
        ],
        Some(50),
    );
    let europe_err = BioMcpError::Api {
        api: "europepmc".into(),
        message: "HTTP 500: upstream".into(),
    };

    let merged = merge_federated_pages(
        Ok(pubtator_page),
        Err(europe_err),
        None,
        Ok(Vec::new()),
        Ok(Vec::new()),
        2,
        0,
        &empty_filters(),
    )
    .expect("fallback should return pubtator rows");
    assert_eq!(merged.results.len(), 2);
    assert!(
        merged
            .results
            .iter()
            .all(|r| r.source == ArticleSource::PubTator)
    );
    assert_eq!(merged.total, None);
}

#[test]
fn merge_federated_pages_returns_surviving_europe_leg() {
    let pubtator_err = BioMcpError::Api {
        api: "pubtator3".into(),
        message: "HTTP 500: upstream".into(),
    };
    let europe_page = SearchPage::offset(
        vec![
            row("100", ArticleSource::EuropePmc),
            row("200", ArticleSource::EuropePmc),
            row("300", ArticleSource::EuropePmc),
        ],
        Some(50),
    );

    let merged = merge_federated_pages(
        Err(pubtator_err),
        Ok(europe_page),
        None,
        Ok(Vec::new()),
        Ok(Vec::new()),
        2,
        0,
        &empty_filters(),
    )
    .expect("fallback should return europe rows");
    assert_eq!(merged.results.len(), 2);
    assert!(
        merged
            .results
            .iter()
            .all(|r| r.source == ArticleSource::EuropePmc)
    );
    assert_eq!(merged.total, None);
}

#[test]
fn merge_federated_pages_sorts_surviving_leg_before_offset() {
    let pubtator_err = BioMcpError::Api {
        api: "pubtator3".into(),
        message: "HTTP 500: upstream".into(),
    };
    let europe_page = SearchPage::offset(
        vec![
            row_with(
                "100",
                ArticleSource::EuropePmc,
                Some("2024-01-01"),
                Some(1),
                Some(false),
            ),
            row_with(
                "200",
                ArticleSource::EuropePmc,
                Some("2025-01-01"),
                Some(1),
                Some(false),
            ),
            row_with(
                "300",
                ArticleSource::EuropePmc,
                Some("2023-01-01"),
                Some(1),
                Some(false),
            ),
        ],
        Some(3),
    );

    let merged = merge_federated_pages(
        Err(pubtator_err),
        Ok(europe_page),
        None,
        Ok(Vec::new()),
        Ok(Vec::new()),
        1,
        1,
        &ArticleSearchFilters {
            sort: ArticleSort::Date,
            ..empty_filters()
        },
    )
    .expect("fallback should sort surviving rows before offset");
    assert_eq!(merged.results.len(), 1);
    assert_eq!(merged.results[0].pmid, "100");
}

#[test]
fn merge_federated_pages_returns_first_error_when_both_fail() {
    let pubtator_err = BioMcpError::Api {
        api: "pubtator3".into(),
        message: "HTTP 500: pubtator failed".into(),
    };
    let europe_err = BioMcpError::Api {
        api: "europepmc".into(),
        message: "HTTP 500: europe failed".into(),
    };

    let err = merge_federated_pages(
        Err(pubtator_err),
        Err(europe_err),
        None,
        Ok(Vec::new()),
        Ok(Vec::new()),
        10,
        0,
        &empty_filters(),
    )
    .expect_err("both failing legs should return first error");
    let msg = format!("{err:?}");
    assert!(msg.contains("pubtator"));
}

#[test]
fn federated_offset_applied_after_merge_not_per_leg() {
    let pubtator_page = SearchPage::offset(
        vec![
            row("100", ArticleSource::PubTator),
            row("200", ArticleSource::PubTator),
            row("300", ArticleSource::PubTator),
            row("400", ArticleSource::PubTator),
            row("500", ArticleSource::PubTator),
        ],
        Some(5),
    );
    let europe_page = SearchPage::offset(
        vec![
            row("600", ArticleSource::EuropePmc),
            row("700", ArticleSource::EuropePmc),
        ],
        Some(2),
    );

    let merged = merge_federated_pages(
        Ok(pubtator_page),
        Ok(europe_page),
        None,
        Ok(Vec::new()),
        Ok(Vec::new()),
        2,
        3,
        &empty_filters(),
    )
    .expect("federated merge should succeed");

    let pmids: Vec<&str> = merged.results.iter().map(|row| row.pmid.as_str()).collect();
    assert_eq!(pmids, vec!["400", "500"]);
}

#[test]
fn federated_sort_orders_merged_results_for_citations_and_date() {
    let citation_pubtator_page = SearchPage::offset(
        vec![
            row_with(
                "100",
                ArticleSource::PubTator,
                Some("2025-02-01"),
                Some(50),
                Some(false),
            ),
            row_with(
                "200",
                ArticleSource::PubTator,
                Some("2024-01-01"),
                Some(5),
                Some(false),
            ),
        ],
        Some(2),
    );
    let citation_europe_page = SearchPage::offset(
        vec![
            row_with(
                "300",
                ArticleSource::EuropePmc,
                Some("2025-03-01"),
                Some(100),
                Some(false),
            ),
            row_with(
                "400",
                ArticleSource::EuropePmc,
                Some("2024-06-01"),
                Some(10),
                Some(false),
            ),
        ],
        Some(2),
    );

    let citation_merged = merge_federated_pages(
        Ok(citation_pubtator_page),
        Ok(citation_europe_page),
        None,
        Ok(Vec::new()),
        Ok(Vec::new()),
        10,
        0,
        &ArticleSearchFilters {
            sort: ArticleSort::Citations,
            ..empty_filters()
        },
    )
    .expect("citation merge should succeed");
    let citation_pmids: Vec<&str> = citation_merged
        .results
        .iter()
        .map(|row| row.pmid.as_str())
        .collect();
    assert_eq!(citation_pmids, vec!["300", "100", "400", "200"]);

    let date_pubtator_page = SearchPage::offset(
        vec![
            row_with(
                "500",
                ArticleSource::PubTator,
                Some("2025"),
                Some(25),
                Some(false),
            ),
            row_with(
                "600",
                ArticleSource::PubTator,
                Some("2024-12-31"),
                Some(30),
                Some(false),
            ),
        ],
        Some(2),
    );
    let date_europe_page = SearchPage::offset(
        vec![
            row_with(
                "700",
                ArticleSource::EuropePmc,
                Some("2025-06-01"),
                Some(10),
                Some(false),
            ),
            row_with("800", ArticleSource::EuropePmc, None, Some(99), Some(false)),
        ],
        Some(2),
    );

    let date_merged = merge_federated_pages(
        Ok(date_pubtator_page),
        Ok(date_europe_page),
        None,
        Ok(Vec::new()),
        Ok(Vec::new()),
        10,
        0,
        &ArticleSearchFilters {
            sort: ArticleSort::Date,
            ..empty_filters()
        },
    )
    .expect("date merge should succeed");
    let date_pmids: Vec<&str> = date_merged
        .results
        .iter()
        .map(|row| row.pmid.as_str())
        .collect();
    assert_eq!(date_pmids, vec!["700", "500", "600", "800"]);
}

#[test]
fn merge_federated_pages_preserves_known_retraction_status_from_later_duplicate() {
    let pubtator_page = SearchPage::offset(
        vec![row_with(
            "200",
            ArticleSource::PubTator,
            Some("2025-01-01"),
            Some(1),
            None,
        )],
        Some(1),
    );
    let europe_page = SearchPage::offset(
        vec![row_with(
            "200",
            ArticleSource::EuropePmc,
            Some("2025-01-01"),
            Some(10),
            Some(true),
        )],
        Some(1),
    );

    let merged = merge_federated_pages(
        Ok(pubtator_page),
        Ok(europe_page),
        None,
        Ok(Vec::new()),
        Ok(Vec::new()),
        10,
        0,
        &empty_filters(),
    )
    .expect("federated merge should succeed");

    assert_eq!(merged.results.len(), 1);
    assert_eq!(merged.results[0].source, ArticleSource::PubTator);
    assert_eq!(merged.results[0].is_retracted, Some(true));
}
