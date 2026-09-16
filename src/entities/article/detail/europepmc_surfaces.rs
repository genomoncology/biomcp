use std::path::{Path, PathBuf};

use biomcp_mcp_contract_client::{ContractHarness, call_biomcp_json, first_text};
use rmcp::model::CallToolRequestParams;
use serde_json::{Value, json};

use super::super::test_support::{TestEnv, TestHttpFixture, TestHttpReply, test_http_response};
use super::get_article_base_with_clients;
use crate::error::BioMcpError;
use crate::sources::europepmc::EuropePmcClient;
use crate::sources::pubtator::PubTatorClient;

fn harness() -> ContractHarness {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let binary = std::env::var_os("BIOMCP_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target/debug/biomcp"));
    ContractHarness::new(binary, root)
}

fn europe_response(query: &str, row: Value) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "version": "6.9",
        "hitCount": 1,
        "request": {
            "queryString": query,
            "internalQuery": "fixture",
            "resultType": "LITE",
            "cursorMark": "*",
            "pageSize": 1,
            "sort": "",
            "synonym": false
        },
        "resultList": {"result": [row]}
    }))
    .expect("fixture JSON")
}

async fn fixture() -> TestHttpFixture {
    TestHttpFixture::spawn(|request| {
        let body = if request.starts_with("GET /publications/export/biocjson?") {
            br#"{"PubTator3":[{"id":"7","journal":"PubTator Journal","authors":["Pub Author"],"passages":[{"infons":{"type":"title"},"text":"PubTator title","annotations":[]}]}]}"#.to_vec()
        } else if request.contains("query=DOI%3A10.1%2Fexample") {
            europe_response(
                "DOI:10.1/example",
                json!({
                    "id": "7", "source": "MED", "pmid": "7", "pmcid": "PMC7",
                    "doi": "10.1/example", "title": "Europe title",
                    "authorString": "Europe A, Europe B", "journalTitle": "Europe Journal",
                    "firstPublicationDate": "2024-01-02", "citedByCount": 9,
                    "pubType": "research-article", "isOpenAccess": "Y"
                }),
            )
        } else if request.contains("query=PMCID%3APMC8") {
            europe_response(
                "PMCID:PMC8",
                json!({
                    "id": "8", "source": "PMC", "pmcid": "PMC8",
                    "doi": "10.1/no-pmid", "title": "PMCID without PMID",
                    "authorString": "One A, Two B", "pubYear": "2023"
                }),
            )
        } else if request.contains("query=PMCID%3APMC10") {
            europe_response(
                "PMCID:PMC10",
                json!({"id":"10", "source":"PMC", "pmcid":"PMC10", "title":"   "}),
            )
        } else if request.contains("query=DOI%3A10.1002%2F%28sici%291097-0258") {
            br#"{"hitCount":1,"resultList":{"result":[{"doi":"10.1/neighbor","title":"secret neighbor"}]}}"#.to_vec()
        } else {
            europe_response(
                "DOI:10.1/wrong",
                json!({"id": "9", "source": "MED", "doi": "10.1/neighbor", "title": "secret neighbor"}),
            )
        };
        TestHttpReply::Bytes(test_http_response("200 OK", "application/json", &body))
    })
    .await
}

fn process_env(command: &mut tokio::process::Command, base: &str, cache: &Path) {
    command
        .env("BIOMCP_CACHE_DIR", cache)
        .env("BIOMCP_TEST_UNPACED_ORIGIN", base)
        .env("BIOMCP_PUBTATOR_BASE", base)
        .env("BIOMCP_EUROPEPMC_BASE", base)
        .env("NCBI_API_KEY", "")
        .env("RUST_LOG", "off");
}

async fn run_cli(
    harness: &ContractHarness,
    base: &str,
    cache: &Path,
    args: &[&str],
) -> std::process::Output {
    let mut command = tokio::process::Command::new(&harness.biomcp_bin);
    command.args(args);
    process_env(&mut command, base, cache);
    command.output().await.expect("CLI process")
}

fn json_output(output: &std::process::Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("CLI JSON")
}

fn assert_adopted(value: &Value) {
    assert_eq!(value["pmid"], "7");
    assert_eq!(value["pmcid"], "PMC7");
    assert_eq!(value["doi"], "10.1/example");
    assert_eq!(value["title"], "PubTator title");
    assert_eq!(value["journal"], "PubTator Journal");
    assert_eq!(value["citation_count"], 9);
    assert_eq!(value["open_access"], true);
}

