use std::collections::BTreeMap;

use super::super::{SharedHttpClientKind, build_uncached_http_client};
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
