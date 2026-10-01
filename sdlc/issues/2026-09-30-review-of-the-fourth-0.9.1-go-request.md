# Review of the fourth 0.9.1 go request

Filed 2026-09-30 from an independent read-only review of main at `fd55c4ad`, range `65bff6a0..fd55c4ad`. Two fresh reviewers covered release readiness, walking every job as if a real tag were pushed, and re-ran mutations for every fix claimed. Reviewers used scratch clones with scratch tags, a fake `gh` on the path, offline tests and read-only GitHub queries.

## Verdict

Do not tag `fd55c4ad`. The release would stop at its second job. Nothing irreversible would run first.

## Release blocker

- The `create-draft` job (`.github/workflows/release.yml:53-65`) has no checkout and no repository argument. It runs `gh release create "$TAG" --draft --verify-tag --title "$TAG"`. With no git directory and no `GH_REPO`, gh stops with "failed to run git: fatal: not a git repository". The reviewer reproduced this with gh 2.45. `GITHUB_REPOSITORY` alone does not help, because gh reads `GH_REPO`. No tag push has ever run this path: v0.9.0 ran on the `release` event (runs 35171970209 through 35182821002). The checkout guard looks only at `scripts/` and `tools/` paths, so it misses this.
- Fix: set `GH_REPO: ${{ github.repository }}` for every job that calls `gh`, or pass `--repo`. Extend the guard to fail any job that calls `gh` with neither a checkout nor `GH_REPO`. Then walk every job of the tag-push path once more, because this is its first real run.

## What holds

- Main runs 36774283625 (`77c2ae58`) and 36779399161 (`fd55c4ad`) passed all eight jobs. Main's first-parent history in the range is two merge commits.
- A scratch tag passes version sync, release versions and changelog coverage. The reviewer rebuilt the set of 60 tickets independently, and every one is cited in the changelog. The backfill marker is gone.
- `should-move-latest.sh` moves `latest` for a draft v0.9.1 over v0.9.0, stays for a published v0.10.0 and for an rc tag, and the step fails when `gh` fails.
- Removing or moving the publish-release checkout fails the new guard. A string comparison fails the two-digit test. A renamed evidence row fails the registry check. A new record with no bullet fails the changelog gate.
- The live docs serve `fd55c4ad`.
- The #250, #282, #283, #284, #286 and #287 entries match the code, and all six issues are open.
- Cellosaurus, the Human Protein Atlas and PharmacoDB have their terms stated (`source-licensing.md:46`, `:69`, `:91`).
- Record 1281 exists and is accurate. Ticket 1276 states plainly that it had no review dispatch at landing. Record 1264 is unwrapped. No commit bundles a reformat.
- The review dispatch `d008965b` is genuine and read-only. It read `release.yml` lines 680-760, not the whole workflow.

## Tests and gates

- Nothing pins the `gh` failure fix at `release.yml:725-730`. Adding `|| true` back passes, because the provenance test (line 167) and `spec/surface/docker-image.md` only look for the text `gh release list`. Test that a failing `gh` fails the step.
- `should-move-latest.sh` uses exit 1 for both "stay" and errors, and a missing script (exit 127) also reads as "stay". Give "stay" its own exit code and treat every other failure as a failed step.
- The raise-review gate still accepts "ACCEPT 2026-09-30 but pending fixes" and "ACCEPT dispatch abc will fix" (`tools/check-test-wait-ratchet.py:314`). The lookahead checks only what follows ACCEPT and never anchors the end of the value. Match the whole value against one grammar, for example `ACCEPT`, an optional date, an optional dispatch reference, and nothing else. Nine older tickets (1164, 1242, 1243, 1246, 1250, 1254, 1255, 1265, 1266) no longer parse under the new pattern. No raise cites them, so leave them, but say so in the ticket.
- `test_wheel_installs_skills_without_network` failed on unmodified main in the reviewer's scratch clone. Confirm whether it needs a built wheel and is marked, or whether it is broken.

## Changelog and sources

