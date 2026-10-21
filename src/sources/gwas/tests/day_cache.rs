//! The GWAS day cache. A repeat of the same query inside one day is
//! answered from BioMCP's cache with no network request — the stored
//! answer holds whatever the provider's headers said when it was
//! fetched — an answer a day old goes back to the provider, and a
//! stored body that fails to decode is replaced by one fresh fetch so
//! the command still succeeds. `CacheMode::NoStore` served this source
//! against stale-body decode failures before the day cache existed;
//! the fresh-fetch replacement covers that class instead.

use super::super::*;
use crate::test_support::FixtureEnv;
use http_cache_reqwest::CacheMode;
use std::sync::{Arc, Mutex};
use tokio::io::AsyncWriteExt;

const V2_SEARCH_BODY: &str = r#"{"_embedded":{"associations":[{"accession_id":"GCST90002321","p_value":5e-8,"mapped_genes":["BRAF"]}]},"page":{"totalElements":1}}"#;

#[derive(Clone)]
struct Reply {
    status: &'static str,
    headers: Vec<(&'static str, String)>,
    body: String,
}

impl Reply {
    fn json(body: &str) -> Self {
        Self {
            status: "200 OK",
            headers: vec![
                ("Content-Type", "application/json".to_string()),
                // The day cache stores the answer whatever the headers
                // say, so the fixture forbids storage outright.
                ("Cache-Control", "no-store".to_string()),
            ],
            body: body.to_string(),
        }
    }

    /// A reply already a day old on arrival: the stored entry's policy
    /// carries the provider's `Age` header, so the next lookup sees the
    /// answer past the day window without waiting for one.
    fn json_a_day_old(body: &str) -> Self {
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

/// A counting GWAS fixture: records every request line and answers with
/// `replies` in order, repeating the last one.
async fn day_cache_server(
    replies: Vec<Reply>,
) -> (
    String,
    Arc<Mutex<Vec<String>>>,
    crate::test_support::TestHttpServer,
) {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let captured = Arc::clone(&requests);
    let replies = Arc::new(replies);
    let served = Arc::new(Mutex::new(0_usize));
    let task = crate::test_support::TestHttpServer::spawn(move |mut stream| {
        let captured = Arc::clone(&captured);
        let replies = Arc::clone(&replies);
        let served = Arc::clone(&served);

        async move {
            let Some(request) = crate::test_support::read_request(&mut stream).await? else {
                return Ok(());
            };

            let request = String::from_utf8_lossy(&request).into_owned();
            captured
                .lock()
                .expect("lock fixture requests")
                .push(request);
            let reply = {
                let mut index = served.lock().expect("lock reply index");
                let reply = replies
                    .get(*index)
                    .unwrap_or(replies.last().expect("at least one reply"))
                    .clone();
                *index += 1;
                reply
            };
            stream.write_all(reply.into_response().as_bytes()).await?;
            Ok(())
        }
    })
    .expect("start HTTP fixture");
    let base = task.base.clone();
    (base, requests, task)
}

/// A GWAS client whose cache and base point at the fixture, with the
/// shared client rebuilt so no earlier test's cache root leaks in.
fn day_cached_client(base: &str, env: &mut FixtureEnv) -> GwasClient {
    env.set("BIOMCP_GWAS_BASE", base);
    crate::sources::reset_shared_http_client_for_tests();
    GwasClient::new().expect("client against fixture")
}

fn request_count(requests: &Arc<Mutex<Vec<String>>>) -> usize {
    requests.lock().expect("lock fixture requests").len()
}

/// Every GWAS request carries the day-cache mode into the cache layer:
/// both plan entry points route through `request_day_cached`, and the
/// extension sits on the middleware builder the cache middleware
/// inspects.
#[test]
#[serial_test::serial(source_env)]
fn gwas_requests_carry_the_day_cache_mode_on_the_middleware_builder() {
    let client = GwasClient {
        client: crate::sources::shared_client("gwas-catalog").expect("shared client"),
        base: std::borrow::Cow::Borrowed("http://gwas.test"),
    };

    let plans = [
        GwasClient::associations_by_rsid_plan("rs36053993", 10).expect("rsid plan"),
        GwasClient::association_search_plan(Some("BRAF"), None, 5).expect("search plan"),
    ];
    for plan in plans {
        let mut builder = client.request_day_cached(&plan);
        assert_eq!(
            builder.extensions().get::<CacheMode>(),
            Some(&CacheMode::IgnoreRules),
            "the GWAS request must carry the day-cache mode into the cache layer"
        );
    }
}

/// The repeat inside a day makes no request: the stored answer serves
/// even though the provider forbade storage, and the page is the same.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn a_repeat_gwas_search_inside_a_day_makes_no_request() {
    let (base, requests, mut server) = day_cache_server(vec![Reply::json(V2_SEARCH_BODY)]).await;
    let mut env = FixtureEnv::scratch_cache_only();
    let client = day_cached_client(&base, &mut env);

    let first = client
        .search_associations(Some("BRAF"), None, 5)
        .await
        .expect("first search");
    let ((second, notes), warnings) =
        crate::test_support::capture_warnings(crate::sources::with_stale_serve_notes(async {
            let second = client
                .search_associations(Some("BRAF"), None, 5)
                .await
                .expect("repeat search");
            (second, crate::sources::take_stale_serve_sentences())
        }))
        .await;
    assert!(
        notes.is_empty(),
        "a day-cache hit is still fresh: {notes:?}"
    );
    assert!(!warnings.contains("served from cache"), "{warnings}");

    server.finish().unwrap();
    drop(env);
    assert_eq!(
        request_count(&requests),
        1,
        "the repeat must serve from the cache"
    );
    assert_eq!(first.total, 1);
    assert_eq!(second.total, first.total);
    assert_eq!(second.associations.len(), 1);
}

/// The stored answer a day old goes back to the provider: the entry's
/// policy already carries a day of age, so the repeat refetches and
/// re-stores instead of serving.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn a_gwas_answer_a_day_old_is_refetched() {
    let (base, requests, mut server) =
        day_cache_server(vec![Reply::json_a_day_old(V2_SEARCH_BODY)]).await;
    let mut env = FixtureEnv::scratch_cache_only();
    let client = day_cached_client(&base, &mut env);

