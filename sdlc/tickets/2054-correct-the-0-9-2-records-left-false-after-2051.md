# 2054 — Clear the 0.9.2 leftovers

Status: OPEN.

Milestone: 0.9.2

## Outcome

pm shows no open ask and no landed ticket's reserved number from this release, and the machines hold no leftover builds, clones or branches from it.

## Evidence

Filed 2026-10-11 from the independent review of main at `b039ac3e7`. Trimmed 2026-10-12 under Ian's ruling that pm records exist to drive work, not to keep history: finished tickets and write-ups stay as they are.

- `pm inbox` shows one open ask: the 2026-10-08 message about moving the message store, ask 1. The change it asks for is done in `622a93eb2`.
- pm daily reports 16 abandoned reservations: `refs/pm/2010`, 2016, 2017, 2019-2024 and 2029-2035, all landed.
- Leftovers on the coordinator's machine: `/tmp/rehearsal-clone`, `/tmp/2050-live`, about 72 old review and lane report files in `/tmp`, and the probe branches `probe/2044-red`, `probe/2050-mutation-get299` and `probe/2050-red-main`. On the build host: the landed worktree `/tmp/2036-green/repo`, the 6.3 GB `~/gate-biomcp` clone, the branches `measure-*`, `yr-branch` and `yr-main`, and about 118 old log and script files in the home folder.

- Starts from: main.
- Keeps: nothing.
- Changes: answer the open ask with `pm answer`; release the 16 reservations, or let pm's own sweep release them if that lands first; delete the leftovers above once nothing needs them.
- Proof: `pm inbox` and `pm daily` show no open ask and no abandoned reservation.
- Defers: nothing.

## Amendment, 2026-10-12

pm now checks ticket and review formats itself, so BioMCP's own copy goes. Delete the review-verdict checks in `tests/test_sdlc_review_status_contract.py` and keep its merge-conflict-marker tests in a file of their own. The mailroom ask is answered, so the first Changes item is done. Proof adds: `pm lint` reports no ticket-format findings, and no test under `tests/` restates pm's review-verdict rules.

## Progress

- Ask answered 2026-10-13: the running-time ask (pm 0300) is answered through pm answer — pm daily on the M5 shows the Running section honestly (no biomcp ticket running; one pre-0300 foreign row; the 1.0 branch rename command listed).
- Reservations: the 16 abandoned refs released 2026-10-12 through the sweep's own release commands.
- Coordinator machine: /tmp/rehearsal-clone, /tmp/2050-live, the review and lane report files, and the local probe branches deleted; other repos' scratch files left to their owners.
- Build host: /tmp/2036-green removed, the 6.3 GB ~/gate-biomcp clone removed, measure-* and yr-branch/yr-main cleared (the shared checkout returned to main under the lock), old home log files removed; the study clone left alone as the PM asked. Three setup scripts in the home folder (machine setup, not release leftovers) stay.
- Review-verdict checks: tests/test_sdlc_review_status_contract.py deleted; its merge-conflict-marker tests moved to tests/test_merge_conflict_markers.py. pm lint at this head reports 285 layout findings, all on files older than this release (records whose tickets predate the archive split; legacy ticket sections); none are review-verdict findings.
