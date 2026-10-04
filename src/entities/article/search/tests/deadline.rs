use super::*;

use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use crate::test_support::TempDirGuard;

/// One fixture origin stands in for every article provider. The request
/// target distinguishes them: Europe PMC searches carry `query=`, PubTator3
/// searches carry `text=`, PubMed hits its eutils paths, and Semantic Scholar
/// keeps its `/graph/v1` routes. `europepmc_hold` is the only reply that never
/// resolves on its own; the test owns the sender side as the release signal.
fn fixture_reply(
    request: &str,
    europepmc_hold: Option<&Arc<Mutex<mpsc::Receiver<()>>>>,
) -> TestHttpReply {
    let target = request
        .split_whitespace()
        .nth(1)
        .expect("fixture request line carries a target");
    if target.starts_with("/search") && target.contains("query=") {
        return match europepmc_hold {
            Some(hold) => TestHttpReply::Hold(Arc::clone(hold)),
            None => TestHttpReply::Bytes(test_http_response(
                "200 OK",
                "application/json",
                br#"{"hitCount":1,"resultList":{"result":[{"id":"41800004","pmid":"41800004","title":"deadline fixture Europe PMC row","journalTitle":"Fixture Journal","firstPublicationDate":"2026-01-02","authorString":"Fixture Author"}]}}"#,
            )),
        };
    }
    let body = if target.starts_with("/search/") && target.contains("text=") {
        br#"{"results":[{"_id":"pt-418","pmid":41800001,"title":"deadline fixture PubTator row","journal":"Fixture Journal","date":"2026-01-01","score":42.0}],"count":1,"total_pages":1,"current":1,"page_size":25,"facets":{}}"#.as_slice()
    } else if target.ends_with("/esearch.fcgi") {
        br#"{"esearchresult":{"count":"1","idlist":["41800002"]}}"#.as_slice()
    } else if target.ends_with("/esummary.fcgi") {
        br#"{"result":{"uids":["41800002"],"41800002":{"uid":"41800002","title":"deadline fixture PubMed row","sortpubdate":"2026/01/02 00:00","pubdate":"2026 Jan 2","fulljournalname":"Fixture Journal","source":"Fixture Journal"}}}"#
            .as_slice()
    } else if target.starts_with("/graph/v1/paper/batch") {
        br#"[null,null,null,null,null,null,null,null,null,null]"#.as_slice()
    } else if target.starts_with("/graph/v1/paper/search") {
        br#"{"total":1,"data":[{"paperId":"fixture-s2-paper","externalIds":{"PubMed":"41800003"},"title":"deadline fixture Semantic Scholar row","venue":"Fixture Journal","year":2026,"citationCount":7,"influentialCitationCount":1,"abstract":"deadline fixture abstract."}]}"#
            .as_slice()
    } else if target.starts_with("/publications/export/biocjson") {
        br#"{"documents":[]}"#.as_slice()
    } else {
        return TestHttpReply::Bytes(test_http_response(
            "404 Not Found",
            "application/json",
            b"{}",
        ));
    };
    TestHttpReply::Bytes(test_http_response("200 OK", "application/json", body))
}

fn deadline_filters() -> ArticleSearchFilters {
    ArticleSearchFilters {
        keyword: Some("deadline fixture".into()),
        exclude_retracted: true,
        ..empty_filters()
    }
}

fn deadline_env(
    fixture: &TestHttpFixture,
    cache_root: &std::path::Path,
    deadline_ms: &str,
) -> TestEnv {
    let mut env = TestEnv::new();
    for (key, value) in [
        ("BIOMCP_TEST_UNPACED_ORIGIN", fixture.base.clone()),
        ("BIOMCP_S2_BASE", fixture.base.clone()),
        ("BIOMCP_PUBTATOR_BASE", fixture.base.clone()),
        ("BIOMCP_EUROPEPMC_BASE", fixture.base.clone()),
        (
            "BIOMCP_PUBMED_BASE",
            format!("{}/entrez/eutils", fixture.base),
        ),
        (
            "BIOMCP_CACHE_DIR",
            cache_root.to_string_lossy().into_owned(),
        ),
        (
            "BIOMCP_TEST_ARTICLE_SEARCH_DEADLINE_MS",
            deadline_ms.to_string(),
        ),
    ] {
        env.set(key, value);
    }
    env
}

