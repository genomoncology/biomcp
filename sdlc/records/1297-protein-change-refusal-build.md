# 1297 build: a protein-change query must not resolve to the wrong variant

Ticket: sdlc/tickets/1297-a-protein-change-query-must-not-resolve-to-the-wrong-variant.md
Branch: tickets/1297-protein-change-refusal, cut from main 948205108
(carries 1295, 1301, 1296).

## Change 1: the DICER1 case reproduced on main

Recorded live on 2026-10-06 against MyVariant.info with the built main-tree
CLI (release, no default features):

- `biomcp --json --no-cache get variant 'DICER1 p.Met1483Ile'` returns
  `chr14:g.95562808C>A`, `hgvs_c c.4449G>T`, no `clinvar_id`, no `rsid`. That
  genomic variant has no ClinVar record. This matches the KB QA 0002 finding.
- The CLI queries the normalized alias
  `dbnsfp.genename:DICER1 AND dbnsfp.hgvsp:"p.M1483I"`. MyVariant ranked the
  hits `C>A` (score 27.89), `C>T` (27.73, `clinvar.variant_id` 577152,
  `rs1454569806`), `C>G` (27.49). The gene+protein arm took the first
  compatible hit, so the ClinVar-less variant won.
- NCBI ClinVar esummary for VariationID 577152 confirms
  `NM_177438.3(DICER1):c.4449G>A (p.Met1483Ile)` is `chr14:g.95562808C>T` on
  GRCh37 (`NC_000014.8:g.95562808C>T`), rs1454569806. So the ticket's
  2026-10-05 validation note that called `C>T` "(wrong transcript)" misread the
  dbNSFP merged alias list (it shows a second transcript's spelling,
  `p.Met381Ile`, on the same genomic variant). `C>T` is 577152's variant; the
  proof row "the protein form resolving to 577152's variant" is the correct
  outcome and is what the build delivers.

## Changes

- The gene+protein arm of `get variant` resolution no longer takes the first
  provider-ranked compatible hit. With the compatible hits in hand: one hit
  resolves; exactly one hit carrying a ClinVar record resolves to that hit
  (the variant ClinVar itself names); anything else (no ClinVar record, or
  conflicting ClinVar records across hits) refuses with
  `invalid_argument`, listing every candidate with its genomic HGVS, ClinVar
  VariationID when present, and rsID when present, plus the working input
  forms. A query with no compatible hits keeps the existing `not_found`
  refusal and its search suggestion. The selection lives in
  `resolve_protein_change_hit` (`src/entities/variant/get.rs`) with its tests
  in `src/entities/variant/get/protein_change_tests.rs`.
- `oncokb` and every other `resolve_base` caller inherit the same rule.
- Article search refuses a bare ClinVar VariationID before any provider call;
  the folded-scope unit test pins the refusal (`invalid_argument` naming the
  input). The dedicated guard message arrives with 1292; the test asserts the
  refusal facts both spellings share so it stays green through that merge.
- Spec page `spec/entity/variant-protein-change-resolution.md` proves the
  resolution table from recorded MyVariant responses: `BRAF V600E` (unique
  match resolves), `DICER1 p.Met1483Ile` (three matching variants, one
  ClinVar-named, resolves to `chr14:g.95562808C>T`), and `EGFR M766I` (three
  matching variants, no ClinVar record, refuses with candidates). The
  variant-identity fixture serves the two recorded queries and 400s anything
  else, so rows cannot pass vacuously.
- User guide documents the disambiguation and refusal.

## EGFR protein-range direction: refusal-with-candidates stays

The ticket owns the choice between refusal-with-candidates and alias
expansion for protein-range forms such as `EGFR E746_A750del`. Recorded live
on 2026-10-06: MyVariant's `clinvar.hgvs.protein` alias for
`NP_005219.2:p.Glu746_Ala750del` matches three distinct ClinVar variants:

- `chr7:g.55242462_55242479delinsAAA` — VariationID 45233, a delins, not a
  deletion;
- `chr7:g.55242466_55242480del` — VariationID 177620;
- `chr7:g.55242465_55242479del` — VariationID 163343.

dbNSFP itself holds no protein-range alias (`dbnsfp.hgvsp:"p.E746_A750del"`
and the three-letter spelling both return zero hits). The protein notation
cannot name one of the three deletions, so expansion resolving the query to
any single variant would guess, and the delins hit shows it can answer a
deletion query with a different variant class. Under the no-wrong-answers
rule, refusal-with-candidates stays: `get variant 'EGFR E746_A750del'` keeps
refusing (`not_found` with the search working form once 1292's parsing lands;
unrecognized format before it), and search remains the candidate listing.
Exact genomic or transcript forms resolve it today. The decision is recorded
on the spec page as well.

## Proof

- Rust: `entities::variant::get` 32 passed including the five new
  protein-change resolution table tests;
  `entities::article::variant_search` refusal test passes; full
  `make lint` green (fmt, clippy `-D warnings`, cargo deny, quality ratchet
  with the repinned source-size inventory and zero-coupling hash, test-wait
  ratchet).
- Spec: `variant-protein-change-resolution.md` 5/5 standalone;
  `variant.md` + `variant-protein-change-resolution.md` +
  `variant-gene-first-routing.md` together 118 passed, 1 skipped under the
  shared variant-identity and provider-contract fixtures.
- Live: after the change, `get variant 'DICER1 p.Met1483Ile'` resolves to
  `chr14:g.95562808C>T` (`rs1454569806`, `c.4449G>A`), and
  `get variant 'EGFR M766I'` exits 2 listing the three candidates.
- Registration: isolation contract 107 passed; article fixture lifecycle 30
  passed with the 30 to 31 invocation bump; source package boundary 8 passed
  with 1,388 to 1,390 (the spec page and the protein-change test module; the
  recorded captures stay outside the package); capture receipts audit green
  with both new captures receipted.

## Deferred gaps

- The refusal names the ClinVar VariationID as a working input form; direct
  `get variant <VariationID>` lands with 1292 (this branch merges after it).
  On this branch alone the genomic HGVS and rsID forms work.
- Full HGVS protein notation moves to the shared parser in 1.0 (ticket
  deferral).
- A CHANGELOG bullet is owed before the next release (same as 1292's note).
- Textual merge conflicts with 1292 are expected in
  `src/entities/variant/get.rs` (its transcript-alias and VariationID arms),
  `src/entities/article/variant_search.rs` (its bare-VariationID guard),
  `tests/test_article_spec_fixture_lifecycle.py` (both bump 30 to 31),
  `tests/test_source_package_boundary.py` (both add packaged files), and
  `docs/user-guide/variant.md` (its input-form list).
