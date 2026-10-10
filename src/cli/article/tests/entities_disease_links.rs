//! Ticket 2047's article disease-row links: the recorded PubTator3 reply
//! for PMID 37887282 and the recorded MyDisease crosswalk replies pin the
//! four live-defect rows, so a MeSH identifier opens the named disease's
//! ontology card or keeps the v0.9.1 search command — never a bare
//! crosswalk get.

use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

macro_rules! recorded {
    ($kind:expr, $name:expr) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/",
            $kind,
            "/",
            $name
        ))
    };
}

/// Ticket 2047: the recorded PubTator3 reply for PMID 37887282 and the
/// recorded MyDisease crosswalk replies for its four MeSH descriptors.
/// The fixture answers only the recorded routes, so the pinned rows cannot
/// pass vacuously.
async fn article_disease_link_fixture_server()
-> (String, Arc<Mutex<Vec<String>>>, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind article disease link fixture");
    let base = format!("http://{}", listener.local_addr().expect("fixture address"));
    let requests = Arc::new(Mutex::new(Vec::new()));
    let captured = requests.clone();
    let task = tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let captured = captured.clone();
            tokio::spawn(async move {
                let mut request = vec![0_u8; 64 * 1024];
                let len = stream
                    .read(&mut request)
                    .await
                    .expect("read fixture request");
                let request = String::from_utf8_lossy(&request[..len]).into_owned();
                captured
                    .lock()
                    .expect("lock fixture requests")
                    .push(request.clone());
                let body = if request.starts_with("GET /publications/export/biocjson?") {
                    recorded!("pubtator", "export_37887282.json")
                } else if request.contains("xrefs.mesh") {
                    // MyDisease crosswalk queries: the MeSH descriptor value
                    // names the recorded reply. URL encoding leaves the
                    // descriptor digits intact in the query string.
                    if request.contains("D008175") {
                        recorded!("mydisease", "query_xref_mesh_d008175.json")
                    } else if request.contains("D000230") {
                        recorded!("mydisease", "query_xref_mesh_d000230.json")
                    } else if request.contains("D000236") {
                        recorded!("mydisease", "query_xref_mesh_d000236.json")
                    } else if request.contains("D009369") {
                        recorded!("mydisease", "query_xref_mesh_d009369.json")
                    } else {
                        r#"{"total":0,"hits":[]}"#
                    }
                } else if request.starts_with("GET /search?") {
                    // Europe PMC metadata lookup: an empty answer keeps the
                    // PubTator document's own fields.
                    r#"{"hitCount":0,"resultList":{"result":[]}}"#
                } else {
                    r#"{"total":0,"hits":[]}"#
                };
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nCache-Control: no-store\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream
                    .write_all(response.as_bytes())
                    .await
                    .expect("write fixture response");
            });
        }
    });
    (base, requests, task)
}

struct ArticleDiseaseLinkEnv(Vec<(&'static str, Option<std::ffi::OsString>)>);

impl ArticleDiseaseLinkEnv {
    fn set(&mut self, key: &'static str, value: &str) {
        self.0.push((key, std::env::var_os(key)));
        // SAFETY: this test holds the serial-test process-wide environment lock.
        unsafe { std::env::set_var(key, value) };
    }
}

impl Drop for ArticleDiseaseLinkEnv {
    fn drop(&mut self) {
        for (key, previous) in self.0.drain(..).rev() {
            // SAFETY: this test holds the serial-test process-wide environment lock.
            unsafe {
                if let Some(value) = previous {
                    std::env::set_var(key, value);
                } else {
                    std::env::remove_var(key);
                }
            }
        }
    }
}

/// The four live-defect rows of `article entities 37887282` (ticket 2047):
/// each MeSH descriptor crosswalks to several MONDO records, and the 0.9.2
/// `get disease MESH:…` link opened the lexicographically first one — lung
/// benign neoplasm for "Lung Cancers", granular cell carcinoma for
/// "adenocarcinomas", papillary adenoma for "adenomas", and disease of
/// cellular proliferation for "cancer". The recorded crosswalk replies hold
/// the descriptor's PubTator3 concept name on exactly one hit apiece, so
/// every verified link names that hit's ontology ID and no link trusts the
/// bare crosswalk identifier.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn article_entities_disease_rows_link_the_named_disease_or_search() {
    let (base, requests, server) = article_disease_link_fixture_server().await;
    let cache = crate::test_support::TempDirGuard::new("article-disease-links-cache");
    let mut env = ArticleDiseaseLinkEnv(Vec::new());
    env.set("BIOMCP_PUBTATOR_BASE", &base);
    env.set("BIOMCP_EUROPEPMC_BASE", &base);
    env.set("BIOMCP_MYDISEASE_BASE", &base);
    env.set("BIOMCP_MYGENE_BASE", &base);
    env.set(
        "BIOMCP_CACHE_DIR",
        cache.path().to_str().expect("utf-8 cache root"),
    );

    let markdown = crate::cli::execute(vec![
        "biomcp".to_string(),
        "article".to_string(),
        "entities".to_string(),
        "37887282".to_string(),
    ])
    .await
    .expect("article entities renders");
    server.abort();
    drop(env);

    // Each descriptor's concept name holds on exactly one recorded hit:
    // "Lung Neoplasms" on lung neoplasm, "Adenocarcinoma" on adenocarcinoma,
    // "Adenoma" on adenoma, "Neoplasms" on neoplasm (final-word plural).
    for expected in [
        "`biomcp get disease MONDO:0021117`",
        "`biomcp get disease MONDO:0004970`",
        "`biomcp get disease MONDO:0004972`",
        "`biomcp get disease MONDO:0005070`",
    ] {
        assert!(
            markdown.contains(expected),
            "missing {expected} in: {markdown}"
        );
    }
    // The wrong cards never appear, and no disease row trusts a bare
    // crosswalk identifier.
    for wrong in [
        "MONDO:0002732",
        "MONDO:0003197",
        "MONDO:0002533",
        "get disease MESH:",
    ] {
        assert!(
            !markdown.contains(wrong),
            "{wrong} must not appear: {markdown}"
        );
    }

    let requests = requests.lock().expect("lock fixture requests").join("\n");
    for descriptor in ["D008175", "D000230", "D000236", "D009369"] {
        assert!(
            requests.contains(descriptor),
            "the crosswalk lookup for {descriptor} must run: {requests}"
        );
    }
}