#[serial_test::serial(source_env)]
#[tokio::test]
async fn doi_and_pmcid_use_one_bound_detail_with_hint_reuse_and_no_pmid_path() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    let europe_calls = std::sync::Arc::new(AtomicUsize::new(0));
    let pubtator_calls = std::sync::Arc::new(AtomicUsize::new(0));
    let europe_seen = europe_calls.clone();
    let pubtator_seen = pubtator_calls.clone();
    let fixture = TestHttpFixture::spawn(move |request| {
        let body = if request.starts_with("GET /publications/export/biocjson?") {
            pubtator_seen.fetch_add(1, Ordering::SeqCst);
            if request.contains("pmids=9") {
                br#"{"PubTator3":[{"id":"9","passages":[{"infons":{"type":"title"},"text":"PMCID PubTator","annotations":[]}]}]}"#.to_vec()
            } else {
                br#"{"PubTator3":[{"id":"7","journal":"PubTator Journal","passages":[{"infons":{"type":"title"},"text":"PubTator title","annotations":[]}]}]}"#.to_vec()
            }
        } else if request.contains("query=DOI%3A10.1%2Fexample") {
            europe_seen.fetch_add(1, Ordering::SeqCst);
            europe_response("DOI:10.1/example", json!({
                "id":"7", "source":"MED", "pmid":"7", "pmcid":"PMC7",
                "doi":"10.1/example", "title":"Europe title",
                "journalTitle":"Europe Journal", "firstPublicationDate":"2024-01-02",
                "citedByCount":9, "isOpenAccess":"Y"
            }))
        } else if request.contains("query=DOI%3A10.1%2Fwithout-pmid") {
            europe_seen.fetch_add(1, Ordering::SeqCst);
            europe_response("DOI:10.1/without-pmid", json!({
                "id":"6", "source":"PMC", "pmcid":"PMC6", "doi":"10.1/without-pmid",
                "title":"DOI without PMID"
            }))
        } else if request.contains("query=PMCID%3APMC9") {
            europe_seen.fetch_add(1, Ordering::SeqCst);
            europe_response("PMCID:PMC9", json!({
                "id":"9", "source":"MED", "pmid":"9", "pmcid":"PMC9",
                "title":"Europe PMCID title"
            }))
        } else {
            europe_seen.fetch_add(1, Ordering::SeqCst);
            europe_response("PMCID:PMC8", json!({
                "id":"8", "source":"PMC", "pmcid":"PMC8", "doi":"10.1/no-pmid",
                "title":"No PMID", "authorString":"One A, Two B", "pubYear":"2023",
                "firstIndexDate":"2025-09-19"
            }))
        };
        TestHttpReply::Bytes(test_http_response("200 OK", "application/json", &body))
    })
    .await;
    let cache = crate::test_support::TempDirGuard::new("europepmc-adopted-resolution");
    let mut env = TestEnv::new();
    env.set("BIOMCP_CACHE_DIR", cache.path());
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
    env.set("BIOMCP_PUBTATOR_BASE", &fixture.base);
    env.set("BIOMCP_EUROPEPMC_BASE", &fixture.base);
    env.set("NCBI_API_KEY", "");
    let pubtator = PubTatorClient::new().unwrap();
    let europe = EuropePmcClient::new().unwrap();

    let doi_article = get_article_base_with_clients("10.1/EXAMPLE", &pubtator, &europe)
        .await
        .expect("DOI detail");
    assert_eq!(doi_article.title, "PubTator title");
    assert_eq!(doi_article.journal.as_deref(), Some("PubTator Journal"));
    assert_eq!(doi_article.doi.as_deref(), Some("10.1/example"));
    assert_eq!(doi_article.pmcid.as_deref(), Some("PMC7"));
    assert_eq!(doi_article.citation_count, Some(9));
    assert_eq!(doi_article.open_access, Some(true));

    let pmc_article = get_article_base_with_clients("PMC8", &pubtator, &europe)
        .await
        .expect("PMCID detail without PMID");
    assert_eq!(pmc_article.title, "No PMID");
    assert_eq!(pmc_article.pmid, None);
    assert_eq!(pmc_article.authors, ["One A", "Two B"]);
    assert_eq!(pmc_article.date.as_deref(), Some("2023"));

    let doi_without_pmid = get_article_base_with_clients("10.1/without-pmid", &pubtator, &europe)
        .await
        .expect("DOI detail without PMID");
    assert_eq!(doi_without_pmid.title, "DOI without PMID");
    assert_eq!(doi_without_pmid.pmid, None);

    let pmcid_with_pmid = get_article_base_with_clients("PMC9", &pubtator, &europe)
        .await
        .expect("PMCID detail with PMID");
    assert_eq!(pmcid_with_pmid.title, "PMCID PubTator");
    assert_eq!(pmcid_with_pmid.pmid.as_deref(), Some("9"));
    assert_eq!(europe_calls.load(Ordering::SeqCst), 4);
    assert_eq!(pubtator_calls.load(Ordering::SeqCst), 2);
}

