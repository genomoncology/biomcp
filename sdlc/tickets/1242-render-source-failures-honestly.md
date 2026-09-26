# Render source failures honestly

Split from ticket 1238. Source issue:
`sdlc/issues/2026-09-23-source-failures-render-as-empty-or-complete-results.md`.

## Problem

Several paths turn a source failure into clean-looking output: trial
eligibility keeps a trial whose detail fetch failed and still reports an
exact count; `search all` silently drops the recruiting filter when its
source fails; a CIViC schema change reads as "no evidence"; when Open
Targets fails, the fallback source's genes appear under the Open
Targets heading; offline, http-cache serves stale entries of any age
with no flag. Variant, gene, and drug safety pages already report
per-source status; these paths do not.

## Design

1. Trial eligibility: a failed detail fetch, missing criteria text,
   or a study with no NCT ID (the third keep-unverified path,
   `eligibility.rs:315-320`) marks eligibility as unavailable and the
   count as partial. The partial count is a new `TrialCount` variant
   carrying the numeric value plus an unverified-count reason — not
   `Approximate`, whose meaning is fixed ("upstream total before
   client-side age post-filtering") — and `verify_detail_filters`
   returns failure telemetry alongside the kept rows (it returns only
   `Vec<CtGovStudy>` today). The per-trial marker lands as a section
   note in the search result (the row struct `TrialSearchResult` has
   no eligibility field; the detail card already carries one), so the
   card stays the per-trial surface.
2. `search all`: a recruiting-filter failure reports in the output's
   source status instead of dropping the filter — through
   `SearchAllSection.error`/`note`, which the template already
   renders — and `count_exact` stops ignoring `note` (it is derived
   as `error.is_none() && total.is_some()` at
   `src/cli/search_all/mod.rs:60-62`), so a note-only degradation no
   longer reports an exact count.
3. CIViC: a schema mismatch surfaces as a source failure, not as an
   empty evidence list. The silent seam is
   `resp.data.unwrap_or_default()` (`civic.rs:229`) over defaulted
   totals (`civic.rs:393-410`); the discriminator is `data` absent
   without a surfaced error message, or `totalCount > 0` with zero
   nodes — a legitimate `totalCount=0, nodes=[]` stays a healthy
   empty.
4. Enrichment: fallback data is labeled with its real source — the
   template heading (`templates/disease.md.j2:17-20`) and the
   provenance source summary (`src/render/provenance.rs:510-516`,
   hardcoded `["Open Targets"]` for `top_genes`) both change, so the
   two labels cannot contradict.
5. Stale cache entries are labeled with their age. The serve/revalidate
   decision is made inside the http-cache middleware
   (`src/sources/mod.rs:999-1003`), not at the manager's `get`
   (`manager.rs:226-239` sees both response and policy but not the
   decision), so the label is attached where the decision is known: a
   staleness note derived from the stored `CachePolicy` at serve time,
   plumbed into the per-source status the entity pages already render.
   The trigger is mode-dependent: the default mode caps offline
   staleness at 24 h (`max-stale=86400`, `sources/mod.rs:973`);
   `infinite`/ForceCache serves any age (`sources/mod.rs:352`), which
   is where the age label matters most. Compute the age from the
   stored policy, not from cacache's write time.
6. Anti-regression check, anchored to inventories rather than hand
   lists: a markdown ratchet walks `SOURCE_STATE_ROWS`
   (`source_state_registry.rs:82`), completes each key as
   `Unavailable`, renders the entity markdown, and asserts the status
   line survives (the walk pattern already exists at
   `src/mcp/shell.rs:2045`); plus `TrialCount`-anchored and
   `SearchAllSection`-anchored checks for the two paths outside
   `section_outcomes`, so all six items have a named anchor.

## Acceptance

- Each listed path shows the failure and marks counts partial.
- The anti-regression check exists and covers the listed renderers.

## Implementation state (2026-09-25 close-out)

This ticket landed in two sessions; the first closed after item 1.
The per-item state:

1. DONE. `TrialCount::Partial { total, reason }` with
   `TrialCountPartialReason::DetailVerificationIncomplete` (not
   Approximate — its meaning stays fixed). `verify_detail_filters`
   returns `(Vec<CtGovStudy>, DetailVerificationReport)`; the report
   counts kept-unverified studies and up to three NCT IDs
   (eligibility.rs `DetailVerificationReport`). The three
   keep-unverified paths (failed fetch, missing criteria, no NCT ID)
   increment it. All four apply sites (single page, union search, both
   count loops) thread it; both count loops return Partial instead of
   Exact when unverified_kept > 0 (`completed_ctgov_union_count`).
   `SearchPage<T>` gained `partial_note: Option<String>`; the trial
   renderer/template print `Note: ...` beside the count header, and
   `render_count_only` shows `Total: N (partial, ...)` text plus
   `partial`/`partial_reason` JSON fields. Anchors:
   `json_and_text_mark_a_partial_count_with_its_reason`,
   `search_results_carry_the_partial_detail_verification_note`, and
   the `completed_ctgov_union_count` partial arm in ctgov/tests.rs.
2. NOT STARTED.
3. NOT STARTED.
4. NOT STARTED.
5. NOT STARTED. Design note from this session for whoever picks it
   up: http-cache 0.20's `conditional_fetch` serves the manager's
   cached object directly on stale-within-tolerance and
   offline/must-not-revalidate paths, and serves the `put()` RETURN on
   304/200 revalidation — so stamping a stale marker on the cached
   response in `SizeAwareCacheManager::get` (computed from the stored
   `CachePolicy.is_stale/age`) and clearing it in `put` labels stale
   serves with no false positive after revalidation. The entity-page
   per-source plumbing beyond the response-level flag still needs a
   design decision (source clients discard response headers; there is
   no single response funnel — `request_from_plan` returns a builder).
6. PARTIAL: the two trial anchors in item 1 exist; the
   SOURCE_STATE_ROWS markdown walk and the SearchAllSection anchor are
   NOT STARTED.

## Review

- Design review: REJECT once (stale-cache label named the wrong
  seam; partial-count and per-trial surfaces unspecified; anchors
  unnamed), findings folded, re-review ACCEPT 2026-09-25
- Code review (batch 1): ACCEPT with P2s (spacing fixed; JSON note
  and offset wording recorded for batch 2) 2026-09-25
- Verification (batch 1): yellow gate at 66c55c55 lint/test/spec OK;
  see `sdlc/records/1242-render-source-failures-honestly.md`
