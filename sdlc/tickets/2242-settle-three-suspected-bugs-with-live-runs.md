---
priority: 2
---

# 2242 — Settle three suspected bugs with live runs

Status: OPEN.

Milestone: 0.9.4

## Outcome

Each of three suspected user-facing bugs is either shown fixed on current main or fixed in this ticket.

## Evidence

1. `sdlc/issues/2026-09-28-aspirin-mac-run-never-tested-the-single-ingredient-anchor.md`: `biomcp drug interactions aspirin` may resolve plain aspirin to a five-ingredient combination product and report "not covered".
2. Finding 4 of `sdlc/issues/2026-09-09-findings-from-the-2026-09-01-interface-study-with-no-ticket.md`: `biomcp discover "H3K27M"` suggested `get drug "h3k27me3"`, a histone mark offered as a drug, and suggestions favor the first synonym over the term typed.
3. `sdlc/issues/2026-10-07-diagnostic-synonym-provenance-fails-spec-contracts.md`: a GTR read returned zero rows under its fixture and the spec block was deleted with no replacement.

## Keeps

Fix only what a run shows broken. Clinical judgment about the right drug or entity goes to a clinical reviewer.

## Proof

One live opt-in run per item on current main with synced bundles, recorded. For each broken item, a replay or fixture test that fails first and passes after the fix. Mark each issue resolved with its outcome.
