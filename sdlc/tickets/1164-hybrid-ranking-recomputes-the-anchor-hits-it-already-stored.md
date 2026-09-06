---
flow: build
priority: 3
deps: []
---

# Hybrid ranking recomputes the anchor hits it already stored

## Goal

Hybrid article ranking performs the complete anchor-hit calculation exactly
once per candidate. Today it calculates and publishes lexical metadata, then
rescans the same title, abstract, and anchors to recover the full-width union
count for scoring.

Reconfirmed on current main `e95bb7a4` and ticket 1150 commit `6e1e4987`:
`rank_articles_hybrid` in `src/entities/article/ranking.rs` calls
`populate_lexical_ranking_metadata`, which invokes `lexical_anchor_hits` once
per row and stores saturated public counters. Hybrid then rebuilds the anchor
set and calls `lexical_anchor_hits` a second time per row. Ticket 1150 changes
neither `ranking.rs` nor its hybrid tests, so 1150 is not a dependency.

Record 1102's scoring rule remains correct: lexical score is the full-width
union coverage ratio, not a ratio reconstructed from public `u8` counters that
saturate at 255. The duplicate calculation must be removed without losing the
260-of-300 regression.

## Private calculation contract

Keep the implementation in `src/entities/article/ranking.rs`. Replace the
metadata-only calculation with one private, non-serializable result containing
exactly:

```text
LexicalRankingCalculation {
    metadata: ArticleRankingMetadata,
    anchor_count: usize,
    union_hits: usize,
}
```

One call to `lexical_anchor_hits` constructs this result. `metadata` retains the
existing directness tier, rescue facts, booleans, and saturated `u8`
`anchor_count`, title, abstract, and combined counters. The two private `usize`
values preserve the unsaturated denominator and union numerator used by hybrid
scoring. They are not added to `ArticleCandidate`, `ArticleSearchResult`, or
`ArticleRankingMetadata`, and never serialize.

Build the anchor set once. In the existing metadata-population pass, assign
each result's public metadata to its candidate and push its private
`(anchor_count, union_hits)` into a vector in that same row order. Assert that
the vector length equals the candidate slice length. For hybrid ranking, do not
filter, reorder, insert, remove, or sort candidates between that pass and
`rows.iter_mut().zip(private_counts)`. Consume each private pair exactly once
at that zip point, calculate `0.0` when `anchor_count == 0`, otherwise calculate
`union_hits as f64 / anchor_count as f64`, then set the existing component and
composite scores. Sorting happens only after every aligned score is committed.

Lexical and semantic modes may discard the returned private counts; their
metadata and sort behavior remain unchanged. `lexical_anchor_hits` itself still
checks title, abstract, and their union for each anchor. This ticket removes the
second complete calculation, not those established internal checks.

## Observable contract and proof

- Hybrid invokes the complete anchor-hit calculator exactly `rows.len()` times,
  including zero calls for an empty candidate slice and one call for each
  duplicate candidate. It contains no second `lexical_anchor_hits` call and no
  equivalent title/abstract anchor scan in the scoring loop.
- Add a private injected `FnMut` calculator seam, used by the production wrapper
  with `lexical_anchor_hits`, so a deterministic test-local counter can prove
  the exact hybrid call count without a process-global atomic or timing
  benchmark. Code review also verifies that all hybrid hit calculation passes
  through this seam.
- Extend the existing hybrid owner
  `src/entities/article/ranking/tests/calibration/hybrid.rs`. Keep the
  260-of-300 test exact: public counters remain 255 while lexical score remains
  exactly `260.0 / 300.0`. Cover empty anchors, title/abstract overlap,
  duplicates, empty input, component scores, composite scores, and stable final
  ordering.
- Existing ranking JSON, Markdown `Why` text, `partial_query_match` warning,
  PubMed rescue, lexical and semantic modes, tie-breaking, pagination, and
  deduplication remain byte-for-byte unchanged. No provider, cache, request,
  retry, or concurrency path changes.

## Ownership, gates, and boundary

Production edits belong only in `src/entities/article/ranking.rs`; behavior
tests belong in the existing hybrid sidecar above. Keep `ranking.rs` at or below
575 lines and the hybrid sidecar at or below 525 lines. Do not raise the global
Rust source-size threshold or add an inventory allowance. Add no dependency,
new packaged path, benchmark, public type, feature, or documentation page; the
package remains exactly 1,300 paths.

Run the focused ranking module and hybrid sidecar tests, the record-1102
full-width and output/warning regressions, `cargo clippy --no-default-features
--all-targets -- -D warnings`, the source-size/package ratchets, and
`git diff --check`, followed by `make lint`, `make test`, and `make spec`.

## Boundary

This ticket removes duplicated counting inside hybrid ranking. It does not
change weights, formulas, anchor construction or matching, warnings,
standalone ranking modes, PubMed rescue, public metadata, or any article search
surface. Implementation begins only after independent design acceptance; code
review must inspect count alignment and consumption, the deterministic exact
call-count proof, public serialization neutrality, file caps, package count,
and the complete diff for unrelated edits.

## Review

Accepted after independent design review. The reviewer confirmed the
deterministic one-calculation-per-row proof, the aligned private full-width
carrier, removal of the unsupported 1150 dependency, and the stated ownership,
source-size, package, and gate constraints.
