# Clinical rulings for the 0.9.4 review dispositions

Date: 2026-10-19. Clinical reviewers checked three choices made during 0.9.2 work without a recorded clinical ruling.

1. Disease abbreviations. `MM` leading as multiple myeloma, `MF` pointing to myelofibrosis, and `HD` refused stay as they are. NCI and SEER sources confirm the readings, and callers refuse identity the sources do not support. No change.
2. Variant headline date. Showing the newest evaluation date among the ClinVar records that carry the shown classification (ticket 2022) matches NCBI's distinction between evaluation and submission dates. No change.
3. Protein change on another transcript. `TP53 R116Q` resolving to p.Arg248Gln with an other-transcript note, and the ClinVar-free `BRCA1 Q1878R` resolving to p.Gln1857Arg, are correct: each has paired versioned transcript projections onto one allele. Keep both. Acceptance must require that same-allele proof; a canonical residue mismatch or out-of-range position alone is not enough. Ticket 2261 adds that guard. Do not refuse non-MANE or ClinVar-free answers wholesale.

Sources: ClinVar VCV000012356.113 (https://www.ncbi.nlm.nih.gov/clinvar/variation/12356/); HGVS nomenclature, RefSeq background 21.1.5 (https://hgvs-nomenclature.org/stable/background/refseq/); MANE (https://www.ncbi.nlm.nih.gov/refseq/MANE/); and `sdlc/decisions/2026-10-18-transcript-aliases-need-the-same-genomic-change.md`.
