---
priority: 3
---

# 2079 — Add a replay server and golden output tests

Status: OPEN.

Milestone: 0.9.4

Reviews: revision 8552ed1, accept

Reviews: revision 8552ed1, accept

Reviews: revision e3e6d4a, accept

Reviews: revision 05b98eb, accept

## Outcome

Every command's JSON and Markdown output is checked against committed golden files served by the existing replay server, and format changes use regenerate, inspect and commit.

Depends on: 2156

- Starts from: the existing replay server, recording/request-log support and four JSON/Markdown golden pairs for get gene, get variant and the two search-all modes.
- Keeps: existing goldens, frozen source captures and reader-facing specification claims. Record new replies under the existing replay-data paths.
- Changes: current batch claims `tests/golden/test_golden_output.py`, `tests/golden/cases/get-trial-nct02576665-references.json`, `tests/golden/cases/get-trial-nct02576665-references.md`, `tests/golden/cases/get-trial-nct06131398-references.json`, `tests/golden/cases/get-trial-nct06131398-references.md`, `testdata/replay/manifest.json`, `testdata/replay/responses/ctgov/ctgov-studies-nct02576665-references.json`, `testdata/replay/responses/ctgov/ctgov-studies-nct06131398-references.json`, `CHANGELOG.md` and this ticket. Name actual files for each later batch before editing them; this batch changes no specification page or runner.
- Proof: each remaining command-family batch has executed JSON and Markdown cases that fail when its output changes. Run the affected cases offline through the build job and the routine merged-candidate gate; report coverage without a new percentage gate. Measure the existing sixty-second golden-suite target rather than infer it; it is a suite target, not an elapsed-time failure rule for an individual test.
- Defers: broad shared fixture identifiers/scenario restructuring to the next release unless a named current migration needs a narrow change.

PM decision: 2026-10-17. Extend the existing infrastructure rather than rebuilding its first batch. Prioritize the replacement claims a pending 2081 deletion or named 2111 migration needs. Those claims must execute in an active outside-in lane before deletion; unrelated deletion batches do not wait for the whole ticket. Record batches through pm progress and close only after the remaining contracted command coverage is addressed.

PM decision: 2026-10-17. Moved to 0.9.4. Ian ruled that 0.9.3 ships once every bug is fixed; this ticket improves tests, tooling or dependencies and fixes no user-facing bug. Rebase onto main before resuming; earlier reviews cover the old base only.

## Progress

- 2026-10-18 started
- 2026-10-19 landed a646ba83da934317591a5a5e57f4cb7db24c72fe; next: Record completed review batch; retain 2079 for its remaining command coverage.

## Next bounded batch

Restated batch outcome: add populated and empty trial-reference JSON/Markdown golden cases through the existing replay server. This prepares2111's named test_recorded_references_and_empty_result_keep_section_behavior migration. Reuse unchanged existing CTGov captures for NCT02576665 and NCT06131398. Current inventory has no corresponding committed trial-reference golden pair; this is missing outside-in coverage, not a functional defect requiring an invented failing baseline. Preserve the old consumer and reader-facing specifications.

Generate four goldens using the qualified current-main CLI and frozen local replies; verify source-order references, PMID/citation/source_type and the clinicaltrials.gov link plus explicit empty behavior. In one disposable build-job copy, clear acquired trial.references to demonstrate both populated JSON and Markdown cases fail; restore source and run the affected pairs and existing golden suite offline. Measure actual suite elapsed time, without a new elapsed-time failure rule. No new recorder, live capture, fixture-identifier refactor or product change. Keep the parent open after this batch until its remaining contracted coverage is addressed.
