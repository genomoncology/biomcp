# 2169 — correct-measurement-suite-summary-counts

Status: OPEN.

Milestone: 0.9.3

## Outcome

The release measurement reports actual executed test counts and specification pages from the current runners, including fail-fast runs, while retaining incomplete results whenever a required suite fails or is missing.

Priority: 2.

## Changes

- tools/measure-0-9-3-baseline
- tests/test_measurement_summary_contract.py
- CHANGELOG.md

## Implementation brief

Real coverage job 1792244253514-84513 exposed two parser mismatches. The specification runner prints one aggregate header `=== spec pages: ... ===`, but the measurement at line 553 counts only singular `=== spec page: ` and reports zero pages despite exit zero. The nextest fail-fast summary `1841/4183 tests run` at line 534 is interpreted as 4183 executed instead of 1841. Capture the actual retained log text and demonstrate the wrong summary through the measurement entry point before code. Make the smallest summary-only correction with regression cases for complete, failed and absent runs; no suite skip, false complete percentage, runner change, or grouping redesign. All execution uses build jobs. This parser edit is separate from 2120's environment-restoration function and 2168's startup temporary-directory edit; preserve their later integration.
