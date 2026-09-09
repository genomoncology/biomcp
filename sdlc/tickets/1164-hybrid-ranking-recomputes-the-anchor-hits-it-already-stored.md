---
flow: build
priority: 3
deps: []
---

# Hybrid ranking calculates each candidate's anchor hits once

## Outcome

Hybrid article ranking performs the complete title, abstract, and union anchor-hit
calculation exactly once per candidate and reuses that result for both public
ranking metadata and the lexical score. Public metadata, scores, ordering,
warnings, and rendered meaning remain unchanged.

## Current facts

Reverified on `origin/main` at `a068fabd`, after ticket 1148 integration.
`rank_articles_hybrid` in `src/entities/article/ranking.rs` first calls
`populate_lexical_ranking_metadata`. That pass builds the anchor set and calls
`lexical_anchor_hits` once for every candidate through
`lexical_ranking_metadata`. Hybrid then builds the same anchor set again and
calls `lexical_anchor_hits` a second time for every candidate in its scoring
loop. Ticket 1148 changed gene exact-symbol ordering only; it did not change
article ranking or the hybrid tests. There is no implementation dependency.

The second result is used only for `combined_hits / anchors.len()`. It cannot be
reconstructed from `ArticleRankingMetadata`: its public anchor and hit counters
are `u8` values that saturate at 255. The existing
`hybrid_coverage_uses_full_counts_before_public_counters_saturate` regression
therefore correctly requires public values of 255 and a lexical score of
exactly `260.0 / 300.0`.

At this base, `src/entities/article/ranking.rs` is 552 lines and
`src/entities/article/ranking/tests/calibration/hybrid.rs` is 485 lines. The
existing ticket-local ceilings remain 575 and 525 lines respectively.
The source package contains exactly 1,300 paths. The repository zero-coupling
checker passes on this base.

## Design

Keep the production change inside `src/entities/article/ranking.rs`. Introduce
one private, non-serializable calculation value:

```text
LexicalRankingCalculation {
    metadata: ArticleRankingMetadata,
    anchor_count: usize,
    union_hits: usize,
}
```

Build the anchor set once for a ranking operation. For each candidate, call one
calculator that performs the established title-hit, abstract-hit, and union-hit
checks and returns the calculation above. Assign `metadata` to the candidate and
retain the two full-width values in candidate order for hybrid scoring. Do not
add either full-width value to `ArticleCandidate`, `ArticleSearchResult`, or
`ArticleRankingMetadata`, and do not serialize them.

Make the metadata-population helper accept a private injected `FnMut` calculator
and return the ordered full-width calculations. The production wrappers pass
`lexical_anchor_hits`. Lexical and semantic modes may discard the private
counts after publishing the same metadata. Hybrid must assert equal candidate
and calculation lengths, then consume exactly one calculation beside each
candidate without filtering, inserting, removing, reordering, or sorting
between production and consumption. Compute lexical score as `0.0` when the
full-width `anchor_count` is zero and otherwise as
`union_hits as f64 / anchor_count as f64`. Commit every component and composite
score before the existing final sort.

The injected seam is private and exists only to make call count deterministic;
do not use a process-global atomic, timing benchmark, public hook, feature, or
dependency. `lexical_anchor_hits` continues to check title, abstract, and their
union for every anchor. This ticket removes only the duplicate complete
per-candidate calculation.

## Scope and exclusions

Production edits are limited to `src/entities/article/ranking.rs`; focused test
edits are limited to
`src/entities/article/ranking/tests/calibration/hybrid.rs`. Do not change
weights, formulas, anchor construction or matching, candidate admission,
deduplication, pagination, PubMed rescue, tie-breaking, standalone lexical or
semantic behavior, providers, cache, requests, retries, concurrency, public
types, metadata fields, warnings, renderers, docs, Cargo files, or packaged
paths.

Serialization and MCP wrapping are untouched by this ownership boundary. This
ticket preserves the ranking values they receive, but does not promise a new
byte-level compatibility contract for every output surface. New per-surface
snapshots would test unchanged serializers rather than the duplicated ranking
calculation and are not warranted.

Keep `ranking.rs` at or below 575 lines and the hybrid sidecar at or below 525
lines. Do not raise a source-size threshold or inventory allowance. Keep the
package at exactly 1,300 paths and keep the repository's removed trial-crate
boundary at zero coupling.

## Acceptance and red/green proof

1. Add `hybrid_calculates_anchor_hits_once_per_candidate` in the existing
   hybrid sidecar. It calls the private hybrid helper with an injected closure
   that increments a local counter and delegates to `lexical_anchor_hits`. Use
   distinct candidates plus two duplicate candidates so duplicate values are
   still separate work items. First route both existing calculations through
   the seam without removing either: the assertion `calls == rows.len()` fails
   deterministically with `2 * rows.len()`. After the production change it
   passes with exactly one call per candidate. In the same test, an empty slice
   reports zero calls. No atomic, clock, sleep, or benchmark is permitted.
2. Keep `hybrid_coverage_uses_full_counts_before_public_counters_saturate`
   exact: both public counters remain 255 and lexical score remains
   `260.0 / 300.0`. Keep the existing overlap, zero-anchor, component,
   composite, semantic-source, custom-weight, tie-order, and standalone-mode
   tests green.
3. Extend one existing hybrid regression as the representative
   ranker-to-rendered-output proof. Use the actual `ArticleSearchResult` page
   returned by `finalize_article_candidates` in
   `hybrid_lexical_coverage_beats_a_high_citation_one_anchor_match`, pass those
   rows to the production Markdown article-search renderer, and assert the
   stable `Why` rationale produced from the ranker's metadata. Do not construct
   `ArticleRankingMetadata` manually for this proof. The existing assertions in
   that test continue to pin the two-candidate order, lexical scores `5.0 / 6.0`
   and `1.0 / 6.0`, and composite-score relationship before rendering. The
   separate overlap regression continues to pin title hits `1`, abstract hits
   `2`, union hits `2`, and lexical score `1.0`.
4. Keep
   `partial_keyword_warning_uses_exact_coverage_after_public_counts_saturate`
   green; compact JSON still omits row-level ranking metadata, full JSON retains
   it, and the ranking warning remains keyed by exact
   `all_anchors_in_text`, not the saturated public counters. Run the repository's
   existing article JSON and Markdown tests and raw/typed MCP contract suites;
   no new output snapshot or serialization seam belongs in this ticket.
5. Run the focused ranking module, representative ranker-to-renderer regression,
   and existing article JSON/Markdown/MCP regressions, then
   `cargo clippy --locked --no-default-features --all-targets -- -D warnings`,
   `tools/check-quality-ratchet.sh`,
   `cargo package --list --allow-dirty --locked --offline` with an exact count
   of 1,300, `git diff --check`, and the repository gates `make lint`, `make
   test`, and `make spec`.

## Dependencies

None. Ticket 1148 is already integrated and does not touch this ownership
boundary.

## Review

- Design review: pending
- Code review: pending
