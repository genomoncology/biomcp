//! The openFDA day-cache request kinds: label search and
//! adverse-event counts. A repeat of the same request inside one day is
//! answered from BioMCP's cache with no network request — the stored
//! answer holds whatever the provider's headers said when it was
//! fetched — an answer a day old goes back to the provider, and an
//! openFDA key keeps its unconditional cache bypass.

use crate::sources::openfda::OpenFdaClient;
use crate::test_support::FixtureEnv;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const LABEL_BODY: &str =
    r#"{"meta":{"results":{"skip":0,"limit":5,"total":1}},"results":[{"id":"label-1"}]}"#;
const COUNT_BODY: &str = r#"{"meta":{"results":{"skip":0,"limit":50,"total":1}},"results":[{"term":"nausea","count":3}]}"#;

#[derive(Clone)]
struct Reply {
    status: &'static str,
    headers: Vec<(&'static str, String)>,
    body: &'static str,
}

impl Reply {
    fn json(body: &'static str) -> Self {
        Self {
            status: "200 OK",
            headers: vec![
                ("Content-Type", "application/json".to_string()),
                // The day cache stores the answer whatever the headers
                // say, so the fixture forbids storage outright.
                ("Cache-Control", "no-store".to_string()),
            ],
            body,
        }
    }

    /// A reply already a day old on arrival: the stored entry's policy
    /// carries the provider's `Age` header, so the next lookup sees the
    /// answer past the day window without waiting for one.
    fn json_a_day_old(body: &'static str) -> Self {
        let mut reply = Self::json(body);
        reply.headers.push(("Age", (24 * 60 * 60 + 1).to_string()));
        reply
    }

    fn into_response(self) -> String {
        let mut response = format!("HTTP/1.1 {}\r\n", self.status);
        for (name, value) in &self.headers {
            response.push_str(&format!("{name}: {value}\r\n"));
        }
        response.push_str(&format!(
            "Content-Length: {}\r\nConnection: close\r\n\r\n{}",
            self.body.len(),
            self.body
        ));
        response
    }
}

/// A counting openFDA fixture: records every request line and answers
/// `drug/label.json` with the label reply and `drug/event.json` with
/// the count reply.
async fn day_cache_server(
    label: Reply,
    count: Reply,
) -> (String, Arc<Mutex<Vec<String>>>, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind openFDA day-cache fixture");
    let base = format!("http://{}", listener.local_addr().expect("fixture address"));
    let requests = Arc::new(Mutex::new(Vec::new()));
    let captured = Arc::clone(&requests);
    let label = Arc::new(label);
    let count = Arc::new(count);
    let task = tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let captured = Arc::clone(&captured);
            let label = Arc::clone(&label);
            let count = Arc::clone(&count);
            tokio::spawn(async move {
                let mut request = vec![0_u8; 16 * 1024];
                let len = stream.read(&mut request).await.expect("read request");
                let request = String::from_utf8_lossy(&request[..len]).into_owned();
                captured
                    .lock()
                    .expect("lock fixture requests")
                    .push(request.clone());
                let reply = if request.contains("/drug/label.json") {
                    (*label).clone()
                } else {
                    (*count).clone()
                };
                stream
                    .write_all(reply.into_response().as_bytes())
                    .await
                    .expect("write fixture response");
            });
        }
    });
    (base, requests, task)
}

/// An openFDA client whose cache and base point at the fixture, with
/// the shared client rebuilt so no earlier test's cache root leaks in.
fn day_cached_client(base: &str, env: &mut FixtureEnv) -> OpenFdaClient {
    env.set("BIOMCP_OPENFDA_BASE", base);
    crate::sources::reset_shared_http_client_for_tests();
    OpenFdaClient::new().expect("client against fixture")
}

fn request_count(requests: &Arc<Mutex<Vec<String>>>) -> usize {
    requests.lock().expect("lock fixture requests").len()
}

/// The repeat label search inside a day makes no request: the stored
/// answer serves even though the provider forbade storage, and the
/// answer is the same.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn a_repeat_label_search_inside_a_day_makes_no_request() {
    let (base, requests, server) =
        day_cache_server(Reply::json(LABEL_BODY), Reply::json(COUNT_BODY)).await;
    let mut env = FixtureEnv::scratch_cache_only();
    let client = day_cached_client(&base, &mut env);

    let first = client
        .label_search("vemurafenib")
        .await
        .expect("first label search")
        .expect("a label answer");
    let second = client
        .label_search("vemurafenib")
        .await
        .expect("repeat label search")
        .expect("the repeat serves the same answer");

    server.abort();
    drop(env);
    assert_eq!(
        request_count(&requests),
        1,
        "the repeat must serve from the cache"
    );
    assert_eq!(first, second);
}

