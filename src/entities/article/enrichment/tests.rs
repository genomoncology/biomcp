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
        authors: None,
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

#[serial_test::serial(source_env)]
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
    let mut env = TestEnv::scratch_cache_only();
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

/// Capture every event whose `operation` names the visible-row fallback,
/// with its level, from one thread-default subscriber covering the body,
/// and hand back the body's own output beside them.
async fn capture_visible_fallback_events<T>(
    body: impl std::future::Future<Output = T>,
) -> (T, Vec<(tracing::Level, String)>) {
    use std::sync::{Arc, Mutex};
    use tracing::Event;
    use tracing::field::{Field, Visit};
    use tracing_subscriber::layer::{Context, SubscriberExt};
    use tracing_subscriber::{Layer, Registry};

    type Captured = Arc<Mutex<Vec<(tracing::Level, String)>>>;
    #[derive(Clone, Default)]
    struct Capture {
        events: Captured,
    }
    #[derive(Default)]
    struct Operation(String);
    impl Visit for Operation {
        fn record_str(&mut self, field: &Field, value: &str) {
            if field.name() == "operation" {
                self.0 = value.to_string();
            }
        }
        fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
            if field.name() == "operation" {
                self.0 = format!("{value:?}");
            }
        }
    }
    impl<S: tracing::Subscriber> Layer<S> for Capture {
        fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
            let mut operation = Operation::default();
            event.record(&mut operation);
            if operation.0 == "visible article metadata fallback" {
                self.events
                    .lock()
                    .expect("visible fallback capture")
                    .push((*event.metadata().level(), operation.0));
            }
        }
    }

    let capture = Capture::default();
    let events = Arc::clone(&capture.events);
    let subscriber = Registry::default().with(capture);
    let _guard = tracing::subscriber::set_default(subscriber);
    let output = body.await;
    let events = events.lock().expect("visible fallback capture").clone();
    (output, events)
}

#[serial_test::serial(source_env)]
#[tokio::test]
async fn metadata_fallback_warns_only_when_no_source_covers_the_row() {
    for variant in [false, true] {
        for covered in [true, false] {
            check_metadata_fallback(variant, covered).await;
        }
    }
}

async fn check_metadata_fallback(variant: bool, covered: bool) {
    // The trigger this ticket names, confirmed: PubTator answers 200 with
    // an empty PubTator3 array for a PMID the search itself returned, so
    // the resolve fails without the HTTP 400/404 lag fallback and the row
    // used to warn. Europe PMC still holds the row, so the fallback works:
    // the failure logs at debug and stderr stays empty of warnings.
    use crate::entities::SearchPage;
    use std::sync::{Arc, Mutex};

    let europepmc_search = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/testdata/sources/europepmc/search_pmid_20516115.json"
    ));
    let requests = Arc::new(Mutex::new(Vec::<String>::new()));
    let logged = requests.clone();
    let fixture = TestHttpFixture::spawn(move |request| {
        let target = request
            .split_whitespace()
            .nth(1)
            .unwrap_or_default()
            .to_string();
        logged.lock().unwrap().push(target.clone());
        if target.starts_with("/publications/export/biocjson") {
            TestHttpReply::Bytes(test_http_response(
                "200 OK",
                "application/json",
                br#"{"PubTator3":[]}"#,
            ))
        } else if target.starts_with("/search") {
            TestHttpReply::Bytes(test_http_response(
                "200 OK",
                "application/json",
                if covered {
                    europepmc_search.as_bytes()
                } else {
                    br#"{"hitCount":0,"resultList":{"result":[]}}"#
                },
            ))
        } else {
            TestHttpReply::Bytes(test_http_response(
                "404 Not Found",
                "application/json",
                b"{}",
            ))
        }
    })
    .await;
    let mut env = TestEnv::scratch_cache_only();
    let cache = crate::test_support::TempDirGuard::new("article-pubtator-empty");
    env.set_cache_root(cache.path());
    env.set("BIOMCP_PUBTATOR_BASE", &fixture.base);
    env.set("BIOMCP_EUROPEPMC_BASE", &fixture.base);

    let page = SearchPage {
        results: vec![row_with(
            "20516115",
            ArticleSource::PubMed,
            Some("2010-05-27"),
            None,
            None,
        )],
        total: Some(1),
        next_page_token: None,
        upstream_total: None,
        partial_note: None,
    };
    let (rows, events) = capture_visible_fallback_events(async {
        if variant {
            let mut rows = page.results;
            let execution = super::super::variant_search::VariantArticleExecutionContext::single();
            enrich_visible_article_search_rows_with_article_base_context(
                &mut rows,
                Some(&execution),
            )
            .await;
            rows
        } else {
            enrich_visible_article_search_page(page, &[ArticleSource::PubTator])
                .await
                .page
                .results
        }
    })
    .await;
    let warns: Vec<_> = events
        .iter()
        .filter(|(level, _)| *level == tracing::Level::WARN)
        .cloned()
        .collect();
    assert_eq!(
        warns.is_empty(),
        covered,
        "variant={variant}, covered={covered}: {events:?}"
    );
    if covered {
        assert!(
            events
                .iter()
                .any(|(level, _)| *level == tracing::Level::DEBUG),
            "a covered failure logs at debug: {events:?}"
        );
    }
    assert_eq!(rows[0].citation_count, covered.then_some(133));
    let requests = requests.lock().unwrap().clone();
    assert!(
        requests
            .iter()
            .any(|target| target.starts_with("/search?query=EXT_ID%3A20516115")),
        "Europe PMC is consulted (the colon arrives percent-encoded): {requests:?}"
    );
}
