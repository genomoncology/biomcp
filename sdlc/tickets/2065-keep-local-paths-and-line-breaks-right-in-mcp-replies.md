# 2065 — Keep local paths and line breaks right in MCP replies

Status: OPEN.

Milestone: 0.9.3

## Outcome

No MCP `health` reply contains a local filesystem path, and MCP error text keeps every line break the CLI prints.

## Evidence

Filed 2026-10-13 from the post-release install and product test of v0.9.2 (tag `4049666f5`), run on Linux x86_64 and macOS arm64 against every install channel.

- The MCP `biomcp` tool with `{"command":"health"}` returns rows such as "EMA local data (/home/…/.local/share/biomcp/ema)" and "Cache dir (/tmp/…/cache)", and "/Users/…" on macOS. 0.9.2 refuses cache commands over MCP and removed the WHO data path from MCP replies for this reason.
- MCP `get` for disease `MM` and for variant `TP53 S183Y` returns refusal text with no newlines; the CLI prints the same refusals on several lines. The 0.9.2 changelog says the numbering refusals' line breaks survive.
- Cause: health does not know which surface it renders for, so it always prints paths; `tool_error` runs every MCP error through `sanitize_inline`, which turns newlines into spaces.

- Starts from:
  - `src/cli/health/local.rs`, `local_data_outcome` (line 29): the label is `format!("{label} ({})", root.display())` at line 41 and `row.local_path` is set at line 53; the cache rows use `format!("Cache dir ({})", …)` at lines 301 and 318.
  - `src/cli/health/mod.rs`: `command` (line 217) takes no surface flag; `HealthRow.local_path` is at line 34.
  - `src/cli/outcome.rs`, `run_outcome_inner` (line 344): it receives `alias_suggestions_as_json`, true only on the MCP path, and calls health at line 358.
  - `src/mcp/shell.rs`: `tool_error` (line 543) uses `sanitize_inline`; `bio_mcp_error_text` (line 554) builds the text from `public_projection`; successful replies already use `sanitize_document` (line 626).
  - Fixtures `testdata/sources/mydisease/query_mm.json` and `testdata/sources/myvariant/query_tp53_s183y_20261009.json`; MCP contract tests in `tests/rmcp_client_contract.rs`, with the path-redaction test at line 1617 as the pattern.
- Keeps: the CLI's health output, its paths and its `local_path` JSON field.
- Changes:
  - Pass `mcp: bool` from `run_outcome_inner` into `health::command`. On MCP, rows use the bare label ("EMA local data", "Cache dir") and leave out `local_path`, in Markdown and JSON.
  - `tool_error` uses `sanitize_document` instead of `sanitize_inline`.
  - `docs/reference/mcp-server.md`, next to line 150: one sentence saying MCP health names local datasets and the cache without their paths.
  - One Fixes changelog bullet.
- Changes claims: `src/cli/outcome.rs` `src/cli/health/mod.rs` `src/cli/health/local.rs` `src/cli/health/tests/local.rs` `src/mcp/shell.rs` `tests/rmcp_client_contract.rs` `docs/reference/mcp-server.md` `CHANGELOG.md` `tests/test_docs_changelog_refresh.py`
- Proof: tests in `tests/rmcp_client_contract.rs` with `HOME`, the XDG folders and `BIOMCP_CACHE_DIR` inside a temporary folder: `health`, as text and with `--json`, contains no string starting with the temporary folder; `get disease MM` and `get variant "TP53 S183Y"` against the recorded fixtures return error text in which each line of the CLI's refusal appears as its own line. Both tests fail on current main.
- Defers: nothing.
- Order: safe alongside 2066, because the test compares MCP text with CLI text and so ignores refusal wording. Shares `src/mcp/shell.rs` and `src/cli/outcome.rs` with 2073 in different places; land one after the other.
