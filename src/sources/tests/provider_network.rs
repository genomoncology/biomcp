use std::collections::BTreeMap;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
};

use super::super::{
    RequestBuilderSourceContextExt, SharedHttpClientKind, VariantArticleDeadline,
    build_http_client_with_config_and_manager, build_http_client_with_config_deadline,
    build_uncached_http_client, finish_cached_http_client, provider_url_policy,
    with_variant_article_deadline,
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
    assert_eq!(source_cache_mode(true, Some("infinite")), Off);
    assert_eq!(source_cache_mode(false, Some("off")), Off);
    assert_eq!(source_cache_mode(false, Some("infinite")), Infinite);
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
    .await;
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
    .await;
    let second = super::super::fda_orphan::fetch_with_mode(
        vec![candidate.into()],
        super::super::fda_orphan::SourceCacheMode::Normal,
    )
    .await;
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
    .await;
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
    .await;
    assert_eq!(first.records.len(), 1);
    let later = super::super::fda_orphan::fetch_with_mode(
        vec!["eflornithine hydrochloride".into(), "other drug".into()],
        super::super::fda_orphan::SourceCacheMode::Normal,
    )
    .await;
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
    .await;
    server.abort();
    let second = super::super::fda_orphan::fetch_with_mode(
        vec![candidate.into()],
        super::super::fda_orphan::SourceCacheMode::Infinite,
    )
    .await;
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
    .await;
    bad_server.abort();
    assert_eq!(
        failed.outcome,
        super::super::fda_orphan::FdaOrphanOutcome::Unavailable
    );
    assert!(
        cacache::metadata(
            bad_root.path().join("http"),
            orphan_cache_key(&bad_base, candidate)
        )
        .await
        .unwrap()
        .is_none()
    );
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn fda_orphan_contended_key_lock_cancels_without_a_late_write() {
    let root = TempDirGuard::new("fda-orphan-lock-deadline");
    let (base, _, server) = orphan_server(StatusCode::OK, std::time::Duration::ZERO).await;
    let _env = EnvRestore::set(&[
        ("BIOMCP_FDA_ORPHAN_BASE", Some(&base)),
        ("BIOMCP_CACHE_DIR", Some(root.path().to_str().unwrap())),
    ]);
    let candidate = "eflornithine hydrochloride";
    let key = orphan_cache_key(&base, candidate);
    let held = crate::cache::lock_cache_key_async(
        root.path().to_path_buf(),
        key.clone(),
        Arc::new(|_| {}),
    )
    .await
    .unwrap();
    let started = std::time::Instant::now();
    let result = super::super::fda_orphan::fetch_with_mode_and_deadline(
        vec![candidate.into()],
        super::super::fda_orphan::SourceCacheMode::Normal,
        std::time::Duration::from_millis(40),
    )
    .await;
    assert_eq!(
        result.outcome,
        super::super::fda_orphan::FdaOrphanOutcome::Unavailable
    );
    assert!(started.elapsed() < std::time::Duration::from_millis(500));
    drop(held);
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    assert!(
        cacache::metadata(root.path().join("http"), &key)
            .await
            .unwrap()
            .is_none()
    );
    server.abort();
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn fda_orphan_missing_post_write_metadata_fails_closed() {
    let root = TempDirGuard::new("fda-orphan-finalization");
    let (base, _, server) = orphan_server(StatusCode::OK, std::time::Duration::ZERO).await;
    let _env = EnvRestore::set(&[("BIOMCP_FDA_ORPHAN_BASE", Some(&base))]);
    let manager = crate::cache::SizeAwareCacheManager::new_with_cache_observers(
        root.path().join("http"),
        super::test_cache_config(root.path()),
        |_, _| {},
        |path, key| {
            cacache::remove_sync(path, key).unwrap();
        },
    );
    let result = super::super::fda_orphan::fetch_with_manager_for_test(
        vec!["eflornithine hydrochloride".into()],
        super::super::fda_orphan::SourceCacheMode::Normal,
        std::time::Duration::from_secs(2),
        Arc::new(manager),
    )
    .await;
    server.abort();
    assert_eq!(
        result.outcome,
        super::super::fda_orphan::FdaOrphanOutcome::Unavailable
    );
}

impl Drop for EnvRestore {
    fn drop(&mut self) {
        for (name, value) in &self.0 {
            // SAFETY: see `EnvRestore::set`.
            unsafe {
                match value {
                    Some(value) => std::env::set_var(name, value),
                    None => std::env::remove_var(name),
                }
            }
        }
    }
}

async fn accept_with_timeout(listener: &tokio::net::TcpListener) -> bool {
    tokio::time::timeout(std::time::Duration::from_millis(250), listener.accept())
        .await
        .is_ok()
}

#[tokio::test(start_paused = true)]
async fn middleware_put_settles_after_publication_before_releasing_key_lock() {
    let root = TempDirGuard::new("middleware-put-publication");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let origin = reqwest::Url::parse(&format!("http://{address}")).unwrap();
    let policy = provider_url_policy::ProviderUrlPolicy::test_fixture(
        provider_url_policy::ProviderUrlConsumer::GithubRelease,
        &origin,
    )
    .unwrap();
    let armed = Arc::new(tokio::sync::Notify::new());
    let release = Arc::new(tokio::sync::Notify::new());
    let manager = crate::cache::SizeAwareCacheManager::new(
        root.path().join("http"),
        super::test_cache_config(root.path()),
    )
    .unwrap()
    .with_safe_return_observer({
        let (armed, release) = (Arc::clone(&armed), Arc::clone(&release));
        move || {
            let (armed, release) = (Arc::clone(&armed), Arc::clone(&release));
            async move {
                armed.notify_one();
                release.notified().await
            }
        }
    });
    let client =
        finish_cached_http_client(SharedHttpClientKind::Default, Some(&policy), manager).unwrap();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut request = [0; 1024];
        let _ = stream.read(&mut request).await.unwrap();
        stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nCache-Control: public, max-age=60\r\nConnection: close\r\n\r\nok").await.unwrap();
    });
    let deadline = VariantArticleDeadline::from_now(std::time::Duration::from_secs(10));
    let driving = Arc::new(AtomicBool::new(true));
    let driver = tokio::spawn({
        let driving = Arc::clone(&driving);
        async move {
            while driving.load(Ordering::SeqCst) {
                tokio::task::yield_now().await
            }
        }
    });
    let request = with_variant_article_deadline(deadline.clone(), async {
        client
            .get(origin)
            .send_with_source_context(crate::error::SourceContext::narrow(
                crate::error::SourceProvider::OLS4,
            ))
            .await
    });
    tokio::pin!(request);
    tokio::select! {
        () = armed.notified() => {}
        result = &mut request => panic!("settled before publication pause: {result:?}"),
    }
    driving.store(false, Ordering::SeqCst);
    driver.await.unwrap();
    tokio::time::advance(std::time::Duration::from_secs(11)).await;
    assert!(deadline.is_exhausted());
    assert!(
        crate::cache::try_lock_cache_maintenance(root.path())
            .unwrap()
            .is_none()
    );
    assert!(deadline.run(std::future::pending::<()>()).await.is_err());
    release.notify_one();
    let result = request.await;
    assert!(result.is_ok(), "post-arm request must settle: {result:?}");
    assert!(
        crate::cache::try_lock_cache_maintenance(root.path())
            .unwrap()
            .is_some()
    );
    server.await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn contended_maintenance_deadline_publishes_no_partial_client_or_cache_tree() {
    let root = TempDirGuard::new("client-maintenance-deadline");
    std::fs::create_dir_all(root.path().join("http-cacache")).unwrap();
    std::fs::write(root.path().join("http-cacache/sentinel"), b"legacy").unwrap();
    let held = crate::cache::try_lock_cache_maintenance(root.path())
        .unwrap()
        .unwrap();
    let deadline = VariantArticleDeadline::from_now(std::time::Duration::from_millis(20));
    let error = build_http_client_with_config_deadline(
        SharedHttpClientKind::Default,
        super::test_cache_config(root.path()),
        None,
        &deadline,
    )
    .await
    .expect_err("client construction must expire behind maintenance");
    assert_eq!(error.code(), "io");
    assert!(!root.path().join("http").exists());
    assert_eq!(
        std::fs::read(root.path().join("http-cacache/sentinel")).unwrap(),
        b"legacy"
    );
    drop(held);
    assert!(
        crate::cache::try_lock_cache_maintenance(root.path())
            .unwrap()
            .is_some()
    );
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
    safe_return_pause: Option<(Arc<tokio::sync::Notify>, Arc<tokio::sync::Notify>)>,
) -> (
    reqwest_middleware::ClientWithMiddleware,
    String,
    Arc<AtomicUsize>,
    Arc<AtomicUsize>,
    Arc<AtomicUsize>,
    Arc<AtomicBool>,
    Arc<AtomicBool>,
    Arc<AtomicU64>,
    crate::test_support::TempDirGuard,
    tokio::task::JoinHandle<()>,
) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let base = format!("http://{address}");
    let requests = Arc::new(AtomicUsize::new(0));
    let saw_validator = Arc::new(AtomicBool::new(false));
    let app = Router::new()
        .route("/resource", get(cache_origin))
        .with_state(CacheOriginState {
            mode,
            requests: Arc::clone(&requests),
            saw_validator: Arc::clone(&saw_validator),
        });
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let root = crate::test_support::TempDirGuard::new(name);
    let gets = Arc::new(AtomicUsize::new(0));
    let puts = Arc::new(AtomicUsize::new(0));
    let armed = Arc::new(AtomicBool::new(false));
    let after_put_delay_ms = Arc::new(AtomicU64::new(0));
    let get_count = Arc::clone(&gets);
    let put_count = Arc::clone(&puts);
    let fault = Arc::clone(&armed);
    let delay = Arc::clone(&after_put_delay_ms);
    let client = build_http_client_with_config_and_manager(
        SharedHttpClientKind::Default,
        super::test_cache_config(root.path()),
        None,
        move |path, config| {
            let mut manager = crate::cache::SizeAwareCacheManager::new_with_cache_observers(
                path,
                config,
                move |_, _| {
                    get_count.fetch_add(1, Ordering::SeqCst);
                },
                move |path, key| {
                    put_count.fetch_add(1, Ordering::SeqCst);
                    std::thread::sleep(std::time::Duration::from_millis(
                        delay.load(Ordering::SeqCst),
                    ));
                    if fault.swap(false, Ordering::SeqCst) {
                        assert!(cacache::metadata_sync(path, key).unwrap().is_some());
                        cacache::remove_sync(path, key).unwrap();
                    }
                },
            );
            if let Some((publication_armed, publication_release)) = safe_return_pause.clone() {
                manager = manager.with_safe_return_observer(move || {
                    let publication_armed = Arc::clone(&publication_armed);
                    let publication_release = Arc::clone(&publication_release);
                    async move {
                        publication_armed.notify_one();
                        publication_release.notified().await;
                    }
                });
            }
            Ok(manager)
        },
    )
    .unwrap();
    (
        client,
        format!("{base}/resource"),
        requests,
        gets,
        puts,
        armed,
        saw_validator,
        after_put_delay_ms,
        root,
        server,
    )
}

