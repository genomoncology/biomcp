use biomcp_mcp_contract_client::{
    ContractHarness, article_fulltext_fixture_env, assert_chart_calls,
    assert_explore_core_contract, assert_initialize_and_tools, assert_invalid_resource_error,
    assert_mcp_fulltext_path_redaction, assert_mcp_provenance_calls,
    assert_read_only_and_policy_calls, assert_resource_inventory_and_reads,
    assert_typed_tool_calls, assert_version_call, provision_article_fulltext_fixture,
    provision_study_fixture, start_counting_ols4_stub, start_ols4_stub, study_dir_from_fixture,
    terminate_process,
};
use rmcp::model::CallToolRequestParams;
use serde_json::json;
const MCP_COMPACT_ORDERED_JSON: &str = r###"{
  "items": [
    {
      "input": "22663011",
      "result": {
        "author_completeness": "complete",
        "author_count": 6,
        "author_source": "pubtator",
        "authors": [
          "Ada First",
          "Ben Second",
          "Cyra Middle",
          "Dev Fourth",
          "Eli Fifth",
          "Fay Last"
        ],
        "citation_count": 12,
        "influential_citation_count": 3,
        "journal": "Journal One",
        "pmcid": "PMC123456",
        "pmid": "22663011",
        "requested_id": "22663011",
        "title": "Europe full text winner",
        "tldr": "Fixture compact summary",
        "year": 2025
      },
      "status": "ok"
    },
    {
      "input": "22663012",
      "result": {
        "author_completeness": "unavailable",
        "author_count": 0,
        "author_source": "pubtator",
        "authors": [],
        "journal": "Journal One",
        "pmcid": "PMC123457",
        "pmid": "22663012",
        "requested_id": "22663012",
        "title": "PMC HTML fallback winner",
        "year": 2025
      },
      "status": "ok"
    }
  ],
  "summary": {
    "failed": 0,
    "succeeded": 2,
    "total": 2
  }
}"###;

const MCP_COMPACT_DUPLICATE_JSON: &str = r###"{
  "items": [
    {
      "input": "22663011",
      "result": {
        "author_completeness": "complete",
        "author_count": 6,
        "author_source": "pubtator",
        "authors": [
          "Ada First",
          "Ben Second",
          "Cyra Middle",
          "Dev Fourth",
          "Eli Fifth",
          "Fay Last"
        ],
        "citation_count": 12,
        "influential_citation_count": 3,
        "journal": "Journal One",
        "pmcid": "PMC123456",
        "pmid": "22663011",
        "requested_id": "22663011",
        "title": "Europe full text winner",
        "tldr": "Fixture compact summary",
        "year": 2025
      },
      "status": "ok"
    },
    {
      "input": "22663011",
      "result": {
        "author_completeness": "complete",
        "author_count": 6,
        "author_source": "pubtator",
        "authors": [
          "Ada First",
          "Ben Second",
          "Cyra Middle",
          "Dev Fourth",
          "Eli Fifth",
          "Fay Last"
        ],
        "citation_count": 12,
        "influential_citation_count": 3,
        "journal": "Journal One",
        "pmcid": "PMC123456",
        "pmid": "22663011",
        "requested_id": "22663011",
        "title": "Europe full text winner",
        "tldr": "Fixture compact summary",
        "year": 2025
      },
      "status": "ok"
    }
  ],
  "summary": {
    "failed": 0,
    "succeeded": 2,
    "total": 2
  }
}"###;

const MCP_COMPACT_MIXED_JSON: &str = r###"{
  "items": [
    {
      "input": "22663011",
      "result": {
        "author_completeness": "complete",
        "author_count": 6,
        "author_source": "pubtator",
        "authors": [
          "Ada First",
          "Ben Second",
          "Cyra Middle",
          "Dev Fourth",
          "Eli Fifth",
          "Fay Last"
        ],
        "citation_count": 12,
        "influential_citation_count": 3,
        "journal": "Journal One",
        "pmcid": "PMC123456",
        "pmid": "22663011",
        "requested_id": "22663011",
        "title": "Europe full text winner",
        "tldr": "Fixture compact summary",
        "year": 2025
      },
      "status": "ok"
    },
    {
      "error": {
        "code": "invalid_argument",
        "message": "Invalid argument: Unsupported identifier format. BioMCP resolves PMID (digits only, e.g., 22663011), PMCID (starts with PMC, e.g., PMC9984800), and DOI (starts with 10., e.g., 10.1056/NEJMoa1203421). publisher PIIs (e.g., S1535610826000103) are not indexed by PubMed or Europe PMC and cannot be resolved."
      },
      "input": "not-an-article-id",
      "status": "error"
    },
    {
      "input": "22663011",
      "result": {
        "author_completeness": "complete",
        "author_count": 6,
        "author_source": "pubtator",
        "authors": [
          "Ada First",
          "Ben Second",
          "Cyra Middle",
          "Dev Fourth",
          "Eli Fifth",
          "Fay Last"
        ],
        "citation_count": 12,
        "influential_citation_count": 3,
        "journal": "Journal One",
        "pmcid": "PMC123456",
        "pmid": "22663011",
        "requested_id": "22663011",
        "title": "Europe full text winner",
        "tldr": "Fixture compact summary",
        "year": 2025
      },
      "status": "ok"
    }
  ],
  "summary": {
    "failed": 1,
    "succeeded": 2,
    "total": 3
  }
}"###;

const MCP_S2_SUCCESS_JSON: &str = r###"{
  "items": [
    {
      "input": "22663011",
      "result": {
        "author_completeness": "complete",
        "author_count": 6,
        "author_source": "pubtator",
        "authors": [
          "Ada First",
          "Ben Second",
          "Cyra Middle",
          "Dev Fourth",
          "Eli Fifth",
          "Fay Last"
        ],
        "citation_count": 12,
        "influential_citation_count": 3,
        "journal": "Journal One",
        "pmcid": "PMC123456",
        "pmid": "22663011",
        "requested_id": "22663011",
        "title": "Europe full text winner",
        "tldr": "Fixture compact summary",
        "year": 2025
      },
      "status": "ok"
    }
  ],
  "summary": {
    "failed": 0,
    "succeeded": 1,
    "total": 1
  }
}"###;

