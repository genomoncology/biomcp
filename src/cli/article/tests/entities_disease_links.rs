//! Article disease-row links: the recorded PubTator3 replies and the
//! recorded MyDisease crosswalk replies pin the four live-defect rows of
//! PMID 37887282 and the narrower-title refusal of PMID 22663011, so a
//! MeSH identifier opens the named disease's ontology card or keeps the
//! v0.9.1 search command — never a bare crosswalk get.

use std::sync::{Arc, Mutex};
use tokio::io::AsyncWriteExt;

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

/// The recorded PubTator3 export and the recorded MyDisease crosswalk
/// replies for its MeSH descriptors. The fixture answers only the
/// recorded routes, so a pinned row cannot pass vacuously.
async fn article_disease_link_fixture_server(
    pubtator_export: &'static str,
) -> (
    String,
    Arc<Mutex<Vec<String>>>,
    crate::test_support::TestHttpServer,
) {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let captured = requests.clone();
    let task = crate::test_support::TestHttpServer::spawn(move |mut stream| {
        let captured = captured.clone();
        async move {
            let Some(request) = crate::test_support::read_request(&mut stream).await? else {
                return Ok(());
            };

            let request = String::from_utf8_lossy(&request).into_owned();
            captured
                .lock()
                .expect("lock fixture requests")
                .push(request.clone());
            let body = if request.starts_with("GET /publications/export/biocjson?") {
                pubtator_export
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
                } else if request.contains("D012878") {
                    recorded!("mydisease", "query_xref_mesh_d012878.json")
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
                .await?;
            Ok(())
        }
    })
    .expect("start HTTP fixture");
    let base = task.base.clone();
    (base, requests, task)
}

type ArticleDiseaseLinkEnv = crate::test_support::FixtureEnv;

/// The four live-defect rows of `article entities 37887282`:
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
    let (base, requests, mut server) =
        article_disease_link_fixture_server(recorded!("pubtator", "export_37887282.json")).await;
    let cache = crate::test_support::TempDirGuard::new("article-disease-links-cache");
    let mut env = ArticleDiseaseLinkEnv::scratch_cache_only();
    env.set("BIOMCP_PUBTATOR_BASE", &base);
    env.set("BIOMCP_EUROPEPMC_BASE", &base);
    env.set("BIOMCP_MYDISEASE_BASE", &base);
    env.set("BIOMCP_MYGENE_BASE", &base);
    env.set_cache_root(cache.path().to_str().expect("utf-8 cache root"));

    let markdown = crate::cli::execute(vec![
        "biomcp".to_string(),
        "article".to_string(),
        "entities".to_string(),
        "37887282".to_string(),
    ])
    .await
    .expect("article entities renders");
    server.finish().unwrap();
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

/// The install test's PMID 22663011 row: "skin neoplasms" (MeSH D012878)
/// crosswalks to two records, and the only one holding the concept name is
/// the benign-neoplasm card, whose title narrows the concept (every
/// concept word plus "benign"). The row keeps its v0.9.1 search command
/// instead of linking a subtype card the row never named.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn article_entities_skin_neoplasms_row_keeps_its_search_command() {
    let (base, requests, mut server) =
        article_disease_link_fixture_server(recorded!("pubtator", "export_22663011.json")).await;
    let cache = crate::test_support::TempDirGuard::new("article-disease-links-cache");
    let mut env = ArticleDiseaseLinkEnv::scratch_cache_only();
    env.set("BIOMCP_PUBTATOR_BASE", &base);
    env.set("BIOMCP_EUROPEPMC_BASE", &base);
    env.set("BIOMCP_MYDISEASE_BASE", &base);
    env.set("BIOMCP_MYGENE_BASE", &base);
    env.set_cache_root(cache.path().to_str().expect("utf-8 cache root"));

    let markdown = crate::cli::execute(vec![
        "biomcp".to_string(),
        "article".to_string(),
        "entities".to_string(),
        "22663011".to_string(),
    ])
    .await
    .expect("article entities renders");
    server.finish().unwrap();
    drop(env);

    // The narrower benign-neoplasm card never links, and no disease row
    // trusts a bare crosswalk identifier.
    for wrong in ["MONDO:0002531", "MONDO:0002898", "get disease MESH:"] {
        assert!(
            !markdown.contains(wrong),
            "{wrong} must not appear: {markdown}"
        );
    }
    // The row keeps its text search for the concept name it carries.
    assert!(
        markdown.contains("biomcp search disease --query \"skin neoplasms\""),
        "the skin neoplasms row keeps its search command: {markdown}"
    );

    let requests = requests.lock().expect("lock fixture requests").join("\n");
    assert!(
        requests.contains("D012878"),
        "the crosswalk lookup for D012878 must run before refusing: {requests}"
    );
}
