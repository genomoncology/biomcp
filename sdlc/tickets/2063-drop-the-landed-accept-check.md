# 2063 — Drop the landed-accept check

Status: OPEN.

Milestone: 0.9.3

Reviews: revision 071d620, accept

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

## Progress

- 2026-10-13 started
- 2026-10-13 next: pm rebuilt from f6b5096, which holds 0325; the re-probe on a scratch clone under /tmp refused every bad verdict under both pm ticket land --merge-branch and pm land-check, each exit 1 naming the review verdict — the pending bullet, the pm-form pending, the pm-form findings, the glued, the bold and the wrapped accept spellings, and no review lines at all — while a genuine accept landed: its merge carried Review: accept, the verdict pm read, with Status: COMPLETE. inside it; the check is deleted
