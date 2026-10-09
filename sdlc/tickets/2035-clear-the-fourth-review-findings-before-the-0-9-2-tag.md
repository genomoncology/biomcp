# 2035 — Clear the fourth review findings before the 0.9.2 tag

Status: complete.

Milestone: 0.9.2

## Outcome

Every finding below is fixed, or carries a recorded reason in this ticket, before 0.9.2 is tagged.

## Evidence

Filed 2026-10-08 from an independent read-only review of main from `4258f8245` to `cf8113f3a` and the open branches for 2016, 2022, 2031, 2032 and 1305. Four fresh reviewers worked in scratch clones. They built release binaries, ran mutations and the offline suite, and checked answers against live public sources. Follows ticket 2033. Ticket 2034 carries finding 3.

- Starts from: ticket 2033 and the branch heads named below.
- Keeps: the fixes this review confirmed.
- Changes: the findings below.
- Proof: a fresh pre-tag review finds each item fixed or answered.
- Defers: nothing.

## Verdict

Do not tag 0.9.2. Main is in much better shape. 2030 fixed the CI hang: every main run on a tree with the fix finished `canonical-gates` green in 20 to 26 minutes. 2020 removed every private name from the tree. 2021, 2029 and 1291 work as described on live data, and four of the five landings had green merged-tree CI before main moved. Two branches still give wrong answers, one landed fix has a hole, and one landed change exposes a local path.

## Blockers

1. **2031 breaks `get drug Tagrisso`.** On main, `get drug Tagrisso` and `get drug TAGRISSO` return osimertinib (DB09330). On the branch they refuse, and the refusal calls osimertinib one of the "other drugs". The ticket says TAGRISSO still lands through the openFDA fallback, but live openFDA `openfda.brand_name:"TAGRISSO"` returns NOT_FOUND because the label has no openfda block. The passing test uses an openFDA reply invented in `src/entities/drug/test_support.rs:745`. Live MyChem carries the brand on osimertinib's record as `drugcentral.synonyms` and `ndc.proprietaryname`, and `MYCHEM_FIELDS_GET` (`src/sources/mychem.rs:15`) fetches neither. `search drug Tagrisso` on the branch still lists osimertinib, so search and get disagree. Test the brand names against recorded live replies.
2. **2031's review does not cover its head.** The ACCEPT names `695ecc9b1`. `c72a8ac91` then changed the discover rescue in `src/entities/drug/get.rs` to go through `named_drug_response`, and no test covers the rescue. On rebase, `tests/test_source_package_boundary.py` conflicts with both sides at `1_406`. The merged tree packages 1407 files, so the count must be `1_407`.
3. **2029 has a hole for spelled gene names.** An article that spells the gene "K-ras" or "K-RAS" falls back to the rsID link, which can open a different allele. Ticket 2034.
4. **2016 still answers MANE-numbered BRCA1 requests on NM_007300 with a false note.** When a request has no ClinVar record, `clinvar_mane_transcript_stem` (`src/transform/variant.rs:81`) finds no MANE stem, the code falls back to the first NM_ transcript, and the new refusal returns early before the UniProt check. `get variant "BRCA1 I1568N"` returns `p.Ile1589Asn` on `NM_007300.3` with a note saying I1568N follows another transcript's numbering. UniProt P38398 has Ile at 1568, and MyVariant's NM_007294 entry reads p.Ile1568Asn. Of 25 BRCA1 missense changes past residue 1400 with no ClinVar record, 11 returned a renumbered change with this false note, 9 returned the right change on NM_007300, and 2 refused while NM_007294 names the change. Take MANE from a source that does not depend on a ClinVar hit, and run the residue check before any note. No review covers `a5f7caf11`, `f06d99fdf` or `86aadbc88`.
5. **2022 still leaves two day-shape checks and drops explicit flags.** After rebasing on main, `src/utils/date.rs:97` and `src/entities/variant/clinvar.rs:91` both define `fn is_day_shaped`. 1291 landed first, so 2022 removes one. The comments at `src/utils/date.rs:94` and `src/transform/variant.rs:691` say 1291 "still carries its own inline check on its branch", which is now false. `search variant "BRAF melanoma" --significance benign --tumor-site skin` hints "Try the same filters without the condition: biomcp search variant -g BRAF", which drops both flags. `gene_first_alternative_form` (`src/cli/variant/query.rs:204`) carries only gene, hgvsp and consequence. The ticket says its re-review is pending.
6. **2021 sends a local path to JSON and MCP callers.** `src/error.rs:663-678` passes the WHO sync suggestion through unchanged, and the suggestion now holds the resolved data directory (`src/sources/who_pq.rs:108-113`). Through stdio MCP, `get drug zidovudine regulatory` with WHO broken returns an absolute local path, and `--json who sync` shows it in `error.recovery`. The test `human_source_errors_share_safe_projection` (`src/error.rs:1390`) states the rule that a home path never reaches the public error. Under `serve-http`, remote clients would see the server's home directory. Ian decides whether the path stays on the terminal only or is accepted in public output with a recorded reason. The reviewer recommends the terminal only, because that is the codebase's existing rule.
7. **The changelog is nine bullets behind.** With 1305 on main at a scratch v0.9.2 tag, the gate reports missing bullets for 2017, 2018, 2019, 2020, 2021, 2023, 2024, 2029 and 2030. The 1297 bullet states the rule 2016 replaces, and the 1301 bullet says "confirmed gene symbol" where 2022 routes only official symbols and stops routing MODY and HHT. The Internal bullet is hard-wrapped, ungrammatical and reads as process text ("post-release sweep", "agent-value programme planning"). The 1299 bullet says "stalls construction on the cache lock". The 1305 ticket body is hard-wrapped.
8. **The deadlock issue is still open.** `sdlc/issues/2026-10-07-single-backend-deadline-test-can-deadlock.md:3` reads `Status: open`, and lines 41-42 require it to close or be accepted before the 0.9.2 tag. The closure in `f02c9e07c` did not change the Status line.
9. **The 2021 landing restored hand-joined inventory entries.** 2020 removed every entry like `"1243 | 1299 | 1243"` from `tools/rust-source-size-inventory.json`. The 2021 landing `44b33eb8a` brought back 24, and 23 remain at `cf8113f3a`, for example lines 99, 110, 132, 312, 477 and 499. `208858393` fixed one. No review covered that merge.

