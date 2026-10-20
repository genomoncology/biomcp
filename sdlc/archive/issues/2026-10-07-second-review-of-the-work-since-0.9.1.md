# Second review of the work since 0.9.1

Disposition (2026-10-20): Closed under the finding-specific release dispositions recorded on 2026-10-19, including ticket 2243 at 569d9beb1 and ticket 2261 at d836b949. The original findings, historical failures, clinical rulings and decisions remain below.


Status: open

Filed 2026-10-07 from an independent read-only review. Four fresh reviewers covered main from `f10d42941` to `aaebf7f90` and the open branches for 1291, 1300, 1305 and 2016 to 2024. They worked in scratch clones, built the branches, ran mutations and the offline suite, and checked clinical answers against live public sources. Follows `sdlc/archive/issues/2026-10-07-review-of-the-work-since-0.9.1.md`.

## Verdict

Do not tag 0.9.2. No open branch is ready to land as it stands. The clinical logic in 2016, 2017, 2021, 2022 and 1291 checks out against live data. 1300 can still hand a clinician the wrong drug's label. The 2018 fix is incomplete, and records still change verdicts to pass checks.

## Blockers

1. **2018 is only partly fixed.** An article that mentions only KRAS G12A still prints `biomcp get variant rs121913529`, which opens G12D. Ticket 2018 reads complete, and its own Outcome rules this out. Ticket 2029 carries the fix.
2. **1299's verdict was rewritten again.** `50c19bb35` changed 1299's post-landing review from FIX to "ACCEPT … for the delta's code reconciliation" to satisfy the grammar check. The same line says the defect is not accepted. The three 1299 spec sections are still unfenced on main. Restore FIX until the sections run and pass.
3. **1300 can return another drug's label.** `spl_active_name_run` in `src/entities/drug/label.rs` keeps every repeated token. Real labels list each strength in one string, so inactive ingredients repeat. Against the branch's own capture `label_osimertinib_fulltext_20261007.json`, "mannitol", "talc" and "titanium dioxide" match the Tagrisso label. "edetate disodium" matches Rybrevant Faspro, and "ferric oxide" matches Lazcluze. The test row `tagrisso_row()` lists one strength, which hides this. The fix-round review said all fixture traces were checked.
4. **1300 calls a real label empty.** Cisapride and terfenadine full-text answers are 16.4 MB against the 8 MiB limit, and the result is now `empty`. openFDA holds the Propulsid label, which `search=spl_product_data_elements:"cisapride"` returns. `empty` must mean the source has nothing. Query the identity field or lower the limit. `MAX_PACKAGE_FILES` on main plus 1300 needs `1_404`, or `1_403` after 2020 removes the probe file.
5. **2024 cannot build on Windows.** `tools/with-build-identity` hands off with `os.execvpe`. On Windows Python exits 0 at once while cargo keeps running, so the step passes with no binary. No job has run the script on Windows. Use `subprocess.run`, pass the exit code through, and prove it on a Windows job. The branch has no green CI run: run 37658260567 failed on the review-grammar check carried from its base. The release matrix still uses `macos-latest` and `windows-latest`, while the ticket's Outcome says every job. Pin them or narrow the Outcome with a reason. 2024 has to land before 2026-10-19.
6. **2020 leaves the private names readable.** `sdlc/pm.json:191-195` now base64-encodes the names, and anyone can decode them. The encoding also disables `pm lint forbidden-name`, which matches literal text. The ticket said to move them to a local untracked config. `sdlc/planning/lanes.md` is still stale. The 1284 ticket and record still say "private scratch copy … TestPyPI". `milestones.md:11` does not say who waived QA. `4d85dff69` came after the branch's review with no re-review.
7. **The 2020 spec failure is a race, not the lane.** The failing block runs `search article`, which calls PubTator, Europe PMC, PubMed, Semantic Scholar and LitSense2. The deadline fixture overrides all five. PubTator's 503 is retried three times with full-jitter backoff (`src/sources/mod.rs:1222`), so it settles between 0 and 7 seconds. When it settles inside the 3-second deadline, about 38% of runs, `search.rs:535` returns PubTator's error and hides the deadline. Solo passes were luck. 2023's precedence fix closes it, so 2023 lands before 2020. The steering note that blamed the full-text fixture env was wrong.
8. **The changelog is not ready.** On 1305, the gate at a scratch v0.9.2 tag fails with "missing … 2018, 2019". Bullets for 1291 and 1300 describe work not on main. The 1297 bullet states the rule 2016 reverses. The 1295, 1290 and 1296 bullets need rereading after 2017, 2022 and 2018 land. The Internal bullet is hard-wrapped, reads as process text, and says "(1304, absorbing 2010)". The last verdict is FIX at `ba7abe31e`, and the head has no re-review.
9. **The spec-contract exemption has no triage.** `19223d51e` exempts a failing `spec/surface/mcp.md` case from the 0.9.2 bar because CI never ran that lane. No bisect rules out a regression in the live GTR source. Triage it before the tag.

