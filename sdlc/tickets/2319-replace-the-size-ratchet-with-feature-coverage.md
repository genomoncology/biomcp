---
priority: 2
---

# 2319 — Replace the size ratchet with feature coverage

Status: OPEN.

Milestone: 0.9.4

## Outcome

`sdlc/features.md` lists each feature once. Each outside-in test names its feature, and coverage from those tests stays above the repo's floor. Code no outside test reaches is deleted or gains the test of the feature that needs it. The size ratchet is removed.

- Proof: features.md exists; every outside-in test names a listed feature; the coverage floor check runs in the routine suite; the ratchet is gone.

## Evidence

The shared notes retire size ratchets in favor of feature coverage; until this lands, treat the ratchet as a maximum. From Ian's process assessment of 2026-10-20.
