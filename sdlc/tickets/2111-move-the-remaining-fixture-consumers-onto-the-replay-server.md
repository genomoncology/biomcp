---
flow: build
priority: 3
---

# 2111: Move the remaining fixture consumers onto the replay server

Status: OPEN.

Milestone: 0.9.3

## Outcome

The named remaining HTTP fixture consumers use the existing replay server, and an obsolete setup/cleanup script is removed only after its final retained consumer moves.

Depends on: 2159

- Starts from: the existing replay infrastructure and named article-fulltext, CTGov intervention-alias, provider-contract and ComplexPortal conversion groups.
- Keeps: CLI/MCP behavior, refusals, source receipts, live canaries and their needed helpers, local study datasets and separately approved frozen-capture runners.
- Changes: claims `tests/article_asset_terminal.rs` `crates/biomcp-mcp-contract-client/src/lib.rs` `tests/test_ctgov_trial_search_detail_reuse.py` `tests/test_provider_contract_fixture.py` `tests/surface/test_complexportal_fixture_contract.py` `spec/fixtures/*article-fulltext*` `spec/fixtures/*ctgov-intervention*` `spec/fixtures/*provider-contract*` `spec/fixtures/*complexportal*` `spec/surface/cli.md` `tools/replay-server` `tests/golden/replay_env.py` `testdata/replay/**` `CHANGELOG.md`. Narrow each batch to exact files before editing; add a helper only when its final consumer truly moves.
- Proof: through the build job, affected existing command/MCP cases pass against recorded replay without missing requests, preserving counts, failures and refusals. No retained consumer refers to each deleted script; script-specific tests disappear with obsolete scripts. Run the routine merged-candidate gate without live canaries.
- Defers: local study-dataset provisioning and broad shared fixture identifiers to the next release.

PM decision: 2026-10-17. Study setup provisions files rather than an HTTP server and is outside this migration. Do not require deletion of every fixture outside live canaries. Reuse existing replay infrastructure and depend only on a particular recording batch when a named consumer actually needs it.
