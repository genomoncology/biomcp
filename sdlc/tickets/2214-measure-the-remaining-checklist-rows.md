---
priority: 3
---

# 2214 — Measure the remaining checklist rows

Status: OPEN.

Milestone: 0.9.4

## Outcome

The release measurement covers every row the 0.9.3 checklist named, so the next release compares like with like.

- Evidence: the 0.9.3 baseline record lacks these checklist rows: build folder size of a working checkout, crates compiled for the Linux and Windows release targets, Python test function count and the share that tests BioMCP's own tooling, the edit-then-run-one-test-group loop, release workflow run time, archive download size, and the number of full rehearsals a release needed.
- Changes: the existing measurement tool and its contract tests; the baseline record.
- Proof: each row reported for v0.9.3 and the 0.9.4 candidate on one build machine, with the method stated; rows that cannot be measured say why.

- Changes: `tools/measure-0-9-3-baseline` `tests/test_measurement_coverage_contract.py` `tests/test_measurement_summary_contract.py` and the baseline record. Inventory the exact record path and narrow tests before editing. Reuse retained release workflow and archive evidence; mark unavailable historical rows honestly. Measure frozen revisions; do not call a moving development tip the final release candidate.

## Progress

- 2026-10-19 started
