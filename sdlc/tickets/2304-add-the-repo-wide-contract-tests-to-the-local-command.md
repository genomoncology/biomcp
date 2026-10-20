---
priority: 2
---

# 2304 — Add the repo-wide contract tests to the local command

Status: OPEN.

Milestone: 0.9.4

## Outcome

The single local command from 2290 gains a contract tier that runs every test reading the whole repository: changelog coverage, command and source registries, docs, fixture inventory, command discovery and source policy. Developers run it before pushing, so these meet a change before the shared gate.

- Proof: Planting a missing changelog citation, an unregistered source and an unregistered command each fails the local command.

Changes: `scripts/check-release-metadata.py`, `docs/reference/release-process.md`, `CHANGELOG.md`, `sdlc/records/2304-local-contract-tier.md`.

## Evidence

2161, 2196, 2265 and 2248 failed at the shared gate on repo-wide consistency tests the developer's focused proof did not run; 2224 would have been caught earlier. See the root cause record `sdlc/records/2299-root-cause-of-test-only-check-failures.md`.
