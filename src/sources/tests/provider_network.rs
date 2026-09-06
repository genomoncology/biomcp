use std::collections::BTreeMap;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
};

use super::super::{
    SharedHttpClientKind, build_http_client_with_config_and_manager, build_uncached_http_client,
};
use axum::{
    Router,
    body::Body,
    extract::State,
    http::{HeaderMap, Response},
    response::IntoResponse,
    routing::get,
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
            Ok(
                crate::cache::SizeAwareCacheManager::new_with_cache_observers(
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
                ),
            )
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
            cached_test_client(mode, "post-write-client-failure").await;
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

#[tokio::test]
#[serial_test::serial(article_resolver_env)]
async fn provider_deadline_waits_for_post_publish_fail_closed_finalization() {
    let (client, url, requests, gets, puts, armed, _, delay, root, server) =
        cached_test_client(CacheOriginMode::Initial, "deadline-post-write-finalization").await;
    let _env = EnvRestore::set(&[(
        "BIOMCP_TEST_UNPACED_ORIGIN",
        Some(url.trim_end_matches("/resource")),
    )]);
    delay.store(40, Ordering::SeqCst);
    armed.store(true, Ordering::SeqCst);
    let deadline =
        crate::sources::VariantArticleDeadline::from_now(std::time::Duration::from_millis(10));
    let request_deadline = deadline.clone();
    let result = crate::sources::with_variant_article_deadline(deadline, async move {
        client
            .get(&url)
            .with_extension(request_deadline)
            .send()
            .await
    })
    .await;

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
        cached_test_client(CacheOriginMode::Fresh, "fresh-client-hit").await;
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
        cached_test_client(CacheOriginMode::Initial, "no-store-client-request").await;
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