- `CHANGELOG.md:21` is wrong about the cell-line sections. The Human Protein Atlas data is `biomcp gene cell-lines <symbol> --group` and reports RNA nTPM, not a `get cell-line` section (record 1213). The ChEMBL section lists ChEMBL cell-line records with assay counts, not target compounds (record 1214).
- `CHANGELOG.md:20` ends with "Ian confirmed the entity ships 2026-09-30." Remove that internal note from the public notes.
- `CHANGELOG.md:39` cites ticket 1260, which has an open ticket, no record and no merge. Remove it, or land it.
- The ChEMBL entry in `sources.json` and `source-licensing.md` omits the `get cell-line <id> chembl` surface.
- DepMap and Cell Model Passports have no entries. BioMCP only shows their identifiers through Cellosaurus cross-references, but Ian's rule asks that every source's terms be stated. Add identifier-only entries with their terms.

## Records

- Ticket 1281 line 9 still says the ChEMBL about-page citation carried. Main names "the about page" with no URL (`source-licensing.md:243`).
- Record 1276 line 12 still says "Code review: ACCEPT 2026-09-29". Match ticket 1276's statement.
- Ticket 1269's Fix list (lines 7-39) is still hard-wrapped.
- Ticket 1282 says it was filed with its record, and no record 1282 exists.
- Five commits followed the review's ACCEPT: `9195b9cc`, `a741497d`, `041e3d18`, `77c2ae58` and `fd55c4ad`. The next review must read the whole release workflow and every commit through the final tip.

## Ian's direction, 2026-09-30

- Keep an up-to-date release checklist in the BioMCP repository, and point to it from the repository's `AGENTS.md`, so the whole release plan is written down for every future release.
- The reviewer drafted the checklist below. The developer lands it as `sdlc/release-checklist.md`, corrects it against the fixed workflow, points the `AGENTS.md` Releases section at it, and corrects runbook step 8 in `docs/reference/release-process.md`, which still says `latest` moves after the release goes public.

## Draft release checklist

Work this checklist for every BioMCP release. `docs/reference/release-process.md` explains how the release workflow runs. This file lists what people and agents must do, in order. Copy the checklist into the release's ticket and tick each item with its evidence: a commit, a CI run ID, a URL or a command output.

Ian cuts every release. Users install releases, not main. The release bar is the tagged commit.

### 1. Before the go request

- [ ] Every open review file in `sdlc/issues/` for this release has a disposition line for each finding: fixed with its commit, or deferred with Ian's recorded approval.
- [ ] Every ticket merged since the last release has a ticket file, a code review verdict from a fresh read-only reviewer with its dispatch reference, and a record naming its green CI run.
- [ ] A fresh read-only reviewer has reviewed every commit through the final tip, including records-only commits, and read the commit diffs. Commits after its ACCEPT have their own review.
- [ ] Every claimed fix fails its test when the fix is reverted.
- [ ] Every new or changed data source has its terms stated plainly in `docs/reference/sources.json` and `docs/reference/source-licensing.md`, including sources used only for identifiers.
- [ ] The changelog section `## X.Y.Z — YYYY-MM-DD` describes each change for users, names every GitHub issue the release fixes, and cites every ticket merged since the last release. `scripts/check-changelog-coverage.py --tag vX.Y.Z` passes on a scratch tag.
- [ ] Every public version file says X.Y.Z. `scripts/check-version-sync.sh` and `scripts/check-release-versions.py --tag vX.Y.Z` pass on a scratch tag.
- [ ] Every change landed through a `tickets/*` branch whose CI run finished green, merged with `git merge --no-ff`.
- [ ] The candidate commit's main CI run finished with every job green. Tag pushes start no CI, so this run is the bar.
- [ ] The live docs serve the candidate commit or a descendant.
- [ ] Every `gh` call and every repository script in `.github/workflows/release.yml` has a checkout or an explicit repository in its job.
- [ ] Merged ticket branches, worktrees and local branches are removed. Branches owned by the 1.0 team are left alone. `workspace repos --dirty` is clean for this repository.

### 2. The go request