fn assert_sanitized_cache_error(error: &reqwest_middleware::Error, root: &std::path::Path) {
    let message = error.to_string();
    assert!(message.contains("cache security finalization failed after successful put"));
    assert!(!message.contains("resource"));
    assert!(!message.contains("cached-body"));
    assert!(!message.contains("sensitive"));
    assert!(!message.contains(&root.display().to_string()));
}

#[tokio::test]
#[serial_test::serial(article_resolver_env)]
async fn cached_client_post_write_failure_matrix_is_fail_closed() {
    for mode in [
        CacheOriginMode::Initial,
        CacheOriginMode::Revalidate304,
        CacheOriginMode::Revalidate200,
    ] {
        let (client, url, requests, gets, puts, armed, validator, _, root, server) =
            cached_test_client(mode, "post-write-client-failure", None).await;
        let _env = EnvRestore::set(&[(
            "BIOMCP_TEST_UNPACED_ORIGIN",
            Some(url.trim_end_matches("/resource")),
        )]);
        if !matches!(mode, CacheOriginMode::Initial) {
            client.get(&url).send().await.unwrap();
            assert_eq!(requests.load(Ordering::SeqCst), 1);
            gets.store(0, Ordering::SeqCst);
            puts.store(0, Ordering::SeqCst);
        }
        armed.store(true, Ordering::SeqCst);
        let error = client
            .get(&url)
            .send()
            .await
            .expect_err("post-write fault must fail request");
        assert_eq!(
            requests.load(Ordering::SeqCst),
            if matches!(mode, CacheOriginMode::Initial) {
                1
            } else {
                2
            }
        );
        assert_eq!(gets.load(Ordering::SeqCst), 1);
        assert_eq!(puts.load(Ordering::SeqCst), 1);
        assert_eq!(
            validator.load(Ordering::SeqCst),
            !matches!(mode, CacheOriginMode::Initial)
        );
        assert_sanitized_cache_error(&error, root.path());
        server.abort();
    }
}

