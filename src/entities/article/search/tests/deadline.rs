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
    } else if target.split('?').next().unwrap().ends_with("/esearch.fcgi") {
        br#"{"esearchresult":{"count":"1","idlist":["41800002"]}}"#.as_slice()
    } else if target.split('?').next().unwrap().ends_with("/esummary.fcgi") {
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

// Keep the paused runtime runnable while filesystem and socket workers signal
// progress. Only the test's explicit advance may consume invocation time.
fn clock_driver() -> (Arc<std::sync::atomic::AtomicBool>, tokio::task::JoinHandle<()>) {
    let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let active = Arc::clone(&running);
    let task = tokio::spawn(async move {
        while active.load(std::sync::atomic::Ordering::SeqCst) {
            tokio::task::yield_now().await;
        }
    });
    (running, task)
}

#[serial_test::serial(source_env)]
#[tokio::test(start_paused = true)]
async fn contended_cache_constructor_deadline_and_success_preserve_search_and_enrichment() {
    use fs2::FileExt;
    use crate::entities::article::enrichment::enrich_article_search_rows_with_semantic_scholar;

    for enrichment in [false, true] {
        for expired in [false, true] {
            let (running, driver) = clock_driver();
            let requests = Arc::new(Mutex::new(Vec::new()));
            let observed = Arc::clone(&requests);
            let fixture = TestHttpFixture::spawn(move |request| {
                observed.lock().unwrap().push(request.split_whitespace().nth(1).unwrap().to_string());
                TestHttpReply::Bytes(test_http_response("200 OK", "application/json",
                    if enrichment { b"[null]" } else { br#"{"total":0,"data":[]}"# }))
            }).await;
            let cache = TempDirGuard::new("article-constructor-contention");
            std::fs::create_dir_all(cache.path()).unwrap();
            let held = std::fs::OpenOptions::new().create(true).truncate(false).read(true).write(true)
                .open(cache.path().join(".body-limit-cache-v1.lock")).unwrap();
            held.lock_exclusive().unwrap();
            let _env = deadline_env(&fixture, cache.path(), "60000");
            let contended = Arc::new(tokio::sync::Notify::new());
            let signal = Arc::clone(&contended);
            let cache_root = cache.path().to_path_buf();
            let mut task = tokio::spawn(crate::cache::migration::with_epoch_lock_contention_observer(
                cache_root, signal, async move {
                if enrichment {
                    let mut rows = vec![row("41800002", ArticleSource::PubMed)];
                    let original = serde_json::to_value(&rows).unwrap();
                    let deadline = crate::sources::VariantArticleDeadline::from_now(Duration::from_secs(60));
                    let status = crate::sources::with_variant_article_deadline(deadline,
                        enrich_article_search_rows_with_semantic_scholar(&mut rows)).await.unwrap();
                    assert_eq!(serde_json::to_value(&rows).unwrap(), original);
                    assert_eq!(status.source, ArticleSource::SemanticScholar);
                    assert_eq!(status.status, Some(if expired {
                        ArticleSourceAvailability::Degraded
                    } else { ArticleSourceAvailability::Ok }));
                    assert_eq!(status.message.as_deref().is_some_and(|s| s.contains("deadline")), expired);
                } else {
                    let result = search_page(&deadline_filters(), 5, 0, ArticleSourceFilter::SemanticScholar).await;
                    if expired {
                        assert!(matches!(result, Err(BioMcpError::SourceUnavailable { source_name, reason, .. })
                            if source_name == "article search" && reason.contains("deadline")));
                    } else {
                        let page = result.unwrap();
                        assert!(page.results.is_empty());
                        assert_eq!(page.source_status[0].status, Some(ArticleSourceAvailability::Ok));
                    }
                }
            }));
            // The observer signals only after try_lock_exclusive returns WouldBlock.
            // Both expiry and success controls must reach this same real contention.
            tokio::select! {
                () = contended.notified() => {}
                result = &mut task => panic!("constructor settled before lock contention: {result:?}"),
            }
            assert!(!task.is_finished(), "construction is waiting on the held epoch lock");
            if expired {
                tokio::time::advance(Duration::from_secs(61)).await;
                task.await.unwrap();
                assert!(requests.lock().unwrap().is_empty(), "constructor expiry sends no request");
                FileExt::unlock(&held).unwrap();
            } else {
                FileExt::unlock(&held).unwrap();
                // Drive the existing retry timer after releasing the witnessed lock.
                tokio::time::advance(Duration::from_millis(10)).await;
                task.await.unwrap();
                let requests = requests.lock().unwrap();
                assert_eq!(requests.len(), 1);
                assert!(requests[0].starts_with(if enrichment {
                    "/graph/v1/paper/batch?"
                } else { "/graph/v1/paper/search?" }));
            }
            running.store(false, std::sync::atomic::Ordering::SeqCst);
            driver.await.unwrap();
        }
    }
}

#[serial_test::serial(source_env)]
#[tokio::test(start_paused = true)]
async fn pagination_deadline_retains_completed_rows_and_rejects_zero_answers() {
    for (filter, source, pmid, title, total) in [
        (ArticleSourceFilter::PubTator, ArticleSource::PubTator, "41800001", "deadline fixture PubTator row", 100),
        (ArticleSourceFilter::EuropePmc, ArticleSource::EuropePmc, "41800004", "deadline fixture Europe PMC row", 100),
        (ArticleSourceFilter::PubMed, ArticleSource::PubMed, "41800002", "deadline fixture PubMed row", 100),
    ] {
        // Successful empty, held first page, completed first/held second page.
        for mode in [0, 1, 2] {
            let first_page = mode == 2;
            let (running, driver) = clock_driver();
            let (_release, receiver) = mpsc::channel();
            let hold = Arc::new(Mutex::new(receiver));
            let held = Arc::new(tokio::sync::Notify::new());
            let signal = Arc::clone(&held);
            let requests = Arc::new(Mutex::new(Vec::new()));
            let observed = Arc::clone(&requests);
            let fixture = TestHttpFixture::spawn(move |request| {
                let target = request.split_whitespace().nth(1).unwrap();
                observed.lock().unwrap().push(target.to_string());
                if mode == 0 {
                    return TestHttpReply::Bytes(test_http_response("200 OK", "application/json",
                        match source {
                            ArticleSource::PubTator => br#"{"results":[],"count":0,"total_pages":0,"current":1,"page_size":25,"facets":{}}"#,
                            ArticleSource::EuropePmc => br#"{"hitCount":0,"resultList":{"result":[]}}"#,
                            ArticleSource::PubMed => br#"{"esearchresult":{"count":"0","idlist":[]}}"#,
                            _ => unreachable!(),
                        }));
                }
                let summary = target.contains("esummary.fcgi");
                let first = target.contains("page=1") || target.contains("retstart=0");
                if !summary && (!first_page || !first) {
                    signal.notify_one();
                    return TestHttpReply::Hold(Arc::clone(&hold));
                }
                let reply = fixture_reply(request, None);
                match reply {
                    TestHttpReply::Bytes(bytes) if !summary => {
                        // Replace the small fixture's total, retaining its literal row.
                        let response = String::from_utf8(bytes).unwrap();
                        let (_, body) = response.split_once("\r\n\r\n").unwrap();
                        let body = body.replace("\"count\":1", "\"count\":100")
                            .replace("\"hitCount\":1", "\"hitCount\":100")
                            .replace("\"count\":\"1\"", "\"count\":\"100\"");
                        TestHttpReply::Bytes(test_http_response("200 OK", "application/json", body.as_bytes()))
                    }
                    reply => reply,
                }
            }).await;
            let cache = TempDirGuard::new("article-pagination-deadline");
            let _env = deadline_env(&fixture, cache.path(), "60000");
            let task = tokio::spawn(async move { search_page(&deadline_filters(), 5, 0, filter).await });
            if mode != 0 {
                held.notified().await;
                tokio::time::advance(Duration::from_secs(61)).await;
            }
            let result = task.await.unwrap();
            if mode == 0 {
                let page = result.unwrap();
                assert!(page.results.is_empty());
                assert_eq!(page.total, Some(0));
                assert!(page.source_status.is_empty(), "empty success has no deadline status");
            } else if first_page {
                let page = result.unwrap();
                assert_eq!(page.results.iter().map(|row| (row.pmid.as_str(), row.title.as_str(), row.source))
                    .collect::<Vec<_>>(), vec![(pmid, title, source)]);
                assert_eq!(page.total, Some(total));
                assert_eq!(page.source_status.len(), 1);
                assert_eq!(page.source_status[0].source, source);
                assert_eq!(page.source_status[0].status, Some(ArticleSourceAvailability::Degraded));
                assert_eq!(page.source_status[0].message.as_deref(),
                    Some("article search deadline elapsed during pagination"));
            } else {
                assert!(matches!(result, Err(BioMcpError::SourceUnavailable { source_name, reason, .. })
                    if source_name == "article search" && reason.contains("deadline")));
            }
            let requests = requests.lock().unwrap();
            assert_eq!(requests.len(), if first_page { if source == ArticleSource::PubMed { 3 } else { 2 } } else { 1 });
            assert!(requests.last().unwrap().contains(if source == ArticleSource::PubMed {
                if first_page { "retstart=1" } else { "retstart=0" }
            } else if first_page { "page=2" } else { "page=1" }));
            running.store(false, std::sync::atomic::Ordering::SeqCst);
            driver.await.unwrap();
        }
    }
}
