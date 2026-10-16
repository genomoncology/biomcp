//! Article-search outage rule: a zero-row search while a source was
//! unavailable reports `complete: false`, a structured `source_unavailable`
//! error, and exit 1; an empty search where every source answered stays an
//! ok result with exit 0 and `complete: true`; rows that arrived beside a
//! down source keep exit 0 with `complete: false`. The same rule is already
//! documented for `variant articles` (docs/how-to/find-articles.md) and
//! coded at `src/entities/article/variant_search.rs`.
use std::sync::{Arc, Mutex};

use clap::Parser;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::cli::{Cli, Commands, SearchEntity};

/// Which reply the Semantic Scholar search leg serves. `Down` models the
/// reported outage: no API key and the provider refusing the request.
#[derive(Clone, Copy, PartialEq, Eq)]
enum SemanticScholarMode {
    Down,
    Empty,
}

/// Which reply the primary search legs serve: every provider answering with
/// no rows, or Europe PMC alone returning one row while the rest stay empty.
#[derive(Clone, Copy, PartialEq, Eq)]
enum PrimaryMode {
    Empty,
    EuropeRow,
}

/// One fixture origin stands in for every article provider. The request
/// target distinguishes them, matching the shapes the deadline fixtures
/// route: PubTator3 searches carry `text=`, Europe PMC searches carry
/// `query=`, PubMed hits its eutils paths, and Semantic Scholar keeps its
/// `/graph/v1` routes.
fn outage_fixture_reply(
    request: &str,
    semantic_scholar: SemanticScholarMode,
    primary: PrimaryMode,
) -> Vec<u8> {
    let target = request
        .split_whitespace()
        .nth(1)
        .expect("fixture request line carries a target");
    let path = target.split('?').next().unwrap_or(target);
    let http = |status: &str, body: &str| {
        let mut response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )
        .into_bytes();
        response.extend_from_slice(body.as_bytes());
        response
    };
    if path.starts_with("/entity/autocomplete/") {
        return http("200 OK", "[]");
    }
    if semantic_scholar == SemanticScholarMode::Down && path.starts_with("/graph/v1/") {
        return http("503 Service Unavailable", "{}");
    }
    if path.starts_with("/graph/v1/paper/search") {
        return http("200 OK", r#"{"total":0,"data":[]}"#);
    }
    if path.starts_with("/graph/v1/paper/batch") {
        return http("200 OK", r#"[null,null,null,null,null]"#);
    }
    if path.starts_with("/publications/export/biocjson") {
        return http("200 OK", r#"{"documents":[]}"#);
    }
    if path.starts_with("/search/") && target.contains("text=") {
        return http(
            "200 OK",
            r#"{"results":[],"count":0,"total_pages":1,"current":1,"page_size":25,"facets":{}}"#,
        );
    }
    if path.starts_with("/search") && target.contains("query=") {
        let body = match primary {
            PrimaryMode::EuropeRow => {
                r#"{"hitCount":1,"resultList":{"result":[{"id":"41800004","pmid":"41800004","title":"outage fixture Europe PMC row","journalTitle":"Fixture Journal","firstPublicationDate":"2026-01-02","authorString":"Fixture Author"}]}}"#
            }
            PrimaryMode::Empty => r#"{"hitCount":0,"resultList":{"result":[]}}"#,
        };
        return http("200 OK", body);
    }
    if path.ends_with("/esearch.fcgi") {
        return http("200 OK", r#"{"esearchresult":{"count":"0","idlist":[]}}"#);
    }
    if path.ends_with("/esummary.fcgi") {
        return http("200 OK", r#"{"result":{"uids":[]}}"#);
    }
    http("404 Not Found", "{}")
}

/// A local fixture server answering every article provider route. The shared
/// `source_status` pin records that Semantic Scholar was actually asked, so
/// the down-source assertions cannot pass vacuously.
async fn outage_fixture(
    semantic_scholar: SemanticScholarMode,
    primary: PrimaryMode,
) -> (String, Arc<Mutex<Vec<String>>>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind article outage fixture");
    let base = format!("http://{}", listener.local_addr().expect("fixture address"));
    let requests = Arc::new(Mutex::new(Vec::new()));
    let captured = requests.clone();
    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let captured = captured.clone();
            tokio::spawn(async move {
                let mut request = vec![0_u8; 16 * 1024];
                let length = stream.read(&mut request).await.unwrap_or(0);
                let request = String::from_utf8_lossy(&request[..length]).into_owned();
                captured
                    .lock()
                    .expect("lock fixture requests")
                    .push(request.clone());
                let response = outage_fixture_reply(&request, semantic_scholar, primary);
                stream
                    .write_all(&response)
                    .await
                    .expect("write fixture response");
            });
        }
    });
    (base, requests)
}

async fn outage_env(base: &str) -> crate::test_support::FixtureEnv {
    let mut env = crate::test_support::FixtureEnv::new();
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", base);
    env.set("BIOMCP_S2_BASE", base);
    env.set("BIOMCP_PUBTATOR_BASE", base);
    env.set("BIOMCP_EUROPEPMC_BASE", base);
    env.set("BIOMCP_PUBMED_BASE", &format!("{base}/entrez/eutils"));
    env
}

async fn search_article_outcome(argv: &[&str]) -> crate::cli::CommandOutcome {
    let mut raw = vec![
        "biomcp".to_string(),
        "--json".to_string(),
        "search".to_string(),
        "article".to_string(),
    ];
    raw.extend(argv.iter().map(|value| value.to_string()));
    let cli = Cli::try_parse_from(raw).expect("parse article search argv");
    let Commands::Search {
        entity: SearchEntity::Article(args),
    } = cli.command
    else {
        panic!("expected article search");
    };
    crate::cli::article::handle_search(args, true)
        .await
        .expect("article search completes")
}

fn reached_provider(requests: &Arc<Mutex<Vec<String>>>, prefix: &str) -> bool {
    requests
        .lock()
        .expect("lock fixture requests")
        .iter()
        .any(|request| request.contains(prefix))
}

/// The reported defect: without an API key, Semantic Scholar is unavailable
/// and every other source answers with no rows, so `search article` used to
/// exit 0 with count 0 and no completeness signal. It now reports
/// `complete: false`, error `source_unavailable`, and exits 1.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn zero_rows_with_semantic_scholar_down_exits_one_with_source_unavailable() {
    let (base, requests) = outage_fixture(SemanticScholarMode::Down, PrimaryMode::Empty).await;
    let _env = outage_env(&base).await;
    let outcome = search_article_outcome(&["-g", "ABRAXAS1", "-k", "p.G39V", "--limit", "5"]).await;
    assert_eq!(outcome.exit_code, 1, "stdout:\n{}", outcome.text);
    let value: serde_json::Value = serde_json::from_str(&outcome.text).expect("json reply");
    assert_eq!(value["complete"], serde_json::Value::Bool(false));
    assert_eq!(
        value["error"]["code"],
        serde_json::Value::String("source_unavailable".into())
    );
    assert!(
        value["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("Semantic Scholar")),
        "the error names the down source: {value}"
    );
    assert!(
        reached_provider(&requests, "/graph/v1/paper/search"),
        "the fixture saw no Semantic Scholar request; the test passed vacuously"
    );
}

/// An empty search where every source answered is an ok result: exit 0,
/// `complete: true`, and no error field.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn zero_rows_with_every_source_answered_keeps_exit_zero_complete_true() {
    let (base, _requests) = outage_fixture(SemanticScholarMode::Empty, PrimaryMode::Empty).await;
    let _env = outage_env(&base).await;
    let outcome = search_article_outcome(&["-g", "ABRAXAS1", "-k", "p.G39V", "--limit", "5"]).await;
    assert_eq!(outcome.exit_code, 0, "stdout:\n{}", outcome.text);
    let value: serde_json::Value = serde_json::from_str(&outcome.text).expect("json reply");
    assert_eq!(value["complete"], serde_json::Value::Bool(true));
    assert!(value.get("error").is_none());
}

/// Rows that arrived beside a down source still return: exit 0 with
/// `complete: false` and no structured error.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn rows_with_semantic_scholar_down_keep_exit_zero_complete_false() {
    let (base, _requests) = outage_fixture(SemanticScholarMode::Down, PrimaryMode::EuropeRow).await;
    let _env = outage_env(&base).await;
    let outcome = search_article_outcome(&["-g", "ABRAXAS1", "-k", "p.G39V", "--limit", "5"]).await;
    assert_eq!(outcome.exit_code, 0, "stdout:\n{}", outcome.text);
    let value: serde_json::Value = serde_json::from_str(&outcome.text).expect("json reply");
    assert_eq!(value["complete"], serde_json::Value::Bool(false));
    assert!(value.get("error").is_none());
    assert_eq!(value["count"], serde_json::Value::from(1));
}
