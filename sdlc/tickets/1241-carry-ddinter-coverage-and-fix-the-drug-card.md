# Carry DDInter coverage and fix the drug card

Split from ticket 1238. Source issue:
`sdlc/issues/2026-09-23-drug-card-reports-ddinter-not-covered-as-no-interactions.md`.

## Problem

`get drug <name> interactions` prints "no matching rows… 0 of 0" for a
drug the DDInter bundle does not cover, dropping the coverage status
that `drug interactions` reports correctly. Name matching misses
synonyms (a row filed under acetylsalicylic acid is not found from
aspirin) and can resolve a combination product. The freshness label is
re-read from file times while the index is cached for the process life,
so a long-running server can label old rows fresh. A corrupt bundle
surfaces as a generic "API request failed" without naming the file.

## Design

Revised after the design review (two P1 mechanism corrections — the
freshness freeze and the synonym seam — plus the cap deferral recorded;
the reviewer verified every seam against the code).

1. Card coverage. `Drug` gains an `interaction_coverage_status` field
   (serde `skip_serializing_if` optional, `#[serde(skip)]` not needed
   since the status is small and additive to JSON) populated by
   `apply_interaction_report`, alongside the existing copies; the
   failure branch in `apply_interactions_result` and the
   section-off branch in `get.rs` clear it with the pagination and
   freshness fields. The card note comes from `provenance.rs:260`:
   rows present keeps today's note; covered with zero rows states
   covered-but-no-rows; `not_in_ddinter_coverage` renders the
   not-in-coverage note; a failed or unrequested DDInter section
   states that. An uncovered drug never renders "no matching rows"
   alone. The template line sits inside the always-`Some` pagination
   guard deliberately (stated here). Pinned updates:
   `drug_markdown_uses_truthful_public_unavailable_interactions_message`
   (tests.rs:63-112), the heading pins (:56-57), the template block in
   `templates/drug.md.j2`, and a serializer pin test in `json.rs` for
   the new `interaction_coverage_status` key (non-breaking, additive;
   no stored fixture pins the card's interaction fields — verified).
2. Freshness: cached basis plus clock, not freeze-at-load. `load_index`
   captures the per-file mtimes (or oldest mtime) into the cached index
   entry; `bundle_freshness` derives Fresh/Stale at report time as
   `now - basis >= DDINTER_STALE_AFTER` (72h) without re-reading files.
   This closes the drift window (index loaded at T0, bundle replaced at
   T1, rows labeled fresh from T1 mtimes at T2) while keeping the aging
   signal for long-running servers. Tests use `File::set_modified`
   (existing fixture pattern): (a) replacing files after load does not
   flip a stale-loaded index to fresh; (b) an index loaded from old
   mtimes reports stale with no re-reads.
3. Synonyms. The DDInter index stores no alias table — identity data
   comes from the MyChem anchor, whose GET already fetches
   `drugbank.synonyms`. `Drug` gains a `#[serde(skip)]` synonyms field
   populated in `transform/drug.rs` beside the existing brand fold
   (bounded larger than the 3-cap brands; dedupe in `with_aliases`
   handles overlap), threaded into `DdinterIdentity` at
   `interactions.rs:110`. The aspirin/acetylsalicylic-acid fixture
   test lives at the identity level in
   `src/sources/ddinter/tests/parsing.rs` with a CSV row filed under
   "Acetylsalicylic acid". Accepted risk recorded: a synonym naming a
   distinct salt present in DDInter can pull extra rows.
4. Corrupt bundle: the generic `src/error.rs` mappings (:476-481,
   :525-526) gain the DDInter file name from the parse/read error's
   message, following the PMC-prefix and CaBundle precedent arms.
5. Deferred, recorded here: the 8 MB cap check against the real
   bundle's file sizes and the real-bundle run both defer to the M5
   leg (Ian's machine, authorized separately); probing
   ddinter.scbdd.com from a test needs separate authorization.
6. Scope of the problem statement's third symptom (aspirin resolving
   to a combination product): mitigated incidentally by synonyms and
   verified honestly at the M5 real-bundle run; not separately fixed
   here.
7. Compile ripple stated: 23 full-field `Drug` struct literals across
   11 files (json.rs, provenance.rs, transform/drug.rs, root_tests.rs,
   gene_drug.rs, drug_variant_article_trial.rs, evidence/tests.rs,
   variant_drug.rs, drug/tests.rs ×9, label_warnings.rs,
   cell_lines/tests.rs) gain the new field; size-inventory entries
   needing a ticket-1241 authorization: `src/render/provenance.rs`
   (1850/1847), `src/render/json.rs` (1570/1561), `src/error.rs`
   (1150/1122), and `src/entities/drug/get.rs` (1117/1103) for the
   section-off clear's one line (item 1). The other touched files
   (entities/drug/mod.rs, interactions.rs, sources/ddinter.rs and its
   tests, render/markdown/drug.rs and its tests, templates/drug.md.j2)
   are not inventoried. The freshness rework reshapes the existing
   `bundle_freshness_requires_all_files_to_be_fresh` pin
   (ddinter/tests/construction.rs:7-15), and the heading pins live at
   tests.rs:55-56.

## Review

- Design review: REJECT once (freshness freeze inverted the aging
  signal; the synonym seam did not exist; the cap deferral was not
  recorded); revised above, re-review pending
- Code review: pending
