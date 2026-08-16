---
flow: quickfix
priority: 10
---

# Restore the opt-in verification lane

`make verify` is deterministically red before it can give an honest live-provider verdict: MCP client assertions still expect compacted wording in the raw tool description, the CLI lane contract contradicts its own registry, the variant strict-query canary assumes only one normalized alias, the credentialed UMLS check runs through a wrapper that removes its credential, and the NIH Reporter mode runs a fixture-dependent routine page without its fixtures. Align these checks with the shipped contracts while preserving the routine/live split and fail-closed product errors.

The owning paths are `crates/biomcp-mcp-contract-client/src/lib.rs`, `scripts/run-specs.sh`, `spec/surface/cli.md`, `spec/surface/discover-live.md`, `spec/entity/variant-articles-live.md`, `spec/fixtures/run-variant-article-strict-live-canary.sh`, and a dedicated NIH Reporter live page. Existing assertions in `tests/test_variant_article_live_canary.py`, `tests/test_biomcp_ci_contract.py`, `tests/surface/test_parallel_isolation_contract.py`, and related spec-contract tests may be restated to prove the corrected ownership.
