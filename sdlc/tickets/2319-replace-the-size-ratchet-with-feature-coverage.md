---
priority: 2
---

# 2319 — Replace the size ratchet with feature coverage

Status: OPEN.

Milestone: 0.9.4

Depends on: 2316, 2318

## Outcome

`sdlc/features.md` lists each feature once in caller words. Every existing outside-in test names a listed feature, including drug, managed-data, study and transport behavior; derive links from golden command metadata where it already exists. The routine gate checks these links cheaply and adds no instrumented build.

One corrected outside-in-only coverage collection excludes test and generated code and counts no failed tests as proof. Its valid result establishes the initial configured floor. The floor check runs in the nightly and release targets from 2315. Preserve semantic regression guards and rename their tool to describe them. The old aggregate size ratchet was already removed by 2076.

List unreached production code. Give each item a feature test or a separate reviewed deletion; do not delete product code because one collection misses it. This ticket delivers the complete scope rather than a golden-only initial suite or a later implementation ticket.

- Proof: Every selected outside-in test maps to a listed feature; planted unknown or missing links fail. The corrected collection passes, reports its actual source and exclusions, and establishes the floor checked by the nightly and release targets. Routine feature-link validation adds no instrumented build.

## Evidence

The shared notes retire size ratchets in favor of feature coverage; until this lands, treat the ratchet as a maximum. From Ian's process assessment of 2026-10-20.

PM ruling: m2513 answers 1–3 on 2026-10-20 define the valid baseline, preserve semantic guards and put instrumented coverage in nightly/release rather than routine.