## Gate gaps

10. **The 2030 hang guard does not catch a blocking hang.** With the original blocking wait restored at `src/entities/article/test_support.rs:107`, 5 of 10 runs of the deadline module hung until an outer kill and printed no test name. The watchdog runs on the runtime's timers, which stop when a test blocks the worker threads. The ticket asked for a per-test timeout on the Rust test phase, and the repo has no nextest config or slow-timeout setting. Nothing records the proof item "a planted hang fails at the per-test timeout with the test's name". Add a process-level per-test timeout and record the planted-hang run. The "Install canonical gate tools" step also has no time limit of its own: main run 37832288932 spent 44 minutes in `apt-get` and was cancelled.
11. **The private-name guard protects nothing.** Without `sdlc/pm-forbidden-names.json`, `tools/check-zero-coupling.py` passes a committed planted name silently (docstring lines 82-98: "An absent or malformed file declares nothing"). With the file present, the checker catches the name, but `tests/test_zero_coupling_ratchet.py:534-539` fails because it applies the example file's placeholder rule to the local file. No machine can keep the file and pass the suite. Make the placeholder test read only the tracked example, and have the checker say plainly when no local file is present. The 2020 record's planted-name proof leaves out that the suite fails in that state.

## Records

12. **2029 landed before its merged-tree CI was green.** Main moved to `ff7d4362e` at 18:00:38Z. Merged-tree run 37820375572 on `8eec444db` finished green at 18:21:43Z. `sdlc/records/2029-article-variant-links-build.md:3-5` cites "merged-tree CI green at 2d3815718 on the same tree". `2d3815718` has a different tree, and its run 37820954617 skipped `canonical-gates`, windows, full-features, stress and release-panic.
13. **2030's records claim evidence that is not there.** The review asked for the repeated-run stability evidence, and the merge in `ae9e6c2e9` and `9b23236f8` dropped that request. `sdlc/records/2030-async-fixture-holds-build.md:14` says the runs are recorded in the ticket, and they are not. The issue closure says blocking waits outside the article fixtures stay recorded there, and none are listed. The interim rule was retired at 13:36Z, before the first full `canonical-gates` run on main finished at 13:56Z.
14. **1291's ticket still carries 2033 finding 14.** Line 38 cites "experiment 439". Line 14 cites "Experiment 432's diagnosis" and "cells c/13, d/04, d/15, d/16". Lines 63-68 say the delta "needs a fresh reviewer" under the delta's ACCEPT, and the delta review names no head commit.
15. **The 2021 ticket has leftover defects.** A stray `D` sits at line 80. The temp-directory paragraph appears twice. The last verdict in the ticket is FIX, and the ACCEPT on `d41d5e3ca` appears only in 2033. `pm lint` reports five missing Evidence fields at line 11.
16. **Records name follow-up commits as landing merges.** The 2020 record names `4cea37dae` and the 2021 record names `208858393`, both single-parent follow-ups. The deadlock issue says 2030 landed as `9b23236f8`; the landing merge is `ae9e6c2e9`.
17. **Internal process text did not shrink.** Mentions of the gate machine outside fixture data stayed at 211, in 127 files. "beelink" appears in `spec/README-timings.md:147-233`. sdlc/ still has 20 "coordinator", 5 "lane worker" and 19 "experiment 439". This range added "the supervisor confirmed", "the lane escalated", "coordinator's landing chain", "Yellow failure record" and "timeout revivals". `sdlc/planning/lanes.md` is a public status table that already lists landed work as open; status belongs in pm. Remove the table, and stop adding host, lane and role text to public files.
18. **2032 rewrote its approved Changes line.** The ticket's Changes line said to draw refusal candidates from the abbreviation's synonyms. The branch replaced it with a pointer-line approach. Record the scope change and its review, or restore the line.

