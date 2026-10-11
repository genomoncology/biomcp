# 2054 — Correct the 0.9.2 records left false after 2051

Status: OPEN.

Milestone: 0.9.2

## Outcome

The 0.9.2 records say only what is true, and pm shows no open ask or stale reservation from this release. This ticket keeps to what is false or broken and adds no new process.

## Evidence

Filed 2026-10-11 from the independent review of main at `b039ac3e7`.

- `pm inbox` still shows one open ask: the 2026-10-08 message about moving the message store, ask 1, with an empty answer. The change it asks for is done in `622a93eb2`. `sdlc/records/2051-records-build.md:3-4` says no asks are open.
- The earlier answers were patched by hand in the message files (commit abc4e90 there) and left orphaned wrapped lines. The debug-info answer now says the work is "not wanted before the tag", cites 2050, then repeats "queued behind the 0.9.2 tag". 2051 asked for a ticket or a plain "not wanted".
- `55b059ce9` joined ticket 2038's numbered findings onto single lines (2038:29 holds findings 1 to 7, and :39, :43, :56 and :60 hold the rest) and left wraps at 2038:82-121 and 2045:13-26.
- 2038:93-94 and 2038:106-121 keep the hand-normalized verdict bullets 2051 named for deletion.
- 2039 and 2040 still name branch heads `ea5b22f5f` and `b6891d0f5`; no verdict was recorded on the landed tree.
- `sdlc/records/2048-pm-contract-build.md:3-11` and `records/2051:3-9` claim the items above are done. They and `records/2049` and `records/2050` are hard-wrapped.
- b039ac3e7 added the "Final rehearsal" section to 2049 with a script. It is dated 2026-10-12, says "The queue emptied", cites "ci.yml run 38174800939-equivalent", which does not exist (the real run is 38172604914), and gives 866 tests, which is the contracts slice, not the offline suite. `2049:50-53` describes a docs-live stub in the workflow comments; none exists.
- Ticket text uses names a public reader cannot know: `yr` at 2036:42, 2040:29, 2043:49-51 and 2050:45, and the private message-store names in `records/2048` and `records/2051`.
- 2050:101 gives "main: about 17 s" for 5-FU and Ara-C, where 2050:23 measured 4 and 6 s.
- pm daily reports 16 abandoned reservations: `refs/pm/2010`, 2016, 2017, 2019-2024 and 2029-2035, all landed.
- Leftovers: on the coordinator's machine, `/tmp/rehearsal-clone` with two superseded unpushed commits, `/tmp/2050-live`, about 72 old review and lane report files in `/tmp`, and unmerged probe branches `probe/2044-red`, `probe/2050-mutation-get299` and `probe/2050-red-main`. On the build host, the landed worktree `/tmp/2036-green/repo`, the 6.3 GB `~/gate-biomcp` clone, the branches `measure-*`, `yr-branch` and `yr-main`, and about 118 old log and script files in the home folder.

- Starts from: ticket 2051 and its landing `38971287b`.
- Keeps: the pm `Reviews:` line and the contract test that reads it.
- Changes: answer the open ask and rewrite the garbled answers once through `pm answer`, giving the debug-info ask a ticket or a plain "not wanted"; restore 2038's numbered list from `1e3cf7cdb` and join the remaining wraps by hand; delete the normalized verdict sections and the 2049 "Final rehearsal" section and docs-live claim; record verdicts on the landed trees of 2039 and 2040; rewrite the 2048 to 2051 records as single true lines; say "the build host" for `yr`; correct the 2050 timing; delete the 16 landed reservations and the leftovers above.
- Proof: `pm inbox` and `pm daily` show no open ask and no abandoned reservation; the contract test passes; a reader of 2038 sees one finding per item.
- Defers: the "Build status" sections in tickets 2016 to 2034, which predate this review round.