#[serial_test::serial(source_env)]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn overall_deadline_returns_partial_rows_and_names_the_held_source() {
    // The sender stays alive for the whole search: the Europe PMC reply is
    // only released when the test drops it.
    let (_release_tx, hold_rx) = mpsc::channel::<()>();
    let hold = Arc::new(Mutex::new(hold_rx));
    let fixture = TestHttpFixture::spawn(move |request| fixture_reply(request, Some(&hold))).await;
    let cache = TempDirGuard::new("article-search-deadline");
    let _env = deadline_env(&fixture, cache.path(), "4000");

    // watchdog: the deadline must settle the search in real time; a broken
    // bound hangs here instead of returning partial rows.
    let page = tokio::time::timeout(
        crate::test_support::watchdog(60),
        search_page(&deadline_filters(), 5, 0, ArticleSourceFilter::All),
    )
    .await
    .expect("article search exceeds its watchdog")
    .expect("partial page on deadline");

    assert!(
        page.results
            .iter()
            .any(|row| matches!(row.pmid.as_str(), "41800001" | "41800002" | "41800003")),
        "answered sources keep their rows past the deadline: {:?}",
        page.results
            .iter()
            .map(|row| row.pmid.as_str())
            .collect::<Vec<_>>()
    );
    let europepmc = page
        .source_status
        .iter()
        .find(|status| status.source == ArticleSource::EuropePmc)
        .expect("held Europe PMC leg is reported");
    assert_eq!(europepmc.status, Some(ArticleSourceAvailability::Degraded));
    assert!(
        europepmc
            .message
            .as_deref()
            .is_some_and(|message| message.contains("deadline")),
        "the degraded message names the deadline: {europepmc:?}"
    );
    assert_eq!(page.diagnostics.deadline_ms, 4000);
    assert!(
        page.diagnostics
            .source_timings
            .iter()
            .any(|timing| timing.source == Some(ArticleSource::EuropePmc)),
        "per-source timings cover the held leg"
    );
}

#[serial_test::serial(source_env)]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn failed_primaries_still_return_answered_rows() {
    let (_release_tx, hold_rx) = mpsc::channel::<()>();
    let hold = Arc::new(Mutex::new(hold_rx));
    // Both primaries fail fast; only the auxiliary sources answer.
    let fixture = TestHttpFixture::spawn(move |request| {
        let target = request
            .split_whitespace()
            .nth(1)
            .expect("fixture request line carries a target");
        if target.starts_with("/search") {
            return TestHttpReply::Bytes(test_http_response(
                "503 Service Unavailable",
                "application/json",
                b"{}",
            ));
        }
        fixture_reply(request, Some(&hold))
    })
    .await;
    let cache = TempDirGuard::new("article-search-partial");
    let _env = deadline_env(&fixture, cache.path(), "60000");

    let page = tokio::time::timeout(
        crate::test_support::watchdog(60),
        search_page(&deadline_filters(), 5, 0, ArticleSourceFilter::All),
    )
    .await
    .expect("article search exceeds its watchdog")
    .expect("partial rows survive two failed primaries");

    assert!(
        page.results
            .iter()
            .any(|row| matches!(row.pmid.as_str(), "41800002" | "41800003")),
        "PubMed/Semantic Scholar rows survive the failed primaries: {:?}",
        page.results
            .iter()
            .map(|row| row.pmid.as_str())
            .collect::<Vec<_>>()
    );
    for source in [ArticleSource::PubTator, ArticleSource::EuropePmc] {
        let status = page
            .source_status
            .iter()
            .find(|status| status.source == source)
            .unwrap_or_else(|| panic!("failed {source:?} leg is reported"));
        assert!(
            matches!(
                status.status,
                Some(ArticleSourceAvailability::Degraded | ArticleSourceAvailability::Unavailable)
            ),
            "{source:?} leg reports a failure status: {status:?}"
        );
    }
}

