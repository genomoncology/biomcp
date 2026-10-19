---
priority: 3
---

# 2244 — Find the single-CPU GenCC lease hang

Status: OPEN.

Milestone: 0.9.4

## Outcome

The two GenCC lease tests finish on one CPU, so the two-CPU workaround and the one-CPU ban can go.

## Evidence

`sdlc/issues/2026-09-25-single-cpu-affinity-deadlocks-the-handshake-child.md`. Worked around at `Makefile` near line 229; one-CPU pinning is refused by `tests/test_stress_lane_contract.py` near line 91. Cause unknown.

## Keeps

Bound the investigation: reproduce on one CPU, capture the stacks, and fix the cause. If the cause is in test scaffolding, fix the scaffolding. If it is a product deadlock, it is priority 1 and the PM hears at once.

## Proof

Both tests pass pinned to one CPU, the workaround is removed, and the issue is resolved.
