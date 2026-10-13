# 2063 — Drop the landed-accept check

Status: OPEN.

Milestone: 0.9.3

## Outcome

BioMCP keeps no test of its own that restates pm's review-verdict rules. pm's landing refuses any ticket without an accept of the landed revision, so the repository's interim check goes.

## Evidence

- Ticket 2054 kept `tests/test_landed_tickets_carry_accept.py` under the PM's option-b ruling, until pm 0325 landed and a re-probe passed.
- pm reports 0325 landed on 2026-10-13: `pm ticket land --merge-branch` and `pm land-check` now refuse a ticket without an accept of the landed revision, and the merge trailer states the verdict pm read. Its ask: re-run the 2054 probe, then drop the check and answer with the probe result.

- Starts from: 2054's landing `52fca21` and its probe in `sdlc/records/2054-leftovers-build.md`.
- Keeps: the merge-conflict-marker tests 2054 moved to their own file.
- Changes: pull and rebuild pm; re-run the 2054 probe on a scratch clone (pending-only, findings, glued, bold and wrapped verdicts against `--merge-branch` and `land-check`); delete `tests/test_landed_tickets_carry_accept.py` and its closed list of disposition strings.
- Proof: the probe's results recorded on this ticket, every bad verdict refused by `--merge-branch` and `land-check`; no test under `tests/` restates pm's review-verdict rules; `pm lint` shows no ticket-format finding. Answer pm's ask in `inbox/biomcp/2026-10-13-pm-s-review-gate-is-fixed-re-run-your-2054-probe.md` with the probe result.
- Defers: nothing.
