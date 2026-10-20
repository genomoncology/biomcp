---
priority: 2
---

# 2301 — Finish the paused-clock repairs and forbid the mix

Status: OPEN.

Milestone: 0.9.4

## Outcome

No test pauses or auto-advances the Tokio clock while real sockets are pending. src/cli/system/tests/dev2_contracts.rs:265 and src/entities/author/papers.rs:1172 follow the 2276 and 2230 pattern (finish real IO, then pause). One small check in the routine gate rejects a test that combines start_paused or time::pause with TcpListener or a real HTTP server.

- Proof: The check fails on a planted test that mixes the two and passes on main; both named tests pass repeatedly under parallel load.

## Evidence

2230 and 2276 failed full runs when paused time advanced while real TCP was pending; two more tests still use the same pattern. See the root cause record `sdlc/records/2299-root-cause-of-test-only-check-failures.md`.
