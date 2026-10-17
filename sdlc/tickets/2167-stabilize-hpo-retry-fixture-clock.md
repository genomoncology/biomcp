# 2167 — stabilize-hpo-retry-fixture-clock

Status: OPEN.

Milestone: 0.9.3

## Outcome

Make the existing HPO retry contract deterministic with real loopback sockets. Preserve the production retry count, retry classification, and request assertions; do not suppress the failing test or change provider behavior merely to pass it.

Priority: 1.

## Changes

- src/sources/hpo/tests/parsing.rs
- src/cli/system/tests/dev2_contracts.rs
- CHANGELOG.md

## Implementation brief

The unchanged retry fixture fails both in the full routine suite (one request observed versus four expected) and alone (three versus four), so concurrent Python is not its cause. Standalone reproduction job 1792244038362-64897 exits 100 in 4.557 seconds. The fixture combines paused Tokio time and real loopback TCP. Reproduce on current main, identify the clock/socket mismatch, and make the smallest fixture-only repair with an outside-in request-count check. All execution runs through pm job on the build host. Escalate any required scope expansion before editing production code.

The unchanged cancellation contract `dropping_canonical_compact_command_cancels_provider_retry` also failed during the bounded serial timing run in job `1792244325098-99402`, at dev2_contracts.rs:274: the command settled with http_middleware before the expected retry. Include this instance only if it has the same paused-time/real-socket cause; retain its cancellation and retry assertions.
