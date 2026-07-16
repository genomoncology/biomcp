#[allow(unused_imports)]
use super::super::test_support::*;
use super::*;

fn count_primary_source(rows: &[ArticleSearchResult], source: ArticleSource) -> usize {
    rows.iter().filter(|row| row.source == source).count()
}

#[test]
fn article_source_pubmed_priority() {
    assert_eq!(article_source_priority(ArticleSource::PubMed), 2);
}

#[test]
fn article_source_litsense2_priority() {
    assert_eq!(article_source_priority(ArticleSource::LitSense2), 4);
}

#[test]
fn merge_preserves_semantic_scholar_identifiers() {
    let mut primary = row_with("1", ArticleSource::EuropePmc, None, None, None);
    primary.doi = Some("10.1000/shared".into());
    let mut semantic = row_with("", ArticleSource::SemanticScholar, None, None, None);
    semantic.doi = Some("10.1000/shared".into());
    semantic.arxiv_id = Some("2401.12345".into());
    semantic.semantic_scholar_id = Some("paper-1".into());

    let merged = merge_article_candidates(vec![primary, semantic]);

    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].row.arxiv_id.as_deref(), Some("2401.12345"));
    assert_eq!(
        merged[0].row.semantic_scholar_id.as_deref(),
        Some("paper-1")
    );
}

#[test]
fn pubmed_unique_row_survives_first_page_in_mixed_federation() {
    // Design: "construct a mixed candidate set with one PubMed-only row that
    // has stronger title-anchor coverage than some competing rows, run it
    // through the common finalizer, and assert that the PubMed-only row
    // survives in the first returned page."
    let mut filters = empty_filters();
    filters.gene = Some("BRAF".into());

    // PubMed-only row with strong title-anchor coverage (gene in title)
    let pubmed_row = ArticleSearchResult {
        pmid: "99999".into(),
        pmcid: None,
        doi: None,
        arxiv_id: None,
        semantic_scholar_id: None,
        title: "BRAF V600E mutations in melanoma".into(),
        journal: Some("Nature".into()),
        date: Some("2025-01-01".into()),
        first_index_date: None,
        citation_count: Some(5),
        influential_citation_count: None,
        source: ArticleSource::PubMed,
        matched_sources: vec![ArticleSource::PubMed],
        score: None,
        is_retracted: Some(false),
        abstract_snippet: None,
        ranking: None,
        normalized_title: "braf v600e mutations in melanoma".into(),
        normalized_abstract: String::new(),
        publication_type: None,
        source_local_position: 0,
    };

    // Competing rows from other backends with weaker title-anchor coverage
    let weak_rows: Vec<ArticleSearchResult> = (1..=5)
        .map(|i| ArticleSearchResult {
            pmid: format!("{i}"),
            pmcid: None,
            doi: None,
            arxiv_id: None,
            semantic_scholar_id: None,
            title: format!("Unrelated oncology study {i}"),
            journal: Some("Journal".into()),
            date: Some("2025-01-01".into()),
            first_index_date: None,
            citation_count: Some(100),
            influential_citation_count: None,
            source: ArticleSource::EuropePmc,
            matched_sources: vec![ArticleSource::EuropePmc],
            score: None,
            is_retracted: Some(false),
            abstract_snippet: None,
            ranking: None,
            normalized_title: format!("unrelated oncology study {i}"),
            normalized_abstract: String::new(),
            publication_type: None,
            source_local_position: 3,
        })
        .collect();

    let mut candidates = weak_rows;
    candidates.push(pubmed_row);

    let page = finalize_article_candidates(candidates, 5, 0, None, &filters);

    assert!(
        page.results.iter().any(|r| r.pmid == "99999"),
        "PubMed-unique row should survive in the first visible page"
    );
    // It should rank high because "BRAF" is in the title
    let pubmed_pos = page
        .results
        .iter()
        .position(|r| r.pmid == "99999")
        .expect("PubMed row must be present");
    assert_eq!(
        pubmed_pos, 0,
        "PubMed row with title-anchor match should rank first among rows without anchor coverage"
    );
}

#[test]
fn finalize_article_candidates_preserves_source_local_position() {
    let mut filters = empty_filters();
    filters.sort = ArticleSort::Date;

    let mut first = row("100", ArticleSource::EuropePmc);
    first.source_local_position = 7;
    first.date = Some("2024-01-01".into());

    let mut second = row("200", ArticleSource::PubMed);
    second.source_local_position = 3;
    second.date = Some("2025-01-01".into());

    let page = finalize_article_candidates(vec![first, second], 10, 0, None, &filters);

    assert_eq!(
        page.results
            .iter()
            .find(|row| row.pmid == "100")
            .expect("first row should remain")
            .source_local_position,
        7
    );
    assert_eq!(
        page.results
            .iter()
            .find(|row| row.pmid == "200")
            .expect("second row should remain")
            .source_local_position,
        3
    );
}

