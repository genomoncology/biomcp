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

Reviews: revision cd2923d, accept

Reviews: revision 130ad4b, accept

## Outcome

Every command's JSON and Markdown output is checked against committed golden files served by the existing replay server, and format changes use regenerate, inspect and commit.

Depends on: 2156

- Starts from: the existing replay server, recording/request-log support and four JSON/Markdown golden pairs for get gene, get variant and the two search-all modes.
- Keeps: existing goldens, frozen source captures and reader-facing specification claims. Record new replies under the existing replay-data paths.
- Changes: this batch claims `tests/golden/test_golden_output.py`, `tests/golden/cases/search-gene-braf.json`, `tests/golden/cases/search-gene-braf.md`, `testdata/replay/manifest.json`, `testdata/replay/responses/mygene/mygene-query-braf-complete.json`, `CHANGELOG.md` and this ticket. Preserve existing goldens, full captures and receipts, specification pages and runners.
- Proof: each remaining command-family batch has executed JSON and Markdown cases that fail when its output changes. Run the affected cases offline through the build job and the routine merged-candidate gate; report coverage without a new percentage gate. Measure the existing sixty-second golden-suite target rather than infer it; it is a suite target, not an elapsed-time failure rule for an individual test.
- Defers: broad shared fixture identifiers/scenario restructuring to the next release unless a named current migration needs a narrow change.

PM decision: 2026-10-17. Extend the existing infrastructure rather than rebuilding its first batch. Prioritize the replacement claims a pending 2081 deletion or named 2111 migration needs. Those claims must execute in an active outside-in lane before deletion; unrelated deletion batches do not wait for the whole ticket. Record batches through pm progress and close only after the remaining contracted command coverage is addressed.

PM decision: 2026-10-17. Moved to 0.9.4. Ian ruled that 0.9.3 ships once every bug is fixed; this ticket improves tests, tooling or dependencies and fixes no user-facing bug. Rebase onto main before resuming; earlier reviews cover the old base only.

## Progress

- 2026-10-18 started
- 2026-10-19 landed a646ba83da934317591a5a5e57f4cb7db24c72fe; next: Record completed review batch; retain 2079 for its remaining command coverage.
- 2026-10-19 next: Continue the remaining command JSON and Markdown golden coverage; the CTGov trial-reference batch is landed and qualified through the shared gate.
- 2026-10-19 landed 1d61826ab61846f0a570d7e22607b324906c06df; next: Keep the parent open; choose the next missing golden coverage needed by a named remaining replay consumer.

## Next bounded batch

Restated outcome: add the JSON and Markdown golden pair for `search trial -c melanoma -s recruiting --limit 3`, so the existing Condition-First Search consumer can later move to replay without losing its reader assertions. The previous trial-reference batch is landed. Static inventory on 6aa806ede76f61d106a1076c4b68729feabeed58, including current main 9e9bc75b5, found no standalone trial-search golden; this is a missing contract case rather than an invented product defect.

Copy the complete existing source capture `testdata/sources/ctgov/search_melanoma_recruiting_limit3_20260811.json` into the declared replay response without modification. Match the existing command query and CTGOV_SEARCH_FIELDS exactly. Keep the three reader assertions: Trial Search Results heading, condition=melanoma and status=recruiting query, and NCT ID/Title/Status/Phase/Conditions columns. Preserve the three returned recruiting trials, total 480 and continuation token in frozen source data; inspect public output without assuming every provider field is exposed.

Generate and inspect the two goldens with the qualified current-main CLI through the build job. Use one disposable combined mutation of the existing title mapping and Conditions header to show the JSON and Markdown goldens both fail. Build that mutated source once, restore source and reuse the original qualified binary for passing proof; no additional mutation cycles or separate missing-file proof are needed. Run the affected pair and existing golden suite with no replay misses. Do not remove any old consumer, fixture or specification in this batch. Keep the parent open.

## Trial-search focused proof

Source 3d52eb80b41dcc1c04e08dbdf5b1c608ca213c59 changes exactly the six declared files. Job `2079/1792413852059-46070` used published brief 6d25b5a435d7b5d58806f16dd20a6c5726bd70b7 plus the exact private source patch: both formats rejected one combined title/header mutation, then passed with restored source and the original qualified binary. All golden cases passed without replay misses; the suite took 37.91 seconds. This was local replay, without kernel network isolation. Job `2079/1792413966591-79334` verified published file hashes, original full-capture equality, qualified binary identity and mechanical checks. The initial extraction-option setup failure ran no tests.

The capture and replay copy share SHA256 e2895daa89a2df1835b31a94307e786054e71ffa5dd8d9bde3af725723efbaff. Qualified CLI SHA256 c0b04d420095ea47f21b0751f7241d7031522022db1c933f21c13c6da142887e, mutant 42963fe9d54ff68707af21b7ff1794e969110827eb0f260bc81cf85185e444b6; the mutant and transfer files were removed. Existing consumer deletion is a separate reviewed 2111 slice. Parent coverage remains open.

## Next gene-search batch

Restated outcome: add one gene-search JSON/Markdown golden pair for `search gene BRAF --limit 3`, preparing the named remaining provider-contract consumer in the gene specification. Static inventory found no standalone gene-search pair. Preserve all reader assertions and ranking, deduplication, pagination, local filtering, query escaping, invalid-window rejection, MCP parity, adversarial identity and detail contracts. Do not delete the provider fixture or relist the entire gene page.

Reuse the complete frozen capture `testdata/sources/mygene/search_braf_20260811.json`, SHA256 b180e713dffc626b3a6f7c501367e01ac150c854e017810ffe4870adf5f640d3, without changing it or its receipt. The receipt requested size 3; current BioMCP requests size 50 before local filtering/limit. This recorded response contains total 1 and one hit, so use the unchanged complete body for the exact current size-50 query and disclose that narrow replay mapping. Preserve the actual q, species, from and search fields. Do not invent rows or treat the fixture mapping as a new live observation.

Generate and inspect both outputs with a qualified CLI through the build job. In one disposable source copy, mutate the shared emitted gene name before JSON/Markdown rendering, build once and require both cases to fail. Restore source and reuse the original passing binary for the affected pair and existing golden suite. This is a missing contract case, without an invented product defect or redundant missing-file failure. Preserve all original consumers; keep the parent open after this slice.
