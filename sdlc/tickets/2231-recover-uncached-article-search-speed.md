---
priority: 2
---

# 2231 — Recover uncached article search speed

Status: OPEN.

Milestone: 0.9.4

Reviews: revision 96e8831, accept

Reviews: revision 7b02037, accept

## Outcome

- Changes: `sdlc/records/2231-uncached-article-search.md` and this ticket during the study phase; no product edits are authorized before the exact source claim is declared.

Uncached `biomcp search article` is no slower than v0.9.2 on the same build machine, or the record names the cause and why it stays.

## Evidence

The 0.9.3 release measurement (`sdlc/records/2069-*`) shows `search article --gene BRAF` with `BIOMCP_CACHE_MODE=off` about 0.45 seconds slower than v0.9.2 across five paired samples, while the cached run is unchanged. Article timings in that record also carry PubTator metadata fallback failures, which may be part of the cost.

## Keeps

Keep every source the search queries today, its deadline and its result order. Do not trade correctness or honest error reporting for speed.

## Proof

Reproduce the gap on current main with alternating v0.9.2 and current samples and recorded load. Name the step that costs the time from a timing breakdown, not a guess. Fix it, then show the paired gap is closed. If the cause is a deliberate behavior, record it and close without a change.

## Development brief

Study and reproduce the uncached article-search gap before proposing source edits. Preserve sources, ordering, deadlines and honest errors. Use alternating comparable-profile v0.9.2 and current samples, with load and a timing breakdown. Do not compare an optimized old release with a debug new build. Start from current main 096c11fc416ddc18ee3b05814a6d0b9fede63648; PM record commit 4da16647b changes no product code. Report exact candidate source paths and a failing outside-in case before the coordinator declares any implementation claim. If the cost is intentional, report its evidence and close by record after review.

## Progress

- 2026-10-19 started
