# 2168 — repair-instrumented-gencc-bootstrap-fixture

Status: COMPLETE.

Milestone: 0.9.3

Reviews: revision e084176, accept

Landed: e084176 (fast-forward: Accepted source landed inside carrier 27e59b197 with its passing shared gate; record this carried ticket separately without rerunning unchanged code.)

## Outcome

The GenCC bootstrap durability test runs reliably under the required instrumented coverage configuration, preserving fail-closed behavior and every injected-fsync assertion.

Priority: 1.

## Changes

- src/entities/gene/gencc/tests.rs
- tools/measure-0-9-3-baseline
- CHANGELOG.md

## Implementation brief

Real coverage job 1792244253514-84513 for 2120 fails bootstrap_parent_fsync_failures_fail_closed_before_any_store_is_returned at tests.rs:901: Store::open().unwrap() receives Invalid. It ran 1840 unit tests before failure and cancelled 2342; integration coverage passed 154 tests. Identify the fixture or environment cause with a focused reproduction; do not weaken durability assertions or change production behavior without reporting the necessary scope. Reuse the existing instrumented build through pm job where possible, avoiding another full rebuild or full-suite diagnostic loop. The coverage lane owns the combined run and will rerun once after a reviewed repair.

Coordinator scope decision: job 1792244623336-42119 establishes the cause with the same unchanged instrumented binary. Inherited temporary directory spelling crosses a symlink rejected by the store's intentional trust boundary; canonical spelling passes. Canonicalize the measurement runner's temporary directory like the routine gates. Preserve store validation and all durability tests. This runner startup edit does not overlap 2120's completed coverage-environment restoration.

## Progress

- 2026-10-17 started
- 2026-10-17 next: Batch members: 2159, 2149, 2168, 2128, 2122 and 2171. Shared landing gate: job 1792250897357-93113, candidate ea4161cc814ad034634ffd249e67b475e77e38dc. Required preceding main check 1792249698624-96397 passed. Gate is running; each ticket keeps its individual review and landing commit.
- 2026-10-17 next: Batch members: 2159, 2149, 2168, 2128, 2122 and 2171. Shared configured landing gate PASSED: job 1792251400030-83132, candidate c264da2a525c2c6e5a424693d55dfab0f59fadf4, 240.3 seconds. This replaces the earlier failed batch attempts. Preceding full main check 1792249698624-96397 passed. Land each reviewed member through pm, then run one full check on final batch main.
