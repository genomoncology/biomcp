# 1284 — close the fifth go-request round

Filed 2026-10-01 with `sdlc/records/1284-close-the-fifth-go-request-round.md`; closes `sdlc/issues/2026-09-30-review-of-the-fifth-0.9.1-go-request.md`, whose dispositions are appended to it.

## What landed

- The blocker: container-publish carries `contents: write` (drafts are visible only to write-capable tokens), and `test_jobs_downloading_the_draft_release_can_write_contents` fails any job that downloads the release before publish-release without write access. Mutation-verified: reverting to `contents: read` fails the test.
- The exit-code case is pinned (`test_the_decision_case_accepts_exactly_zero_three_and_catchall`; widening `3)` to `3|*)` fails), and the gh guard finds gh mid-line (`if ! gh`, `$(gh …)`), verified on samples.
- The raise-review grammar's reference is now a hex dispatch ID or a one/two-word capitalized reviewer name; the review's five values plus the two fake dispatch-shaped references are tested rejections. Tickets 1264 and 1269 cite no dispatch because none exists, and say so plainly.
- Record 1283 written with all eight run IDs, both failures included. Dispositions appended to the fourth and fifth review files.
- The changelog says "RNA levels (nTPM) for the gene across one cancer group of cell lines" (the CLI help's own words) and drops the internal phrasing ("bookkeeping" is gone; the sentence now says a push that skips the Rust jobs still runs the record tests).
- DepMap and Cell Model Passports: terms_url points at DepMap's own licensing guidance thread and the Sanger DepMap Data Usage Policy; the summaries state only what those pages say; two evidence rows added to the 2026-09-27 evidence file (52 rows pinned); "fourth go-request review" and "Ian's rule" removed from sources.json.
- The rehearsal item landed in checklist section 1; the rehearsal runs on the public scratch repository `genomoncology/biomcp-release-rehearsal` — a private copy stops mid-flight because private-repository runners are not available to this account — with the publish step as a credential-free `twine check` (no TestPyPI registration exists and none is planned).

## Convention recorded

sources.json entry moves (re-sorts) stay out of content commits; the 1283-round sort that mixed them is accepted history.

## Worked release checklist for 0.9.1 (section 1)

- [x] Dispositions: every finding in the fourth and fifth review files carries a disposition (this round's commits; the lesser notes are accepted with reasons).
- [x] Ticket/record/review: tickets 1219-1284 have ticket files and records; review verdicts with dispatch references are in the tickets (1264 and 1269 honestly cite none); record 1283 names its runs.
- [x] Fresh read-only review through the final tip: dispatch recorded below; commits e84d83f6 and dda87d73 (post-verdict commits from the last round) are in its evidence bundle explicitly.
- [x] Claimed fixes fail when reverted: container-publish permissions, the case arms, and the grammar rejections were mutation-verified (commands and outputs in the review artifacts).
- [x] New sources' terms stated: DepMap and Cell Model Passports carry terms in both registries with evidence rows.
- [x] Changelog: 61 tickets covered on a scratch tag (`check-changelog-coverage.py --tag v0.9.1`); the six issues named.
- [x] Version files: `check-version-sync.sh` and `check-release-versions.py --tag v0.9.1` pass.
- [x] Branches: this round merges `--no-ff` after its green run; merged branches deleted; worktrees removed; 1.0 team branches untouched.
- [x] Candidate main run green: recorded in the go request (eight jobs).
- [x] Live docs serve the candidate commit: recorded in the go request.
- [x] GH_REPO/checkout guard and draft-write guard: pinned by tests (above).
- [x] Rehearsal: run 36896962624 on genomoncology/biomcp-release-rehearsal (public scratch copy; attempt 3) completed with every job green, after the private attempt stopped mid-flight because private-repository runners are not available to this account and Ian approved the public flip (2026-10-01). The rehearsal found and fixed two release blockers, both landed, reviewed and merged: ticket 1285 (wheel-smoke queried DDInter on a cold cache, which errors by design — the smoke now syncs first) and ticket 1286 (GitHub moved macos-latest to arm64 mid-day; the x86_64 leg pins macos-15-intel). Proof, read from the attempt-3 logs: the draft download (gh release download on the still-draft release with contents: write, both tarball checksums verified), both image smokes (linux/amd64 and linux/arm64 each ran as uid 65532 with the revision label matching), the latest move (buildx imagetools create tagged :latest from :0.9.1, inspect confirms the manifest), the public flip (gh release edit --draft=false; the API then answered draft:false), the Homebrew skip (no HOMEBREW_TAP_TOKEN in the scratch repo, exit 0), and the credential-free distribution check (twine check: every wheel PASSED). Scratch cleanup: the release and tag are deleted; the test image remains until Ian deletes it (the token lacks delete:packages); the repository is kept for future rehearsals per the checklist.

## Review

- Code review: ACCEPT 2026-10-01 dispatch 16d922f8-6baa-4a4a-be90-959000d4580b (Claude vendor, read-only) — the blocker fix, the case pin, the mid-line gh guard, the grammar's five rejections, the records and dispositions, the changelog wording and the sources terms all verified; one second-priority finding folded at this commit (the word "bookkeeping" still in the changelog line; now reworded away), and one note that the scratch repository's commit graph is external to the clone — the go request carries the run URL instead.