#[serial_test::serial(source_env)]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn healthy_federated_search_keeps_output_shape_and_records_timings() {
    let fixture = TestHttpFixture::spawn(move |request| fixture_reply(request, None)).await;
    let cache = TempDirGuard::new("article-search-healthy");
    let _env = deadline_env(&fixture, cache.path(), "60000");

    let page = tokio::time::timeout(
        crate::test_support::watchdog(60),
        search_page(&deadline_filters(), 5, 0, ArticleSourceFilter::All),
    )
    .await
    .expect("article search exceeds its watchdog")
    .expect("healthy federated page");

    assert!(!page.results.is_empty());
    assert_eq!(page.diagnostics.deadline_ms, 60000);
    for source in [
        ArticleSource::PubTator,
        ArticleSource::EuropePmc,
        ArticleSource::PubMed,
        ArticleSource::SemanticScholar,
    ] {
        assert!(
            page.diagnostics
                .source_timings
                .iter()
                .any(|timing| timing.source == Some(source) && timing.stage == "search"),
            "timing recorded for {source:?}"
        );
    }
}

#[serial_test::serial(source_env)]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn explicit_semantic_scholar_deadline_is_unavailable_but_successful_empty_is_not() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    let wrapped = BioMcpError::Api {
        api: "semanticscholar".into(),
        message: "invocation deadline exceeded".into(),
    }
    .with_source_context(crate::error::SourceContext::retry(
        crate::error::SourceProvider::SEMANTIC_SCHOLAR,
    ));
    assert!(is_search_deadline_error(&wrapped));
    assert!(!is_search_deadline_error(&BioMcpError::Api {
        api: "semanticscholar".into(),
        message: "HTTP 503".into(),
    }));
    for held in [false, true] {
        let (_release_tx, hold_rx) = mpsc::channel::<()>();
        let hold = Arc::new(Mutex::new(hold_rx));
        let hits = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&hits);
        let fixture = TestHttpFixture::spawn(move |request| {
            assert!(request.starts_with("GET /graph/v1/paper/search?"));
            observed.fetch_add(1, Ordering::SeqCst);
            if held {
                TestHttpReply::Hold(Arc::clone(&hold))
            } else {
                TestHttpReply::Bytes(test_http_response(
                    "200 OK",
                    "application/json",
                    br#"{"total":0,"data":[]}"#,
                ))
            }
        })
        .await;
        let cache = TempDirGuard::new("article-search-explicit-s2-deadline");
        let _env = deadline_env(&fixture, cache.path(), "4000");
        let result = tokio::time::timeout(
            crate::test_support::watchdog(60),
            search_page(&deadline_filters(), 5, 0, ArticleSourceFilter::SemanticScholar),
        )
        .await
        .expect("explicit search settles within its watchdog");
        assert!(hits.load(Ordering::SeqCst) > 0, "the request reached the fixture");
        if held {
            let BioMcpError::SourceUnavailable {
                source_name,
                reason,
                suggestion,
            } =
                result.expect_err("zero answered rows on invocation expiry")
            else {
                panic!("expiry must be retryable SourceUnavailable");
            };
            assert_eq!(source_name, "article search");
            assert!(reason.contains("deadline"));
            assert!(suggestion.contains("Retry"));
        } else {
            let page = result.expect("a successful empty search stays successful");
            assert!(page.results.is_empty());
            assert_eq!(page.source_status.len(), 1);
            assert_eq!(page.source_status[0].status, Some(ArticleSourceAvailability::Ok));
        }
    }
}

