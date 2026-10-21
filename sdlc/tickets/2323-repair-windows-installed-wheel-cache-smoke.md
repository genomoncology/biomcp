---
priority: 1
---

# 2323 — Repair Windows installed-wheel cache smoke

Status: COMPLETE.

Milestone: 0.9.4

Reviews: revision 7cfdfef, accept

Reviews: revision 7cfdfef, accept

Reviews: revision 3b5877b, accept

Reviews: revision 9bd41c0, accept

## Outcome

The installed Windows wheel passes the existing clean-home, keyless cache-stats smoke without weakening product checks. Name the demonstrated failure cause, repair its owning layer and retain the original failed rehearsal.

- Proof: Reproduce the failing boundary on current main through a build job with an outside-in control. Preserve the original cache-stats exit and JSON assertions and all trial, drug, absent-FAERS and packaged-skill checks. The final complete untagged release rehearsal must pass its actual Windows installed-wheel job.

Changes: `.github/workflows/release.yml`, `scripts/installed-wheel-smoke.sh`, `tests/test_installed_wheel_smoke.py`, `tests/test_release_check_counterparts.py`, `tests/test_release_workflow_provenance.py`, `sdlc/tickets/2323-repair-windows-installed-wheel-cache-smoke.md`, `sdlc/records/2323-windows-wheel-smoke.md`, `CHANGELOG.md`.

## Evidence

Full untagged rehearsal 39541833383 uses published source `13eb246d8a566ead86bb1fbe2c6ac76dc45c0a28`. All archive and wheel builds passed. Windows wheel installation and its exact version assertion passed, but job 119048004261 failed the installed smoke: the initial `cache stats --json` returned error code `io`, message `I/O operation failed.`, and the script reported `installed wheel cache stats failed`. The failure is not yet attributed to product code or fixture paths. Fetch the completed job log through the GitHub jobs API; the whole run is still completing its other platform smokes.

### Repair and review evidence

The repair keeps the installed-wheel checks intact and supplies the actual Windows account identity to the isolated smoke process. The smoke's `env -i` removed `USERNAME`, while Windows cache directory security in `src/cache/private.rs::secure_entry` requires it before calling `icacls`. The path conversion already exists; no path defect or product defect was demonstrated. M5 job j1047 reproduced the missing-identity boundary on source `30f0543ee` with a caller regression payload. The final helper preserves only this required identity alongside the existing operating-system environment. A missing identity still fails the control. The existing Windows wheel job now also invokes cache statistics from native PowerShell with a separate cold, keyless home. Actual Windows proof remains pending the complete untagged rehearsal.

The first review ran the release-counterpart inventory, focused installed-smoke controls and mechanical checks. It did not inspect or run the separate provenance contract for ordered steps, conditions, shells and step hashes. The missing ordered-step entry prevented the mutation controls from reaching their intended assertions. Candidate job j1194 failed this contract; its Rust stage passed. This review gap requires a complete routine-suite pass before renewed acceptance. Merging the two inventories is deferred to the next release.

Gator job j1195 reproduced the missing ordered-step entry on `bf6e93b3dc3e53e037b62db0632e5543c80bc053`. Repair source `ed2eabfb6ab4b9d963d00b0dc37eebccd802591f` adds the native step to that inventory, pins its existing body, and permits its Windows condition and PowerShell shell only for that named step. Three mutation controls reject changes to those boundaries. The workflow and product code did not change in this repair. Focused job j1196 used this test payload on `bf6e93b3d`: 168 controls passed and two native-builder controls failed because the direct interpreter invocation omitted the prepared environment's `python` executable from PATH. That attempt remains failed; the controls were not weakened.

The required complete routine command `env RUSTUP_TOOLCHAIN=1.95.0 make test` passed in Gator job j1198 on published repair source `ed2eabfb6ab4b9d963d00b0dc37eebccd802591f`, which contains main `13eb246d8a566ead86bb1fbe2c6ac76dc45c0a28`. It exited 0 in 743320 ms: 4287 Rust cases passed with 34 skipped; 1834 Python cases passed with 12 skipped; the strict offline documentation build passed. This includes the actual pipeline contract, its mutation controls and the two native-builder controls that lacked the prepared interpreter in j1196. Preceding job j1197 ended naturally with setup timeout 124 and runtime 0 ms while checkout waited on a nested GitHub SSH fetch. No tests started in that attempt. The PM authorized the unchanged-source retry only after that terminal result; the failed setup remains retained as a setup timeout, not a test failure.

Gator mechanical job j1205 passed Ruff for the changed provenance test file and printed `rustc 1.95.0 (59807616e 2026-04-14)` under the same toolchain selection. Raw caller logs for j1195 through j1198 and j1205 are retained under `yellow:data/biomcp/release-evidence/0.9.4/windows-wheel-smoke/2323/`; copied file hashes matched their originals. The passing j1198 caller log SHA-256 is `779151b73dd344e1618bd4ed46d9cdd8ae72673da9a2ff373f55fbff552137ee`. These checks do not claim an actual Windows pass.

## Development boundary

Read the ticket with pm brief, restate its outcome, inspect the actual complete diagnostic and reproduce the named boundary before code. Check how the shared smoke passes native executable, cache, home and temporary paths on Git Bash before assuming a product defect. Use the configured build hosts through pm job; do not add a diagnostic GitHub workflow or use pushes to trigger CI. A controlled test of the native path contract can establish a script defect, but label platform emulation honestly and leave actual Windows proof to the complete rehearsal. Keep the original smoke assertions, source-access policy, current semantic guards and no-publication rule. No tag, release, package upload or announcement. One writer, coherent review-ready push, then a fresh reviewer; no blind full-rehearsal rerun.
