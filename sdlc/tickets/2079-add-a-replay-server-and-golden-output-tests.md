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

Reviews: revision 51c30b1, accept

Reviews: revision 43ba915, accept

Reviews: revision 600f232, accept

Reviews: revision a1c81a7, accept

Reviews: revision dc199db, accept

Reviews: revision 8d4e66e, accept

Reviews: revision fd90050, accept

Reviews: revision c0387f8, accept

Reviews: revision 01dcd9c, accept

## Outcome

The recorded replay and committed JSON/Markdown goldens cover the approved release batches: search all, gene, trial and pathway, and get gene, variant, trial and pathway. Format changes use regenerate, inspect and commit. Remaining selector families, traversal and local-data outputs belong to follow-up 2236 outside this release.

Depends on: 2156

- Starts from: the existing replay server, recording/request-log support and four JSON/Markdown golden pairs for get gene, get variant and the two search-all modes.
- Keeps: existing goldens, frozen source captures and reader-facing specification claims. Record new replies under the existing replay-data paths.
- Changes: `tests/golden/test_golden_output.py` `tests/golden/cases/search-pathway-mapk-*.json` `tests/golden/cases/search-pathway-mapk-*.md` `tests/golden/cases/get-pathway-hsa05200.json` `tests/golden/cases/get-pathway-hsa05200.md` `tests/golden/cases/get-pathway-r-hsa-5673001-genes-events.json` `tests/golden/cases/get-pathway-r-hsa-5673001-genes-events.md` `testdata/replay/manifest.json` `testdata/replay/responses/kegg/*` `testdata/replay/responses/reactome/*` `CHANGELOG.md` `sdlc/tickets/2236-finish-the-remaining-replay-command-coverage.md` `sdlc/planning/milestones.md` `sdlc/tickets/2237-repair-garbled-gtr-trademark-characters.md` `sdlc/tickets/2238-reject-limit-and-offset-in-variant-erepo-caid-mode.md` `sdlc/tickets/2239-cap-the-drug-trial-alias-cache.md` `sdlc/tickets/2240-use-the-ca-bundle-for-alphagenome.md` `sdlc/tickets/2241-carry-stale-cache-notes-in-get-json.md` `sdlc/tickets/2242-settle-three-suspected-bugs-with-live-runs.md` `sdlc/tickets/2243-fix-the-remaining-test-reliability-leftovers.md` `sdlc/tickets/2244-find-the-single-cpu-gencc-lease-hang.md` `sdlc/tickets/1260-unify-variant-transcript-numbering-in-lookup-and-display.md` and this ticket for the next bounded partial batch. Preserve source captures, receipts and remaining consumers.
- Proof: each remaining command-family batch has executed JSON and Markdown cases that fail when its output changes. Run the affected cases offline through the build job and the routine merged-candidate gate; report coverage without a new percentage gate. Measure the existing sixty-second golden-suite target rather than infer it; it is a suite target, not an elapsed-time failure rule for an individual test.
- Defers: broad shared fixture identifiers/scenario restructuring to the next release unless a named current migration needs a narrow change.

PM decision: 2026-10-17. Extend the existing infrastructure rather than rebuilding its first batch. Prioritize the replacement claims a pending 2081 deletion or named 2111 migration needs. Those claims must execute in an active outside-in lane before deletion; unrelated deletion batches do not wait for the whole ticket. Record batches through pm progress and close only after the remaining contracted command coverage is addressed.

PM decision: 2026-10-17. Moved to 0.9.4. Ian ruled that 0.9.3 ships once every bug is fixed; this ticket improves tests, tooling or dependencies and fixes no user-facing bug. Rebase onto main before resuming; earlier reviews cover the old base only.

## Progress

