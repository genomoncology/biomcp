# 2345 — Check CDKN2A Q50P genomic normalization

Status: WITHDRAWN.

Opened as: 2026-10-21, Current main j4144 emits the correct GRCh37 allele; clinical sources confirm the GRCh38 coordinate is the same allele on another assembly. No wrong-coordinate defect reproduced; raw proof archived..

Milestone: 0.9.4

## Outcome

`biomcp variant articles 'CDKN2A Q50P'` reports the genomic allele that matches the protein change on the stated transcript and assembly, or says the equivalence is unconfirmed. If BioMCP emits a position one base off from the true allele, fix the normalization and every caller; if both spellings describe the same change, say so with the evidence. A clinical reviewer adjudicates the equivalence from HGVS and reference sources.

- Proof: an outside-in replay test for CDKN2A Q50P shows the adjudicated allele and transcript; a regression covers the off-by-one class on minus-strand genes if that is the cause.

## Evidence

- On 2026-10-21, research reran five hereditary cancer variants with blind agents on qualified BioMCP 0.9.4, with no search cap and direct provider calls allowed. Evidence: research repo commit 561cb47, sdlc/records/0041-tool-issues.tsv (27 observations with exact commands) and 0041-direct-only-findings.tsv (56 direct-only traces). These are observations, not verified defects; confirm on current main first and withdraw with evidence if it does not reproduce.
- Item 5: `biomcp --json variant articles 'CDKN2A Q50P' --limit 50 --debug-plan` emits chr9:g.21974678T>G, while the supplied allele is NC_000009.12:g.21974679T>G, one base apart. CDKN2A lies on the minus strand. A wrong genomic anchor could send searches to a different variant, so this comes first.

- M5 j4144 passed on ff4e1c04eaa40ad8ada96ac400545132718aa399. Exact study command returned chr9:g.21974678T>G, c.149A>C and rs587778189. MyVariant identity retains GRCh37 internally; clinical review confirms this coordinate and GRCh38 NC_000009.12:g.21974679T>G are the same allele. No product changes or extra regression justified. Serialized output omits assembly/transcript context; this is not evidence of a mislabeled coordinate. Yellow ticket-evidence/2345 manifest SHA256 616769e36e8c72599905cf1d1605ab89a1b491d6090b3c8ec9c735d0b847b87e retains command, stdout and job log. Owned lane/build copies removed.
