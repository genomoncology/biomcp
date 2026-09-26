---
base: b7c9a756
head: 66c55c55
---

Batch 1 of 1242: trial eligibility partial counts, plus two of the
three anti-regression anchors.

A trial kept without detail verification — failed detail fetch,
missing criteria text, or no NCT ID — no longer reads as an exact
count. `verify_detail_filters` returns a `DetailVerificationReport`
(count plus at most three NCT IDs) alongside the kept rows; all four
apply sites thread it; both count loops return the new
`TrialCount::Partial { total, reason }` when anything was kept
unverified (with `IncompleteCoverage` still dominating and the
traversal cap still discarding to `Unknown`). The count-only JSON
gains `partial`/`partial_reason` additively, the text renders
"(partial, some trials kept without detail verification)", and the
search result carries a section note naming how many kept trials
could not be detail-verified (the card keeps its per-trial surface).
Two anchors landed: the count JSON/text anchor (relocated to its own
`count_tests` module after the 700-line cli cap caught the growth —
no allowlist entry needed) and the trial-search note render test.

Items 2-5 (search-all note, CIViC discriminator, enrichment labels,
stale-cache age label) and the third anchor (the SOURCE_STATE_ROWS
walk plus the SearchAllSection anchor) are the next batch, with a
verified design note for item 5: stamp the stale marker in
`SizeAwareCacheManager::get` from the stored `CachePolicy` and clear
it in `put` (http-cache 0.20 serves `get`'s object on stale serves
and both revalidation arms serve `put`'s return, so the label cannot
survive a fresh revalidation), wording the label as "older than the
provider's freshness window" with the computed age.

Evidence: design REJECT once (the stale-cache label named the wrong
seam; partial-count and per-trial surfaces unspecified; anchors
unnamed), folded and re-reviewed ACCEPT with two code-review
checkpoints carried; code review ACCEPT with P2s only (the note's
spacing fixed, the redundant conversion dropped, the JSON note and
offset wording recorded for the next batch); yellow gate at
66c55c55 — lint, test, and spec OK after one cycle that caught the
cli line cap and the ctgov baselines.
