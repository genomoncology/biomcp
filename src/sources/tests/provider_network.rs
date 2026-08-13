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
