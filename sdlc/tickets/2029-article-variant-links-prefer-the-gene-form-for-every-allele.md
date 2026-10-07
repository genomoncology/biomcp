# 2029 — Article variant links prefer the gene form for every allele

Status: OPEN.

Milestone: 0.9.2

## Outcome

Every variant row in `biomcp article entities <PMID>` that has a gene symbol and a protein change prints the gene-plus-change command. This holds when the article mentions only one allele of a multi-allele rsID. A row never prints an rsID link that opens a different allele.

## Evidence

Filed 2026-10-07 from the second review of the work since v0.9.1 (`sdlc/issues/2026-10-07-second-review-of-the-work-since-0.9.1.md`, finding 1).

- Ticket 2018 landed in `6b504219e` and reads complete. Its Outcome says "A row never points at an rsID that covers several alleles."
- The landed fix notices a shared rsID only when the same article names two of its alleles. A probe test with one article that mentions only KRAS G12A (`rs121913529`, gene 3845) still prints `biomcp get variant rs121913529`. That command opens G12D.
- Ticket 2018's deferral says BioMCP has no offline rsID-to-alleles table. The table is not needed. When a row has a gene symbol and a protein change, the gene-qualified form names the right allele.
- Replacing `distinct_change_key(hgvs)` with the raw HGVS text in `MutationContext::collect` (`src/transform/article/annotations.rs`) passes all 89 article tests. No test covers one allele written two ways (`p.G12D` and `p.Gly12Asp`).

- Starts from: ticket 2018 and its landed fix in `6b504219e`.
- Keeps: rsID links for rows with no gene symbol or no protein change; the mention-text fallback; 1296's identifier and namespace contract.
- Changes: prefer the gene-plus-change command whenever both parts exist, whatever else the article mentions.
- Proof: a single-mention G12A article prints `biomcp get variant "KRAS p.G12A"`, and a test fails on `6b504219e`. A test covers one allele written as `p.G12D` and as `p.Gly12Asp`.
- Defers: ClinGen allele ID support in `get variant`.
