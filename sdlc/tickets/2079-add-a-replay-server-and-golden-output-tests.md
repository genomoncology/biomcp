---
priority: 3
---

# 2079 — Add a replay server and golden output tests

Status: OPEN.

Milestone: 0.9.4

Reviews: revision 8552ed1, accept

Reviews: revision 8552ed1, accept

## Outcome

Every command's JSON and Markdown output is checked against committed golden files served by the existing replay server, and format changes use regenerate, inspect and commit.

Depends on: 2156

- Starts from: the existing replay server, recording/request-log support and four JSON/Markdown golden pairs for get gene, get variant and the two search-all modes.
- Keeps: existing goldens, frozen source captures and reader-facing specification claims. Record new replies under the existing replay-data paths.
- Changes: claims `tools/replay-server` `tests/golden/test_golden_output.py` `tests/golden/replay_env.py` `tests/golden/conftest.py` `tests/golden/cases/**` `tests/golden/spec/**` `testdata/replay/**` `CHANGELOG.md`. Name affected specification pages and `scripts/run-specs.sh` in each batch's actual claim before editing them.
- Proof: each remaining command-family batch has executed JSON and Markdown cases that fail when its output changes. Run the affected cases offline through the build job and the routine merged-candidate gate; report coverage without a new percentage gate. Measure the existing sixty-second golden-suite target rather than infer it; it is a suite target, not an elapsed-time failure rule for an individual test.
- Defers: broad shared fixture identifiers/scenario restructuring to the next release unless a named current migration needs a narrow change.

PM decision: 2026-10-17. Extend the existing infrastructure rather than rebuilding its first batch. Prioritize the replacement claims a pending 2081 deletion or named 2111 migration needs. Those claims must execute in an active outside-in lane before deletion; unrelated deletion batches do not wait for the whole ticket. Record batches through pm progress and close only after the remaining contracted command coverage is addressed.

PM decision: 2026-10-17. Moved to 0.9.4. Ian ruled that 0.9.3 ships once every bug is fixed; this ticket improves tests, tooling or dependencies and fixes no user-facing bug. Rebase onto main before resuming; earlier reviews cover the old base only.
