use super::super::test_support::*;
use super::*;
use crate::sources::semantic_scholar::SemanticScholarPaper;

fn semantic_paper(
    citation_count: Option<u64>,
    abstract_text: Option<&str>,
) -> SemanticScholarPaper {
    SemanticScholarPaper {
        paper_id: Some("paper-8896569".into()),
        external_ids: None,
        title: None,
        venue: None,
        year: None,
        citation_count,
        influential_citation_count: Some(17),
        abstract_text: abstract_text.map(str::to_string),
        reference_count: None,
        is_open_access: None,
        open_access_pdf: None,
        tldr: None,
    }
}

#[test]
fn semantic_scholar_merge_preserves_existing_nonempty_primary_metadata() {
    let primary_abstract = "Primary source abstract.";
    let primary_normalized =
        crate::transform::article::normalize_article_search_text(primary_abstract);
    let mut row = ArticleSearchResult {
        abstract_snippet: Some(primary_abstract.into()),
        normalized_abstract: primary_normalized.clone(),
        ..row_with(
            "8896569",
            ArticleSource::PubMed,
            Some("1996-10-24"),
            Some(99),
            Some(false),
        )
    };

    merge_article_search_row_with_semantic_scholar(
        &mut row,
        &semantic_paper(Some(231), Some("Semantic Scholar replacement abstract.")),
    );

    assert_eq!(row.citation_count, Some(99));
    assert_eq!(row.influential_citation_count, Some(17));
    assert_eq!(row.abstract_snippet.as_deref(), Some(primary_abstract));
    assert_eq!(row.normalized_abstract, primary_normalized);
}

#[test]
fn semantic_scholar_merge_fills_missing_citation_and_abstract_metadata() {
    let mut row = row_with(
        "8896569",
        ArticleSource::PubMed,
        Some("1996-10-24"),
        None,
        Some(false),
    );

    merge_article_search_row_with_semantic_scholar(
        &mut row,
        &semantic_paper(
            Some(231),
            Some("Glial cell line-derived neurotrophic factor signaling through RET is increased."),
        ),
    );

    assert_eq!(row.citation_count, Some(231));
    assert_eq!(row.influential_citation_count, Some(17));
    assert!(
        row.abstract_snippet
            .as_deref()
            .is_some_and(|value| value.contains("Glial cell"))
    );
    assert!(row.normalized_abstract.contains("glial cell"));
}

#[test]
fn semantic_scholar_merge_treats_zero_citation_as_missing() {
    let mut row = row_with(
        "8896569",
        ArticleSource::EuropePmc,
        Some("1996-10-24"),
        Some(0),
        Some(false),
    );

    merge_article_search_row_with_semantic_scholar(&mut row, &semantic_paper(Some(231), None));

    assert_eq!(row.citation_count, Some(231));
    assert_eq!(row.influential_citation_count, Some(17));
}

#[test]
fn article_base_merge_fills_abstract_when_semantic_scholar_has_none() {
    let mut row = row_with(
        "8896569",
        ArticleSource::PubMed,
        Some("1996-10-24"),
        Some(231),
        Some(false),
    );
    let article = Article {
        section_outcomes: crate::entities::section_outcome::SectionOutcomes::with_keys(
            crate::entities::article::ARTICLE_OUTCOME_KEYS,
        ),
        pmid: Some("8896569".into()),
        pmcid: None,
        doi: None,
        title: "RET study".into(),
        authors: Vec::new(),
        author_count: 0,
        author_completeness: crate::entities::article::ArticleAuthorCompleteness::Unavailable,
        author_source: ArticleSource::PubTator,
        journal: Some("Nature".into()),
        date: Some("1996-10-24".into()),
        citation_count: Some(250),
        publication_type: None,
        open_access: None,
        abstract_text: Some(
            "Hirschsprung disease is a congenital malformation of the enteric nervous system."
                .into(),
        ),
        full_text_path: None,
        full_text_note: None,
        full_text_source: None,
        full_text_manifest: None,
        full_text_coverage: None,
        not_included: None,
        europepmc_license: None,
        europepmc_retracted: None,
        annotations: None,
        indexing: None,
        semantic_scholar: None,
        pubtator_fallback: false,
    };

    merge_article_search_row_with_article_base(&mut row, &article);

    assert_eq!(row.citation_count, Some(231));
    assert!(
        row.abstract_snippet
            .as_deref()
            .is_some_and(|value| value.contains("Hirschsprung"))
    );
    assert!(row.normalized_abstract.contains("hirschsprung"));
}

#[serial_test::serial(article_resolver_env)]
#[tokio::test]
async fn empty_enrichment_plan_makes_no_provider_requests() {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    let hits = Arc::new(AtomicUsize::new(0));
    let observed = hits.clone();
    let fixture = TestHttpFixture::spawn(move |_| {
        observed.fetch_add(1, Ordering::SeqCst);
        TestHttpReply::Bytes(test_http_response("200 OK", "application/json", b"[null]"))
    })
    .await;
    let mut env = TestEnv::new();
    for key in [
        "BIOMCP_TEST_UNPACED_ORIGIN",
        "BIOMCP_S2_BASE",
        "BIOMCP_PUBTATOR_BASE",
        "BIOMCP_EUROPEPMC_BASE",
    ] {
        env.set(key, &fixture.base);
    }
    let rows = vec![row_with("8896569", ArticleSource::PubMed, None, None, None)];
    enrich_and_finalize_article_candidates(rows, 1, 0, None, &empty_filters(), &[]).await;
    assert_eq!(hits.load(Ordering::SeqCst), 0);
}