#[serial_test::serial(source_env)]
#[tokio::test]
async fn empty_pmcid_detail_keeps_original_not_found_identity() {
    let fixture = TestHttpFixture::spawn(|_| {
        let body = serde_json::to_vec(&json!({
            "version":"6.9", "hitCount":0,
            "request": {"queryString":"PMCID:PMC8", "internalQuery":"fixture",
                "resultType":"LITE", "cursorMark":"*", "pageSize":1,
                "sort":"", "synonym":false},
            "resultList":{"result":[]}
        }))
        .expect("empty fixture JSON");
        TestHttpReply::Bytes(test_http_response("200 OK", "application/json", &body))
    })
    .await;
    let cache = crate::test_support::TempDirGuard::new("europepmc-empty-pmcid");
    let mut env = TestEnv::new();
    env.set("BIOMCP_CACHE_DIR", cache.path());
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
    env.set("BIOMCP_PUBTATOR_BASE", &fixture.base);
    env.set("BIOMCP_EUROPEPMC_BASE", &fixture.base);
    let error = get_article_base_with_clients(
        "pmcid:pmc8",
        &PubTatorClient::new().unwrap(),
        &EuropePmcClient::new().unwrap(),
    )
    .await
    .expect_err("empty PMCID detail");
    assert!(matches!(
        error,
        BioMcpError::NotFound { id, .. } if id == "pmcid:pmc8"
    ));
}

#[serial_test::serial(source_env)]
#[tokio::test]
async fn invalid_old_doi_heuristic_is_rejected_before_http() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    let calls = std::sync::Arc::new(AtomicUsize::new(0));
    let seen = calls.clone();
    let fixture = TestHttpFixture::spawn(move |_| {
        seen.fetch_add(1, Ordering::SeqCst);
        TestHttpReply::Bytes(test_http_response(
            "500 Internal Server Error",
            "application/json",
            b"{}",
        ))
    })
    .await;
    let mut env = TestEnv::new();
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
    env.set("BIOMCP_PUBTATOR_BASE", &fixture.base);
    env.set("BIOMCP_EUROPEPMC_BASE", &fixture.base);
    let error = get_article_base_with_clients(
        "10./invalid",
        &PubTatorClient::new().unwrap(),
        &EuropePmcClient::new().unwrap(),
    )
    .await
    .expect_err("invalid DOI");
    assert!(matches!(error, BioMcpError::InvalidArgument(_)));
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[serial_test::serial(source_env)]
#[tokio::test]
async fn guarded_legacy_detail_retains_long_abstract_and_license() {
    let payload = serde_json::to_vec(&json!({
        "hitCount": 1,
        "resultList": {"result": [{
            "id": "9", "source": "PMC", "pmcid": "PMC9", "title": "Legacy",
            "abstractText": "A".repeat(1700), "license": " CC BY "
        }]}
    }))
    .unwrap();
    let fixture = TestHttpFixture::spawn(move |_| {
        TestHttpReply::Bytes(test_http_response("200 OK", "application/json", &payload))
    })
    .await;
    let cache = crate::test_support::TempDirGuard::new("europepmc-legacy-detail");
    let mut env = TestEnv::new();
    env.set("BIOMCP_CACHE_DIR", cache.path());
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
    env.set("BIOMCP_PUBTATOR_BASE", &fixture.base);
    env.set("BIOMCP_EUROPEPMC_BASE", &fixture.base);
    let article = get_article_base_with_clients(
        "PMC9",
        &PubTatorClient::new().unwrap(),
        &EuropePmcClient::new().unwrap(),
    )
    .await
    .expect("guarded legacy detail");
    assert!(
        article
            .abstract_text
            .as_deref()
            .unwrap()
            .contains("1700 chars total")
    );
    assert_eq!(article.europepmc_license.as_deref(), Some("CC BY"));
}

