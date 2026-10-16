# 2114 — Check the structure residue against the chosen protein

Status: OPEN.

Milestone: 0.9.3

## Outcome

`biomcp variant structure` never reports a residue on a protein whose sequence holds a different reference amino acid there. `variant structure 'MUTYH G368D'` either places the change on a verified position of the protein it names or says the structural placement is unresolved and gives no residue-specific domain, PDB or AlphaFold claims. `variant structure 'BRAF V600E'` is unchanged.

## Evidence

Found 2026-10-16 by an outside QA audit of the v0.9.2 release binary on recorded public replies, and confirmed against main by reading the code and querying UniProt.

- `structure()` (`src/entities/variant/structure.rs:111`) takes the accession from the gene symbol alone (line 120) and builds the residue separately (line 123).
- `residue_summary` (320-376) takes the position from a matching dbNSFP alias or the request (348-351), reads the reference and alternate letters from the request text (356-359), and sets `requested_hgvsp_exact_match` whenever any alias spelling matched (360-361). Nothing compares the reference letter with the protein sequence; the UniProt record supplies only accession and length (273-275). Domains (131) and structures (164) use that position.
- UniProt Q9UIF7 on 2026-10-16 is 546 residues with S at 368 and G at 393. The audit reports `variant structure 'MUTYH G368D'` and `rs36053993` returning position 368, reference G, `requested_hgvsp_exact_match`, on Q9UIF7.
- 2036 and 2042 added a canonical UniProt residue check for `get variant`'s protein-change resolution in `get.rs`; `structure.rs` does not use it. 1260 covers `get variant` numbering, not structure.
- `structure.rs` is unchanged since v0.9.2.

- Starts from: the lines above and the 2036/2042 residue check in `get.rs`.
- Decisions:
  - Fetch the chosen protein's sequence and check the reference amino acid at the chosen position. A mismatch never yields an exact-match position.
  - On a mismatch, mark the placement unresolved and suppress residue-specific domain and structure rows, keeping the protein-level rows. Do not pick another alias position because its letter happens to match; a different position needs a recorded transcript-to-isoform link to the chosen protein, which this ticket does not build.
  - Reuse the 2036/2042 check rather than adding a second one.
- Changes: claims `src/entities/variant/structure.rs` `src/entities/variant/structure/**` `testdata/sources/uniprot/**` `testdata/sources/myvariant/**` `testdata/sources/capture-receipts.json` `spec/entity/variant.md` `CHANGELOG.md`.
- Proof: recorded-reply tests, each failing on current main: MUTYH G368D on Q9UIF7 never reports position 368 as an exact match; BRAF V600E on P15056 keeps position 600. Routine suite.
