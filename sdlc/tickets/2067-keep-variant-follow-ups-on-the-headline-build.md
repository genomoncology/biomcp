# 2067 — Keep variant follow-ups on the headline build

Status: OPEN.

Milestone: 0.9.3

## Outcome

Every follow-up command a variant card prints uses the headline's coordinate, following the printed civic command returns the evidence the card counted, and each ClinVar row shows its submission count.

## Evidence

Filed 2026-10-13 from the post-release install and product test of v0.9.2, run on Linux x86_64 and macOS arm64 against every install channel; rechecked against current main on 2026-10-14.

- `biomcp get variant "BRAF V600E" all` headlines "Genomic coordinate (GRCh37, provider default): chr7:g.140453136A>T" but prints "Therapeutic evidence: 13 CIViC predictive item(s) — see get variant \"chr7:g.140753336A>T\" civic", and every other see-also command uses the GRCh38 coordinate. Following it returns "Evidence Items: 0". Any request that includes `population` does this, because only the population section stores the resolved GRCh38 coordinate.
- `get variant "BRAF V600E" clinvar` rows read "evaluated 2014-09-04;  submitter(s)." with no number, while the JSON carries `submission_count: 1`. NCBI rows never carry `NumberOfSubmitters`, serde skips the missing field, and in minijinja a missing field is undefined, not none, so the template's `is not none` test passes and prints an empty value.
- PM decision 2026-10-14: follow-ups always use the headline coordinate (`variant.id`); the resolved GRCh38 coordinate stays on its own line. The count is labelled "submission(s)", because NCBI gives a submission count, not a submitter count.

- Starts from: `preferred_variant_follow_up_id` (`src/render/markdown/mod.rs:304`), which prefers `population.resolved_coordinate` over `variant.id`, and its callers in `src/render/markdown/variant.rs` (lines 58, 59, 143, 145, and the CIViC pointer at 194-219), `src/render/markdown/sections.rs:342`, `src/render/markdown/related.rs:275` and `related_variant_with_recovery` (`src/render/markdown/mod.rs:314-321`); `add_population` (`src/entities/variant/get.rs:1774`), which stores the dbSNP GRCh38 coordinate; `templates/variant.md.j2` lines 48 and 53.
- Keeps: the headline's build label; the "Resolved GRCh38 coordinate" line (`templates/variant.md.j2:119`); gnomAD's use of the resolved coordinate.
- Changes: claims `src/render/markdown/mod.rs` `src/render/markdown/variant.rs` `src/render/markdown/sections.rs` `src/render/markdown/related.rs` `src/render/markdown/variant/tests.rs` `templates/variant.md.j2` `src/cli/tests/printed_card_commands.rs` `testdata/sources/ncbi_efetch/**` `testdata/sources/dbsnp/**` `testdata/sources/capture-receipts.json` `spec/entity/variant.md` `CHANGELOG.md`.
  - Every printed follow-up uses `variant.id`; delete `preferred_variant_follow_up_id`.
  - Rows show `N submission(s)` from `submission_count` when `number_submitters` is undefined; lines 48 and 53 test `is defined`, not `is not none`.
  - One Fixes changelog bullet.
- Proof: a recorded-reply test of `get variant "BRAF V600E" all` (new BRAF V600E ClinVar efetch and dbSNP rs113488920 captures) in which every printed command carries the headline coordinate, and following the civic command reports the counted evidence items; a renderer test for a direct-NCBI row with only `submission_count`. Both fail on current main. Routine suite.
- Defers: nothing.
- Order: runs alongside the build tickets; it shares no claimed file with 2065, 2066, 2084, 2085 or 2087.
