# Review of the fifth 0.9.1 go request

Filed 2026-09-30 from an independent read-only review of main at `09567779`, range `87a6a697..09567779`. Two fresh reviewers covered the whole release workflow and this round's fixes and records. One reviewer read every line of `.github/workflows/release.yml` and walked each job as if the v0.9.1 tag were pushed. Reviewers used scratch clones with scratch tags, offline tests, actionlint, a fake `gh` for the `latest` step, and read-only GitHub queries.

## Verdict

Do not tag `09567779`. The container-publish job cannot download from the draft release, so the release would stop before PyPI.

## What holds

- `09567779` is a merge commit. Main runs 36803863302 (CI, eight jobs) and 36803863237 (documentation) passed every job. Branch runs 36798399476 and 36802151034 and main run 36800544768 are green.
- A scratch tag passes version sync, release versions and the changelog gate, which covers 61 tickets.
- The live docs serve `09567779`. No v0.9.1 tag, draft or release exists on GitHub.
- Every gh-calling job names its repository or checks out the code. create-draft has GH_REPO and `contents: write`. publish-release checks out before it runs `scripts/should-move-latest.sh`, which is committed as executable.
- The `latest` step behaves correctly under a fake `gh`. A `gh` failure fails the step. A missing script fails the step. A newer published release makes the script exit 3, and the step leaves `latest` alone. Otherwise `latest` moves.
- Mutations fail the tests: removing GH_REPO from create-draft, removing publish-release's checkout, restoring `|| true` on the `gh` query, and making the script exit 0 for a stay.
- actionlint 1.7.7 reports no findings on `release.yml`.
- Every command the changelog names exists with that spelling. The 0.9.1 section names #250, #282, #283, #284, #286 and #287, and the internal note and the 1260 citation are gone.
- `sdlc/release-checklist.md` matches the draft in substance, `AGENTS.md` points to it, and runbook step 8 matches the workflow.
- The offline suite passes 822 tests. The zero-coupling check passes.
- The review dispatch `3a612275` is genuine and read-only.

## Release blocker

1. container-publish declares `contents: read` (`.github/workflows/release.yml:598-600`). Its "Stage the released Linux executables" step runs `gh release download "$TAG"` (`:635-638`). On a tag push the release is still a draft until publish-release runs `gh release edit --draft=false` (`:784`). GitHub lists draft releases only to callers with push access, which for `GITHUB_TOKEN` means `contents: write`. With `contents: read` the download reports that the release is not found. container-publish fails, so pypi-publish, publish-release and homebrew-tap are skipped, and the draft and tag are left behind. v0.9.0 never ran this path, because it ran on the `release` event with `contents: write` for the whole workflow. Give container-publish `contents: write`, and add a test that every job that reads the draft release has `contents: write`.

## Gate and test holes

- The raise-review grammar still accepts promises (`tools/check-test-wait-ratchet.py:313`). The `by [^;]+` and `\(dispatch [^)]+\)` parts take any text. "ACCEPT by Sol, but will fix later", "ACCEPT 2026-09-30 by reviewer pending fixes", "ACCEPT (dispatch TBD after fixes)", "ACCEPT dispatch -" and "ACCEPT2026-09-30" all pass. The comment says "nothing more", which is false. Limit the reference to a dispatch ID (hex) or a reviewer name of one or two words, and test these five values.
- Tickets 1264 and 1269 were made to pass by writing text that is not a dispatch ID: "(dispatch via ticket 1269's review)" and "(dispatch folded-and-rereviewed-2026-09-28)". Cite the real dispatch, or mark them as history in a way the check reads.
- No test pins the workflow's exit-code case. Widening `3)` to `3|*)` (`release.yml:772`) passes every test.
- The gh guard (`tests/test_release_workflow_provenance.py:1400`) finds `gh` only at the start of a line. It misses `if ! gh …` and `$(gh …)`. It also checks only that the repository is named, not that the token can see a draft, which is how the blocker passed.

## Records

