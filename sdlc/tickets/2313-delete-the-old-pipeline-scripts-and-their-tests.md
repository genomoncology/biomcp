---
priority: 2
---

# 2313 — Delete the old pipeline scripts and their tests

Status: OPEN.

Milestone: 0.9.4

## Outcome

`sdlc/project/` and `sdlc/scripts/` are removed together with the contract tests that only keep them alive. This replaces 2306's plan to move those scripts out of sdlc/.

- Proof: Neither folder exists; the routine suite passes; grep finds no reference to them outside archived records.

## Evidence

They are leftovers of the old automated pipeline, kept alive by contract tests, and they cause nine pm lint findings. From Ian's process assessment of 2026-10-20.
