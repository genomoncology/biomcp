---
priority: 3
---

# 2244 — Find the single-CPU GenCC lease hang

Status: OPEN.

Milestone: 0.9.4

Reviews: revision 7a99c77, accept

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

## Coordinator implementation brief

- Changes: `src/test_support.rs` `Makefile` `tests/test_stress_lane_contract.py` `CHANGELOG.md` `sdlc/issues/2026-09-25-single-cpu-affinity-deadlocks-the-handshake-child.md` and this ticket.

Restated outcome: make the existing readiness handshake recognize its actual raw marker with one test worker, then remove the two-CPU workaround without weakening readiness assertions. The coordinator merged accepted2243 scaffold dependency before granting this claim; do not reimplement its bounded joins.

Linux study retained in experiments/171-biomcp-2244-single-cpu. First driver1792423006178-132612 completed but all six individual cells failed at Store::open before the handshake: workspace ancestors were group-writable. Corrected1792423346884-138831 used private non-symlink temporary storage beneath sticky system tmp without altering shared permissions. Both one-CPU tests failed readiness watchdog; both two-CPU controls and both one-CPU RUST_TEST_THREADS=2 controls passed. Trace proves libtest wrote its 56byte test-prefix before the child wrote entered plus newline; the parent received them as one line and rejected exact marker comparison. The child successfully wrote readiness and then waited for stdin. This is a test-scaffold framing defect, not a demonstrated product deadlock.

Smallest proposed repair prepends a newline at the existing raw readiness-marker writer. Preserve exact whole-line comparison and watchdog bounds. Use the two existing one-CPU lease cases as failing outside-in cases; show both pass after repair, confirm focused routine controls, remove Makefile two-CPU workaround and invert the one-CPU prohibition contract. Retain red traces, green commands and manifests privately on executing host. No sleeps, broadened marker matching, shared directory permission change or product workaround. Final gate-definition change lands separately after2243.

Keep: experiments/171-biomcp-2244-single-cpu — Retain cited raw proof until release evidence is explicitly released.

## Focused repair proof

Source92fe6d624bf57e25b4eadeb290723488b8aedc5d prepends a newline before the raw readiness marker, restores one-CPU stress pinning and updates the existing contract. Exact marker comparison, watchdogs, bounded joins and product logic remain unchanged. Yellow1792423602245-139578 passed both existing lease regressions and both cancellation/publication controls, each on one CPU. Build-host1792423617395-95233 passed formatting, Ruff, quality boundaries, warnings-denied all-target Clippy and all seven stress contracts. Both applied the same private patch to794b84fc7; retained diffs match the pushed source. Raw logs, commands, manifests and original red syscall traces remain in experiments/171-biomcp-2244-single-cpu on the Linux execution host, including the copied build-host mechanical evidence.
