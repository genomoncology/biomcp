use super::super::*;
use crate::sources::HttpMethod;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[test]
fn cspec_production_timeouts_match_shared_provider_policy() {
    let timeouts = CspecTimeouts::default();
    assert_eq!(timeouts.connect, Duration::from_secs(10));
    assert_eq!(timeouts.request, Duration::from_secs(30));
}

#[test]
fn cspec_plans_keep_manifest_and_document_provider_paths() {
    let manifest = CspecClient::manifest_plan("ATM");
    assert_eq!(manifest.method, HttpMethod::Get);
    assert_eq!(
        manifest.path,
        "cspec/Gene/id/ATM/SequenceVariantInterpretation/version"
    );
    assert_eq!(manifest.query_value("detail"), Some("low"));

    let iri = Url::parse(
        "https://cspec.genome.network/cspec/SequenceVariantInterpretation/id/GN020/version/1.5.1",
    )
    .expect("CSpec IRI");
    let document = CspecClient::document_plan(&iri);
    assert_eq!(document.method, HttpMethod::Get);
    assert_eq!(
        document.path,
        "/cspec/SequenceVariantInterpretation/id/GN020/version/1.5.1"
    );
    assert!(document.query.is_empty());
}

#[tokio::test]
async fn cspec_execution_methods_consume_manifest_and_document_plans() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind CSpec fixture");
    let base = format!("http://{}", listener.local_addr().expect("fixture address"));
    let manifest = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/testdata/sources/clingen_cspec/atm-manifest.json"
    ));
    let document = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/testdata/sources/clingen_cspec/atm-gn020-1.5.1.json"
    ));
    let server = tokio::spawn(async move {
        let mut requests = Vec::new();
        for body in [manifest, document] {
            let (mut stream, _) = listener.accept().await.expect("accept CSpec request");
            let mut request = Vec::new();
            loop {
                let mut chunk = [0_u8; 4096];
                let read = stream.read(&mut chunk).await.expect("read CSpec request");
                if read == 0 {
                    break;
                }
                request.extend_from_slice(&chunk[..read]);
                if request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
                    break;
                }
            }
            requests.push(String::from_utf8_lossy(&request).into_owned());
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body,
            );
            stream
                .write_all(response.as_bytes())
                .await
                .expect("write response");
        }
        requests
    });
    let client =
        CspecClient::with_test_client_at(crate::sources::test_client().expect("test client"), base);

    let manifest = client.manifest("ATM").await.expect("manifest response");
    assert_eq!(
        manifest["data"][0]["@id"].as_str(),
        Some(
            "https://cspec.genome.network/cspec/SequenceVariantInterpretation/id/GN020/version/1.5.1"
        )
    );
    client
        .document(&Url::parse("https://cspec.genome.network/cspec/SequenceVariantInterpretation/id/GN020/version/1.5.1").expect("CSpec IRI"))
        .await
        .expect("document response");

    let requests = server.await.expect("CSpec fixture server");
    assert!(
        requests[0].starts_with(
            "GET /cspec/Gene/id/ATM/SequenceVariantInterpretation/version?detail=low "
        )
    );
    assert!(
        requests[1].starts_with("GET /cspec/SequenceVariantInterpretation/id/GN020/version/1.5.1 ")
    );
}

#[tokio::test]
async fn cspec_request_deadline_covers_headers_and_body_with_safe_attribution() {
    for stall_after_headers in [false, true] {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind timeout fixture");
        let base = format!("http://{}", listener.local_addr().expect("fixture address"));
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept request");
            let mut request = [0_u8; 1024];
            let _ = stream.read(&mut request).await.expect("read request");
            if stall_after_headers {
                stream
                    .write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 100\r\n\r\n{")
                    .await
                    .expect("write partial response");
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        });
        let client = CspecClient::with_test_timeouts_at(
            base,
            Duration::from_millis(25),
            Duration::from_millis(25),
        )
        .expect("test client");

        let error = client
            .manifest("PTEN")
            .await
            .expect_err("stalled request must time out");
        let projection = error.public_projection();
        assert_eq!(projection.source, Some("ClinGen CSpec"));
        assert!(!projection.message.contains("127.0.0.1"));
        server.await.expect("timeout fixture");
    }
}
