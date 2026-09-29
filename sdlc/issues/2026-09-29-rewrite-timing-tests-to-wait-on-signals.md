# Rewrite timing tests to wait on signals

Moved to 1.0 by Ian's 2026-09-29 direction. Timing tests that sleep
or poll wall-clock windows break under CI's parallel load; each
break costs a gate cycle and a CI run. The rewrite replaces sleeps
with child-exit signals and readiness markers the way ticket 1252
converted the lease waits.

## Scope

Every test that waits on a wall-clock window: the stale-serve
freshness tests, the fixture readiness polls, the stdio
warn-once budget. The stress lane already pins the two-CPU
contract; the rewrite makes the waits deterministic instead.

## Owner and trigger

Owner: the developer agent on Ian's queue. Trigger: the 1.0
feature-track start, or the next timing-test flake, whichever
comes first.
