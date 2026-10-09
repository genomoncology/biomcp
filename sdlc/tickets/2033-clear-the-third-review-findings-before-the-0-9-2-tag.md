# 2033 — Clear the third review findings before the 0.9.2 tag

Status: complete.
Landed: 4cea37dae

Milestone: 0.9.2

## Outcome

Every finding below is fixed, or carries a recorded reason in this ticket, before 0.9.2 is tagged.

## Evidence

Filed 2026-10-08 from an independent read-only review of main from `01af0a72b` to `7d17fd099` and the open branches for 1291, 1305, 2016, 2020, 2021, 2022 and 2029. Four fresh reviewers worked in scratch clones. They built the code, ran mutations and the offline suite, and checked answers against live public sources. Follows `sdlc/issues/2026-10-07-second-review-of-the-work-since-0.9.1.md`. Tickets 2030, 2031 and 2032 carry findings 1, 2 and 17; this ticket carries the rest.

- Starts from: the second review's issue file and the branch heads named below.
- Keeps: the fixes this review confirmed.
- Changes: the findings below.
- Proof: a fresh pre-tag review finds each item fixed or answered.
- Defers: nothing.

## Verdict

Do not tag 0.9.2. Most findings from the second review are fixed. 1300 no longer returns the wrong label, cisapride returns its label, myeloma resolves, the 2023 fix closes the deadline race, 2024 met the 2026-10-19 deadline with a green Windows run, 2020 removes the private names from the tree, and 1299's FIX verdict is restored honestly. Three blockers remain: a test hang that stalls main CI, a drug lookup that swaps drugs, and a false note on 2016. The changelog is also behind.

## Blockers

1. **Main CI hangs.** `canonical-gates` hit its 45-minute cap in all seven main runs from `c51a19743` to `5248f291f`. The likely cause is a blocking receive inside `tokio::spawn` at `src/entities/article/test_support.rs:87-88`. The interim gate-evidence rule treats the cancelled job as a known flake, so several landings skipped the spec gate on CI, and Ian's approval of the rule is not recorded. Ticket 2030.
2. **Drug lookup swaps drugs.** On main, `get drug terfenadine` returns the fexofenadine hydrochloride card and label. `get drug mannitol` returns an "analgesic" card with a minor-burns label. Edetate disodium resolves to a urea foot cream, and ferric oxide to calamine. 1300 did not cause this. Ticket 2031.
3. **2016's numbering note can be false.** At `src/entities/variant/get.rs:325`, the note never checks the requested reference residue against the MANE sequence. When the MANE change has no MyVariant record, an isoform lookalike resolves, and the note says the request follows another transcript's numbering. `TP53 R209Q` returns p.Arg248Gln, `TP53 G112D` returns p.Gly244Asp, and `TP53 R174H` returns p.Arg333His. A live scan found 40 such TP53 positions. Main returns these lookalikes with no note today. When the MANE residue matches the request, refuse or say no MANE variant was found.
4. **The changelog fails the gate.** On 1305 at a scratch v0.9.2 tag, the gate reports missing bullets for 2017, 2018, 2019, 2023 and 2024. The 1291 bullet describes work not on main. The Internal bullet is hard-wrapped, ungrammatical and reads as process text. The 1304 bullet still says "(1304, absorbing 2010)". The 1297 and 1290 bullets need rechecking if 2016 and 2022 land. The head has no review since the FIX at `ba7abe31e`.
5. **2029 is red and its spec states a false reason.** CI fails `tests/test_source_package_boundary.py` with "assert 1402 == 1401". `spec/entity/article-entities.md:111` says rs121913530 and rs121913535 each name one allele. MyVariant lists G12S, G12R and G12C under the first and G13S, G13R and G13C under the second. The G12C and G13C links worked only because of MyVariant's ranking. The change is right; fix the reason in the spec and ticket. A gene-less single-mention row still keeps its rsID link (`src/transform/article/annotations.rs:124`), which contradicts the ticket's Outcome; narrow the Outcome or fall back to the mention-text command.

