use std::process::{Command, Output};

use wiremock::MockServer;

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

fn run_article_search(
    args: &[&str],
    pubtator_base: &str,
    europepmc_base: &str,
    s2_base: &str,
) -> CommandResult {
    let cache_home = tempfile::Builder::new()
        .prefix("biomcp-article-usage-stderr-cache-")
        .tempdir()
        .expect("temp dir should be created");
    let mut command = Command::new(env!("CARGO_BIN_EXE_biomcp"));
    command.args(["search", "article"]);
    command.args(args);
    command.env("BIOMCP_PUBTATOR_BASE", pubtator_base);
    command.env("BIOMCP_EUROPEPMC_BASE", europepmc_base);
    command.env("BIOMCP_S2_BASE", s2_base);
    command.env("BIOMCP_CACHE_MODE", "off");
    command.env("XDG_CACHE_HOME", cache_home.path());
    command.env_remove("RUST_LOG");
    command.env_remove("S2_API_KEY");

    let output = command.output().expect("article search command should run");
    CommandResult::from_output(output)
}

fn assert_clean_usage_error(result: &CommandResult, expected_stderr_line: &str) {
    assert_eq!(
        result.status.code(),
        Some(2),
        "expected invalid-usage exit code 2\nstdout:\n{}\nstderr:\n{}",
        result.stdout,
        result.stderr
    );
    assert!(
        result.stdout.trim().is_empty(),
        "usage failure should not print stdout\nstdout:\n{}",
        result.stdout
    );
    let stderr_lines = result.stderr.lines().collect::<Vec<_>>();
    assert!(
        result.stderr.starts_with("Error: Invalid argument:"),
        "stderr should start with the invalid-argument prefix\nstderr:\n{}",
        result.stderr
    );
    assert_eq!(
        stderr_lines,
        vec![expected_stderr_line],
        "stderr should stay a single clean usage-error line\nstderr:\n{}",
        result.stderr
    );
    for forbidden in [
        "WARN",
        "PubTator",
        "Europe PMC",
        "Semantic Scholar",
        "Retry attempt",
    ] {
        assert!(
            !result.stderr.contains(forbidden),
            "stderr should not contain backend warning noise: {forbidden}\nstderr:\n{}",
            result.stderr
        );
    }
}

async fn assert_no_backend_requests(server: &MockServer, label: &str) {
    let requests = server
        .received_requests()
        .await
        .expect("mock server should record requests");
    assert!(
        requests.is_empty(),
        "expected no {label} requests for invalid front-door input, saw {}\nrequests:\n{requests:#?}",
        requests.len()
    );
}

#[tokio::test]
async fn invalid_article_date_is_clean_usage_error() {
    let pubtator = MockServer::start().await;
    let europepmc = MockServer::start().await;
    let s2 = MockServer::start().await;

    let result = run_article_search(
        &["-g", "BRAF", "--date-from", "2025-99-01", "--limit", "1"],
        &pubtator.uri(),
        &europepmc.uri(),
        &s2.uri(),
    );

    assert_clean_usage_error(
        &result,
        "Error: Invalid argument: Invalid month 99 in --date-from (must be 01-12)",
    );
    assert_no_backend_requests(&pubtator, "PubTator").await;
    assert_no_backend_requests(&europepmc, "Europe PMC").await;
    assert_no_backend_requests(&s2, "Semantic Scholar").await;
}

#[tokio::test]
async fn missing_article_filters_is_clean_usage_error() {
    let pubtator = MockServer::start().await;
    let europepmc = MockServer::start().await;
    let s2 = MockServer::start().await;

    let result = run_article_search(
        &["--limit", "1"],
        &pubtator.uri(),
        &europepmc.uri(),
        &s2.uri(),
    );

    assert_clean_usage_error(
        &result,
        "Error: Invalid argument: At least one filter is required. Example: biomcp search article -g BRAF",
    );
    assert_no_backend_requests(&pubtator, "PubTator").await;
    assert_no_backend_requests(&europepmc, "Europe PMC").await;
    assert_no_backend_requests(&s2, "Semantic Scholar").await;
}

#[tokio::test]
async fn invalid_article_session_token_rejected_before_backend() {
    let pubtator = MockServer::start().await;
    let europepmc = MockServer::start().await;
    let s2 = MockServer::start().await;

    let result = run_article_search(
        &[
            "-k",
            "BRAF",
            "--session",
            "../unsafe",
            "--source",
            "pubtator",
            "--limit",
            "1",
        ],
        &pubtator.uri(),
        &europepmc.uri(),
        &s2.uri(),
    );

    assert_clean_usage_error(
        &result,
        "Error: Invalid argument: --session must be 1-128 ASCII characters containing only letters, digits, '.', '_', ':', or '-'",
    );
    assert_no_backend_requests(&pubtator, "PubTator").await;
    assert_no_backend_requests(&europepmc, "Europe PMC").await;
    assert_no_backend_requests(&s2, "Semantic Scholar").await;
}

#[tokio::test]
async fn inverted_article_date_range_is_clean_usage_error() {
    let pubtator = MockServer::start().await;
    let europepmc = MockServer::start().await;
    let s2 = MockServer::start().await;

    let result = run_article_search(
        &[
            "-g",
            "BRAF",
            "--date-from",
            "2024-01-01",
            "--date-to",
            "2020-01-01",
            "--limit",
            "1",
        ],
        &pubtator.uri(),
        &europepmc.uri(),
        &s2.uri(),
    );

    assert_clean_usage_error(
        &result,
        "Error: Invalid argument: --date-from must be <= --date-to",
    );
    assert_no_backend_requests(&pubtator, "PubTator").await;
    assert_no_backend_requests(&europepmc, "Europe PMC").await;
    assert_no_backend_requests(&s2, "Semantic Scholar").await;
}

#[tokio::test]
async fn invalid_article_date_to_is_clean_usage_error() {
    let pubtator = MockServer::start().await;
    let europepmc = MockServer::start().await;
    let s2 = MockServer::start().await;

    let result = run_article_search(
        &["-g", "BRAF", "--date-to", "2024-99", "--limit", "1"],
        &pubtator.uri(),
        &europepmc.uri(),
        &s2.uri(),
    );

    assert_clean_usage_error(
        &result,
        "Error: Invalid argument: Invalid month 99 in --date-to (must be 01-12)",
    );
    assert_no_backend_requests(&pubtator, "PubTator").await;
    assert_no_backend_requests(&europepmc, "Europe PMC").await;
    assert_no_backend_requests(&s2, "Semantic Scholar").await;
}

#[tokio::test]
async fn invalid_article_type_is_clean_usage_error_before_pubtator_route() {
    let pubtator = MockServer::start().await;
    let europepmc = MockServer::start().await;
    let s2 = MockServer::start().await;

    let result = run_article_search(
        &[
            "-g", "BRAF", "--type", "nonsense", "--source", "pubtator", "--limit", "1",
        ],
        &pubtator.uri(),
        &europepmc.uri(),
        &s2.uri(),
    );

    assert_clean_usage_error(
        &result,
        "Error: Invalid argument: --type must be one of: review, research, research-article, case-reports, meta-analysis",
    );
    assert_no_backend_requests(&pubtator, "PubTator").await;
    assert_no_backend_requests(&europepmc, "Europe PMC").await;
    assert_no_backend_requests(&s2, "Semantic Scholar").await;
}
