# 2350 — Trace papers direct provider queries find

Status: OPEN.

Milestone: 0.9.4

Depends on: 2353
Depends on: 2354
Depends on: 2355

## Outcome

For each of research's 42 direct-only candidates, starting with BRCA1 PMIDs 10486320 and 31788995, record whether an equivalent BioMCP query should find it, and why BioMCP missed it: fields, aliases, dates, page size or ranking. File one fix ticket per cause that applies to more than one paper. Fix every cause that applies to more than one paper in 0.9.4; Ian moved this in on 2026-10-21.

- Proof: a record lists each candidate with its cause class, and an outside-in test per fixed cause recovers its papers.

## Evidence

- On 2026-10-21, research reran five hereditary cancer variants with blind agents on qualified BioMCP 0.9.4, with no search cap and direct provider calls allowed. Evidence: research repo commit 561cb47, sdlc/records/0041-tool-issues.tsv (27 observations with exact commands) and 0041-direct-only-findings.tsv (56 direct-only traces). These are observations, not verified defects; confirm on current main first and withdraw with evidence if it does not reproduce.
- Item 1: direct Europe PMC queries recovered 2 of BRCA1's expert papers that no BioMCP search returned, plus 40 other direct-only candidates. Direct queries often differ in fields, aliases, dates or page size.
- Primary expert-paper recall rose from 12 of 43 to 24 of 43 with uncapped search and direct calls.
