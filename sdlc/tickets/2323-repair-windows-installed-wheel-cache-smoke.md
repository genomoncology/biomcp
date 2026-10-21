---
priority: 1
---

# 2323 — Repair Windows installed-wheel cache smoke

Status: OPEN.

Milestone: 0.9.4

## Outcome

The installed Windows wheel passes the existing clean-home, keyless cache-stats smoke without weakening product checks. Name the demonstrated failure cause, repair its owning layer and retain the original failed rehearsal.

- Proof: Reproduce the failing boundary on current main through a build job with an outside-in control. Preserve the original cache-stats exit and JSON assertions and all trial, drug, absent-FAERS and packaged-skill checks. The final complete untagged release rehearsal must pass its actual Windows installed-wheel job.

Changes: `.github/workflows/release.yml`, `scripts/installed-wheel-smoke.sh`, `tests/test_installed_wheel_smoke.py`, `tests/test_release_check_counterparts.py`, `sdlc/tickets/2323-repair-windows-installed-wheel-cache-smoke.md`, `sdlc/records/2323-windows-wheel-smoke.md`, `CHANGELOG.md`.

## Evidence

Full untagged rehearsal 39541833383 uses published source `13eb246d8a566ead86bb1fbe2c6ac76dc45c0a28`. All archive and wheel builds passed. Windows wheel installation and its exact version assertion passed, but job 119048004261 failed the installed smoke: the initial `cache stats --json` returned error code `io`, message `I/O operation failed.`, and the script reported `installed wheel cache stats failed`. The failure is not yet attributed to product code or fixture paths. Fetch the completed job log through the GitHub jobs API; the whole run is still completing its other platform smokes.

## Development boundary

Read the ticket with pm brief, restate its outcome, inspect the actual complete diagnostic and reproduce the named boundary before code. Check how the shared smoke passes native executable, cache, home and temporary paths on Git Bash before assuming a product defect. Use the configured build hosts through pm job; do not add a diagnostic GitHub workflow or use pushes to trigger CI. A controlled test of the native path contract can establish a script defect, but label platform emulation honestly and leave actual Windows proof to the complete rehearsal. Keep the original smoke assertions, source-access policy, current semantic guards and no-publication rule. No tag, release, package upload or announcement. One writer, coherent review-ready push, then a fresh reviewer; no blind full-rehearsal rerun.
