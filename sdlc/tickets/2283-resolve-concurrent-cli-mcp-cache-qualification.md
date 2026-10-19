---
priority: 1
---

# 2283 — Resolve concurrent CLI MCP cache qualification

Status: OPEN.

Milestone: 0.9.4

## Outcome

Final qualification proves concurrent initialized MCP and CLI calls return correct data, share the supported SQLite cache and reuse admitted responses. Diagnose the failing one-entry observation before deciding whether the defect belongs to production or to a test that assumes every best-effort concurrent write must succeed.

Changes: `tests/test_cache_sqlite.py`, `CHANGELOG.md`, `sdlc/records/2283-concurrent-cache-qualification.md`.

## Evidence

Final Linux job 2233/1792451222629-265544 on source 51f5e03b218c87cedc82bd9e1a67552bd5ab9df8, tree 90d9f94dfa5541a135bc728d88f849e595537d15, failed test_initialized_mcp_and_cli_share_cache_during_concurrent_requests at tests/test_cache_sqlite.py:146. Both requests returned results and SQLite integrity_check passed, but the database contained one entry rather than two. Routine Rust passed 4279 with 34 declared skips. Python passed 1587, failed one and skipped five. Strict docs and full-feature stages did not run after failure. No source file changed. Retained original log SHA256 is 3d027914386627aa22db3c524ff1779c59aba2c13e4e231677562f311c35db2c.

Private original evidence remains on the qualifying host in experiments/170-biomcp-2233-linux-offline-evidence/final-1792451222629-265544, with commands, XML, source hashes, isolation proof, job state and output. The original per-case database may have been removed by fixture teardown; verify this limit. Ticket 2275 already diagnosed a different test in this class: permitted concurrent cache-admission bypass does not prove broken warm reuse. That precedent is a hypothesis, not the cause of this failure.

## Keeps

Before code, restate the outcome and reproduce the actual assertion contract on current main with the existing controlled contention mechanisms. Inspect both returned identities, exact distinct request/cache keys, persistence and warm reuse. Preserve returned-data, integrity, isolation, initialized-process concurrency and true cache-sharing assertions. Never replace the entry assertion with a loose count or add sleeps/retries to hide uncertainty. Do not change runtime cache policy, default thresholds, provider data or clinical identity in this test claim.

If deterministic evidence proves a permitted cold admission bypass, use a bounded uncontended admission phase and keep strict warm reuse and distinct-key assertions that still reject real cache-sharing or key-collision defects. Audit only the related SQLite/concurrent cache proofs for the same assumption; do not build a new test framework or broaden into a fixture refactor. If a runtime defect or shared helper change is needed, report the cause and exact proposed additional paths before editing outside this claim. No live provider requests.

## Proof

Retain the original failure independently. Use one bounded focused diagnosis through pm job, controlled expected-red and corrected-green proof, and affected tests plus required lint. Prefer existing controls and unchanged qualified artifacts. Run focused Linux proof when necessary for the observed platform, also through pm job; this is authorized occasional Linux validation, not another full qualification run. Show the corrected proof still fails for genuine reuse failure or key collision. Final complete Linux qualification follows after reviewed repair lands. Push once only after green and stop for fresh review; no full gate, tag or landing by the developer.

Keep: experiments/170-biomcp-2233-linux-offline-evidence — retain the untouched final failure that raised this repair and its initial comparison evidence.

## Progress

- 2026-10-19 started

## Coordinator verification

Fresh reviewer /root/review_concurrent_cache_2283 accepted f64031106cdf84e604fa31e4715bd40ec8287675 without findings. The coordinator read the complete product-test diff and retained green job log, checked the test SHA256 against the tested patch, and confirmed the clean branch. Job 2283/1792454062120-44369 passed 29 affected cases and required focused checks. Controlled bypass and key-collision failures remain retained. PM explicitly approved best-effort admission and this test-only correction; final Linux qualification must rerun on the new main tip. The original failure cause remains unknown. This is the only ready repair and lands alone so qualification can resume.

Keep: experiments/2103-biomcp-2283-concurrent-cache — controlled diagnosis, corrected proof, rejected mutations, databases, exact source and job evidence.

Keep: experiments/172-biomcp-2283-concurrent-cache — cancelled ticket-owned queued Linux attempt; no build or test ran.