    let first = client
        .search_associations(Some("BRAF"), None, 5)
        .await
        .expect("first search");
    let second = client
        .search_associations(Some("BRAF"), None, 5)
        .await
        .expect("refetched search");

    server.finish().unwrap();
    drop(env);
    assert_eq!(request_count(&requests), 2, "the aged answer must refetch");
    assert_eq!(first.total, 1);
    assert_eq!(second.total, first.total);
}

/// `BIOMCP_CACHE_MODE=infinite` keeps its meaning for the day-cache
/// kinds too: the mode exists to serve expired entries without
/// revalidation, so the day window yields to it. The aged answer is
/// stored first under the default mode (the day-cache store rule holds
/// whatever the provider's headers say), and the infinite repeat makes
/// no request.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn infinite_cache_mode_still_serves_the_aged_gwas_answer() {
    let (base, requests, mut server) =
        day_cache_server(vec![Reply::json_a_day_old(V2_SEARCH_BODY)]).await;
    let mut env = FixtureEnv::scratch_cache_only();
    let client = day_cached_client(&base, &mut env);

    let first = client
        .search_associations(Some("BRAF"), None, 5)
        .await
        .expect("first search");
    let _infinite = crate::sources::test_cache_mode::infinite();
    let ((second, notes), warnings) =
        crate::test_support::capture_warnings(crate::sources::with_stale_serve_notes(async {
            let second = client
                .search_associations(Some("BRAF"), None, 5)
                .await
                .expect("infinite-mode repeat");
            (second, crate::sources::take_stale_serve_sentences())
        }))
        .await;
    assert_eq!(
        notes.len(),
        1,
        "an expired infinite-mode serve keeps its note"
    );
    assert!(warnings.contains("served from cache"), "{warnings}");

