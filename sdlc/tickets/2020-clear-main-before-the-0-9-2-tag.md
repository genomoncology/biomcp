# 2020 — Clear main before the 0.9.2 tag

Status: OPEN.

Milestone: 0.9.2

## Outcome

Main carries no stray files, no private names, and no spec proofs that stopped running. Every 0.9.2 landing has a recorded fresh code review of what landed.

## Evidence

Filed 2026-10-07 from the review of the work since v0.9.1 (`sdlc/issues/2026-10-07-review-of-the-work-since-0.9.1.md`, findings 5 to 9).

1. `tools/zero-coupling-historical.json.probe` reached main in the 1292 landing merge `152984a12`. It was never on the ticket branch. It is an old copy of the coupling inventory, and its keys encode the forbidden coupling token. `Cargo.toml` excludes the inventory but not the probe, so `cargo package --list` ships it. The package-count failure it caused was then absorbed into `MAX_PACKAGE_FILES`. The comment in `tests/test_source_package_boundary.py:23` blames "1292's variant-input-forms fixture handler edit", which adds no file.
2. `sdlc/pm.json` `forbiddenNames` (added in `022fe4f52`) spells out private project names in this public repository.
3. Merge `913044c5e` stripped the code fences from three 1299 sections of `spec/entity/article.md` (single-backend expiry, the deadline error, the held cache lock). `mustmatch test -v article.md` runs none of them. `e035cd7ad` re-fenced only the 1298 section.
4. Seven entries in `tools/rust-source-size-inventory.json` carry hand-joined merge text, such as `"ticket": "1243 | 1299 | 1243"` near line 132.
5. Reviews missing for what landed: 1304 has no recorded ticket review or code review beyond a merge-commit trailer. 1299's post-review merge `913044c5e` changed `src/error.rs`, `src/entities/article/variant_search.rs` and the deadline tests with no re-review. 1302 (`8e3c84498`) and 1298 (`2d75971de`) added commits after review. `60d9c247c` rewrote 1298's design-review line from FIX to ACCEPT under the same dispatch.
6. Stale records: `sdlc/tickets/1284-*.md:13` and `sdlc/records/1284-*.md:11` still describe a private rehearsal using TestPyPI. `sdlc/planning/lanes.md` lists the removed 1294 worktree and omits 1291, 1300 and 1305. `sdlc/planning/milestones.md` still names a QA step that commits `5048fd906` and `612b0414b` dropped without saying who waived it.

- Starts from: the six findings above.
- Keeps: the coupling inventory, the forbidden-name lint, the size ratchet.
- Changes: delete the probe and lower `MAX_PACKAGE_FILES` by one with a true comment; move the forbidden names out of the public repository into a local, untracked config that `pm lint forbidden-name` reads; restore the 1299 spec fences and matcher lines from `921f17c75`; clean the inventory metadata; run fresh code reviews on the 1304 landing and on the post-review deltas of 1299, 1302 and 1298, and record them; correct the 1298 design-review line; correct the 1284 records, the lane table and the milestone exit criteria, and record who waived the QA step.
- Proof: `cargo package --list` has no probe; `mustmatch test -v article.md` runs the three 1299 blocks; `git grep` finds no private names in the tree; the reviews are recorded on the tickets.
- Defers: rewriting public history that already carries leaked names. That is Ian's call, recorded in the review file.

## Build status

- Built on branch `tickets/2020-clear-main-before-the-0-9-2-tag`,
  commits 04df5d21d through 4d85dff69, 2026-10-07, across two timeout
  revivals with checkpoints (nothing lost).
- Code review: REJECT 2026-10-07, both findings fixed the same day.
  P1 (the who_pq removal_condition join surviving the item-4 repair)
  fixed in fa51560e0, verified by grep-zero, the bare tool's
  byte-identity, and ratchet exit 0. P2 (two team-sense phrases)
  neutralized in the same commit, plus the sixth-go billing phrases.
- Code re-review (fold delta da0bf7969..fa51560e0): ACCEPT 2026-10-07.
  All three folds verified character-level; one residual P2 phrase in
  the 1287 record reworded by the coordinator in 4d85dff69.

## Yellow failure record (2026-10-07)

The first clean yellow run failed one spec block: the restored
"Deadline Expiry Names Itself" fence (adf71898e). Root cause, proven on
Yellow with the lane's env loaded: the routine article lane loads the
full-text fixture's source env, so PubTator answers fast while Europe
PMC is held past the deadline, and main's federated error pick surfaces
PubTator's error — hiding the deadline error the block pins. The fence
is correct; the code it exercises carries the precedence defect that
ticket 2023 item 2 fixes on its branch. Landing order therefore runs
2023 before 2020; 2020 re-gates after 2023 lands. The block passes solo
without the lane env, which is why earlier checks missed it.

## Second-fix review

- Code re-review (the post-4d85dff69 coverage the second review
  demanded; branch tickets/2020-names-and-records head 185d8e458):
  ACCEPT 2026-10-08. The names are fully out of tracked files with the
  untracked-declaration guard proven by planted-name test; the rebase
  unions carry main's landed history; MAX_PACKAGE_FILES 1_404 measured;
  the lanes table, the 1284 records, and the QA waiver naming Ian's
  dissolution are in; the restored deadline fences pass beside main's
  cursor rewrite.

## Fix2 review fold (2026-10-08)

The Changes line's "that pm lint forbidden-name reads" is amended: the
delivered design has the repo's own zero-coupling check read the
untracked declaration; pm lint stays inert on the absent key by
design. The pm-side change (teach pm to read the local declaration
file) is handed to the sdlc repo by the inbox message filed today
(repos/sdlc/inbox/biomcp/2026-10-08-pm-read-the-local-forbidden-name-declaration.md).
