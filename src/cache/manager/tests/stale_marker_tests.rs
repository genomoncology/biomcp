//! Stale-serve marker behavior: get stamps the age when the stored policy
//! is past its freshness window, put strips the marker, and a fresh entry
//! never carries one. Pure manager logic over a real temp cache dir.

use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, SystemTime};

use http::Request;
use http::Response;
use http_cache::{CacheManager, HttpResponse, HttpVersion};
use http_cache_semantics::{CacheOptions, CachePolicy};

use super::super::{STALE_SERVE_AGE_HEADER, SizeAwareCacheManager};
use super::test_config;
use crate::cache::DiskFreeThreshold;
use crate::test_support::TempDirGuard;

fn policy_with_max_age(max_age_secs: u64, response_age: Duration) -> CachePolicy {
    let request = Request::builder()
        .method("GET")
        .uri("https://example.test/cache-key")
        .body(())
        .expect("request");
    let response = Response::builder()
        .status(200)
        .header("cache-control", format!("max-age={max_age_secs}"))
        .body(())
        .expect("response");
    CachePolicy::new_options(
        &request,
        &response,
        SystemTime::now()
            .checked_sub(response_age)
            .expect("response time"),
        CacheOptions::default(),
    )
}

fn http_response() -> HttpResponse {
    HttpResponse {
        body: b"payload".to_vec(),
        headers: HashMap::from([("cache-control".to_string(), "max-age=60".to_string())]),
        status: 200,
        url: reqwest::Url::parse("https://example.test/cache-key").expect("url"),
        version: HttpVersion::Http11,
    }
}

fn manager(cache_root: &Path) -> SizeAwareCacheManager {
    SizeAwareCacheManager::new(
        cache_root.join("http"),
        test_config(cache_root, u64::MAX / 2, DiskFreeThreshold::Percent(1)),
    )
    .expect("new cache manager")
}

#[tokio::test]
async fn a_stale_serve_carries_the_marker_with_its_policy_age() {
    let root = TempDirGuard::new("stale-marker-stamped");
    let manager = manager(root.path());
    // Stored one hour ago with a one-minute window: stale by ~59 minutes.
    let stale = policy_with_max_age(60, Duration::from_secs(3600));
    manager
        .put("key".to_string(), http_response(), stale)
        .await
        .expect("store stale entry");

    let (response, _) = manager.get("key").await.expect("get").expect("stored");
    let age = response
        .headers
        .get(STALE_SERVE_AGE_HEADER)
        .expect("stale serve carries the age marker");
    let age: u64 = age.parse().expect("numeric age");
    assert!(
        (3500..=3600).contains(&age),
        "the age comes from the stored policy clock, not the file: {age}"
    );
}

#[tokio::test]
async fn a_fresh_serve_carries_no_marker() {
    let root = TempDirGuard::new("stale-marker-fresh");
    let manager = manager(root.path());
    let fresh = policy_with_max_age(3600, Duration::from_secs(5));
    manager
        .put("key".to_string(), http_response(), fresh)
        .await
        .expect("store fresh entry");

    let (response, _) = manager.get("key").await.expect("get").expect("stored");
    assert!(
        !response.headers.contains_key(STALE_SERVE_AGE_HEADER),
        "a fresh entry is not a stale serve"
    );
}

#[tokio::test]
async fn put_clears_the_marker_so_revalidation_cannot_carry_it() {
    let root = TempDirGuard::new("stale-marker-cleared");
    let manager = manager(root.path());
    // conditional_fetch hands put the STAMPED cached response after a 304;
    // put must return and store it without the marker.
    let mut stamped = http_response();
    stamped
        .headers
        .insert(STALE_SERVE_AGE_HEADER.to_string(), "3599".to_string());
    let fresh_policy = policy_with_max_age(3600, Duration::from_secs(5));

    let served = manager
        .put("key".to_string(), stamped, fresh_policy)
        .await
        .expect("revalidated put");
    assert!(
        !served.headers.contains_key(STALE_SERVE_AGE_HEADER),
        "a revalidated response never carries the stale marker"
    );

    let (stored, _) = manager.get("key").await.expect("get").expect("stored");
    assert!(
        !stored.headers.contains_key(STALE_SERVE_AGE_HEADER),
        "the stored entry carries no marker either"
    );
}
