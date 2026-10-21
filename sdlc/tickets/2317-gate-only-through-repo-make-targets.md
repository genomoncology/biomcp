---
priority: 2
---

# 2317 — Gate only through repo make targets

Status: OPEN.

Milestone: 0.9.4

## Outcome

Every landing gate and full check runs the repo's own make targets. No gate runs a private script from an experiment folder.

- Proof: pm job command lines for gates name only make targets; the routine suite passes.

## Evidence

The 2299 gate ran a private landing_gate.py from an experiment folder on the M5, so the gate's real contents lived outside the repo. From Ian's process assessment of 2026-10-20.