- Record 1283 does not exist. Ticket 1283 says it was filed with its record. The same slip happened with record 1282 last round. Ticket 1283's evidence names no run IDs and points to a section that does not exist.
- The fourth review file has no disposition lines, which checklist item 1.1 requires. Each finding maps to a fix in `b5a25a89` or `e84d83f6`.
- Commits `e84d83f6` and `dda87d73` landed after the reviewer's verdict with no review of their own, against checklist item 1.3. The reviewer read a prepared bundle and ran no tests. It returned "OK with notes", which the ticket records as "ACCEPT-with-notes".
- `sdlc/release-checklist.md:52` says the checklist was worked for 0.9.1. No worked copy with evidence exists in a ticket.

## Changelog and sources

- `CHANGELOG.md:21` calls HPA nTPM "RNA nTPM protein expression". It is RNA expression, as the CLI help says (`src/cli/gene/mod.rs:173`). "Bookkeeping pushes still run the record tests" is internal phrasing.
- The DepMap terms say "most datasets" are CC BY 4.0 and restricted data needs "a signed agreement". DepMap's own forum says its generated data are "generally" CC BY 4.0 and other hosted projects may differ. The Cell Model Passports terms could not be confirmed. Both `terms_url` values point to landing pages, not terms pages. The licensing evidence file has no row for either source. Link the actual terms pages, state only what they say, and add evidence rows.
- `sources.json` notes carry internal phrases ("fourth go-request review", "Ian's rule") into a public reference file.
- The `sources.json` diff mixes moves (Cellosaurus, PharmacoDB, the ChEMBL surfaces) with about 45 lines of new content. Keep moves out of content commits.

## Lesser release notes

- If `gh release edit --draft=false` fails after `latest` has moved, `latest` points at an image whose release is still a draft.
- homebrew-tap exits 0 when `HOMEBREW_TAP_TOKEN` is not set. That skip is by design. The checklist's Homebrew check in section 4 catches it.
- `gh release list` returns at most 30 releases by default (`release.yml:756`). This has no effect today.

## Dispositions (recorded 2026-10-01, fifth round)

- Blocker (container-publish contents): fixed — `contents: write` with the comment naming this review, and `test_jobs_downloading_the_draft_release_can_write_contents` fails any job that downloads the release before publish-release without write access (mutation-verified: reverting to read fails the test).
- Rehearsal: done — run 36896962624 on genomoncology/biomcp-release-rehearsal (public scratch copy, Ian's 2026-10-01 redirect: a credential-free distribution check instead of TestPyPI, docs-live stubbed, both listed as stated differences; Ian approved the public flip after the private copy stopped for lack of private-repository runners). Attempt 3 completed with all jobs green. The rehearsal caught two release blockers this review had not found: ticket 1285 (the DDInter cold-cache smoke gap) and ticket 1286 (macos-latest moved to arm64); both landed with their own reviews. The draft download, both image smokes, the latest move, the public flip and the Homebrew skip all ran and passed; evidence is in ticket 1284 and the go request.
- Raise-review grammar: fixed — the reference is now a hex dispatch ID or a one/two-word capitalized reviewer name; the five values from this review are tested rejections; tickets 1264 and 1269 cite no dispatch (none exists) and say so plainly on their own lines.
- Exit-code case pin and the mid-line gh guard: fixed — `test_the_decision_case_accepts_exactly_zero_three_and_catchall` (widening `3)` to `3|*)` fails) and the guard now matches gh anywhere in a run line.
- Record 1283: written with its run IDs, including both failing runs.
- Fourth review file dispositions: added (above).
- e84d83f6 and dda87d73: re-reviewed with this round's dispatch, named in ticket 1284.
- Worked checklist: ticket 1284 carries the worked copy for 0.9.1 with evidence.
- Changelog "RNA nTPM protein expression" and "bookkeeping pushes": reworded to the CLI help's own terms.
- DepMap and Cell Model Passports: terms_url points at the providers' own terms/guidance pages, the summaries state only what those pages say, evidence rows added to the 2026-09-27 file (52 rows), and "fourth go-request review"/"Ian's rule" phrasings removed from sources.json.
- sources.json move-mixing: the sort landed inside a content commit and stays as history; the convention going forward (moves out of content commits) is recorded in ticket 1284.
- Lesser notes: (a) a failed `gh release edit` after `latest` moved is an accepted residual risk — the checklist's post-release verification catches it, and moving the publish first would reintroduce the public-before-Docker failure this round removed; (b) the Homebrew skip is by design and checklist section 4 catches it; (c) the 30-release page limit has no effect today (the repository has six releases) — noted, no change.
