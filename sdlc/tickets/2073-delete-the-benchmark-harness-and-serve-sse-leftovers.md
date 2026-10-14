# 2073 — Delete the benchmark harness and serve-sse leftovers

Status: OPEN.

Milestone: 0.9.3

## Outcome

`src/cli/benchmark/` and the hidden `serve-sse` command are gone, and `biomcp serve-sse` fails as an unrecognized command.

## Evidence

- `src/cli/benchmark/` (17 files, 2,560 lines) compiles only under `#[cfg(test)]`, has no command, and nothing outside it refers to it (`sdlc/designs/2026-10-13-0-9-3-dependencies-and-dead-code.md`, Dead code).
- `serve-sse` is a hidden command that only prints a removal message; `main.rs` keeps two match arms for it, and three Python tests, two doc tests and one Rust help test assert it.
- The reactome field the study named stays: `Id(i64)` in `ReactomeContainedEvent` (`src/sources/reactome.rs:384-387`) lets the untagged enum accept numeric rows, and without it one numeric row fails the whole response.

- Starts from:
  - Benchmark harness: `src/cli/mod.rs:6-11` (`#[cfg(test)] mod benchmark;`, its dead-code allow and pointer comment); `src/cli/benchmark/`; `tests/benchmark_cli_structure.rs`; `tests/test_central_test_support_helpers.py:78`, which allowlists a benchmark file; `architecture/technical/benchmark-cli-ownership-decision.md` and its pointers in `architecture/technical/cli-decomposition-2026.md` (lines 13-16, line 45, and the "Benchmark internal harness" section) and `tests/surface/test_ticket_405_architecture_operator_contracts.py:16`; the `mod benchmark;` entry in `tools/dead-code-exceptions.json`.
  - serve-sse code: `src/cli/commands.rs:185` (`ServeSse`), `src/main.rs:128` and `:159`, `src/mcp/mod.rs:43-51` (`sse_deprecation_message`, `run_sse`), `src/cli/shared.rs:10` (`RUNTIME_HELP_SUBCOMMANDS`), `src/cli/outcome.rs:333`, `src/cli/response_contract.rs:238`, `src/mcp/shell.rs:805`.
  - serve-sse tests: `src/cli/tests/facade/help.rs:76-115`, `tests/test_mcp_http_surface.py:490-535`, `tests/test_public_skill_docs_contract.py:287-290`, `tests/test_upstream_planning_analysis_docs.py:770-775`.
  - serve-sse docs: `architecture/ux/cli-reference.md:43`, `docs/user-guide/cli-reference.md:829`.
- Keeps: `benchmarks/output-footprint` and `benchmarks/bioasq`, which are separate tools; `ReactomeContainedEvent`; the test-only items, which go with 2081.
- Changes:
  - Delete the benchmark module, its structure test, the decision note (decided: delete it, not keep it) and the note's pointers.
  - Delete the `ServeSse` variant, both `main.rs` arms, `run_sse` and `sse_deprecation_message`, and the match arms in outcome, response_contract and shell. Shrink `RUNTIME_HELP_SUBCOMMANDS` to three entries.
  - Replace the three serve-sse tests in `tests/test_mcp_http_surface.py` with one test asserting `biomcp serve-sse` exits 2 with clap's unrecognized-subcommand error.
  - Delete the serve-sse doc lines and the doc-test assertions on them.
  - Remove the `mod benchmark;` exception entry.
  - One Internal changelog bullet naming both removals.
- Changes claims: `src/cli/benchmark/**` `src/cli/mod.rs` `tests/benchmark_cli_structure.rs` `tests/test_central_test_support_helpers.py` `architecture/technical/benchmark-cli-ownership-decision.md` `architecture/technical/cli-decomposition-2026.md` `tests/surface/test_ticket_405_architecture_operator_contracts.py` `src/main.rs` `src/cli/commands.rs` `src/cli/shared.rs` `src/cli/outcome.rs` `src/cli/response_contract.rs` `src/mcp/mod.rs` `src/mcp/shell.rs` `src/cli/tests/facade/help.rs` `tests/test_mcp_http_surface.py` `tests/test_public_skill_docs_contract.py` `tests/test_upstream_planning_analysis_docs.py` `architecture/ux/cli-reference.md` `docs/user-guide/cli-reference.md` `tools/dead-code-exceptions.json` `CHANGELOG.md` `tests/test_docs_changelog_refresh.py`
- Proof: the build and every lane pass; the new test shows `biomcp serve-sse` exits 2; `grep -rn 'serve-sse\|ServeSse\|run_sse' src tests docs architecture spec` matches only that test; `tools/dead-code-exceptions.json` has one entry fewer.
- Defers: the 63 test-only items and the 58 fields no code reads (2081).
- Order: after 2070, which also edits `tests/test_upstream_planning_analysis_docs.py`. Safe alongside 2071. Shares `src/mcp/shell.rs` and `src/cli/outcome.rs` with 2065 in different places, so the two land one after the other.