    server.finish().unwrap();
    drop(env);
    assert_eq!(
        request_count(&requests),
        1,
        "infinite mode must not refetch"
    );
    assert_eq!(second.total, first.total);
}

/// Cache mode off keeps every request on the network.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn cache_mode_off_never_serves_the_stored_gwas_answer() {
    let (base, requests, mut server) = day_cache_server(vec![Reply::json(V2_SEARCH_BODY)]).await;
    let mut env = FixtureEnv::new();
    let client = day_cached_client(&base, &mut env);

    let first = client
        .search_associations(Some("BRAF"), None, 5)
        .await
        .expect("first search");
    let second = client
        .search_associations(Some("BRAF"), None, 5)
        .await
        .expect("second search");

    server.finish().unwrap();
    drop(env);
    assert_eq!(
        request_count(&requests),
        2,
        "mode off must hit the provider each time"
    );
    assert_eq!(second.total, first.total);
}

/// The `--no-cache` flag bypasses the day cache the same way.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn no_cache_flag_never_serves_the_stored_gwas_answer() {
    let (base, requests, mut server) = day_cache_server(vec![Reply::json(V2_SEARCH_BODY)]).await;
    let mut env = FixtureEnv::scratch_cache_only();
    let client = day_cached_client(&base, &mut env);

    let first =
        crate::sources::with_no_cache(true, client.search_associations(Some("BRAF"), None, 5))
            .await
            .expect("first search");
    let second =
        crate::sources::with_no_cache(true, client.search_associations(Some("BRAF"), None, 5))
            .await
            .expect("second search");

    server.finish().unwrap();
    drop(env);
    assert_eq!(
        request_count(&requests),
        2,
        "--no-cache must hit the provider each time"
    );
    assert_eq!(second.total, first.total);
}

/// The stored body that fails to decode is replaced by one fresh fetch
/// and the command succeeds: the first search stores a good entry, the
/// stored copy is then corrupted in place (the shape an older BioMCP
/// left behind), and the next search serves it, refuses to decode it,
/// and refetches through `CacheMode::NoStore`, which also drops the
/// corrupt copy.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn a_stored_body_that_fails_to_decode_is_replaced_by_a_fresh_fetch() {
    let (base, requests, mut server) = day_cache_server(vec![Reply::json(V2_SEARCH_BODY)]).await;
    let mut env = FixtureEnv::scratch_cache_only();
    let client = day_cached_client(&base, &mut env);

    client
        .search_associations(Some("BRAF"), None, 5)
        .await
        .expect("first search stores a good entry");
    corrupt_the_stored_gwas_body();

    let healed = client
        .search_associations(Some("BRAF"), None, 5)
        .await
        .expect("the corrupt stored body is replaced by a fresh fetch");

    server.finish().unwrap();
    drop(env);
    assert_eq!(
        request_count(&requests),
        2,
        "the cache serve makes no request; exactly the replacement fetch runs"
    );
    assert_eq!(healed.total, 1);
}

/// Rewrite the one stored HTTP-cache entry's body into bytes that
/// cannot decode, leaving the stored policy untouched so the entry
/// still serves. The entry is found by listing the scratch cache, so
/// the test never restates the cache key.
fn corrupt_the_stored_gwas_body() {
    let cache_root = std::env::var("BIOMCP_CACHE_DIR").expect("scratch cache root");
    let connection =
        rusqlite::Connection::open(std::path::Path::new(&cache_root).join("biomcp.db")).unwrap();
    let count: i64 = connection
        .query_row("SELECT count(*) FROM entries", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1, "the first search stored exactly one entry");
    use sha2::Digest;
    connection
        .execute(
            "UPDATE entries SET body=?1,body_hash=?2",
            rusqlite::params![
                b"not json".as_slice(),
                format!("{:x}", sha2::Sha256::digest(b"not json"))
            ],
        )
        .unwrap();
}
