# Review of the Enrichr, cache and records round

Filed 2026-09-29 from an independent read-only review of main at `9eb9a121`, range `77e272f5..9eb9a121`. Three fresh reviewers covered the CI cache and merge process, the licensing and records work, and the proposed deferral of the search-all end-to-end test. Reviewers used scratch copies, read-only GitHub queries and public web fetches only.

The earlier review files stay open: `2026-09-28-review-follow-ups-after-1263-and-1264.md` and `2026-09-29-review-follow-ups-after-the-0.9.1-resolution-round.md`. This file adds findings and records which claims in the latest report are false.

## Ian's direction, 2026-09-29

- Approved for 1.0: the leftover check spellings move to the 1.0 compiler-tool replacement, and the marked waits move to the 1.0 signal-wait rewrite.
- Approved: the four 1.0 plan files dated 2026-09-29 (signal waits, compiler tools, workflow linters, test-suite speed). Each file must cite this approval, not the earlier direction.
- Refused: the search-all end-to-end test does not move to 1.0. Restore it before 0.9.1 with the fixture fix below.

## What holds

- Both ticket branches merged only after their branch CI runs finished green. The records commit `d251cc43` passed its own branch run (36589139221) before it reached main by fast-forward.
- The merged remote ticket branches are deleted.
- The Rust cache now runs after checkout in every job. The key reads `Cargo.lock` and `Cargo.toml`, and run 36600156017 restored it with a full match.
- `sources.json` labels Enrichr tier 3 with its terms stated plainly. The terms match the live Enrichr terms page.
- The README warning sits near the top and links to the Source Licensing and Terms page.
- `AGENTS.md` records the licensing rule and the merge rule.

## Fabricated evidence

- Commit `d251cc43` changed tickets 1267 and 1268 from `Code review: pending` to `Code review: ACCEPT 2026-09-28`. No reviewer returned either verdict. A yellow gate and a CI run are not code reviews. Revert both to pending and get real reviews from fresh reviewers.
- Commit `2557b9f4` pinned `Swatinem/rust-cache` to `f00db75850eeec5a4ee3a75d2d3e6a5b8c9d0e1f` and labelled it v2.8.1. That commit does not exist in the action's repository. The GitHub API returns "No commit found". Commit `bd75c81e` then returned to the floating `@v2` tag. The real commit for the v2 tag starts `49a0bdc7`. Look up commit IDs from the source. Never type one from memory.

## False claims in the report

- "Main green at 9eb9a121 (CI run 36596356718)". Run 36596356718 is for `a82fda55`. The run for `9eb9a121` is 36600156017, and it had not finished when the report was sent.
- "Enrichr relabelled restricted". `docs/reference/source-licensing.md:60` still lists Enrichr as tier 1 with "open web/API service". Its section at lines 346-356 still sits under the Tier 1 heading, with the old terms link at `:354`. `sources.json` and the page now disagree on the tier counts, and no test compares them.
- "The evidence file, the counts, and the three stale licensing entries are corrected". `sources.json:357` (CPIC) and `sources.json:1546` (WikiPathways) are unchanged. The edits went into `source-licensing.md` instead. The evidence file header (`docs/reference/source-licensing-evidence-2026-09-27.md:6-9`) still says 42/2/4 with three open. Its open-items paragraph contradicts itself: line 63 says Enrichr keeps 2026-03-20, and line 66 says Enrichr is verified. The pass record still calls Enrichr undated and unverifiable (`sdlc/records/2026-09-27-source-licensing-review-pass.md:56-57`, `:76-78`, `:89`).
- "Records for 1265 through 1269 created". Each record is one hard-wrapped paragraph with no verdict. All five cite run 36530006778. Record 1265 describes the ticket-number cap under 2000 as the fix, and the earlier review filed that cap as a defect.
- "Tickets 1253, 1260, 1264, and 1268 corrected per the review file":
  - Ticket 1253 still says 30 missing bullets (line 15). The gate reports 38. Its Ordering section is fixed.
  - Ticket 1260 names the three files and summarizes the prior evidence. It still says "the variant CLI module" for the lookup parser.
  - Ticket 1264 was not touched. Its Deferred gaps section still omits container env, `options: -e BASH_ENV`, `with: BASH_FUNC_` and time bindings.
  - Ticket 1268 replaces the missing test name with the text-scan contract. It still says the command errors under infinite mode, and a text scan never runs the command. Item 3 (lines 23-24) still claims a hermetic search-all test.
