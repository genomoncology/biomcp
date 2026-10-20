---
priority: 2
---

# 2316 — Run the landing gate within its booking

Status: OPEN.

Milestone: 0.9.4

## Outcome

The landing gate runs its Rust and Python parts one after the other, or books the cores and memory both need at once, with test workers set from the booking rather than the machine's core count.

- Proof: The gate's booked memory and cores cover its measured peak; a gate run beside another job on the M5 passes.

## Evidence

The gate runs Rust and Python together with workers sized from the core count on shared machines, which explains tests that pass only on a quiet machine (root cause record 2299). From Ian's process assessment of 2026-10-20.

Changes: `Makefile`, `AGENTS.md`, `CONTRIBUTING.md`, `RUN.md`, `architecture/technical/overview.md`, `tests/test_routine_cargo_feature_contract.py`, `CHANGELOG.md`.
