# 2347 — Make returned full-text paths readable

Status: OPEN.

Milestone: 0.9.4

## Outcome

Every file path BioMCP returns for full text or a conversion exists and is readable when the command exits, by the documented line-read command. If a path is temporary or written later, BioMCP does not return it.

- Proof: an outside-in test runs the full-text command, then immediately reads each returned path with the documented line-read command and succeeds; a failing write returns no path and names the failure.

## Evidence

- On 2026-10-21, research reran five hereditary cancer variants with blind agents on qualified BioMCP 0.9.4, with no search cap and direct provider calls allowed. Evidence: research repo commit 561cb47, sdlc/records/0041-tool-issues.tsv (27 observations with exact commands) and 0041-direct-only-findings.tsv (56 direct-only traces). These are observations, not verified defects; confirm on current main first and withdraw with evidence if it does not reproduce.
- Item 6: returned conversion paths exist, but immediate line reads of them fail.
