use super::*;

use std::sync::atomic::{AtomicUsize, Ordering};
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
    let query = target.split_once('?').map(|(_, rest)| rest).unwrap_or("");
    // Route on the bare path: eutils targets always carry the query string,
    // so matching `ends_with("/esearch.fcgi")` on the raw target never
    // matched and the PubMed leg read a 404 (ticket 2023).
    let path = target.split('?').next().unwrap_or(target);
    // Distinct fixture keywords route one server across the deadline
    // scenarios: "deadline silence" holds or fails every source, and
    // "pagebound" answers Europe PMC page one and holds every later page.
    let wants_nothing = query.contains("silence");
    let wants_single_source_hold = query.contains("pagebound");
    if target.starts_with("/search") && target.contains("query=") {
        if wants_nothing {
            return match europepmc_hold {
                Some(hold) => TestHttpReply::Hold(Arc::clone(hold)),
                None => TestHttpReply::Bytes(test_http_response(
                    "503 Service Unavailable",
                    "application/json",
                    b"{}",
                )),
            };
        }
        // Single-source deadline case: the first cursor page answers fast
        // with distinct rows; every later cursor page is held past the
        // deadline (ticket 1298 moved the wire from page= to cursorMark=).
        let first_cursor = !query.contains("cursorMark=")
            || query
                .split('&')
                .any(|pair| pair == "cursorMark=%2A" || pair == "cursorMark=*");
        if wants_single_source_hold && !first_cursor {
            return match europepmc_hold {
                Some(hold) => TestHttpReply::Hold(Arc::clone(hold)),
                None => TestHttpReply::Bytes(test_http_response(
                    "200 OK",
                    "application/json",
                    br#"{"hitCount":30,"resultList":{"result":[]}}"#,
                )),
            };
        }
        if wants_single_source_hold {
            return TestHttpReply::Bytes(test_http_response(
                "200 OK",
                "application/json",
                br#"{"hitCount":30,"nextCursorMark":"CUR2","resultList":{"result":[{"id":"41800021","pmid":"41800021","title":"single-source deadline page one row","journalTitle":"Fixture Journal","firstPublicationDate":"2026-01-02","authorString":"Fixture Author"},{"id":"41800022","pmid":"41800022","title":"single-source deadline page one row two","journalTitle":"Fixture Journal","firstPublicationDate":"2026-01-02","authorString":"Fixture Author"}]}}"#, 
            ));
        }
        return match europepmc_hold {
            Some(hold) => TestHttpReply::Hold(Arc::clone(hold)),
            None => TestHttpReply::Bytes(test_http_response(
                "200 OK",
                "application/json",
                br#"{"hitCount":1,"resultList":{"result":[{"id":"41800004","pmid":"41800004","title":"deadline fixture Europe PMC row","journalTitle":"Fixture Journal","firstPublicationDate":"2026-01-02","authorString":"Fixture Author"}]}}"#,
            )),
        };
    }
    if wants_nothing {
        return TestHttpReply::Bytes(test_http_response(
            "503 Service Unavailable",
            "application/json",
            b"{}",
        ));
    }
    let body = if target.starts_with("/search/") && target.contains("text=") {
        br#"{"results":[{"_id":"pt-418","pmid":41800001,"title":"deadline fixture PubTator row","journal":"Fixture Journal","date":"2026-01-01","score":42.0}],"count":1,"total_pages":1,"current":1,"page_size":25,"facets":{}}"#.as_slice()
    } else if path.ends_with("/esearch.fcgi") {
        br#"{"esearchresult":{"count":"1","idlist":["41800002"]}}"#.as_slice()
    } else if path.ends_with("/esummary.fcgi") {
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
        page.results.iter().any(|row| row.pmid == "41800002"),
        "PubMed rows survive the failed primaries: {:?}",
        page.results
            .iter()
            .map(|row| row.pmid.as_str())
            .collect::<Vec<_>>()
    );
    assert!(
        page.results.iter().any(|row| row.pmid == "41800003"),
        "Semantic Scholar rows survive the failed primaries: {:?}",
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

/// Grace the wall-clock assertions allow on top of the forced budget: covers
/// scheduler jitter and cache teardown without masking a spin.
const DEADLINE_WALL_GRACE: std::time::Duration = std::time::Duration::from_millis(2500);

#[serial_test::serial(source_env)]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn single_backend_deadline_keeps_fetched_rows_and_names_the_source() {
    // Europe PMC page one answers with distinct rows; page two is held. The
    // deadline fires mid-flight with rows already accumulated.
    let (_release_tx, hold_rx) = mpsc::channel::<()>();
    let hold = Arc::new(Mutex::new(hold_rx));
    let fixture = TestHttpFixture::spawn(move |request| fixture_reply(request, Some(&hold))).await;
    let cache = TempDirGuard::new("article-search-deadline-single-source");
    let _env = deadline_env(&fixture, cache.path(), "1500");
    let mut filters = deadline_filters();
    filters.keyword = Some("pagebound single source".into());

    let budget = std::time::Duration::from_millis(1500);
    let started = std::time::Instant::now();
    let page = tokio::time::timeout(
        crate::test_support::watchdog(60),
        search_page(&filters, 10, 0, ArticleSourceFilter::EuropePmc),
    )
    .await
    .expect("single-backend search exceeds its watchdog")
    .expect("partial rows survive a single-backend deadline");

    // The invocation exits near the deadline; a post-deadline spin or a
    // locked construction blows this bound.
    assert!(
        started.elapsed() <= budget + DEADLINE_WALL_GRACE,
        "single-backend expiry honors the deadline: {:?} elapsed",
        started.elapsed()
    );
    assert!(
        page.results
            .iter()
            .any(|row| matches!(row.pmid.as_str(), "41800021" | "41800022")),
        "rows fetched before expiry are kept: {:?}",
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
        "the degradation names the deadline: {europepmc:?}"
    );
}

