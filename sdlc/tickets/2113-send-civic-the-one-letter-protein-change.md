# 2113 — Send CIViC the one-letter protein change

Status: OPEN.

Milestone: 0.9.3

## Outcome

`biomcp get variant 'BRAF V600E' civic` and `get variant 'EGFR C797S' civic` query CIViC with the molecular profile names `BRAF V600E` and `EGFR C797S` and show the accepted evidence CIViC holds, where today they query the three-letter names and show an empty section.

## Evidence

Found 2026-10-16 by an outside QA audit of the v0.9.2 release binary on recorded public replies, and confirmed against main by reading the code and querying CIViC.

- `civic_molecular_profile_name` (`src/entities/variant/get.rs:1554-1573`) strips only `p.` from the three-letter HGVSp (line 1566) and sends `"{gene} {change}"`, so CIViC receives `BRAF Val600Glu`. `add_civic` (1575-1608) reports CIViC's healthy zero reply as an empty section.
- `amino_acid_one_letter` (`src/entities/variant/resolution.rs:536`) already converts residues, but this path does not use it.
- CIViC GraphQL on 2026-10-16, `status: ACCEPTED`: `BRAF Val600Glu` 0 evidence and 0 assertions; `BRAF V600E` 115 and 3; `EGFR Cys797Ser` 0 and 0; `EGFR C797S` 2 and 0.
- `civic_molecular_profile_name_prefers_gene_and_hgvs_p` (`src/entities/variant/get/tests.rs:560`) feeds the one-letter `p.V600E`, so it never sees the real input. `src/cli/tests/printed_card_commands.rs:300-304` pins the empty reply as expected behavior.

- Starts from: the lines above and `src/sources/civic.rs`.
- Decisions:
  - Convert the protein change to one-letter notation with the existing helper before building the profile name. A change the helper cannot convert keeps today's name.
  - Compound or complex alterations keep their own names; this ticket changes only simple protein changes.
- Changes: claims `src/entities/variant/get.rs` `src/entities/variant/get/tests.rs` `src/cli/tests/printed_card_commands.rs` `testdata/sources/civic/**` `testdata/sources/capture-receipts.json` `CHANGELOG.md`.
- Proof: the name test feeds the three-letter HGVSp the variant record carries and expects `BRAF V600E` and `EGFR C797S`; recorded CIViC replies for both show nonzero accepted evidence on the card; the printed-card test stops pinning an empty CIViC section. Each fails on current main. Order: lands before or after 2112 with a rebase; both touch `get.rs` in separate functions.
