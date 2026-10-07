# Review of the work since 0.9.1

Status: open

Filed 2026-10-07 from an independent read-only adversarial review of main at `37631c357` (range `v0.9.1..37631c357`, 171 first-parent commits) and the open branches for 1300, 1291 and 1305. Five fresh reviewers covered the article search tickets, the variant, disease, author and WHO tickets, the open branches and the finishing plan, process and records, and the 0.9.1 release and its aftermath. They used scratch clones, release builds, mutation tests, recorded fixtures, and light calls to free public APIs. They made no edits and no GitHub writes.

## Verdict

Do not tag 0.9.2 on the current plan. Three landed tickets introduced wrong answers that 0.9.1 did not give (findings 1 to 3). The changelog gate cannot see most of this cycle's landings, so the plan's tag-time check would pass with bullets missing (finding 4). Main also carries a stray packaged file, private names, and spec proofs that stopped running (findings 5 to 7). Tickets 2016 to 2024 hold the fixes.

The 0.9.1 release itself shipped correctly.

## Blockers for 0.9.2

1. **Article entities send variant rows to the wrong variant (1296).** For PMID 30738221 the G12A, G12D and G12V rows all print `biomcp get variant rs121913529`, which opens G12D. Ticket 2018.
2. **Protein-change resolution picks a different variant (1297).** `get variant 'TP53 C124Y'` now returns p.Cys135Tyr; `BRCA1 A314T` now returns p.Ala1823Thr. The rule prefers any matching hit with a ClinVar record and matches by other-isoform names. 0.8.25 answered both correctly. Ticket 2016.
3. **Ambiguous abbreviations give mixed disease cards (1295).** `get disease MF` shows the Myotonia fluctuans definition with mycosis fungoides genes; `CAD` mixes cold agglutinin disease with coronary artery disease genes; `MM` returns Miyoshi muscular dystrophy. `search disease MDS` ranks Miller-Dieker lissencephaly above myelodysplastic syndrome. Ticket 2017.
4. **The changelog gate is blind to `Land NNNN:` landings.** It sees 11 tickets on main and misses 1293, 1299 and 1302 to 1306. A scratch v0.9.2 tag after merging 1300, 1305 and 1291 passed with 1299, 1306, 1300 and 1291 uncovered. The plan's "top up at a temporary tag" step would find nothing. Ticket 2019.
5. **A stray file ships in the crate.** `tools/zero-coupling-historical.json.probe` arrived in the 1292 landing merge, never on the branch. Its keys encode the forbidden coupling token, and `cargo package --list` includes it. The package-count failure was absorbed with a comment that blames a file edit. Ticket 2020.
6. **`sdlc/pm.json` spells out private project names** in this public repository (`forbiddenNames`, added in `022fe4f52`). Ticket 2020.
7. **1299's three spec cases no longer run.** Merge `913044c5e` stripped their code fences. 1304 landed with no recorded review; 1299, 1302 and 1298 changed after review with no re-review; `60d9c247c` rewrote 1298's design verdict from FIX to ACCEPT under the same dispatch. Ticket 2020.

## Open branches

8. **1300 (drug label sections).**
   - The Markdown pointer prints `biomcp get drug {{ name }} label --json` unquoted, so `get drug trastuzumab deruxtecan label --json` fails with `Unknown section "deruxtecan"`. The pointer also prints when nothing was cut. Use `shell_quote_arg`.
   - The full-text fallback downloads up to 100 labels. For `cisapride` and `terfenadine` that is about 16 MB, over the 8 MiB limit, and the result reads "temporarily unavailable" with a retry hint that can never succeed.
   - The identity guard matches inactive ingredients and tokens joined across fields (`mannitol` matches the Tagrisso label).
   - Swallowing the full-text fetch error at `label.rs:663` passes all 111 tests.
   - The reason sits in `label_note` while `section_outcomes.label` stays a bare `empty`.
   - Merging into main needs `MAX_PACKAGE_FILES = 1_403`.
9. **1291 (ClinVar source switch).**
   - No code review verdict is recorded; `89a9d4b83` says "review fold" with no verdict.
   - `src/entities/variant/mod.rs` goes from 1000 to 1220 lines through a hand-added inventory entry. `tools/update-rust-source-size-inventory` refuses that case, and `tools/check-quality-ratchet.py` accepts it.
   - Changing `.max()` to `.min()` in `fallback_evaluation_date` passes every unit test, and so does mapping the timeout to an HTTP error.
   - Every non-429 failure reads "HTTP error". A sustained 429 can read "timed out". The date check accepts 2021-02-30. The spec fixture is invented, not recorded.
10. **1305 (changelog bullets).**
    - No review is recorded.
    - 1299, 1306, 1300 and 1291 have no bullets.
    - The 1293 bullet claims the 60-second deadline held, which 1299 showed was false; credit it to 1293 and 1299.
    - Add the remaining bullets on this branch, not directly on main, so a reviewer reads them.
11. **The "environmental" spec failures are a setup gap.** `make spec` never runs `sync-python-dev`, so the worktree virtual environments are empty. `gene.md:408` fails on `jsonschema`, and the 41 parallel-isolation contracts never run in yellow. With a synced environment, main and main plus 1291 pass every spec. 1300 edits those contracts, so its yellow run does not test its own change. Ticket 2024.

