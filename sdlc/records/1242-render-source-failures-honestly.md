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

## Batch 1 fixes (2026-09-26)

The review's five findings closed: the partial count now says it may
be too high (never a floor) on every surface; the note names all
three keep reasons in plain words ("we could not check N of the kept
trials (ids), because the detail fetch failed, the eligibility text
was missing, or the trial had no NCT ID"); the three keep paths each
have a driving test through a real fixture server and the
BIOMCP_CTGOV_BASE seam (the no-NCT fixture serves rejecting criteria
so a stray fetch would fail the test; the verified control carries
the literal keyword token); trial search JSON carries the note in
_meta.notes with the omission case tested; and the age filter runs
before detail verification so an over-age unchecked trial cannot mark
the count partial (both directions pinned). Code review ACCEPT with
four P2s (the inventory miscount 59→71 corrected, the fixture
hardening, the control's literal token, singular/plural reworded
away); yellow gate at ca7c1fa0 — lint, test, and spec OK.

## Batch 2 (2026-09-26)

Items 2-6 landed. search all reports a dropped recruiting filter
through the section note and count_exact stops ignoring notes; the
CIViC discriminator turns a missing data block (without a surfaced
error) or a positive total with zero rows into a source failure while
a genuine zero stays healthy; enrichment labels fallback genes with
their real source in the heading and the provenance row together
(assign_top_genes is the single seam); the stale-cache marker is
stamped in the manager's get from the stored policy and cleared in
put — verified against http-cache 0.20's serve paths, so it labels
only direct stale serves, never survives revalidation, and is
stripped before any response returns — with the log line as the
landed consumer and entity-page plumbing recorded as a deliberate
deferral; and the SOURCE_STATE_ROWS walk plus the end-to-end disease
anchor complete the anti-regression set. Code review ACCEPT with
P2s: the anchor's vacuous negative assertion replaced by an exact
provenance-row check, the boundary comment names the new test file,
and the residuals are recorded in the ticket (the default-card
owns=true mislabel path, the missing dispatch-level drive, the CIViC
double-rename empty, the single-template end-to-end). Yellow gate at
bb516645 — lint, test, and spec OK.