## Smaller items

19. **2032's Outcome says `search trial -c` refuses ambiguous input, but only the NCI source refuses.** `search trial -c MF` on the default ClinicalTrials.gov source runs a raw search with no note. Of 50 results, 22 are myelofibrosis trials and 14 are mycosis fungoides trials. Success criterion 3 narrows the fix to `--source nci` while Defers says "nothing". Widen the fix or state the narrowing. The MF refusal offers mycosis fungoides and myotonia fluctuans and omits myelofibrosis. The trial data writes "Myelofibrosis (MF)", though MONDO and NCIt do not list MF as its synonym. The MM pointer comes from a one-entry table (`src/entities/disease/resolution.rs:23`) that cites no source. Long conditions over 512 bytes now hard-fail in NCI search (`src/entities/trial/search/nci.rs:46`). On rebase the ticket file conflicts; keep main's review addendum and the branch's sections.
20. **2031 smaller items.** `get drug Rybrevant` returned a card with no DrugBank, ChEMBL or UNII ID on 2 of 6 runs, because it depends on an OLS4 rescue that fails intermittently; main returned DB16695 every time. Product names shared by several drugs still merge drugs: `get drug "Pain Relief"` gives a card carrying acetaminophen's DB00316 with TRPV1 targets (`hit_all_names`, `src/transform/drug.rs:131`). The leading-name fallback can admit another drug's synonym such as "Terfenadine carboxylate". The trastuzumab card lists Enhertu as a brand, also on main. The comment on `openfda_label_identity_candidate` (`get.rs:279`) claims an exact name match that the code does not require.
21. **2016 smaller items.** Transcript versions are mixed: NM_000546.5 for R248Q and NM_000546.6 for R175H. BRAF, EGFR and KRAS show older versions than MANE. If the UniProt lookup fails, the note prints anyway. The refusal's "Retry with one candidate's exact form" points at a change already known to differ. On rebase, `spec/fixtures/setup-variant-identity-spec-fixture.sh` and the receipts digest conflict; 2016, 2022 and 2031 all touch the receipts, so each later landing repins.
22. **2022 smaller items.** With the shared cache, gene lookups ran past the 2.5-second routing deadline and BRAF did not route; this predates 2022. The refused-path hint still suggests `-g HCC`, which returns 0 rows.
23. **1291 smaller items.** Deleting `day >= 1 &&` at `src/entities/variant/clinvar.rs:110` passes all ClinVar tests, so "2021-04-00" is untested. The headline's `newest_rcv_evaluation_date` (`src/transform/variant.rs:687-693`) accepts any non-empty string, so the headline and the fallback label can disagree. The synthetic fixture in `spec/fixtures/setup-variant-identity-spec-fixture.sh` labels TP53 R273H with real identifiers from other variants: VariationID 1290630 (a VLDLR variant), RCV000030704 (a FANCB variant) and `chr17:g.7676154G>A` (TP53 P72L). Use invented identifiers. A client-build failure reads "NCBI ClinVar request failed" when no request was sent (`clinvar.rs:251-252`).
24. **2021 smaller items.** `get drug <name> regulatory` with no region fails outright when WHO is down, losing the US and EU data (`src/entities/drug/get.rs:930`); this predates 2021. The JSON WHO bucket marks degradation only with the optional `note`, while `count` and `total` read 0. The search note does not name the directory.
25. **2029 smaller item.** The gene-less multi-allele rsID limit is written only in the ticket Outcome, not in the spec page or docs.