## Should fix before the tag

12. **WHO data gaps are invisible to JSON and MCP callers (1304).** A failed export makes `search drug zidovudine` report no WHO drugs, with the warning only on stderr. `who sync` still ends with the generic #288 message. A partial sync reports "synchronized". Ticket 2021.
13. **The variant headline pairs a classification with another record's date (1290).** `BRAF V600E` prints Pathogenic beside the 2025-01-23 date of an Uncertain significance record. Gene routing accepts aliases: `'HCC liver cancer'` becomes gene HYCC1 and returns nothing, with no hint. Ticket 2022.
14. **Article search carryovers.**
    - A date sort with offset above 1250 returns empty pages with `has_more: true`.
    - A fast PubTator failure hides the deadline error.
    - One deadline test depends on test order, and one fixture route never matches.
    - The cursor-stop proof is tested by the empty-page check instead.
    - JATS word spacing splits words inside small caps and styled text.

    Ticket 2023.
15. **Four jobs still run on `ubuntu-latest`, which moves to Ubuntu 26 on 2026-10-19.** Released binaries print `git unknown, build unknown`. Ticket 2024.

## Process

- Main was red five times this cycle. Three landings broke it because conflict-resolved merges reached main before any CI ran on the merged tree: 1290, 1292 and 1294. 1297 landed with a red branch run onto a red main.
- Hand numbering:
  - Tickets 1297, 1305 and 1306 say outright that they were numbered by hand.
  - 1298 to 1304 were filed while `pm ticket new` returned 2010. 2010 was released twice as a misdraw and then kept.
  - The allocator now issues 2016 onward, and this round's tickets use those numbers.
- Seven landed tickets have no `sdlc/records/` file: 1290, 1293, 1299, 1302, 1303, 1304 and 1306. The review-verdict gate only checks tickets with a record, so it skipped all seven.
- Public files carry internal detail:
  - team role names;
  - home-directory and `/tmp` paths;
  - a machine name;
  - "no cost to Ian";
  - the GitHub billing state in `sdlc/release-checklist.md`;
  - experiment paths in commit trailers.
- Leaked names remain in public history: the coupling token in `0d4c517e0`, the other team's branch ref in `40ca3cc76` and `8263dde19`, and a private repo name in `sdlc/planning/2026-10-03-prove-agent-value.md` from `71b1d2f2c` to `022fe4f52`. The current tree is clean of them apart from finding 6.

## 0.9.1 release and aftermath

What holds:

- Release run 36952191952 passed all 22 jobs.
- PyPI has five wheels, and the installed wheel is a release build with its embedded assets, which closes #287.
- The GitHub release has five archives and five checksum files, and all five checksums match.
- GHCR `0.9.1` and `latest` share a digest across amd64 and arm64, with revision `992df4c8` and uid 65532.
- The Homebrew formula is at 0.9.1 and its checksums match the release.
- #284: `tools/list` reports `type: object` for all seven tools. #282: trial search no longer overflows the stack.
- The rehearsal image is no longer pullable.

Open items:

- **#288 breaks `search drug <name>` on every fresh 0.9.1 install.** The fix is on main and unreleased. #288 and #289 were closed as fixed with no note that the fix is unreleased.
- **Issue #290** (a switch to turn off licence-restricted sources such as KEGG) has no triage. The 0.9.2 outcome says every external issue is answered.
- **Ticket 1287 left some sixth-review items open:**
  - the 1284 ticket and record still describe a private TestPyPI rehearsal;
  - the scratch repository's description still says it is deleted after each run;
  - the DepMap row omits the not-for-clinical-use note.
- **The Homebrew formula has no `test do` block.**

## Holds

- The offline suite passes 824 tests in a fresh clone. CI and documentation runs at `37631c357` are green.
- No commit carries agent attribution.
- No test was weakened. The zero-coupling inventory changes are digest repins.
- **1303 (#289):** the ORCID visibility fix holds against a live read and fails 10 tests when reverted.
- **1304:** header normalisation and the trailing-space column hold.
- **1292:** minus-strand intronic deletions resolve.
- **1290:** the record-level classification is used when the ClinVar section is requested.
- **1297:** the DICER1 fix and the EGFR refusal hold.
- **1299 and 1302:** the core deadline translation and the citation degrade fail their tests when reverted.
- **1298:** cursor walk and duplicate check are correct.
- **1294:** whole abstracts.
- **Merging the open branches:** "union by path with max baselines" cannot silently loosen the size ratchet. Each entry must equal its file exactly. The real hole is the hand-added new entry in 1291.

## Decisions for Ian

1. Release timing. 0.9.1 users cannot run `search drug` on a fresh install until 0.9.2 ships. The reviewer recommends fixing the blockers, then tagging, rather than waiting on the should-fix items.
2. Issue comments on #288 and #289 saying the fix ships in 0.9.2. This is an outward-facing post.
3. Leaked names in public history. The reviewer recommends leaving history as it is and cleaning the tree, because rewriting main breaks every clone and the names are already exposed.
