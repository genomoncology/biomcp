---
priority: 1
---

# 2220 — Repair the repeat GWAS cache gate failure

Status: OPEN.

Milestone: 0.9.4

## Outcome

Explain and fix the repeat-search cache failure that stopped the unrelated Windows review landing. Preserve the outside-in claim that a same-day repeat search with an isolated cache returns the same rows without another provider request.

- Evidence: landing candidate2201 job1792353069557-23636 failed243.003s in `tests/test_gwas_day_cache_contract.py::test_a_repeat_gwas_search_inside_a_day_makes_no_request`: observed2 requests, expected1;1285 Python cases passed and10 skipped. Candidate source tree0f5c6dfed19faa01abaf17e826c04a8e794b3ba1. Main product3dc had previously passed its full check. Windows changes do not edit GWAS or cache production code.
- Changes: `tests/test_gwas_day_cache_contract.py` `tests/golden/replay_env.py` `tests/test_cache_persistent_size.py` `tests/test_http_cache_fail_soft.py` `tests/test_no_cache_managed_state.py` `tests/conftest.py` `tests/test_python_lane_cache_isolation.py` `tests/test_central_test_support_helpers.py` `CHANGELOG.md`. Reuse the existing isolated environment, central cache fixture and helper guard; no production cache policy changes.
- Proof: use the saved failing gate log, reproduce on current main through a focused build-machine job with the same artifact/environment where possible, then run the existing affected test after the correction. Distinguish a product defect from a leaky or scheduling-dependent test. Reuse qualified artifacts rather than repeat a full suite during diagnosis; one reviewed repair and landing qualification close the hold.

## Progress

- 2026-10-18 started

## Pre-code diagnosis

Both the official0.9.3 binary and the exact failed-gate0.9.4-dev binary reproduced two provider requests in the existing focused case on job2220/1792353589157-21420. Free disk194718138368/1995165736960bytes (9.76%) was below the product default10% minimum, so valid pressure eviction prevents this test from assuming retention. The test inherited HOME/config/cache policy. Preserve product eviction and the assertion; extract the existing shared replay command-environment body into a reusable function in its current module, delegate its existing caller, and use it for this case with a tiny explicitly bounded owned cache and one-byte minimum free-space policy. No new environment framework or product policy change.

PM ruling: fix the cache-retention test class raised by GitHub291, not only GWAS. Inventory found four Python retention consumers; migrate them to the shared isolated retained-cache policy and extend the existing guard against new handwritten retention environments. The central Python CLI cache fixture owns its disk floor. Existing Rust FixtureEnv/StaleNoteEnv and provider-network tests already pin one byte; the accepted Windows2201 candidate also pins one byte and its own maximum. Do not widen to no-cache consumers or injected production-policy tests.
