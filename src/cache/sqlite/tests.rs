//! Contracts for shared storage, integrity and request identity.
use super::*;
use crate::test_support::TempDirGuard;
use sha2::Digest;

#[test]
fn request_identity_includes_body_and_only_declared_vary_headers() {
    let request = |url: &str, body: &str, language: &str| {
        reqwest::Client::new()
            .get(url)
            .body(body.to_owned())
            .header("accept-language", language)
            .build()
            .unwrap()
    };
    let first = request_key::RequestContext::from_request(&request(
        "https://EXAMPLE.test/path?b=2&a=1&api_key=secret",
        "first",
        "en",
    ))
    .unwrap();
    let reordered = request_key::RequestContext::from_request(&request(
        "https://example.test/path?a=1&b=2&api_key=other",
        "first",
        "fr",
    ))
    .unwrap();
    let other_body = request_key::RequestContext::from_request(&request(
        "https://example.test/path?a=1&b=2",
        "second",
        "en",
    ))
    .unwrap();
    assert_eq!(first.base, reordered.base);
    assert_ne!(first.base, other_body.base);
    assert_eq!(first.variant(""), reordered.variant(""));
    assert_ne!(
        first.variant("Accept-Language"),
        reordered.variant("Accept-Language")
    );
    assert!(first.variant("*").is_none());
    assert!(first.variant("bad header name").is_none());
}

