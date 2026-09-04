use std::net::TcpListener;
use std::process::{Command, Output};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use std::thread;
use std::time::Duration;

struct CountingFixture {
    base: String,
    requests: Arc<AtomicUsize>,
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}

impl CountingFixture {
    fn start() -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind counting fixture");
        listener.set_nonblocking(true).expect("nonblocking fixture");
        let base = format!("http://{}", listener.local_addr().expect("fixture address"));
        let requests = Arc::new(AtomicUsize::new(0));
        let stop = Arc::new(AtomicBool::new(false));
        let thread_requests = Arc::clone(&requests);
        let thread_stop = Arc::clone(&stop);
        let thread = thread::spawn(move || {
            while !thread_stop.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((_stream, _)) => {
                        thread_requests.fetch_add(1, Ordering::SeqCst);
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(_) => return,
                }
            }
        });
        Self {
            base,
            requests,
            stop,
            thread: Some(thread),
        }
    }

    fn assert_zero(&self) {
        thread::sleep(Duration::from_millis(20));
        assert_eq!(
            self.requests.load(Ordering::SeqCst),
            0,
            "invalid input reached a provider"
        );
    }
}

impl Drop for CountingFixture {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            thread.join().expect("join fixture");
        }
    }
}

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

fn run_article_search(args: &[&str]) -> CommandResult {
    let fixture = CountingFixture::start();
    let result = run_article_search_at(args, &fixture.base);
    fixture.assert_zero();
    result
}

fn run_article_search_at(args: &[&str], base: &str) -> CommandResult {
    let cache_home = tempfile::Builder::new()
        .prefix("biomcp-article-usage-stderr-cache-")
        .tempdir()
        .expect("temp dir should be created");
    let mut command = Command::new(env!("CARGO_BIN_EXE_biomcp"));
    command.args(["search", "article"]);
    command.args(args);
    for name in [
        "BIOMCP_PUBTATOR_BASE",
        "BIOMCP_EUROPEPMC_BASE",
        "BIOMCP_PUBMED_BASE",
        "BIOMCP_S2_BASE",
        "BIOMCP_LITSENSE2_BASE",
    ] {
        command.env(name, base);
    }
    command.env("BIOMCP_CACHE_MODE", "off");
    command.env("XDG_CACHE_HOME", cache_home.path());
    command.env_remove("RUST_LOG");
    command.env_remove("S2_API_KEY");

    let output = command.output().expect("article search command should run");
    CommandResult::from_output(output)
}

