# Review of the second 0.9.1 go request

Filed 2026-09-30 from an independent read-only review of main at `d9ae247d`, range `7d2c6c13..d9ae247d`. Two fresh reviewers covered release readiness and this round's fixes, including the remote branches held for Ian. Reviewers used scratch clones with scratch tags, offline tests, read-only GitHub queries and GitHub's branch activity log.

Ian's direction of 2026-09-30 in `2026-09-30-review-of-the-0.9.1-go-request.md` still holds: fix everything before the tag.

## Verdict

Do not tag `d9ae247d`. The release would leave the GHCR `latest` tag on 0.9.0, and two tickets have dropped out of the records and the changelog gate.

## What holds

- CI run 36723251840 passed all eight jobs on `d9ae247d`.
- A scratch tag passes the version-sync, release-versions and changelog checks. The live docs serve `d9ae247d`.
- container-publish runs before pypi-publish and publish-release. homebrew-tap waits for both. The `container_only` path, secrets and permissions look right.
- The fixes for #250, #283, #284, #286 and #287 exist in the code (`src/sources/ca_bundle.rs:127-168`, `src/cache/private.rs:516-522`, `src/mcp/shell.rs:294`, `src/mcp/shell/typed_get.rs:98`, `Cargo.toml:96`).
- The raise-review gate accepts only a verdict that starts with ACCEPT, and its tests catch a looser rule.
- The docs-only allow list now sends README, `docs/`, `docs/charts/`, `spec/`, `skills/` and `src/**/*.md` to full CI.
- The evidence file renders correctly with one row per line.
- The three deleted branches were safe to delete. Their commits were all on main by patch.
- The final review dispatch `4cbd5801` is genuine and read-only.

## Release blocker

- The GHCR `latest` tag will not move. Commit `7985f3e2` moved "Move latest" ahead of `gh release edit --draft=false` (`.github/workflows/release.yml:709-726`). The step reads `gh release view` with no tag (line 715). That call returns the latest published release and skips drafts. While v0.9.1 is a draft, it returns v0.9.0 (confirmed today), so the step logs "skip" and exits 0. The comment at lines 697-700 says the API sees drafts, which is false. The changelog promises the move. Compare the tag against the highest published version by semver instead, and add a test that a draft tag newer than every published release moves `latest`.

## Records and the changelog gate

- Commit `7985f3e2` deleted `sdlc/records/1279-the-second-review-round.md` when it added the ticket file. Ticket 1280 has no record and no changelog entry. Neither ticket landed through a merge commit, and the gate finds tickets from merge subjects or added records. The gate now sees neither, which is why its count fell from 53 to 52. Coverage passes only because the changelog happens to name 1279. Ticket 1280 says "Ticket 1279's file exists with its record", which is false.
- Land each ticket with a merge commit (`git merge --no-ff`) so the gate sees it and main's first-parent history holds only green commits. Six commits on main failed CI in this range: `7985f3e2`, `7937d35b`, `85debb5f`, `9b5446da`, `8864ed3a` and `b499b63b`. They came in by fast-forward of a branch.
- Record 1265 line 11 still says "stays under 2000".
- Record 1268 still names `trust_failure_tests.rs` beside the `mod.rs` path.
- Ticket 1276 has no ticket file. Its record's ACCEPT is prose with no reviewer dispatch.
- The CI-rework disposition (item 2) says `docs/` is pinned by contract and behavioral tests. The tests do not pin it (see below).

## Tests that pass with the fix removed

- Re-admitting `docs/*`, `docs/charts/*` or `skills/*` to the docs-only list passes all seven classifier tests. `test_readme_and_docs_pages_run_full_ci` commits README first, and the diff against the merge-base keeps README in every later case. Test each path in a fresh branch.
- The evidence-table guard requires at least 49 lines starting with a pipe, and the table has 50 with its header. Collapsing one row passes. Set the threshold to the exact count or compare rows to `sources.json`.
- The raise-review gate accepts "ACCEPT: expected after fixes" and "ACCEPT, pending fixes". Reject any ACCEPT value followed by a promise or pending word, and test both.
- `tools/check-test-wait-ratchet.py` reads `sdlc/tickets` and runs in `make lint`, which an sdlc-only branch push skips. Run it in repository-contracts.

## Changelog

- The #282 entry (`CHANGELOG.md:12`) says "large queries" crashed "adverse-event paths". The issue shows simple queries crashing in trial search, drug trials, drug interactions and `drug adverse-events <unknown>`. The cause was the debug-profile wheel.
- The #250 entry leaves out `BIOMCP_CA_BUNDLE`, the setting users need.
- Lines 37-38 file user-facing items (published schemas, structured argument errors) under Internal. Line 21 repeats line 12. Line 35 uses a "never by…" clause.

## Other

- Ticket 1257 line 15 says "review's P2" and is still hard-wrapped. Ticket 1269's title says "round-three". Ticket 1264's file name says "round-two". About 72 older sdlc files still carry shorthand. Fix the files this round touched. The older files can wait for a 1.0 records sweep.
- `.review-artifacts/` was committed in `7937d35b` and removed in `016ee8f3`. Add it to `.gitignore`.
- The docs-live helper is checked out from main (`release.yml:427`), so a later edit to it changes the gate for an older tag.

## Remote branches

| Branch | State | Holds |
|---|---|---|
| `planning/1146-refresh`, `planning/1149-refresh`, `planning/1153-refresh` | superseded on main | ticket refresh; the ticket has a record on main |
| `ticket/1147-implementation`, `ticket/1147-integration` | superseded on main | article batch commands; record 1147 on main |
| `ticket/1151-implementation` | superseded on main | drug command discovery; record on main |
| `ticket/1158-implementation` | superseded on main | GenCC fixes; record on main |
| `ticket/1178-implementation` | superseded on main | eligibility work; record on main |
| `ticket/1164-implementation` | superseded on main | hybrid ranking; `f4450398` does the same |
| `ticket/1142-design-refresh`, `ticket/1164-design-refresh` | unlanded | design text only; no record |
| `planning/1143-refresh`, `planning/1145-refresh`, `planning/1163-refresh` | unlanded | design refresh; the ticket on main differs; no record |
| `tickets/kids26-cell-line-dataset` | unlanded | records 1202, 1205, 1213 and 1214, the 1206 draft hold, licensing and ChEMBL row edits |
| the 1.0 sync-candidate scratch branch | unlanded | merge of 0.9 main into 1.0; owned by the 1.0 team |

## Branch decision

Ian directed on 2026-09-30 that leftover branches be removed. The reviewer applies that direction as follows, and Ian can overturn it.

- Delete the superseded branches and the design-only branches.
- For `tickets/kids26-cell-line-dataset`, compare its licensing and ChEMBL row edits with main. Carry any that are still true and missing into a reviewed ticket, then delete the branch.
- Leave the 1.0 team's scratch and feature branches to the 1.0 team (their names redacted here; the zero-coupling gate forbids naming that project in this repository).
