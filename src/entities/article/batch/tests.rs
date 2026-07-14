#[allow(unused_imports)]
use super::super::test_support::*;
use super::*;

#[test]
fn article_batch_item_projection_keeps_requested_id_year_and_top_entities() {
    let article = Article {
        pmid: Some("22663011".to_string()),
        pmcid: Some("PMC9984800".to_string()),
        doi: Some("10.1056/NEJMoa1203421".to_string()),
        title: " Improved survival with vemurafenib ".to_string(),
        authors: vec!["A".into(), "B".into(), "C".into(), "D".into(), "E".into()],
        author_count: 5,
        author_completeness: ArticleAuthorCompleteness::Complete,
        author_source: ArticleSource::PubTator,
        journal: Some("NEJM".to_string()),
        date: Some("2012-06-07".to_string()),
        citation_count: Some(77),
        publication_type: None,
        open_access: None,
        abstract_text: None,
        full_text_path: None,
        full_text_note: None,
        full_text_source: None,
        full_text_manifest: None,
        not_included: None,
        europepmc_license: None,
        europepmc_retracted: None,
        annotations: Some(ArticleAnnotations {
            genes: vec![
                AnnotationCount {
                    text: "BRAF".to_string(),
                    count: 4,
                },
                AnnotationCount {
                    text: "NRAS".to_string(),
                    count: 3,
                },
                AnnotationCount {
                    text: "MAP2K1".to_string(),
                    count: 2,
                },
                AnnotationCount {
                    text: "PTEN".to_string(),
                    count: 1,
                },
            ],
            diseases: vec![AnnotationCount {
                text: "melanoma".to_string(),
                count: 2,
            }],
            chemicals: vec![AnnotationCount {
                text: "vemurafenib".to_string(),
                count: 2,
            }],
            mutations: vec![AnnotationCount {
                text: "V600E".to_string(),
                count: 3,
            }],
        }),
        indexing: None,
        semantic_scholar: Some(ArticleSemanticScholar {
            paper_id: Some("paper-1".to_string()),
            tldr: Some("BRAF inhibitor benefit in melanoma.".to_string()),
            citation_count: Some(120),
            influential_citation_count: Some(18),
            reference_count: None,
            is_open_access: None,
            open_access_pdf: None,
        }),
        pubtator_fallback: false,
    };

    let item = article_batch_item_from_article(" 10.1056/NEJMoa1203421 ", &article);
    assert_eq!(item.requested_id, "10.1056/NEJMoa1203421");
    assert_eq!(item.pmid.as_deref(), Some("22663011"));
    assert_eq!(item.pmcid.as_deref(), Some("PMC9984800"));
    assert_eq!(item.doi.as_deref(), Some("10.1056/NEJMoa1203421"));
    assert_eq!(item.title, "Improved survival with vemurafenib");
    assert_eq!(item.journal.as_deref(), Some("NEJM"));
    assert_eq!(item.year, Some(2012));
    assert_eq!(item.authors, article.authors);
    assert_eq!(item.author_count, item.authors.len());
    assert_eq!(
        item.author_completeness,
        ArticleAuthorCompleteness::Complete
    );
    assert_eq!(item.author_source, ArticleSource::PubTator);
    assert_eq!(item.tldr, None);
    assert_eq!(item.citation_count, None);
    assert_eq!(item.influential_citation_count, None);

    let entity_summary = item.entity_summary.expect("entity summary");
    assert_eq!(entity_summary.genes.len(), 3);
    assert_eq!(
        entity_summary
            .genes
            .iter()
            .map(|row| row.text.as_str())
            .collect::<Vec<_>>(),
        vec!["BRAF", "NRAS", "MAP2K1"]
    );
    assert_eq!(entity_summary.diseases[0].text, "melanoma");
    assert_eq!(entity_summary.chemicals[0].text, "vemurafenib");
    assert_eq!(entity_summary.mutations[0].text, "V600E");
}

#[tokio::test]
async fn article_batch_rejects_more_than_max_ids_before_network() {
    let ids = (0..ARTICLE_BATCH_MAX_IDS + 1)
        .map(|idx| format!("{}", 22000000 + idx))
        .collect::<Vec<_>>();

    let err = get_batch_compact(&ids)
        .await
        .expect_err("batch over the max should fail");
    assert_eq!(
        err.to_string(),
        format!("Invalid argument: Article batch is limited to {ARTICLE_BATCH_MAX_IDS} IDs")
    );
}

#[test]
fn batch_semantic_scholar_merge_fills_fields_and_skips_none_rows_and_pmcid_only() {
    use crate::sources::semantic_scholar::{SemanticScholarPaper, SemanticScholarTldr};

    fn blank_item(requested_id: &str) -> ArticleBatchItem {
        ArticleBatchItem {
            requested_id: requested_id.to_string(),
            pmid: None,
            pmcid: None,
            doi: None,
            title: String::new(),
            authors: Vec::new(),
            author_count: 0,
            author_completeness: ArticleAuthorCompleteness::Unavailable,
            author_source: ArticleSource::PubTator,
            journal: None,
            year: None,
            entity_summary: None,
            tldr: None,
            citation_count: None,
            influential_citation_count: None,
        }
    }

    let mut items = vec![
        ArticleBatchItem {
            pmid: Some("22663011".to_string()),
            ..blank_item("22663011")
        },
        // PMCID-only: not in the S2 lookup list (no PMID or DOI)
        ArticleBatchItem {
            pmcid: Some("PMC9984800".to_string()),
            ..blank_item("PMC9984800")
        },
        // Second PMID lookup — S2 returns None (paper not found)
        ArticleBatchItem {
            pmid: Some("00000000".to_string()),
            ..blank_item("00000000")
        },
    ];

    // positions 0 and 2 have PMIDs; position 1 is PMCID-only and not looked up
    let item_positions = vec![0usize, 2usize];
    let rows: Vec<Option<SemanticScholarPaper>> = vec![
        Some(SemanticScholarPaper {
            tldr: Some(SemanticScholarTldr {
                text: Some("  Compact summary  ".to_string()),
                model: None,
            }),
            citation_count: Some(120),
            influential_citation_count: Some(18),
            ..Default::default()
        }),
        None, // S2 returned no match for position 2
    ];

    merge_semantic_scholar_compact_rows(&mut items, &item_positions, rows);

    // Item 0: enriched
    assert_eq!(items[0].tldr.as_deref(), Some("Compact summary")); // whitespace trimmed
    assert_eq!(items[0].citation_count, Some(120));
    assert_eq!(items[0].influential_citation_count, Some(18));
    assert_eq!(
        items[0].author_completeness,
        ArticleAuthorCompleteness::Unavailable
    );
    assert_eq!(items[0].author_source, ArticleSource::PubTator);

    // Item 1: PMCID-only, not in lookup, untouched
    assert_eq!(items[1].tldr, None);
    assert_eq!(items[1].citation_count, None);

    // Item 2: None row, fields stay unset
    assert_eq!(items[2].tldr, None);
    assert_eq!(items[2].citation_count, None);
}
