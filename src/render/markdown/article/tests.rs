use super::super::test_support::article_filters_for_test;
use super::*;
use crate::cli::debug_plan::DebugPlan;
use crate::entities::article::{ArticleSearchResult, ArticleSort, ArticleSource};
use chrono::NaiveDate;
#[test]
fn article_entities_markdown_uses_safe_gene_search_commands() {
    let annotations = ArticleAnnotations {
        genes: vec![
            AnnotationCount {
                text: "BRAF".to_string(),
                count: 5,
            },
            AnnotationCount {
                text: "serine-threonine protein kinase".to_string(),
                count: 1,
            },
        ],
        diseases: Vec::new(),
        chemicals: Vec::new(),
        mutations: vec![AnnotationCount {
            text: "V600E".to_string(),
            count: 2,
        }],
    };

    let markdown =
        article_entities_markdown("22663011", Some(&annotations), Some(5)).expect("markdown");
    assert!(markdown.contains("`biomcp search gene -q BRAF`"));
    assert!(markdown.contains("`biomcp search gene -q \"serine-threonine protein kinase\"`"));
    assert!(!markdown.contains("`biomcp get gene serine-threonine protein kinase`"));
    assert!(markdown.contains("`biomcp get variant V600E`"));
}

#[test]
fn article_markdown_renders_semantic_scholar_section() {
    let article = Article {
        pmid: Some("22663011".to_string()),
        pmcid: None,
        doi: Some("10.1000/example".to_string()),
        title: "Example".to_string(),
        authors: Vec::new(),
        journal: Some("Example Journal".to_string()),
        date: Some("2024-01-01".to_string()),
        citation_count: Some(12),
        publication_type: None,
        open_access: Some(true),
        abstract_text: None,
        full_text_path: None,
        full_text_note: None,
        full_text_source: None,
        annotations: None,
        semantic_scholar: Some(crate::entities::article::ArticleSemanticScholar {
            paper_id: Some("paper-1".to_string()),
            tldr: Some("A concise summary.".to_string()),
            citation_count: Some(20),
            influential_citation_count: Some(4),
            reference_count: Some(10),
            is_open_access: Some(true),
            open_access_pdf: Some(crate::entities::article::ArticleSemanticScholarPdf {
                url: "https://example.org/paper.pdf".to_string(),
                status: Some("GREEN".to_string()),
                license: Some("CC-BY".to_string()),
            }),
        }),
        pubtator_fallback: false,
    };

    let markdown =
        article_markdown(&article, &["tldr".to_string()]).expect("markdown should render");
    assert!(markdown.contains("## Semantic Scholar"));
    assert!(markdown.contains("TLDR: A concise summary."));
    assert!(markdown.contains("Influential citations: 4"));
    assert!(markdown.contains("Open-access PDF: https://example.org/paper.pdf"));
}

#[test]
fn article_markdown_renders_resolved_fulltext_source_label() {
    let article = Article {
        pmid: Some("22663011".to_string()),
        pmcid: Some("PMC123456".to_string()),
        doi: Some("10.1000/example".to_string()),
        title: "Example".to_string(),
        authors: Vec::new(),
        journal: Some("Example Journal".to_string()),
        date: Some("2024-01-01".to_string()),
        citation_count: Some(12),
        publication_type: None,
        open_access: Some(true),
        abstract_text: None,
        full_text_path: Some(std::path::PathBuf::from("/tmp/fulltext.md")),
        full_text_note: None,
        full_text_source: Some(crate::entities::article::ArticleFulltextSource {
            kind: crate::entities::article::ArticleFulltextKind::JatsXml,
            label: "Europe PMC XML".to_string(),
            source: "Europe PMC".to_string(),
        }),
        annotations: None,
        semantic_scholar: None,
        pubtator_fallback: false,
    };

    let markdown =
        article_markdown(&article, &["fulltext".to_string()]).expect("markdown should render");
    assert!(markdown.contains("## Full Text (Europe PMC XML)"));
    assert!(markdown.contains("Saved to: /tmp/fulltext.md"));
}

