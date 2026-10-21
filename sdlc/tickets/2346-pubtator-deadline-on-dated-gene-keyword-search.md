# 2346 — Finish dated gene and keyword PubTator searches

Status: OPEN.

Milestone: 0.9.4

## Outcome

A dated gene plus keyword article search completes its PubTator3 leg, or returns the other sources' results and names PubTator3 as timed out with the request it sent. Find why the leg exceeds the 12-second source guard when a simple PubTator3 search answers at once: query shape, page size, pagination or serial requests. Fix the cause within provider limits; do not just raise the guard.

- Proof: an outside-in test of the study command against recorded PubTator3 replies completes within the guard; a timeout case still returns partial results naming PubTator3.

## Evidence

- On 2026-10-21, research reran five hereditary cancer variants with blind agents on qualified BioMCP 0.9.4, with no search cap and direct provider calls allowed. Evidence: research repo commit 561cb47, sdlc/records/0041-tool-issues.tsv (27 observations with exact commands) and 0041-direct-only-findings.tsv (56 direct-only traces). These are observations, not verified defects; confirm on current main first and withdraw with evidence if it does not reproduce.
- Item 4: `biomcp --json search article -g CHEK2 -k 'G167R OR Gly167Arg' --date-from 2026-09-01 --limit 50 --max-per-source 50 --debug-plan` hits a 12-second timeout, and an exact retry does too. A simple PubTator3 search answers HTTP 200 from Blue and the Beelink.
- 2336 added one retry for transient failures. It does not cover a leg that is slow every time.
