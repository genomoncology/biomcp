---
priority: 3
---

# 2257 — Route every test cache root through the shared guard

Status: OPEN.

Milestone: 0.9.4

## Outcome

Changes: `CHANGELOG.md`, `crates/biomcp-mcp-contract-client/src/lib.rs`, `scripts/run-specs.sh`, `sdlc/tickets/2257-route-every-test-cache-root-through-the-shared-guard.md`, `spec/entity/article-assets-live.md`, `spec/entity/drug.md`, `spec/entity/gene.md`, `spec/entity/variant.md`, `spec/fixtures/cache-policy.sh`, `spec/fixtures/render-article-fulltext-jats-markdown.sh`, `spec/fixtures/run-article-semanticscholar-source-search.sh`, `spec/fixtures/run-variant-article-entity-fixture.sh`, `spec/fixtures/run-variant-article-identity-fixture.sh`, `spec/fixtures/run-variant-articles-live-canary.sh`, `spec/fixtures/setup-article-federated-timeout-fixture.sh`, `spec/fixtures/setup-article-fulltext-source-fixture.sh`, `spec/fixtures/setup-article-search-deadline-fixture.sh`, `spec/fixtures/setup-europepmc-cursor-fixture.sh`, `src/cli/article/tests/entities_disease_links.rs`, `src/cli/discover.rs`, `src/cli/stale_json_note_tests.rs`, `src/cli/system/batch.rs`, `src/cli/system/tests/dev2_contracts.rs`, `src/cli/tests/printed_card_commands.rs`, `src/entities/article/assets.rs`, `src/entities/article/backends/tests.rs`, `src/entities/article/detail/tests.rs`, `src/entities/article/enrichment/tests.rs`, `src/entities/article/fulltext.rs`, `src/entities/article/graph/tests.rs`, `src/entities/article/graph/tests/admission.rs`, `src/entities/article/graph/tests/cache.rs`, `src/entities/article/search/tests/deadline.rs`, `src/entities/article/search/tests/relevance_paging.rs`, `src/entities/article/test_support.rs`, `src/entities/article/variant_search.rs`, `src/entities/author/papers.rs`, `src/entities/disease/get/tests.rs`, `src/entities/drug/search/tests.rs`, `src/entities/variant/clinvar.rs`, `src/entities/variant/structure/tests.rs`, `src/mcp/shell.rs`, `src/mcp/shell/typed_get_tests.rs`, `src/sources/gwas/tests/day_cache.rs`, `src/sources/opencitations.rs`, `src/sources/tests/provider_network.rs`, `src/sources/who_pq/tests/sync.rs`, `src/test_support.rs`, `tests/article_batch_outcome.rs`, `tests/article_usage_stderr.rs`, `tests/cache_clear.rs`, `tests/cache_fixture.py`, `tests/conftest.py`, `tests/disable_sources_cli.rs`, `tests/golden/replay_env.py`, `tests/json_error_contract.rs`, `tests/live_smoke.rs`, `tests/managed_state_permissions.rs`, `tests/pty_helpers.rs`, `tests/rmcp_client_contract.rs`, `tests/surface/test_semantic_scholar_retry_after_contract.py`, `tests/test_author_surface_docs.py`, `tests/test_biomcp_ci_contract.py`, `tests/test_central_test_support_helpers.py`, `tests/test_ctgov_trial_search_detail_reuse.py`, `tests/test_no_cache_managed_state.py`, `tests/unit/cli/variant.rs`, `tests/windows_stdio_contract.rs`, `tools/biomcp-ci`.

No test creates its own cache directory outside the shared fixture guard, so every test gets the same disk-floor and cache policy, and a check refuses new ones.

## Evidence

The same failure has occurred twice: 2220 and 2255 each traced an intermittent failure on a nearly full build machine to a test fixture with its own cache root that bypassed the shared policy pinning `BIOMCP_CACHE_MIN_DISK_FREE`. Each repair fixed one fixture.

## Keeps

Change test fixtures only; no product change. Keep every test assertion. No repository-wide test refactor beyond moving cache roots onto the shared guard.

## Proof

An inventory lists every test that sets a cache directory and shows each uses the shared guard after the change. A check in the existing quality checks fails on a new test that sets a cache directory without it.

## Fixture migration boundary

Use a dedicated shared Rust set_cache_root helper, the existing dev-only fixture crate for child cache environment pairs, a shared Python root setter, and one shell fixture policy helper. Preserve explicit policy probes and passed cache configurations; controlled measurement arms and their harness method stay unchanged. Replace independent wrapper mutation/restoration only where equivalent; otherwise retain its provider guard and hold the shared cache guard separately. Keep every assertion and intentional override. The existing central-helper quality check must reject a new uncontrolled root assignment, with narrow exceptions only for the actual shared helper definitions. Complete the fixture-only groups as one coherent change with one final green source push.

Keep: experiments/491-biomcp-2257-cache-fixture-inventory — retain the original static inventory until its complete contents are verified in the execution evidence archive.
