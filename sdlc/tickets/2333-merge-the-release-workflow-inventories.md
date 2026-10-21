# 2333 — Merge the release workflow inventories

Status: OPEN.

Milestone: 0.9.4

## Outcome

Changes: `tests/release_workflow_inventory.py`, `tests/test_release_check_counterparts.py`, `tests/test_release_workflow_provenance.py`, `CHANGELOG.md`.

One inventory pins the release workflow's jobs and steps. Both tests/test_release_check_counterparts.py and tests/test_release_workflow_provenance.py read it, so adding, removing or reordering a workflow step changes one place and a missed update fails in one test with a message naming that inventory. Keep every existing protection: step order, conditions, shells, body hashes and counterpart coverage.

- Proof: a mutation test that adds a workflow step without updating the inventory fails once, naming the inventory; the existing mutation controls in both tests still pass; the full routine suite passes.

## Evidence

- In 2323, a reviewer updated the counterpart inventory and missed PINNED_JOB_STEP_LISTS in the provenance test. 42 Python tests failed, found only by a later Gator early-signal gate.
- Ian asked on 2026-10-21 to clear every known issue in 0.9.4. This replaces the 0.9.5 idea to merge the inventories.
