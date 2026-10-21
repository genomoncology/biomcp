# 2349 — Report empty supplement containers honestly

Status: OPEN.

Milestone: 0.9.4

## Outcome

When a supplement route returns HTTP 200 with an empty XML container, BioMCP reports that the source listed no supplements, not that none exist, and names any other route the user can try.

- Proof: outside-in replay tests with an empty container show the in-band 'source listed none' status and wording; a populated container is unchanged.

## Evidence

- On 2026-10-21, research reran five hereditary cancer variants with blind agents on qualified BioMCP 0.9.4, with no search cap and direct provider calls allowed. Evidence: research repo commit 561cb47, sdlc/records/0041-tool-issues.tsv (27 observations with exact commands) and 0041-direct-only-findings.tsv (56 direct-only traces). These are observations, not verified defects; confirm on current main first and withdraw with evidence if it does not reproduce.
- Item 8: supplement routes report empty XML containers with HTTP 200. Neither proves that no supplements exist.