## Branch findings

10. **Reviews missing or behind the head.** 2016 and 2022 have no recorded code review. 2022 reads "IMPLEMENTED (unreviewed)" after `aa8317a4f`. The 1291 review covers `7026e9a77`, not `bb4d52250`, which edits tests and raises the test-wait ceiling from 43 to 46. The 1305 and 2020 heads came after their reviews. No ticket from 2016 to 2024 records a ticket review.
11. **Branches turn the suite red.** 2016 and 2017 fail `test_landed_tickets_record_their_review_verdicts`, because their ACCEPT lines sit only on main. 2021 fails the review-line grammar at `sdlc/tickets/2021-who-data-gaps-reach-every-caller.md:38`, and merging main leaves two "Build status" sections.
12. **Merge conflicts.** 2016, 2017, 2020, 2022, 2023, 1291 and 1300 conflict with main. The conflicts sit in ticket files, `capture-receipts.json`, `tools/zero-coupling-historical.json`, `tests/test_source_package_boundary.py` and `rust-source-size-inventory.json`. 2022 re-indents all of `capture-receipts.json` from two spaces to one, about 6,700 changed lines for 12 real ones. Revert the re-indent. Each later landing must recount `MAX_PACKAGE_FILES`.
13. **2016 calls the wrong transcript canonical.** `src/transform/variant.rs:91` takes the first NM_ transcript in the SnpEff list, not MANE. For BRCA1 that is NM_007300, which numbers residues 21 higher than NM_007294 after the insert. A314T is right only because the two agree at 314. Prefer the MANE transcript, and test a BRCA1 change past the insert. A single hit whose reference residue does not match still resolves with no note, for example `TP53 R116Q` to p.Arg248Gln. Say in the answer which numbering matched.
14. **2017 refusals.** `get disease myeloma` refuses as an "ambiguous abbreviation" because a veterinary record (MONDO:1013329) holds the word. `get disease CAD` lists `MONDO:0018922 (no label in the search response)`. Leave non-human records out of the count, call a full word ambiguous only by that name, and fill missing labels. The trial search at `src/entities/trial/search/nci.rs:24` drops to a plain keyword search with only a log warning.
15. **2022's zero-row hint repeats the command that returned zero rows** (`src/cli/variant/dispatch.rs:454`). Two day-shape date checks now exist (`utils/date.rs:96` and 1291's `clinvar.rs:91`). Whichever lands second removes one.
16. **2023 smaller items.** The `exhaustion_rows` 45 spec change has never run. The inventory entry for `src/sources/mod.rs` drops 1268's removal condition. `OnceLock` became `RwLock` for a test seam, which leaves the "initialization race" error unreachable.
17. **Smaller items.** 2021's sync suggestion no longer names the data directory. 1291's ticket says `ncbi_efetch.rs` keeps its pin, but the inventory raises it from 1281 to 1322. Public spec pages cite "the recorded repro experiment" (`spec/entity/variant.md:628`, `setup-variant-identity-spec-fixture.sh:76`). `make spec-pr` and `make spec-contracts` still skip the Python sync.

## Landed work

18. **2018 moved main before its CI.** Main took the merge at 19:09Z, and the merged-tree run 37669278747 finished at 20:01Z. The 2019 record had just promised the opposite. The 2019 merged-tree run 37644694628 failed, and the merge reached main anyway; the ticket records that. The 2018 code review ran no tests.
19. **The changelog gate is sound.** At a scratch tag with an empty Unreleased section, it reports all 20 landed tickets and misses none. Three mutations survive: dropping the colon from `^Land ([0-9]+):` (`scripts/check-changelog-coverage.py:19`), widening `--diff-filter=A` to `AM` (line 110), and ignoring `Status:` past line 25 (line 82). None blocks.
20. **Records leave things out.** The backfilled 1290 record says "branch CI green, yellow gates rc=0". It leaves out that the 1290 merge broke main and the date gap 2022 carries.
21. **Internal detail keeps growing in public files.** Mentions of the gate machine's name in `sdlc/` and `docs/` went from 177 to 188. New text says "coordinator", "lane worker", "this sandbox", "this host", "load average", "revival after a timeout" and "QA-era". No private project names appear, and the zero-coupling test passes.
22. **Main is green.** The offline suite passes 833 tests after `make sync-python-dev`.

