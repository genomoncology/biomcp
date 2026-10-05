# 1292 — resolve intronic deletion ranges in get variant

Proposed 2026-10-03 by the BioMCP owner for the next 0.9 release.

Status: OPEN.

## Outcome

`get variant` accepts transcript-qualified deletion ranges with intronic offsets and a ClinVar VariationID, or refuses with a message that prints a working input form.

## Evidence

- Starts from: Experiment 432 question clinvar-09 names `NM_000249.4(MLH1):c.678-14_678-3del`. Two agent legs could not reach it: `get variant` did not parse it and it did not appear in MLH1 search listings. KB QA 0002 confirmed on 2026-10-04 that `clinvar-09` still resolves on neither 0.9.1 nor the 1290 branch. Experiment 439 confirmed on 2026-10-05 that a bare ClinVar VariationID is also refused: `get variant 1463720 clinvar -j` exits 2 `invalid_argument` on main. The KB lead's EGFR-wiki message of 2026-10-04 adds a third form: `get variant 'EGFR E746_A750del' -j` is refused with `invalid_argument`, and protein-range deletions are how exon 19 deletions are usually written. Root cause for that form: `gene_protein_re` (`src/entities/variant/resolution.rs:245`) accepts only single-residue substitutions (`^GENE P123A$`); ranges never parse.
- Keeps: Every input form that works today keeps working.
- Changes: See Change detail.
- Proof: A table of input forms (substitution, deletion range, intronic offsets, VariationID, rsID) with expected results, as a spec page.
- Defers: HGVS parsing moves to a shared parser in 1.0. This ticket fixes the 0.9 behavior only.

## Change detail

Scope widened 2026-10-05 to carry the input forms QA and the KB lead found: intronic deletion ranges, bare VariationID, and protein-range deletions.

1. Record the exact failing commands and errors on 0.9.1.
2. Accept transcript-qualified deletion ranges with intronic offsets, or refuse with a message that prints a working form, such as a ClinVar VariationID or rsID lookup.
3. Accept a ClinVar VariationID as a `get variant` input if it is not accepted today.

## Review

- Design review: ACCEPT 2026-10-03 on the first pass, dispatch 26de9b8f (fresh SWE-2 researcher, read-only).