#[test]
fn article_graph_markdown_renders_expected_table_headers() {
    let result = crate::entities::article::ArticleGraphResult {
        article: crate::entities::article::ArticleRelatedPaper {
            paper_id: Some("paper-1".to_string()),
            pmid: Some("22663011".to_string()),
            doi: None,
            arxiv_id: None,
            title: "Seed".to_string(),
            journal: None,
            year: Some(2012),
        },
        edges: vec![crate::entities::article::ArticleGraphEdge {
            paper: crate::entities::article::ArticleRelatedPaper {
                paper_id: Some("paper-2".to_string()),
                pmid: Some("24200969".to_string()),
                doi: None,
                arxiv_id: None,
                title: "Related paper".to_string(),
                journal: Some("Nature".to_string()),
                year: Some(2014),
            },
            intents: vec!["Background".to_string()],
            contexts: vec!["Important supporting context".to_string()],
            is_influential: true,
        }],
    };

    let markdown = article_graph_markdown("Citations", &result).expect("graph markdown");
    assert!(markdown.contains("# Citations for PMID 22663011"));
    assert!(markdown.contains("| PMID | Title | Intents | Influential | Context |"));
    assert!(markdown.contains(
        "| 24200969 | Related paper | Background | yes | Important supporting context |"
    ));
}

#[test]
fn article_batch_markdown_renders_compact_rows() {
    let rows = vec![
        crate::entities::article::ArticleBatchItem {
            requested_id: "22663011".to_string(),
            pmid: Some("22663011".to_string()),
            pmcid: None,
            doi: Some("10.1056/NEJMoa1203421".to_string()),
            title: "Improved survival with vemurafenib".to_string(),
            journal: Some("NEJM".to_string()),
            year: Some(2012),
            entity_summary: Some(crate::entities::article::ArticleBatchEntitySummary {
                genes: vec![crate::entities::article::AnnotationCount {
                    text: "BRAF".to_string(),
                    count: 4,
                }],
                diseases: vec![crate::entities::article::AnnotationCount {
                    text: "melanoma".to_string(),
                    count: 2,
                }],
                chemicals: Vec::new(),
                mutations: Vec::new(),
            }),
            tldr: Some("BRAF inhibitor benefit in melanoma.".to_string()),
            citation_count: Some(120),
            influential_citation_count: Some(18),
        },
        crate::entities::article::ArticleBatchItem {
            requested_id: "PMC9984800".to_string(),
            pmid: Some("24200969".to_string()),
            pmcid: Some("PMC9984800".to_string()),
            doi: None,
            title: "Follow-up trial".to_string(),
            journal: Some("Nature".to_string()),
            year: Some(2014),
            entity_summary: None,
            tldr: None,
            citation_count: None,
            influential_citation_count: None,
        },
    ];

    let markdown = article_batch_markdown(&rows).expect("batch markdown");
    assert!(markdown.contains("# Article Batch (2)"));
    assert!(markdown.contains("## 1. Improved survival with vemurafenib"));
    assert!(markdown.contains("PMID: 22663011"));
    assert!(markdown.contains("Entities: Genes: BRAF (4); Diseases: melanoma (2)"));
    assert!(markdown.contains("TLDR: BRAF inhibitor benefit in melanoma."));
    assert!(markdown.contains("Citations: 120 (influential: 18)"));
    assert!(markdown.contains("## 2. Follow-up trial"));
    assert!(markdown.contains("PMID: 24200969"));
    // Absent optional fields are omitted, not printed as placeholders
    assert!(!markdown.contains("TLDR: -"));
    assert!(!markdown.contains("Entities: -"));
}