const MCP_S2_FAIL_OPEN_JSON: &str = r###"{
  "items": [
    {
      "input": "22663012",
      "result": {
        "author_completeness": "unavailable",
        "author_count": 0,
        "author_source": "pubtator",
        "authors": [],
        "journal": "Journal One",
        "pmcid": "PMC123457",
        "pmid": "22663012",
        "requested_id": "22663012",
        "title": "PMC HTML fallback winner",
        "year": 2025
      },
      "status": "ok"
    }
  ],
  "summary": {
    "failed": 0,
    "succeeded": 1,
    "total": 1
  }
}"###;

const MCP_DETAIL_JSON: &str = r###"{
  "items": [
    {
      "input": "22663011",
      "result": {
        "_meta": {
          "evidence_urls": [
            {
              "label": "PubMed",
              "url": "https://pubmed.ncbi.nlm.nih.gov/22663011/"
            },
            {
              "label": "PMC",
              "url": "https://pmc.ncbi.nlm.nih.gov/articles/PMC123456/"
            }
          ],
          "next_commands": [
            "biomcp article references 22663011 --limit 3",
            "biomcp article citations 22663011 --limit 3",
            "biomcp article recommendations 22663011 --limit 3"
          ],
          "section_sources": [
            {
              "key": "bibliography",
              "label": "Bibliography",
              "outcome": "data",
              "sources": [
                "PubMed",
                "Europe PMC"
              ]
            },
            {
              "key": "authors",
              "label": "Authors",
              "outcome": "data",
              "sources": [
                "PubTator3"
              ]
            },
            {
              "key": "abstract",
              "label": "Abstract",
              "outcome": "data",
              "sources": [
                "PubMed",
                "Europe PMC"
              ]
            },
            {
              "key": "tldr",
              "label": "Semantic Scholar",
              "outcome": "data",
              "sources": [
                "Semantic Scholar"
              ]
            }
          ]
        },
        "abstract_text": "Abstract text.",
        "author_completeness": "complete",
        "author_count": 6,
        "author_source": "pubtator",
        "authors": [
          "Ada First",
          "Ben Second",
          "Cyra Middle",
          "Dev Fourth",
          "Eli Fifth",
          "Fay Last"
        ],
        "date": "2025-01-01",
        "journal": "Journal One",
        "open_access": true,
        "pmcid": "PMC123456",
        "pmid": "22663011",
        "pubtator_fallback": false,
        "section_outcomes": {
          "fulltext": {
            "outcome": "not_requested",
            "sources": []
          },
          "indexing": {
            "outcome": "not_requested",
            "sources": []
          },
          "tldr": {
            "outcome": "data",
            "sources": [
              "Semantic Scholar"
            ]
          }
        },
        "semantic_scholar": {
          "citation_count": 12,
          "influential_citation_count": 3,
          "paper_id": "paper-1",
          "tldr": "Fixture detail summary"
        },
        "title": "Europe full text winner"
      },
      "status": "ok"
    }
  ],
  "summary": {
    "failed": 0,
    "succeeded": 1,
    "total": 1
  }
}"###;

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

struct RequestCounterFixture {
    base: String,
    requests: Arc<AtomicUsize>,
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}

impl RequestCounterFixture {
    fn start() -> anyhow::Result<Self> {
        let listener = TcpListener::bind(("127.0.0.1", 0))?;
        listener.set_nonblocking(true)?;
        let base = format!("http://{}", listener.local_addr()?);
        let requests = Arc::new(AtomicUsize::new(0));
        let stop = Arc::new(AtomicBool::new(false));
        let thread_requests = Arc::clone(&requests);
        let thread_stop = Arc::clone(&stop);
        let thread = thread::spawn(move || {
            while !thread_stop.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        thread_requests.fetch_add(1, Ordering::SeqCst);
                        let mut request = [0_u8; 2048];
                        let _ = stream.read(&mut request);
                        let body = r#"{"total":0,"data":[]}"#;
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                            body.len()
                        );
                        let _ = stream.write_all(response.as_bytes());
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5))
                    }
                    Err(_) => return,
                }
            }
        });
        Ok(Self {
            base,
            requests,
            stop,
            thread: Some(thread),
        })
    }
}

impl Drop for RequestCounterFixture {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            thread.join().expect("join request counter");
        }
    }
}
use std::thread;
use std::time::Duration;

const FAERS_REPORT_PAGE: &str = r#"{
  "meta":{"results":{"skip":0,"limit":1,"total":1}},
  "results":[{
    "safetyreportid":"1001","serious":"1","receivedate":"20250101",
    "seriousnesshospitalization":"1",
    "patient":{"reaction":[{"reactionmeddrapt":"Rash"}],"drug":[
      {"medicinalproduct":"DRUG NAME","drugcharacterization":"1","drugindication":"LUNG CANCER"},
      {"medicinalproduct":"OTHER DRUG","drugcharacterization":"2"}
    ]}
  }]
}"#;

struct OpenFdaFixture {
    base: String,
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}

impl OpenFdaFixture {
    fn start() -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind OpenFDA fixture");
        listener
            .set_nonblocking(true)
            .expect("nonblocking OpenFDA fixture");
        let base = format!("http://{}", listener.local_addr().expect("fixture address"));
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let thread = thread::spawn(move || {
            while !thread_stop.load(Ordering::SeqCst) {
                let Ok((mut stream, _)) = listener.accept() else {
                    thread::sleep(Duration::from_millis(5));
                    continue;
                };
                let mut request = Vec::new();
                let mut chunk = [0_u8; 1024];
                while !request.windows(4).any(|window| window == b"\r\n\r\n") {
                    let Ok(read) = stream.read(&mut chunk) else {
                        break;
                    };
                    if read == 0 {
                        break;
                    }
                    request.extend_from_slice(&chunk[..read]);
                }
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    FAERS_REPORT_PAGE.len(),
                    FAERS_REPORT_PAGE
                );
                let _ = stream.write_all(response.as_bytes());
            }
        });
        Self {
            base,
            stop,
            thread: Some(thread),
        }
    }
}