#[tokio::test(start_paused = true)]
#[serial_test::serial(article_resolver_env)]
async fn provider_deadline_waits_for_post_publish_fail_closed_finalization() {
    let publication_armed = Arc::new(tokio::sync::Notify::new());
    let publication_release = Arc::new(tokio::sync::Notify::new());
    let (client, url, requests, gets, puts, armed, _, _, root, server) = cached_test_client(
        CacheOriginMode::Initial,
        "deadline-post-write-finalization",
        Some((
            Arc::clone(&publication_armed),
            Arc::clone(&publication_release),
        )),
    )
    .await;
    let _env = EnvRestore::set(&[(
        "BIOMCP_TEST_UNPACED_ORIGIN",
        Some(url.trim_end_matches("/resource")),
    )]);
    armed.store(true, Ordering::SeqCst);
    let deadline =
        crate::sources::VariantArticleDeadline::from_now(std::time::Duration::from_millis(10));
    let driving = Arc::new(AtomicBool::new(true));
    let driver = tokio::spawn({
        let driving = Arc::clone(&driving);
        async move {
            while driving.load(Ordering::SeqCst) {
                tokio::task::yield_now().await
            }
        }
    });
    let request_deadline = deadline.clone();
    let mut request = tokio::spawn(crate::sources::with_variant_article_deadline(
        deadline.clone(),
        async move {
            client
                .get(&url)
                .with_extension(request_deadline)
                .send()
                .await
        },
    ));
    tokio::select! {
        () = publication_armed.notified() => {}
        result = &mut request => panic!("settled before publication pause: {result:?}"),
    }
    assert!(!deadline.is_exhausted());
    driving.store(false, Ordering::SeqCst);
    driver.await.unwrap();
    tokio::time::advance(std::time::Duration::from_millis(11)).await;
    assert!(deadline.is_exhausted());
    publication_release.notify_one();
    let result = request.await.unwrap();

    let error = result.expect_err("post-write finalization must remain fail closed");
    assert_eq!(requests.load(Ordering::SeqCst), 1);
    assert_eq!(gets.load(Ordering::SeqCst), 1);
    assert_eq!(puts.load(Ordering::SeqCst), 1);
    assert_sanitized_cache_error(&error, root.path());
    server.abort();
}