#[test]
fn article_search_markdown_preserves_rank_order_and_shows_rationale() {
    let rows = vec![
        ArticleSearchResult {
            pmid: "1".into(),
            title: "Entity-ranked".into(),
            pmcid: Some("PMC1".into()),
            doi: Some("10.1000/one".into()),
            journal: Some("Journal A".into()),
            date: Some("2025-01-01".into()),
            first_index_date: None,
            citation_count: Some(10),
            influential_citation_count: Some(4),
            source: ArticleSource::PubTator,
            score: Some(99.1),
            is_retracted: Some(false),
            abstract_snippet: Some("Abstract one".into()),
            ranking: Some(crate::entities::article::ArticleRankingMetadata {
                directness_tier: 3,
                anchor_count: 2,
                title_anchor_hits: 2,
                abstract_anchor_hits: 0,
                combined_anchor_hits: 2,
                all_anchors_in_title: true,
                all_anchors_in_text: true,
                study_or_review_cue: false,
                pubmed_rescue: false,
                pubmed_rescue_kind: None,
                pubmed_source_position: None,
                mode: Some(crate::entities::article::ArticleRankingMode::Lexical),
                semantic_score: None,
                lexical_score: None,
                citation_score: None,
                position_score: None,
                composite_score: None,
                avg_source_rank: None,
            }),
            matched_sources: vec![ArticleSource::PubTator, ArticleSource::SemanticScholar],
            normalized_title: "entity-ranked".into(),
            normalized_abstract: "abstract one".into(),
            publication_type: None,
            source_local_position: 0,
        },
        ArticleSearchResult {
            pmid: "2".into(),
            title: "Field-ranked".into(),
            pmcid: None,
            doi: None,
            journal: Some("Journal B".into()),
            date: Some("2025-01-02".into()),
            first_index_date: None,
            citation_count: Some(12),
            influential_citation_count: Some(1),
            source: ArticleSource::EuropePmc,
            score: None,
            is_retracted: Some(false),
            abstract_snippet: Some("Abstract two".into()),
            ranking: Some(crate::entities::article::ArticleRankingMetadata {
                directness_tier: 2,
                anchor_count: 2,
                title_anchor_hits: 1,
                abstract_anchor_hits: 1,
                combined_anchor_hits: 2,
                all_anchors_in_title: false,
                all_anchors_in_text: true,
                study_or_review_cue: true,
                pubmed_rescue: false,
                pubmed_rescue_kind: None,
                pubmed_source_position: None,
                mode: Some(crate::entities::article::ArticleRankingMode::Lexical),
                semantic_score: None,
                lexical_score: None,
                citation_score: None,
                position_score: None,
                composite_score: None,
                avg_source_rank: None,
            }),
            matched_sources: vec![ArticleSource::EuropePmc],
            normalized_title: "field-ranked".into(),
            normalized_abstract: "abstract two".into(),
            publication_type: Some("Review".into()),
            source_local_position: 1,
        },
    ];

    let markdown = article_search_markdown_with_footer_and_context(
        "gene=BRAF",
        &rows,
        "",
        &article_filters_for_test(crate::entities::article::ArticleSort::Relevance),
        ArticleSearchRenderContext {
            source_filter: crate::entities::article::ArticleSourceFilter::All,
            semantic_scholar_enabled: true,
            note: Some(
                "Note: --type restricts article search to Europe PMC and PubMed. PubTator3, LitSense2, and Semantic Scholar do not support publication-type filtering.",
            ),
            debug_plan: None,
            exact_entity_commands: &[],
            source_status: &[],
        },
    )
    .expect("markdown should render");
    assert!(markdown.contains(
            "> Note: --type restricts article search to Europe PMC and PubMed. PubTator3, LitSense2, and Semantic Scholar do not support publication-type filtering."
        ));
    assert!(markdown.contains("Semantic Scholar: enabled"));
    assert!(markdown.contains("Ranking: calibrated PubMed rescue + lexical directness"));
    assert!(markdown.contains("| PMID | Title | Source(s) | Date | Why | Cit. |"));
    assert!(markdown.contains("PubTator3, Semantic Scholar"));
    assert!(markdown.contains("title 2/2"));
    assert!(markdown.contains("title+abstract 2/2"));
    assert!(
        markdown
            .contains("--date-from/--date-to <YYYY|YYYY-MM|YYYY-MM-DD> (alias: --since/--until)")
    );
    assert!(!markdown.contains("## PubTator3"));
    assert!(!markdown.contains("## Europe PMC"));
    assert!(markdown.find("|1|").unwrap() < markdown.find("|2|").unwrap());
}

#[test]
fn article_ranking_why_tier1_mixed_shows_title_plus_abstract() {
    let row = ArticleSearchResult {
        pmid: "1".into(),
        title: "Partial coverage".into(),
        pmcid: None,
        doi: None,
        journal: None,
        date: None,
        first_index_date: None,
        citation_count: None,
        influential_citation_count: None,
        source: ArticleSource::EuropePmc,
        matched_sources: vec![ArticleSource::EuropePmc],
        score: None,
        is_retracted: None,
        abstract_snippet: None,
        ranking: Some(crate::entities::article::ArticleRankingMetadata {
            directness_tier: 1,
            anchor_count: 3,
            title_anchor_hits: 1,
            abstract_anchor_hits: 1,
            combined_anchor_hits: 2,
            all_anchors_in_title: false,
            all_anchors_in_text: false,
            study_or_review_cue: false,
            pubmed_rescue: false,
            pubmed_rescue_kind: None,
            pubmed_source_position: None,
            mode: Some(crate::entities::article::ArticleRankingMode::Lexical),
            semantic_score: None,
            lexical_score: None,
            citation_score: None,
            position_score: None,
            composite_score: None,
            avg_source_rank: None,
        }),
        normalized_title: "partial coverage".into(),
        normalized_abstract: String::new(),
        publication_type: None,
        source_local_position: 0,
    };
    let why = article_ranking_why(&row, &article_filters_for_test(ArticleSort::Relevance));
    assert_eq!(why, "title+abstract 2/3");
}