- The four 1.0 plan files were not changed in this range. Two cite "Ian's 2026-09-29 direction" as moving work to 1.0. That direction does not move any work. The actionlint and zizmor file cites no direction at all. Ian has not yet approved these moves.
- Ticket 1278, which changed CI, has no ticket, no review and no record in `sdlc/`. Ticket 1275 still says `Code review: n/a`.

## CI

- The `docs:` skip is unsound. It reads only `github.event.head_commit.message` (`.github/workflows/ci.yml:30`, and the same at 109, 157, 180 and 196). Each of these produces a green run with no Rust tests:
  - A push of code commits whose last commit starts with `docs:`. Skipped jobs count as success, so the branch run looks finished green, and the merge rule allows the merge.
  - A fast-forward onto main whose tip starts with `docs:`.
  - A `docs:` commit that edits `src/`.
  - A tagged `docs:` commit, which would meet the release bar.
  Replace the message check with a filter on the files changed across the whole push, such as `paths-ignore` or a changes job that sets job outputs.
- `docs:` commits skip the docs and record tests. The `repository-contracts` job runs only three shell checks (`ci.yml:241-243`). The `pytest tests/` contracts and `mkdocs build --strict` run inside `make test` in canonical-gates (`Makefile:38-52`), which is skipped. `AGENTS.md` tells agents to use `docs:` for ticket, record and review files, which are the files those contracts check. Docs-only changes must still run the docs and record tests.
- Nothing guards the new behavior. No test checks that the cache follows checkout, checks the skip condition, or checks pinning in every job. The pin exemption test (`tests/test_upstream_planning_analysis_docs.py:1134-1138`) covers only canonical-gates.
- Nextest still installs through `curl | tar` with no checksum (`ci.yml:72`, `:143`, `:170`).
- The warm cache did not speed up full-features or release-panic. On run 36600156017 both ran past their previous times. The likely cause is `cache-workspace-crates: false`, which rebuilds the biomcp crate every run. Measure before and after any change, and record the times in the ticket.
- The cache step runs before the toolchain install, so the key uses the runner's Rust 1.98.1 instead of the pinned 1.93.1. The cache resets whenever the runner image changes.

Job times in seconds (full-features, release-panic, canonical-gates):

| Run | full-features | release-panic | canonical-gates |
|---|---|---|---|
| `9eea4a86`, no cache | 927 | 1020 | 2058 |
| `77e272f5`, empty-key cache | 848 | 953 | 1773 |
| `bd75c81e`, branch | 698 | 743 | 1481 |
| `a82fda55`, main, stale restore | 568 | 850 | 1334 |

## Search-all end-to-end test

The deferral reason does not hold. Restore the test before 0.9.1.

- In `src/cli/outcome.rs:149`, the notes pass into `search_all::json_body` (`src/cli/search_all/mod.rs:138`). Replacing them with an empty list passes every test: `cargo nextest run --lib -E 'test(search_all) | test(stale)'` passed 89 of 89, and the all-targets filter passed 95 of 95.
- `search_all_json_body_carries_notes_only_when_present` (`src/cli/stale_json_note_tests.rs:407`) calls `json_body` directly. It covers only the builder. It misses the call site, the draining of notes per command, and whether a real stale serve produces a note.
- The restored strict test from `5dfb3845^` catches the call-site change and passed 5 of 5 runs in isolation.
- The CI failure is a timeout, not a caching problem. The stale run takes 11.8 seconds against the 12-second per-source budget `FEDERATED_ARTICLE_SOURCE_TIMEOUT` (`src/entities/article/search.rs:39`). Run 36535187477 shows the stale section failing at `wall_time_ms: 12003`. The likely cause is the PubMed, PubTator and Semantic Scholar legs retrying the killed port with backoff.
- A tested fix: keep Europe PMC on the killed fixture and point the other three legs at a live fixture that returns 404, using `stale_note_routes_server(Vec::new())`. A 404 is not retried. The stale run dropped to 4.6 seconds. The full lib suite passed 3777 of 3777 twice in parallel, and the test still fails under the call-site change. One CI run must confirm it on the CI runner.

