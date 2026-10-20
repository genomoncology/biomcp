---
priority: 2
---

# 2304 — Add the repo-wide contract tests to the local command

Status: OPEN.

Milestone: 0.9.4

Reviews: revision 0016493, accept

Reviews: revision c2e7c63, accept

Reviews: revision 2bc747c, accept

## Outcome

The single local command from 2290 gains a contract tier that runs every test reading the whole repository: changelog coverage, command and source registries, docs, fixture inventory, command discovery and source policy. Developers run it before pushing, so these meet a change before the shared gate.

- Proof: Planting a missing changelog citation, an unregistered source and an unregistered command each fails the local command.


## Evidence

Changes: `CHANGELOG.md`, `docs/reference/release-process.md`, `scripts/check-release-metadata.py`, `sdlc/records/2299-root-cause-of-test-only-check-failures.md`, `sdlc/records/2304-local-contract-tier.md`, `sdlc/tickets/2299-keep-a-failed-test-cases-state.md`, `sdlc/tickets/2300-share-the-best-effort-cache-reuse-helper.md`, `sdlc/tickets/2301-finish-the-paused-clock-repairs-and-forbid-the-mix.md`, `sdlc/tickets/2302-replace-fixed-test-deadlines-with-event-waits.md`, `sdlc/tickets/2303-share-one-rust-test-http-server.md`, `sdlc/tickets/2304-add-the-repo-wide-contract-tests-to-the-local-command.md`

2161, 2196, 2265 and 2248 failed at the shared gate on repo-wide consistency tests the developer's focused proof did not run; 2224 would have been caught earlier. See the root cause record `sdlc/records/2299-root-cause-of-test-only-check-failures.md`.