#[test]
fn failed_transaction_keeps_rows_accounting_and_shared_blobs_readable() {
    let root = TempDirGuard::new("sqlite-rollback");
    let mut store = Store::open(root.path()).unwrap();
    let body = vec![42; blobs::INLINE_LIMIT + 1];
    store
        .put("http", "one", "one", b"metadata", &body, None)
        .unwrap();
    store
        .put("fulltext", "two", "two", b"", &body, None)
        .unwrap();
    let result: Result<()> = store.transaction(|tx, root| {
        delete_record(tx, root, "http", "one")?;
        delete_record(tx, root, "fulltext", "two")?;
        Err(io::Error::other("rollback"))
    });
    assert!(result.is_err());
    assert_eq!(store.get("http", "one").unwrap().unwrap().body, body);
    store.delete("http", "one").unwrap();
    assert_eq!(store.get("fulltext", "two").unwrap().unwrap().body, body);
    store.delete("fulltext", "two").unwrap();
    assert_eq!(
        store
            .connection()
            .query_row("SELECT bytes FROM cache_meta", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert_eq!(fs::read_dir(root.path().join("blobs")).unwrap().count(), 0);
}

#[cfg(unix)]
#[test]
fn database_and_blob_paths_reject_links_and_keep_private_permissions() {
    use std::os::unix::{fs::PermissionsExt, fs::symlink};
    let root = TempDirGuard::new("sqlite-private");
    let outside = TempDirGuard::new("sqlite-outside");
    let target = outside.path().join("unchanged");
    fs::write(&target, b"sentinel").unwrap();
    symlink(&target, root.path().join(DATABASE_NAME)).unwrap();
    assert!(Store::open(root.path()).is_err());
    assert_eq!(fs::read(&target).unwrap(), b"sentinel");
    fs::remove_file(root.path().join(DATABASE_NAME)).unwrap();
    let mut store = Store::open(root.path()).unwrap();
    store
        .put("http", "one", "one", b"", b"small", None)
        .unwrap();
    for path in [
        root.path().to_path_buf(),
        root.path().join(DATABASE_NAME),
        root.path().join(format!("{DATABASE_NAME}-wal")),
        root.path().join(format!("{DATABASE_NAME}-shm")),
    ] {
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o077,
            0,
            "{}",
            path.display()
        );
    }
    let body = vec![42; blobs::INLINE_LIMIT + 1];
    store
        .put("http", "large", "large", b"", &body, None)
        .unwrap();
    let digest = format!("{:x}", sha2::Sha256::digest(&body));
    let path = root.path().join("blobs").join(digest);
    fs::remove_file(&path).unwrap();
    symlink(&target, &path).unwrap();
    assert!(store.get("http", "large").is_err());
    assert_eq!(fs::read(&target).unwrap(), b"sentinel");
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn http_middleware_separates_bodies_and_vary_and_preserves_original_transport_url() {
    use crate::cache::{CacheConfigOrigins, ConfigOrigin, DiskFreeThreshold, ResolvedCacheConfig};
    use axum::{Router, extract::State, response::IntoResponse, routing::get};
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    };
    type StateData = (Arc<AtomicUsize>, Arc<Mutex<Vec<String>>>);
    async fn reply(
        State(state): State<StateData>,
        uri: http::Uri,
        headers: http::HeaderMap,
        body: axum::body::Bytes,
    ) -> impl IntoResponse {
        state.0.fetch_add(1, Ordering::SeqCst);
        state.1.lock().unwrap().push(uri.to_string());
        let language = headers.get("accept-language").unwrap().to_str().unwrap();
        let representation = headers.get("x-representation").unwrap().to_str().unwrap();
        let mut response = (
            [
                ("cache-control", "public,max-age=3600"),
                ("vary", "Accept-Language"),
            ],
            format!(
                "{language}:{body}:{representation}",
                body = String::from_utf8_lossy(&body)
            ),
        )
            .into_response();
        response.headers_mut().append(
            http::header::VARY,
            http::HeaderValue::from_static("X-Representation"),
        );
        response
    }
    let count = Arc::new(AtomicUsize::new(0));
    let urls = Arc::new(Mutex::new(Vec::new()));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let app = Router::new()
        .route("/resource", get(reply))
        .with_state((count.clone(), urls.clone()));
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let root = TempDirGuard::new("sqlite-request-identity");
    let config = ResolvedCacheConfig {
        cache_root: root.path().to_path_buf(),
        max_size: 10000000,
        min_disk_free: DiskFreeThreshold::Bytes(1),
        max_age: Duration::from_secs(86400),
        origins: CacheConfigOrigins {
            cache_root: ConfigOrigin::Default,
            max_size: ConfigOrigin::Default,
            min_disk_free: ConfigOrigin::Default,
            max_age: ConfigOrigin::Default,
        },
    };
    let manager =
        crate::cache::SizeAwareCacheManager::new(root.path().join(DATABASE_NAME), config).unwrap();
    let cache = http_cache_reqwest::Cache(http_cache_reqwest::HttpCache {
        mode: http_cache_reqwest::CacheMode::Default,
        manager: crate::cache::HttpCacheManager(manager),
        options: http_cache_reqwest::HttpCacheOptions::default(),
    });
    let client = reqwest_middleware::ClientBuilder::new(reqwest::Client::new())
        .with(crate::cache::http::RequestCache(cache))
        .with(crate::cache::http::RestoreCacheUrl)
        .build();
    let original = format!("{base}/resource?b=2&a=1&api_key=FAKE-CANARY-ONE");
    let equivalent = format!("{base}/resource?a=1&api_key=FAKE-CANARY-TWO&b=2");
    for (language, body, url, representation) in [
        ("en", "first", &original, "small"),
        ("fr", "first", &original, "small"),
        ("en", "second", &original, "small"),
        ("en", "first", &equivalent, "small"),
        ("en", "first", &equivalent, "large"),
    ] {
        let response = client
            .get(url)
            .header("accept-language", language)
            .header("x-representation", representation)
            .body(body)
            .send()
            .await
            .unwrap();
        assert_eq!(response.url().as_str(), url);
        assert_eq!(
            response.text().await.unwrap(),
            format!("{language}:{body}:{representation}")
        );
    }
    for _ in 0..2 {
        let response = client
            .get(&original)
            .header("accept-language", "en")
            .header("authorization", "Bearer FAKE-HEADER-CANARY")
            .header("x-representation", "small")
            .body("first")
            .send()
            .await
            .unwrap();
        assert_eq!(response.text().await.unwrap(), "en:first:small");
    }
    assert_eq!(count.load(Ordering::SeqCst), 6);
    assert_eq!(
        urls.lock().unwrap()[0],
        "/resource?b=2&a=1&api_key=FAKE-CANARY-ONE"
    );
    let store = Store::open(root.path()).unwrap();
    let rows: i64 = store
        .connection()
        .query_row("SELECT count(*) FROM entries", [], |r| r.get(0))
        .unwrap();
    assert_eq!(rows, 4);
    let mut statement = store
        .connection()
        .prepare("SELECT metadata FROM entries")
        .unwrap();
    for bytes in statement.query_map([], |r| r.get::<_, Vec<u8>>(0)).unwrap() {
        let text = String::from_utf8(bytes.unwrap()).unwrap();
        assert!(!text.contains("FAKE-CANARY"));
    }
    server.abort();
}