- [ ] The request names the exact commit to tag, its green main CI run, the gate outputs, and the GitHub issues the release fixes.
- [ ] Ian's reviewer returns GO.
- [ ] Ian says go.

### 3. Tag and release

- [ ] Tag the named commit explicitly: `git tag vX.Y.Z <sha>` and `git push origin vX.Y.Z`. Never tag `HEAD` after main has moved.
- [ ] Watch the release workflow. The order is: version-check, create-draft, build, pypi-build, wheel-smoke, docs-live, container-publish, pypi-publish, publish-release, homebrew-tap.
- [ ] If a job fails before pypi-publish, fix the cause, delete the draft release and the tag, and start again from section 1. If a job fails after pypi-publish, re-run only the failed jobs. PyPI never accepts the same version twice.

### 4. Verify the published release

- [ ] PyPI serves X.Y.Z. A clean `uv tool install biomcp-cli==X.Y.Z` (or pip) runs `biomcp --version`.
- [ ] The GitHub release is public with all five archives and their SHA-256 sidecars.
- [ ] `ghcr.io/genomoncology/biomcp:X.Y.Z` and `:latest` both point at the new image for `linux/amd64` and `linux/arm64`.
- [ ] The Homebrew formula names X.Y.Z with the published checksums, and `brew install` runs `biomcp --version`.
- [ ] Each GitHub issue the release fixes is reproduced against the published artifacts and now passes. The evidence goes in the release record.

### 5. After the release

- [ ] Post a short reply on each fixed issue with the evidence, and close it.
- [ ] Write the release record in `sdlc/records/` with the tag, the commit, the release run ID and the verification evidence.
- [ ] Bump main to the next development version (for example `X.Y.(Z+1)-dev.1` in Cargo and `X.Y.(Z+1).dev1` in Python) through a ticket branch.
- [ ] Add an empty `## Unreleased` section to the changelog.
- [ ] Review the committed `server.json` and submit to the MCP Registry and other directories separately. Record each acceptance.
- [ ] Record any lesson from the release in this checklist or the runbook.

## Dispositions (recorded 2026-10-01, fifth round)

- Release blocker (create-draft, GH_REPO): fixed at b5a25a89 — GH_REPO on all seven gh-calling jobs, plus `test_every_job_calling_gh_names_its_repository`.
- Tests and gates, gh-failure pin: fixed at b5a25a89 — `test_the_latest_step_fails_when_gh_fails`; the guard was widened at the fifth round to find gh mid-line (`if ! gh`, `$(gh …)`).
- Tests and gates, stay exit code: fixed at b5a25a89 — exit 3 stay / 2 bad tag, `test_a_stay_is_exit_three_and_errors_are_distinct`; the case arms are pinned by the fifth round's `test_the_decision_case_accepts_exactly_zero_three_and_catchall`.
- Tests and gates, raise-review grammar: fixed at b5a25a89 (whole-value fullmatch) and tightened at the fifth round (hex dispatch or one/two-word name; promise phrases reject); the nine older verdicts are named in ticket 1283.
- Tests and gates, wheel test: confirmed at b5a25a89 — the module carries `needs_binary` and reads `BIOMCP_BIN`; the scratch-clone failure is designed behavior, recorded in ticket 1283.
- Changelog 21/20/39: fixed at b5a25a89 and bc5a7882 (the expected-set correction).
- ChEMBL surface: sources.json at b5a25a89; the page detail section at e84d83f6.
- DepMap and Cell Model Passports entries: added at b5a25a89; terms corrected to the providers' own pages at the fifth round.
- Records 1281/1276/1269/1282: fixed at b5a25a89.
- The five post-ACCEPT commits: 9195b9cc, a741497d, 041e3d18, 77c2ae58 and fd55c4ad were covered by the third round's dispatch d008965b and the merge re-checks; e84d83f6 and dda87d73 were re-reviewed with the fifth round's dispatch (ticket 1284).
- Checklist landing, AGENTS.md pointer, runbook step 8: fixed at b5a25a89; the duplicate Releases heading from that landing was folded at e84d83f6.