#[test]
fn article_ranking_why_rescue_composes_with_lexical_reason() {
    let row = ArticleSearchResult {
        pmid: "1".into(),
        title: "Rescued partial coverage".into(),
        pmcid: None,
        doi: None,
        journal: None,
        date: None,
        first_index_date: None,
        citation_count: None,
        influential_citation_count: None,
        source: ArticleSource::PubMed,
        matched_sources: vec![ArticleSource::PubMed],
        score: None,
        is_retracted: None,
        abstract_snippet: None,
        ranking: Some(crate::entities::article::ArticleRankingMetadata {
            directness_tier: 1,
            anchor_count: 3,
            title_anchor_hits: 1,
            abstract_anchor_hits: 1,
            combined_anchor_hits: 2,
            all_anchors_in_title: false,
            all_anchors_in_text: false,
            study_or_review_cue: false,
            pubmed_rescue: true,
            pubmed_rescue_kind: Some(crate::entities::article::ArticlePubMedRescueKind::Unique),
            pubmed_source_position: Some(0),
            mode: Some(crate::entities::article::ArticleRankingMode::Lexical),
            semantic_score: None,
            lexical_score: None,
            citation_score: None,
            position_score: None,
            composite_score: None,
            avg_source_rank: None,
        }),
        normalized_title: "rescued partial coverage".into(),
        normalized_abstract: String::new(),
        publication_type: None,
        source_local_position: 0,
    };

    let why = article_ranking_why(&row, &article_filters_for_test(ArticleSort::Relevance));
    assert_eq!(why, "pubmed-rescue + title+abstract 2/3");
}

#[test]
fn article_ranking_why_semantic_includes_score_and_lexical_context() {
    let row = ArticleSearchResult {
        pmid: "1".into(),
        title: "Semantic lead".into(),
        pmcid: None,
        doi: None,
        journal: None,
        date: None,
        first_index_date: None,
        citation_count: None,
        influential_citation_count: None,
        source: ArticleSource::EuropePmc,
        matched_sources: vec![ArticleSource::EuropePmc],
        score: Some(0.81234),
        is_retracted: None,
        abstract_snippet: None,
        ranking: Some(crate::entities::article::ArticleRankingMetadata {
            directness_tier: 2,
            anchor_count: 3,
            title_anchor_hits: 2,
            abstract_anchor_hits: 0,
            combined_anchor_hits: 2,
            all_anchors_in_title: true,
            all_anchors_in_text: false,
            study_or_review_cue: false,
            pubmed_rescue: false,
            pubmed_rescue_kind: None,
            pubmed_source_position: None,
            mode: Some(crate::entities::article::ArticleRankingMode::Semantic),
            semantic_score: Some(0.81234),
            lexical_score: None,
            citation_score: None,
            position_score: None,
            composite_score: None,
            avg_source_rank: None,
        }),
        normalized_title: "semantic lead".into(),
        normalized_abstract: String::new(),
        publication_type: None,
        source_local_position: 0,
    };

    let why = article_ranking_why(&row, &article_filters_for_test(ArticleSort::Relevance));
    assert_eq!(why, "semantic 0.812 + title 2/3");
}

#[test]
fn article_ranking_why_hybrid_includes_score_and_lexical_context() {
    let row = ArticleSearchResult {
        pmid: "1".into(),
        title: "Hybrid lead".into(),
        pmcid: None,
        doi: None,
        journal: None,
        date: None,
        first_index_date: None,
        citation_count: None,
        influential_citation_count: None,
        source: ArticleSource::EuropePmc,
        matched_sources: vec![ArticleSource::EuropePmc],
        score: Some(0.9),
        is_retracted: None,
        abstract_snippet: None,
        ranking: Some(crate::entities::article::ArticleRankingMetadata {
            directness_tier: 1,
            anchor_count: 3,
            title_anchor_hits: 1,
            abstract_anchor_hits: 1,
            combined_anchor_hits: 2,
            all_anchors_in_title: false,
            all_anchors_in_text: false,
            study_or_review_cue: false,
            pubmed_rescue: false,
            pubmed_rescue_kind: None,
            pubmed_source_position: None,
            mode: Some(crate::entities::article::ArticleRankingMode::Hybrid),
            semantic_score: Some(0.9),
            lexical_score: Some(1.0 / 3.0),
            citation_score: Some(0.1),
            position_score: Some(0.4),
            composite_score: Some(0.61234),
            avg_source_rank: Some(1.0),
        }),
        normalized_title: "hybrid lead".into(),
        normalized_abstract: String::new(),
        publication_type: None,
        source_local_position: 0,
    };

    let why = article_ranking_why(&row, &article_filters_for_test(ArticleSort::Relevance));
    assert_eq!(why, "hybrid 0.612 + title+abstract 2/3");
}

