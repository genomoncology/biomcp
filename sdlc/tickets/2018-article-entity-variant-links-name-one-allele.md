# 2018 — Article entity variant links name one allele

Status: OPEN.

Milestone: 0.9.2

## Outcome

Each variant row in `biomcp article entities <PMID>` prints a `get variant` command that opens that row's variant. A row never points at an rsID that covers several alleles.

## Evidence

Filed 2026-10-07 from the review of the work since v0.9.1 (`sdlc/issues/2026-10-07-review-of-the-work-since-0.9.1.md`, finding 1).

- `src/transform/article/annotations.rs:73-94` prefers the rsID over the HGVS form for variant rows. `src/render/markdown/related/article_support.rs:90` prints `biomcp get variant <rsID>`.
- For PMID 30738221, the G12A, G12D and G12V rows all print `biomcp get variant rs121913529`. `biomcp get variant rs121913529 -j` returns `chr12:g.25398284C>T p.Gly12Asp` (G12D). MyVariant lists three alleles under that rsID. Two of the three rows open the wrong variant.
- `spec/entity/article-entities.md:34-37` records the shared rsID but checks only the G12C and G13C commands.

- Starts from: ticket 1296 and `sdlc/records/1296-article-entity-identifiers-build.md`.
- Keeps: identifiers for genes, diseases and chemicals; rsID links where the rsID names one allele.
- Changes: prefer the allele-specific identifier PubTator returns (ClinGen allele ID or gene plus HGVS) over the rsID. Fall back to a search command when no allele-specific form exists.
- Proof: the spec page checks the G12A, G12D and G12V commands and each opens its own variant; a test fails on `37631c357`.
- Defers: nothing.
