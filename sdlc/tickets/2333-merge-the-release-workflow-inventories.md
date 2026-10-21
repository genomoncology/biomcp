# 2333 — Merge the release workflow inventories

Status: OPEN.

Milestone: 0.9.4

Reviews: revision 4e7f582bcd1aa17f7f737d2aac982fc50ae7fe8b, accept

Reviews: revision 54f9ce93f50c51cfb32f79d5708c3421be3bd7fb, accept

Reviews: revision 5407796a69d0335e5755e0c4d46dcc189eabd7b3, accept

## Outcome

Changes: `tests/release_workflow_inventory.py`, `tests/test_release_check_counterparts.py`, `tests/test_release_workflow_provenance.py`, `CHANGELOG.md`.

One inventory pins the release workflow's jobs and steps. Both tests/test_release_check_counterparts.py and tests/test_release_workflow_provenance.py read it, so adding, removing or reordering a workflow step changes one place and a missed update fails in one test with a message naming that inventory. Keep every existing protection: step order, conditions, shells, body hashes and counterpart coverage.

- Proof: a mutation test that adds a workflow step without updating the inventory fails once, naming the inventory; the existing mutation controls in both tests still pass; the full routine suite passes.

## Evidence

- In 2323, a reviewer updated the counterpart inventory and missed PINNED_JOB_STEP_LISTS in the provenance test. 42 Python tests failed, found only by a later Gator early-signal gate.
- Ian asked on 2026-10-21 to clear every known issue in 0.9.4. This replaces the 0.9.5 idea to merge the inventories.