#[serial_test::serial(source_env)]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn deadline_expiry_without_rows_names_the_deadline_error() {
    // Every source fails or is held, so nothing answers and the terminal
    // error must name the deadline instead of a generic failure.
    let (_release_tx, hold_rx) = mpsc::channel::<()>();
    let hold = Arc::new(Mutex::new(hold_rx));
    let fixture = TestHttpFixture::spawn(move |request| fixture_reply(request, Some(&hold))).await;
    let cache = TempDirGuard::new("article-search-deadline-terminal");
    let _env = deadline_env(&fixture, cache.path(), "1200");
    let mut filters = deadline_filters();
    filters.keyword = Some("deadline silence".into());

    let budget = std::time::Duration::from_millis(1200);
    let started = std::time::Instant::now();
    let error = tokio::time::timeout(
        crate::test_support::watchdog(60),
        search_page(&filters, 5, 0, ArticleSourceFilter::All),
    )
    .await
    .expect("empty search exceeds its watchdog")
    .expect_err("a search where nothing answered is an error");

    assert!(
        started.elapsed() <= budget + DEADLINE_WALL_GRACE,
        "empty expiry honors the deadline: {:?} elapsed",
        started.elapsed()
    );
    let reason = match &error {
        BioMcpError::SourceUnavailable { reason, .. } => reason.clone(),
        other => panic!("expected a source-unavailable deadline error, got {other:?}"),
    };
    assert!(
        reason.starts_with(crate::entities::article::ARTICLE_SEARCH_DEADLINE_REASON_PREFIX),
        "the terminal error names the deadline: {reason}"
    );
    let projection = error.public_projection();
    assert!(
        projection
            .message
            .starts_with(crate::entities::article::ARTICLE_SEARCH_DEADLINE_REASON_PREFIX),
        "the public error names the deadline: {}",
        projection.message
    );
}

