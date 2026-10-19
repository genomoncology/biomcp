---
priority: 1
---

# 2275 — Make parallel cache reuse proof respect admission

Status: OPEN.

Milestone: 0.9.4

## Outcome

Changes: `tests/test_http_cache_fail_soft.py`.

The parallel cold-start cache test proves correct returned data and reuse of admitted responses without requiring best-effort cache writes to succeed during contention. Keep production cache behavior and its bounded failure policy unchanged.

## Evidence

Nine-ticket candidate gate 2264/1792442282819-60438 failed only test_parallel_article_sources_fill_cold_cache_then_reuse_answers[command1], with a repeated Europe PMC request. All Rust and 1556 Python tests passed; actual main 4c27c60 remains green. A single bounded reproduction, 2268/1792442816783-59042, passed and retained nineteen entries plus correct warm reuse. Original failed case artifacts were removed by fixture teardown, so the original cause is unknown. Source inspection shows cache operations intentionally bypass after bounded SQLite contention; existing contention controls explicitly permit refetch after skipped writes. This makes admission versus reuse a named uncertainty, not proof of a runtime cache defect.

Keep: experiments/2090-biomcp-2268-abstract-recovery — retain the bounded diagnosis, cache snapshots, request log and artifact hashes under cache-reuse-diagnosis.

## Keeps

Reproduce the contract problem on current main using the existing deterministic contention controls before editing. Preserve all returned-data, isolation and true warm-reuse assertions. Do not weaken strict replay, change expected provider data, add retry loops or sleeps, or alter production cache policy. No new checker or admission mechanism. If deterministic evidence does not support the mismatch, report that instead of inventing a fix.

A small valid design may separate parallel startup from a single uncontended admission phase, followed by one identical warm call that must reuse the admitted responses. Verify why admission is reliable in that phase. The test must still fail when cache reuse is genuinely broken.

## Proof

A controlled case on current main demonstrates the old proof confuses an allowed bypass with failed reuse. The corrected outside-in case passes for the supported cache policy and still detects a genuine reuse failure. Run affected cache cases and required lint through the configured build job; no full gate. Record exact limitations, including the unrecovered original cause.
