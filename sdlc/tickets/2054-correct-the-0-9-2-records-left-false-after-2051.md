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