## Landing order

2024 first, because of the deadline. Then 2023 before 2020. The rest follow in any order, each rebased on main with a review covering its head and a green merged-tree run before main moves. 1305 lands last, with bullets checked against main at that point.

## Residual closure in 2243

Resolved finding17 Python-sync residual: 2243 makes spec-pr and spec-contracts depend on sync-python-dev and pins the ordering through the existing contract test. Other findings retain their own dispositions.

## Release dispositions (2026-10-19)

- Finding 1: FIXED by 2029 at 8eec444db and 2034 at 6bf1a550a, preserving allele-specific article links.
- Finding 2: FIXED by 2020 at d1c6d713f, which restores executable specification fences and corrects the review disposition. Ian’s later 2054 ruling retains finished writeups.
- Finding 3: FIXED by 2031 at 548f0b854 and 2043 at 97b237b6f, with subsequent drug regression repairs in 2052 at a6aefbb95.
- Finding 4: FIXED by 2031 at 548f0b854 and the later drug-label repairs. The final package boundary no longer carries the deleted size inventory.
- Finding 5: FIXED by 2024 at f6eca26b3 and the release workflow corrections in 2046 at c6b85fc05, with actual platform qualification recorded there.
- Finding 6: FIXED by 2020 at d1c6d713f and the residual cleanup in 2243 at 569d9beb1. The current tree carries no encoded-name workaround.
- Finding 7: FIXED by 2023 at 05eeda3f3, which gives deadline exhaustion precedence; the earlier fixture-environment diagnosis remains historical.
- Finding 8: FIXED by 2053 at a168abf8e and 2058 at 90221a07f with the final truthful 0.9.2 changelog.
- Finding 9: FIXED by 2242 at b36ad664f with deterministic synonym controls and honest missing-synonym output. The live study did not establish a GTR parser bug.
- Finding 10: FIXED by the review-record corrections in 2020 at d1c6d713f and the reviewed landings of the named branches. Ian’s 2054 ruling retains completed writeups; the original branch observations remain historical.
- Finding 11: MOOT. The named branches landed and their branch-only grammar and duplicate-status conflicts no longer exist. This does not change the original observed failures.
- Finding 12: MOOT. The named branches landed. The obsolete source-size inventory was deleted at 8c2d822af, so recounting it is no longer an admission requirement.
- Finding 13: FIXED by 2016 at 27478cc1e and the MANE/isoform follow-ups in 2036 at 2d5c61492 and 2047 at e16d049c5. The same-allele guard landed in 2261 at d836b949 under the recorded 2026-10-19 clinical ruling.
- Finding 14: FIXED by 2017 at d8afa297f and 2032 at d18d30e7f. The 2026-10-19 clinical ruling requires no further abbreviation change.
- Finding 15: FIXED by 2022 at 74c0fe87f and the hint/date follow-ups retained in 2038 at 07d660890 and 2052 at a6aefbb95.
- Finding 16: FIXED by 2023 at 05eeda3f3 and its subsequent reviewed deadline repairs. The deleted inventory makes its old removal-condition wording moot.
- Finding 17: FIXED by 2021 at 44b33eb8a and the 2020/2038 cleanup, with 2243 at 569d9beb1 completing alternate-target Python setup. The obsolete inventory is deleted.
- Finding 18: HISTORICAL. Later passing checks do not repair the recorded early main updates. Current landings require the repository’s merged-tree checks.
- Finding 19: NOT A BUG. The review found the changelog gate sound; its listed surviving mutations did not invalidate that result.
- Finding 20: HISTORICAL. Preserve the recorded failure and date gap; Ian’s 2054 direction keeps completed writeups rather than deleting or rewriting their history.
- Finding 21: DECIDED. Ian’s 2026-10-19 ruling permits names in sdlc records. Public docs and public text omit private project names. Preserve historical records and commit history; add no name check, linter or guard.
- Finding 22: NOT A BUG. This is a passing-suite observation at the reviewed snapshot, not qualification of a later candidate.
