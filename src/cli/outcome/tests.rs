use super::outcome_to_mcp_output;
use crate::cli::CommandOutcome;

#[test]
fn non_utf8_binary_outcome_is_never_converted_to_mcp_text() {
    let error = outcome_to_mcp_output(CommandOutcome::stdout_bytes(vec![0xff, 0xfe]))
        .expect_err("MCP must reject binary output");
    assert!(error.to_string().contains("binary downloads are CLI-only"));
    assert!(!error.to_string().contains('\u{fffd}'));
}

#[test]
fn mcp_keeps_text_from_a_nonzero_structured_outcome() {
    let output = outcome_to_mcp_output(CommandOutcome::stdout_with_exit(
        r#"{"summary":{"failed":1}}"#.to_string(),
        1,
    ))
    .expect("MCP consumes the completed report rather than its process status");
    assert_eq!(output.text, r#"{"summary":{"failed":1}}"#);
}
