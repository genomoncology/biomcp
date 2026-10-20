---
flow: build
priority: 1
---
# Align the Rust MCP catalog budget

## Outcome

The Rust MCP contract enforces the accepted 22,600-byte catalog ceiling at every assertion site and reports both the measured size and the ceiling when it fails. A mechanical agreement test prevents the Rust and Python ceilings from drifting again.

## Current facts

At BioMCP `34daa72ba5e03a57bc22e64afc471dc9fdd0197a`, `tests/test_mcp_tool_catalog.py` and `scripts/measure-mcp-tools.py` enforce 22,600 bytes and 5,800 tokens. Record 1024 chose those limits to preserve useful headroom, and record 1099 reproduced them. `crates/biomcp-mcp-contract-client/src/lib.rs` still contains two independent 16,000-byte assertions. The reviewed clinical-trial integration produces a 16,044-byte catalog, so `rmcp_child_process_client_verifies_stdio_core_contract` fails against the stale Rust value even though the catalog remains within the accepted budget.

## Scope

Define one named Rust byte-ceiling constant in the existing MCP contract-client module and use it at both assertion sites. Both failures state the measured byte count and the 22,600-byte ceiling. Extend the existing Python catalog contract to read the Rust source and prove that the named Rust value equals `TOOLS_LIST_BYTE_CEILING`.

Keep the Python byte ceiling, token ceiling, description ceiling, catalog contents, tool descriptions, schemas, and runtime behavior unchanged. Do not resize the budget to the current catalog, edit ticket 1172, remove historical ticket files, add a configuration framework, add a packaged file, publish, or use Factory.

## Acceptance

1. The existing Python catalog contract gains a focused agreement test that first fails because Rust declares 16,000 bytes while Python declares 22,600 bytes.
2. Both Rust assertions use one named 22,600-byte constant and include the measured count and ceiling in their failure messages.
3. The existing Python catalog contract fails if the Rust and Python byte ceilings differ.
4. The catalog measurement remains within 22,600 bytes, 5,800 tokens, and 4,000 description bytes without changing catalog content.
5. The source package remains at or below 1,300 files. The focused Rust MCP contract and Python catalog contract pass, followed by `make lint`, `make test`, and `make spec`.

## Dependencies

None. This ticket repairs duplicated contract state left behind by record 1024 and must land before the clinical-trial integration rebases onto current main.

## Review

- Design review: rejected because the first acceptance step depended on the separate uncommitted clinical-trial worktree to reproduce a 16,044-byte catalog. The corrected ticket uses a local red agreement test for Rust's 16,000-byte value against Python's accepted 22,600-byte value. Independent re-review accepted the corrected design.
- Code review: rejected because the first agreement test checked only the named Rust constant and could pass if either assertion stopped using it. The remediation requires exactly two serialized-length assertions and proves that both use the constant and report the measurement and ceiling. Independent re-review accepted the final diff.

## Implementation evidence

- Red: the new agreement test failed because Rust declared 16,000 bytes while Python declared 22,600 bytes.
- Focused green: both Rust MCP core contracts, all eight Python catalog tests, all six source-package policy tests, Ruff, Rust formatting, and `git diff --check` passed.
- The unchanged catalog measured 15,992 bytes, 4,064 tokens, and 211 description bytes against ceilings of 22,600, 5,800, and 4,000.
- Complete gates: `make lint`, `make test`, and `make spec` passed. The test gate ran 3,250 Rust tests with 30 skipped and 903 Python tests with 3 skipped. Strict documentation and every specification suite passed. The source package contains exactly 1,300 files.
