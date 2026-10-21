# 2345 — Check CDKN2A Q50P genomic normalization

Status: OPEN.

Milestone: 0.9.4

## Outcome

`biomcp variant articles 'CDKN2A Q50P'` reports the genomic allele that matches the protein change on the stated transcript and assembly, or says the equivalence is unconfirmed. If BioMCP emits a position one base off from the true allele, fix the normalization and every caller; if both spellings describe the same change, say so with the evidence. A clinical reviewer adjudicates the equivalence from HGVS and reference sources.

- Proof: an outside-in replay test for CDKN2A Q50P shows the adjudicated allele and transcript; a regression covers the off-by-one class on minus-strand genes if that is the cause.

## Evidence

- On 2026-10-21, research reran five hereditary cancer variants with blind agents on qualified BioMCP 0.9.4, with no search cap and direct provider calls allowed. Evidence: research repo commit 561cb47, sdlc/records/0041-tool-issues.tsv (27 observations with exact commands) and 0041-direct-only-findings.tsv (56 direct-only traces). These are observations, not verified defects; confirm on current main first and withdraw with evidence if it does not reproduce.
- Item 5: `biomcp --json variant articles 'CDKN2A Q50P' --limit 50 --debug-plan` emits chr9:g.21974678T>G, while the supplied allele is NC_000009.12:g.21974679T>G, one base apart. CDKN2A lies on the minus strand. A wrong genomic anchor could send searches to a different variant, so this comes first.
