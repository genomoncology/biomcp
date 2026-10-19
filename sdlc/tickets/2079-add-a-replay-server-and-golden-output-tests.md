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
- Changes: this batch claims `tests/golden/test_golden_output.py`, `tests/golden/cases/search-trial-melanoma-recruiting.json`, `tests/golden/cases/search-trial-melanoma-recruiting.md`, `testdata/replay/manifest.json`, `testdata/replay/responses/ctgov/ctgov-studies-melanoma-recruiting-limit3.json`, `CHANGELOG.md` and this ticket. Preserve existing goldens, full source captures, specification pages and runners.
- Proof: each remaining command-family batch has executed JSON and Markdown cases that fail when its output changes. Run the affected cases offline through the build job and the routine merged-candidate gate; report coverage without a new percentage gate. Measure the existing sixty-second golden-suite target rather than infer it; it is a suite target, not an elapsed-time failure rule for an individual test.
- Defers: broad shared fixture identifiers/scenario restructuring to the next release unless a named current migration needs a narrow change.

PM decision: 2026-10-17. Extend the existing infrastructure rather than rebuilding its first batch. Prioritize the replacement claims a pending 2081 deletion or named 2111 migration needs. Those claims must execute in an active outside-in lane before deletion; unrelated deletion batches do not wait for the whole ticket. Record batches through pm progress and close only after the remaining contracted command coverage is addressed.

PM decision: 2026-10-17. Moved to 0.9.4. Ian ruled that 0.9.3 ships once every bug is fixed; this ticket improves tests, tooling or dependencies and fixes no user-facing bug. Rebase onto main before resuming; earlier reviews cover the old base only.

## Progress

- 2026-10-18 started
- 2026-10-19 landed a646ba83da934317591a5a5e57f4cb7db24c72fe; next: Record completed review batch; retain 2079 for its remaining command coverage.
- 2026-10-19 next: Continue the remaining command JSON and Markdown golden coverage; the CTGov trial-reference batch is landed and qualified through the shared gate.

## Next bounded batch

Restated outcome: add the JSON and Markdown golden pair for `search trial -c melanoma -s recruiting --limit 3`, so the existing Condition-First Search consumer can later move to replay without losing its reader assertions. The previous trial-reference batch is landed. Static inventory on 6aa806ede76f61d106a1076c4b68729feabeed58, including current main 9e9bc75b5, found no standalone trial-search golden; this is a missing contract case rather than an invented product defect.

Copy the complete existing source capture `testdata/sources/ctgov/search_melanoma_recruiting_limit3_20260811.json` into the declared replay response without modification. Match the existing command query and CTGOV_SEARCH_FIELDS exactly. Keep the three reader assertions: Trial Search Results heading, condition=melanoma and status=recruiting query, and NCT ID/Title/Status/Phase/Conditions columns. Preserve the three returned recruiting trials, total 480 and continuation token in frozen source data; inspect public output without assuming every provider field is exposed.

Generate and inspect the two goldens with the qualified current-main CLI through the build job. Use one disposable combined mutation of the existing title mapping and Conditions header to show the JSON and Markdown goldens both fail. Build that mutated source once, restore source and reuse the original qualified binary for passing proof; no additional mutation cycles or separate missing-file proof are needed. Run the affected pair and existing golden suite with no replay misses. Do not remove any old consumer, fixture or specification in this batch. Keep the parent open.
