use super::*;
use sha2::{Digest, Sha256};

pub(super) fn key(sha256: &str) -> String {
    format!("biomcp:article-supplement-text:v1:{sha256}")
}

pub(super) fn read(cache: &Path, key: &str) -> serde_json::Value {
    let db = rusqlite::Connection::open(cache).unwrap();
    let bytes: Vec<u8> = db
        .query_row(
            "SELECT body FROM entries WHERE namespace='http' AND key=?1",
            [key],
            |r| r.get(0),
        )
        .expect("extraction must use managed SQLite");
    serde_json::from_slice(&bytes).unwrap()
}

pub(super) fn write(cache: &Path, key: &str, value: &serde_json::Value) {
    let db = rusqlite::Connection::open(cache).unwrap();
    let body = serde_json::to_vec(value).unwrap();
    db.execute("UPDATE entries SET body=?2,body_hash=?3,bytes=length(metadata)+length(?2) WHERE namespace='http' AND key=?1", rusqlite::params![key,body,format!("{:x}",Sha256::digest(&body))]).unwrap();
}

fn exists(cache: &Path, key: &str) -> bool {
    let db = rusqlite::Connection::open(cache).unwrap();
    db.query_row(
        "SELECT count(*) FROM entries WHERE namespace='http' AND key=?1",
        [key],
        |r| r.get::<_, i64>(0),
    )
    .unwrap()
        > 0
}

#[test]
#[serial_test::serial(article_output_fixture)]
fn supplement_text_obeys_cache_stats_age_and_size_limits() {
    let fixture = ArticleFixture::start();
    let args = [
        "--json",
        "article",
        "supplement-search",
        "30209399",
        "F1761I",
    ];
    let search = successful_output(&fixture, &args);
    let doc: serde_json::Value = serde_json::from_slice(&search.stdout).unwrap();
    let key = key(doc["hits"][0]["sha256"].as_str().unwrap());
    let cache = fixture.root.join("cache/biomcp.db");
    // Unrelated files in the retired extraction directory are never traversed
    // by a keyed hit. Broad cache maintenance owns that directory now.
    #[cfg(unix)]
    let untouched = {
        use std::os::unix::fs::PermissionsExt;
        let legacy = fixture
            .root
            .join("cache/sessions/article-supplement-text-v1");
        fs::create_dir_all(&legacy).unwrap();
        let sentinel = legacy.join("unrelated.json");
        fs::write(&sentinel, b"unrelated").unwrap();
        fs::set_permissions(&sentinel, fs::Permissions::from_mode(0o644)).unwrap();
        sentinel
    };
    successful_output(&fixture, &args);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(untouched).unwrap().permissions().mode() & 0o777,
            0o644,
            "a keyed lookup must not walk the extraction directory"
        );
    }
    let text = read(&cache, &key);
    let stats = successful_output(&fixture, &["--json", "cache", "stats"]);
    let stats: serde_json::Value = serde_json::from_slice(&stats.stdout).unwrap();
    let db = rusqlite::Connection::open(&cache).unwrap();
    let entries: i64 = db
        .query_row(
            "SELECT count(*) FROM entries WHERE namespace='http'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(stats["http_entry_count"], entries);
    assert!(
        stats["blob_bytes"].as_u64().unwrap() >= serde_json::to_vec(&text).unwrap().len() as u64
    );
    db.execute(
        "UPDATE entries SET created=1 WHERE namespace='http' AND key=?1",
        [&key],
    )
    .unwrap();
    drop(db);
    successful_output(&fixture, &["cache", "clean", "--max-age", "1s"]);
    assert!(!exists(&cache, &key), "old extraction must expire");

    successful_output(&fixture, &args);
    assert!(exists(&cache, &key), "search repopulates extraction");
    successful_output(&fixture, &["cache", "clean", "--max-size", "1B"]);
    assert!(!exists(&cache, &key), "extraction must share size eviction");
}
