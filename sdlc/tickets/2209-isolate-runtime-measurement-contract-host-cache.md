---
priority: 1
---

# 2209 — Isolate the runtime measurement contract from the host cache

Status: OPEN.

Milestone: 0.9.3

## Outcome

The fake-CLI runtime measurement contract uses its own home and cache tree. It must not inspect the operator's real cache or depend on that cache's size or filesystem availability. Preserve the real measurement helper, all sample/order/output assertions and the existing timeout.

- Evidence: required fresh-clone offline Linux job `2208/1792331957777-4022455` on main `f7c08d7c09479a3cf6c7336f779f4e46e1ce861b`, canonical `make test`, passed the complete Rust lane: 4373 passed, 34 skipped, five slow warnings. Python reported one failed, 1285 passed, four skipped. `test_runtime_retains_alternating_isolated_samples` timed out at60s in `tests/test_measurement_summary_contract.py:187`; its captured output stopped at the measurement helper's real-host cache `find`/`du` preamble before the fake CLI observations. This is a fixture isolation failure, not a reason to raise the timeout or modify production measurement behavior.
- Changes: `tests/test_measurement_summary_contract.py`, `CHANGELOG.md`. Inspect only this fixture class, restate the outcome, identify the exact environment change and current-main focused reproduction before editing. No new benchmark, wrapper framework or production behavior change.
- Proof: retain the actual full-Linux failure and make the existing outside-in contract complete against an isolated empty home while preserving its assertions. Run the focused measurement contract module through a build job, obtain fresh review, and run the full Linux Python suite offline from a fresh clone after the fix. Reuse the complete unaffected Linux Rust pass while its source and execution settings are unchanged. Native post-merge main checks and one full nonpublishing rehearsal still precede release.

## Progress

- 2026-10-18 started