#[tokio::test]
#[serial_test::serial(article_resolver_env)]
async fn cached_client_fresh_hit_and_request_no_store_bypass_writes() {
    let (client, url, requests, gets, puts, armed, _, _, _root, server) =
        cached_test_client(CacheOriginMode::Fresh, "fresh-client-hit", None).await;
    let _env = EnvRestore::set(&[(
        "BIOMCP_TEST_UNPACED_ORIGIN",
        Some(url.trim_end_matches("/resource")),
    )]);
    assert_eq!(
        client.get(&url).send().await.unwrap().text().await.unwrap(),
        "cached-body"
    );
    gets.store(0, Ordering::SeqCst);
    puts.store(0, Ordering::SeqCst);
    armed.store(true, Ordering::SeqCst);
    assert_eq!(
        client.get(&url).send().await.unwrap().text().await.unwrap(),
        "cached-body"
    );
    assert_eq!(requests.load(Ordering::SeqCst), 1);
    assert_eq!(gets.load(Ordering::SeqCst), 1);
    assert_eq!(puts.load(Ordering::SeqCst), 0);
    server.abort();

    let (client, url, requests, gets, puts, armed, _, _, _root, server) =
        cached_test_client(CacheOriginMode::Initial, "no-store-client-request", None).await;
    let _env = EnvRestore::set(&[(
        "BIOMCP_TEST_UNPACED_ORIGIN",
        Some(url.trim_end_matches("/resource")),
    )]);
    armed.store(true, Ordering::SeqCst);
    let response = client
        .get(&url)
        .with_extension(CacheMode::NoStore)
        .send()
        .await
        .unwrap();
    assert_eq!(response.text().await.unwrap(), "cached-body");
    assert_eq!(requests.load(Ordering::SeqCst), 1);
    assert_eq!(gets.load(Ordering::SeqCst), 0);
    assert_eq!(puts.load(Ordering::SeqCst), 0);
    server.abort();
}