#[test]
fn article_search_markdown_prepends_debug_plan_block() {
    let debug_plan = DebugPlan {
        surface: "search_article",
        query: "gene=BRAF".to_string(),
        anchor: None,
        legs: vec![crate::cli::debug_plan::DebugPlanLeg {
            leg: "article".to_string(),
            entity: "article".to_string(),
            filters: vec!["gene=BRAF".to_string()],
            routing: vec!["planner=federated".to_string()],
            sources: vec!["PubTator3".to_string(), "Europe PMC".to_string()],
            matched_sources: vec!["PubTator3".to_string()],
            source_status: Vec::new(),
            count: 1,
            total: Some(1),
            note: None,
            error: None,
        }],
    };
    let rows = vec![ArticleSearchResult {
        pmid: "1".into(),
        title: "Entity-ranked".into(),
        pmcid: None,
        doi: None,
        journal: Some("Journal A".into()),
        date: Some("2025-01-01".into()),
        first_index_date: None,
        citation_count: Some(10),
        influential_citation_count: Some(4),
        source: ArticleSource::PubTator,
        score: Some(99.1),
        is_retracted: Some(false),
        abstract_snippet: Some("Abstract one".into()),
        ranking: None,
        matched_sources: vec![ArticleSource::PubTator],
        normalized_title: "entity-ranked".into(),
        normalized_abstract: "abstract one".into(),
        publication_type: None,
        source_local_position: 0,
    }];

    let markdown = article_search_markdown_with_footer_and_context(
        "gene=BRAF",
        &rows,
        "",
        &article_filters_for_test(crate::entities::article::ArticleSort::Relevance),
        ArticleSearchRenderContext {
            source_filter: crate::entities::article::ArticleSourceFilter::All,
            semantic_scholar_enabled: true,
            note: None,
            debug_plan: Some(&debug_plan),
            exact_entity_commands: &[],
            source_status: &[],
        },
    )
    .expect("markdown should render");

    assert!(markdown.starts_with("## Debug plan"));
    assert!(markdown.contains("\"surface\": \"search_article\""));
    assert!(markdown.contains("# Articles: gene=BRAF"));
}

#[test]
fn article_search_markdown_renders_related_block_before_pagination() {
    let rows = vec![ArticleSearchResult {
        pmid: "22663011".into(),
        title: "Entity-aware article".into(),
        pmcid: None,
        doi: None,
        journal: Some("Journal".into()),
        date: Some("2025-01-01".into()),
        first_index_date: Some(NaiveDate::from_ymd_opt(2025, 1, 15).expect("valid date")),
        citation_count: Some(12),
        influential_citation_count: Some(4),
        source: ArticleSource::EuropePmc,
        score: None,
        is_retracted: Some(false),
        abstract_snippet: Some("Abstract".into()),
        ranking: None,
        matched_sources: vec![ArticleSource::EuropePmc],
        normalized_title: "entity-aware article".into(),
        normalized_abstract: "abstract".into(),
        publication_type: None,
        source_local_position: 0,
    }];
    let mut filters = article_filters_for_test(crate::entities::article::ArticleSort::Relevance);
    filters.keyword = Some("BRAF".into());
    let exact_commands = vec!["biomcp get gene BRAF".to_string()];

    let markdown = article_search_markdown_with_footer_and_context(
        "keyword=BRAF",
        &rows,
        "Showing 1-1 of 3 results. Use --offset 1 for more.",
        &filters,
        ArticleSearchRenderContext {
            source_filter: crate::entities::article::ArticleSourceFilter::All,
            semantic_scholar_enabled: true,
            note: None,
            debug_plan: None,
            exact_entity_commands: &exact_commands,
            source_status: &[],
        },
    )
    .expect("markdown should render");

    let footer_line = markdown.find("Newest indexed:").expect("index footer");
    let filters_line = markdown.find("Filters:").expect("filters line");
    let related_line = markdown.find("See also:").expect("related block");
    let pagination_line = markdown
        .find("Showing 1-1 of 3 results. Use --offset 1 for more.")
        .expect("pagination footer");

    assert!(footer_line < filters_line);
    assert!(filters_line < related_line);
    assert!(related_line < pagination_line);
    assert!(markdown.contains("biomcp get gene BRAF"));
    assert!(!markdown.contains("biomcp search article -g BRAF -k"));
}

