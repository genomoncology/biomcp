---
flow: build
priority: 3
deps: [1150]
---

# Hybrid ranking recomputes the anchor hits it already stored

## Goal

Hybrid article ranking performs the complete anchor-hit calculation once per candidate, not twice. Today the same helper is invoked twice and the result is derived along two paths that can drift.

Reconfirmed at commit `b2e05326`: `rank_articles_hybrid` in `src/entities/article/ranking.rs` opens by calling `populate_lexical_ranking_metadata`, which builds the anchor set and records `anchor_count`, `title_anchor_hits`, `abstract_anchor_hits`, and `combined_anchor_hits` on every candidate. The function then rebuilds the same anchor set and, inside its per-row loop, calls `lexical_anchor_hits` again on every candidate to recover the combined count it just stored.

`lexical_anchor_hits` checks title, abstract, and their union for each anchor. This ticket removes the second complete helper invocation; it does not claim that the helper's internal title/abstract/union checks are a single text inspection.

Record 1102 introduced the second call when it replaced the tier-derived lexical score with a coverage ratio, and it is right about the ratio: it must come from the full counts, before the public `u8` display counters saturate at 255. Reading the saturated counters back would be wrong. That is a reason to keep the exact ratio, not a reason to recount.

## Done, observably

- A hybrid keyword search invokes the complete anchor-hit calculation once per candidate.
- The stored lexical score still equals the exact union coverage ratio, derived from full counts rather than the saturated public counters, so record 1102's 260-of-300 regression keeps passing.
- The metadata-building path returns or carries the full-width combined count for immediate internal scoring; that private count is not serialized or added to the public ranking shape.
- Code-review evidence confirms that hybrid construction receives that full-width count from the single metadata calculation and that `rank_articles_hybrid` contains no second `lexical_anchor_hits` invocation. A noisy runtime benchmark is not the acceptance gate.
- Ranking order, per-row `Why` text, JSON ranking diagnostics, and the `partial_query_match` warning are unchanged for every existing test.

## Boundary

This ticket removes duplicated counting inside hybrid ranking. It does not change the hybrid weights, the coverage formula, the warning policy, standalone lexical or semantic ranking, PubMed rescue, or the public shape of the ranking metadata.
