//! Score-session parsing and orchestration tests.

use std::fs;
use std::path::PathBuf;

use super::*;
use crate::test_support::TempDirGuard;

fn temp_path(root: &TempDirGuard, name: &str) -> PathBuf {
    root.path().join(name)
}

#[test]
fn score_session_extracts_counts_tokens_errors_and_coverage() {
    let root = TempDirGuard::new("benchmark-score-session");
    let session_path = temp_path(&root, "session.jsonl");
    let expected_path = temp_path(&root, "expected.txt");

    let session = [
        r#"{"timestamp":"2026-02-17T12:00:00Z","tool":{"name":"bash","input":{"cmd":"biomcp get gene BRAF"}},"usage":{"input_tokens":10,"output_tokens":4,"cache_read_tokens":2,"cache_write_tokens":1,"cost_usd":0.001}}"#,
        r#"{"timestamp":"2026-02-17T12:00:02Z","event":{"name":"biomcp","arguments":{"command":"biomcp --help"}}}"#,
        r#"{"timestamp":"2026-02-17T12:00:04Z","tool":{"name":"bash","input":{"cmd":"cat skills/use-cases/03-trial-searching.md"}}}"#,
        r#"{"timestamp":"2026-02-17T12:00:05Z","stderr":"HTTP 503 from api"}"#,
        r#"{"timestamp":"2026-02-17T12:00:06Z","error":"unterminated quote in command"}"#,
    ]
    .join("\n");

    fs::write(&session_path, session).expect("write session");
    fs::write(
        &expected_path,
        "biomcp get gene TP53\nbiomcp --help\nbiomcp search trial -c melanoma --limit 5\n",
    )
    .expect("write expected");

    let report = score_session_file(&ScoreSessionOptions {
        session: session_path.clone(),
        expected: Some(expected_path.clone()),
        brief: false,
    })
    .expect("score");

    assert_eq!(report.total_tool_calls, 3);
    assert_eq!(report.biomcp_commands, 2);
    assert_eq!(report.help_calls, 1);
    assert_eq!(report.skill_reads, 1);
    assert_eq!(report.errors_total, 2);
    assert_eq!(report.error_categories.api, 1);
    assert_eq!(report.error_categories.quoting, 1);
    assert_eq!(report.tokens.input_tokens, 10);
    assert_eq!(report.tokens.output_tokens, 4);
    assert_eq!(report.tokens.cache_read_tokens, 2);
    assert_eq!(report.tokens.cache_write_tokens, 1);
    assert!((report.tokens.cost_usd - 0.001).abs() < 1e-9);
    assert_eq!(report.wall_time_ms, Some(6000));

    let coverage = report.coverage.expect("coverage");
    assert_eq!(coverage.expected_total, 3);
    assert_eq!(coverage.hits, 2);
    assert_eq!(coverage.misses, 1);
    assert_eq!(coverage.extras, 0);
}

#[test]
fn recognizes_legacy_and_current_biomcp_tool_names() {
    assert!(parse::is_biomcp_tool_name("biomcp"));
    assert!(parse::is_biomcp_tool_name("mcp.biomcp"));
    assert!(parse::is_biomcp_tool_name("shell"));
    assert!(parse::is_biomcp_tool_name("mcp.shell"));
    assert!(parse::is_biomcp_tool_name("bash"));
    assert!(parse::is_biomcp_tool_name("mcp.bash"));
    assert!(!parse::is_biomcp_tool_name("python"));
}

#[test]
fn classify_error_detects_expected_categories() {
    assert!(matches!(
        parse::classify_error("ghost command not found"),
        parse::ErrorCategory::Ghost
    ));
    assert!(matches!(
        parse::classify_error("unterminated quote"),
        parse::ErrorCategory::Quoting
    ));
    assert!(matches!(
        parse::classify_error("HTTP 503 from api"),
        parse::ErrorCategory::Api
    ));
    assert!(matches!(
        parse::classify_error("misc issue"),
        parse::ErrorCategory::Other
    ));
}

#[test]
fn fails_on_invalid_jsonl_line() {
    let root = TempDirGuard::new("benchmark-score-invalid");
    let session_path = temp_path(&root, "invalid.jsonl");
    fs::write(&session_path, "{not-json}\n").expect("write");

    let err = score_session_file(&ScoreSessionOptions {
        session: session_path.clone(),
        expected: None,
        brief: true,
    })
    .expect_err("invalid json should fail");

    assert!(err.to_string().contains("invalid JSONL line"));
}
