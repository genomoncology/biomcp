//! Actual optional transport and NotFound continuity complement refusal witnesses.
use super::*;

fn reply(status: &str, body: &str) -> TestHttpReply {
    TestHttpReply::Bytes(test_http_response(
        status,
        "application/json",
        body.as_bytes(),
    ))
}
pub(super) async fn optional_paths(harness: &ContractHarness) {
    {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&requests);
        let fixture = TestHttpFixture::spawn(move |request| {
            captured
                .lock()
                .unwrap()
                .push(request.lines().next().unwrap().to_owned());
            if request.starts_with("GET /disease/MONDO:1?") {
                reply(
                    "200 OK",
                    r#"{"_id":"MONDO:1","mondo":{"name":"Root tumor","parents":["MONDO:2"]}}"#,
                )
            } else if request.starts_with("GET /disease/") || request.starts_with("GET /query?") {
                if request.starts_with("GET /query?") {
                    reply("200 OK", r#"{"total":0,"hits":[]}"#)
                } else {
                    reply("404 Not Found", "{}")
                }
            } else {
                reply("200 OK", r#"{"data":{}}"#)
            }
        })
        .await;
        let cache = tempfile::tempdir().unwrap();
        let value = command(
            harness,
            &["--json", "--no-cache", "get", "disease", "MONDO:1", "civic"],
            &environment(&fixture.base, cache.path()),
            false,
        )
        .await;
        assert_eq!(value["parents"][0], "MONDO:2");
        let observed = requests.lock().unwrap();
        assert!(observed.iter().any(|line| line.starts_with("GET /query?")));
        assert!(observed.iter().any(|line| line.starts_with("POST /")));
    }
    for detail in [false, true] {
        {
            let requests = Arc::new(Mutex::new(Vec::new()));
            let captured = Arc::clone(&requests);
            let discovers = Mutex::new(0);
            let first_gets = Mutex::new(0);
            let fixture = TestHttpFixture::spawn(move |request| {
                captured.lock().unwrap().push(request.lines().next().unwrap().to_owned());
                if request.starts_with("GET /query?") {
                    reply("200 OK", r#"{"total":0,"hits":[]}"#)
                } else if request.starts_with("GET /api/search?") {
                    let mut count = discovers.lock().unwrap();
                    *count += 1;
                    // Candidate failure continues to the next candidate. Detail failure
                    // continues to the next resolver query before fetching its candidate.
                    if detail && *count == 1 {
                        reply("200 OK", r#"{"response":{"docs":[{"iri":"https://example.invalid/MONDO_1","obo_id":"MONDO:1","ontology_prefix":"mondo","label":"Synthetic cancer","type":"class"}]}}"#)
                    } else {
                        reply("200 OK", r#"{"response":{"docs":[{"iri":"https://example.invalid/MONDO_1","obo_id":"MONDO:1","ontology_prefix":"mondo","label":"Synthetic cancer","type":"class"},{"iri":"https://example.invalid/MONDO_2","obo_id":"MONDO:2","ontology_prefix":"mondo","label":"Synthetic cancer","type":"class"}]}}"#)
                    }
                } else if request.starts_with("GET /disease/MONDO:1?") {
                    let mut count = first_gets.lock().unwrap();
                    *count += 1;
                    if detail && *count == 1 {
                        reply("200 OK", r#"{"_id":"MONDO:1","mondo":{"name":"Synthetic cancer"}}"#)
                    } else { reply("404 Not Found", "{}") }
                } else if request.starts_with("GET /disease/MONDO:2?") {
                    reply("200 OK", r#"{"_id":"MONDO:2","mondo":{"name":"Synthetic cancer"}}"#)
                } else { reply("200 OK", r#"{"data":{}}"#) }
            }).await;
            let cache = tempfile::tempdir().unwrap();
            let value = command(
                harness,
                &["--json", "--no-cache", "get", "disease", "Synthetic cancer"],
                &environment(&fixture.base, cache.path()),
                false,
            )
            .await;
            assert_eq!(value["id"], "MONDO:2", "detail={detail}: {value}");
            let observed = requests.lock().unwrap();
            assert!(
                observed
                    .iter()
                    .any(|line| line.starts_with("GET /disease/MONDO:2?"))
            );
            if detail {
                assert!(
                    observed
                        .iter()
                        .filter(|line| line.starts_with("GET /api/search?"))
                        .count()
                        >= 2
                );
            }
        }
    }
    unavailable_service_paths(harness).await;
}

async fn unavailable_service_paths(harness: &ContractHarness) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    for accepted in [1, 2, 3] {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            for step in 0..if accepted == 1 { 1 } else { accepted - 1 } {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut buf = vec![0; 16384];
                let n = socket.read(&mut buf).await.unwrap();
                let line = String::from_utf8_lossy(&buf[..n]);
                let body = if accepted == 1 {
                    assert!(line.starts_with("GET /disease/MONDO:1?"));
                    r#"{"_id":"MONDO:1","mondo":{"name":"Root tumor","parents":["MONDO:2"]}}"#
                } else if step == 0 {
                    assert!(line.starts_with("GET /query?"));
                    r#"{"total":0,"hits":[]}"#
                } else {
                    assert!(line.starts_with("GET /disease/MONDO:1?"));
                    r#"{"_id":"MONDO:1","mondo":{"name":"Synthetic tumor"}}"#
                };
                // Close the listener before the last accepted response becomes visible.
                if step + 1 == if accepted == 1 { 1 } else { accepted - 1 } {
                    drop(listener);
                    socket
                        .write_all(&test_http_response(
                            "200 OK",
                            "application/json",
                            body.as_bytes(),
                        ))
                        .await
                        .unwrap();
                    return;
                }
                socket
                    .write_all(&test_http_response(
                        "200 OK",
                        "application/json",
                        body.as_bytes(),
                    ))
                    .await
                    .unwrap();
            }
        });
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&requests);
        let other = TestHttpFixture::spawn(move |request| {
            captured.lock().unwrap().push(request.lines().next().unwrap().to_owned());
            if request.starts_with("GET /api/search?") {
                reply("200 OK", r#"{"response":{"docs":[{"iri":"https://example.invalid/MONDO_1","obo_id":"MONDO:1","ontology_prefix":"mondo","label":"Synthetic tumor","type":"class"}]}}"#)
            } else { reply("200 OK", r#"{"data":{}}"#) }
        }).await;
        let cache = tempfile::tempdir().unwrap();
        let mut env = environment(&other.base, cache.path());
        env[0].1 = base.clone();
        env[4].1 = base;
        let value = command(
            harness,
            if accepted == 1 {
                &["--json", "--no-cache", "get", "disease", "MONDO:1", "civic"]
            } else {
                &["--json", "--no-cache", "get", "disease", "Synthetic tumor"]
            },
            &env,
            accepted != 1,
        )
        .await;
        server.await.unwrap();
        if accepted == 1 {
            assert_eq!(value["parents"][0], "MONDO:2");
        } else {
            assert_eq!(
                value["_meta"]["not_found"], true,
                "optional unavailable service must reach ordinary unresolved outcome: {value}"
            );
        }
    }
}
