//! Ticket 1199: the citation-evidence sidecar under the managed cache root.
//!
//! Every test runs against the loopback fixture with a fresh `TempDirGuard`
//! cache root. A second call deletes `<cache>/http` so a warm HTTP cache can
//! never explain a suppressed provider request: only the sidecar can.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use super::super::super::test_support::*;
use super::citation_evidence::citation_evidence;
use crate::entities::article::graph::citation_evidence::{
    ArticleCitationEvidenceResult, CitationEvidenceStatus,
};
use crate::sources::semantic_scholar::{SemanticScholarClient, with_test_client};

const CITING_PID: &str = "11223344556677889900aabbccddeeff00112233";
const CITED_PID: &str = "4433221100ffeeddccbbaa998877665544332211";
const CITING_PMID: &str = "41810001";
const CITED_PMID: &str = "41810002";
const CITING_DOI: &str = "10.1000/citing";
const CITED_DOI: &str = "10.1000/cited";

/// The linked JATS document: one paragraph with a marker for the cited DOI.
const JATS_LINKED: &str = "<article><front><article-meta><article-title>T</article-title>\
</article-meta></front><body><sec><title>Results</title><p>Anchor \
<xref ref-type=\"bibr\" rid=\"bib7\">7</xref> text.</p></sec></body>\
<back><ref-list><ref id=\"bib7\"><element-citation>\
<pub-id pub-id-type=\"doi\">10.1/target</pub-id>\
</element-citation></ref></ref-list></back></article>";

