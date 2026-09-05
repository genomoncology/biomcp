use super::*;

#[test]
fn hybrid_lexical_coverage_beats_a_high_citation_one_anchor_match() {
    let mut filters = empty_filters();
    filters.keyword = Some("zolgensma treatment of retinoblastoma randomized trial".into());
    filters.ranking.requested_mode = Some(ArticleRankingMode::Hybrid);

    let five_anchor_match = worked_example_row(
        "26427984",
        ArticleSource::EuropePmc,
        "Treatment of retinoblastoma: a prospective randomized trial",
        "",
        0,
        25,
        None,
    );
    let one_anchor_match = worked_example_row(
        "32328653",
        ArticleSource::EuropePmc,
        "Treatment options in adult glioblastoma",
        "",
        0,
        982,
        None,
    );

    let page = finalize_article_candidates(
        vec![one_anchor_match, five_anchor_match],
        10,
        0,
        None,
        &filters,
    );

    assert_eq!(
        page.results
            .iter()
            .map(|row| row.pmid.as_str())
            .collect::<Vec<_>>(),
        vec!["26427984", "32328653"]
    );
    let five = row_by_pmid(&page.results, "26427984")
        .ranking
        .as_ref()
        .expect("five-anchor ranking");
    let one = row_by_pmid(&page.results, "32328653")
        .ranking
        .as_ref()
        .expect("one-anchor ranking");
    assert_eq!(five.lexical_score, Some(5.0 / 6.0));
    assert_eq!(one.lexical_score, Some(1.0 / 6.0));
    assert!(five.composite_score > one.composite_score);
}

#[test]
fn hybrid_coverage_counts_title_and_abstract_duplicate_once() {
    let mut filters = empty_filters();
    filters.keyword = Some("BRAF melanoma".into());
    filters.ranking.requested_mode = Some(ArticleRankingMode::Hybrid);
    let duplicate = worked_example_row(
        "6001",
        ArticleSource::EuropePmc,
        "BRAF resistance",
        "BRAF melanoma cohort",
        0,
        0,
        None,
    );

    let page = finalize_article_candidates(vec![duplicate], 10, 0, None, &filters);
    let ranking = page.results[0].ranking.as_ref().expect("ranking");

    assert_eq!(ranking.title_anchor_hits, 1);
    assert_eq!(ranking.abstract_anchor_hits, 2);
    assert_eq!(ranking.combined_anchor_hits, 2);
    assert_eq!(ranking.lexical_score, Some(1.0));
}

#[test]
fn hybrid_zero_anchor_lexical_score_is_finite_zero() {
    let mut filters = empty_filters();
    filters.ranking.requested_mode = Some(ArticleRankingMode::Hybrid);
    let candidate = worked_example_row(
        "6002",
        ArticleSource::EuropePmc,
        "Unanchored article",
        "",
        0,
        0,
        None,
    );

    let page = finalize_article_candidates(vec![candidate], 10, 0, None, &filters);
    let score = page.results[0]
        .ranking
        .as_ref()
        .and_then(|ranking| ranking.lexical_score)
        .expect("lexical score");

    assert_eq!(score, 0.0);
    assert!(score.is_finite());
}

#[test]
fn hybrid_coverage_uses_full_counts_before_public_counters_saturate() {
    let anchors = (0..300)
        .map(|index| format!("anchor{index}"))
        .collect::<Vec<_>>();
    let mut filters = empty_filters();
    filters.keyword = Some(anchors.join(" "));
    filters.ranking.requested_mode = Some(ArticleRankingMode::Hybrid);
    let title = anchors[..260].join(" ");
    let candidate = worked_example_row("6003", ArticleSource::EuropePmc, &title, "", 0, 0, None);

    let page = finalize_article_candidates(vec![candidate], 10, 0, None, &filters);
    let ranking = page.results[0].ranking.as_ref().expect("ranking");

    assert_eq!(ranking.anchor_count, u8::MAX);
    assert_eq!(ranking.combined_anchor_hits, u8::MAX);
    assert_eq!(ranking.lexical_score, Some(260.0 / 300.0));
}

