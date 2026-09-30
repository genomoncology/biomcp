# 1282 — close the third go-request round

Filed 2026-09-30 with its record; the round closes `sdlc/issues/2026-09-30-review-of-the-third-0.9.1-go-request.md`.

## What landed

- The release blocker: publish-release checks out the repository before any step runs repository code (pinned actions/checkout, first step, pinned by the provenance list and a new guard that fails any job running scripts/ or tools/ paths without a preceding checkout); a failed `gh release list` now exits the job instead of reading as an empty list; the latest decision stays the numeric version comparison with a new v0.10.0-vs-v0.9.0 test.
- The cell-line blocker: the backfill markers and their gate mechanism are gone — the four records (1202, 1205, 1213, 1214) document work whose commits (37336c22, eed4f2c1, fd6a100c, bccd2871) are not in v0.9.0, and 0.9.1 is that work's first release; the changelog documents the cell-line entity as a new entity type with its PharmacoDB, HPA and ChEMBL sections, per Ian's 2026-09-30 confirmation (public research data is in scope; local processing of primary data files is not).
- The raise-review gate honors the acceptance token only when followed by end-of-value, a date, or a reviewer/dispatch reference; the four promise phrasings named by the review are tested rejections.
- The evidence guard now actually compares squashed row names to sources.json ids.
- Records: 1281's record written; 1276 states plainly it had no dedicated dispatch at landing and names the later verifying reviews; 1280's line corrected; ticket 1281's "landed long ago" corrected; 1269 and record 1264 unwrapped; no "P1" token in the classifier tests; the repeated schemas clause removed from the changelog.

## Evidence

Branch head 5823a106, green branch run 36768467803 (all eight jobs). The scratch-tag gate covers all 59 merged tickets.

## Review

- Code review: ACCEPT 2026-09-30 (dispatch d008965b-fdfe-4319-a140-afefb4a89df2) — a fresh read-only reviewer on the Claude vendor read the workflow, the script and the gate changes; no findings.
