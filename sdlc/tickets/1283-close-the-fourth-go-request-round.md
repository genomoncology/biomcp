# 1283 — close the fourth go-request round

Filed 2026-09-30 with its record; the round closes `sdlc/issues/2026-09-30-review-of-the-fourth-0.9.1-go-request.md` and works `sdlc/release-checklist.md` for 0.9.1.

## What landed

- The blocker: every job that calls gh carries `GH_REPO: ${{ github.repository }}` (seven jobs), so create-draft's `gh release create` knows its repository without a git directory; a new provenance guard fails any job calling gh with neither GH_REPO nor a checkout, and a test pins the fail-closed gh query (restoring `|| true` inside the guarded window fails).
- `should-move-latest.sh` now exits 3 for a legitimate stay and 2 for a bad tag; the workflow treats only 0 and 3 as answers and fails the step on anything else; the tests cover both codes.
- The raise-review gate matches each verdict value against one whole-value grammar: the acceptance token, an optional date, an optional dispatch reference, nothing more. "ACCEPT 2026-09-30 but pending fixes" and "ACCEPT dispatch abc will fix" are tested rejections. Tickets 1264 and 1269's verdict lines were reshaped to the grammar; nine older verdict lines (tickets 1164, 1242, 1243, 1246, 1250, 1254, 1255, 1265, 1266) no longer parse under it — no raise cites them, so they stand as history (noted here per the review).
- `test_wheel_installs_skills_without_network` confirmed: its module carries `pytest.mark.needs_binary` and reads `BIOMCP_BIN`; the scratch-clone failure is the designed needs-built-wheel behavior, not a break.
- Changelog: the cell-line sections corrected (`gene cell-lines <symbol> --group` reports RNA nTPM from the Human Protein Atlas; `get cell-line <id> chembl` lists ChEMBL records with assay counts), the internal confirmation note removed, ticket 1260's citation removed (it never landed).
- Sources: ChEMBL's `get cell-line <id> chembl` surface recorded; DepMap and Cell Model Passports added as identifier-only entries with their terms stated (the 2026-09-27 DepMap decision stands: restricted bytes never rehosted); the README's indirect count now says ten.
- Records: ticket 1281's about-page claim corrected (the URL landed in notes, not the terms field), record 1276 aligned with its ticket, ticket 1269's Fix list unwrapped to one line per item, record 1282 written.
- The release checklist landed at `sdlc/release-checklist.md` (Ian's draft, corrected against the fixed workflow: GH_REPO requirement, latest moving before the public flip, the stay exit code), AGENTS.md's Releases section points at it, and the runbook's step 8 corrected.

## Evidence

Branch green runs recorded in the go request; the worked checklist is section 5 of `sdlc/release-checklist.md`'s source review file.

## Review

- Code review: ACCEPT-with-notes 2026-10-01 (dispatch 3a612275-a823-46a6-b19f-6e929b88aab5, Claude vendor) — the reviewer walked every job of the tag-push path as if the tag were pushed: create-draft's repository discovery fixed and extended to every gh-calling job, no step-level GH_REPO overrides, the fail-closed query and the stay exit code verified against the workflow case, the whole-value verdict grammar traced by hand with the nine older verdicts confirmed non-load-bearing, the changelog and sources corrections verified, the checklist landing and runbook step 8 verified against the fixed workflow. Two notes folded at e84d83f6: the ChEMBL detail section now lists the cell-line surface (the two registries agree), and AGENTS.md carries one Releases section with the checklist pointer merged in.
