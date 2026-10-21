# 2348 — Return rows for typed gene and date queries

Status: OPEN.

Milestone: 0.9.4

## Outcome

Article searches with a typed gene or date filter return the rows that the equivalent literal or long-name query finds, or explain in-band why a typed filter excluded them. Typed filters never silently return zero rows where the literal query finds matches.

- Proof: outside-in replay tests compare the study's typed and literal queries and show the typed query returns the expected rows or an in-band reason.

## Evidence

- On 2026-10-21, research reran five hereditary cancer variants with blind agents on qualified BioMCP 0.9.4, with no search cap and direct provider calls allowed. Evidence: research repo commit 561cb47, sdlc/records/0041-tool-issues.tsv (27 observations with exact commands) and 0041-direct-only-findings.tsv (56 direct-only traces). These are observations, not verified defects; confirm on current main first and withdraw with evidence if it does not reproduce.
- Item 7: typed gene and date queries return zero rows where literal or long-name queries return candidates. Exact commands are in 0041-tool-issues.tsv.

## Changes

- Share article search limitation notes across JSON, Markdown, discovery and debug plans. Explain Europe PMC's GENE_PROTEIN annotation-index restriction and the literal --keyword alternative without changing typed identity or date filtering.
- Compare the study's FANCL/date typed and literal queries through the CLI using the recorded literal response and a controlled empty annotation-index overlay; require an in-band explanation in JSON and Markdown.