#[test]
fn finalize_article_candidates_default_cap_skips_two_source_pools() {
    let mut filters = empty_filters();
    filters.sort = ArticleSort::Date;

    let mut rows = Vec::new();
    for (idx, pmid) in ["100", "101", "102"].into_iter().enumerate() {
        let mut row = row(pmid, ArticleSource::PubTator);
        row.source_local_position = idx;
        rows.push(row);
    }
    for (idx, pmid) in ["200", "201", "202"].into_iter().enumerate() {
        let mut row = row(pmid, ArticleSource::EuropePmc);
        row.source_local_position = idx;
        rows.push(row);
    }

    let page = finalize_article_candidates(rows, 5, 0, None, &filters);

    assert_eq!(
        page.results.len(),
        5,
        "default capping should not shrink a two-source federated pool"
    );
}

#[test]
fn finalize_article_candidates_default_cap_limits_three_source_pool() {
    let mut filters = empty_filters();
    filters.sort = ArticleSort::Date;

    let mut rows = Vec::new();
    for (idx, pmid) in ["100", "101", "102", "103"].into_iter().enumerate() {
        let mut row = row(pmid, ArticleSource::PubTator);
        row.source_local_position = idx;
        rows.push(row);
    }
    for (idx, pmid) in ["200", "201"].into_iter().enumerate() {
        let mut row = row(pmid, ArticleSource::EuropePmc);
        row.source_local_position = idx;
        rows.push(row);
    }
    let mut pubmed = row("300", ArticleSource::PubMed);
    pubmed.source_local_position = 0;
    rows.push(pubmed);

    let page = finalize_article_candidates(rows, 5, 0, None, &filters);

    assert_eq!(
        count_primary_source(&page.results, ArticleSource::PubTator),
        2,
        "default cap should keep at most floor(40% of limit) rows from one source when three primary sources survive"
    );
}

#[test]
fn finalize_article_candidates_explicit_cap_applies_on_two_source_pools() {
    let mut filters = empty_filters();
    filters.sort = ArticleSort::Date;
    filters.max_per_source = Some(1);

    let mut rows = Vec::new();
    for (idx, pmid) in ["100", "101", "102"].into_iter().enumerate() {
        let mut row = row(pmid, ArticleSource::PubTator);
        row.source_local_position = idx;
        rows.push(row);
    }
    for (idx, pmid) in ["200", "201", "202"].into_iter().enumerate() {
        let mut row = row(pmid, ArticleSource::EuropePmc);
        row.source_local_position = idx;
        rows.push(row);
    }

    let page = finalize_article_candidates(rows, 5, 0, None, &filters);

    assert_eq!(page.results.len(), 2);
    assert_eq!(
        count_primary_source(&page.results, ArticleSource::PubTator),
        1
    );
    assert_eq!(
        count_primary_source(&page.results, ArticleSource::EuropePmc),
        1
    );
}

#[test]
fn finalize_article_candidates_explicit_cap_uses_primary_source_native_position() {
    let mut filters = empty_filters();
    filters.sort = ArticleSort::Date;
    filters.max_per_source = Some(2);

    let mut pubmed_duplicate = row("100", ArticleSource::PubMed);
    pubmed_duplicate.source_local_position = 8;

    let mut europe_duplicate = row("100", ArticleSource::EuropePmc);
    europe_duplicate.source_local_position = 1;

    let mut pubmed_best = row("101", ArticleSource::PubMed);
    pubmed_best.source_local_position = 2;

    let mut pubmed_second = row("102", ArticleSource::PubMed);
    pubmed_second.source_local_position = 4;

    let mut europe = row("201", ArticleSource::EuropePmc);
    europe.source_local_position = 0;

    let mut pubtator = row("301", ArticleSource::PubTator);
    pubtator.source_local_position = 0;

    let page = finalize_article_candidates(
        vec![
            pubmed_duplicate,
            europe_duplicate,
            pubmed_best,
            pubmed_second,
            europe,
            pubtator,
        ],
        10,
        0,
        None,
        &filters,
    );

    let pmids = page
        .results
        .iter()
        .map(|row| row.pmid.as_str())
        .collect::<Vec<_>>();
    assert!(
        pmids.contains(&"101") && pmids.contains(&"102"),
        "the two best PubMed-primary rows should survive the explicit per-source cap"
    );
    assert!(
        !pmids.contains(&"100"),
        "the merged row should be capped by the PubMed primary-source position, not the min merged position"
    );
}

