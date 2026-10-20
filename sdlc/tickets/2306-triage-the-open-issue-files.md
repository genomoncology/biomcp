---
priority: 3
---

# 2306 — Triage the open issue files and make sdlc lint clean

Status: OPEN.

Milestone: 0.9.4

## Outcome

Every file in `sdlc/issues/` is either closed into `sdlc/archive/issues/` with a one-line disposition citing the landing that fixed it, or turned into an open ticket, or closed as moot with its reason. `pm lint` reports no findings for BioMCP's `sdlc/`. No product code changes.

- Proof: `ls sdlc/issues/*.md` lists only issues with an open ticket cited in them, and `pm lint` on main reports zero BioMCP findings.

## Evidence

On 2026-10-20 `sdlc/issues/` held 79 files: 69 from September and 4 from August. At least six say resolved or closed but were never moved, and 72 carry no status line. `pm lint` reported 383 findings: 285 layout-record (records named for ticket numbers no ticket file carries, such as 1180 to 1185), 55 layout-ticket-section, 18 ticket-evidence, 9 layout-script, 9 layout-issues, 5 layout-entry and 2 layout-status. Stale issue files hide which findings are still live, and lint noise hides real lint findings.

Mechanical moves and renames suit a cheap delegated job; a developer verifies each disposition against main before closing. Real still-open findings become tickets in the current milestone. Lands as records only.

The PM approved the canonical archive/issues destination. Preserve the zero-lint outcome. Necessary moves of the nine executable scripts out of SDLC and the checklist may update AGENTS.md, tests and other consumer references, but they are post-tag layout work separate from release qualification. Product behavior remains unchanged. Current admission records stay protected until their active work closes.

For this implementation, historical completed-build records with no ticket may move unchanged into sdlc/archive/records with Markdown links and active references corrected; they must not be relabeled as design studies. Preserve original evidence and quoted command results. Existing deferred ideas, recurrence-only watches and future preconditions can close as active defects with their original ruling and reason preserved in archive/issues; this does not complete a future task or revoke a recurrence watch. Unresolved real findings still require an open ticket.
