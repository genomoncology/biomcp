---
flow: build
priority: 3
---

# 2105: Pin the disease-survival fixture reap test against load

Status: OPEN.

Milestone: 0.9.3

Reviews: revision d8d329a, accept

## Evidence

- CI run 38803434062 (ticket 2084's landing tip f8c85e01d, 2026-10-15): `tests/test_disease_survival_fixture_lifecycle.py::test_real_bounded_runner_timeout_reaps_disease_server_and_root` failed with FileNotFoundError on the fixture base-url file — the bounded runner's reap fired before the server wrote its base-url under runner load. The same branch's two earlier runs passed it; 1290 other tests passed. Same load-defect family as 2102/2103.

## Work

- Make the test's reap wait robust to a slow start (wait for base-url existence before arming the reap timeout), or mark it to skip under CI load the way the other deadline tests do once fixed.

## Acceptance

1. The test passes twice in a row on a loaded CI runner, or refuses to run where load makes its timing meaningless, without weakening what it proves.
