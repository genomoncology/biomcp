# 2167 — stabilize-hpo-retry-fixture-clock

Status: OPEN.

Milestone: 0.9.3

## Outcome

Make the existing HPO retry contract deterministic with real loopback sockets. Preserve the production retry count, retry classification, and request assertions; do not suppress the failing test or change provider behavior merely to pass it.

Priority: 1.

## Changes

- src/sources/hpo/tests/parsing.rs
- CHANGELOG.md

## Implementation brief

The unchanged retry fixture fails both in the full routine suite (one request observed versus four expected) and alone (three versus four), so concurrent Python is not its cause. Standalone reproduction job 1792244038362-64897 exits 100 in 4.557 seconds. The fixture combines paused Tokio time and real loopback TCP. Reproduce on current main, identify the clock/socket mismatch, and make the smallest fixture-only repair with an outside-in request-count check. All execution runs through pm job on the build host. Escalate any required scope expansion before editing production code.
