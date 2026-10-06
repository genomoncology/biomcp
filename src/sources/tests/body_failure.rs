use super::*;

pub(super) async fn read_limited_source_body_classifies_chunk_failures_as_retryable() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind fixture");
    let address = listener.local_addr().expect("fixture address");
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept request");
        let mut request = [0; 1024];
        let bytes_read = stream
            .read(&mut request)
            .await
            .expect("read fixture request");
        assert!(bytes_read > 0, "fixture request must not be empty");
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\nConnection: close\r\n\r\nshort")
            .await
            .expect("write truncated response");
    });
    let response = reqwest::Client::builder()
        .no_proxy()
        .build()
        .expect("fixture client")
        .get(format!("http://{address}"))
        .send()
        .await
        .expect("receive response headers");
    let error = read_limited_source_body_with_limit(
        response,
        SourceContext::narrow(crate::error::SourceProvider::OLS4),
        1_000,
    )
    .await
    .expect_err("truncated response body should fail");

    assert_eq!(error.code(), "http");
    assert_eq!(
        error.public_projection().recovery,
        Some(crate::error::RecoveryAction::RetryRemoteSource.message())
    );
    server.await.expect("fixture server");
}
