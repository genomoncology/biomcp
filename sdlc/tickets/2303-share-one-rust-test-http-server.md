---
priority: 2
---

# 2303 — Share one Rust test HTTP server

Status: OPEN.

Milestone: 0.9.4

## Outcome

Rust tests that need a local HTTP server use one shared test server that handles WouldBlock and connection resets, shuts down without panicking in Drop, and reports its own failures as test errors. Hand-written per-test socket servers move to it.

- Proof: The shared server passes a parallel stress run with clients that disconnect early; a grep finds no other hand-rolled TcpListener test server.

## Evidence

The 2294 batch gate failed when a hand-written fixture server got WouldBlock and panicked in Drop. 2229 and 2260 earlier cleaned up leaked replay servers built the same ad hoc way. See the root cause record `sdlc/records/2299-root-cause-of-test-only-check-failures.md`.
