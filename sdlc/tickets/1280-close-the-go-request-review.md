# 1280 — close the go-request review findings

Ian's 2026-09-30 direction (recorded in `sdlc/issues/2026-09-30-review-of-the-0.9.1-go-request.md`): fix every finding before the tag, including the follow-ups; no rush.

## What landed

- The wait-ratchet's raise gate now honors only verdict values that begin with the acceptance token; every promise phrase that merely mentions it stays a rejection, each is pinned by a test, and ticket 1269's acceptance line was reworded so its value begins with the acceptance token.
- The docs-only allow list shrank to sdlc/, notes/, CHANGELOG.md, AGENTS.md and .github/*.md: README.md and every docs/ page run the full suite, because docs/charts is compiled into the binary and the CLI-structure benchmark reads README and all docs pages; behavioral tests cover README and docs pages, and the duplicated dead case arm the review noted is gone.
- publish-release now runs all its Docker work (buildx, login, the latest-tag move) before `gh release edit --draft=false`, so a failed Docker step leaves a deletable draft; the provenance pin matches the new order.
- The changelog gate's stale cap comment and constant are gone.
- Records: 1265's cap text corrected with an honest no-verdict statement, 1267/1268 name the real green chain runs and state plainly that their branch-head runs expired, 1268's plain-send text says fixed with the right test path, 1276 carries a real verdict. Ticket 1279's file exists with its record; the CI-rework review file has a disposition covering every finding.
- Shorthand cleared (tickets 1253, 1257, 1261, 1264, 1268, 1278 and records 1261); the evidence file has a real heading, its summary paragraph, one row per table line and an unwrapped closing paragraph, guarded by a new test after the first repair collapsed the table.
- The 0.9.1 changelog was rewritten for users: house subsections, the GitHub issues named (#282, #283, #284, #286, #287, #250), internal round-shorthand gone, housekeeping folded into one maintenance line, and the gate green over all merged tickets.

## Evidence

Branch head 668ff768, green CI run 36713820487 (all eight jobs). Intermediate commits 7985f3e2 through 8864ed3a failed their runs (ruff findings and the evidence-table repair cycle); each was fixed on the branch.

## Review

- Code review (pass 1): REJECT 2026-09-30 (dispatch 662bf5b5-3427-497c-a86b-75113e07b93a, Claude vendor, read-only) — the changelog misattributed ticket 1261 and omitted GitHub #286; folded (fixed), plus the noted dead case arm and shorthand.
- Code review (pass 2): REJECT 2026-09-30 (dispatch dbdb96ec-df51-4d6d-bb6a-18c62de080e0) — the evidence file's table had collapsed to one line with the summary paragraph lost (a corruption my own heading repair introduced), and the supplied CI artifact predated the green run; folded (fixed: the table restored row-per-line with a guard test, and run 36713820487 finished green on 668ff768).
- Code review (final pass): ACCEPT 2026-09-30 (dispatch 4cbd5801-a204-4a91-a99a-79da506f4dca) — every fold verified from the live files and the regenerated diff artifacts; no findings.
