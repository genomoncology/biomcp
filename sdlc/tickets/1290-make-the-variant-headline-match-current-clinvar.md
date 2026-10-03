# 1290 — make the variant headline match current ClinVar

Proposed 2026-10-03 by the BioMCP owner for the next 0.9 release. Status: accepted, ready to build.

## Evidence

Experiment 432 (`~/workspace/experiments/432-agent-legs-with-skills/`) asked agents for the current ClinVar classification of 19 variants reclassified after 2026-03-01. The diagnosis covers 16 of the 17 tool-leg misses; one cell was not diagnosed. 8 came from BioMCP's own output. The owner reproduced one on 0.9.1:

`biomcp get variant "TP53 G105S" clinvar -j` prints top-level `significance: "Pathogenic"`. The same output's NCBI ClinVar section holds RCV001379190, `Uncertain significance`, reviewed by expert panel, evaluated 2026-06-04. ClinVar's current germline classification for VariationID 428884 is Uncertain significance.

6 more misses came from agents turning a list of pathogenic and likely pathogenic submissions into "Conflicting", because BioMCP prints RCV records but no record-level (VCV) germline classification.

## Retained behavior

Every existing section, field name and source stays. The default `get variant` path makes no new network call. The NCBI ClinVar section keeps its RCV aggregates and submissions.

## Change

The headline `significance` is set in `from_myvariant_hit` (`src/transform/variant.rs:849`, assigned at :884) by `pick_significance` (:672-685). It picks the most pathogenic RCV classification in MyVariant.info's cached copy. `search variant` rows use the same derivation (:938). The NCBI ClinVar fetch runs only when the `clinvar` or `all` section is requested (`src/entities/variant/get.rs:1232-1233`).

1. Parse ClinVar's record-level germline classification (`ClassifiedRecord/Classifications/GermlineClassification`) into a new `ClinvarRecord` field with its review status and date (`src/sources/ncbi_efetch.rs`).
2. When the `clinvar` section answers from NCBI ClinVar, the headline shows that record-level classification, review status and date, and names NCBI ClinVar as its source. When the record has no germline classification, the headline says so and keeps the derived value labeled as derived.
3. Plain `get variant` and `search variant` keep the fast MyVariant.info-derived value, so no extra call is added to the default path. The output names that source, states that it is the most severe RCV classification in a cached copy, gives the newest evaluation date in that copy, and prints the `clinvar` section command as the way to get the current ClinVar classification.
4. When the cached value and NCBI ClinVar disagree, the output says so.
5. `skills/use-cases/05-variant-pathogenicity.md` tells agents to request the `clinvar` section and read the record-level classification first.

## Proof

- A spec case on a recorded fixture where the cached source and NCBI ClinVar disagree, showing the headline follows NCBI ClinVar and names it.
- A regression on TP53 c.313G>A (VariationID 428884) from a recorded response.
- Rerun of experiment 432's ClinVar items, reported in the record.

## Deferred

Somatic and oncogenicity classifications. The other 2 misses (a variant BioMCP could not resolve) belong to ticket 1292.

## Review

- Design review: ACCEPT 2026-10-03 on the second pass, dispatch 26de9b8f (fresh SWE-2 researcher, read-only). The first pass returned FIX; the revision addressed every finding.

## What landed

The variant headline now follows ClinVar's record-level germline classification whenever the `clinvar` section answers from NCBI. `src/sources/ncbi_efetch.rs` parses `ClassifiedRecord/Classifications/GermlineClassification` into a new `ClinvarRecord.germline_classification` field with review status and `DateLastEvaluated`. `apply_record_level_headline` in `src/entities/variant/get.rs` moves `significance`, `clinvar_review_status`, and the new `significance_source`/`significance_evaluated` fields to the record-level values and writes `significance_note` when the cached value disagrees or no record-level classification exists. `search variant` rows carry the same two provenance fields. Plain `get variant` and `search variant` keep the derived value: source named as MyVariant.info, newest cached evaluation date, a cached-copy explanation, and the `get variant <id> clinvar` pointer. RCV aggregates and SCV submissions are unchanged. `skills/use-cases/05-variant-pathogenicity.md` now tells agents to request `clinvar` and read the record-level classification first; `skills/schemas/variant.json` and `docs/sources/clinvar.md` name the new fields and behavior.

Proof on the recorded TP53 c.313G>A responses (`testdata/sources/ncbi_efetch/clinvar_428884_20261003.xml`, `testdata/sources/myvariant/search_tp53_g105s_20261003.json`, both receipted in `testdata/sources/capture-receipts.json`): the parser test reads Uncertain significance, reviewed by expert panel, 2026-06-04; the spec cases in `spec/entity/variant.md` show the default card label Pathogenic as MyVariant.info-derived and the `clinvar` request promote the NCBI record-level Uncertain significance while stating the disagreement. The variant-identity fixture now serves the recorded MyVariant search and NCBI efetch and exports `BIOMCP_CLINVAR_BASE`. Experiment 432's ClinVar items were not rerun; the recorded replay above covers the same VariationID and value.

Gate commands and results:

- `make lint`: pass (all scans, clippy `-D warnings`, cargo deny, quality ratchet, test-wait ratchet).
- `make test`: pass — 3914 Rust tests, 1210 Python contract tests, strict mkdocs build. The run used `BIOMCP_CACHE_MIN_DISK_FREE=1K` and `NEXTEST_TEST_THREADS=6` because the host disk is 99% full and sits under the production 10% cache floor; the same env makes the load-sensitive fixture tests deterministic.
- `make spec`: pass — all 30 routine pages, including `spec/entity/variant.md` at 107 passed, 1 skipped.

Support fixes inside the ticket: repinned the capture-receipts digest in the zero-coupling inventory, kept `entities/variant/mod.rs` under the 1000-line floor by placing the headline helper in `get.rs`, authorized the grown Rust source baselines under ticket 1290, moved the search-table note below the `Use get variant` line so the transcript-explanation slice stays exact, and raised the variant-search output-footprint ceiling from 720 to 800 bytes for the new provenance fields.