impl Drop for OpenFdaFixture {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            thread.join().expect("join OpenFDA fixture");
        }
    }
}

fn harness() -> ContractHarness {
    ContractHarness::from_repo_root(env!("CARGO_MANIFEST_DIR"))
}

async fn assert_raw_mcp_global_flags_follow_the_parsed_command<T>(
    client: &rmcp::service::RunningService<rmcp::RoleClient, T>,
) -> anyhow::Result<()>
where
    T: rmcp::Service<rmcp::RoleClient>,
{
    let private_path = "/server/private-variants.json";
    for (command, expected) in [
        (
            format!("biomcp variant --json articles --input {private_path}"),
            "server-local state",
        ),
        (
            "biomcp get --no-cache article 22663011 asset supplement.xlsx".to_string(),
            "Binary article asset downloads are CLI-only",
        ),
    ] {
        let result = biomcp_mcp_contract_client::call_biomcp(client, &command).await?;
        assert_eq!(result.is_error, Some(true), "command={command}");
        let text = biomcp_mcp_contract_client::first_text(&result.content);
        assert!(
            text.contains(expected),
            "command={command}, response={text}"
        );
        assert!(
            !text.contains(private_path),
            "rejection disclosed the server-local path: {text}"
        );
    }

    let allowed =
        biomcp_mcp_contract_client::call_biomcp(client, "biomcp skill --json list").await?;
    assert_eq!(allowed.is_error, Some(false));

    let after_rejection = biomcp_mcp_contract_client::call_biomcp(client, "biomcp version").await?;
    assert_eq!(after_rejection.is_error, Some(false));
    Ok(())
}