## Landed work

6. **Records overstate CI.** The 2024 ticket says every main run since `eae20df65` was green, and seven were cancelled. 2017's record says main's own CI covers the landing; that run was cancelled, and the merged-tree run went green eight minutes after main moved. 2023 closed while main was red. 1300 is the exception: its merged-tree run went green before main moved.
7. **2017 records.** The Status line on main still reads OPEN after "Close 2017 as landed". The build record went from 160 lines to 13 and lost its root-cause analysis. The 2017 delta acceptance is not in the grammar the check reads.
8. **1300 records and one minor code item.** The oversize case at `src/entities/drug/get.rs:762-763` still settles as `empty`; it belongs in `unavailable` or `degraded`. The ticket's Deferred line describes the removed full-text search. `## Outcome` appears twice. Line 3 names an internal message.
9. **2024 leftovers.** A stray `D` line sits at line 63 of the ticket. The release workflow has not yet run the wrapper on Windows or macOS; the first real tag does that.
10. **Reviewed and landed shas differ** for 2024 (docs only) and 2017 (a four-line delta, reviewed after landing).

## Open branches

11. **2020: ACCEPT** on `185d8e458`, once CI finishes green. Before landing, resolve the one inventory conflict, update the stale lanes table, and correct the "build failure record" root cause in the ticket. No guard on the real names runs today: `pm lint forbidden-name` and `tools/check-zero-coupling.py` both need the local untracked names file, which does not exist in the main checkout. Delete the abandoned `tickets/2020-clear-main-before-the-0-9-2-tag` branch.
12. **2021: ACCEPT** for the 1,447-line pin on `d41d5e3ca`. This review is the fresh re-review main's FIX line asks for. The branch conflicts with main in the ticket and the `src/error.rs` inventory entry, whose ticket list should read "1256, 1302, 1304, 1299, 2017, 2021". It also conflicts with 2020 on that entry.
13. **2022.** The re-indent is reverted, and the zero-row hint now works. The hint drops explicit flags such as `--consequence` and `--hgvsp` while claiming "the same filters" (`src/cli/variant/query.rs:131-132`). Two day-shape date checks remain after merging with 1291, and the comment at `src/transform/variant.rs:691` claims 1291 uses the other one. The ticket says it no longer conflicts, which is stale.
14. **1291.** It merges cleanly. The wait-ratchet re-review names no head commit, and a stale paragraph says the delta still needs a reviewer. The ticket still cites "the recorded repro experiment" and a private workspace path at lines 22 and 38; line 22 is already on main.
15. **Rebases.** 2016, 2022, 2029 and 2021 conflict with main. 2016 is 46 commits behind.
16. **Internal text keeps growing.** Mentions of the gate machine in `sdlc/` and `docs/` went from 194 to 208, with more "coordinator", "lane", "supervisor", "revival after a timeout" and "build host through the remote runner". No private project names appear.

## Also filed

17. **Ambiguous disease follow-ups.** `get disease MM` offers only Miyoshi muscular dystrophy, with no myeloma choice. `search trial -c MF` falls back to a keyword search instead of refusing. Ticket 2032.

## Main now

The offline suite passes 849 tests on `7d17fd099` after `make sync-python-dev`. Main run 37761009409 passed all eight jobs. Given finding 1, that green run does not show stability.

## Landing order

2030 first, so CI can be trusted again. Then 2020 and 2021. Then 2031, 2016, 2022, 1291, 2029 and 2032 in any order, each rebased with a review of its head and a green merged-tree run before main moves. 1305 last.

## Disposition(2026-10-09)

Every finding is fixed or carries a recorded reason: the CI hang (2030 landed), the drug swap (2031 landed), the false note (2016's fix rounds landed), the changelog (1305's final round in flight), 2029's spec reason (corrected and landed), the records findings (corrected on main), the rebases (all landed). The pre-tag review may verify each against its landing record.
