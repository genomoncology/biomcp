# 2038 — Clear the pre-tag review findings before the 0.9.2 tag

Status: OPEN.

Milestone: 0.9.2


## Outcome

Every finding below is fixed, or carries a recorded reason in this ticket, before 0.9.2 is tagged. A fresh rehearsal in the public scratch repository passes on the tag candidate.

## Evidence

Filed 2026-10-09 from an independent read-only pre-tag review of main at `e71ac046b`, covering the range from `cf8113f3a` and the release path since v0.9.1. Four fresh reviewers worked in scratch clones. They built release binaries for main, `cf8113f3a` and v0.9.1, ran about 210 drug names, 45 disease abbreviations and 80 variant requests against live sources, ran mutations, the offline suite and actionlint, and compared every landing's main push with its CI runs. Follows ticket 2035. Tickets 2036 and 2037 carry findings 2 and 3.

- Starts from: tickets 2033 and 2035 and main at `e71ac046b`.
- Keeps: the fixes this review confirmed.
- Changes: the findings below.
- Proof: a fresh pre-tag review finds each item fixed or answered, and a rehearsal run passes on the candidate.
- Defers: nothing.

## Verdict

Do not tag 0.9.2. The work since the fourth review fixed the named problems: Tagrisso and Rybrevant resolve, `BRCA1 I1568N` returns p.Ile1568Asn on NM_007294, K-ras articles print the gene form, and main CI no longer stalls. Main's tip is red, the version is not bumped, the release workflow has not run since it changed, one variant gap and one drug regression remain, and ticket 2035 was closed with eleven findings neither fixed nor answered.

## Blockers

1. **Main is red.** Run 37892168813 on `e71ac046b` fails `canonical-gates` and `repository-contracts`. `sdlc/planning/2026-10-09-final-queue-report.md:56` names another team, which the coupling check forbids. The file is also a status report in Markdown, which the house rule forbids, and it repeats false claims (finding 12). Remove it. `cdeb4c0b5` was green on all eight jobs.
2. **Isoform-only variant matches.** `TP53 S183Y` and `BRCA1 S1587F` resolve to a different variant with no note. Ticket 2036.
3. **Brand drug regressions.** `get drug Tarceva`, Lartruvo, Portrazza and Lumoxiti refuse where v0.9.1 resolved them, and `get drug Zejula` returns the Akeega card and label. Ticket 2037.
4. **The version is not bumped.** `Cargo.toml`, `pyproject.toml`, `CITATION.cff` and `server.json` say 0.9.1, and the changelog heading reads `## Unreleased`. At a scratch v0.9.2 tag, `check-release-versions.py --tag v0.9.2` fails with "tag=0.9.2, Cargo.toml=0.9.1", and the first release job runs the same check. The checklist step to bump main after 0.9.1 was not done.
5. **The release workflow changed and has not run.** The last rehearsal is run 36896962624 on 2026-10-01. Since v0.9.1, every build and wheel job goes through `tools/with-build-identity`, the macOS x86_64 build cross-compiles on `macos-15`, the Windows jobs moved to `windows-2022`, the publishing jobs moved to `ubuntu-24.04`, and the Linux build passes an env file into the build container. Ticket 2024 deferred the rehearsal, against checklist section 1. Run a full rehearsal in the public scratch repository on the candidate, and check that every leg's `--version` prints exactly `0.9.2` with a real commit. Wheel-smoke and the container smoke print the version but never compare it with the tag; add the comparison or read the lines by hand.
6. **New source calls are not listed.** Free-text `search variant` calls MyGene (`src/cli/variant/query.rs:220`). `get variant "GENE change"` can call MyGene and UniProt (`src/entities/variant/get.rs:366-381`). `get article` and `article entities` call MyGene (`src/entities/article/detail.rs:288-309`). `docs/reference/sources.json` and `source-licensing.md` list none of these uses, and the changelog does not mention them.
7. **Ticket 2035 was closed with findings neither fixed nor answered.** Its closing note says every finding is fixed or carries a recorded reason. Findings 5, 6, 8 to 17 and 22 to 25 have no change and no reason at `e71ac046b`. They are carried here:
   - 2035 #5: `fn is_day_shaped` is still defined at `src/utils/date.rs:97` and `src/entities/variant/clinvar.rs:91`. The comments at `src/utils/date.rs:94-96` and `src/transform/variant.rs:838-842` still say 1291 "still carries its own inline check on its branch". The routed hint for `search variant "BRAF melanoma" --significance benign --tumor-site skin` still drops both flags, and `--max-frequency`, `--min-cadd` and `--review-status` too. The refused-path hint drops `--significance`.
   - 2035 #6: `--json who sync` with WHO broken still puts a local data path in `error.recovery`, and stdio MCP returns it for `get drug zidovudine regulatory`. Ian's decision is not recorded. Apply the terminal-only fix or record Ian's decision.
   - 2035 #8: the deadlock issue still reads `Status: open`.
   - 2035 #9: `tools/rust-source-size-inventory.json` still holds the hand-joined entries, for example lines 99, 132 and 499.
   - 2035 #10: no process-level per-test timeout exists. With the blocking wait planted, nextest printed SLOW at 60, 120 and 180 seconds and never killed the test. "Install canonical gate tools" has no `timeout-minutes`.
   - 2035 #11: without a local names file the checker exits 0 silently. With one, `test_repository_declares_only_the_inert_example_names` fails.
   - 2035 #12 to #16: the 2029, 2030, 1291 and 2021 records are unchanged, and the stray `D`, the duplicate paragraph and the five lint findings in 2021 remain.
   - 2035 #17: internal text grew. In sdlc/, docs/ and spec/, "lane" went from 385 to 400 mentions and "build host" from 9 to 14. `spec/README-timings.md` still names a gate machine, and `sdlc/planning/lanes.md` still exists.
   - 2035 #22 to #25: nothing recorded on 2022, 1291, 2021 or 2029. `get drug zidovudine regulatory` still errors outright when WHO is down, and `day >= 1` in `clinvar.rs` is still untested.
