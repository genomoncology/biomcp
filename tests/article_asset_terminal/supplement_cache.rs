use super::*;
use std::io::Write;

#[derive(serde::Deserialize, serde::Serialize)]
struct StoredResponse {
    response: http_cache::HttpResponse,
    policy: http_cache_semantics::CachePolicy,
}

pub(super) fn key(sha256: &str) -> String {
    format!("biomcp:article-supplement-text:v1:{sha256}")
}

pub(super) fn read(cache: &Path, key: &str) -> serde_json::Value {
    let bytes = cacache::read_sync(cache, key).expect("extraction must use the managed cache");
    let stored: StoredResponse = bincode::deserialize(&bytes).unwrap();
    serde_json::from_slice(&stored.response.body).unwrap()
}

pub(super) fn write(cache: &Path, key: &str, value: &serde_json::Value) {
    let bytes = cacache::read_sync(cache, key).unwrap();
    let mut stored: StoredResponse = bincode::deserialize(&bytes).unwrap();
    stored.response.body = serde_json::to_vec(value).unwrap();
    cacache::write_sync(cache, key, bincode::serialize(&stored).unwrap()).unwrap();
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
    let cache = fixture.root.join("cache/http");
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
    let text = cacache::read_sync(&cache, &key).expect("extraction must use the managed cache");
    let stats = successful_output(&fixture, &["--json", "cache", "stats"]);
    let stats: serde_json::Value = serde_json::from_slice(&stats.stdout).unwrap();
    let entries = cacache::list_sync(&cache).map(Result::unwrap).count();
    assert_eq!(stats["http_entry_count"], entries);
    assert!(stats["blob_bytes"].as_u64().unwrap() >= text.len() as u64);

    let mut writer = cacache::WriteOpts::new()
        .time(1)
        .open_sync(&cache, &key)
        .unwrap();
    writer.write_all(&text).unwrap();
    writer.commit().unwrap();
    successful_output(&fixture, &["cache", "clean", "--max-age", "1s"]);
    assert!(
        cacache::read_sync(&cache, &key).is_err(),
        "old extraction must expire"
    );

    successful_output(&fixture, &args);
    assert!(
        cacache::read_sync(&cache, &key).is_ok(),
        "search repopulates extraction"
    );
    successful_output(&fixture, &["cache", "clean", "--max-size", "1B"]);
    assert!(
        cacache::read_sync(&cache, &key).is_err(),
        "extraction must share size eviction"
    );
}