#[serial_test::serial(source_env)]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn construction_under_a_held_epoch_lock_honors_the_deadline() {
    // Hold the cache epoch lock so shared-client construction cannot take
    // it; the invocation must still exit at its deadline (ticket 1299).
    // Process-state independence (ticket 2023): the shared client is built
    // once per process, so the test first builds it and then clears the
    // slot. Without that reset the test passes only when it runs before
    // every other shared-client test — nextest gives each test a fresh
    // process, plain `cargo test` does not.
    let fixture = TestHttpFixture::spawn(move |request| fixture_reply(request, None)).await;
    let cache = TempDirGuard::new("article-search-deadline-epoch-lock");
    std::fs::create_dir_all(cache.path()).expect("cache root");
    let _env = deadline_env(&fixture, cache.path(), "1200");
    crate::sources::shared_client().expect("shared client builds before the lock is held");
    crate::sources::reset_shared_http_client_for_tests();
    let lock_path = cache.path().join(".body-limit-cache-v1.lock");
    let held = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&lock_path)
        .expect("epoch lock file");
    fs2::FileExt::lock_exclusive(&held).expect("hold epoch lock");

    let budget = std::time::Duration::from_millis(1200);
    let started = std::time::Instant::now();
    let result = tokio::time::timeout(
        crate::test_support::watchdog(60),
        search_page(&deadline_filters(), 3, 0, ArticleSourceFilter::PubMed),
    )
    .await
    .expect("locked construction exceeds its watchdog");

    assert!(
        started.elapsed() <= budget + DEADLINE_WALL_GRACE,
        "construction contention honors the deadline: {:?} elapsed",
        started.elapsed()
    );
    match result {
        Err(BioMcpError::SourceUnavailable { reason, .. }) => assert!(
            reason.starts_with(crate::entities::article::ARTICLE_SEARCH_DEADLINE_REASON_PREFIX),
            "the locked-construction error names the deadline: {reason}"
        ),
        Ok(page) => panic!(
            "held epoch lock must not yield a page: {} rows",
            page.results.len()
        ),
        other => panic!("expected a deadline error, got {other:?}"),
    }
    fs2::FileExt::unlock(&held).expect("release epoch lock");
}

#[test]
fn deadline_recognition_covers_the_io_cancellation_shape() {
    // The cache layer reports deadline cancellation as a TimedOut io error
    // with one exact message; recognition must cover it directly and through
    // the source-context envelope.
    // The literal is deliberate: recognition is pinned to the exact message
    // every deadline io producer emits; drift must fail this test.
    let io_deadline = BioMcpError::Io(std::io::Error::new(
        std::io::ErrorKind::TimedOut,
        "variant article invocation deadline exceeded",
    ));
    assert!(is_search_deadline_error(&io_deadline));
    assert!(is_search_deadline_error(&BioMcpError::WithSourceContext {
        context: crate::error::SourceContext::retry(crate::error::SourceProvider::PUBMED),
        source: Box::new(io_deadline),
    }));
    let other_io = BioMcpError::Io(std::io::Error::new(
        std::io::ErrorKind::TimedOut,
        "some other timeout",
    ));
    assert!(!is_search_deadline_error(&other_io));
}

#[serial_test::serial(source_env)]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn deep_offset_is_refused_on_every_sort_before_any_request() {
    // The relevance arms already refused --offset + --limit above the fetch
    // window; the date and citations arms walked provider pages instead and
    // answered `has_more: true` on an empty page (ticket 2023).
    let requests = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&requests);
    let fixture = TestHttpFixture::spawn(move |request| {
        counter.fetch_add(1, Ordering::SeqCst);
        fixture_reply(request, None)
    })
    .await;
    let cache = TempDirGuard::new("article-search-deep-offset");
    let _env = deadline_env(&fixture, cache.path(), "60000");

    for sort in [ArticleSort::Date, ArticleSort::Citations] {
        let mut filters = deadline_filters();
        filters.sort = sort;
        let error = search_page(&filters, 5, 1300, ArticleSourceFilter::EuropePmc)
            .await
            .expect_err("a deep offset is refused on every sort");
        match error {
            BioMcpError::InvalidArgument(message) => assert!(
                message.contains("--offset + --limit must be <= 1250"),
                "the refusal names the fetch window: {message}"
            ),
            other => panic!("expected an invalid-argument refusal, got {other:?}"),
        }
    }

    assert_eq!(
        requests.load(Ordering::SeqCst),
        0,
        "the window guard fires before any provider request"
    );
}

