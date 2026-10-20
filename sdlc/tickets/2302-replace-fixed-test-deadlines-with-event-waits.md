---
priority: 2
---

# 2302 — Replace fixed test deadlines with event waits

Status: OPEN.

Milestone: 0.9.4

Depends on: 2300

Reviews: revision b7f06f3, accept

Reviews: revision e2b32da, accept

## Outcome

Changes: `src/cli/stale_json_note_tests.rs`, `tests/test_cache_sqlite.py`, `tests/test_mcp_http_surface.py`, `tests/test_mcp_stdio_guidance.py`, `tests/test_mcp_2026_protocol.py`, `tests/test_replay_server_lifetime.py`, `tests/test_routine_cargo_feature_contract.py`, `tests/test_no_cache_managed_state.py`, `tests/test_cache_invalid_settings.py`, `tests/test_variant_article_disabled_sources.py`, `tests/test_variant_article_transcript_aliases.py`, `tests/test_variant_article_gene_aliases.py`, `CHANGELOG.md`, `sdlc/records/2302-event-waits.md`, `sdlc/records/2300-shared-cache-reuse-contract.md`, `sdlc/tickets/2300-share-the-best-effort-cache-reuse-helper.md`, `tests/golden/cache_reuse.py`, `tests/test_http_cache_fail_soft.py`.

Subprocess and server tests wait on the real event (readiness line, exit, reaped child, forwarded signal) under one generous outer test timeout, as 2298 does, instead of short fixed deadlines. The 14 remaining short deadlines across 7 test files, including test_cache_sqlite.py, test_mcp_http_surface.py and test_mcp_stdio_guidance.py, change.

The five freshness sleeps in `src/cli/stale_json_note_tests.rs` also change for 0.9.4. Age the isolated stored HTTP policy explicitly after the fresh-response assertions, preserving production cache policy, real I/O and all stale-note assertions.

- Proof: Each changed event wait passes with a controlled pause injected before its wait, and still fails when the awaited event never happens. Freshness cases establish their stale-policy premise without sleeping; an omitted-age negative control checks that premise without assuming a quiet host. The GWAS case retains its distinct day-cache behavior.

## Evidence

2298, 2178 and 2175 failed full runs on fixed deadlines that a correct program missed when the host stalled. See the root cause record `sdlc/records/2299-root-cause-of-test-only-check-failures.md`.