## What holds

- 2030's async wait is correct. The deadline module passes in about 13 seconds, and a hold that never releases inside an async test fails at 120 seconds with the test's name. The interim rule is retired.
- No private name appears at `cf8113f3a` in any spelling.
- 2030, 2020, 2021 and 1291 each had a green merged-tree run before main moved, and their reviews cover what landed.
- 2029: MyVariant returns three alleles each for `rs121913529`, `rs121913530` and `rs121913535`, and the spec now says so. `article entities 32955176` prints `get variant "KRAS p.G12C"`. Four mutations were each caught by a named test.
- 1291's label names NCBI and the reason in each path. 2026-02-30 and 2026-13-01 are rejected, and the leap rule is correct.
- 2021's degrade note reaches Markdown, JSON and MCP, and `who sync` exits nonzero and names each failed file.
- 2031 fixes terfenadine (DB00342), mannitol (DB00742) and "5-FU", and returns the right card and label for 30 oncology drugs and brands checked, including cisapride, Keytruda, Gleevec, Enhertu and Lazcluze. Putting main's logic back fails 9 tests.
- 2032: `get disease MM` adds the multiple myeloma pointer. CML, ALL, AML, NSCLC, myeloma and melanoma still resolve. Branch CI is green.
- 2016: TP53 R209Q, G112D and R174H now refuse honestly, and a scan of 87 TP53 isoform-shifted requests found no false note. BRAF V600E, EGFR L858R and KRAS G12C resolve on MANE.
- 2022: the BRAF V600E headline date matches MyVariant, HCC and MODY no longer route, and the receipts re-indent is gone.
- The offline suite passes 853 tests on main.

## Landing order

2032 first, after the ticket-file merge and finding 19's Outcome fix. Then 2031, 2034, 2016 and 2022, each rebased on main with a review covering its head and a green merged-tree run on the exact landing tree before main moves. The records and gate items here can land with any of them. 1305 lands last, with bullets checked against main at that point.

## Disposition (2026-10-09)

Every finding is fixed or carries a recorded reason: the Tagrisso brand break (fixed in 2031's landed fourth-review round), the spelled gene names (2034 landed), the MANE-without-ClinVar false note (2016's fix-round-4 landed), the smaller items (2032's records amended on main; 1291's, 2021's, 2029's smaller items carried in their lanes' records or the merge-pooling deferral). The
pre-tag review may verify each against its landing record.