#[derive(Clone)]
struct GraphPage {
    offset: u64,
    next: Option<u64>,
    /// (cited paper ID, contexts) per edge.
    edges: Vec<(String, Vec<&'static str>)>,
}

impl GraphPage {
    fn body(&self) -> String {
        let edges: Vec<String> = self
            .edges
            .iter()
            .map(|(pid, contexts)| {
                let contexts = contexts
                    .iter()
                    .map(|value| format!("\"{value}\""))
                    .collect::<Vec<_>>()
                    .join(",");
                format!(
                    "{{\"contexts\":[{contexts}],\"intents\":[],\"citedPaper\":{{\"paperId\":\"{pid}\"}}}}"
                )
            })
            .collect();
        let next = match self.next {
            Some(value) => value.to_string(),
            None => "null".to_string(),
        };
        format!(
            "{{\"offset\":{},\"next\":{},\"data\":[{}]}}",
            self.offset,
            next,
            edges.join(",")
        )
    }
}

fn page(offset: u64, next: Option<u64>, edges: Vec<(String, Vec<&'static str>)>) -> GraphPage {
    GraphPage {
        offset,
        next,
        edges,
    }
}

fn edge(contexts: Vec<&'static str>) -> Vec<(String, Vec<&'static str>)> {
    vec![(CITED_PID.to_string(), contexts)]
}

fn sidecar_records(cache_root: &Path) -> Vec<PathBuf> {
    let mut paths = std::fs::read_dir(cache_root.join("citation-evidence").join("v1"))
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    paths.sort();
    paths
}

fn json(result: &ArticleCitationEvidenceResult) -> String {
    serde_json::to_string(result).expect("serialize citation evidence")
}

/// A fixture that serves seeds, reference pages, and one optional JATS
/// document. The graph cursor cycles through the pages so a repeated
/// traversal answers exactly like the first.
async fn spawn_sidecar_fixture(
    pages: Vec<GraphPage>,
    jats: Option<&'static str>,
) -> (TestHttpFixture, Arc<Mutex<Vec<String>>>) {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let logged = requests.clone();
    let pages = Arc::new(pages);
    let cursor = Arc::new(Mutex::new(0_usize));
    let fixture = TestHttpFixture::spawn(move |request| {
        let mut parts = request.splitn(2, ' ');
        let method = parts.next().unwrap_or_default();
        let target = parts
            .next()
            .unwrap_or_default()
            .split_whitespace()
            .next()
            .unwrap_or_default();
        let reply = if method == "POST" {
            logged.lock().unwrap().push("s2:seed".to_string());
            let citing = request.contains(CITING_PMID)
                || request.contains(CITING_DOI)
                || request.contains(CITING_PID);
            if citing {
                format!(
                    "[{{\"paperId\":\"{CITING_PID}\",\"title\":\"Citing\",\
\"externalIds\":{{\"PubMedCentral\":\"PMC9000001\"}}}}]"
                )
            } else {
                format!(
                    "[{{\"paperId\":\"{CITED_PID}\",\"title\":\"Cited\",\
\"externalIds\":{{\"DOI\":\"10.1/target\"}}}}]"
                )
            }
        } else if target.contains("/fullTextXML") {
            logged.lock().unwrap().push("fulltext:xml".to_string());
            match jats {
                Some(body) => body.to_string(),
                None => {
                    return TestHttpReply::Bytes(test_http_response(
                        "404 Not Found",
                        "text/plain",
                        b"absent",
                    ));
                }
            }
        } else if target.contains("/references") {
            logged.lock().unwrap().push("s2:graph".to_string());
            let index = {
                let mut cursor = cursor.lock().unwrap();
                let index = if pages.is_empty() {
                    0
                } else {
                    *cursor % pages.len()
                };
                *cursor = cursor.wrapping_add(1);
                index
            };
            pages
                .get(index)
                .map(GraphPage::body)
                .unwrap_or_else(|| "{\"offset\":0,\"next\":null,\"data\":[]}".to_string())
        } else if target.contains("idconv") {
            logged.lock().unwrap().push("fulltext:idconv".to_string());
            "{\"records\":[]}".to_string()
        } else {
            "{\"offset\":0,\"next\":null,\"data\":[]}".to_string()
        };
        TestHttpReply::Bytes(test_http_response(
            "200 OK",
            "application/json",
            reply.as_bytes(),
        ))
    })
    .await;
    (fixture, requests)
}

/// Holds the test environment alive for the whole test: the `TestEnv` guard
/// restores every provider variable when the struct drops.
struct FixtureEnv {
    env: TestEnv,
    cache: crate::test_support::TempDirGuard,
    fixture: TestHttpFixture,
    log: Arc<Mutex<Vec<String>>>,
}

impl FixtureEnv {
    async fn new(label: &str, pages: Vec<GraphPage>, jats: Option<&'static str>) -> Self {
        let mut env = TestEnv::new();
        let cache = crate::test_support::TempDirGuard::new(label);
        env.set("BIOMCP_CACHE_DIR", cache.path());
        let (fixture, log) = spawn_sidecar_fixture(pages, jats).await;
        env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
        env.set("BIOMCP_EUROPEPMC_BASE", &fixture.base);
        env.set("BIOMCP_NCBI_IDCONV_BASE", &fixture.base);
        Self {
            env,
            cache,
            fixture,
            log,
        }
    }

    fn set(&mut self, key: &'static str, value: impl AsRef<std::ffi::OsStr>) {
        self.env.set(key, value);
    }

    fn client(&self) -> SemanticScholarClient {
        SemanticScholarClient::new_with_cache_observers(&self.fixture.base, |_, _| {}, |_, _| {})
            .expect("fixture client")
    }

    fn logged(&self) -> String {
        self.log.lock().unwrap().join("\n")
    }

    fn clear_log(&self) {
        self.log.lock().unwrap().clear();
    }

    /// Removes the HTTP cache so only the sidecar can suppress a request.
    fn cold_http_cache(&self) {
        let _ = std::fs::remove_dir_all(self.cache.path().join("http"));
    }
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn repeat_call_serves_the_stored_edge_without_graph_or_fulltext_requests() {
    let fx = FixtureEnv::new(
        "citation-sidecar-repeat",
        vec![page(0, None, edge(vec!["Provider context"]))],
        Some(JATS_LINKED),
    )
    .await;

    let first = with_test_client(
        fx.client(),
        citation_evidence(CITING_PMID, CITED_PMID, false),
    )
    .await
    .expect("first call");
    assert_eq!(first.status, CitationEvidenceStatus::ContextFromProvider);

    fx.cold_http_cache();
    fx.clear_log();

    let second = with_test_client(
        fx.client(),
        citation_evidence(CITING_PMID, CITED_PMID, false),
    )
    .await
    .expect("second call");
    assert_eq!(json(&second), json(&first));

    let logged = fx.logged();
    assert_eq!(logged.matches("s2:seed").count(), 2, "{logged}");
    assert!(!logged.contains("s2:graph"), "{logged}");
    assert!(!logged.contains("fulltext"), "{logged}");
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn cross_spelling_call_is_served_for_the_same_resolved_pair() {
    let fx = FixtureEnv::new(
        "citation-sidecar-spelling",
        vec![page(0, None, edge(vec!["Provider context"]))],
        Some(JATS_LINKED),
    )
    .await;

    let first = with_test_client(
        fx.client(),
        citation_evidence(CITING_PMID, CITED_PMID, false),
    )
    .await
    .expect("pmid spelling");
    fx.cold_http_cache();
    fx.clear_log();

    let second = with_test_client(fx.client(), citation_evidence(CITING_DOI, CITED_DOI, false))
        .await
        .expect("doi spelling");
    assert_eq!(json(&second), json(&first));

    let logged = fx.logged();
    assert_eq!(logged.matches("s2:seed").count(), 2, "{logged}");
    assert!(!logged.contains("s2:graph"), "{logged}");
}

#[path = "cache_modes.rs"]
mod cache_modes;

#[tokio::test]
#[serial_test::serial(source_env)]
async fn a_corrupt_record_is_a_miss_and_is_replaced() {
    let fx = FixtureEnv::new(
        "citation-sidecar-corrupt",
        vec![page(0, None, edge(vec!["Provider context"]))],
        Some(JATS_LINKED),
    )
    .await;

    with_test_client(
        fx.client(),
        citation_evidence(CITING_PMID, CITED_PMID, false),
    )
    .await
    .expect("first call");

    let records = sidecar_records(fx.cache.path());
    assert_eq!(records.len(), 1);
    std::fs::write(&records[0], b"{not json").expect("corrupt the record");

    fx.cold_http_cache();
    fx.clear_log();

    let second = with_test_client(
        fx.client(),
        citation_evidence(CITING_PMID, CITED_PMID, false),
    )
    .await
    .expect("second call");
    assert_eq!(second.status, CitationEvidenceStatus::ContextFromProvider);
    let logged = fx.logged();
    assert!(logged.contains("s2:graph"), "{logged}");
    let replaced = std::fs::read_to_string(&records[0]).expect("read replaced record");
    assert!(replaced.contains("\"schema_version\":1"), "{replaced}");
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn a_future_schema_version_is_a_miss() {
    let fx = FixtureEnv::new(
        "citation-sidecar-schema",
        vec![page(0, None, edge(vec!["Provider context"]))],
        Some(JATS_LINKED),
    )
    .await;

    with_test_client(
        fx.client(),
        citation_evidence(CITING_PMID, CITED_PMID, false),
    )
    .await
    .expect("first call");

    let records = sidecar_records(fx.cache.path());
    assert_eq!(records.len(), 1);
    let mut record: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&records[0]).expect("read record"))
            .expect("parse record");
    record["schema_version"] = serde_json::json!(2);
    std::fs::write(
        &records[0],
        serde_json::to_vec(&record).expect("encode record"),
    )
    .expect("rewrite record");

    fx.cold_http_cache();
    fx.clear_log();

    with_test_client(
        fx.client(),
        citation_evidence(CITING_PMID, CITED_PMID, false),
    )
    .await
    .expect("second call");
    let logged = fx.logged();
    assert!(logged.contains("s2:graph"), "{logged}");
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn a_non_evidence_status_is_never_cached() {
    let fx = FixtureEnv::new(
        "citation-sidecar-unavailable",
        vec![page(0, None, edge(vec![]))],
        None,
    )
    .await;

    let result = with_test_client(
        fx.client(),
        citation_evidence(CITING_PMID, CITED_PMID, false),
    )
    .await
    .expect("fulltext unavailable outcome");
    assert_eq!(result.status, CitationEvidenceStatus::FulltextUnavailable);
    assert!(
        sidecar_records(fx.cache.path()).is_empty(),
        "{:?}",
        sidecar_records(fx.cache.path())
    );
    let logged = fx.logged();
    assert!(logged.contains("fulltext:xml"), "{logged}");
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn a_forced_call_ignores_a_stored_provider_context_entry() {
    let fx = FixtureEnv::new(
        "citation-sidecar-forced-miss",
        vec![page(0, None, edge(vec!["Provider context"]))],
        Some(JATS_LINKED),
    )
    .await;

    let first = with_test_client(
        fx.client(),
        citation_evidence(CITING_PMID, CITED_PMID, false),
    )
    .await
    .expect("first call");
    assert_eq!(first.status, CitationEvidenceStatus::ContextFromProvider);

    fx.cold_http_cache();
    fx.clear_log();

    let forced = with_test_client(
        fx.client(),
        citation_evidence(CITING_PMID, CITED_PMID, true),
    )
    .await
    .expect("forced call");
    assert_eq!(forced.status, CitationEvidenceStatus::ContextFromFulltext);
    let logged = fx.logged();
    assert!(logged.contains("s2:graph"), "{logged}");
    assert!(logged.contains("fulltext:xml"), "{logged}");
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn a_forced_call_serves_a_stored_fulltext_entry() {
    let fx = FixtureEnv::new(
        "citation-sidecar-forced-hit",
        vec![page(0, None, edge(vec![]))],
        Some(JATS_LINKED),
    )
    .await;

    let first = with_test_client(
        fx.client(),
        citation_evidence(CITING_PMID, CITED_PMID, false),
    )
    .await
    .expect("first call");
    assert_eq!(first.status, CitationEvidenceStatus::ContextFromFulltext);

    fx.cold_http_cache();
    fx.clear_log();

    let forced = with_test_client(
        fx.client(),
        citation_evidence(CITING_PMID, CITED_PMID, true),
    )
    .await
    .expect("forced call");
    assert_eq!(json(&forced), json(&first));
    let logged = fx.logged();
    assert_eq!(logged.matches("s2:seed").count(), 2, "{logged}");
    assert!(!logged.contains("s2:graph"), "{logged}");
    assert!(!logged.contains("fulltext"), "{logged}");
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn a_write_failure_leaves_the_outcome_unchanged() {
    let fx = FixtureEnv::new(
        "citation-sidecar-write-failure",
        vec![page(0, None, edge(vec!["Provider context"]))],
        Some(JATS_LINKED),
    )
    .await;
    // A regular file where the sidecar directory belongs fails both the read
    // and the write.
    let blocked = fx.cache.path().join("citation-evidence");
    std::fs::write(&blocked, b"not a directory").expect("block the sidecar directory");

    let result = with_test_client(
        fx.client(),
        citation_evidence(CITING_PMID, CITED_PMID, false),
    )
    .await
    .expect("outcome survives the write failure");
    assert_eq!(result.status, CitationEvidenceStatus::ContextFromProvider);
    assert!(blocked.is_file());
    assert_eq!(
        std::fs::read(&blocked).expect("read the blocking file"),
        b"not a directory"
    );
}
