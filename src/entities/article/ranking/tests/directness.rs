use super::*;

#[test]
fn directness_ranking_uses_full_title_and_token_boundaries() {
    let mut filters = empty_filters();
    filters.gene = Some("MET".into());
    filters.keyword = Some("ALL".into());

    let long_prefix =
        "This intentionally long prefix exists to push the anchors well past sixty bytes";
    let mut rows = vec![
        ArticleSearchResult {
            pmid: "100".into(),
            pmcid: None,
            doi: None,
            arxiv_id: None,
            semantic_scholar_id: None,
            title: format!("{long_prefix} MET ALL response study"),
            journal: Some("Journal A".into()),
            date: Some("2025-01-01".into()),
            first_index_date: None,
            citation_count: Some(10),
            influential_citation_count: Some(1),
            source: ArticleSource::EuropePmc,
            score: None,
            is_retracted: Some(false),
            abstract_snippet: Some("Direct abstract".into()),
            ranking: None,
            matched_sources: vec![ArticleSource::EuropePmc],
            normalized_title: format!(
                "{} met all response study",
                long_prefix.to_ascii_lowercase()
            ),
            normalized_abstract: "direct abstract".into(),
            publication_type: None,
            source_local_position: 0,
        },
        ArticleSearchResult {
            pmid: "200".into(),
            pmcid: None,
            doi: None,
            arxiv_id: None,
            semantic_scholar_id: None,
            title: "Meta-analysis of small molecule therapy".into(),
            journal: Some("Journal B".into()),
            date: Some("2025-01-01".into()),
            first_index_date: None,
            citation_count: Some(500),
            influential_citation_count: Some(50),
            source: ArticleSource::EuropePmc,
            score: None,
            is_retracted: Some(false),
            abstract_snippet: None,
            ranking: None,
            matched_sources: vec![ArticleSource::EuropePmc],
            normalized_title: "meta-analysis of small molecule therapy".into(),
            normalized_abstract: String::new(),
            publication_type: Some("Meta-Analysis".into()),
            source_local_position: 1,
        },
        ArticleSearchResult {
            pmid: "300".into(),
            pmcid: None,
            doi: None,
            arxiv_id: None,
            semantic_scholar_id: None,
            title: "ALL biomarker response study".into(),
            journal: Some("Journal C".into()),
            date: Some("2025-01-01".into()),
            first_index_date: None,
            citation_count: Some(100),
            influential_citation_count: Some(5),
            source: ArticleSource::EuropePmc,
            score: None,
            is_retracted: Some(false),
            abstract_snippet: Some("MET is discussed in the abstract".into()),
            ranking: None,
            matched_sources: vec![ArticleSource::EuropePmc],
            normalized_title: "all biomarker response study".into(),
            normalized_abstract: "met is discussed in the abstract".into(),
            publication_type: None,
            source_local_position: 2,
        },
    ];

    rank_result_rows_by_directness(&mut rows, &filters);

    assert_eq!(rows[0].pmid, "100");
    assert_eq!(
        rows[0]
            .ranking
            .as_ref()
            .map(|ranking| ranking.directness_tier),
        Some(3)
    );
    assert_eq!(
        rows[1]
            .ranking
            .as_ref()
            .map(|ranking| ranking.directness_tier),
        Some(2)
    );
    assert_eq!(
        rows[2]
            .ranking
            .as_ref()
            .map(|ranking| ranking.directness_tier),
        Some(0)
    );
    assert_eq!(
        rows[2]
            .ranking
            .as_ref()
            .map(|ranking| ranking.combined_anchor_hits),
        Some(0)
    );
}

#[test]
fn directness_ranking_prefers_cue_then_citation_then_source_local_position() {
    let mut filters = empty_filters();
    filters.gene = Some("BRAF".into());
    filters.keyword = Some("melanoma".into());

    let mut rows = vec![
        ArticleSearchResult {
            pmid: "100".into(),
            pmcid: None,
            doi: None,
            arxiv_id: None,
            semantic_scholar_id: None,
            title: "BRAF melanoma study".into(),
            journal: Some("Journal A".into()),
            date: Some("2025-01-01".into()),
            first_index_date: None,
            citation_count: Some(10),
            influential_citation_count: Some(1),
            source: ArticleSource::EuropePmc,
            score: None,
            is_retracted: Some(false),
            abstract_snippet: None,
            ranking: None,
            matched_sources: vec![ArticleSource::EuropePmc],
            normalized_title: "braf melanoma study".into(),
            normalized_abstract: String::new(),
            publication_type: None,
            source_local_position: 0,
        },
        ArticleSearchResult {
            pmid: "200".into(),
            pmcid: None,
            doi: None,
            arxiv_id: None,
            semantic_scholar_id: None,
            title: "BRAF melanoma systematic review".into(),
            journal: Some("Journal B".into()),
            date: Some("2025-01-01".into()),
            first_index_date: None,
            citation_count: Some(5),
            influential_citation_count: Some(0),
            source: ArticleSource::EuropePmc,
            score: None,
            is_retracted: Some(false),
            abstract_snippet: None,
            ranking: None,
            matched_sources: vec![ArticleSource::EuropePmc],
            normalized_title: "braf melanoma systematic review".into(),
            normalized_abstract: String::new(),
            publication_type: Some("Review".into()),
            source_local_position: 1,
        },
        ArticleSearchResult {
            pmid: "300".into(),
            pmcid: None,
            doi: None,
            arxiv_id: None,
            semantic_scholar_id: None,
            title: "BRAF melanoma clinical trial review".into(),
            journal: Some("Journal C".into()),
            date: Some("2025-01-01".into()),
            first_index_date: None,
            citation_count: Some(50),
            influential_citation_count: Some(7),
            source: ArticleSource::EuropePmc,
            score: None,
            is_retracted: Some(false),
            abstract_snippet: None,
            ranking: None,
            matched_sources: vec![ArticleSource::EuropePmc],
            normalized_title: "braf melanoma clinical trial review".into(),
            normalized_abstract: String::new(),
            publication_type: Some("Clinical Trial".into()),
            source_local_position: 2,
        },
    ];

    rank_result_rows_by_directness(&mut rows, &filters);

    let pmids: Vec<&str> = rows.iter().map(|row| row.pmid.as_str()).collect();
    assert_eq!(pmids, vec!["300", "200", "100"]);
    assert_eq!(
        rows[0]
            .ranking
            .as_ref()
            .map(|ranking| ranking.study_or_review_cue),
        Some(true)
    );
}
