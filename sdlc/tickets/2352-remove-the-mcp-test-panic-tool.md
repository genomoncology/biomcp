# 2352 — Remove the MCP test panic tool from the shipped binary

Status: OPEN.

Milestone: 0.9.4

## Outcome

The released BioMCP binary contains no test-only MCP tool. The `__biomcp_test_panic` tool, which panics on purpose and is switched on by `BIOMCP_TEST_PANIC_TOOL=1`, moves behind a test-only build: `#[cfg(test)]`, or a non-default cargo feature that only the panic-recovery contract test enables. The MCP server's panic recovery stays proven by that test.

- Proof: the release build's MCP tool list never includes `__biomcp_test_panic`, even with `BIOMCP_TEST_PANIC_TOOL=1` set; the panic-recovery contract test still shows a panicking tool returns an error result and the server keeps serving.

## Evidence

- src/mcp/shell.rs:1336-1343 defines the `__biomcp_test_panic` tool, which panics with "injected MCP tool panic". src/mcp/shell.rs:536-547 removes it from the router and adds it back when `BIOMCP_TEST_PANIC_TOOL` equals 1. It was added on 2026-09-23 in f9c6839e0, "Close the panic recovery gaps at dispatch and locks", to test that a crashing tool does not take down the server.
- The 2026-10-21 maintainability review found it ships in the release binary. Ian did not approve shipping a test tool and wants it out of 0.9.4.

- Changes: `src/mcp/shell.rs`, `src/mcp/shell/panic_tests.rs`, `tests/rmcp_client_contract.rs`, `tests/surface/test_source_configuration_docs_contract.py`, `tests/feature-links.json`, `Makefile`, `scripts/run-nightly-checks.sh`, `.github/workflows/ci.yml`, `tests/test_release_check_counterparts.py`, `tests/test_offline_gate_contract.py`, `spec/README-timings.md`, `CHANGELOG.md`. Remove the production panic route and select the test-only recovery contract in its existing release consumers.
