---
priority: 2
---

# 2290 — Run every release-metadata check in one local command

Status: OPEN.

Milestone: 0.9.4

## Outcome

One local command runs every version, changelog, runbook and release-metadata check, Python and Rust, against a candidate with a proposed version and tag. A developer runs it before any metadata gate, so a gate never stops on a second metadata check.

- Proof: Running the command on the 0.9.4 metadata commit's parent with the 0.9.4 changes applied reproduces both original failures in one run.

## Evidence

The 0.9.4 stable-metadata change (2262) stopped twice in gates: first on the runbook version check in the quality ratchet, then on the development-cycle changelog test. Each stop cost a full gate. The fix that finally worked was a focused pass of 134 Python and 8 Rust metadata checks run together.

Collect the existing checks; add none. The release checklist names the command.
