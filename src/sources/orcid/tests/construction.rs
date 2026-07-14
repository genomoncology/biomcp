use super::super::*;
use crate::error::BioMcpError;
use crate::sources::{HttpMethod, apply_no_store, request_from_plan};
use http::Extensions;
use http_cache_reqwest::CacheMode;
use reqwest_middleware::{Middleware, Next};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::{Duration, timeout};

const RECORD: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/sources/orcid/record.json"
));

#[test]
fn accepts_ordinary_and_x_checksums_and_builds_exact_anonymous_plans() {
    let record = OrcidClient::record_plan("0000-0002-7433-2740").unwrap();
    assert_eq!(record.method, HttpMethod::Get);
    assert_eq!(record.path, "0000-0002-7433-2740/record");
    assert_eq!(record.header_value("accept"), Some(ORCID_MEDIA_TYPE));
    assert_eq!(record.header_value("authorization"), None);
    assert!(record.query.is_empty());

    let works = OrcidClient::works_plan("0000-0002-1694-233X").unwrap();
    assert_eq!(works.method, HttpMethod::Get);
    assert_eq!(works.path, "0000-0002-1694-233X/works");
    assert_eq!(works.header_value("Accept"), Some(ORCID_MEDIA_TYPE));
    assert!(!works.has_query("offset"));
    assert!(!works.has_query("limit"));
}

#[test]
fn rejects_malformed_checksum_and_path_shaped_ids_before_construction() {
    for value in [
        "",
        "   ",
        "0000-0002-7433-2741",
        "0000/0002/7433/2740",
        r"0000\0002\7433\2740",
        ".",
        "..",
        "0000-0002-7433-2740/../record",
        " 0000-0002-7433-2740",
        "0000-0002-1694-233x",
    ] {
        assert!(
            matches!(
                OrcidClient::record_plan(value),
                Err(BioMcpError::InvalidArgument(_))
            ),
            "unexpected valid ORCID: {value:?}"
        );
    }
}

#[derive(Clone)]
struct CaptureNoStore(Arc<AtomicBool>);

#[async_trait::async_trait]
impl Middleware for CaptureNoStore {
    async fn handle(
        &self,
        req: reqwest::Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<reqwest::Response> {
        self.0.store(
            matches!(extensions.get::<CacheMode>(), Some(CacheMode::NoStore)),
            Ordering::SeqCst,
        );
        next.run(req, extensions).await
    }
}

#[tokio::test]
async fn apply_no_store_overrides_force_cache_before_middleware_execution() {
    let (listener, base) = bind_server().await;
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        read_request(&mut stream).await;
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
            .await
            .unwrap();
    });
    let observed = Arc::new(AtomicBool::new(false));
    let raw = reqwest::Client::new();
    let client = reqwest_middleware::ClientBuilder::new(raw)
        .with(CaptureNoStore(observed.clone()))
        .build();
    let plan = OrcidClient::record_plan("0000-0002-7433-2740").unwrap();
    apply_no_store(request_from_plan(&client, &base, &plan).with_extension(CacheMode::ForceCache))
        .send()
        .await
        .unwrap();
    server.await.unwrap();
    assert!(observed.load(Ordering::SeqCst));
}

async fn bind_server() -> (tokio::net::TcpListener, String) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    (listener, base)
}

async fn read_request(stream: &mut tokio::net::TcpStream) -> String {
    let mut request = Vec::new();
    loop {
        let mut chunk = [0_u8; 2048];
        let read = stream.read(&mut chunk).await.unwrap();
        if read == 0 {
            break;
        }
        request.extend_from_slice(&chunk[..read]);
        if request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
            break;
        }
    }
    String::from_utf8(request).unwrap()
}

fn fixture_client(base: String, redirect: reqwest::redirect::Policy) -> OrcidClient {
    let raw = reqwest::Client::builder()
        .redirect(redirect)
        .build()
        .unwrap();
    OrcidClient {
        client: reqwest_middleware::ClientBuilder::new(raw).build(),
        base: Cow::Owned(base),
    }
}

#[tokio::test]
async fn plans_are_consumed_and_same_origin_canonical_redirect_is_preserved() {
    let (listener, base) = bind_server().await;
    let canonical = "0000-0002-7433-2740";
    let requested = "0000-0002-1825-0097";
    let response_body = String::from_utf8(RECORD.to_vec()).unwrap();
    let server_base = base.clone();
    let server = tokio::spawn(async move {
        let (mut first, _) = listener.accept().await.unwrap();
        let first_request = read_request(&mut first).await;
        let redirect = format!(
            "HTTP/1.1 301 Moved Permanently\r\nLocation: {server_base}/{canonical}/record\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        );
        first.write_all(redirect.as_bytes()).await.unwrap();

        let (mut second, _) = listener.accept().await.unwrap();
        let second_request = read_request(&mut second).await;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {ORCID_MEDIA_TYPE};charset=UTF-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response_body}",
            response_body.len()
        );
        second.write_all(response.as_bytes()).await.unwrap();
        (first_request, second_request)
    });

    let client = fixture_client(base, crate::sources::rate_limit::orcid_redirect_policy());
    let outcome = client.record(requested).await.unwrap();
    match outcome {
        OrcidFetchOutcome::Redirected {
            requested_orcid,
            canonical_orcid,
            ..
        } => {
            assert_eq!(requested_orcid, requested);
            assert_eq!(canonical_orcid, canonical);
        }
        other => panic!("unexpected outcome: {other:?}"),
    }
    let (first, second) = server.await.unwrap();
    assert!(first.starts_with(&format!("GET /{requested}/record HTTP/1.1")));
    assert!(second.starts_with(&format!("GET /{canonical}/record HTTP/1.1")));
    assert!(
        first
            .to_ascii_lowercase()
            .contains("accept: application/vnd.orcid+json")
    );
    assert!(!first.to_ascii_lowercase().contains("authorization:"));
}

#[tokio::test]
async fn cross_origin_redirect_is_rejected_before_target_receives_request() {
    let (target_listener, target_base) = bind_server().await;
    let (source_listener, source_base) = bind_server().await;
    let source = tokio::spawn(async move {
        let (mut stream, _) = source_listener.accept().await.unwrap();
        read_request(&mut stream).await;
        let response = format!(
            "HTTP/1.1 302 Found\r\nLocation: {target_base}/outside\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        );
        stream.write_all(response.as_bytes()).await.unwrap();
    });
    let target =
        tokio::spawn(
            async move { timeout(Duration::from_millis(400), target_listener.accept()).await },
        );

    let client = fixture_client(
        source_base,
        crate::sources::rate_limit::orcid_redirect_policy(),
    );
    let error = client.record("0000-0002-7433-2740").await.unwrap_err();
    assert!(error.to_string().contains("redirect"));
    source.await.unwrap();
    assert!(
        target.await.unwrap().is_err(),
        "target unexpectedly received a request"
    );
}
