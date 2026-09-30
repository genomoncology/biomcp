# Review of the 0.9.1 go request

Filed 2026-09-30 from an independent read-only review of main at `7d2c6c13`, range `857a2ba4..7d2c6c13`. Two fresh reviewers covered release readiness and re-ran a mutation for every fix claimed in the go request. Reviewers used scratch clones with scratch tags, offline fixture tests and read-only GitHub queries.

## Ian's direction, 2026-09-30

- Fix every finding in this file before tagging 0.9.1, including the ones listed as follow-ups. There is no rush.
- Bring the changelog up to date and describe what changed for users.
- Merge everything, then remove leftover branches and anything else left over.
- The tag must be clean, the version numbers must check out, and the release must publish the container image.
- The open GitHub issues fixed in 0.9.1 must be solved in the published release. The issues are #250, #282, #283, #284, #286 and #287.

## Verdict

The release is ready. No finding blocks tagging `v0.9.1` on `7d2c6c13`. The product fixes are proven. The findings below are record text, one check's coverage, and wording in the public changelog.

## What holds

- CI run 36693388098 passed all eight jobs on `7d2c6c13`, and branch run 36689893012 passed on the same commit. Every Rust job ran.
- A scratch tag passed `check-version-sync.sh`, `check-release-versions.py --tag v0.9.1` and the changelog gate, which covers all 53 tickets merged since v0.9.0. `server.json` and `CITATION.cff` say 0.9.1.
- The live docs serve `7d2c6c13`, so the docs-live gate passes.
- The release job order holds: pypi-publish waits for every build, smoke and container job, then publish-release, then homebrew-tap.
- Five changelog bullets checked against the code are true (1223, 1230, 1232, 1263 and 1279, 1268).
- A push to main cannot skip the Rust jobs. The merge-base equals the pushed commit, the diff is empty, and the classifier runs full CI.
- Every claimed fix fails its test when reverted: the trust-failure `handle`, the plain-send stop, the GWAS NoStore extension on the middleware builder, the changelog gate's cap, `--release --locked` per command, the README link and hero count, the Enrichr heading tier, and the search-all notes call site.
- The review dispatch `4e671b58` is genuine and stayed read-only.

## Findings

1. Twelve commits after the review's ACCEPT had no review: `8817ba85` (version bump and changelog), `32cd006c`, `b0e6d58f`, `ce17b9da`, `6d9aac02`, `efdbe562`, `9134cdf5`, `349dd636`, `763d68e9`, `bd605214`, `5e3426c2` and `7d2c6c13`. Their Rust changes are all in `src/sources/trust_failure_tests/mod.rs`. The reviewer read files at `767c7292` without git, so it never saw commit diffs. Record `sdlc/records/1279-the-second-review-round.md:3` says the round covered the changelog and version bump, which landed after the review.
2. The raise-review check is a block list (`tools/check-test-wait-ratchet.py:305`). "reviewer returns ACCEPT or findings", "awaiting ACCEPT", "ACCEPT expected after fixes" and "ACCEPT once fixes land" still count as accepted. Disabling the new branch passes all 18 tests in `tests/test_test_wait_ratchet_check.py`. A raise can still cite any old accepted ticket. Accept only a verdict line whose value starts with ACCEPT, and add tests for these phrases.
3. The public changelog uses internal shorthand: "round-two gaps" (`CHANGELOG.md:65`) and "round-three gaps" (`:77`). About a dozen Bug fixes bullets describe test or record hygiene, not user-facing changes: 1228, 1239, 1248, 1250, 1254, 1259, 1261, 1264, 1269, 1270 and part of 1279. The 1279 bullet is one long line, unlike the rest.
4. Stale record text:
   - Record 1265 lines 11 and 15 say "stays under 2000" and "The cap stays". The cap is gone. Line 20 has no real verdict.
   - Record 1268 line 11 says the plain-send path still retries and is not fixed here. It names `src/sources/trust_failure_tests.rs`, now `trust_failure_tests/mod.rs`.
   - Records 1265, 1267 and 1268 say their run is "not retrievable" and name no green run.
   - Record 1276 line 12 has no real verdict.
   - Ticket 1279 has a record and no ticket file.
   - The CI-rework review file has no disposition section.
5. Shorthand and wraps remain:
   - `sdlc/tickets/1253-release-preparation-checklist.md:17` says "1254 b2".
   - `sdlc/tickets/1278-the-ci-cache-and-the-docs-only-skip.md:28` says "~700/850 s".
   - Ticket 1264's title says "round-two".
   - Ticket 1268 lines 21 and 23 use "P0" and "P1".
   - `docs/reference/source-licensing-evidence-2026-09-27.md:1` merges the heading and the first paragraph into one heading line, which breaks the markdown. Lines 54-57 are still hard-wrapped.
   - `scripts/check-changelog-coverage.py:23-25` still describes the removed cap.
6. The docs allow list still has two holes. `docs/charts/` is compiled into the binary (`src/cli/chart.rs:9`). `tests/benchmark_cli_structure.rs:274-279` reads `README.md` and every `docs/**/*.md`. A branch touching only those skips the Rust jobs. Main still runs everything, so this cannot affect a tag.
7. publish-release makes the release public before its Docker steps. If a Docker step fails, the release is already public.
8. Twelve earlier commits in the range each failed their branch run (36662501429 through 36681892037). None was the tip of main, but they break bisection.