8. **2022 landed without its re-review.** The last verdict in the ticket is REJECT, followed by "Re-review of this round pending". `sdlc/records/2022-variant-headline-build.md` claims "the full review chain ACCEPT" and "every explicit filter kept", and cites a follow-up "noted by ticket 2033" that does not exist.
9. **Three landing merges committed conflict markers to main.** 2016 `27478cc1e` in `tools/zero-coupling-historical.json`, 2022 `74c0fe87f` in `tools/rust-source-size-inventory.json`, and 2031 `548f0b854` in `tests/test_source_package_boundary.py`. Follow-up commits fixed them, so no merged tree was tested before main moved. `04b7f6c79` restored constants the 2016 merge dropped. Main was also red at `928cd396a`, `f3804d21b` and `be10f4b90`. 2032's landcheck run started 29 seconds before main moved and its main run then failed.

## Release and changelog

10. **The changelog does not name GitHub issues.** Ticket 1304 fixes #288 and ticket 1303 fixes #289. The 0.9.1 section uses "GitHub #NNN".
11. **Changelog bullets that overstate.**
    - 2016's bullet says it fixed "a false other-transcript note" that 0.9.1 never had. It leaves out the new notes, the headline transcript change and the new refusal.
    - 2022's bullet belongs with 1301's feature bullet, and "keeps every explicit filter" is false (finding 7).
    - 2029 and 2034 do not reach `variant articles`.
    - 2017 leaves out that any one- or two-letter abbreviation now refuses.
    - 2032 reads as trial search in general; only NCI refuses.
    - 2031 says a hit counts only when the query matches its name, but a leading-name match still resolves.
    - 1300 says "whole label sections" while safety and interaction sections keep caps; `docs/user-guide/drug.md` repeats this.
    - 2023's "T ABLE" fix is in full-text rendering, not search.
    - 1302's "under three seconds" is one measurement.
    - 2024's "a host upgrade cannot change" is too strong.
    - 2030's bullet says "a hung test fails at its own named timeout", which finding 7 (2035 #10) shows is false for a blocking hang.
    - The 2020 bullet says the tree "no longer ships a private name list", which points readers to a public commit that holds the names. Reword it as "a stray packaging file".
12. **The final queue report overstates its CI evidence.** It claims three landings moved main before a green merged-tree run. Its own rows have eight: 2019, 2018, 2023, the 2024 fix, 2017, 2029, 2032 and 1305. It cites `2d3815718` for 2029, which is not on main and has a different tree. The tickets from 1290 to 1299 also reached main with no green merged-tree run first. "853 passed" does not match CI at `cdeb4c0b5`, which reports 850 passed and 3 skipped.
13. **Process text and jargon in the changelog.** The Internal bullets for 2033, 2035, 2010 and 2019 are process text. Rewrite cursorMark, 240-byte snippet, cache lock, headline transcript, renumbered lookalike, zero-row routed search, CLI line cap and MANE-numbered in plain words.
14. **Docs made stale.** `docs/blog/variant-structure-in-commands.md:33` shows the old headline format. `docs/sources/clinvar.md:10` describes the old evaluation-date rule. The variant guide does not mention gene-first routing. The article docs do not mention the 60-second search deadline, the offset limit or the OpenCitations fallback.

