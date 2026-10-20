---
priority: 2
---

# 2291 — Run the checks only GitHub CI ran in the local gate

Status: COMPLETE.

Milestone: 0.9.4

Reviews: revision 3394100, accept

## Outcome

Every check `.github/workflows/ci.yml` runs also runs in the local landing gate or the full main check, or is recorded here as deliberately release-only with its reason. Everyday GitHub Actions no longer run, so a check that lives only in ci.yml never runs.

Changes: `bin/lint`, `scripts/check-changelog-coverage.py`, `tests/test_changelog_coverage_check.py`, `tests/test_lint.py`, `sdlc/records/2291-local-ci-check-inventory.md`, `CHANGELOG.md`.

- Proof: A landing that omits its changelog citation fails the local gate. The inventory lists every ci.yml step and where it now runs.

## Evidence

`ci.yml` runs `scripts/check-changelog-coverage.py --branch`, but the local gates do not. 0.9.4 reached release with twelve landed tickets missing from the changelog, and later 2286 also landed without its citation; both were caught only at release metadata time.

Inventory ci.yml steps against the local gate and the full main check. Move each missing check into the local path. Keep ci.yml only where a public release needs it.

## Implementation boundary

Inventory every ci.yml step and map it to local execution or explicit release-only platform/load/toolchain reason. Add existing branch changelog and tracked no-climb checks to local lint. Integrate stable dated candidates without weakening development-section or missing-citation failure. Use real committed Git fixture branches for outside-in lint checks. No Makefile or release workflow edits; no local GitHub query. Preserve existing release-only Windows, stress and optimized checks with reasons; document protobuf generation tooling limits explicitly.

Keep: experiments/2102-biomcp-2258-final-live-replay — retain local-ci-parity/2291 raw preimage and corrected job evidence while release admission consumes this batch.

Landing alone: this changes the local gate implementation, which must land separately from other gate changes.
