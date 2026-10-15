# 2083 — Stop changelog conflicts between ticket branches

Status: COMPLETE.

Milestone: 0.9.3

Reviews: revision 8b08177, accept

## Outcome

Two ticket branches that each add a changelog bullet merge without a conflict, and no test lists the tickets of the release in progress by hand.

## Evidence

- `tests/test_docs_changelog_refresh.py` keeps `EXPECTED_RELEASE_TICKETS`, whose `DEV_CYCLE_VERSION` entry lists every ticket landed for 0.9.3 by hand, and `test_dev_cycle_section_carries_its_own_tickets` compares it with the section. Every landing appends one line to that list and one bullet to the end of a 0.9.3 subsection in `CHANGELOG.md`, so any two branches clash. In the 0.9.3 landings so far, the 2064 branch spent 2 of its 13 commits merging these two files, and the 2069 branch needed a conflict fix that glued a heading to the next line.
- `scripts/check-changelog-coverage.py` already proves at tag time that every landed ticket has a described bullet (`described_tickets`, line 180; `main`, line 230), so the hand list adds no protection for the release in progress.
- 2074 runs the coverage check on every branch and lets process-only tickets skip a bullet; it changes neither the file shape nor this test.

- Starts from: `tests/test_docs_changelog_refresh.py` (`EXPECTED_RELEASE_TICKETS` at line 13, `test_dev_cycle_section_carries_its_own_tickets` at line 540); `scripts/check-changelog-coverage.py` (`described_tickets`, `main`); `.gitattributes`; the 0.9.3 section of `CHANGELOG.md`, which has the subsections New features, Fixes and Internal.
- Keeps: the hand lists for released versions, which never change; the tag-time coverage check; the section's place above `## 0.9.2`.
- Changes: claims `tests/test_docs_changelog_refresh.py` `scripts/check-changelog-coverage.py` `.gitattributes` `CHANGELOG.md` `AGENTS.md`.
  - Delete the `DEV_CYCLE_VERSION` entry from `EXPECTED_RELEASE_TICKETS`. `test_dev_cycle_section_carries_its_own_tickets` checks shape only: the section sits above the last released heading, it carries the New features, Fixes and Internal headings, every bullet ends with a ticket number in parentheses, and every ticket it cites has a ticket file under `sdlc/tickets/` with `Milestone: 0.9.3`.
  - Add that last check to `scripts/check-changelog-coverage.py` as one function the test imports, and call it from `main` so the tag run applies it to the released section too.
  - Add `CHANGELOG.md merge=union` to `.gitattributes`.
  - Keep all three 0.9.3 subsection headings present even when empty, so no branch adds a heading. Say so in one sentence after `AGENTS.md:68`, which describes changelog bullets.
  - One Internal changelog bullet.
- Proof: two scratch branches from main, each adding a bullet under Fixes, merge with no conflict; the shape test fails on a planted bullet citing a ticket of another milestone and on a bullet with no ticket number; the coverage check still passes on tag v0.9.2 (`scripts/check-changelog-coverage.py --tag v0.9.2`).
- Defers: running the coverage check on every branch, and skipping process-only tickets (2074).
- Order: right after 2070, before the other 0.9.3 tickets, because every later ticket edits these two files.
- Built on branch `ticket/2083-changelog-conflicts`: `EXPECTED_RELEASE_TICKETS` lost its `0.9.3` entry and `test_dev_cycle_section_carries_its_own_tickets` checks shape only — the section sits above the released heading, carries the New features, Fixes and Internal headings, every bullet closes with a ticket number in parentheses, and `milestone_problems` (one new function in `scripts/check-changelog-coverage.py`, called from `main` and imported by the test) reports every cited ticket that is not filed for the milestone. `.gitattributes` gained `CHANGELOG.md merge=union`, `AGENTS.md` now says the three headings stay in place even when a section is empty, and `CHANGELOG.md` carries one Internal bullet. No test lists the dev-cycle tickets by hand anymore.
- The tag run tolerates a cited ticket with no file or no Milestone line — 1287, 1290, 1292, 1293, 1295, 1296, 1297, 1301 and 1303 carry no Milestone line, and 1288 and 1289 have no ticket file — and fails only a ticket whose file names another milestone.
- Proof runs: two scratch branches from main (before the change), each adding a Fixes bullet, merge with `CONFLICT (content): Merge conflict in CHANGELOG.md` (exit 1); the same two branches from the change merge clean under the union driver (exit 0, both bullets kept, `git check-attr merge -- CHANGELOG.md` prints `union`); a planted bullet citing 2040 fails the shape test with `tickets cited outside milestone 0.9.3: {'2040': '0.9.2'}` and a bullet without a ticket number fails with `a dev-cycle bullet must close with its ticket number`; `python3 scripts/check-changelog-coverage.py --tag v0.9.2` prints `changelog 0.9.2 covers all 58 tickets merged since v0.9.1`. New gate tests live in `tests/test_changelog_coverage_check.py` and `tests/test_binary_asset_gitattributes_contract.py`.
