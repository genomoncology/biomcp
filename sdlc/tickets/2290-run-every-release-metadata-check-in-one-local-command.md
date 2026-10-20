---
priority: 2
---

# 2290 — Run every release-metadata check in one local command

Status: OPEN.

Milestone: 0.9.4

Reviews: revision 6a057f3, accept

Reviews: revision 854f005, accept

Reviews: revision 045f43d, accept

Reviews: revision a4a2e13, accept

## Outcome

One local command runs every version, changelog, runbook and release-metadata check, Python and Rust, against a candidate with a proposed version and tag. A developer runs it before any metadata gate, so a gate never stops on a second metadata check.

Changes: `scripts/check-release-metadata.py`, `sdlc/release-checklist.md`, `docs/reference/release-process.md`, `sdlc/records/2290-collected-release-metadata-check.md`, `CHANGELOG.md`.

- Proof: Running the command on the 0.9.4 metadata commit's parent with the 0.9.4 changes applied reproduces both original failures in one run.

## Evidence

The 0.9.4 stable-metadata change (2262) stopped twice in gates: first on the runbook version check in the quality ratchet, then on the development-cycle changelog test. Each stop cost a full gate. The fix that finally worked was a focused pass of 134 Python and 8 Rust metadata checks run together.

Collect the existing checks; add none. The release checklist names the command.

## Implementation boundary

Collect the existing metadata tests and actual runbook audit in one command; add no new checks. Interface accepts candidate SHA, version and tag. Run all groups after failures and report every failed group. Reproduce the two original stable-metadata failures together at fc1e2deb62dc7e122aad7677d7f4c703d7f3f21c using the existing tests and audit, then pass on the corrected candidate. Do not edit Makefile or the release workflow.

Keep: experiments/2102-biomcp-2258-final-live-replay — retain metadata-command/2290 raw preimage and corrected job evidence while release admission consumes this batch.

Landing alone: This adds the release-metadata gate collector entry point. Gate implementation changes land separately.
