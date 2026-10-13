# 2054 — Clear the 0.9.2 leftovers

Status: COMPLETE.

Milestone: 0.9.2

Reviews: revision 392f504, accept

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

## Amendment, 2026-10-13

The PM ruled on the review's blocker, option b: "pm's merge-branch flow lacks a verdict gate (bug filed as pm 0325), so BioMCP keeps ONE minimal check until pm 0325 lands and a re-probe passes: every landed ticket carries an accept." That check is `tests/test_landed_tickets_carry_accept.py`. A landed ticket, its number the leading number of a file in `sdlc/records/`, passes on pm's own `Reviews: revision <hex>, accept` line or a review line whose state token is ACCEPT; glued, bold and wrapped shapes stay pm's alone. The ruled shapes alone fail the tickets the 1259 records round left with honest "no record kept" verdicts, so the check also accepts a closed list of those exact disposition strings, enumerated in the module; a new spelling fails rather than matching a pattern. Delete the whole check when pm 0325 lands and the re-probe passes.

## Progress

- 2026-10-13 next: the accept check is back under the PM's ruling and pm 0325 is filed; delete it when pm 0325 lands and the re-probe passes
