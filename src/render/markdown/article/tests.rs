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
fn article_markdown_renders_semantic_scholar_and_indexing_sections() {
    let mut article = Article {
        section_outcomes: crate::entities::section_outcome::SectionOutcomes::with_keys(
            crate::entities::article::ARTICLE_OUTCOME_KEYS,
        ),
        pmid: Some("22663011".to_string()),
        pmcid: None,
        doi: Some("10.1000/example".to_string()),
        title: "Example".to_string(),
        authors: Vec::new(),
        author_count: 0,
        author_completeness: ArticleAuthorCompleteness::Unavailable,
        author_source: ArticleSource::PubTator,
        journal: Some("Example Journal".to_string()),
        date: Some("2024-01-01".to_string()),
        citation_count: Some(12),
        publication_type: None,
        open_access: Some(true),
        abstract_text: None,
        full_text_path: None,
        full_text_note: None,
        full_text_source: None,
        full_text_manifest: None,
        not_included: None,
        europepmc_license: None,
        europepmc_retracted: None,
        annotations: None,
        indexing: None,
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

    let detail = article_markdown(&article, &[]).expect("detail markdown should render");
    assert!(detail.contains("Authorship: unavailable (no author list supplied by PubTator3)"));

    article.indexing = Some(crate::entities::article::ArticleIndexing {
        status: crate::entities::article::ArticleIndexingStatus::Available,
        source: ArticleSource::PubMed,
        authors: vec![crate::entities::article::ArticleIndexingAuthor {
            name: "Ada First".into(),
            orcid: Some("0000-0002-1825-0097".into()),
            affiliations: vec![crate::entities::article::ArticleAffiliation {
                text: "Fixture University".into(),
                identifiers: vec![crate::entities::article::ArticleAffiliationIdentifier {
                    source: "ROR".into(),
                    value: "shared".into(),
                }],
            }],
        }],
        failure: None,
        mesh_headings: vec![crate::entities::article::ArticleMeshHeading {
            descriptor: crate::entities::article::ArticleMeshTerm {
                text: "Melanoma".into(),
                ui: Some("D008545".into()),
                major_topic: true,
            },
            qualifiers: vec![crate::entities::article::ArticleMeshTerm {
                text: "genetics".into(),
                ui: Some("Q000235".into()),
                major_topic: false,
            }],
        }],
    });
    let indexing = article_markdown(&article, &["indexing".to_string()])
        .expect("indexing markdown should render");
    for expected in [
        "## Article Indexing",
        "Status: available",
        "Source: PubMed",
        "Ada First",
        "Fixture University",
        "Melanoma (D008545); major topic: yes",
        "genetics (Q000235); major topic: no",
    ] {
        assert!(
            indexing.contains(expected),
            "missing {expected:?}: {indexing}"
        );
    }

    article.indexing = Some(crate::entities::article::ArticleIndexing {
        status: crate::entities::article::ArticleIndexingStatus::Unavailable,
        source: ArticleSource::PubMed,
        authors: Vec::new(),
        mesh_headings: Vec::new(),
        failure: Some(crate::entities::article::ArticleIndexingFailure {
            code: crate::entities::article::ArticleIndexingFailureCode::ParseError,
            message: "PubMed indexing response could not be parsed.".into(),
        }),
    });
    let unavailable = article_markdown(&article, &["indexing".to_string()])
        .expect("unavailable indexing should render");
    for expected in [
        "Status: unavailable",
        "indexing metadata is unavailable",
        "Failure code: parse_error",
        "PubMed indexing response could not be parsed.",
    ] {
        assert!(
            unavailable.contains(expected),
            "missing {expected:?}: {unavailable}"
        );
    }
    for sentinel in [
        "raw-body-sentinel",
        "api_key=secret-sentinel",
        "parser-internal-sentinel",
    ] {
        assert!(!unavailable.contains(sentinel));
    }
}

#[test]
fn article_markdown_renders_resolved_fulltext_source_label() {
    let mut article = Article {
        section_outcomes: crate::entities::section_outcome::SectionOutcomes::with_keys(
            crate::entities::article::ARTICLE_OUTCOME_KEYS,
        ),
        pmid: Some("22663011".to_string()),
        pmcid: Some("PMC123456".to_string()),
        doi: Some("10.1000/example".to_string()),
        title: "Example".to_string(),
        authors: vec![
            "First Author".to_string(),
            "Second Author".to_string(),
            "Middle Author".to_string(),
            "Fourth Author".to_string(),
            "Fifth Author".to_string(),
            "Last Author".to_string(),
        ],
        author_count: 6,
        author_completeness: ArticleAuthorCompleteness::SourceLimited,
        author_source: ArticleSource::EuropePmc,
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
        full_text_manifest: None,
        not_included: None,
        europepmc_license: None,
        europepmc_retracted: None,
        annotations: None,
        indexing: None,
        semantic_scholar: None,
        pubtator_fallback: false,
    };
    article.section_outcomes.complete(
        "fulltext",
        crate::entities::section_outcome::SectionOutcome::data("Europe PMC"),
    );

    let markdown =
        article_markdown(&article, &["fulltext".to_string()]).expect("markdown should render");
    assert!(markdown.contains("## Full Text (Europe PMC XML)"));
    assert!(markdown.contains("Saved to: /tmp/fulltext.md"));

    let detail = article_markdown(&article, &[]).expect("detail markdown should render");
    assert!(detail.contains(
        "First Author, Second Author, Middle Author, Fourth Author, Fifth Author, Last Author"
    ));
    assert!(detail.contains("Authorship: source-limited (6 returned; Europe PMC)"));
}

#[test]
fn article_recommendations_markdown_sanitizes_provider_fields() {
    let paper = crate::entities::article::ArticleRelatedPaper {
        paper_id: Some("paper-1".into()),
        pmid: None,
        doi: None,
        arxiv_id: None,
        title: "Control\u{7} Title | α-synuclein\u{202e}".into(),
        journal: Some("Journal\u{1b}[31m Name".into()),
        year: Some(2025),
    };
    let result = crate::entities::article::ArticleRecommendationsResult {
        positive_seeds: vec![paper.clone()],
        negative_seeds: Vec::new(),
        recommendations: vec![paper],
    };

    let markdown = article_recommendations_markdown(&result).expect("recommendations markdown");
    assert!(markdown.contains("Control Title \\| α-synuclein"));
    assert!(markdown.contains("Journal Name"));
    assert!(!markdown.contains(['\u{7}', '\u{1b}', '\u{202e}']));
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
    assert!(markdown.contains("| Identifier | Title | Intents | Influential | Context |"));
    assert!(markdown.contains(
        "| PMID 24200969 | Related paper | Background | yes | Important supporting context |"
    ));
}

#[test]
fn article_related_tables_render_typed_identifiers() {
    let paper =
        |paper_id: Option<&str>, pmid: Option<&str>, doi: Option<&str>, arxiv: Option<&str>| {
            crate::entities::article::ArticleRelatedPaper {
                paper_id: paper_id.map(str::to_string),
                pmid: pmid.map(str::to_string),
                doi: doi.map(str::to_string),
                arxiv_id: arxiv.map(str::to_string),
                title: "Related".into(),
                journal: None,
                year: None,
            }
        };
    let result = crate::entities::article::ArticleRecommendationsResult {
        positive_seeds: vec![paper(Some("seed"), Some("1"), None, None)],
        negative_seeds: Vec::new(),
        recommendations: vec![
            paper(None, Some("2"), None, None),
            paper(None, None, Some("10.1000/example"), None),
            paper(None, None, None, Some("2401.12345")),
            paper(Some("paper-4"), None, None, None),
        ],
    };

    let markdown = article_recommendations_markdown(&result).expect("typed recommendations");
    assert!(markdown.contains("| Identifier | Title | Journal | Year |"));
    for identifier in [
        "PMID 2",
        "DOI 10.1000/example",
        "arXiv 2401.12345",
        "Semantic Scholar paper-4",
    ] {
        assert!(
            markdown.contains(identifier),
            "missing {identifier}: {markdown}"
        );
    }
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
            authors: vec!["A. One".into(), "B. Two".into(), "C. Three".into()],
            author_count: 3,
            author_completeness: ArticleAuthorCompleteness::Complete,
            author_source: ArticleSource::PubTator,
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
            requested_id: "source-limited".to_string(),
            pmid: Some("24000000".to_string()),
            pmcid: None,
            doi: None,
            title: "Source-limited authors".to_string(),
            authors: vec![
                "First Author".into(),
                "Middle Author".into(),
                "Last Author".into(),
            ],
            author_count: 3,
            author_completeness: ArticleAuthorCompleteness::SourceLimited,
            author_source: ArticleSource::EuropePmc,
            journal: None,
            year: None,
            entity_summary: None,
            tldr: None,
            citation_count: None,
            influential_citation_count: None,
        },
        crate::entities::article::ArticleBatchItem {
            requested_id: "PMC9984800".to_string(),
            pmid: Some("24200969".to_string()),
            pmcid: Some("PMC9984800".to_string()),
            doi: None,
            title: "Follow-up trial".to_string(),
            authors: Vec::new(),
            author_count: 0,
            author_completeness: ArticleAuthorCompleteness::Unavailable,
            author_source: ArticleSource::EuropePmc,
            journal: Some("Nature".to_string()),
            year: Some(2014),
            entity_summary: None,
            tldr: None,
            citation_count: None,
            influential_citation_count: None,
        },
    ];

    let markdown = article_batch_markdown(&rows).expect("batch markdown");
    assert!(markdown.contains("# Article Batch (3)"));
    assert!(markdown.contains("## 1. Improved survival with vemurafenib"));
    assert!(markdown.contains("PMID: 22663011"));
    assert!(markdown.contains("Authors: A. One, B. Two, C. Three"));
    assert!(markdown.contains("Authorship: complete (3 returned; PubTator3)"));
    assert!(markdown.contains("Entities: Genes: BRAF (4); Diseases: melanoma (2)"));
    assert!(markdown.contains("TLDR: BRAF inhibitor benefit in melanoma."));
    assert!(markdown.contains("Citations: 120 (influential: 18)"));
    assert!(markdown.contains("## 2. Source-limited authors"));
    assert!(markdown.contains("Authors: First Author, Middle Author, Last Author"));
    assert!(markdown.contains("Authorship: source-limited (3 returned; Europe PMC)"));
    assert!(markdown.contains("## 3. Follow-up trial"));
    assert!(markdown.contains("PMID: 24200969"));
    assert!(markdown.contains("Authorship: unavailable (no author list supplied by Europe PMC)"));
    // Absent optional fields are omitted, not printed as placeholders
    assert!(!markdown.contains("TLDR: -"));
    assert!(!markdown.contains("Entities: -"));
}

