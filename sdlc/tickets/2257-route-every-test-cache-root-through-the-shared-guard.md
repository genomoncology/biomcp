---
priority: 3
---

# 2257 — Route every test cache root through the shared guard

Status: COMPLETE.

Milestone: 0.9.4

Reviews: revision dcf5b3d, accept

Reviews: revision c905fd2, accept

Reviews: revision 8190e51, accept

Reviews: revision 626be1f, accept

## Outcome

Changes: `tests/test_slow_test_reporting.py`, `tests/surface/test_biomcp_verify_live.py`, `tools/dead-code-exceptions.json`, `tests/test_replay_server_lifetime.py`, `tests/test_prepare_spec_artifacts.py`, `src/sources/mod.rs`, `src/cli/article/tests/outage.rs`, `src/entities/drug/get/tests.rs`, `src/entities/drug/test_support.rs`, `CHANGELOG.md`, `crates/biomcp-mcp-contract-client/src/lib.rs`, `scripts/run-specs.sh`, `sdlc/tickets/2257-route-every-test-cache-root-through-the-shared-guard.md`, `spec/entity/article-assets-live.md`, `spec/entity/drug.md`, `spec/entity/gene.md`, `spec/entity/variant.md`, `spec/fixtures/cache-policy.sh`, `spec/fixtures/render-article-fulltext-jats-markdown.sh`, `spec/fixtures/run-article-semanticscholar-source-search.sh`, `spec/fixtures/run-variant-article-entity-fixture.sh`, `spec/fixtures/run-variant-article-identity-fixture.sh`, `spec/fixtures/run-variant-articles-live-canary.sh`, `spec/fixtures/setup-article-federated-timeout-fixture.sh`, `spec/fixtures/setup-article-fulltext-source-fixture.sh`, `spec/fixtures/setup-article-search-deadline-fixture.sh`, `spec/fixtures/setup-europepmc-cursor-fixture.sh`, `src/cli/article/tests/entities_disease_links.rs`, `src/cli/discover.rs`, `src/cli/stale_json_note_tests.rs`, `src/cli/system/batch.rs`, `src/cli/system/tests/dev2_contracts.rs`, `src/cli/tests/printed_card_commands.rs`, `src/entities/article/assets.rs`, `src/entities/article/backends/tests.rs`, `src/entities/article/detail/tests.rs`, `src/entities/article/enrichment/tests.rs`, `src/entities/article/fulltext.rs`, `src/entities/article/graph/tests.rs`, `src/entities/article/graph/tests/admission.rs`, `src/entities/article/graph/tests/cache.rs`, `src/entities/article/search/tests/deadline.rs`, `src/entities/article/search/tests/relevance_paging.rs`, `src/entities/article/test_support.rs`, `src/entities/article/variant_search.rs`, `src/entities/author/papers.rs`, `src/entities/disease/get/tests.rs`, `src/entities/drug/search/tests.rs`, `src/entities/variant/clinvar.rs`, `src/entities/variant/structure/tests.rs`, `src/mcp/shell.rs`, `src/mcp/shell/typed_get_tests.rs`, `src/sources/gwas/tests/day_cache.rs`, `src/sources/opencitations.rs`, `src/sources/tests/provider_network.rs`, `src/sources/who_pq/tests/sync.rs`, `src/test_support.rs`, `tests/article_batch_outcome.rs`, `tests/article_usage_stderr.rs`, `tests/cache_clear.rs`, `tests/cache_fixture.py`, `tests/conftest.py`, `tests/disable_sources_cli.rs`, `tests/golden/replay_env.py`, `tests/json_error_contract.rs`, `tests/live_smoke.rs`, `tests/managed_state_permissions.rs`, `tests/pty_helpers.rs`, `tests/rmcp_client_contract.rs`, `tests/surface/test_semantic_scholar_retry_after_contract.py`, `tests/test_author_surface_docs.py`, `tests/test_biomcp_ci_contract.py`, `tests/test_central_test_support_helpers.py`, `tests/test_ctgov_trial_search_detail_reuse.py`, `tests/test_no_cache_managed_state.py`, `tests/unit/cli/variant.rs`, `tests/windows_stdio_contract.rs`, `tools/biomcp-ci`.

No test creates its own cache directory outside the shared fixture guard, so every test gets the same disk-floor and cache policy, and a check refuses new ones.

## Evidence

Coordinator scope grant: remove the seven redundant borrows before existing format! expressions in the three added test-only paths required by the shared root setter signature; preserve assertions, provider behavior and production code.

The same failure has occurred twice: 2220 and 2255 each traced an intermittent failure on a nearly full build machine to a test fixture with its own cache root that bypassed the shared policy pinning `BIOMCP_CACHE_MIN_DISK_FREE`. Each repair fixed one fixture.

