# 2168 — repair-instrumented-gencc-bootstrap-fixture

Status: OPEN.

Milestone: 0.9.3

## Outcome

The GenCC bootstrap durability test runs reliably under the required instrumented coverage configuration, preserving fail-closed behavior and every injected-fsync assertion.

Priority: 1.

## Changes

- src/sources/gencc/tests.rs
- CHANGELOG.md

## Implementation brief

Real coverage job 1792244253514-84513 for 2120 fails bootstrap_parent_fsync_failures_fail_closed_before_any_store_is_returned at tests.rs:901: Store::open().unwrap() receives Invalid. It ran 1840 unit tests before failure and cancelled 2342; integration coverage passed 154 tests. Identify the fixture or environment cause with a focused reproduction; do not weaken durability assertions or change production behavior without reporting the necessary scope. Reuse the existing instrumented build through pm job where possible, avoiding another full rebuild or full-suite diagnostic loop. The coverage lane owns the combined run and will rerun once after a reviewed repair.