#[test]
fn article_search_markdown_includes_cross_entity_discover_hint_for_short_keyword_phrase() {
    let rows = vec![ArticleSearchResult {
        pmid: "22663011".into(),
        title: "Entity-aware article".into(),
        pmcid: None,
        doi: None,
        journal: Some("Journal".into()),
        date: Some("2025-01-01".into()),
        first_index_date: None,
        citation_count: Some(12),
        influential_citation_count: Some(4),
        source: ArticleSource::EuropePmc,
        score: None,
        is_retracted: Some(false),
        abstract_snippet: Some("Abstract".into()),
        ranking: None,
        matched_sources: vec![ArticleSource::EuropePmc],
        normalized_title: "entity-aware article".into(),
        normalized_abstract: "abstract".into(),
        publication_type: None,
        source_local_position: 0,
    }];
    let mut filters = article_filters_for_test(crate::entities::article::ArticleSort::Relevance);
    filters.keyword = Some("live attenuated vaccines".into());

    let markdown = article_search_markdown_with_footer_and_context(
        "keyword=live attenuated vaccines",
        &rows,
        "",
        &filters,
        ArticleSearchRenderContext {
            source_filter: crate::entities::article::ArticleSourceFilter::All,
            semantic_scholar_enabled: true,
            note: None,
            debug_plan: None,
            exact_entity_commands: &[],
            source_status: &[],
        },
    )
    .expect("markdown should render");

    assert!(markdown.contains("See also:"));
    assert!(markdown.contains("biomcp discover \"live attenuated vaccines\""));
}

#[test]
fn format_newest_indexed_footer_is_deterministic() {
    let indexed = NaiveDate::from_ymd_opt(2025, 1, 15).expect("valid date");
    let today = NaiveDate::from_ymd_opt(2025, 1, 20).expect("valid date");

    assert_eq!(
        format_newest_indexed_footer(indexed, today),
        "Newest indexed: 2025-01-15 (5 days ago)"
    );
}

#[test]
fn format_newest_indexed_footer_clamps_future_dates_to_zero_days() {
    let indexed = NaiveDate::from_ymd_opt(2025, 1, 15).expect("valid date");
    let today = NaiveDate::from_ymd_opt(2025, 1, 14).expect("valid date");

    assert_eq!(
        format_newest_indexed_footer(indexed, today),
        "Newest indexed: 2025-01-15 (0 days ago)"
    );
}

#[test]
fn article_search_markdown_omits_index_footer_when_no_rows_have_it() {
    let rows = vec![ArticleSearchResult {
        pmid: "22663011".into(),
        title: "Entity-aware article".into(),
        pmcid: None,
        doi: None,
        journal: Some("Journal".into()),
        date: Some("2025-01-01".into()),
        first_index_date: None,
        citation_count: Some(12),
        influential_citation_count: Some(4),
        source: ArticleSource::EuropePmc,
        score: None,
        is_retracted: Some(false),
        abstract_snippet: Some("Abstract".into()),
        ranking: None,
        matched_sources: vec![ArticleSource::EuropePmc],
        normalized_title: "entity-aware article".into(),
        normalized_abstract: "abstract".into(),
        publication_type: None,
        source_local_position: 0,
    }];

    let markdown = article_search_markdown_with_footer_and_context(
        "gene=BRAF",
        &rows,
        "",
        &article_filters_for_test(crate::entities::article::ArticleSort::Relevance),
        ArticleSearchRenderContext {
            source_filter: crate::entities::article::ArticleSourceFilter::All,
            semantic_scholar_enabled: true,
            note: None,
            debug_plan: None,
            exact_entity_commands: &[],
            source_status: &[],
        },
    )
    .expect("markdown should render");

    assert!(!markdown.contains("Newest indexed:"));
}
