use std::process::{Command, Output};

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

fn run_biomcp(args: &[&str]) -> CommandResult {
    let cache_home = tempfile::Builder::new()
        .prefix("biomcp-live-smoke-cache-")
        .tempdir()
        .expect("temp dir should be created");
    let mut command = Command::new(env!("CARGO_BIN_EXE_biomcp"));
    command.args(args);
    command.env("BIOMCP_CACHE_MODE", "off");
    command.envs(biomcp_mcp_contract_client::child_cache_env(
        cache_home.path(),
    ));
    command.env_remove("RUST_LOG");

    let output = command.output().expect("biomcp command should run");
    CommandResult::from_output(output)
}

fn assert_success_json_contains(result: &CommandResult, expected: &[&str]) {
    assert!(
        result.status.success(),
        "expected smoke command to succeed\nstdout:\n{}\nstderr:\n{}",
        result.stdout,
        result.stderr
    );
    assert!(
        result.stderr.trim().is_empty(),
        "smoke command should not emit stderr noise\nstderr:\n{}",
        result.stderr
    );
    let value: serde_json::Value =
        serde_json::from_str(&result.stdout).expect("stdout should be JSON");
    let rendered = value.to_string();
    for needle in expected {
        assert!(
            rendered.contains(needle),
            "expected JSON output to contain {needle:?}\nstdout:\n{}",
            result.stdout
        );
    }
}

#[test]
#[ignore = "live network smoke"]
fn live_cli_smoke_get_gene_braf_returns_gene_information() {
    let result = run_biomcp(&["--json", "get", "gene", "BRAF"]);

    assert_success_json_contains(&result, &["BRAF"]);
}

#[test]
#[ignore = "live network smoke"]
fn live_cli_smoke_get_variant_braf_v600e_returns_variant_information() {
    let result = run_biomcp(&["--json", "get", "variant", "chr7:g.140453136A>T"]);

    assert_success_json_contains(&result, &["chr7:g.140453136A>T", "BRAF"]);
}

#[test]
#[ignore = "live network smoke"]
fn live_cli_smoke_get_article_returns_article_information() {
    let result = run_biomcp(&["--json", "get", "article", "22663011"]);

    assert_success_json_contains(&result, &["22663011"]);
}
