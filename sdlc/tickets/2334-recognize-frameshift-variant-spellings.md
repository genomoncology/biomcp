# 2334 — Recognize frameshift variant spellings

Status: OPEN.

Milestone: 0.9.4

## Outcome

`biomcp variant articles` and the other variant commands accept protein frameshift spellings such as `FANCL p.T367fs` and their genomic duplication spellings, and anchor the variant search on them. A spelling BioMCP truly cannot parse fails as invalid_argument with the accepted forms named. A parsed variant with no matching record reports not_found, never the reverse.

- Proof: outside-in replay tests show `FANCL p.T367fs`, a p.X###fs*N form and the study's genomic duplication spelling each resolve and reach provider queries; an unparseable spelling returns invalid_argument naming accepted forms; a well-formed absent variant returns not_found.

## Evidence

- On 2026-10-21, research ran five fresh agents with BioMCP 0.9.4 (commit cdb9f4a1b) on literature searches for five hereditary cancer variants, with no answer key. Exact commands and output are in research's protected store under runs/0010/five-v1.
- Item 1: `FANCL p.T367fs` and its genomic duplication spelling were both rejected as invalid_argument, so the agent could not anchor a variant search. The study's other errors called this not_found, which was false.
- Ian asked on 2026-10-21 to clear every known issue in 0.9.4.