## Keeps

Change test fixtures only; no product change. Keep every test assertion. No repository-wide test refactor beyond moving cache roots onto the shared guard.

## Proof

An inventory lists every test that sets a cache directory and shows each uses the shared guard after the change. A check in the existing quality checks fails on a new test that sets a cache directory without it.

## Fixture migration boundary

Use a dedicated shared Rust set_cache_root helper, the existing dev-only fixture crate for child cache environment pairs, a shared Python root setter, and one shell fixture policy helper. Preserve explicit policy probes and passed cache configurations; controlled measurement arms and their harness method stay unchanged. Replace independent wrapper mutation/restoration only where equivalent; otherwise retain its provider guard and hold the shared cache guard separately. Keep every assertion and intentional override. The existing central-helper quality check must reject a new uncontrolled root assignment, with narrow exceptions only for the actual shared helper definitions. Complete the fixture-only groups as one coherent change with one final green source push.

Keep: experiments/802-biomcp-2257-cache-fixture-migration — retain the full classified inventory, controlled baseline and accepted-batch failures, owning red/green results, commands and final source hashes.

Coordinator fixture-only grant: baseline f63ba97 reproduced the FDA cache-row loss with available disk below its explicit ten-percent helper default. Authorize only cfg(test) test_cache_config in src/sources/mod.rs to use the same shared numeric disk-floor constant as FixtureEnv root policy; preserve per-test overrides, size, age, origins, all assertions and production configuration. Retain baseline and disk observations in experiment 802.

Coordinator dependency grant: the two copied-runner harnesses may copy the actual spec/fixtures/cache-policy.sh alongside the actual scripts/run-specs.sh. Preserve synthetic methods and assertions; retain owning red/green evidence.

## Progress

- 2026-10-19 started

## Implementation evidence

Developer stopped at cf6acb525eb89deaee068b3a61adf77b398126db, published once and clean. Shared fixture defaults now cover Rust environment and explicit config helpers, child environments, Python and executable shells without changing production defaults, deliberate overrides, assertions or measurement methods. All 254 inventory rows are classified in the retained archive; the archived manifest matches all 73 changed source files.

Focused proof on the configured build host passed final formatting and all-target warnings-denied Clippy; 127 remaining or affected Rust checks including both FDA cases, with 670 earlier passing checks retained; 84 Python checks, 32 audit and copied-runner checks, one replay-root check, four local shell fixtures and three setup/export smokes. No full routine or release gate, live-provider, Windows or Linux offline claim. The actual landing and main gates follow fresh review.

Coordinator owning-audit repair grant: landing job 2257/1792432359521-28124 stopped before tests because tools/dead-code-exceptions.json retains the removed src/cli/stale_json_note_tests.rs field root: crate::test_support::TempDirGuard,. Remove only that stale registration after verifying the migrated shared guard makes the original allowance obsolete. Preserve all remaining registrations and audit enforcement. Retain complete failed gate and owning corrected audit proof in experiment 802.

## Discovery failure

Gate 2257/1792432639728-12031 passed lint, compiled and archived the test binaries, then stalled in the unit binary's --list --format terse invocation before running Rust cases. The coordinator observed that child sleeping for more than five minutes with 0.02 CPU seconds, retained both lane logs and process identity under experiment 802/discovery-stall, and stopped only the verified job process group. The gate exited 143; this is cancelled discovery proof, not a passing or slow-case run. A fresh developer owns one narrow startup/discovery repair within the declared fixture paths; preserve every test assertion and production behavior.

## Discovery diagnosis

Fresh diagnostic job 2257/1792433189245-24862 made no source changes or rebuilds. The retained archived unit binary listed all 4,119 tests; the original binary produced identical output; fresh cargo-nextest archive extraction and discovery passed. A sample of a fresh archive child showed a synchronous operating-system loader notification wait before Rust startup, and signature verification passed. The cancelled original child was not sampled, so its precise cause remains unproven; no fixture-code cause was demonstrated and no speculative source repair was made. Archive 802/discovery-repair retains raw outputs, sample, hashes and exited-process proof. Resume the landing after this successful exact-artifact discovery prerequisite; do not claim the cancelled run passed.

## Copied helper dependencies

Resumed gate 2257/1792433343478-49899 passed all 4,266 routine Rust cases; Python reported 1,390 passed and nine failures. Eight slow-warning harness cases copy conftest without its new cache_fixture module; one NIH verification harness copies the specification runner without its shared shell helper. Grant only tests/test_slow_test_reporting.py and tests/surface/test_biomcp_verify_live.py to copy the actual shared dependency alongside the actual copied consumer. Preserve every assertion, status classification and synthetic method. Retain the full failed gate and focused owning red/green under archive 802; no new fixture-policy copy or weakened import/runner behavior.