#[serial_test::serial(source_env)]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn in_flight_semantic_scholar_enrichment_deadline_retains_rows_and_names_source() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use crate::entities::article::enrichment::enrich_article_search_rows_with_semantic_scholar;

    for held in [false, true] {
        let (_release_tx, hold_rx) = mpsc::channel::<()>();
        let hold = Arc::new(Mutex::new(hold_rx));
        let hits = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&hits);
        let fixture = TestHttpFixture::spawn(move |request| {
            assert!(request.starts_with("POST /graph/v1/paper/batch?"));
            observed.fetch_add(1, Ordering::SeqCst);
            if held {
                TestHttpReply::Hold(Arc::clone(&hold))
            } else {
                TestHttpReply::Bytes(test_http_response("200 OK", "application/json", b"[null]"))
            }
        })
        .await;
        let cache = TempDirGuard::new("article-search-s2-enrichment-deadline");
        let _env = deadline_env(&fixture, cache.path(), "4000");
        let mut rows = vec![row("41800002", ArticleSource::PubMed)];
        let retained = serde_json::to_value(&rows).unwrap();
        let deadline = crate::sources::VariantArticleDeadline::from_now(Duration::from_secs(4));
        let status = tokio::time::timeout(
            crate::test_support::watchdog(60),
            crate::sources::with_variant_article_deadline(
                deadline,
                enrich_article_search_rows_with_semantic_scholar(&mut rows),
            ),
        )
        .await
        .expect("enrichment settles within its watchdog")
        .expect("the attempted batch has a status");
        assert!(hits.load(Ordering::SeqCst) > 0, "the batch reached the fixture");
        assert_eq!(serde_json::to_value(&rows).unwrap(), retained);
        assert_eq!(status.source, ArticleSource::SemanticScholar);
        if held {
            assert_eq!(status.status, Some(ArticleSourceAvailability::Degraded));
            assert!(status.message.as_deref().unwrap().contains("deadline"));
        } else {
            assert_eq!(status.status, Some(ArticleSourceAvailability::Ok));
            assert!(status.message.is_none(), "successful null enrichment is not a failure");
        }
    }
}

#[serial_test::serial(source_env)]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn final_metadata_fallback_deadline_retains_row_and_names_both_consulted_sources() {
    use crate::entities::article::enrichment::enrich_visible_article_search_rows_with_article_base;

    for held in [false, true] {
        let (_release_tx, hold_rx) = mpsc::channel::<()>();
        let hold = Arc::new(Mutex::new(hold_rx));
        let requests = Arc::new(Mutex::new(Vec::new()));
        let observed = Arc::clone(&requests);
        let fixture = TestHttpFixture::spawn(move |request| {
            if request.starts_with("GET /publications/export/biocjson?") {
                observed.lock().unwrap().push(ArticleSource::PubTator);
                if held {
                    // A lag response consults Europe PMC before its in-flight expiry.
                    TestHttpReply::Bytes(test_http_response(
                        "404 Not Found",
                        "application/json",
                        b"{}",
                    ))
                } else {
                    TestHttpReply::Bytes(test_http_response(
                        "200 OK",
                        "application/json",
                        br#"{"PubTator3":[]}"#,
                    ))
                }
            } else {
                assert!(held && request.starts_with("GET /search?"));
                observed.lock().unwrap().push(ArticleSource::EuropePmc);
                TestHttpReply::Hold(Arc::clone(&hold))
            }
        })
        .await;
        let cache = TempDirGuard::new("article-search-final-metadata-deadline");
        let _env = deadline_env(&fixture, cache.path(), "4000");
        // The only row is also the final row: a next-iteration check cannot cover it.
        let mut rows = vec![row("41800002", ArticleSource::PubMed)];
        let retained = serde_json::to_value(&rows).unwrap();
        let deadline = crate::sources::VariantArticleDeadline::from_now(Duration::from_secs(4));
        let statuses = tokio::time::timeout(
            crate::test_support::watchdog(60),
            crate::sources::with_variant_article_deadline(
                deadline,
                enrich_visible_article_search_rows_with_article_base(&mut rows),
            ),
        )
        .await
        .expect("final metadata fallback settles within its watchdog");
        assert_eq!(serde_json::to_value(&rows).unwrap(), retained);
        if held {
            assert_eq!(
                *requests.lock().unwrap(),
                vec![ArticleSource::PubTator, ArticleSource::EuropePmc]
            );
            assert_eq!(statuses.len(), 2);
            for (status, source) in statuses
                .iter()
                .zip([ArticleSource::PubTator, ArticleSource::EuropePmc])
            {
                assert_eq!(status.source, source);
                assert_eq!(status.status, Some(ArticleSourceAvailability::Degraded));
                assert!(status.message.as_deref().unwrap().contains("deadline"));
            }
        } else {
            assert_eq!(*requests.lock().unwrap(), vec![ArticleSource::PubTator]);
            assert!(statuses.is_empty(), "successful empty detail is not deadline degradation");
        }
    }
}