/// The stored label answer a day old goes back to the provider.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn a_label_answer_a_day_old_is_refetched() {
    let (base, requests, server) =
        day_cache_server(Reply::json_a_day_old(LABEL_BODY), Reply::json(COUNT_BODY)).await;
    let mut env = FixtureEnv::scratch_cache_only();
    let client = day_cached_client(&base, &mut env);

    let first = client
        .label_search("vemurafenib")
        .await
        .expect("first label search")
        .expect("a label answer");
    let second = client
        .label_search("vemurafenib")
        .await
        .expect("refetched label search")
        .expect("the refetch answers");

    server.abort();
    drop(env);
    assert_eq!(request_count(&requests), 2, "the aged answer must refetch");
    assert_eq!(first, second);
}

/// The repeat adverse-event count inside a day makes no request.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn a_repeat_adverse_event_count_inside_a_day_makes_no_request() {
    let (base, requests, server) =
        day_cache_server(Reply::json(LABEL_BODY), Reply::json(COUNT_BODY)).await;
    let mut env = FixtureEnv::scratch_cache_only();
    let client = day_cached_client(&base, &mut env);

    let first = client
        .faers_count(
            "patient.drug.medicinalproduct:\"pembrolizumab\"",
            "patient.reaction.reactionmeddrapt",
            5,
        )
        .await
        .expect("first count")
        .expect("a count answer");
    let second = client
        .faers_count(
            "patient.drug.medicinalproduct:\"pembrolizumab\"",
            "patient.reaction.reactionmeddrapt",
            5,
        )
        .await
        .expect("repeat count")
        .expect("the repeat serves the same answer");

    server.abort();
    drop(env);
    assert_eq!(
        request_count(&requests),
        1,
        "the repeat must serve from the cache"
    );
    assert_eq!(first.results, second.results);
}

/// The stored adverse-event count a day old goes back to the provider.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn an_adverse_event_count_a_day_old_is_refetched() {
    let (base, requests, server) =
        day_cache_server(Reply::json(LABEL_BODY), Reply::json_a_day_old(COUNT_BODY)).await;
    let mut env = FixtureEnv::scratch_cache_only();
    let client = day_cached_client(&base, &mut env);

    let first = client
        .faers_count(
            "patient.drug.medicinalproduct:\"pembrolizumab\"",
            "patient.reaction.reactionmeddrapt",
            5,
        )
        .await
        .expect("first count")
        .expect("a count answer");
    let second = client
        .faers_count(
            "patient.drug.medicinalproduct:\"pembrolizumab\"",
            "patient.reaction.reactionmeddrapt",
            5,
        )
        .await
        .expect("refetched count")
        .expect("the refetch answers");

    server.abort();
    drop(env);
    assert_eq!(request_count(&requests), 2, "the aged answer must refetch");
    assert_eq!(first.results, second.results);
}

/// An openFDA key keeps the cache bypassed: both label searches go to
/// the provider carrying the key.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn an_openfda_key_keeps_the_cache_bypassed() {
    let (base, requests, server) =
        day_cache_server(Reply::json(LABEL_BODY), Reply::json(COUNT_BODY)).await;
    let mut env = FixtureEnv::scratch_cache_only();
    env.set("OPENFDA_API_KEY", "test-key");
    let client = day_cached_client(&base, &mut env);

    let first = client
        .label_search("vemurafenib")
        .await
        .expect("first label search")
        .expect("a label answer");
    let second = client
        .label_search("vemurafenib")
        .await
        .expect("second label search")
        .expect("the second fetch answers");

    server.abort();
    drop(env);
    let requests = requests.lock().expect("lock fixture requests").join("\n");
    assert_eq!(
        requests.matches("/drug/label.json").count(),
        2,
        "a keyed search must hit the provider each time"
    );
    assert!(
        requests.contains("api_key=test-key"),
        "the keyed requests must carry the key: {requests}"
    );
    assert_eq!(first, second);
}

/// Cache mode off keeps every request on the network.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn cache_mode_off_never_serves_the_stored_label_answer() {
    let (base, requests, server) =
        day_cache_server(Reply::json(LABEL_BODY), Reply::json(COUNT_BODY)).await;
    let mut env = FixtureEnv::new();
    let client = day_cached_client(&base, &mut env);

    let first = client
        .label_search("vemurafenib")
        .await
        .expect("first label search")
        .expect("a label answer");
    let second = client
        .label_search("vemurafenib")
        .await
        .expect("second label search")
        .expect("the second fetch answers");

    server.abort();
    drop(env);
    assert_eq!(
        request_count(&requests),
        2,
        "mode off must hit the provider each time"
    );
    assert_eq!(first, second);
}