#[test]
fn finalize_article_candidates_explicit_cap_equal_limit_disables_capping() {
    let mut filters = empty_filters();
    filters.sort = ArticleSort::Citations;
    filters.max_per_source = Some(5);

    let mut rows = Vec::new();
    for (idx, (pmid, citations)) in [
        ("100", 1_u64),
        ("101", 2),
        ("102", 3),
        ("103", 4),
        ("104", 5),
        ("105", 500),
    ]
    .into_iter()
    .enumerate()
    {
        let mut row = row(pmid, ArticleSource::PubTator);
        row.source_local_position = idx;
        row.citation_count = Some(citations);
        rows.push(row);
    }

    let mut europe = row("200", ArticleSource::EuropePmc);
    europe.citation_count = Some(10);
    rows.push(europe);

    let mut pubmed = row("300", ArticleSource::PubMed);
    pubmed.citation_count = Some(9);
    rows.push(pubmed);

    let page = finalize_article_candidates(rows, 5, 0, None, &filters);
    let pmids = page
        .results
        .iter()
        .map(|row| row.pmid.as_str())
        .collect::<Vec<_>>();

    assert!(
        pmids.contains(&"105"),
        "setting --max-per-source equal to --limit should disable capping before ranking"
    );
}

#[test]
fn finalize_article_candidates_default_cap_ignores_empty_pmid_rows() {
    let mut filters = empty_filters();
    filters.sort = ArticleSort::Date;

    let mut pubtator_first = row("100", ArticleSource::PubTator);
    pubtator_first.source_local_position = 0;

    let mut pubtator_empty = row("", ArticleSource::PubTator);
    pubtator_empty.title = "title-empty".into();
    pubtator_empty.normalized_title = "title-empty".into();
    pubtator_empty.source_local_position = 1;

    let mut pubtator_second = row("101", ArticleSource::PubTator);
    pubtator_second.source_local_position = 2;

    let mut europe_first = row("200", ArticleSource::EuropePmc);
    europe_first.source_local_position = 0;

    let mut europe_second = row("201", ArticleSource::EuropePmc);
    europe_second.source_local_position = 1;

    let mut pubmed = row("300", ArticleSource::PubMed);
    pubmed.source_local_position = 0;

    let page = finalize_article_candidates(
        vec![
            pubtator_first,
            pubtator_empty,
            pubtator_second,
            europe_first,
            europe_second,
            pubmed,
        ],
        5,
        0,
        None,
        &filters,
    );

    let pmids = page
        .results
        .iter()
        .map(|row| row.pmid.as_str())
        .collect::<Vec<_>>();
    assert!(pmids.contains(&"100"));
    assert!(pmids.contains(&"101"));
    assert!(!pmids.iter().any(|pmid| pmid.trim().is_empty()));
}

#[test]
fn federated_relevance_uses_source_local_position_not_merge_order() {
    let mut filters = empty_filters();
    filters.gene = Some("BRAF".into());
    filters.keyword = Some("melanoma".into());

    let mut europe_first = row("100", ArticleSource::EuropePmc);
    europe_first.title = "BRAF melanoma study".into();
    europe_first.normalized_title = "braf melanoma study".into();
    europe_first.citation_count = Some(5);
    europe_first.source_local_position = 0;

    let mut europe_second = row("200", ArticleSource::EuropePmc);
    europe_second.title = "BRAF melanoma study".into();
    europe_second.normalized_title = "braf melanoma study".into();
    europe_second.citation_count = Some(5);
    europe_second.source_local_position = 1;

    let mut europe_third = row("300", ArticleSource::EuropePmc);
    europe_third.title = "BRAF melanoma study".into();
    europe_third.normalized_title = "braf melanoma study".into();
    europe_third.citation_count = Some(5);
    europe_third.source_local_position = 2;

    let mut pubmed_first = row("900", ArticleSource::PubMed);
    pubmed_first.title = "BRAF melanoma study".into();
    pubmed_first.normalized_title = "braf melanoma study".into();
    pubmed_first.citation_count = Some(5);
    pubmed_first.source_local_position = 0;

    let page = finalize_article_candidates(
        vec![europe_first, europe_second, europe_third, pubmed_first],
        10,
        0,
        None,
        &filters,
    );

    let pubmed_rank = page
        .results
        .iter()
        .position(|row| row.pmid == "900")
        .expect("pubmed row should remain in the ranked output");
    assert!(
        pubmed_rank <= 1,
        "a source-local first PubMed row should rank with other source-local first rows"
    );
}

