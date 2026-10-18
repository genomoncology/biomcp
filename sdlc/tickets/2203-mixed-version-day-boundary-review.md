---
priority: 3
---

# 2203 — Review mixed-version use across the daily clean

Status: OPEN.

Milestone: 0.9.4

## Outcome

An agent test review alternates v0.9.2 and current BioMCP on one cache with a small size limit across at least two daily cleans and checks the saved byte total and size enforcement.

- Evidence: the 0.9.3 release review could not run past 24 hours of wall time.
- Changes: `sdlc/records/2203-mixed-version-day-boundary-review.md` `sdlc/tickets/2203-mixed-version-day-boundary-review.md`. Review and report only; file any product defect separately.
- Proof: recorded results; file defects as tickets.

## Progress

- 2026-10-18 started

Pre-execution scope: begin from qualified product87d53c6782aaf07dcba4269f7dbd573a5ef00b58 and fixed published v0.9.2, reusing existing cache fixtures and day-boundary mechanisms. Prepare a bounded shared-cache scenario and artifact identities before executing; do not introduce clock frameworks or product hooks. Development builds/tests run only through pm job. No execution until coordinator admission release.
