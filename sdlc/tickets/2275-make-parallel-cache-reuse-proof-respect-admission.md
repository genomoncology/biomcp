---
priority: 1
---

# 2275 — Make parallel cache reuse proof respect admission

Status: OPEN.

Milestone: 0.9.4

Reviews: revision fe3de71, accept

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

## Progress

- 2026-10-19 started

## Implementation evidence

Developer 331fc49ef55f16c9d53754feba51d9bd16384d80 changes only the claimed cache test (+8/-1). Current-main expected-red proof 2275/1792443253344-87619 uses existing deterministic SQLite contention: correct cold data, zero admitted responses, release lock, then the original assertion rejects the permitted refetch. The original batch failure cause remains unknown. Initial harness attempt 1792443238120-86025 had a Python scoping error and is not evidence. Green 1792443291780-92328 passes the corrected zero-admission control, rejects forced final warm bypass, and passes all sixteen affected cache cases plus Ruff, tracked-text and quality checks. After the cold CLI exits, one uncontended call into its private cache admits skipped responses, then the original warm request-reuse assertion remains strict. No runtime cache behavior changes. Checked file SHA256 a2b99c8fb9cfb50cb31aa7d5ed751f944c57854441111a9118d351f565f07875 matches the pushed file. Existing private experiment 2090 cache-reuse-diagnosis retains all proof. No full gate ran.
