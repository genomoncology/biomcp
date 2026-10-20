---
priority: 2
---

# 2315 — Run release-only checks before the release

Status: OPEN.

Milestone: 0.9.4

Depends on: 2316, 2319

## Outcome

Checks that today run first at release rehearsal run earlier: the stress lane, the optimized panic-recovery test, protobuf generation and the long suites run nightly on main through pm job, and the Windows checks run once per milestone. A test fails when a release.yml step has no local or nightly counterpart, so a check that stops running is noticed.

- Proof: The nightly job runs those checks on main and reports through pm; the counterpart test fails on a planted release step with no local twin.

## Evidence

0.9.4's first rehearsal found the fresh-root wheel bug and two smoke gaps that no earlier check ran; 2291 covered ci.yml but not release.yml. From Ian's process assessment of 2026-10-20.