fn run_search_all(args: &[&str], base: &str) -> CommandResult {
    let mut command = Command::new(env!("CARGO_BIN_EXE_biomcp"));
    command.args(["search", "all"]);
    command.args(args);
    // Keep this list adjacent to the seven-leg assertion so a new provider cannot
    // silently escape the zero-request proof.
    for name in [
        "BIOMCP_MYGENE_BASE",
        "BIOMCP_MYVARIANT_BASE",
        "BIOMCP_MYCHEM_BASE",
        "BIOMCP_CTGOV_BASE",
        "BIOMCP_PUBTATOR_BASE",
        "BIOMCP_EUROPEPMC_BASE",
        "BIOMCP_PUBMED_BASE",
        "BIOMCP_S2_BASE",
        "BIOMCP_LITSENSE2_BASE",
        "BIOMCP_REACTOME_BASE",
        "BIOMCP_KEGG_BASE",
        "BIOMCP_WIKIPATHWAYS_BASE",
        "BIOMCP_CPIC_BASE",
    ] {
        command.env(name, base);
    }
    command.env("BIOMCP_CACHE_MODE", "off");
    command.env_remove("RUST_LOG");
    CommandResult::from_output(command.output().expect("search all command should run"))
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

#[test]
fn invalid_article_date_is_clean_usage_error() {
    let result = run_article_search(&["-g", "BRAF", "--date-from", "2025-99-01", "--limit", "1"]);

    assert_clean_usage_error(
        &result,
        "Error: Invalid argument: Invalid month 99 in --date-from (must be 01-12)",
    );
}

#[test]
fn missing_article_filters_is_clean_usage_error() {
    let result = run_article_search(&["--limit", "1"]);

    assert_clean_usage_error(
        &result,
        "Error: Invalid argument: At least one filter is required. Example: biomcp search article -g BRAF",
    );
}

#[test]
fn invalid_article_session_token_rejected_before_backend() {
    let result = run_article_search(&[
        "-k",
        "BRAF",
        "--session",
        "../unsafe",
        "--source",
        "pubtator",
        "--limit",
        "1",
    ]);

    assert_clean_usage_error(
        &result,
        "Error: Invalid argument: --session must be 1-128 ASCII characters containing only letters, digits, '.', '_', ':', or '-'",
    );
}

#[test]
fn inverted_article_date_range_is_clean_usage_error() {
    let result = run_article_search(&[
        "-g",
        "BRAF",
        "--date-from",
        "2024-01-01",
        "--date-to",
        "2020-01-01",
        "--limit",
        "1",
    ]);

    assert_clean_usage_error(
        &result,
        "Error: Invalid argument: --date-from must be <= --date-to",
    );
}

#[test]
fn invalid_article_date_to_is_clean_usage_error() {
    let result = run_article_search(&["-g", "BRAF", "--date-to", "2024-99", "--limit", "1"]);

    assert_clean_usage_error(
        &result,
        "Error: Invalid argument: Invalid month 99 in --date-to (must be 01-12)",
    );
}

#[test]
fn invalid_article_type_is_clean_usage_error_before_pubtator_route() {
    let result = run_article_search(&[
        "-g", "BRAF", "--type", "nonsense", "--source", "pubtator", "--limit", "1",
    ]);

    assert_clean_usage_error(
        &result,
        "Error: Invalid argument: --type must be one of: review, research, research-article, case-reports, meta-analysis",
    );
}

#[test]
fn malformed_article_query_inputs_are_clean_and_make_zero_requests() {
    let fixture = CountingFixture::start();
    let cases = [
        (
            "gene:RB1",
            "Error: Invalid argument: keyword is provider-neutral and does not accept gene: filter syntax. Use --gene RB1 for CLI or raw MCP, or the typed MCP field, for example \"gene\":\"RB1\".",
        ),
        (
            "disease:melanoma",
            "Error: Invalid argument: keyword is provider-neutral and does not accept disease: filter syntax. Use --disease melanoma for CLI or raw MCP, or the typed MCP field, for example \"disease\":\"melanoma\".",
        ),
        (
            "drug:vemurafenib",
            "Error: Invalid argument: keyword is provider-neutral and does not accept drug: filter syntax. Use --drug vemurafenib for CLI or raw MCP, or the typed MCP field, for example \"drug\":\"vemurafenib\".",
        ),
    ];
    for (keyword, expected) in cases {
        for alias in [Some("-k"), Some("-q"), Some("--query"), None] {
            let args = alias.map_or_else(|| vec![keyword], |flag| vec![flag, keyword]);
            let result = run_article_search_at(&args, &fixture.base);
            assert_clean_usage_error(&result, expected);
        }
    }
    let malicious = run_article_search_at(&["-k", "gene:RB1;$(touch /tmp/nope)"], &fixture.base);
    assert_clean_usage_error(&malicious, cases[0].1);
    let gene_line = "Error: Invalid argument: gene accepts one symbol, for example TPMT. Put additional concepts in keyword: use --gene TPMT --keyword mercaptopurine for CLI or raw MCP, or typed MCP fields \"gene\":\"TPMT\" and \"keyword\":[\"mercaptopurine\"].";
    for gene in ["TPMT mercaptopurine", ""] {
        let result = run_article_search_at(&["--gene", gene], &fixture.base);
        assert_clean_usage_error(&result, gene_line);
    }
    fixture.assert_zero();
}

#[test]
fn malformed_search_all_inputs_fail_before_the_seven_leg_plan() {
    let fixture = CountingFixture::start();
    let cases = [
        (
            vec!["--keyword", "gene:RB1"],
            "Error: Invalid argument: keyword is provider-neutral and does not accept gene: filter syntax. Use --gene RB1 for CLI or raw MCP, or the typed MCP field, for example \"gene\":\"RB1\".",
        ),
        (
            vec!["--keyword", "disease:melanoma"],
            "Error: Invalid argument: keyword is provider-neutral and does not accept disease: filter syntax. Use --disease melanoma for CLI or raw MCP, or the typed MCP field, for example \"disease\":\"melanoma\".",
        ),
        (
            vec!["--keyword", "drug:vemurafenib"],
            "Error: Invalid argument: keyword is provider-neutral and does not accept drug: filter syntax. Use --drug vemurafenib for CLI or raw MCP, or the typed MCP field, for example \"drug\":\"vemurafenib\".",
        ),
        (
            vec!["--gene", "TPMT mercaptopurine"],
            "Error: Invalid argument: gene accepts one symbol, for example TPMT. Put additional concepts in keyword: use --gene TPMT --keyword mercaptopurine for CLI or raw MCP, or typed MCP fields \"gene\":\"TPMT\" and \"keyword\":[\"mercaptopurine\"].",
        ),
        (
            vec!["--gene", ""],
            "Error: Invalid argument: gene accepts one symbol, for example TPMT. Put additional concepts in keyword: use --gene TPMT --keyword mercaptopurine for CLI or raw MCP, or typed MCP fields \"gene\":\"TPMT\" and \"keyword\":[\"mercaptopurine\"].",
        ),
    ];
    for (args, expected) in &cases {
        let result = run_search_all(args, &fixture.base);
        assert_clean_usage_error(&result, expected);
        assert!(!result.stderr.contains("# Cross-Entity Search"));
    }
    for (args, expected) in cases {
        let mut json_args = vec!["--json"];
        json_args.extend(args);
        let result = run_search_all(&json_args, &fixture.base);
        assert_eq!(result.status.code(), Some(2));
        assert!(result.stderr.is_empty(), "stderr={}", result.stderr);
        let value: serde_json::Value =
            serde_json::from_str(&result.stdout).expect("structured error");
        assert_eq!(value["error"]["code"], "invalid_argument");
        assert_eq!(
            value["error"]["message"],
            expected.strip_prefix("Error: ").unwrap()
        );
        assert!(!result.stdout.contains("# Cross-Entity Search"));
    }
    fixture.assert_zero();
}
