# 2167 — stabilize-hpo-retry-fixture-clock

Status: OPEN.

Milestone: 0.9.3

Reviews: revision 0070c21, accept

## Outcome

PM decision needed: two independently reviewed main repairs cannot pass the whole landing gate separately. Changelog repair 2166 passes 78 focused contracts but its full gate fails the unchanged HPO fixture (1792244741687-84723); clock repair 2167 passes all 13 focused cases but its gate still sees the known changelog mismatch. Recommend one combined, reviewed repair landing containing both changes, then an immediate full main check, with both ticket records closed through pm against that landing. Alternative is an explicit temporary targeted-gate exception for these two repairs only. All ordinary accepted work remains queued until main is green; neither failure will be hidden or weakened.

Make the existing HPO retry contract deterministic with real loopback sockets. Preserve the production retry count, retry classification, and request assertions; do not suppress the failing test or change provider behavior merely to pass it.

Priority: 1.

## Changes

- src/sources/hpo/tests/parsing.rs
- src/cli/system/tests/dev2_contracts.rs
- CHANGELOG.md

## Implementation brief

The unchanged retry fixture fails both in the full routine suite (one request observed versus four expected) and alone (three versus four), so concurrent Python is not its cause. Standalone reproduction job 1792244038362-64897 exits 100 in 4.557 seconds. The fixture combines paused Tokio time and real loopback TCP. Reproduce on current main, identify the clock/socket mismatch, and make the smallest fixture-only repair with an outside-in request-count check. All execution runs through pm job on the build host. Escalate any required scope expansion before editing production code.

The unchanged cancellation contract `dropping_canonical_compact_command_cancels_provider_retry` also failed during the bounded serial timing run in job `1792244325098-99402`, at dev2_contracts.rs:274: the command settled with http_middleware before the expected retry. Include this instance only if it has the same paused-time/real-socket cause; retain its cancellation and retry assertions.