## Records

15. **Stray `D` lines** sit at 2016:207, 2021:80, 2031:25, 2032:24 and 2034:26.
16. **Reviews cited only at temporary paths.** 2016 and 2034 cite `/tmp/review-2016-fix4.md` and `/tmp/review-2034-code.md`, which are not in the repository, and their ticket lines name no head commit. Both reviewers state they had no shell and ran no gates. Record each review's head and evidence in the ticket.
17. **Records still name follow-up commits as landing merges.** 2016 names `04b7f6c79`, 2022 names `d7a1323be` and 2031 names `50679cc02`; all three are single-parent commits.
18. **Deferrals point at follow-ups that were never filed.** 2032 defers the default-source refusal and 2031 defers the product-name pooling to owners no ticket names. 2032's Outcome still says `search trial -c` refuses without limit. Its MF choices omit myelofibrosis, its MM pointer table (`src/entities/disease/resolution.rs:16-24`) cites no source, and NCI conditions over 512 bytes fail outright. No user doc says the default source does not refuse.
19. **The 2033 and 2035 closing notes are hard-wrapped**, `pm lint` flags both as complete with no Landed line, and the 1305 and 2034 ticket bodies are hard-wrapped.
20. **2033 #8 is still open.** Oversize 1300 labels still settle as `empty_with_reason` (`src/entities/drug/get.rs:833-834`), the 1300 ticket still has `## Outcome` twice, and its stale Deferred line remains.

## Smaller items

21. **Gene routing times out on a machine with a full cache.** With an 844 MB cache, each client build spends about 3.4 seconds walking the cache, with about 290,000 file opens. That uses up the 2.5-second routing deadline, so `search variant "SCN5A brugada"` reads as a condition. `search gene SCN5A` takes 3.4 seconds where 0.8.25 takes 0.3. This predates the range.
22. **`get disease ET` resolves to ethmoid sinus cancer** on all three builds. `HD` refuses with only Huntington's disease and no Hodgkin pointer. On the default trial source, `-c MM` mixes myeloma with periodontitis and hip trials with no note.
23. **The GTR spec-contract failure is untriaged.** `sdlc/issues/2026-10-07-diagnostic-synonym-provenance-fails-spec-contracts.md` is still open, and the contract smoke workflow last ran in June.

## What holds

- Main CI runs take 21 to 27 minutes with no stalls since 2030. `cdeb4c0b5` is green on all eight jobs, and the docs site serves main.
- Drugs: Tagrisso, Rybrevant (5 of 5), terfenadine, mannitol and cisapride resolve correctly. 5-FU and Ara-C refuse honestly. Keytruda, Gleevec, Enhertu, Lazcluze, Opdivo, Lynparza, Ibrance, Revlimid, Xeloda, Kadcyla, Imbruvica, Tecentriq, Kisqali, Verzenio, Tukysa, carboplatin, cisplatin, paclitaxel and docetaxel return the right drug and label. About 100 more oncology brands match v0.9.1 apart from finding 3.
- Disease: MM refuses with the multiple myeloma pointer. CML, ALL, AML, NSCLC, CRC, HCC, GBM, myeloma and melanoma resolve. No abbreviation that resolved in v0.9.1 now refuses, and DLBCL, MCL, NHL and MPN are now right. `search trial -c MF --source nci` refuses with choices.
- Variants: every request with a ClinVar record returns the requested change on the MANE transcript and version. `BRCA1 I1568N`, H1862D, A1611D and S1457T resolve on NM_007294. TP53 R209Q, G112D and R174H refuse honestly. A sample of 30 ClinVar-free changes gave no false note.
- Articles: 37887282, 30738221, 37948018 and 32955176 print gene-plus-change links.
- The changelog gate covers all 37 tickets at a scratch v0.9.2 tag. actionlint is clean, every runner label is pinned, and the Ubuntu 26 deadline of 2026-10-19 is handled.
- The offline suite passes 853 tests at `cdeb4c0b5`.

## Landing order

First remove the queue report so main is green. Then 2036 and 2037, and the code items in finding 7, each rebased on main with a review covering its head recorded in the ticket and a green merged-tree run on the exact landing tree before main moves. Then the records and docs items, the changelog fixes, and the version bump in one candidate commit. Run the rehearsal on that candidate. Then ask for the pre-tag review.

