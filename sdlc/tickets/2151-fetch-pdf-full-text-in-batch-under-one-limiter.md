---
flow: build
priority: 2
---

# 2151: Fetch PDF full text in batch under one limiter

Status: OPEN.

Milestone: 0.9.3

Reviews: revision 86572f8, accept

## Outcome

Coordinator implementation brief: preserve the adopted batch PDF command and typed MCP behavior. Prove Semantic Scholar discovery spacing through the actual batch command with its limiter enabled. The old acceptance test times publisher downloads, so it does not establish this requirement. Expand the claim to `src/sources/rate_limit.rs` for a fixture-only `BIOMCP_TEST_PRESERVE_RATE_LIMITS` switch: retain existing exact-loopback authorization while bypassing only the fixture's unpaced exemption. The default product policy remains unchanged. Limit new test edits to `tests/article_batch_outcome.rs`; preserve the existing implementation and its evidence.

Before new code, current-main job `1792241038040-77137` on `078ce756d` ran `cargo run --locked --no-default-features -- --json batch article 22663026,22663028,22663029 --mode detail --sections fulltext --pdf` and exited 2 with `invalid_argument`, `unexpected argument '--pdf' found`.

Batch article full-text reads accept the PDF fallback through one process-wide provider limiter and existing page/size limits, and typed MCP article get accepts the same PDF option with per-article source/failure reporting.

Depends on: 2142
Depends on: 2147
Depends on: 2088
Depends on: 2159

- Starts from: existing single-article PDF options, batch dispatch, typed MCP mapping and the latest pushed implementation branch.
- Keeps: shared provider limits, bounded downloads/pages and named per-article failures.
- Changes: claims `src/sources/rate_limit.rs` `src/cli/system/mod.rs` `src/cli/system/batch.rs` `src/cli/system/dispatch.rs` `src/cli/system/tests.rs` `src/cli/commands.rs` `src/cli/list/literature.rs` `src/mcp/shell.rs` `src/mcp/shell/typed_get.rs` `src/mcp/shell/typed_get_tests.rs` `crates/biomcp-mcp-contract-client/src/lib.rs` `tests/article_batch_outcome.rs` `tests/rmcp_client_contract.rs` `spec/fixtures/setup-article-fulltext-source-fixture.sh` `spec/entity/article.md` `docs/user-guide/article.md` `docs/user-guide/cli-reference.md` `CHANGELOG.md`.
- Proof: reproduce the missing batch/typed option on current main. A three-article recorded batch exercises Semantic Scholar discovery through its actual limiter policy and returns each text or named failure; a typed MCP contract uses the PDF option. Publisher-download spacing is not proof of Semantic Scholar discovery pacing, and a fixture that bypasses its pacing cannot establish that claim. Run affected checks and the routine merged-candidate gate through build jobs.
- Defers: supplement extraction to 2152.

PM decision: 2026-10-17. Reuse existing implementation and close the specific limiter-proof gap before acceptance. Serialize overlapping article/MCP changes before 2152.
