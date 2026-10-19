---
priority: 2
---

# 2231 — Recover uncached article search speed

Status: OPEN.

Milestone: 0.9.4

## Outcome

Uncached `biomcp search article` is no slower than v0.9.2 on the same build machine, or the record names the cause and why it stays.

## Evidence

The 0.9.3 release measurement (`sdlc/records/2069-*`) shows `search article --gene BRAF` with `BIOMCP_CACHE_MODE=off` about 0.45 seconds slower than v0.9.2 across five paired samples, while the cached run is unchanged. Article timings in that record also carry PubTator metadata fallback failures, which may be part of the cost.

## Keeps

Keep every source the search queries today, its deadline and its result order. Do not trade correctness or honest error reporting for speed.

## Proof

Reproduce the gap on current main with alternating v0.9.2 and current samples and recorded load. Name the step that costs the time from a timing breakdown, not a guess. Fix it, then show the paired gap is closed. If the cause is a deliberate behavior, record it and close without a change.
