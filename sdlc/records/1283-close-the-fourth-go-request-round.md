---
ticket: 1283
heads: b5a25a89, a0396200, bc5a7882, e84d83f6, 6a79c465, dda87d73
merges: a2e5a901, 09567779
---

# 1283 — the fourth go-request round

## What changed

Every gh-calling job names its repository (GH_REPO on seven jobs; create-draft's repository discovery was the round's blocker). The gh query in publish-release is fail-closed and pinned. `should-move-latest.sh` exits 3 for a legitimate stay and 2 for a bad tag, and the workflow case treats only 0 and 3 as answers. The raise-review verdict became a whole-value grammar. The changelog's cell-line sections were corrected, ticket 1260's citation removed, the DepMap and Cell Model Passports identifier-only entries added with the README's indirect count set to ten. Records 1276 and 1281 were corrected, ticket 1269's Fix list unwrapped, record 1282 written. The release checklist landed at `sdlc/release-checklist.md` with AGENTS.md pointing at it and runbook step 8 corrected.

## Evidence, read from the API

- The first two branch runs FAILED: 36792516128 (b5a25a89) and 36792547060 (a0396200) both hit the changelog expected-set still carrying ticket 1260, which the same commit removed from the changelog. Fixed at bc5a7882; run 36793985110 passed. The failures are recorded plainly as this round's history.
- Run 36793985110 (bc5a7882): success, all jobs. Run 36796390160 (e84d83f6): success. Run 36798399476 (6a79c465): success, all eight jobs.
- Merge a2e5a901: main run 36800544768 success. Changelog coverage commit dda87d73 on tickets/1283-changelog: run 36802151034 success; merge 09567779: main run 36803863302 success, all eight jobs.
- Code review: ACCEPT 2026-10-01 dispatch 3a612275-a823-46a6-b19f-6e929b88aab5 (Claude vendor, read-only), with two notes folded at e84d83f6 (the ChEMBL detail-section surface; one Releases heading in AGENTS.md). The two commits that landed after that dispatch's evidence snapshot (e84d83f6, dda87d73) were re-reviewed with the fifth round's dispatch — see ticket 1284.