#[cfg(unix)]
mod cache_security {
    use std::os::unix::fs::{PermissionsExt, symlink};

    use super::super::*;

    #[test]
    fn concurrent_constructor_does_not_wait_for_an_existing_shared_operation() {
        let root = TempDirGuard::new("parallel-http-cache-construction");
        crate::cache::ensure_body_limited_cache_epoch(root.path(), false)
            .expect("seed cache epoch");
        let existing = crate::cache::lock_cache_shared(root.path()).expect("shared operation");
        let config = test_cache_config(root.path());
        let (finished_tx, finished_rx) = std::sync::mpsc::channel();
        let constructor = std::thread::spawn(move || {
            let result = build_http_client_with_config(SharedHttpClientKind::Default, config, None);
            finished_tx
                .send(result.map(|_| ()))
                .expect("report constructor result");
        });

        finished_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("constructor must not wait for an unrelated shared operation")
            .expect("construct HTTP client");
        drop(existing);
        constructor.join().expect("constructor thread");
    }

    #[test]
    fn build_http_client_repairs_unrelated_permissive_cache_state() {
        let root = TempDirGuard::new("http-cache-permission-repair");
        let cache_root = root.path().join("cache-root");
        let sentinel = cache_root.join("http/unrelated/sentinel");
        std::fs::create_dir_all(sentinel.parent().expect("sentinel parent")).expect("cache tree");
        std::fs::write(&sentinel, b"cached response").expect("sentinel");
        std::fs::write(
            cache_root.join(".body-limit-cache-v1"),
            b"bounded-response-body-v1\n",
        )
        .expect("current cache epoch");
        std::fs::set_permissions(&sentinel, std::fs::Permissions::from_mode(0o644))
            .expect("permissive sentinel");

        build_http_client_with_config(
            SharedHttpClientKind::Default,
            test_cache_config(&cache_root),
            None,
        )
        .expect("client construction repairs cache state");
        assert_eq!(
            std::fs::metadata(&sentinel)
                .expect("sentinel metadata")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }

    #[test]
    fn build_http_client_rejects_directory_symlink_inside_content_tree() {
        let root = TempDirGuard::new("http-cache-content-link");
        let cache_root = root.path().join("cache-root");
        let outside = root.path().join("outside");
        std::fs::create_dir_all(cache_root.join("http/content-v2/sha256")).expect("content tree");
        std::fs::write(
            cache_root.join(".body-limit-cache-v1"),
            b"bounded-response-body-v1\n",
        )
        .expect("current cache epoch");
        std::fs::create_dir(&outside).expect("outside directory");
        symlink(&outside, cache_root.join("http/content-v2/sha256/aa"))
            .expect("content directory symlink");

        let error = match build_http_client_with_config(
            SharedHttpClientKind::Default,
            test_cache_config(&cache_root),
            None,
        ) {
            Ok(_) => panic!("content directory symlink must be rejected"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("managed content directory"));
        assert_eq!(
            std::fs::read_dir(&outside)
                .expect("outside directory")
                .count(),
            0
        );
    }
}

#[tokio::test]
#[serial_test::serial(article_resolver_env)]
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
#[serial_test::serial(article_resolver_env)]
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
