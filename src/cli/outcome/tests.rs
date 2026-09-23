use super::outcome_to_mcp_output;
use crate::cli::{CommandOutcome, VariantArticlesMcpDisposition};

#[test]
fn worker_panic_on_execute_thread_surfaces_as_an_error_result() {
    // Mirrors the join in run_outcome_with_worker_stack: a panicking execute
    // worker must surface as the mapped error result, not end the process.
    // Under a release profile with panic = "abort" the thread panic kills the
    // test binary, so the focused release-profile run of this test is the
    // proof that unwinding survives (ticket 1230).
    let handle = std::thread::Builder::new()
        .name("biomcp-cli-execute".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(|| -> anyhow::Result<()> { panic!("execute worker failed") })
        .expect("spawn execute worker");

    let error = handle
        .join()
        .map_err(|_| anyhow::anyhow!("in-process CLI worker panicked"))
        .expect_err("worker panic must surface as an error result");
    assert_eq!(error.to_string(), "in-process CLI worker panicked");
    // Reaching this assertion proves the process survived the panic.
}

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

#[test]
fn variant_article_disposition_survives_the_raw_mcp_execution_seam() {
    let output = outcome_to_mcp_output(
        CommandOutcome::stdout_with_exit("structured".into(), 1)
            .with_variant_articles_mcp_disposition(VariantArticlesMcpDisposition::StructuredError),
    )
    .expect("text output");
    assert_eq!(
        output.variant_articles_mcp_disposition,
        Some(VariantArticlesMcpDisposition::StructuredError)
    );
}