- 2026-10-18 started
- 2026-10-19 landed a646ba83da934317591a5a5e57f4cb7db24c72fe; next: Record completed review batch; retain 2079 for its remaining command coverage.
- 2026-10-19 next: Continue the remaining command JSON and Markdown golden coverage; the CTGov trial-reference batch is landed and qualified through the shared gate.
- 2026-10-19 landed 1d61826ab61846f0a570d7e22607b324906c06df; next: Keep the parent open; choose the next missing golden coverage needed by a named remaining replay consumer.
- 2026-10-19 landed fe24bfc32968b2609cebd38a1b0dd19a52b2e9bb; next: Retain 2079 for remaining golden coverage; complete the accepted article-speed and GWAS evidence outcomes after this shared source batch.
- 2026-10-19 next: Pathway batch accepted on SQLite main; wait for the main check and its own merged gate. Retained raw job evidence is copied to the private experiments/2079-replay-evidence folder, exempt from lane cleanup under PM ruling. Parent coverage inventory is running.
- 2026-10-19 next: Accepted pathway batch awaits main qualification and its merged gate. PM corrected evidence retention to numbered experiments: the verified archive is now in experiments/774-biomcp-2079-replay-evidence/1792418666605-1603; the temporary ticket-number folder was removed. Remaining selector inventory and helper-retirement boundary sent to PM for release scope.

## Next bounded batch

Restated outcome: add the JSON and Markdown golden pair for `search trial -c melanoma -s recruiting --limit 3`, so the existing Condition-First Search consumer can later move to replay without losing its reader assertions. The previous trial-reference batch is landed. Static inventory on 6aa806ede76f61d106a1076c4b68729feabeed58, including current main 9e9bc75b5, found no standalone trial-search golden; this is a missing contract case rather than an invented product defect.

Copy the complete existing source capture `testdata/sources/ctgov/search_melanoma_recruiting_limit3_20260811.json` into the declared replay response without modification. Match the existing command query and CTGOV_SEARCH_FIELDS exactly. Keep the three reader assertions: Trial Search Results heading, condition=melanoma and status=recruiting query, and NCT ID/Title/Status/Phase/Conditions columns. Preserve the three returned recruiting trials, total 480 and continuation token in frozen source data; inspect public output without assuming every provider field is exposed.

Generate and inspect the two goldens with the qualified current-main CLI through the build job. Use one disposable combined mutation of the existing title mapping and Conditions header to show the JSON and Markdown goldens both fail. Build that mutated source once, restore source and reuse the original qualified binary for passing proof; no additional mutation cycles or separate missing-file proof are needed. Run the affected pair and existing golden suite with no replay misses. Do not remove any old consumer, fixture or specification in this batch. Keep the parent open.

## Trial-search focused proof

Source 3d52eb80b41dcc1c04e08dbdf5b1c608ca213c59 changes exactly the six declared files. Job `2079/1792413852059-46070` used published brief 6d25b5a435d7b5d58806f16dd20a6c5726bd70b7 plus the exact private source patch: both formats rejected one combined title/header mutation, then passed with restored source and the original qualified binary. All golden cases passed without replay misses; the suite took 37.91 seconds. This was local replay, without kernel network isolation. Job `2079/1792413966591-79334` verified published file hashes, original full-capture equality, qualified binary identity and mechanical checks. The initial extraction-option setup failure ran no tests.

The capture and replay copy share SHA256 e2895daa89a2df1835b31a94307e786054e71ffa5dd8d9bde3af725723efbaff. Qualified CLI SHA256 c0b04d420095ea47f21b0751f7241d7031522022db1c933f21c13c6da142887e, mutant 42963fe9d54ff68707af21b7ff1794e969110827eb0f260bc81cf85185e444b6; the mutant and transfer files were removed. Existing consumer deletion is a separate reviewed 2111 slice. Parent coverage remains open.

## Next gene-search batch

Restated outcome: add one gene-search JSON/Markdown golden pair for `search gene BRAF --limit 3`, preparing the named remaining provider-contract consumer in the gene specification. Static inventory found no standalone gene-search pair. Preserve all reader assertions and ranking, deduplication, pagination, local filtering, query escaping, invalid-window rejection, MCP parity, adversarial identity and detail contracts. Do not delete the provider fixture or relist the entire gene page.

Reuse the already recorded exact size-50 MyGene BRAF route and its complete response `testdata/replay/responses/mygene/mygene-query-893da30e4c6a.bin`, SHA256 ab54067e8168ab36a2bd3c49acacb51fc6953be50ac608f42e615e7fa969a411. Its existing manifest entry records the original exact upstream query; it entered the initial replay batch at b20dd130e78e2b5eb269498c351b66f93f1803b6 and contains total 1 and one hit. It has replay-manifest provenance rather than a separate timestamped source receipt. Preserve its bytes and manifest unchanged. No new capture, remapping, receipt or live observation is needed. This narrows the preliminary six-file proposal to four product files.

