# 1290 — make the variant headline match current ClinVar

Proposed 2026-10-03 by the BioMCP owner for the next 0.9 release.

Status: COMPLETE.

Landed: 1db90d93.

## Outcome

`get variant` states the current ClinVar classification in its headline when the `clinvar` section answers from NCBI ClinVar, and names the classification's source, review status and date. The quick default view keeps its value but says it is a cached copy and prints the command that returns the current classification.

## Evidence

- Starts from: Experiment 432 (`~/workspace/experiments/432-agent-legs-with-skills/`) asked agents for the current ClinVar classification of 19 variants reclassified after 2026-03-01. The diagnosis covers 16 of the 17 tool-leg misses. 8 came from BioMCP's own output. The owner reproduced one on 0.9.1: `biomcp get variant "TP53 G105S" clinvar -j` prints top-level `significance: "Pathogenic"`, while the same output's NCBI ClinVar section holds RCV001379190, `Uncertain significance`, reviewed by expert panel, evaluated 2026-06-04. ClinVar's current germline classification for VariationID 428884 is Uncertain significance. 6 more misses came from agents turning a list of pathogenic and likely pathogenic submissions into "Conflicting", because BioMCP prints RCV records but no record-level (VCV) germline classification.
- Keeps: Every existing section, field name and source stays. The default `get variant` path makes no new network call. The NCBI ClinVar section keeps its RCV aggregates and submissions.
- Changes: The headline `significance` is set in `from_myvariant_hit` (`src/transform/variant.rs:849`, assigned at :884) by `pick_significance` (:672-685), which picks the most pathogenic RCV classification in MyVariant.info's cached copy. `search variant` rows use the same derivation (:938). The NCBI ClinVar fetch runs only when the `clinvar` or `all` section is requested (`src/entities/variant/get.rs:1232-1233`). The numbered change follows in Change detail.
- Proof: A spec case on a recorded fixture where the cached source and NCBI ClinVar disagree, showing the headline follows NCBI ClinVar and names it; a regression on TP53 c.313G>A (VariationID 428884) from a recorded response; a rerun of experiment 432's ClinVar items, reported below in QA.
- Defers: Somatic and oncogenicity classifications. The other 2 misses (a variant BioMCP could not resolve) belong to ticket 1292.

## Change detail

1. Parse ClinVar's record-level germline classification (`ClassifiedRecord/Classifications/GermlineClassification`) into a new `ClinvarRecord` field with its review status and date (`src/sources/ncbi_efetch.rs`).
2. When the `clinvar` section answers from NCBI ClinVar, the headline shows that record-level classification, review status and date, and names NCBI ClinVar as its source. When the record has no germline classification, the headline says so and keeps the derived value labeled as derived.
3. Plain `get variant` and `search variant` keep the fast MyVariant.info-derived value, so no extra call is added to the default path. The output names that source, states that it is the most severe RCV classification in a cached copy, gives the newest evaluation date in that copy, and prints the `clinvar` section command as the way to get the current ClinVar classification.
4. When the cached value and NCBI ClinVar disagree, the output says so.
5. `skills/use-cases/05-variant-pathogenicity.md` tells agents to request the `clinvar` section and read the record-level classification first.

## What landed

The variant headline now follows ClinVar's record-level germline classification whenever the `clinvar` section answers from NCBI. `src/sources/ncbi_efetch.rs` parses `ClassifiedRecord/Classifications/GermlineClassification` into a new `ClinvarRecord.germline_classification` field with review status and `DateLastEvaluated`. `apply_record_level_headline` in `src/entities/variant/get.rs` moves `significance`, `clinvar_review_status`, and the new `significance_source`/`significance_evaluated` fields to the record-level values and writes `significance_note` when the cached value disagrees or no record-level classification exists. `search variant` rows carry the same two provenance fields. Plain `get variant` and `search variant` keep the derived value: source named as MyVariant.info, newest cached evaluation date, a cached-copy explanation, and the `get variant <id> clinvar` pointer. RCV aggregates and SCV submissions are unchanged. `skills/use-cases/05-variant-pathogenicity.md` now tells agents to request `clinvar` and read the record-level classification first; `skills/schemas/variant.json` and `docs/sources/clinvar.md` name the new fields and behavior.

Proof on the recorded TP53 c.313G>A responses (`testdata/sources/ncbi_efetch/clinvar_428884_20261003.xml`, `testdata/sources/myvariant/search_tp53_g105s_20261003.json`, both receipted in `testdata/sources/capture-receipts.json`): the parser test reads Uncertain significance, reviewed by expert panel, 2026-06-04; the spec cases in `spec/entity/variant.md` show the default card label Pathogenic as MyVariant.info-derived and the `clinvar` request promotes the NCBI record-level Uncertain significance while stating the disagreement. The variant-identity fixture now serves the recorded MyVariant search and NCBI efetch and exports `BIOMCP_CLINVAR_BASE`. The builder did not rerun experiment 432's ClinVar items; KB QA 0002 reran the agent legs on the branch and reports them below.

Gate commands and results:

- `make lint`: pass (all scans, clippy `-D warnings`, cargo deny, quality ratchet, test-wait ratchet).
- `make test`: pass — 3914 Rust tests, 1210 Python contract tests, strict mkdocs build. The run used `BIOMCP_CACHE_MIN_DISK_FREE=1K` and `NEXTEST_TEST_THREADS=6` because the host disk is 99% full and sits under the production 10% cache floor; the same env makes the load-sensitive fixture tests deterministic.
- `make spec`: pass — all 30 routine pages, including `spec/entity/variant.md` at 107 passed, 1 skipped.
- CI: run 37157448782 green across all jobs at branch HEAD 0d656338.

## QA

KB ticket 0002, landed at e68111b in the KB repository, tested branch HEAD 0d656338 built from git archive. With the `clinvar` section the headline matched the frozen ClinVar key on 18 of 18 resolvable variants, against 1 of 18 on 0.9.1. Agents rerun on the 19 ClinVar questions: leg B 12 to 18, leg C 16 to 19. The plain path kept 0.9.1's value on 18 of 18 and added `significance_source`, `significance_evaluated` and `significance_note`; the full JSON diff shows 0 removed paths and 0 changed values. With `BIOMCP_CLINVAR_BASE` pointed at a local logger, plain `get variant` made 0 ClinVar requests and the `clinvar` section made 4. The no-germline-classification fallback was not exercised live: none of 50 ClinVar records checked lacked the element, so only the branch's fixture test covers it. One defect outside this ticket's scope, filed as ticket 1297: a protein-change query can resolve to the wrong variant.

## Review

- Design review: ACCEPT 2026-10-03 on the second pass, dispatch 26de9b8f (fresh SWE-2 researcher, read-only). The first pass returned FIX; the revision addressed every finding.
- Code review: ACCEPT 2026-10-04, fresh read-only reviewer, no findings. Five minor notes, accepted as deviations: (1) section-only markdown skips the headline block, so the disagreement note does not print in that one output mode; JSON and the full card carry it. (2) The `search variant` markdown footer's cached-copy line is not pinned by a spec case; only the JSON row fields are asserted. (3) `docs/blog/variant-structure-in-commands.md` still shows the old plain `Significance: Pathogenic` sample line; fixed in the landing follow-up. (4) A VCV record with a germline classification but zero aggregates and zero submissions would be discarded by a pre-existing gate; theoretical. (5) The branch carried the pre-build ticket revision; the landing merges it into this record.
