// sleep reason: existing fixture delays remain until signal-based fixture work
#![allow(clippy::disallowed_methods)]
use std::collections::BTreeMap;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

use super::super::{
    SharedHttpClientKind, VariantArticleDeadline, build_uncached_http_client,
    finish_cached_http_client, provider_url_policy, with_variant_article_deadline,
};
use crate::test_support::TempDirGuard;
use axum::{
    Router,
    body::{Body, Bytes},
    extract::State,
    http::{HeaderMap, Response},
    response::IntoResponse,
    routing::{get, post},
};
use http_cache_reqwest::CacheMode;
use reqwest::StatusCode;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

struct EnvRestore(BTreeMap<&'static str, Option<std::ffi::OsString>>);

impl EnvRestore {
    fn set(values: &[(&'static str, Option<&str>)]) -> Self {
        let mut prior = BTreeMap::new();
        for (name, value) in values {
            prior.insert(*name, std::env::var_os(name));
            // SAFETY: these tests share a serial-test group, and the guard restores
            // every value before releasing that group.
            unsafe {
                match value {
                    Some(value) => std::env::set_var(name, value),
                    None => std::env::remove_var(name),
                }
            }
        }
        Self(prior)
    }
}

impl Drop for EnvRestore {
    fn drop(&mut self) {
        for (name, value) in &self.0 {
            // SAFETY: source_env serializes these environment mutations.
            unsafe {
                match value {
                    Some(value) => std::env::set_var(name, value),
                    None => std::env::remove_var(name),
                }
            }
        }
    }
}

#[derive(Clone)]
struct OrphanOriginState {
    body: Arc<Vec<u8>>,
    status: StatusCode,
    requests: Arc<AtomicUsize>,
    active: Arc<AtomicUsize>,
    peak: Arc<AtomicUsize>,
    forms: Arc<Mutex<Vec<String>>>,
    delay: std::time::Duration,
}

async fn orphan_origin(State(state): State<OrphanOriginState>, body: Bytes) -> Response<Body> {
    let active = state.active.fetch_add(1, Ordering::SeqCst) + 1;
    state.peak.fetch_max(active, Ordering::SeqCst);
    state.requests.fetch_add(1, Ordering::SeqCst);
    state
        .forms
        .lock()
        .unwrap()
        .push(String::from_utf8(body.to_vec()).unwrap());
    tokio::time::sleep(state.delay).await;
    state.active.fetch_sub(1, Ordering::SeqCst);
    Response::builder()
        .status(state.status)
        .body(Body::from(state.body.as_ref().clone()))
        .unwrap()
}

async fn orphan_server(
    status: StatusCode,
    delay: std::time::Duration,
) -> (String, OrphanOriginState, tokio::task::JoinHandle<()>) {
    let state = OrphanOriginState {
        body: Arc::new(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/testdata/sources/fda_orphan/provider-shaped.html"
            ))
            .to_vec(),
        ),
        status,
        requests: Default::default(),
        active: Default::default(),
        peak: Default::default(),
        forms: Default::default(),
        delay,
    };
    let app = Router::new()
        .route("/OOPD_Results.cfm", post(orphan_origin))
        .with_state(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (format!("http://{address}"), state, task)
}

fn orphan_cache_key(base: &str, candidate: &str) -> String {
    super::super::fda_orphan::cache_key(base, &super::super::fda_orphan::ordered_form(candidate))
}

#[test]
fn fda_orphan_cache_modes_and_expiry_are_exact() {
    use super::super::fda_orphan::{SourceCacheMode, cache_entry_is_usable, source_cache_mode};
    use SourceCacheMode::{Infinite, Normal, Off};
    use http_cache_reqwest::CacheMode::{ForceCache, NoStore};
    assert_eq!(source_cache_mode(true, Some(ForceCache)), Off);
    assert_eq!(source_cache_mode(false, Some(NoStore)), Off);
    assert_eq!(source_cache_mode(false, Some(ForceCache)), Infinite);
    assert_eq!(source_cache_mode(false, None), Normal);
    assert!(cache_entry_is_usable(Normal, 100, 99));
    assert!(!cache_entry_is_usable(Normal, 86_401, 0));
    assert!(cache_entry_is_usable(Infinite, u64::MAX, 0));
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn fda_orphan_caps_forms_concurrency_and_off_bypasses_cache() {
    let (base, state, server) =
        orphan_server(StatusCode::OK, std::time::Duration::from_millis(20)).await;
    let root = TempDirGuard::new("fda-orphan-off");
    let _env = EnvRestore::set(&[
        ("BIOMCP_FDA_ORPHAN_BASE", Some(&base)),
        ("BIOMCP_CACHE_DIR", Some(root.path().to_str().unwrap())),
    ]);
    let result = super::super::fda_orphan::fetch_with_mode(
        (0..8).map(|index| format!("candidate {index}")).collect(),
        super::super::fda_orphan::SourceCacheMode::Off,
    )
    .await
    .expect("fda orphan fetch");
    server.abort();
    assert_eq!(
        result.outcome,
        super::super::fda_orphan::FdaOrphanOutcome::Empty
    );
    assert_eq!(state.requests.load(Ordering::SeqCst), 6);
    assert!(state.peak.load(Ordering::SeqCst) <= 2);
    let mut forms = state.forms.lock().unwrap().clone();
    forms.sort();
    let mut expected = (0..6)
        .map(|index| format!("Product_name=candidate+{index}&sponsor_name=&Designation=&Designation_Start_Date=&Designation_End_Date=&Search_param=DESDATE&Output_Format=Excel&Sort_order=GENERIC_NAME&RecordsPerPage=25&newSearch=Run+Search"))
        .collect::<Vec<_>>();
    expected.sort();
    assert_eq!(forms, expected);
    assert!(!root.path().join("http").exists());
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn fda_orphan_normal_cache_is_fresh_then_refreshes_when_expired() {
    let (base, state, server) = orphan_server(StatusCode::OK, std::time::Duration::ZERO).await;
    let root = TempDirGuard::new("fda-orphan-normal");
    let _env = EnvRestore::set(&[
        ("BIOMCP_FDA_ORPHAN_BASE", Some(&base)),
        ("BIOMCP_CACHE_DIR", Some(root.path().to_str().unwrap())),
        ("BIOMCP_CACHE_MIN_DISK_FREE", Some("1B")),
    ]);
    let candidate = "eflornithine hydrochloride";
    let first = super::super::fda_orphan::fetch_with_mode(
        vec![candidate.into()],
        super::super::fda_orphan::SourceCacheMode::Normal,
    )
    .await
    .expect("fda orphan fetch");
    let second = super::super::fda_orphan::fetch_with_mode(
        vec![candidate.into()],
        super::super::fda_orphan::SourceCacheMode::Normal,
    )
    .await
    .expect("fda orphan fetch");
    assert_eq!(first, second);
    assert_eq!(state.requests.load(Ordering::SeqCst), 1);
    let manager = crate::cache::SizeAwareCacheManager::new(
        root.path().join("http"),
        super::test_cache_config(root.path()),
    )
    .unwrap();
    let key = orphan_cache_key(&base, candidate);
    super::super::fda_orphan::rewrite_cached_time_for_test(&manager, &key, 0).await;
    server.abort();
    let expired = super::super::fda_orphan::fetch_with_mode(
        vec![candidate.into()],
        super::super::fda_orphan::SourceCacheMode::Normal,
    )
    .await
    .expect("fda orphan fetch");
    assert_eq!(
        expired.outcome,
        super::super::fda_orphan::FdaOrphanOutcome::Unavailable
    );
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn fda_orphan_form_cache_is_independent_of_prior_candidate_set() {
    let fixture = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/testdata/sources/fda_orphan/provider-shaped.html"
    ));
    let second = fixture
        .lines()
        .find(|line| line.starts_with("<tr><td>Eflornithine"))
        .unwrap()
        .replace("Eflornithine hydrochloride", "Other drug")
        .replace("992323", "992324");
    let broad = Arc::new(fixture.replace("</table>", &format!("{second}\n</table>")));
    let app = Router::new().route(
        "/OOPD_Results.cfm",
        post(move |body: Bytes| {
            let broad = Arc::clone(&broad);
            async move {
                if String::from_utf8_lossy(&body).starts_with("Product_name=eflornithine+") {
                    (StatusCode::OK, broad.as_str().to_owned())
                } else {
                    (StatusCode::INTERNAL_SERVER_ERROR, String::new())
                }
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let base = format!("http://{address}");
    let root = TempDirGuard::new("fda-orphan-candidate-independent");
    let _env = EnvRestore::set(&[
        ("BIOMCP_FDA_ORPHAN_BASE", Some(&base)),
        ("BIOMCP_CACHE_DIR", Some(root.path().to_str().unwrap())),
    ]);
    let first = super::super::fda_orphan::fetch_with_mode(
        vec!["eflornithine hydrochloride".into()],
        super::super::fda_orphan::SourceCacheMode::Normal,
    )
    .await
    .expect("fda orphan fetch");
    assert_eq!(first.records.len(), 1);
    let later = super::super::fda_orphan::fetch_with_mode(
        vec!["eflornithine hydrochloride".into(), "other drug".into()],
        super::super::fda_orphan::SourceCacheMode::Normal,
    )
    .await
    .expect("fda orphan fetch");
    server.abort();
    assert_eq!(
        later.outcome,
        super::super::fda_orphan::FdaOrphanOutcome::Degraded
    );
    assert_eq!(later.total_matching, Some(2));
    assert_eq!(
        later
            .records
            .iter()
            .map(|row| row.generic_name.as_str())
            .collect::<Vec<_>>(),
        vec!["Eflornithine hydrochloride", "Other drug"]
    );
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn fda_orphan_infinite_miss_stores_and_failed_http_does_not_cache() {
    let root = TempDirGuard::new("fda-orphan-infinite");
    let (base, _state, server) = orphan_server(StatusCode::OK, std::time::Duration::ZERO).await;
    let _env = EnvRestore::set(&[
        ("BIOMCP_FDA_ORPHAN_BASE", Some(&base)),
        ("BIOMCP_CACHE_DIR", Some(root.path().to_str().unwrap())),
    ]);
    let candidate = "eflornithine hydrochloride";
    let first = super::super::fda_orphan::fetch_with_mode(
        vec![candidate.into()],
        super::super::fda_orphan::SourceCacheMode::Infinite,
    )
    .await
    .expect("fda orphan fetch");
    server.abort();
    let second = super::super::fda_orphan::fetch_with_mode(
        vec![candidate.into()],
        super::super::fda_orphan::SourceCacheMode::Infinite,
    )
    .await
    .expect("fda orphan fetch");
    assert_eq!(first, second);
    let bad_root = TempDirGuard::new("fda-orphan-http-failure");
    let (bad_base, _, bad_server) =
        orphan_server(StatusCode::INTERNAL_SERVER_ERROR, std::time::Duration::ZERO).await;
    let _bad_env = EnvRestore::set(&[
        ("BIOMCP_FDA_ORPHAN_BASE", Some(&bad_base)),
        ("BIOMCP_CACHE_DIR", Some(bad_root.path().to_str().unwrap())),
    ]);
    let failed = super::super::fda_orphan::fetch_with_mode(
        vec![candidate.into()],
        super::super::fda_orphan::SourceCacheMode::Normal,
    )
    .await
    .expect("fda orphan fetch");
    bad_server.abort();
    assert_eq!(
        failed.outcome,
        super::super::fda_orphan::FdaOrphanOutcome::Unavailable
    );
    let connection = rusqlite::Connection::open(bad_root.path().join("biomcp.db")).unwrap();
    assert_eq!(
        connection
            .query_row("SELECT count(*) FROM entries", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        0
    );
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn parallel_sources_share_one_sqlite_cache_construction() {
    let root = TempDirGuard::new("parallel-shared-cache");
    let _env = EnvRestore::set(&[
        ("BIOMCP_CACHE_DIR", Some(root.path().to_str().unwrap())),
        ("BIOMCP_CACHE_MIN_DISK_FREE", Some("1B")),
        ("BIOMCP_CACHE_MODE", None),
    ]);
    super::super::reset_shared_http_client_for_tests();
    let counts = Arc::new([AtomicUsize::new(0), AtomicUsize::new(0)]);
    *super::super::CACHE_SETUP_OBSERVER.write().unwrap() =
        Some((root.path().to_path_buf(), Arc::clone(&counts)));
    let deadline = VariantArticleDeadline::from_now(std::time::Duration::from_secs(3));
    let results = futures::future::join_all(
        (0..8).map(|_| super::super::shared_client_with_deadline("europe-pmc", &deadline)),
    )
    .await;
    let policy = provider_url_policy::ProviderUrlPolicy::semantic_scholar_api(
        &reqwest::Url::parse("https://api.semanticscholar.org").unwrap(),
    )
    .unwrap();
    for authenticated in [true, false] {
        super::super::semantic_scholar_provider_client("semantic-scholar", &policy, authenticated)
            .unwrap();
        super::super::semantic_scholar_provider_client_with_deadline(
            "semantic-scholar",
            &policy,
            authenticated,
            &deadline,
        )
        .await
        .unwrap();
    }
    super::super::provider_url_client("semantic-scholar", &policy).unwrap();
    super::super::provider_url_client_with_deadline("semantic-scholar", &policy, &deadline)
        .await
        .unwrap();
    super::super::operator_provider_url_client(&policy).unwrap();
    *super::super::CACHE_SETUP_OBSERVER.write().unwrap() = None;
    super::super::reset_shared_http_client_for_tests();
    for result in results {
        result.expect("parallel source client");
    }
    assert_eq!(
        counts[0].load(Ordering::SeqCst),
        1,
        "one cache construction for parallel sources"
    );
    assert_eq!(
        counts[1].load(Ordering::SeqCst),
        0,
        "SQLite initialization takes no legacy epoch lock"
    );
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn a_later_invocation_retries_cache_setup_after_a_failed_expired_invocation() {
    let root = TempDirGuard::new("retry-shared-cache-setup");
    let cache_root = root.path().join("cache");
    let _env = EnvRestore::set(&[
        ("BIOMCP_CACHE_DIR", Some(cache_root.to_str().unwrap())),
        ("BIOMCP_CACHE_MIN_DISK_FREE", Some("1B")),
        ("BIOMCP_CACHE_MODE", None),
    ]);
    super::super::reset_shared_http_client_for_tests();
    std::fs::write(&cache_root, b"temporary obstruction").unwrap();
    let first = VariantArticleDeadline::from_now(std::time::Duration::from_millis(100));
    super::super::shared_client_with_deadline("europe-pmc", &first)
        .await
        .expect("cache setup failure leaves provider available");
    tokio::time::sleep(first.remaining()).await;
    assert!(first.is_exhausted());
    std::fs::remove_file(&cache_root).unwrap();
    let later = VariantArticleDeadline::from_now(std::time::Duration::from_secs(3));
    super::super::shared_client_with_deadline("europe-pmc", &later)
        .await
        .expect("later invocation gets a cached client");
    assert!(
        cache_root.join("biomcp.db").is_file(),
        "later invocation retries setup without retaining the first deadline"
    );
    super::super::reset_shared_http_client_for_tests();
}

#[derive(Clone, Copy)]
enum CacheOriginMode {
    Initial,
    Fresh,
    Revalidate304,
    Revalidate200,
}

#[derive(Clone)]
struct CacheOriginState {
    mode: CacheOriginMode,
    requests: Arc<AtomicUsize>,
    saw_validator: Arc<AtomicBool>,
}

async fn cache_origin(State(state): State<CacheOriginState>, headers: HeaderMap) -> Response<Body> {
    let attempt = state.requests.fetch_add(1, Ordering::SeqCst);
    if headers.get("if-none-match").is_some() {
        state.saw_validator.store(true, Ordering::SeqCst);
    }
    let (status, cache_control, body) = match (state.mode, attempt) {
        (CacheOriginMode::Revalidate304, 1..) => (StatusCode::NOT_MODIFIED, "max-age=60", ""),
        (CacheOriginMode::Revalidate200, 1..) => {
            (StatusCode::OK, "max-age=60", "replacement-sensitive-body")
        }
        (CacheOriginMode::Fresh, _) => (StatusCode::OK, "max-age=3600", "cached-body"),
        _ => (StatusCode::OK, "max-age=0, must-revalidate", "cached-body"),
    };
    (
        status,
        [("cache-control", cache_control), ("etag", "\"validator\"")],
        body,
    )
        .into_response()
}

async fn cached_test_client(
    mode: CacheOriginMode,
    name: &str,
) -> (
    reqwest_middleware::ClientWithMiddleware,
    String,
    Arc<AtomicUsize>,
    Arc<AtomicBool>,
    TempDirGuard,
    tokio::task::JoinHandle<()>,
) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let requests = Arc::new(AtomicUsize::new(0));
    let validator = Arc::new(AtomicBool::new(false));
    let state = CacheOriginState {
        mode,
        requests: requests.clone(),
        saw_validator: validator.clone(),
    };
    let app = Router::new()
        .route("/resource", get(cache_origin))
        .with_state(state);
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let root = TempDirGuard::new(name);
    let manager = crate::cache::SizeAwareCacheManager::new_with_deadline(
        root.path().join("biomcp.db"),
        super::test_cache_config(root.path()),
        &VariantArticleDeadline::from_now(std::time::Duration::from_secs(3)),
    )
    .await
    .unwrap();
    let client = finish_cached_http_client(SharedHttpClientKind::Default, None, manager).unwrap();
    (
        client,
        format!("{base}/resource"),
        requests,
        validator,
        root,
        server,
    )
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn sqlite_cached_client_preserves_fresh_hits_and_conditional_revalidation() {
    for mode in [
        CacheOriginMode::Fresh,
        CacheOriginMode::Revalidate304,
        CacheOriginMode::Revalidate200,
    ] {
        let (client, url, requests, validator, _root, server) =
            cached_test_client(mode, "sqlite-revalidation").await;
        let _env = EnvRestore::set(&[(
            "BIOMCP_TEST_UNPACED_ORIGIN",
            Some(url.trim_end_matches("/resource")),
        )]);
        assert_eq!(
            client.get(&url).send().await.unwrap().text().await.unwrap(),
            "cached-body"
        );
        let second = client.get(&url).send().await.unwrap();
        assert!(
            !second
                .headers()
                .contains_key(crate::cache::manager::STALE_SERVE_AGE_HEADER)
        );
        let expected = if matches!(mode, CacheOriginMode::Revalidate200) {
            "replacement-sensitive-body"
        } else {
            "cached-body"
        };
        assert_eq!(second.text().await.unwrap(), expected);
        assert_eq!(
            requests.load(Ordering::SeqCst),
            if matches!(mode, CacheOriginMode::Fresh) {
                1
            } else {
                2
            }
        );
        assert_eq!(
            validator.load(Ordering::SeqCst),
            !matches!(mode, CacheOriginMode::Fresh)
        );
        server.abort();
    }
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn sqlite_request_no_store_bypasses_reads_and_publication() {
    let (client, url, requests, _, root, server) =
        cached_test_client(CacheOriginMode::Initial, "sqlite-request-bypass").await;
    let _env = EnvRestore::set(&[(
        "BIOMCP_TEST_UNPACED_ORIGIN",
        Some(url.trim_end_matches("/resource")),
    )]);
    for _ in 0..2 {
        assert_eq!(
            client
                .get(&url)
                .with_extension(CacheMode::NoStore)
                .send()
                .await
                .unwrap()
                .text()
                .await
                .unwrap(),
            "cached-body"
        );
    }
    assert_eq!(requests.load(Ordering::SeqCst), 2);
    assert!(
        crate::cache::sqlite::Store::open(root.path())
            .unwrap()
            .variants("http", "irrelevant")
            .unwrap()
            .is_empty()
    );
    let connection = rusqlite::Connection::open(root.path().join("biomcp.db")).unwrap();
    assert_eq!(
        connection
            .query_row("SELECT count(*) FROM entries", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        0
    );
    server.abort();
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn sqlite_busy_deadline_finishes_without_late_publication() {
    let (client, url, _, _, root, server) =
        cached_test_client(CacheOriginMode::Fresh, "sqlite-busy-deadline").await;
    let _env = EnvRestore::set(&[(
        "BIOMCP_TEST_UNPACED_ORIGIN",
        Some(url.trim_end_matches("/resource")),
    )]);
    let mut connection = rusqlite::Connection::open(root.path().join("biomcp.db")).unwrap();
    let held = connection
        .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
        .unwrap();
    let deadline = VariantArticleDeadline::from_now(std::time::Duration::from_millis(40));
    let start = std::time::Instant::now();
    let response = with_variant_article_deadline(deadline, client.get(&url).send()).await;
    assert!(response.is_err());
    assert!(start.elapsed() < std::time::Duration::from_millis(500));
    drop(held);
    assert_eq!(
        connection
            .query_row("SELECT count(*) FROM entries", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        0
    );
    server.abort();
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn ordinary_client_blocks_untrusted_local_destinations_before_contact() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let _env = EnvRestore::set(&[("BIOMCP_TEST_UNPACED_ORIGIN", None)]);
    let client = build_uncached_http_client(SharedHttpClientKind::Default, None).unwrap();

    client
        .get(format!("http://{address}/private"))
        .send()
        .await
        .expect_err("untrusted loopback must be rejected");

    assert!(!accept_with_timeout(&listener).await);
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn ordinary_client_allows_exact_override_but_ignores_proxy_and_cross_origin_redirects() {
    let provider = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let provider_address = provider.local_addr().unwrap();
    let proxy = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_url = format!("http://{}", proxy.local_addr().unwrap());
    let escaped = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let escaped_address = escaped.local_addr().unwrap();
    let base = format!("http://{provider_address}");
    let _env = EnvRestore::set(&[
        ("BIOMCP_TEST_UNPACED_ORIGIN", Some(base.as_str())),
        ("HTTP_PROXY", Some(proxy_url.as_str())),
        ("HTTPS_PROXY", Some(proxy_url.as_str())),
        ("ALL_PROXY", Some(proxy_url.as_str())),
        ("http_proxy", Some(proxy_url.as_str())),
        ("https_proxy", Some(proxy_url.as_str())),
        ("all_proxy", Some(proxy_url.as_str())),
        ("NO_PROXY", Some("")),
        ("no_proxy", Some("")),
    ]);

    let server = tokio::spawn(async move {
        let (mut first, _) = provider.accept().await.unwrap();
        let mut request = [0_u8; 2048];
        let _ = first.read(&mut request).await.unwrap();
        first
            .write_all(
                format!(
                    "HTTP/1.1 302 Found\r\nLocation: http://{provider_address}/final\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                )
                .as_bytes(),
            )
            .await
            .unwrap();

        let (mut second, _) = provider.accept().await.unwrap();
        let count = second.read(&mut request).await.unwrap();
        let final_request = String::from_utf8_lossy(&request[..count]).to_ascii_lowercase();
        second
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok")
            .await
            .unwrap();
        final_request.contains("authorization: bearer approved-origin")
    });

    let client = build_uncached_http_client(SharedHttpClientKind::Default, None).unwrap();
    let response = client
        .get(format!("{base}/start"))
        .header("Authorization", "Bearer approved-origin")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        server.await.unwrap(),
        "credential was not retained on the approved origin"
    );
    assert!(
        !accept_with_timeout(&proxy).await,
        "ambient proxy received provider traffic"
    );

    let redirector = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let redirector_address = redirector.local_addr().unwrap();
    let redirector_base = format!("http://{redirector_address}");
    let _redirect_env =
        EnvRestore::set(&[("BIOMCP_TEST_UNPACED_ORIGIN", Some(redirector_base.as_str()))]);
    let redirect_server = tokio::spawn(async move {
        let (mut socket, _) = redirector.accept().await.unwrap();
        let mut request = [0_u8; 1024];
        let _ = socket.read(&mut request).await.unwrap();
        socket
            .write_all(
                format!(
                    "HTTP/1.1 302 Found\r\nLocation: http://{escaped_address}/stolen\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                )
                .as_bytes(),
            )
            .await
            .unwrap();
    });
    let error = client
        .get(format!("{redirector_base}/start"))
        .header("Authorization", "Bearer must-not-escape")
        .send()
        .await
        .expect_err("cross-origin redirect must fail");
    assert!(error.to_string().contains("redirect"));
    redirect_server.await.unwrap();
    assert!(
        !accept_with_timeout(&escaped).await,
        "redirect target received credentials"
    );
}

async fn accept_with_timeout(listener: &tokio::net::TcpListener) -> bool {
    tokio::time::timeout(std::time::Duration::from_millis(50), listener.accept())
        .await
        .is_ok()
}
