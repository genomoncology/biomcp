---
priority: 3
---

# 2073 — Delete the benchmark harness and serve-sse leftovers

Status: OPEN.

Milestone: 0.9.3

## Outcome

The unused benchmark harness and hidden serve-sse command are removed, and `biomcp serve-sse` exits 2 with the unrecognized-subcommand error.

Depends on: 2159

- Starts from: the existing implementation branch, `src/cli/benchmark/`, the CLI command dispatch and affected MCP help surfaces.
- Keeps: the separate output-footprint and BioASQ tools and the Reactome numeric response variant.
- Changes: claims `src/cli/benchmark/**` `src/cli/mod.rs` `src/cli/commands.rs` `src/main.rs` `src/cli/shared.rs` `src/cli/outcome.rs` `src/cli/response_contract.rs` `src/mcp/mod.rs` `src/mcp/shell.rs` `src/cli/tests/facade/help.rs` `tests/benchmark_cli_structure.rs` `tests/test_central_test_support_helpers.py` `tests/test_mcp_http_surface.py` `tests/test_public_skill_docs_contract.py` `tests/test_upstream_planning_analysis_docs.py` `tests/test_docs_changelog_refresh.py` `tests/test_source_package_boundary.py` `tests/surface/test_ticket_405_architecture_operator_contracts.py` `architecture/technical/benchmark-cli-ownership-decision.md` `architecture/technical/cli-decomposition-2026.md` `architecture/ux/cli-reference.md` `docs/user-guide/cli-reference.md` `spec/surface/cli.md` `scripts/prepare-spec-artifacts.py` `scripts/run-specs.sh` `tools/dead-code-exceptions.json` `CHANGELOG.md`.
- Proof: the affected command test and routine merged-candidate gate pass through build jobs. Searching the declared source, test, documentation and specification paths finds the removed command names only in the replacement refusal test.
- Defers: unrelated test-only code to 2081.

PM decision: 2026-10-17. Reuse the existing implementation; rebase and review its actual merged result. Do not restore obsolete exact package-count edits after 2159.

PM decision: 2026-10-17. Back in 0.9.3. Faster tests and builds are a 0.9.3 goal, and this ticket carries it. Rebase onto main before resuming; earlier reviews cover the old base only.