#[tokio::test]
async fn actual_cli_json_markdown_and_both_batches_preserve_europepmc_detail() {
    let fixture = fixture().await;
    let cache = tempfile::tempdir().expect("cache");
    let harness = harness();
    let json = run_cli(
        &harness,
        &fixture.base,
        cache.path(),
        &["--json", "get", "article", "10.1/example"],
    )
    .await;
    assert_adopted(&json_output(&json));
    let markdown = run_cli(
        &harness,
        &fixture.base,
        cache.path(),
        &["get", "article", "10.1/example"],
    )
    .await;
    assert!(markdown.status.success());
    assert!(String::from_utf8_lossy(&markdown.stdout).contains("PubTator title"));

    for mode in ["compact", "detail"] {
        let output = run_cli(
            &harness,
            &fixture.base,
            cache.path(),
            &[
                "batch",
                "article",
                "10.1/example,PMC8,PMC10",
                "--mode",
                mode,
                "--json",
            ],
        )
        .await;
        let value = json_output(&output);
        assert_eq!(value["items"][0]["status"], "ok");
        assert_eq!(value["items"][1]["status"], "ok");
        assert_eq!(value["items"][1]["result"]["title"], "PMCID without PMID");
        let blank_title = if mode == "compact" { "PMC10" } else { "" };
        assert_eq!(value["items"][2]["result"]["title"], blank_title);

        let mixed = run_cli(
            &harness,
            &fixture.base,
            cache.path(),
            &[
                "batch",
                "article",
                "PMC8,10.1/wrong,10.1/example",
                "--mode",
                mode,
                "--json",
            ],
        )
        .await;
        assert!(!mixed.status.success());
        let mixed: Value = serde_json::from_slice(&mixed.stdout).expect("mixed batch JSON");
        assert_eq!(mixed["items"][0]["input"], "PMC8");
        assert_eq!(mixed["items"][1]["input"], "10.1/wrong");
        assert_eq!(mixed["items"][2]["input"], "10.1/example");
        assert_eq!(mixed["items"][0]["status"], "ok");
        assert_eq!(mixed["items"][1]["status"], "error");
        assert_eq!(mixed["items"][2]["status"], "ok");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn actual_raw_and_typed_mcp_preserve_adopted_detail_and_sanitize_refusal() {
    let fixture = fixture().await;
    let cache = tempfile::tempdir().expect("cache");
    let harness = harness();
    let env = [
        ("BIOMCP_CACHE_DIR", cache.path().display().to_string()),
        ("BIOMCP_TEST_UNPACED_ORIGIN", fixture.base.clone()),
        ("BIOMCP_PUBTATOR_BASE", fixture.base.clone()),
        ("BIOMCP_EUROPEPMC_BASE", fixture.base.clone()),
        ("NCBI_API_KEY", String::new()),
        ("RUST_LOG", "off".to_string()),
    ];
    let client = harness.spawn_stdio_client(&env).await.expect("MCP client");
    let raw = call_biomcp_json(&client, "biomcp get article 10.1/example")
        .await
        .expect("raw MCP");
    assert_eq!(raw.is_error, Some(false));
    assert_adopted(&serde_json::from_str(first_text(&raw.content)).expect("raw detail"));
    let typed = client
        .peer()
        .call_tool(
            CallToolRequestParams::new("get").with_arguments(
                json!({"entity": "article", "id": "10.1/example", "json": true})
                    .as_object()
                    .expect("typed arguments")
                    .clone(),
            ),
        )
        .await
        .expect("typed MCP");
    assert_eq!(typed.is_error, Some(false));
    assert_adopted(&serde_json::from_str(first_text(&typed.content)).expect("typed detail"));

    let typed_rejected = client
        .peer()
        .call_tool(
            CallToolRequestParams::new("get").with_arguments(
                json!({"entity": "article", "id": "10.1002/(sici)1097-0258", "json": true})
                    .as_object()
                    .expect("typed refusal arguments")
                    .clone(),
            ),
        )
        .await
        .expect("typed refusal");
    let invalid = call_biomcp_json(&client, "biomcp get article 10./invalid")
        .await
        .expect("invalid DOI refusal");
    for rejected in [typed_rejected, invalid] {
        assert_eq!(rejected.is_error, Some(true));
        let text = first_text(&rejected.content);
        assert!(!text.contains("secret neighbor"));
        assert!(!text.contains("identity_mismatch"));
    }
    client.cancel().await.expect("cancel MCP client");
}