Generate and inspect both outputs with a qualified CLI through the build job. In one disposable source copy, mutate the shared emitted gene name before JSON/Markdown rendering, build once and require both cases to fail. Restore source and reuse the original passing binary for the affected pair and existing golden suite. This is a missing contract case, without an invented product defect or redundant missing-file failure. Preserve all original consumers; keep the parent open after this slice.

## Gene-search focused proof

Source 655a5a90da707339f61977f4cafdf5d93fd738c9 changes only the four declared files. Job `2079/1792415449524-17338` used published brief 4cf5eadaf62c2b79c1ea785e740cbf6d51388a09 plus the exact private overlay. One emitted-name mutant build made both formats fail; restored source with the original qualified binary passed both cases and 17 golden output checks in 17.44 seconds. Local replay reported no missing requests; this was not kernel network isolation. Ruff, formatting, tracked-text, source-capture, quality and whitespace checks passed.

Published verification `2079/1792415580220-38607` matched all four tested file hashes and confirmed the manifest, original complete response and restored Rust source were unchanged. Qualified CLI SHA256 e47d903c9a83a1c8d18d9cd208e20ee4a27a03caa1f0e1a72b075a42542d949d; mutant b91ce89f61a9e23358957be56255af3f29d1bb0e70f9b6eacc55ec56207d4ee9. The mutant and transfer scratch were removed. No consumer deletion is part of this slice, and parent coverage remains open.

## Next cohesive development batch

Add the five inventoried pathway command pairs together using the seven unchanged receipted source captures and the existing unchanged WikiPathways bulk response. Preserve every existing specification assertion and source receipt. One combined mutation build must make all ten JSON/Markdown checks fail through changed output; restore source and reuse a qualified normal binary for the passing golden suite. No framework change or live capture is authorized. Keep the parent open.

## Pathway golden proof

Source `49a2cd1d3908bd0269e597878b5bbd1b66b738d7` adds five JSON/Markdown command pairs and seven exact-request replay copies in the declared twenty files. Source captures and receipt hashes are unchanged, as are the existing WikiPathways bulk response and reader specifications. The original receipts disclose minimized provider fields; these recordings do not represent complete provider schemas.

Job `2079/1792418666605-1603` generated the normal outputs without missing requests, then rejected all ten formats after one combined pathway-name mutation build. Restored source and the qualified normal CLI passed all 27 golden checks in 14.36 seconds. `make lint` passed, including all-target Clippy and the quality ratchet. All twenty published file hashes match the passing outputs. This was local replay rather than kernel network isolation.

The job retains `pathway-golden-evidence.tar.gz`, SHA256 `77f01441a882b4555d64ed1d7d20ece4d75da98cb4a93eb52db34fae66eeeb4c`, beneath `job-runs/biomcp/2079/1792418666605-1603/`. Private scratch and the mutant binary were removed; the shared qualified normal CLI remains available to the consumer lane. Fresh review accepted the unchanged source and verified the archive. The parent stays open for remaining contracted command coverage.

PM decision: 2026-10-19. Narrow this release outcome to the exact landed and accepted command batches above. Close this parent after the accepted pathway golden batch lands. Keep authored fault scenarios and excluded local datasets; the remaining inventory moves to 2236 outside 0.9.4. The verified pathway raw archive is retained in the private experiment `774-biomcp-2079-replay-evidence/1792418666605-1603`, SHA256 `77f01441a882b4555d64ed1d7d20ece4d75da98cb4a93eb52db34fae66eeeb4c`, rather than relying on cleanup-managed job storage.

PM decision: 2026-10-19. Carry the unchanged known-bug ticket batch from 239d7adc4 with the declared milestone; final release measurements wait for those bugs, the article recovery fix and the final supported pathway reader migration.

Keep: experiments/774-biomcp-2079-replay-evidence — Retain cited raw proof until the release evidence is explicitly released.
