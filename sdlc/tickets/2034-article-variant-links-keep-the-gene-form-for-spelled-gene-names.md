# 2034 — Article variant links keep the gene form for spelled gene names

Status: OPEN.

Milestone: 0.9.2

## Outcome

A variant row in `biomcp article entities <PMID>` that has a gene and a protein change prints the gene-plus-change command even when the article spells the gene "K-ras", "K-RAS" or another non-symbol form. A row never falls back to an rsID link that can open a different allele because of how the article spelled the gene.

## Evidence

Filed 2026-10-08 from the fourth independent review of main at `cf8113f3a` (ticket 2035, finding 3).

- `src/transform/article/annotations.rs:200-224` takes the gene's most-mentioned text as its symbol. `src/entities/variant/resolution.rs:259` builds the gene-plus-change form only when that text is all capitals and digits. "K-ras" and "K-RAS" fail that check, so the row falls back to the rsID.
- A probe test with "K-ras" twice, "KRAS" once and a single G12A row on `rs121913529` printed `rsID rs121913529`. That link opens G12D, the case 2029 set out to stop.
- `biomcp article entities 37887282`, which writes the gene as K-Ras and K-RAS, prints `get variant rs121913530` for G12C, and bare `get variant G12V` and `get variant G12D` for the others. G12C opens the right allele only because MyVariant ranks it first.
- The 2029 Outcome and the spec page do not mention this case.

- Starts from: ticket 2029 and its landing `8eec444db`.
- Keeps: 2029's gene-first rule; rsID links for rows with no gene; the mention-text fallback.
- Changes: choose the most-mentioned gene text that passes the symbol check; when a row has a gene identifier and no usable symbol text, build the gene form from the gene identifier's official symbol or the mention text, not the rsID.
- Proof: a test with "K-ras" mentions and a single G12A row prints `biomcp get variant "KRAS p.G12A"` and fails on `cf8113f3a`; a test covers a gene with no symbol-shaped mention.
- Defers: gene-less multi-allele rsID rows, which 2029 records as a known limit.

## Review record (2034)

- Code review: ACCEPT 2026-10-09 (fresh reviewer, full head). Spelled gene names keep the gene form; the official-symbol fallback verified with source evidence; report-only P2s recorded in the review artifacts (/tmp/review-2034-code.md).
