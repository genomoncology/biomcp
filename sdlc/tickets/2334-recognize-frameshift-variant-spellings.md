# 2334 — Recognize frameshift variant spellings

Status: OPEN.

Milestone: 0.9.4

## Outcome

`biomcp variant articles` and the other variant commands accept protein frameshift spellings such as `FANCL p.T367fs` and their genomic duplication spellings, and anchor the variant search on them. A spelling BioMCP truly cannot parse fails as invalid_argument with the accepted forms named. A parsed variant with no matching record reports not_found, never the reverse.

- Changes: `src/entities/variant/resolution.rs`, `tests/test_variant_frameshift_inputs.py`, `tests/feature-links.json`, `docs/user-guide/variant.md`, `CHANGELOG.md`.

- Proof: outside-in replay tests show `FANCL p.T367fs`, a p.X###fs*N form and the study's genomic duplication spelling each resolve and reach provider queries; an unparseable spelling returns invalid_argument naming accepted forms; a well-formed absent variant returns not_found.

## Evidence

- On 2026-10-21, research ran five fresh agents with BioMCP 0.9.4 (commit cdb9f4a1b) on literature searches for five hereditary cancer variants, with no answer key. Exact commands and output are in research's protected store under runs/0010/five-v1.
- Item 1: `FANCL p.T367fs` and its genomic duplication spelling were both rejected as invalid_argument, so the agent could not anchor a variant search. The study's other errors called this not_found, which was false.
- Ian asked on 2026-10-21 to clear every known issue in 0.9.4.

- PM accepted the clinical review on 2026-10-21 (m3084). HGVS permits short protein frameshift descriptions. Normalize one/three-letter residues and Ter/* while preserving supplied spelling, omitted replacement or termination details, reference accession/version or its absence, uncertainty and predicted-parenthesis meaning. Keep `T367fs`, `T367fs*13`, `T367Nfs*13` and `T367Nfs*?` distinct. A complete provider allele may be a candidate for a partial protein query; prefix agreement alone proves no allele equivalence. `fs*1` describes an immediate nonsense stop, not a canonical frameshift. No genomic duplication follows from `FANCL p.T367fs`; same-allele claims still need the existing versioned-reference/genomic/build evidence or authoritative allele bridge. The clinical reviewer read [HGVS protein frameshift recommendations](https://hgvs-nomenclature.org/stable/recommendations/protein/frameshift/) and [HGVS general recommendations](https://hgvs-nomenclature.org/stable/recommendations/general/), version 21.1.5 dated 2026-09-17. The study's exact genomic duplication spelling remains unavailable.
