# 2347 — Make returned full-text paths readable

Status: WITHDRAWN.

Opened as: 2026-10-21, Current main j4148 returned an existing conversion and immediate documented line reads succeeded. No unreadable returned path reproduced; raw proof archived..

Milestone: 0.9.4

## Outcome

Every file path BioMCP returns for full text or a conversion exists and is readable when the command exits, by the documented line-read command. If a path is temporary or written later, BioMCP does not return it.

- Proof: an outside-in test runs the full-text command, then immediately reads each returned path with the documented line-read command and succeeds; a failing write returns no path and names the failure.

## Evidence

- On 2026-10-21, research reran five hereditary cancer variants with blind agents on qualified BioMCP 0.9.4, with no search cap and direct provider calls allowed. Evidence: research repo commit 561cb47, sdlc/records/0041-tool-issues.tsv (27 observations with exact commands) and 0041-direct-only-findings.tsv (56 direct-only traces). These are observations, not verified defects; confirm on current main first and withdraw with evidence if it does not reproduce.
- Item 6: returned conversion paths exist, but immediate line reads of them fail.

- Gator j4148 passed on ff4e1c04eaa40ad8ada96ac400545132718aa399. `biomcp --json get article 22419737 fulltext` returned an existing 42475-byte/195-line conversion; immediate `biomcp get article 22419737 fulltext --lines 1:219` exited 0. For 12533788 the first provider bot challenge returned no path, and subsequent line read succeeded. No unreadable returned path reproduced. Yellow ticket-evidence/2347 reproduction tar SHA256 a1b673069ad264593109b38ca55c7a94361e38f903664aa3c8d35d3ff823fd51; j4148.log SHA256 58efcdc8d2ff6ce2cfbc16b25ba85964f996fb889baf7ad59f5adf498b5e21a7. No source changes; owned lane/build copies removed.
