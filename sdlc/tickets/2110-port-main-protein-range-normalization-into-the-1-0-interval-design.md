# 2110 — Port main's protein-range normalization into the 1.0 interval design

Status: OPEN.

Milestone: 1.0

## Outcome

On the 1.0 line, a protein-range deletion resolves the same whether it is spelled with one-letter or three-letter residues (EGFR `E746_A750del` and `p.Glu746_Ala750del`), and duplicate ranges refuse, as they do on main, through the line's own interval alias design.

## Evidence

- The 2041 merge of main into the 1.0 line met a design clash in `src/entities/variant/resolution.rs`. Main normalizes protein-range deletions in `normalize_protein_change` through `normalize_protein_range_deletion`; the 1.0 line resolves intervals through its pinned interval alias design in `src/entities/variant/resolution/point_alias.rs`.
- The merge kept the line's design (commit b76f3f569 on the 2041 branch) and dropped main's code and two tests from `src/entities/variant/resolution/tests.rs`: `parse_variant_id_accepts_gene_protein_range_deletions` and `protein_range_deletions_normalize_across_one_and_three_letter_spellings`. Until this ticket lands, the 1.0 line lacks that behavior.

- Starts from: commit b76f3f569 (what the merge removed), main's `src/entities/variant/resolution.rs`, and the line's `point_alias.rs`.
- Decisions: express the range normalization and the duplicate refusal inside the line's interval alias design; do not restore main's separate regex path beside it.
- Changes: claims `src/entities/variant/resolution.rs` `src/entities/variant/resolution/**` on the 1.0 line only.
- Proof: both dropped tests return and pass unmodified, beside the line's existing interval tests, which keep passing.
- Order: after 2041 lands on the 1.0 line; independent of 2097.
- Keeps: main untouched; this ticket only changes the 1.0 line.