#[serial_test::serial(source_env)]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn fast_pubtator_failure_does_not_hide_the_deadline_error() {
    // PubTator fails fast and non-retryably (400) while Europe PMC is held
    // past the deadline: the terminal error must still name the deadline,
    // not the fast failure (ticket 2023).
    let (_release_tx, hold_rx) = mpsc::channel::<()>();
    let hold = Arc::new(Mutex::new(hold_rx));
    let fixture = TestHttpFixture::spawn(move |request| {
        let target = request
            .split_whitespace()
            .nth(1)
            .expect("fixture request line carries a target");
        if target.starts_with("/search") && target.contains("query=") {
            return TestHttpReply::Hold(Arc::clone(&hold));
        }
        if target.starts_with("/search") && target.contains("text=") {
            return TestHttpReply::Bytes(test_http_response(
                "400 Bad Request",
                "application/json",
                b"{}",
            ));
        }
        fixture_reply(request, Some(&hold))
    })
    .await;
    let cache = TempDirGuard::new("article-search-fast-failure-deadline");
    let _env = deadline_env(&fixture, cache.path(), "1200");
    let mut filters = deadline_filters();
    filters.keyword = Some("deadline silence".into());

    let budget = std::time::Duration::from_millis(1200);
    let started = std::time::Instant::now();
    let error = tokio::time::timeout(
        crate::test_support::watchdog(60),
        search_page(&filters, 5, 0, ArticleSourceFilter::All),
    )
    .await
    .expect("fast-failure search exceeds its watchdog")
    .expect_err("a search where nothing answered is an error");

    assert!(
        started.elapsed() <= budget + DEADLINE_WALL_GRACE,
        "fast-failure expiry honors the deadline: {:?} elapsed",
        started.elapsed()
    );
    match &error {
        BioMcpError::SourceUnavailable { reason, .. } => assert!(
            reason.starts_with(crate::entities::article::ARTICLE_SEARCH_DEADLINE_REASON_PREFIX),
            "the terminal error names the deadline, not the fast PubTator failure: {reason}"
        ),
        other => panic!("expected a source-unavailable deadline error, got {other:?}"),
    }
}

#[test]
fn both_primaries_failing_prefers_a_deadline_error_over_the_fast_failure() {
    // When both primaries fail, the federated terminal error prefers a
    // deadline error from either primary over PubTator's fast failure
    // (ticket 2023).
    let fast_failure = BioMcpError::Api {
        api: "pubtator".into(),
        message: "HTTP 400".into(),
    };
    let deadline_failure = BioMcpError::Io(std::io::Error::new(
        std::io::ErrorKind::TimedOut,
        "variant article invocation deadline exceeded",
    ));
    let unavailable = |source, error| FederatedSourceOutcome::Unavailable {
        error: Some(error),
        status: source_degraded_status(source, "provider unavailable".into()),
    };
    let semantic_status = ArticleSourceStatus {
        source: ArticleSource::SemanticScholar,
        enabled: true,
        auth_mode: None,
        status: Some(ArticleSourceAvailability::Ok),
        message: None,
    };

    let federated = collect_federated_article_rows(
        unavailable(ArticleSource::PubTator, fast_failure),
        unavailable(ArticleSource::EuropePmc, deadline_failure),
        None,
        FederatedSourceOutcome::Available(
            crate::entities::article::backends::SemanticScholarCandidateOutcome {
                rows: Vec::new(),
                status: semantic_status,
            },
        ),
        FederatedSourceOutcome::Available(Vec::new()),
    )
    .expect("both primaries failing still collects rows");

    let error = federated
        .primary_error
        .expect("both primaries failing carries a terminal error");
    assert!(
        is_search_deadline_error(&error),
        "the deadline error wins over the fast failure: {error:?}"
    );
}