#[test]
fn merge_article_candidates_dedups_transitively_across_identifiers() {
    let merged = merge_article_candidates(vec![
        ArticleSearchResult {
            pmid: "100".into(),
            pmcid: Some("PMC100".into()),
            doi: None,
            arxiv_id: None,
            semantic_scholar_id: None,
            title: "Primary PMID row".into(),
            journal: Some("Journal".into()),
            date: Some("2025-01-01".into()),
            first_index_date: None,
            citation_count: None,
            influential_citation_count: None,
            source: ArticleSource::PubTator,
            score: Some(42.0),
            is_retracted: None,
            abstract_snippet: None,
            ranking: None,
            matched_sources: vec![ArticleSource::PubTator],
            normalized_title: "primary pmid row".into(),
            normalized_abstract: String::new(),
            publication_type: None,
            source_local_position: 3,
        },
        ArticleSearchResult {
            pmid: String::new(),
            pmcid: Some("PMC100".into()),
            doi: Some("10.1000/example".into()),
            arxiv_id: None,
            semantic_scholar_id: None,
            title: "Europe metadata".into(),
            journal: Some("Journal".into()),
            date: Some("2025-01-01".into()),
            first_index_date: None,
            citation_count: Some(15),
            influential_citation_count: None,
            source: ArticleSource::EuropePmc,
            score: None,
            is_retracted: Some(false),
            abstract_snippet: Some("Europe abstract".into()),
            ranking: None,
            matched_sources: vec![ArticleSource::EuropePmc],
            normalized_title: "europe metadata".into(),
            normalized_abstract: "europe abstract".into(),
            publication_type: Some("Review".into()),
            source_local_position: 1,
        },
        ArticleSearchResult {
            pmid: String::new(),
            pmcid: None,
            doi: Some("10.1000/example".into()),
            arxiv_id: None,
            semantic_scholar_id: None,
            title: "Semantic Scholar metadata".into(),
            journal: Some("Journal".into()),
            date: Some("2025-01-01".into()),
            first_index_date: None,
            citation_count: Some(99),
            influential_citation_count: Some(7),
            source: ArticleSource::SemanticScholar,
            score: None,
            is_retracted: None,
            abstract_snippet: Some("Semantic Scholar abstract".into()),
            ranking: None,
            matched_sources: vec![ArticleSource::SemanticScholar],
            normalized_title: "semantic scholar metadata".into(),
            normalized_abstract: "semantic scholar abstract".into(),
            publication_type: None,
            source_local_position: 2,
        },
    ]);

    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].row.source, ArticleSource::PubTator);
    assert_eq!(merged[0].row.pmid, "100");
    assert_eq!(merged[0].row.pmcid.as_deref(), Some("PMC100"));
    assert_eq!(merged[0].row.doi.as_deref(), Some("10.1000/example"));
    assert_eq!(
        merged[0].row.matched_sources,
        vec![
            ArticleSource::PubTator,
            ArticleSource::EuropePmc,
            ArticleSource::SemanticScholar,
        ]
    );
    assert_eq!(merged[0].row.citation_count, Some(15));
    assert_eq!(merged[0].row.influential_citation_count, Some(7));
    assert_eq!(
        merged[0].row.abstract_snippet.as_deref(),
        Some("Europe abstract")
    );
    assert_eq!(merged[0].row.is_retracted, Some(false));
    assert_eq!(merged[0].row.source_local_position, 1);
}

#[test]
fn merge_article_candidates_keeps_min_source_local_position() {
    let mut europe = row("100", ArticleSource::EuropePmc);
    europe.source_local_position = 3;
    let mut pubmed = row("100", ArticleSource::PubMed);
    pubmed.source_local_position = 1;

    let merged = merge_article_candidates(vec![europe, pubmed]);

    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].row.source_local_position, 1);
    assert_eq!(
        merged[0].row.matched_sources,
        vec![ArticleSource::EuropePmc, ArticleSource::PubMed]
    );
}

#[test]
fn pubmed_led_rescue_preserves_per_source_positions_through_merge() {
    let mut europe = row("100", ArticleSource::EuropePmc);
    europe.source_local_position = 4;
    let mut pubmed = row("100", ArticleSource::PubMed);
    pubmed.source_local_position = 0;
    let mut semantic = row("100", ArticleSource::SemanticScholar);
    semantic.source_local_position = 2;

    let merged = merge_article_candidates(vec![europe, pubmed, semantic]);

    assert_eq!(merged.len(), 1);
    assert_eq!(
        merged[0].source_positions,
        vec![
            ArticleSourcePosition {
                source: ArticleSource::EuropePmc,
                local_position: 4,
            },
            ArticleSourcePosition {
                source: ArticleSource::PubMed,
                local_position: 0,
            },
            ArticleSourcePosition {
                source: ArticleSource::SemanticScholar,
                local_position: 2,
            },
        ]
    );
}
