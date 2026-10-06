# 1292 — resolve intronic deletion ranges in get variant

Proposed 2026-10-03 by the BioMCP owner for the next 0.9 release.

Status: complete.

## Build status

- Built on branch `tickets/1292-intronic-deletion-ranges`, sha `4f6af54b6`, 2026-10-06.
- Code review: ACCEPT 2026-10-06, no blocking findings. Four P2 report-only notes: no unit test pins the article-search bare-VariationID refusal (guard verified in code; the test moves to 1297's scope, same file cluster); one duplicate ClinVar alias query on the transcript fallthrough (bounded, one extra call); over-length transcript input reaches one bounded provider query before refusing cleanly; and `search variant 'EGFR E746_A750del'` now routes to the exact gene+hgvsp filter where 1301's oracle previously made it a gene+condition candidate (the exact filter is an honest empty on the recorded capture).
- Root causes beyond triage, recorded in sdlc/records/1292-intronic-deletion-ranges-build.md: the transcript alias fallback existed but its identity filter dropped every ClinVar-only hit (NM_004333.6:c.1799T>A was not_found on main), and Mutalyzer 422s on intronic ranges.
- Resolutions proven live and from recorded captures: NM_000249.4:c.678-14_678-3del to chr3:g.37055906_37055917del (MLH1); both colon transcript forms; VariationID 577152; protein-range forms parse with exact spelling preserved. ClinVar-style parenthesized names keep refusing with a printed working form.
- Full Rust suite 3938 green at the code commit; spec page 14/14 standalone and 20/20 combined; registrations bumped (lifecycle 30 to 31, package 1387 to 1388).

## Outcome

`get variant` accepts transcript-qualified deletion ranges with intronic offsets and a ClinVar VariationID, or refuses with a message that prints a working input form.

## Evidence

- Starts from: Experiment 432 question clinvar-09 names `NM_000249.4(MLH1):c.678-14_678-3del`. Two agent legs could not reach it: `get variant` did not parse it and it did not appear in MLH1 search listings. the maintenance owner's historical consumer QA report 0002 confirmed on 2026-10-04 that `clinvar-09` still resolves on neither 0.9.1 nor the 1290 branch. Experiment 439 confirmed on 2026-10-05 that a bare ClinVar VariationID is also refused: `get variant 1463720 clinvar -j` exits 2 `invalid_argument` on main. The maintenance owner's consumer input message of 2026-10-04 adds a third form: `get variant 'EGFR E746_A750del' -j` is refused with `invalid_argument`, and protein-range deletions are how exon 19 deletions are usually written. Root cause for that form: `gene_protein_re` (`src/entities/variant/resolution.rs:245`) accepts only single-residue substitutions (`^GENE P123A$`); ranges never parse.
- Keeps: Every input form that works today keeps working.
- Changes: See Change detail.
- Proof: A table of input forms (substitution, deletion range, intronic offsets, VariationID, rsID) with expected results, as a spec page. Validated 2026-10-05 on main: the bare VariationID is refused; the ClinVar-style parenthesized name `NM_177438.3(DICER1) c.4449G>A` is refused; the colon form `NM_177438.3:c.4449G>A` parses but returns `not_found` for a real variant. The table adds the parenthesized and colon transcript forms, and the bare VariationID row uses 577152, the DICER1 variant from the 59-question set.
- Defers: HGVS parsing moves to a shared parser in 1.0. This ticket fixes the 0.9 behavior only.

## Change detail

Scope widened 2026-10-05 to carry the input forms QA and the KB lead found: intronic deletion ranges, bare VariationID, and protein-range deletions.

1. Record the exact failing commands and errors on 0.9.1.
2. Accept transcript-qualified deletion ranges with intronic offsets, or refuse with a message that prints a working form, such as a ClinVar VariationID or rsID lookup.
3. Accept a ClinVar VariationID as a `get variant` input if it is not accepted today.

## Review

- Design review: ACCEPT 2026-10-03 on the first pass, dispatch 26de9b8f (fresh SWE-2 researcher, read-only).