async fn assert_raw_article_batch_contract<T>(
    client: &rmcp::service::RunningService<rmcp::RoleClient, T>,
    fixture: &biomcp_mcp_contract_client::ArticleFulltextFixture,
) -> anyhow::Result<()>
where
    T: rmcp::Service<rmcp::RoleClient>,
{
    const COMPACT_MARKDOWN: &str = "# Batch: article (2)\n\n---\n\n## 22663011 — ok\n\n# Article Batch (1)\n\n## 1. Europe full text winner\nPMID: 22663011\nJournal: Journal One\nYear: 2025\nAuthors: Ada First, Ben Second, Cyra Middle, Dev Fourth, Eli Fifth, Fay Last\nAuthorship: complete (6 returned; PubTator3)\nTLDR: Fixture compact summary\nCitations: 12 (influential: 3)\n\n\n---\n\n## 22663012 — ok\n\n# Article Batch (1)\n\n## 1. PMC HTML fallback winner\nPMID: 22663012\nJournal: Journal One\nYear: 2025\nAuthorship: unavailable (no author list supplied by PubTator3)\n\n\n## Summary\n\nTotal: 2; succeeded: 2; failed: 0.\n";
    const DETAIL_MARKDOWN: &str = "# Batch: article (1)\n\n---\n\n## 22663011 — ok\n\n# Europe full text winner\n\nPMID: 22663011\nPMCID: PMC123456\n\nJournal: Journal One\nDate: 2025-01-01\n\n\nOpen Access: Yes\n[PubMed](https://pubmed.ncbi.nlm.nih.gov/22663011/)\nSource: PubMed / Europe PMC\n\n## Authors (PubTator3)\n\nAda First, Ben Second, Cyra Middle, Dev Fourth, Eli Fifth, Fay Last\nAuthorship: complete (6 returned; PubTator3)\n## Abstract (PubMed / Europe PMC)\n\nAbstract text.\n## Semantic Scholar\n\nPaper ID: paper-1\nTLDR: Fixture detail summary\nCitations: 12\nInfluential citations: 3\nReferences: \nOpen access: No\nMore:\n  biomcp get article 22663011 annotations   - PubTator normalized entity mentions\n  biomcp get article 22663011 fulltext   - cached full text when available\n  biomcp get article 22663011 tldr   - Semantic Scholar summary and influence\nAll:\n  biomcp get article 22663011 all\nSee also:\n  biomcp article references 22663011 --limit 3   - background evidence this paper builds on; use if the primary paper lacks context\n  biomcp article citations 22663011 --limit 3   - later papers that cite this article; use only if the primary paper lacks your answer\n  biomcp article recommendations 22663011 --limit 3   - related papers to broaden coverage; use only if the primary paper lacks your answer\n\n[PubMed](https://pubmed.ncbi.nlm.nih.gov/22663011/) | [PMC](https://pmc.ncbi.nlm.nih.gov/articles/PMC123456/)\n\n## Summary\n\nTotal: 1; succeeded: 1; failed: 0.\n";
    assert_initialize_and_tools(client, env!("CARGO_MANIFEST_DIR")).await?;
    let tools = client.peer().list_tools(Default::default()).await?;
    assert_eq!(tools.tools.len(), 7);
    let schemas = serde_json::to_value(&tools.tools)?;
    fn has_property(value: &serde_json::Value, needle: &str) -> bool {
        match value {
            serde_json::Value::Object(map) => {
                map.get("properties")
                    .and_then(serde_json::Value::as_object)
                    .is_some_and(|properties| properties.contains_key(needle))
                    || map.values().any(|child| has_property(child, needle))
            }
            serde_json::Value::Array(values) => {
                values.iter().any(|child| has_property(child, needle))
            }
            _ => false,
        }
    }
    for forbidden in ["ids", "mode"] {
        assert!(
            !has_property(&schemas, forbidden),
            "typed MCP schemas exposed batch field {forbidden}: {schemas}"
        );
    }

    let compact_expected = json!({
        "items": [
            {"input":"22663011","status":"ok","result":{
                "requested_id":"22663011","pmid":"22663011","pmcid":"PMC123456",
                "title":"Europe full text winner","authors":["Ada First","Ben Second","Cyra Middle","Dev Fourth","Eli Fifth","Fay Last"],
                "author_count":6,"author_completeness":"complete","author_source":"pubtator",
                "journal":"Journal One","year":2025,"tldr":"Fixture compact summary",
                "citation_count":12,"influential_citation_count":3
            }},
            {"input":"22663012","status":"ok","result":{
                "requested_id":"22663012","pmid":"22663012","pmcid":"PMC123457",
                "title":"PMC HTML fallback winner","authors":[],"author_count":0,
                "author_completeness":"unavailable","author_source":"pubtator",
                "journal":"Journal One","year":2025
            }}
        ],
        "summary":{"total":2,"succeeded":2,"failed":0}
    });
    let detail_expected = json!({
        "items":[{"input":"22663011","status":"ok","result":{
            "pmid":"22663011","pmcid":"PMC123456","title":"Europe full text winner",
            "authors":["Ada First","Ben Second","Cyra Middle","Dev Fourth","Eli Fifth","Fay Last"],
            "author_count":6,"author_completeness":"complete","author_source":"pubtator",
            "journal":"Journal One","date":"2025-01-01","abstract_text":"Abstract text.",
            "open_access":true,"pubtator_fallback":false,
            "semantic_scholar":{"paper_id":"paper-1","tldr":"Fixture detail summary","citation_count":12,"influential_citation_count":3},
            "section_outcomes":{"fulltext":{"outcome":"not_requested","sources":[]},"indexing":{"outcome":"not_requested","sources":[]},"tldr":{"outcome":"data","sources":["Semantic Scholar"]}},
            "_meta":{
                "evidence_urls":[{"label":"PubMed","url":"https://pubmed.ncbi.nlm.nih.gov/22663011/"},{"label":"PMC","url":"https://pmc.ncbi.nlm.nih.gov/articles/PMC123456/"}],
                "next_commands":["biomcp article references 22663011 --limit 3","biomcp article citations 22663011 --limit 3","biomcp article recommendations 22663011 --limit 3"],
                "section_sources":[
                    {"key":"bibliography","label":"Bibliography","outcome":"data","sources":["PubMed","Europe PMC"]},
                    {"key":"authors","label":"Authors","outcome":"data","sources":["PubTator3"]},
                    {"key":"abstract","label":"Abstract","outcome":"data","sources":["PubMed","Europe PMC"]},
                    {"key":"tldr","label":"Semantic Scholar","outcome":"data","sources":["Semantic Scholar"]}
                ]
            }
        }}],"summary":{"total":1,"succeeded":1,"failed":0}
    });

    for json in [false, true] {
        let compatibility = if json {
            biomcp_mcp_contract_client::call_biomcp_json(
                client,
                "biomcp article batch 22663011 22663012",
            )
            .await?
        } else {
            biomcp_mcp_contract_client::call_biomcp(
                client,
                "biomcp article batch 22663011 22663012",
            )
            .await?
        };
        let canonical = if json {
            biomcp_mcp_contract_client::call_biomcp_json(
                client,
                "biomcp batch article 22663011,22663012 --mode compact",
            )
            .await?
        } else {
            biomcp_mcp_contract_client::call_biomcp(
                client,
                "biomcp batch article 22663011,22663012 --mode compact",
            )
            .await?
        };
        // Settled item errors are data over raw MCP, not tool-call errors.
        assert_eq!(compatibility.is_error, Some(false));
        assert_eq!(canonical.is_error, Some(false));
        assert_eq!(compatibility.structured_content, None);
        assert_eq!(canonical.structured_content, None);
        let text = biomcp_mcp_contract_client::first_text(&canonical.content);
        let compatibility_text = biomcp_mcp_contract_client::first_text(&compatibility.content);
        if json {
            assert_eq!(compatibility_text, MCP_COMPACT_ORDERED_JSON);
            assert_eq!(text, MCP_COMPACT_ORDERED_JSON);
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(compatibility_text)?,
                compact_expected
            );
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(text)?,
                compact_expected
            );
        } else {
            assert_eq!(compatibility_text, COMPACT_MARKDOWN);
            assert_eq!(text, COMPACT_MARKDOWN);
        }

        let detail_command = "biomcp batch article 22663011";
        let explicit_command = "biomcp batch article 22663011 --mode detail";
        let detail = if json {
            biomcp_mcp_contract_client::call_biomcp_json(client, detail_command).await?
        } else {
            biomcp_mcp_contract_client::call_biomcp(client, detail_command).await?
        };
        let explicit_detail = if json {
            biomcp_mcp_contract_client::call_biomcp_json(client, explicit_command).await?
        } else {
            biomcp_mcp_contract_client::call_biomcp(client, explicit_command).await?
        };
        assert_eq!(detail.is_error, Some(false));
        assert_eq!(detail.structured_content, None);
        assert_eq!(explicit_detail.structured_content, None);
        for result in [&detail, &explicit_detail] {
            let rendered = biomcp_mcp_contract_client::first_text(&result.content);
            if json {
                assert_eq!(rendered, MCP_DETAIL_JSON);
                assert_eq!(
                    serde_json::from_str::<serde_json::Value>(rendered)?,
                    detail_expected
                );
            } else {
                assert_eq!(rendered, DETAIL_MARKDOWN);
            }
        }
    }

    let mixed = biomcp_mcp_contract_client::call_biomcp_json(
        client,
        "biomcp batch article 22663011,not-an-article-id,22663011 --mode compact",
    )
    .await?;
    assert_eq!(mixed.is_error, Some(false));
    assert_eq!(mixed.structured_content, None);
    let mixed_text = biomcp_mcp_contract_client::first_text(&mixed.content);
    assert_eq!(mixed_text, MCP_COMPACT_MIXED_JSON);
    let mixed: serde_json::Value = serde_json::from_str(mixed_text)?;
    assert_eq!(
        mixed["summary"],
        json!({"total":3,"succeeded":2,"failed":1})
    );
    assert_eq!(
        mixed["items"][0],
        json!({"input":"22663011","status":"ok","result":compact_expected["items"][0]["result"]})
    );
    assert_eq!(mixed["items"][1]["input"], "not-an-article-id");
    assert_eq!(
        mixed["items"][1],
        json!({
            "input":"not-an-article-id","status":"error","error":{
                "code":"invalid_argument",
                "message":"Invalid argument: Unsupported identifier format. BioMCP resolves PMID (digits only, e.g., 22663011), PMCID (starts with PMC, e.g., PMC9984800), and DOI (starts with 10., e.g., 10.1056/NEJMoa1203421). publisher PIIs (e.g., S1535610826000103) are not indexed by PubMed or Europe PMC and cannot be resolved."
            }
        })
    );
    assert_eq!(
        mixed["items"][2],
        json!({"input":"22663011","status":"ok","result":compact_expected["items"][0]["result"]})
    );

    for (command, literal, expected_items) in [
        (
            "biomcp batch article 22663011,22663011 --mode compact",
            MCP_COMPACT_DUPLICATE_JSON,
            json!([compact_expected["items"][0], compact_expected["items"][0]]),
        ),
        (
            "biomcp batch article 22663011 --mode compact",
            MCP_S2_SUCCESS_JSON,
            json!([compact_expected["items"][0]]),
        ),
        (
            "biomcp batch article 22663012 --mode compact",
            MCP_S2_FAIL_OPEN_JSON,
            json!([compact_expected["items"][1]]),
        ),
    ] {
        let result = biomcp_mcp_contract_client::call_biomcp_json(client, command).await?;
        assert_eq!(result.is_error, Some(false));
        assert_eq!(result.structured_content, None);
        let rendered = biomcp_mcp_contract_client::first_text(&result.content);
        assert_eq!(rendered, literal);
        let parsed: serde_json::Value = serde_json::from_str(rendered)?;
        assert_eq!(parsed["items"], expected_items);
    }

    for forbidden in [
        fixture.base_url.as_str(),
        fixture.cache_dir.to_string_lossy().as_ref(),
        "signed.example.invalid",
        "token=secret",
    ] {
        assert!(
            !mixed.to_string().contains(forbidden),
            "raw batch leaked fixture-private text: {forbidden}"
        );
    }

    for (command, expected) in [
        (
            "biomcp batch article 1 --mode compact --sections tldr",
            "Error: Invalid argument: --sections is not supported for compact article batches",
        ),
        (
            "biomcp batch article 1 --mode compact --offset 1",
            "Error: BioMCP allows read-only commands only. Allowed families are search/get/helpers/list/version/health/batch/enrich/discover/skill plus MCP-safe study commands (`study list`, `study download --list`, `study top-mutated`, `study query`, `study filter`, `study cohort`, `study survival`, `study compare`, `study co-occurrence`).",
        ),
        (
            "biomcp batch article 1 --sections unknown",
            "Error: Invalid argument: Unknown section \"unknown\" for article. Available: annotations, fulltext, tldr, indexing, assets, asset, all",
        ),
        (
            "biomcp batch article 1 --sections ''",
            "Error: Invalid argument: Article batch sections must be a comma-separated list of nonempty section names",
        ),
        (
            "biomcp batch gene BRAF --mode detail",
            "Error: Invalid argument: --mode is only supported for article batches",
        ),
    ] {
        let result = biomcp_mcp_contract_client::call_biomcp(client, command).await?;
        assert_eq!(result.is_error, Some(true), "command={command}");
        assert_eq!(result.structured_content, None);
        assert_eq!(
            biomcp_mcp_contract_client::first_text(&result.content),
            expected
        );
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn raw_mcp_global_flags_are_safe_over_stdio() -> anyhow::Result<()> {
    let harness = harness();
    let client = harness.spawn_stdio_client(&[]).await?;
    assert_raw_mcp_global_flags_follow_the_parsed_command(&client).await?;
    client.cancel().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn raw_article_batch_contract_is_safe_over_stdio() -> anyhow::Result<()> {
    let harness = harness();
    let fixture = provision_article_fulltext_fixture(&harness.repo_root)?;
    let env = article_fulltext_fixture_env(&fixture);
    let client = harness.spawn_stdio_client(&env).await?;
    assert_raw_article_batch_contract(&client, &fixture).await?;
    client.cancel().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn raw_mcp_global_flags_are_safe_over_http() -> anyhow::Result<()> {
    let harness = harness();
    let (mut child, base_url) = harness.spawn_http_server(&[]).await?;
    let result = async {
        let client = harness.http_client(format!("{base_url}/mcp")).await?;
        assert_raw_mcp_global_flags_follow_the_parsed_command(&client).await?;
        client.cancel().await?;
        Ok::<(), anyhow::Error>(())
    }
    .await;

    child.kill().await.ok();
    result
}

#[tokio::test(flavor = "multi_thread")]
async fn raw_article_batch_contract_is_safe_over_http() -> anyhow::Result<()> {
    let harness = harness();
    let fixture = provision_article_fulltext_fixture(&harness.repo_root)?;
    let env = article_fulltext_fixture_env(&fixture);
    let (mut child, base_url) = harness.spawn_http_server(&env).await?;
    let result = async {
        let client = harness.http_client(format!("{base_url}/mcp")).await?;
        assert_raw_article_batch_contract(&client, &fixture).await?;
        client.cancel().await?;
        Ok::<(), anyhow::Error>(())
    }
    .await;
    child.kill().await.ok();
    result
}

#[tokio::test(flavor = "multi_thread")]
async fn human_mcp_command_dispatches_to_provider_once() -> anyhow::Result<()> {
    let harness = harness();
    let (_thread, ols_url, requests) = start_counting_ols4_stub()?;
    let (_medline_thread, medline_url) = start_ols4_stub()?;
    let client = harness
        .spawn_stdio_client(&[
            ("BIOMCP_OLS4_BASE", ols_url),
            ("BIOMCP_MEDLINEPLUS_BASE", medline_url),
        ])
        .await?;

    let result = biomcp_mcp_contract_client::call_biomcp(&client, "biomcp discover BRCA1").await?;
    assert_eq!(result.is_error, Some(false));
    assert_eq!(requests.load(Ordering::SeqCst), 1);

    client.cancel().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn raw_and_typed_article_query_validation_converges_before_provider_work()
-> anyhow::Result<()> {
    let harness = harness();
    let fixture = RequestCounterFixture::start()?;
    let mut env = Vec::new();
    for name in [
        "BIOMCP_PUBTATOR_BASE",
        "BIOMCP_EUROPEPMC_BASE",
        "BIOMCP_PUBMED_BASE",
        "BIOMCP_S2_BASE",
        "BIOMCP_LITSENSE2_BASE",
        "BIOMCP_MYGENE_BASE",
        "BIOMCP_MYVARIANT_BASE",
        "BIOMCP_MYCHEM_BASE",
        "BIOMCP_CTGOV_BASE",
        "BIOMCP_REACTOME_BASE",
        "BIOMCP_KEGG_BASE",
        "BIOMCP_WIKIPATHWAYS_BASE",
        "BIOMCP_CPIC_BASE",
    ] {
        env.push((name, fixture.base.clone()));
    }
    env.push(("BIOMCP_TEST_UNPACED_ORIGIN", fixture.base.clone()));
    let client = harness.spawn_stdio_client(&env).await?;
    const GENE: &str = "Error: Invalid argument: keyword is provider-neutral and does not accept gene: filter syntax. Use --gene RB1 for CLI or raw MCP, or the typed MCP field, for example \"gene\":\"RB1\".";
    const DISEASE: &str = "Error: Invalid argument: keyword is provider-neutral and does not accept disease: filter syntax. Use --disease melanoma for CLI or raw MCP, or the typed MCP field, for example \"disease\":\"melanoma\".";
    const DRUG: &str = "Error: Invalid argument: keyword is provider-neutral and does not accept drug: filter syntax. Use --drug vemurafenib for CLI or raw MCP, or the typed MCP field, for example \"drug\":\"vemurafenib\".";
    const SYMBOL: &str = "Error: Invalid argument: gene accepts one symbol, for example TPMT. Put additional concepts in keyword: use --gene TPMT --keyword mercaptopurine for CLI or raw MCP, or typed MCP fields \"gene\":\"TPMT\" and \"keyword\":[\"mercaptopurine\"].";
    let keyword_cases = [
        ("gene:RB1", GENE),
        ("disease:melanoma", DISEASE),
        ("drug:vemurafenib", DRUG),
    ];

    for (keyword, expected) in keyword_cases {
        for command in [
            format!("biomcp search article -k {keyword}"),
            format!("biomcp search article -q {keyword}"),
            format!("biomcp search article --query {keyword}"),
            format!("biomcp search article {keyword}"),
            format!("biomcp search all --keyword {keyword}"),
        ] {
            let result = biomcp_mcp_contract_client::call_biomcp(&client, &command).await?;
            assert_eq!(result.is_error, Some(true), "{command}");
            assert_eq!(result.content.len(), 1, "{command}");
            assert_eq!(
                biomcp_mcp_contract_client::first_text(&result.content),
                expected,
                "{command}"
            );
        }
    }

    for command in [
        "biomcp search article --gene 'TPMT mercaptopurine'",
        "biomcp search all --gene 'TPMT mercaptopurine'",
    ] {
        let result = biomcp_mcp_contract_client::call_biomcp(&client, command).await?;
        assert_eq!(result.is_error, Some(true), "{command}");
        assert_eq!(result.content.len(), 1, "{command}");
        assert_eq!(
            biomcp_mcp_contract_client::first_text(&result.content),
            SYMBOL,
            "{command}"
        );
    }

    for (arguments, expected) in [
        (
            BTreeMap::from([
                ("entity".to_string(), json!("article")),
                ("keyword".to_string(), json!(["gene:RB1"])),
            ]),
            GENE,
        ),
        (
            BTreeMap::from([
                ("entity".to_string(), json!("article")),
                ("keyword".to_string(), json!(["disease:melanoma"])),
            ]),
            DISEASE,
        ),
        (
            BTreeMap::from([
                ("entity".to_string(), json!("article")),
                ("keyword".to_string(), json!(["drug:vemurafenib"])),
            ]),
            DRUG,
        ),
        (
            BTreeMap::from([
                ("entity".to_string(), json!("article")),
                ("gene".to_string(), json!("TPMT mercaptopurine")),
            ]),
            SYMBOL,
        ),
    ] {
        let result = client
            .peer()
            .call_tool(
                CallToolRequestParams::new("search")
                    .with_arguments(arguments.into_iter().collect()),
            )
            .await?;
        assert_eq!(result.is_error, Some(true));
        assert_eq!(result.content.len(), 1);
        assert_eq!(
            biomcp_mcp_contract_client::first_text(&result.content),
            expected
        );
    }

    let harmless = biomcp_mcp_contract_client::call_biomcp(&client, "biomcp version").await?;
    assert_eq!(harmless.is_error, Some(false));
    thread::sleep(Duration::from_millis(20));
    assert_eq!(fixture.requests.load(Ordering::SeqCst), 0);

    let quoted = client
        .peer()
        .call_tool(
            CallToolRequestParams::new("search").with_arguments(
                BTreeMap::from([
                    ("entity".to_string(), json!("article")),
                    ("keyword".to_string(), json!(["\"gene:gene interaction\""])),
                    ("source".to_string(), json!("semanticscholar")),
                ])
                .into_iter()
                .collect(),
            ),
        )
        .await?;
    let quoted_text = biomcp_mcp_contract_client::first_text(&quoted.content);
    assert!(
        !quoted_text.contains("does not accept gene:"),
        "{quoted_text}"
    );
    thread::sleep(Duration::from_millis(20));
    assert!(
        fixture.requests.load(Ordering::SeqCst) > 0,
        "quoted literal should reach its provider"
    );
    client.cancel().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn raw_and_typed_mcp_reject_unknown_adverse_event_sections_before_provider_work()
-> anyhow::Result<()> {
    let harness = harness();
    let client = harness
        .spawn_stdio_client(&[("BIOMCP_OPENFDA_BASE", "http://127.0.0.1:9".to_string())])
        .await?;

    let raw = biomcp_mcp_contract_client::call_biomcp(
        &client,
        "biomcp get adverse-event 1001 not-a-section",
    )
    .await?;
    assert_eq!(raw.is_error, Some(true));
    assert!(biomcp_mcp_contract_client::first_text(&raw.content).contains("Unknown section"));

    let typed = client
        .peer()
        .call_tool(
            CallToolRequestParams::new("get").with_arguments(
                BTreeMap::from([
                    ("entity".to_string(), json!("adverse-event")),
                    ("id".to_string(), json!("1001")),
                    ("sections".to_string(), json!(["not-a-section"])),
                ])
                .into_iter()
                .collect(),
            ),
        )
        .await
        .expect_err("typed MCP rejects an unknown section at its schema boundary");
    assert!(typed.to_string().contains("invalid adverse-event section"));

    client.cancel().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn raw_mcp_rejects_contradictory_variant_filters_and_non_trial_batch_source()
-> anyhow::Result<()> {
    let harness = harness();
    let client = harness
        .spawn_stdio_client(&[
            ("BIOMCP_MYVARIANT_BASE", "http://127.0.0.1:9".to_string()),
            ("BIOMCP_MYGENE_BASE", "http://127.0.0.1:9".to_string()),
        ])
        .await?;

    let variant = biomcp_mcp_contract_client::call_biomcp(
        &client,
        "biomcp search variant --min-cadd 10 --missing cadd",
    )
    .await?;
    assert_eq!(variant.is_error, Some(true));
    assert!(
        biomcp_mcp_contract_client::first_text(&variant.content)
            .contains("cannot be combined with --missing")
    );

    let batch =
        biomcp_mcp_contract_client::call_biomcp(&client, "biomcp batch gene BRAF --source ctgov")
            .await?;
    assert_eq!(batch.is_error, Some(true));
    assert!(
        biomcp_mcp_contract_client::first_text(&batch.content)
            .contains("--source is only supported for trial batches")
    );

    client.cancel().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn raw_and_typed_mcp_preserve_adverse_event_subset_and_full_json_contracts()
-> anyhow::Result<()> {
    let harness = harness();
    let fixture = OpenFdaFixture::start();
    let cache = tempfile::tempdir()?;
    let client = harness
        .spawn_stdio_client(&[
            ("BIOMCP_OPENFDA_BASE", fixture.base.clone()),
            (
                "BIOMCP_CACHE_DIR",
                cache.path().to_string_lossy().into_owned(),
            ),
        ])
        .await?;

    let raw_subset = biomcp_mcp_contract_client::call_biomcp_json(
        &client,
        "biomcp get adverse-event 1001 reactions reactions guidance guidance",
    )
    .await?;
    assert_eq!(raw_subset.is_error, Some(false));
    let raw_subset: serde_json::Value =
        serde_json::from_str(biomcp_mcp_contract_client::first_text(&raw_subset.content))?;

    let typed_subset = client
        .peer()
        .call_tool(
            CallToolRequestParams::new("get").with_arguments(
                BTreeMap::from([
                    ("entity".to_string(), json!("adverse-event")),
                    ("id".to_string(), json!("1002")),
                    (
                        "sections".to_string(),
                        json!(["guidance", "reactions", "guidance", "reactions"]),
                    ),
                    ("json".to_string(), json!(true)),
                ])
                .into_iter()
                .collect(),
            ),
        )
        .await?;
    assert_eq!(typed_subset.is_error, Some(false));
    let typed_subset: serde_json::Value = serde_json::from_str(
        biomcp_mcp_contract_client::first_text(&typed_subset.content),
    )?;
    assert_eq!(typed_subset, raw_subset);
    assert!(typed_subset["data"].get("reactions").is_some());
    assert!(typed_subset["data"].get("outcomes").is_none());
    assert!(typed_subset["data"].get("patient").is_none());
    assert_eq!(
        typed_subset["_meta"]["next_commands"]
            .as_array()
            .expect("guidance commands")
            .len(),
        4
    );

    let raw_full =
        biomcp_mcp_contract_client::call_biomcp_json(&client, "biomcp get adverse-event 1003")
            .await?;
    assert_eq!(raw_full.is_error, Some(false));
    let raw_full: serde_json::Value =
        serde_json::from_str(biomcp_mcp_contract_client::first_text(&raw_full.content))?;

    let typed_all = client
        .peer()
        .call_tool(
            CallToolRequestParams::new("get").with_arguments(
                BTreeMap::from([
                    ("entity".to_string(), json!("adverse-event")),
                    ("id".to_string(), json!("1004")),
                    ("sections".to_string(), json!(["all"])),
                    ("json".to_string(), json!(true)),
                ])
                .into_iter()
                .collect(),
            ),
        )
        .await?;
    assert_eq!(typed_all.is_error, Some(false));
    let typed_all: serde_json::Value =
        serde_json::from_str(biomcp_mcp_contract_client::first_text(&typed_all.content))?;
    assert_eq!(typed_all, raw_full);
    for key in ["indication", "serious", "date"] {
        assert!(
            typed_all["data"].get(key).is_some(),
            "missing {key}: {typed_all}"
        );
    }
    assert!(
        typed_all["_meta"]["section_sources"]
            .as_array()
            .is_some_and(|sources| sources.iter().any(|source| source["key"] == "overview"))
    );

    client.cancel().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn raw_mcp_preserves_faers_report_share_context_in_json_and_markdown() -> anyhow::Result<()> {
    let harness = harness();
    let fixture = OpenFdaFixture::start();
    let cache = tempfile::tempdir()?;
    let client = harness
        .spawn_stdio_client(&[
            ("BIOMCP_OPENFDA_BASE", fixture.base.clone()),
            (
                "BIOMCP_CACHE_DIR",
                cache.path().to_string_lossy().into_owned(),
            ),
        ])
        .await?;

    let json_result = biomcp_mcp_contract_client::call_biomcp_json(
        &client,
        "biomcp --no-cache search adverse-event 'drug name' --source faers --limit 1",
    )
    .await?;
    assert_eq!(json_result.is_error, Some(false));
    let value: serde_json::Value =
        serde_json::from_str(biomcp_mcp_contract_client::first_text(&json_result.content))?;
    assert_eq!(
        value["summary"]["percentage_context"],
        json!({
            "measure": "share_of_faers_reports",
            "denominator": "returned_reports",
            "denominator_count": 1,
            "is_incidence": false,
            "establishes_causality": false
        })
    );

    let markdown_result = biomcp_mcp_contract_client::call_biomcp(
        &client,
        "biomcp --no-cache search adverse-event 'drug name' --source faers --limit 1",
    )
    .await?;
    assert_eq!(markdown_result.is_error, Some(false));
    let markdown = biomcp_mcp_contract_client::first_text(&markdown_result.content);
    assert!(markdown.contains("Share of returned reports"));
    assert!(markdown.contains("not incidence"));
    assert!(markdown.contains("does not establish causality"));

    client.cancel().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn rmcp_child_process_client_verifies_stdio_core_contract() -> anyhow::Result<()> {
    let harness = harness();
    let client = harness.spawn_stdio_client(&[]).await?;
    assert_explore_core_contract(&client).await?;
    client.cancel().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "live external-service full contract; run through make verify"]
async fn rmcp_child_process_client_verifies_stdio_full_contract() -> anyhow::Result<()> {
    let harness = harness();
    let (_ols_thread, ols_url) = start_ols4_stub()?;
    let (client, pid) = harness
        .spawn_stdio_client_with_pid(&[
            ("BIOMCP_OLS4_BASE", ols_url.clone()),
            ("BIOMCP_MEDLINEPLUS_BASE", ols_url),
        ])
        .await?;

    assert_initialize_and_tools(&client, &harness.repo_root).await?;
    assert_version_call(&client).await?;
    assert_resource_inventory_and_reads(&client, &harness.repo_root).await?;
    assert_read_only_and_policy_calls(&client).await?;
    assert_mcp_provenance_calls(&client).await?;
    assert_typed_tool_calls(&client).await?;
    assert_invalid_resource_error(&client).await?;

    terminate_process(pid)?;
    client.cancel().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn rmcp_child_process_redacts_fulltext_paths_from_text_and_json() -> anyhow::Result<()> {
    let harness = harness();
    let fixture = provision_article_fulltext_fixture(&harness.repo_root)?;
    let env = article_fulltext_fixture_env(&fixture);
    let client = harness.spawn_stdio_client(&env).await?;

    assert_mcp_fulltext_path_redaction(&client, &fixture).await?;

    client.cancel().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn rmcp_streamable_http_client_redacts_fulltext_paths_from_text_and_json()
-> anyhow::Result<()> {
    let harness = harness();
    let fixture = provision_article_fulltext_fixture(&harness.repo_root)?;
    let env = article_fulltext_fixture_env(&fixture);
    let (mut child, base_url) = harness.spawn_http_server(&env).await?;
    let result = async {
        let client = harness.http_client(format!("{base_url}/mcp")).await?;
        assert_mcp_fulltext_path_redaction(&client, &fixture).await?;
        client.cancel().await?;
        Ok::<(), anyhow::Error>(())
    }
    .await;

    child.kill().await.ok();
    result
}

#[tokio::test(flavor = "multi_thread")]
async fn rmcp_child_process_client_verifies_stdio_chart_contract() -> anyhow::Result<()> {
    let harness = harness();
    let fixture_root = provision_study_fixture(&harness.repo_root)?;
    let study_dir = study_dir_from_fixture(fixture_root.path())?;
    let client = harness
        .spawn_stdio_client(&[("BIOMCP_STUDY_DIR", study_dir)])
        .await?;

    assert_chart_calls(&client).await?;

    client.cancel().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn rmcp_streamable_http_client_verifies_core_contract() -> anyhow::Result<()> {
    let harness = harness();
    let (mut child, base_url) = harness.spawn_http_server(&[]).await?;
    let result = async {
        let client = harness.http_client(format!("{base_url}/mcp")).await?;
        assert_explore_core_contract(&client).await?;
        client.cancel().await?;
        Ok::<(), anyhow::Error>(())
    }
    .await;

    child.kill().await.ok();
    result
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "live external-service full contract; run through make verify"]
async fn rmcp_streamable_http_client_verifies_full_contract() -> anyhow::Result<()> {
    let harness = harness();
    let (_ols_thread, ols_url) = start_ols4_stub()?;
    let (mut child, base_url) = harness
        .spawn_http_server(&[
            ("BIOMCP_OLS4_BASE", ols_url.clone()),
            ("BIOMCP_MEDLINEPLUS_BASE", ols_url),
        ])
        .await?;
    let result = async {
        let client = harness.http_client(format!("{base_url}/mcp")).await?;
        assert_initialize_and_tools(&client, &harness.repo_root).await?;
        assert_version_call(&client).await?;
        assert_resource_inventory_and_reads(&client, &harness.repo_root).await?;
        assert_read_only_and_policy_calls(&client).await?;
        assert_mcp_provenance_calls(&client).await?;
        assert_typed_tool_calls(&client).await?;
        assert_invalid_resource_error(&client).await?;
        client.cancel().await?;
        Ok::<(), anyhow::Error>(())
    }
    .await;

    child.kill().await.ok();
    result
}

#[tokio::test(flavor = "multi_thread")]
async fn rmcp_streamable_http_client_verifies_chart_contract() -> anyhow::Result<()> {
    let harness = harness();
    let fixture_root = provision_study_fixture(&harness.repo_root)?;
    let study_dir = study_dir_from_fixture(fixture_root.path())?;
    let (mut child, base_url) = harness
        .spawn_http_server(&[("BIOMCP_STUDY_DIR", study_dir)])
        .await?;
    let result = async {
        let client = harness.http_client(format!("{base_url}/mcp")).await?;
        assert_chart_calls(&client).await?;
        client.cancel().await?;
        Ok::<(), anyhow::Error>(())
    }
    .await;

    child.kill().await.ok();
    result
}
