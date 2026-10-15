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
2. DONE (2026-09-26, batch 2). The trial arm's recruiting-filter
   failure now reports through the section note when the unfiltered
   backfill still returned rows (the pure constructor is
   `trial_recruiting_filter_note`, tested directly); an empty result
   still returns the preferred error. `count_exact` requires
   `note.is_none()`, so a note-carrying section never claims an exact
   count. Anchors:
   `a_degraded_trial_section_renders_the_note_and_drops_count_exact`
   (JSON count_exact false + markdown note) and
   `trial_recruiting_filter_note_names_the_failure_and_the_widening`.
   The one format fixture that carried an arbitrary note now asserts
   `note` null with `count_exact` true (notes are degradations).
3. DONE (2026-09-26, batch 2). `context_from_response` rejects a
   missing data block (schema change) and a `totalCount > 0` with
   zero parsed rows for both evidence and assertions; a genuine
   `totalCount=0, nodes=[]` stays a healthy empty. Anchors:
   `context_response_rejects_a_missing_data_block`,
   `context_response_rejects_a_total_without_parsed_rows`,
   `context_response_accepts_a_genuine_empty_page`.
4. DONE (2026-09-26, batch 2). `assign_top_genes(disease, owns)`
   assigns the list and its label in one place (both the base-context
   and sections paths call it): Open Targets keeps the default
   heading; when the base fetch failed, the label names the fallbacks
   that actually pushed the genes (Monarch Initiative, CIViC) — even
   when a late Open Targets augment only attached scores. The
   template heading uses the label (`top_gene_source or "Open
   Targets"`), and the provenance row derives its sources from the
   same field, so the two cannot contradict. New additive
   `Disease.top_gene_source` field. Anchors:
   `top_genes_label_names_the_fallback_when_open_targets_did_not_produce_them`,
   `top_genes_label_joins_both_fallback_sources`,
   `disease_markdown_heading_and_provenance_credit_the_fallback_source`.
5. DONE at the verified seam (2026-09-26, batch 2), with the
   remaining plumbing recorded. `SizeAwareCacheManager::get` stamps
   `x-biomcp-cache-stale-age: <seconds>` on a served entry whose
   stored `CachePolicy` is stale at serve time (age from the policy
   clock, never the file write time); `put` strips the marker before
   storing and returning, so a 304/200-revalidated response can never
   carry it (http-cache 0.20 serves put's return on both
   revalidation arms — verified in the crate source). The consumer
   landed is the shared request funnel: both
   `send_with_source_context` impls read the marker, log "served
   from cache, older than the provider's freshness window (N h
   old)" (never implying revalidation failure), and strip it before
   the response reaches source clients. Anchors:
   `a_stale_serve_carries_the_marker_with_its_policy_age`,
   `a_fresh_serve_carries_no_marker`,
   `put_clears_the_marker_so_revalidation_cannot_carry_it` (the
   304-flow unit: put receives the stamped cached response exactly as
   conditional_fetch hands it over). REMAINS: per-source status lines
   on the entity pages do not yet carry the label — source clients
   discard response headers after the log, and there is no single
   response funnel into `section_outcomes`; that plumbing is a
   deliberate deferral to the next design pass, not an oversight.
6. DONE (2026-09-26, batch 2). The SOURCE_STATE_ROWS walk
   (`every_registry_source_status_row_survives_the_render_context`)
   completes each row as Unavailable and asserts the status line
   survives the render-context seam with the row's label, providers,
   and state — it immediately caught that the disease template's
   diagnostics status only renders under the opt-in section name
   (recorded in the test). The end-to-end disease anchor
   (`a_fully_unavailable_disease_card_keeps_every_status_line`)
   proves the template renders every disease row's status line. The
   SearchAllSection anchor is the item-2 test above. All six items
   now have named anchors.

## Review

- Design review: REJECT once (stale-cache label named the wrong
  seam; partial-count and per-trial surfaces unspecified; anchors
  unnamed), findings folded, re-review ACCEPT 2026-09-25
- Batch 2 (2026-09-26): implemented; validation is fmt + clippy
  (-D warnings) clean and the focused filters green (search_all 41,
  civic 9, cache::manager 27, root_tests 10, entities::disease 77);
  the yellow gate had not run at write time; it ran afterward —
  see the Verification line below (bb516645 per the record). This
  paragraph is kept as the batch-2 checkpoint, not the final state.
- Code review (batch 1): ACCEPT with P2s (spacing fixed; JSON note
  and offset wording recorded for batch 2) 2026-09-25
- Verification (batch 1): yellow gate at 66c55c55 lint/test/spec OK;
- Batch 1 fixes (2026-09-26, review follow-ups): the partial count
  now says it may be too high (never a floor); the note names all
  three keep reasons in plain words; the three keep paths each have a
  driving test (failed fetch, missing criteria text, no NCT ID) plus
  a verified control (the no-NCT fixture serves rejecting criteria
  so a stray fetch would fail the test; the control criteria carry
  the literal keyword token); trial search JSON carries the note in
  _meta.notes; the age filter now runs before detail verification so
  an over-age unchecked trial cannot mark the count partial (both
  directions tested). Inventory: ctgov/tests.rs grew 71 lines (the
  review's 59 was a miscount; nothing unaccounted).
  see `sdlc/records/1242-render-source-failures-honestly.md`

### Batch 2 residuals (2026-09-26 review)

- The default disease card (no explicit sections) still passes
  owns=true to assign_top_genes, so a MyDisease-seeded DisGeNET gene
  list renders under "Genes (Open Targets)" when the Open Targets
  fetch fails — the exact mislabel item 4 targets, on the common
  path. Pre-existing; next pass computes owns from
  !top_gene_scores.is_empty() and labels MyDisease-seeded genes.
- Item 2's dispatch wiring (note attaches only when the backfill
  returned rows) is pinned by compilation and the constructor test,
  not by a fixture-server drive through dispatch_section(Trial);
  batch 3 candidate alongside the item-1 standard.
- CIViC: a present-but-empty data block from a double schema rename
  still reads as a healthy empty (serde defaults); outside the two
  designed discriminators, recorded as accepted.
- The item-6 walk proves every entity at the render-context seam but
  only the disease template end-to-end.
