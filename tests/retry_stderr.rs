use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use wiremock::matchers::{body_string_contains, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const BATCH_PAPER_FIELDS: &str = "paperId,externalIds,title,venue,year";
const CITATION_EDGE_FIELDS: &str = "contexts,intents,isInfluential,citingPaper.paperId,citingPaper.externalIds,citingPaper.title,citingPaper.venue,citingPaper.year";

struct CommandResult {
    stdout: String,
    stderr: String,
    status: std::process::ExitStatus,
}

impl CommandResult {
    fn from_output(output: Output) -> Self {
        Self {
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            status: output.status,
        }
    }
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("biomcp-{label}-{}-{stamp}", std::process::id()));
    fs::create_dir_all(&path).expect("temp dir should be created");
    path
}

fn run_article_citations(mock_base: &str, rust_log: Option<&str>) -> CommandResult {
    let cache_home = unique_temp_dir("retry-stderr-cache");
    let mut command = Command::new(env!("CARGO_BIN_EXE_biomcp"));
    command.args(["article", "citations", "22663011", "--limit", "1"]);
    command.env("S2_API_KEY", "test-key");
    command.env("BIOMCP_S2_BASE", mock_base);
    command.env("BIOMCP_CACHE_MODE", "off");
    command.env("XDG_CACHE_HOME", &cache_home);

    if let Some(rust_log) = rust_log {
        command.env("RUST_LOG", rust_log);
    } else {
        command.env_remove("RUST_LOG");
    }

    let output = command
        .output()
        .expect("article citations command should run");
    let _ = fs::remove_dir_all(cache_home);
    CommandResult::from_output(output)
}

async fn semantic_scholar_retry_server() -> MockServer {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/graph/v1/paper/batch"))
        .and(query_param("fields", BATCH_PAPER_FIELDS))
        .and(header("x-api-key", "test-key"))
        .and(body_string_contains("\"PMID:22663011\""))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "paperId": "paper-1",
                "externalIds": {"PubMed": "22663011"},
                "title": "Seed paper",
                "venue": "Science",
                "year": 2012
            }
        ])))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/graph/v1/paper/paper-1/citations"))
        .and(query_param("fields", CITATION_EDGE_FIELDS))
        .and(query_param("limit", "1"))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(429))
        .up_to_n_times(1)
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/graph/v1/paper/paper-1/citations"))
        .and(query_param("fields", CITATION_EDGE_FIELDS))
        .and(query_param("limit", "1"))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{
                "contexts": ["Example retry context"],
                "intents": ["Background"],
                "isInfluential": false,
                "citingPaper": {
                    "paperId": "paper-2",
                    "externalIds": {"PubMed": "24200969"},
                    "title": "Recovered after retry",
                    "venue": "Nature",
                    "year": 2024
                }
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;

    server
}

#[tokio::test]
async fn article_citations_suppresses_retry_warnings_on_default_stderr_but_keeps_debug_logs() {
    let default_server = semantic_scholar_retry_server().await;
    let debug_server = semantic_scholar_retry_server().await;

    let default_result = run_article_citations(&default_server.uri(), None);
    assert!(
        default_result.status.success(),
        "expected success with default logs\nstdout:\n{}\nstderr:\n{}",
        default_result.stdout,
        default_result.stderr
    );
    assert!(
        !default_result.stderr.contains("Retry attempt"),
        "default stderr should not contain retry attempts\nstderr:\n{}",
        default_result.stderr
    );
    assert!(
        !default_result.stderr.contains("reqwest_retry"),
        "default stderr should not contain reqwest_retry target logs\nstderr:\n{}",
        default_result.stderr
    );

    let debug_result = run_article_citations(&debug_server.uri(), Some("debug"));
    assert!(
        debug_result.status.success(),
        "expected success with debug logs\nstdout:\n{}\nstderr:\n{}",
        debug_result.stdout,
        debug_result.stderr
    );
    assert!(
        debug_result.stderr.contains("Retry attempt #0"),
        "debug stderr should retain retry diagnostics\nstderr:\n{}",
        debug_result.stderr
    );
}