## Decisions recorded 2026-10-10 (Ian)

- Finding 7, 2035 #6 — the WHO local path stays terminal-only. JSON
  error.recovery and MCP callers never carry local paths; the terminal
  keeps its hint. That is the project's existing rule.
- Finding 5 — Ian authorizes one full release rehearsal in the public
  scratch repository on the tag candidate, after the fixes land.

## Finding-7 code lane (2026-10-10)

- Built on `tickets/2038-code-pre-tag`, head `e4f10fad3` (nine commits
  over `203daec02`): one shared day-shape rule with its tests; every
  explicit filter kept in the routed and refused hints (pinned by
  `apply_gene_first_routing_keeps_every_explicit_filter_in_both_hints`
  and the dispatch spelling tests); the WHO local path terminal-only
  with the drug card degrading under a path-free note; the per-test
  kill budget (`.config/nextest.toml` slow-timeout 120s,
  terminate-after 2, carried into the archive lane by the Makefile, plus
  a 20-minute bound on the CI tool-install step); the loud
  `--require-local-names` mode with CI provisioning from the
  `PM_FORBIDDEN_NAMES` secret; the day-zero gate tested; and the MyGene
  and UniProt surfaces listed in the sources registry and licensing
  page.
- Kill proof: a scratch hanging test with period 2s and
  terminate-after 2 on nextest 0.9.132 (older than CI's 0.9.146) was
  marked TIMEOUT at 4.003s, the run cancelled and failed — the
  process-level kill the planted-hang request asked for; run on the
  build host with the config file honored through the archive lane.
- Code review: ACCEPT 2026-10-10, head `e4f10fad3`, fresh read-only
  reviewer, verdict and findings recorded in
  `/tmp/review-2038-code.md` and mirrored here: items 1-8 confirmed
  with file:line evidence; two report-only P2s — this kill-proof record
  (folded here at landing) and a full diff-file-list check against the
  landing-time main (the landing step owns it).

## Code-lane wiring recorded 2026-10-10

- Finding 7, 2035 #11 — CI reads the forbidden-name declaration from the
  `PM_FORBIDDEN_NAMES` secret when present and then requires it (the
  checker fails loudly when it is missing). Until the secret exists, CI
  prints the checker's loud notice that the guard runs on inert example
  placeholders only. Ian's action: add the `PM_FORBIDDEN_NAMES` secret
  holding the local declaration's JSON.

## Accepted gap recorded 2026-10-10 (finding 21, 2035 #22)

- Gene routing can time out on a machine with a full cache. With an
  844 MB cache, each client build spent about 3.4 seconds walking the
  cache (about 290,000 file opens), which uses up the 2.5-second
  routing deadline, so `search variant "SCN5A brugada"` read the
  phrase as a condition. `search gene SCN5A` took 3.4 seconds on the
  same machine where 0.8.25 takes 0.3. Accepted as a pre-existing gap
  that predates the reviewed range; no 0.9.2 change, and the cache
  walk belongs to a future ticket if the deadline keeps missing.

## Candidate-split reviews (2026-10-10)

- Code review (contributing-record head e4148351d): ACCEPT 2026-10-10,
  recorded through pm and normalized here; fresh reviewer, both records
  branches verified.
- Code review (docs head 2c7b709aa): ACCEPT 2026-10-10, recorded
  through pm and normalized here; all six doc fixes verified against
  code with one report-only stale-header note.
- Code review (size-inventory head c907cdd60): ACCEPT 2026-10-10,
  recorded through pm and normalized here; the seven ticket strings
  normalized with no count changes.

- Code review (drug-label-variant-hint head 9ca6cd18e): ACCEPT 2026-10-10,
  recorded through pm and normalized here; the oversize settle and the
  condition-form refused hint verified with their biting tests.

## Version-bump review (2026-10-10)

- Code review (version-changelog head b9d4bf8d9): ACCEPT 2026-10-10,
  recorded through pm and normalized here. Every version file reads
  0.9.2, the changelog dated, the bullet fixes truthful, the scratch-tag
  gates pass; the Docs bullet depends on the docs branch already landed
  in Group A. The release-smoke review's P0 (a version pattern that
  could never match) and P1 (shallow checkouts missing the tag ref)
  were fixed on the branch and re-verified: the exact-line comparison,
  the fetch-depth 0 build checkouts, and the re-pinned step hashes with
  the provenance suite green and CI green at the fixed head.
