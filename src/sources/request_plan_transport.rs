use super::*;
use serde::Deserialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

struct CapturedRequest {
    head: String,
    body: Vec<u8>,
}

async fn read_request(stream: &mut tokio::net::TcpStream) -> CapturedRequest {
    let mut bytes = Vec::new();
    let (head_end, content_length) = loop {
        let mut chunk = [0_u8; 1024];
        let read = stream.read(&mut chunk).await.expect("read request");
        assert!(read > 0, "request ended before its headers");
        bytes.extend_from_slice(&chunk[..read]);
        if let Some(head_end) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
            let head_end = head_end + 4;
            let head = String::from_utf8_lossy(&bytes[..head_end]);
            let content_length = head
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().expect("content length"))
                })
                .unwrap_or(0);
            break (head_end, content_length);
        }
    };
    while bytes.len() < head_end + content_length {
        let mut chunk = [0_u8; 1024];
        let read = stream.read(&mut chunk).await.expect("read request body");
        assert!(read > 0, "request ended before its declared body");
        bytes.extend_from_slice(&chunk[..read]);
    }
    CapturedRequest {
        head: String::from_utf8(bytes[..head_end].to_vec()).expect("UTF-8 request head"),
        body: bytes[head_end..head_end + content_length].to_vec(),
    }
}

async fn send_and_capture(
    plan: RequestPlan,
    status: &str,
    content_type: &str,
    response_body: &str,
) -> (CapturedRequest, reqwest::Response) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind request-plan fixture");
    let address = listener.local_addr().expect("fixture address");
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response_body}",
        response_body.len()
    );
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept request");
        let captured = read_request(&mut stream).await;
        stream
            .write_all(response.as_bytes())
            .await
            .expect("write response");
        captured
    });
    let client = test_client().expect("test client");
    let response = request_from_plan(&client, &format!("http://{address}/"), &plan)
        .send_with_source_context(SourceContext::retry(crate::error::SourceProvider::OLS4))
        .await
        .expect("send request plan");
    (server.await.expect("fixture server"), response)
}

#[derive(Debug, Deserialize, PartialEq)]
struct FixturePayload {
    value: String,
}

#[tokio::test]
async fn request_plan_preserves_path_query_headers_and_success_bytes() {
    let plan = RequestPlan::get("/v1/search")
        .query("tag", "alpha beta")
        .query("tag", "A/B")
        .header("x-trace", "one")
        .header("x-trace", "two");
    let (captured, response) =
        send_and_capture(plan, "200 OK", "application/json", r#"{"value":"ok"}"#).await;

    assert!(
        captured
            .head
            .starts_with("GET /v1/search?tag=alpha+beta&tag=A%2FB HTTP/1.1\r\n")
    );
    assert_eq!(captured.head.matches("x-trace:").count(), 2);
    assert!(captured.head.contains("x-trace: one\r\n"));
    assert!(captured.head.contains("x-trace: two\r\n"));
    assert!(captured.body.is_empty());

    let status = response.status();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .cloned();
    let bytes = read_limited_source_body_with_limit(
        response,
        SourceContext::narrow(crate::error::SourceProvider::OLS4),
        64,
    )
    .await
    .expect("read response through production body seam");
    let decoded: FixturePayload = decode_json(
        SourceContext::narrow(crate::error::SourceProvider::OLS4),
        status,
        content_type.as_ref(),
        &bytes,
        true,
    )
    .expect("decode response through production JSON seam");
    assert_eq!(decoded, FixturePayload { value: "ok".into() });
}

#[tokio::test]
async fn request_plan_preserves_each_supported_post_body() {
    let cases = [
        (
            RequestPlan::post("text")
                .header("content-type", "text/plain")
                .text("alpha beta"),
            "text/plain",
            "alpha beta",
        ),
        (
            RequestPlan::post("form").form(vec![
                ("name".into(), "alpha beta".into()),
                ("name".into(), "A/B".into()),
            ]),
            "application/x-www-form-urlencoded",
            "name=alpha+beta&name=A%2FB",
        ),
        (
            RequestPlan::post("json").json(serde_json::json!({"name": "alpha beta"})),
            "application/json",
            r#"{"name":"alpha beta"}"#,
        ),
    ];

    for (plan, content_type, expected_body) in cases {
        let (captured, _) = send_and_capture(plan, "200 OK", "application/json", "{}").await;
        assert!(captured.head.starts_with("POST /"));
        assert!(
            captured
                .head
                .to_ascii_lowercase()
                .contains(&format!("content-type: {content_type}"))
        );
        assert_eq!(captured.body, expected_body.as_bytes());
    }
}

#[tokio::test]
async fn request_plan_failures_keep_safe_source_context() {
    let (_, response) = send_and_capture(
        RequestPlan::get("status"),
        "503 Service Unavailable",
        "application/json",
        r#"{"error":"down"}"#,
    )
    .await;
    let status = response.status();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .cloned();
    let bytes = read_limited_source_body_with_limit(
        response,
        SourceContext::retry(crate::error::SourceProvider::OLS4),
        64,
    )
    .await
    .expect("read error body");
    let error = decode_json::<FixturePayload>(
        SourceContext::retry(crate::error::SourceProvider::OLS4),
        status,
        content_type.as_ref(),
        &bytes,
        true,
    )
    .expect_err("HTTP failure should not decode");
    assert_eq!(error.public_projection().source, Some("OLS4"));
    assert!(!error.to_string().contains("127.0.0.1"));

    let (_, response) = send_and_capture(
        RequestPlan::get("malformed"),
        "200 OK",
        "application/json",
        "not-json",
    )
    .await;
    let status = response.status();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .cloned();
    let bytes = read_limited_source_body_with_limit(
        response,
        SourceContext::retry(crate::error::SourceProvider::OLS4),
        64,
    )
    .await
    .expect("read malformed body");
    let error = decode_json::<FixturePayload>(
        SourceContext::retry(crate::error::SourceProvider::OLS4),
        status,
        content_type.as_ref(),
        &bytes,
        true,
    )
    .expect_err("malformed JSON should fail");
    assert_eq!(error.public_projection().source, Some("OLS4"));
}