#[test]
fn hybrid_exact_composite_tie_uses_stable_identifier_order() {
    let mut filters = empty_filters();
    filters.keyword = Some("BRAF".into());
    filters.ranking.requested_mode = Some(ArticleRankingMode::Hybrid);
    let higher_id = worked_example_row(
        "7002",
        ArticleSource::EuropePmc,
        "BRAF study",
        "",
        0,
        0,
        None,
    );
    let lower_id = worked_example_row(
        "7001",
        ArticleSource::EuropePmc,
        "BRAF study",
        "",
        0,
        0,
        None,
    );

    let page = finalize_article_candidates(vec![higher_id, lower_id], 10, 0, None, &filters);

    assert_eq!(
        page.results
            .iter()
            .map(|row| row.pmid.as_str())
            .collect::<Vec<_>>(),
        vec!["7001", "7002"]
    );
    assert_eq!(
        page.results[0]
            .ranking
            .as_ref()
            .and_then(|ranking| ranking.composite_score),
        page.results[1]
            .ranking
            .as_ref()
            .and_then(|ranking| ranking.composite_score)
    );
}

#[test]
fn hybrid_default_weights_orders_example_one() {
    let (filters, candidates) = hybrid_worked_example_fixture();
    let page = finalize_article_candidates(candidates, 10, 0, None, &filters);

    assert_eq!(
        page.results
            .iter()
            .map(|row| row.pmid.as_str())
            .collect::<Vec<_>>(),
        vec!["1003", "1001", "1002", "1005", "1004"]
    );
}

#[test]
fn lexical_mode_matches_current_ordering() {
    let (mut filters, candidates) = hybrid_worked_example_fixture();
    filters.ranking.requested_mode = Some(ArticleRankingMode::Lexical);
    let page = finalize_article_candidates(candidates, 10, 0, None, &filters);

    assert_eq!(
        page.results
            .iter()
            .map(|row| row.pmid.as_str())
            .collect::<Vec<_>>(),
        vec!["1002", "1003", "1004", "1001", "1005"]
    );
}

#[test]
fn semantic_mode_prefers_score_before_lexical_fallback() {
    let (mut filters, candidates) = hybrid_worked_example_fixture();
    filters.ranking.requested_mode = Some(ArticleRankingMode::Semantic);
    let page = finalize_article_candidates(candidates, 10, 0, None, &filters);

    assert_eq!(
        page.results
            .iter()
            .map(|row| row.pmid.as_str())
            .collect::<Vec<_>>(),
        vec!["1001", "1005", "1003", "1002", "1004"]
    );
}

#[test]
fn hybrid_entity_only_falls_back_without_nan() {
    let mut filters = empty_filters();
    filters.disease = Some("Hirschsprung disease".into());
    filters.ranking.requested_mode = Some(ArticleRankingMode::Hybrid);

    let rows = vec![
        worked_example_row(
            "2001",
            ArticleSource::EuropePmc,
            "Hirschsprung disease review",
            "",
            0,
            5,
            None,
        ),
        worked_example_row(
            "2002",
            ArticleSource::EuropePmc,
            "Enteric neuropathy mechanisms",
            "Hirschsprung disease cases were reviewed in the cohort",
            0,
            100,
            None,
        ),
        worked_example_row(
            "2003",
            ArticleSource::EuropePmc,
            "Ganglion development note",
            "",
            0,
            1000,
            None,
        ),
    ];

    let page = finalize_article_candidates(rows, 10, 0, None, &filters);

    assert_eq!(
        page.results
            .iter()
            .map(|row| row.pmid.as_str())
            .collect::<Vec<_>>(),
        vec!["2002", "2001", "2003"]
    );
    assert!(page.results.iter().all(|row| {
        let ranking = row.ranking.as_ref().expect("ranking should be present");
        ranking.semantic_score == Some(0.0)
            && ranking
                .composite_score
                .is_some_and(|score: f64| score.is_finite())
    }));
}

#[test]
fn hybrid_custom_weights_shift_ordering() {
    let (mut filters, candidates) = hybrid_worked_example_fixture();
    filters.ranking = ArticleRankingOptions::from_inputs(
        Some("hybrid"),
        Some(0.1),
        Some(0.6),
        Some(0.2),
        Some(0.1),
    )
    .expect("options should parse");
    let page = finalize_article_candidates(candidates, 10, 0, None, &filters);

    assert_eq!(
        page.results
            .iter()
            .map(|row| row.pmid.as_str())
            .collect::<Vec<_>>(),
        vec!["1003", "1002", "1001", "1004", "1005"]
    );
}

