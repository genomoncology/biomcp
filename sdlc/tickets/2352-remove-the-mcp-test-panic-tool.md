# 2352 — Remove the MCP test panic tool from the shipped binary

Status: OPEN.

Milestone: 0.9.4

Reviews: revision 4b1055bfc9b79b24ab1ad79e1292f820c9df22db, accept

## Outcome

The released BioMCP binary contains no test-only MCP tool. The `__biomcp_test_panic` tool, which panics on purpose and is switched on by `BIOMCP_TEST_PANIC_TOOL=1`, moves behind a test-only build: `#[cfg(test)]`, or a non-default cargo feature that only the panic-recovery contract test enables. The MCP server's panic recovery stays proven by that test.

- Proof: the release build's MCP tool list never includes `__biomcp_test_panic`, even with `BIOMCP_TEST_PANIC_TOOL=1` set; the panic-recovery contract test still shows a panicking tool returns an error result and the server keeps serving.

## Evidence

- src/mcp/shell.rs:1336-1343 defines the `__biomcp_test_panic` tool, which panics with "injected MCP tool panic". src/mcp/shell.rs:536-547 removes it from the router and adds it back when `BIOMCP_TEST_PANIC_TOOL` equals 1. It was added on 2026-09-23 in f9c6839e0, "Close the panic recovery gaps at dispatch and locks", to test that a crashing tool does not take down the server.
- The 2026-10-21 maintainability review found it ships in the release binary. Ian did not approve shipping a test tool and wants it out of 0.9.4.

- Changes: `src/mcp/shell.rs`, `src/mcp/shell/panic_tests.rs`, `tests/rmcp_client_contract.rs`, `tests/surface/test_source_configuration_docs_contract.py`, `tests/feature-links.json`, `Makefile`, `scripts/run-nightly-checks.sh`, `.github/workflows/ci.yml`, `tests/test_release_check_counterparts.py`, `tests/test_offline_gate_contract.py`, `spec/README-timings.md`, `CHANGELOG.md`. Remove the production panic route and select the test-only recovery contract in its existing release consumers.

- Coordinator restatement: remove the production test-only MCP route and environment switch entirely, while proving the actual server catches a test-tool panic and continues through the real session boundary in a cfg(test)-only library build. No optional feature that leaks into all-features release builds. Inventory includes release workflow, Make verify, nightly runner and their command contracts, documentation/feature links and obsolete environment allowlist. Preserve the actual recovery selector when moving it.
- Current-main RED M5 j4308 on f7165ac3b6e2c25367f31f25393343313564b5e7 failed nextest100 on the shipped-server regression: with BIOMCP_TEST_PANIC_TOOL=1, the catalog hides the tool but a direct __biomcp_test_panic call returns the injected panic error instead of unknown-tool rejection. The source test transport patch SHA256 is 4597964b7968d4c640df4fd6261bc3609b12ec2da4c93d39db27c776afababfc. This run built an optimized CLI, but its unprofiled nextest invocation can use a debug CARGO_BIN_EXE; it proves current-main runtime failure, not conclusive optimized-artifact identity. Earlier origin-SHA admission/command refusals executed no build and are not product proof. Gator j4303 was cancelled before execution, then the unchanged repro moved to M5.
- The cfg(test) implementation and all caller selections are under final affected proof j4349, which explicitly uses nextest --release for both library recovery session and executable rejection contract. Its source transport patch SHA256 is 15d6f1f465cc96e24c444a308e2c45f4409bd1ef2575afff0bfe0abeebc97e17. No success is claimed until that job terminates. j4347 stopped at formatting before compilation; preserve it as such.

- Final affected M5 j4349 passed exit0 (607884ms): both actual optimized contracts ran with nextest --release. The cfg(test)-only real rmcp session returned a tool error after the injected panic and served the next version request; the shipped optimized executable hid and rejected the test tool even with its old switch set, then kept serving. Twenty-eight counterpart/docs checks passed with seven expected platform skips; formatting, Ruff and Clippy --lib --tests -D warnings passed. This source equals f7165ac plus transport patch15d6f1f465cc96e24c444a308e2c45f4409bd1ef2575afff0bfe0abeebc97e17; later SDLC Evidence does not rebind the executed artifact.
- The optimized test compile exceeded its routine7GiB booking (observed rustc approximately7.6GiB RSS); pm allows only downward booking edits. The active job completed successfully without a hold/config change or duplicate execution. Future optimized attempts must book measured headroom instead of claiming the routine booking fits. One full routine gate belongs to the accepted batch lander; the explicit affected-tests/one-gate policy supersedes generic per-developer full-suite guidance.

- Fresh independent reviewer accepts4b1055bfc9b79b24ab1ad79e1292f820c9df22db after full production/test route, unchanged panic boundary, optimized artifact/session, release-selector and documentation review. It verified j4349 matches the complete non-SDLC patch. No corrections required. One batch landing gate remains required; no developer full-suite duplicate was run.
