# 1281 — carry the kids26 branch's still-true work, then retire it

Filed 2026-09-30 under Ian's branch direction (second go-request review): `tickets/kids26-cell-line-dataset` holds four records for cell-line work whose code merged to main the day after v0.9.0 was tagged (commits 37336c22, eed4f2c1, fd6a100c, bccd2871 — none in v0.9.0), so 0.9.1 is that work's first release and the records were missing from main (corrected 2026-09-30 third go-request review; the earlier 'backfill' marker wrongly treated the work as already released) and a ChEMBL licensing row with citation URLs main lacks. This ticket carries the still-true parts and deletes the branch.

## What is carried

- The four branch-only records: 1202 (cell-line entity), 1205 (PharmacoDB drug-response sections), 1213 (HPA expression), 1214 (ChEMBL cell-line section). Each describes landed, gated work; main had no record for it.
- ChEMBL's licensing citations into `docs/reference/sources.json` and the licensing page: the gitbook about page and the release LICENSE file that both carry the CC BY-SA 3.0 grant, and the history that the shipped attribution line has said CC BY-SA 3.0 since ticket 1214. Main's licence summary already named CC BY-SA 3.0 (2026-09-27 pass); what was missing is the citations and the ShareAlike-aware reuse summary.

## What is NOT carried (and why)

- The branch's sources.json otherwise predates the 2026-09-27 licensing pass (its rows say 2026-03-20); main's data is newer and more accurate, including Enrichr's restricted relabel and the CPIC and PharmGKB corrections.
- The 1206 DepMap draft hold: superseded by the recorded decision that DepMap stays out of the repository (rehosting restricted bytes in a public MIT repo imposes narrower terms; the 21-month-stale mirror; recorded in the licensing pass).
- The entity-track planning notes and the 0.9.1-1.0 backlog edits: superseded by the planning files on main.

After this lands, the branch is deleted; its remaining diff against main is exactly the superseded material above.

## This round's own work and review

Beyond the kids26 carry, this ticket carried the second go-request round: the latest-tag version decision (scripts/should-move-latest.sh wired into publish-release, pinned by tests and the docker-image spec), the per-path classifier tests, the exact evidence-table guard, the raise-review separator phrases, the ratchet in repository-contracts, the changelog corrections, the records (1279 restored, 1280 added, 1276 filed), the backfill gate marker, and the zero-coupling redaction of the review file's branch names (main was red from the literal name).

- Code review (pass 1): REJECT 2026-09-30 (dispatch 85cf92fc-5419-48d9-80cd-2d3ed6357d11, Claude vendor) — the kids26 carry made the tag-time changelog gate demand bullets for 1202/1205/1213/1214 (the scratch-tag artifact showed it), and the 1257 unwrap had collapsed the numbered lists; folded (fixed): backfill frontmatter plus a gate that reads the committed body, and the lists restored.
- Code review (pass 2): REJECT 2026-09-30 (dispatch 94c43888-e700-4969-a919-c7dc07e8ca79) — the Fix section still ran on (my first repair dropped items), and the CI artifact predated the fixing HEAD; folded (fixed) in c7fbd83a and e759120c, with branch run 36754521995 green at e759120c.
- Code review (final pass): ACCEPT 2026-09-30 (dispatch 598c0b9a-a9df-46f8-ac8d-7c614809d438) — both folds verified, no findings.