#[test]
fn article_search_markdown_preserves_rank_order_and_shows_rationale() {
    let rows = vec![
        ArticleSearchResult {
            pmid: "1".into(),
            arxiv_id: None,
            semantic_scholar_id: None,
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
            arxiv_id: None,
            semantic_scholar_id: None,
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
            warning: None,
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
    assert!(markdown.contains("| Identifier | Title | Source(s) | Date | Why | Cit. |"));
    assert!(markdown.contains("PubTator3, Semantic Scholar"));
    assert!(markdown.contains("title 2/2"));
    assert!(markdown.contains("title+abstract 2/2"));
    assert!(
        markdown
            .contains("--date-from/--date-to <YYYY|YYYY-MM|YYYY-MM-DD> (alias: --since/--until)")
    );
    assert!(!markdown.contains("## PubTator3"));
    assert!(!markdown.contains("## Europe PMC"));
    assert!(markdown.find("|PMID 1|").unwrap() < markdown.find("|PMID 2|").unwrap());
}

#[test]
fn ticket_377_article_renderer_envelope_contracts_markdown_status() {
    let rows = vec![ArticleSearchResult {
        pmid: "22663011".into(),
        arxiv_id: None,
        semantic_scholar_id: None,
        title: "BRAF melanoma fixture".into(),
        pmcid: None,
        doi: None,
        journal: None,
        date: Some("2012-06-30".into()),
        first_index_date: None,
        citation_count: Some(42),
        influential_citation_count: None,
        source: ArticleSource::PubMed,
        matched_sources: vec![ArticleSource::PubMed, ArticleSource::SemanticScholar],
        score: None,
        is_retracted: Some(false),
        abstract_snippet: None,
        ranking: None,
        normalized_title: "braf melanoma fixture".into(),
        normalized_abstract: String::new(),
        publication_type: None,
        source_local_position: 0,
    }];
    let source_status = vec![crate::entities::article::ArticleSourceStatus {
        source: ArticleSource::SemanticScholar,
        enabled: true,
        auth_mode: Some(crate::sources::semantic_scholar::SemanticScholarAuthMode::SharedPool),
        status: Some(crate::entities::article::ArticleSourceAvailability::Degraded),
        message: None,
    }];
    let markdown = article_search_markdown_with_footer_and_context(
        "BRAF melanoma",
        &rows,
        "",
        &article_filters_for_test(ArticleSort::Relevance),
        ArticleSearchRenderContext {
            source_filter: crate::entities::article::ArticleSourceFilter::All,
            semantic_scholar_enabled: true,
            warning: None,
            note: None,
            debug_plan: None,
            exact_entity_commands: &[],
            source_status: &source_status,
        },
    )
    .expect("article_search_markdown_with_footer_and_context");
    assert!(markdown.contains("| Identifier | Title | Source(s) | Date | Why | Cit. |"));
    assert!(markdown.lines().any(|line| {
        line.contains("Semantic Scholar")
            && line.contains("degraded")
            && line.contains("shared_pool")
    }));
}

#[test]
fn article_search_markdown_renders_non_semantic_source_status() {
    let rows = vec![ArticleSearchResult {
        pmid: "41800001".into(),
        arxiv_id: None,
        semantic_scholar_id: None,
        title: "BRAF melanoma bounded federation fixture".into(),
        pmcid: None,
        doi: None,
        journal: None,
        date: Some("2026-01-01".into()),
        first_index_date: None,
        citation_count: None,
        influential_citation_count: None,
        source: ArticleSource::PubTator,
        matched_sources: vec![ArticleSource::PubTator],
        score: None,
        is_retracted: Some(false),
        abstract_snippet: None,
        ranking: None,
        normalized_title: "braf melanoma bounded federation fixture".into(),
        normalized_abstract: String::new(),
        publication_type: None,
        source_local_position: 0,
    }];
    let source_status = vec![crate::entities::article::ArticleSourceStatus {
        source: ArticleSource::EuropePmc,
        enabled: true,
        auth_mode: None,
        status: Some(crate::entities::article::ArticleSourceAvailability::Degraded),
        message: Some("Europe PMC timed out after 12s".to_string()),
    }];

    let markdown = article_search_markdown_with_footer_and_context(
        "BRAF melanoma",
        &rows,
        "",
        &article_filters_for_test(ArticleSort::Relevance),
        ArticleSearchRenderContext {
            source_filter: crate::entities::article::ArticleSourceFilter::All,
            semantic_scholar_enabled: true,
            warning: None,
            note: None,
            debug_plan: None,
            exact_entity_commands: &[],
            source_status: &source_status,
        },
    )
    .expect("article_search_markdown_with_footer_and_context");

    assert!(markdown.lines().any(|line| {
        line.contains("Europe PMC source status: degraded") && line.contains("timed out after 12s")
    }));
}

#[test]
fn article_ranking_why_tier1_mixed_shows_title_plus_abstract() {
    let row = ArticleSearchResult {
        pmid: "1".into(),
        arxiv_id: None,
        semantic_scholar_id: None,
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
        arxiv_id: None,
        semantic_scholar_id: None,
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
        arxiv_id: None,
        semantic_scholar_id: None,
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
        arxiv_id: None,
        semantic_scholar_id: None,
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
        arxiv_id: None,
        semantic_scholar_id: None,
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
            warning: None,
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
        arxiv_id: None,
        semantic_scholar_id: None,
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
            warning: None,
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
        arxiv_id: None,
        semantic_scholar_id: None,
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
            warning: None,
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
fn article_search_markdown_renders_each_typed_identifier() {
    let row = |pmid: &str,
               pmcid: Option<&str>,
               doi: Option<&str>,
               arxiv_id: Option<&str>,
               semantic_scholar_id: Option<&str>| ArticleSearchResult {
        pmid: pmid.into(),
        pmcid: pmcid.map(str::to_string),
        doi: doi.map(str::to_string),
        arxiv_id: arxiv_id.map(str::to_string),
        semantic_scholar_id: semantic_scholar_id.map(str::to_string),
        title: "Typed identifier row".into(),
        journal: None,
        date: None,
        first_index_date: None,
        citation_count: None,
        influential_citation_count: None,
        source: ArticleSource::SemanticScholar,
        matched_sources: vec![ArticleSource::SemanticScholar],
        score: None,
        is_retracted: None,
        abstract_snippet: None,
        ranking: None,
        normalized_title: "typed identifier row".into(),
        normalized_abstract: String::new(),
        publication_type: None,
        source_local_position: 0,
    };
    let rows = vec![
        row("1", None, None, None, None),
        row("", Some("PMC2"), None, None, None),
        row("", None, Some("10.1000/example"), None, None),
        row("", None, None, Some("2401.12345"), None),
        row("", None, None, None, Some("paper-5")),
    ];

    let markdown = article_search_markdown_with_footer_and_context(
        "sort=relevance",
        &rows,
        "",
        &article_filters_for_test(ArticleSort::Relevance),
        ArticleSearchRenderContext {
            source_filter: crate::entities::article::ArticleSourceFilter::SemanticScholar,
            semantic_scholar_enabled: true,
            warning: None,
            note: None,
            debug_plan: None,
            exact_entity_commands: &[],
            source_status: &[],
        },
    )
    .expect("typed identifier markdown");

    for identifier in [
        "PMID 1",
        "PMCID PMC2",
        "DOI 10.1000/example",
        "arXiv 2401.12345",
        "Semantic Scholar paper-5",
    ] {
        assert!(
            markdown.contains(identifier),
            "missing {identifier}: {markdown}"
        );
    }
}

#[test]
fn article_search_markdown_renders_date_sort_warning() {
    let markdown = article_search_markdown_with_footer_and_context(
        "sort=date",
        &[],
        "",
        &article_filters_for_test(ArticleSort::Date),
        ArticleSearchRenderContext {
            source_filter: crate::entities::article::ArticleSourceFilter::All,
            semantic_scholar_enabled: false,
            warning: Some(
                "Date sort replaces relevance ranking; results are ordered by publication date.",
            ),
            note: None,
            debug_plan: None,
            exact_entity_commands: &[],
            source_status: &[],
        },
    )
    .expect("date warning markdown");

    assert!(markdown.contains("> Warning: Date sort replaces relevance ranking"));
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
        arxiv_id: None,
        semantic_scholar_id: None,
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
            warning: None,
            note: None,
            debug_plan: None,
            exact_entity_commands: &[],
            source_status: &[],
        },
    )
    .expect("markdown should render");

    assert!(!markdown.contains("Newest indexed:"));
}