#[test]
fn hybrid_scoring_is_zero_safe() {
    let mut filters = empty_filters();
    filters.keyword = Some("ganglion".into());
    filters.ranking.requested_mode = Some(ArticleRankingMode::Hybrid);

    let rows = vec![
        worked_example_row(
            "3001",
            ArticleSource::LitSense2,
            "Ganglion atlas",
            "",
            0,
            0,
            Some(0.8),
        ),
        worked_example_row(
            "3002",
            ArticleSource::EuropePmc,
            "Ganglion case note",
            "",
            0,
            0,
            None,
        ),
    ];

    let page = finalize_article_candidates(rows, 10, 0, None, &filters);
    assert!(page.results.iter().all(|row| {
        let ranking = row.ranking.as_ref().expect("ranking should be present");
        ranking.citation_score == Some(0.0)
            && ranking.position_score == Some(0.0)
            && ranking
                .composite_score
                .is_some_and(|score: f64| score.is_finite())
    }));
}

#[test]
fn hybrid_uses_litsense2_signal_for_semantic_score() {
    let mut filters = empty_filters();
    filters.keyword = Some("BRAF melanoma".into());
    filters.ranking.requested_mode = Some(ArticleRankingMode::Hybrid);

    let pubtator_only = worked_example_row(
        "4001",
        ArticleSource::PubTator,
        "BRAF melanoma resistance map",
        "",
        0,
        10,
        Some(285.0),
    );
    let litsense2_only = worked_example_row(
        "4002",
        ArticleSource::LitSense2,
        "BRAF melanoma pathway atlas",
        "",
        1,
        5,
        Some(0.85),
    );
    let pubtator_duplicate = worked_example_row(
        "4003",
        ArticleSource::PubTator,
        "Merged BRAF melanoma evidence",
        "",
        0,
        12,
        Some(285.0),
    );
    let litsense2_duplicate = worked_example_row(
        "4003",
        ArticleSource::LitSense2,
        "Merged BRAF melanoma evidence",
        "",
        2,
        12,
        Some(0.95),
    );

    let page = finalize_article_candidates(
        vec![
            pubtator_only,
            litsense2_only,
            pubtator_duplicate,
            litsense2_duplicate,
        ],
        10,
        0,
        None,
        &filters,
    );

    let pubtator = row_by_pmid(&page.results, "4001");
    let litsense2 = row_by_pmid(&page.results, "4002");
    let merged = row_by_pmid(&page.results, "4003");

    assert_eq!(
        pubtator
            .ranking
            .as_ref()
            .expect("ranking should be present")
            .semantic_score,
        Some(0.0)
    );
    assert_eq!(
        litsense2
            .ranking
            .as_ref()
            .expect("ranking should be present")
            .semantic_score,
        Some(0.85)
    );
    assert_eq!(merged.score, Some(285.0));
    assert_eq!(
        merged
            .ranking
            .as_ref()
            .expect("ranking should be present")
            .semantic_score,
        Some(0.95)
    );
}

#[test]
fn semantic_mode_ignores_non_litsense2_raw_scores() {
    let mut filters = empty_filters();
    filters.keyword = Some("BRAF melanoma".into());
    filters.ranking.requested_mode = Some(ArticleRankingMode::Semantic);

    let pubtator_only = worked_example_row(
        "5001",
        ArticleSource::PubTator,
        "BRAF melanoma resistance map",
        "",
        0,
        10,
        Some(285.0),
    );
    let litsense2_only = worked_example_row(
        "5002",
        ArticleSource::LitSense2,
        "BRAF melanoma pathway atlas",
        "",
        1,
        5,
        Some(0.85),
    );

    let page =
        finalize_article_candidates(vec![pubtator_only, litsense2_only], 10, 0, None, &filters);

    assert_eq!(
        page.results
            .iter()
            .map(|row| row.pmid.as_str())
            .collect::<Vec<_>>(),
        vec!["5002", "5001"]
    );

    let litsense2 = row_by_pmid(&page.results, "5002");
    let pubtator = row_by_pmid(&page.results, "5001");

    assert_eq!(
        litsense2
            .ranking
            .as_ref()
            .expect("ranking should be present")
            .mode,
        Some(ArticleRankingMode::Semantic)
    );
    assert_eq!(
        litsense2
            .ranking
            .as_ref()
            .expect("ranking should be present")
            .semantic_score,
        Some(0.85)
    );
    assert_eq!(
        pubtator
            .ranking
            .as_ref()
            .expect("ranking should be present")
            .mode,
        Some(ArticleRankingMode::Semantic)
    );
    assert_eq!(
        pubtator
            .ranking
            .as_ref()
            .expect("ranking should be present")
            .semantic_score,
        Some(0.0)
    );
}
