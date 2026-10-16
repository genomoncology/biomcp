---
flow: build
priority: 2
deps: [2117, 2082]
---

# 2120: Stop the coverage show-env exports from reaching the Python lane

Status: OPEN.

Milestone: 0.9.3

## Evidence

- 2069 and 2117's clean proof (2026-10-16) both show the instrumented coverage env failing the same 26 cargo-driving Python tests, which read the show-env exports. Until owned, 2082's re-measure prints incomplete for outside-in and combined sets.

## Work

- Scope the show-env exports to the suites that consume them, or gate the 26 tests' cargo-driving on the instrumented lane marker, so the Python lane runs green inside the coverage instrumentation.

## Acceptance

1. The coverage measure runs every suite green; outside-in and combined print numbers, not incomplete.
