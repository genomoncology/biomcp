---
priority: 2
---

# 2314 — Remove baseline re-stamping checkers

Status: OPEN.

Milestone: 0.9.4

## Outcome

Every checker whose only effect is re-stamping a stored baseline file is deleted with its baseline, applying idea 2056. Checkers that block a real regression stay.

- Proof: A list of removed checkers with one line each on why none blocks a regression; the routine suite passes.

## Evidence

Idea 2056 found baseline re-stamping wastes commits; 67 percent of 2,452 commits since 2026-10-16 touch only sdlc/. From Ian's process assessment of 2026-10-20.
