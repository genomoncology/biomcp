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

## Coordinator study brief

Restated outcome: diagnose the two recorded GenCC lease hangs on one CPU with one bounded reproduction and stacks, then remove the workaround only after fixing the cause. First read the existing issue and current code. This phase permits only reproduction/diagnosis and evidence retention; report exact proposed source files and failing command before edits.

Use the configured Linux test host through pm job --machine yellow because CPU affinity is Linux-specific. This is the explicitly requested occasional Linux qualification. Reuse normal routine preparation, including a real nonsymlink temporary directory; the unrelated Store::open Invalid results in2243 disappeared with that setup and are not this issue. Pin only your job/children to one allowed CPU, never a machine-wide process. Bound the reproduction, retain stacks and terminate only owned processes. If a product deadlock is demonstrated, tell the coordinator immediately so PM hears it as priority1. A test-scaffolding cause stays bounded to that scaffolding. Do not alter test assertions, add a framework, or widen into other GenCC failures. Raw evidence goes in the next numbered experiment folder on the executing host.2243 changes the GenCC test file, so any overlapping implementation must wait for that accepted source or a coordinator-prepared dependency base.
