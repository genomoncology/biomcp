---
priority: 1
---

# 2293 — Qualify real replay response formats

Status: OPEN.

Milestone: 0.9.4

## Outcome

The existing live replay checker handles the real response formats needed by the approved recorded-command coverage without claiming stronger verification than it performs.

Changes: `scripts/replay_drift.py`, `tools/check-replay-live`, `tests/test_replay_live_drift.py`, `sdlc/records/2293-real-replay-format-qualification.md`, `CHANGELOG.md`.

## Evidence

The current checker accepts only JSON and XML, rejects XML declarations containing a DOCTYPE, and blocks the genuine JPEG, DOC, ZIP and HTML responses needed for the CC BY open-access article fixture. The current S3 metadata serves JSON bytes as binary/octet-stream. These observed headers must remain truthful; the checker cannot relabel them to make validation pass. Existing source and size/security limits remain in force.

## PM decision

The PM approved a bounded 0.9.4 repair in the mailroom message replay-qualification-needs-real-format-disposition on 2026-10-20:

1. Accept XML with a DOCTYPE while refusing entity expansion and external resources.
2. Check every non-JSON, non-XML reply, including JPEG, DOC, ZIP, HTML and octet-stream, for status, content type exactly as served and file-signature match. Report the literal scope status/type/signature only; never label that a full structural pass.
3. Add meaningful outside-in malformed controls for each. Add no document parser or framework.
4. Do not reclassify mislabeled bodies. S3 octet-stream JSON remains compared as served.

## Proof

Restate the outcome, exact old failure and intended files before code. Exercise the real checker entry point against local controlled replies: supported formats fail before correction, then pass with the limited scope visible; wrong status/type/signature and malformed replies fail. Benign XML DOCTYPE succeeds, while internal entity expansion and external resource attempts remain refused without any external fetch. Reuse unchanged JSON/XML behavior controls. No public request or new full live collection is required; every test runs through pm job on the build machine.

Keep: experiments/2102-biomcp-2258-final-live-replay — retain real-format-qualification/2293 raw preimage/corrected job evidence while final release admission consumes it.

Landing alone: this changes the release qualification checker implementation and must pass its own permitted landing gate.