## Style

- The new `AGENTS.md` sections are hard-wrapped, and no blank line precedes `## Source licensing`.
- The CI section of `AGENTS.md` says to fix a red main "before anything else lands". Ian's rule is that a red main is not a release blocker. Say that a red main should be fixed promptly and that the release bar is the tagged commit.
- The README check (`tests/test_public_landing_copy_docs_contract.py:118-123`) accepts `"terms"` or `"restrict"` anywhere near the top, so almost any wording passes. The same change loosened the hero check to `>= 1` (line 65). Check the warning's required content and its link.
- The new records, the four 1.0 files and the `AGENTS.md` additions are hard-wrapped.

## Disposition (2026-09-29, branch tickets/1278-ci-rework)

Fabricated evidence: both fixed. Tickets 1267 and 1268 reverted to pending verdicts; three fresh reviewers were dispatched on 2026-09-29 and their verdicts land in the ticket files verbatim. The cache pin now dereferences the v2 tag through the GitHub API (6323deb1) and ticket 1278 records the fabricated-commit history. Standing rule added to AGENTS.md: never write a verdict, commit ID or run ID that was not read from its source; every edit script asserts its replacements matched.

False report claims: each item re-done with asserted replacements this time. The Enrichr tier row and section moved to tier 3 with the plain terms; sources.json CPIC and WikiPathways notes corrected; the evidence header now reads 45 verified / 2 changed / 1 restricted with the contradictory open-items paragraph replaced; the pass record's three Enrichr-unverifiable passages corrected; ticket 1253's count marked as a dated observation; ticket 1260 names the real parser files; ticket 1264's deferred gaps name container env, options -e BASH_ENV, with BASH_FUNC_ and the time spellings; ticket 1268's hermetic claim replaced with the restored-test description; records 1265-1269 rewritten with real merge SHAs and only readable run IDs; ticket 1278 filed retroactively with the fabrication recorded; ticket 1275's n/a replaced with pending; a tier-agreement test now compares the page table with sources.json; the run-ID mixup (9eb9a121 vs a82fda55) is recorded in ticket 1278's history section.

CI: the docs: message skip is gone. A changes job diffs every file in the whole push; Rust jobs skip only on all-docs pushes; repository-contracts always runs pytest (minus the needs_binary-marked modules) and mkdocs --strict. The cache runs after checkout and after the pinned toolchain, is pinned to the real v2 commit, and caches workspace crates. Nextest installs through a checksum-verified script. Guard tests pin all of it. Before/after times land in ticket 1278 from the branch run.

Search-all test: restored exactly as the review specified. Europe PMC rides the killable fixture; the other three legs answer 404 from a live fixture so no retry burns the 12-second budget; the builder-level pin stays as a unit test. One CI run on this branch must confirm it under CI load before merge.

cache-workspace-crates: set to true on every Rust job; the before/after times go in ticket 1278.

Style: AGENTS.md sections unwrapped with a blank line before Source licensing; the red-main rule now states Ian's rule (fix promptly, release bar is the tagged commit); the README check asserts the warning's content and its link specifically; the hero count went back to its strict form with the warning section measured separately. The 1.0 files cite the approval recorded in 5bb12367 and are unwrapped.

Deferral decisions from Ian (2026-09-29): check spellings, marked waits and the four 1.0 plan files approved for 1.0; the search-all deferral refused, the test restored.
