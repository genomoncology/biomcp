# Release checklist

Work this checklist for every BioMCP release. `docs/reference/release-process.md` explains how the release workflow runs. This file lists what people and agents must do, in order. Copy the checklist into the release's ticket and tick each item with its evidence: a commit, a CI run ID, a URL or a command output.

Ian cuts every release. Users install releases, not main. The release bar is the tagged commit.

### 1. Before the go request

- [ ] Every open review file in `sdlc/issues/` for this release has a disposition line for each finding: fixed with its commit, or deferred with Ian's recorded approval.
- [ ] Every ticket merged since the last release has a ticket file, a code review verdict from a fresh read-only reviewer with its dispatch reference, and its local build-job gate evidence.
- [ ] A fresh read-only reviewer has reviewed every commit through the final tip, including records-only commits, and read the commit diffs. Commits after its ACCEPT have their own review.
- [ ] Every claimed fix fails its test when the fix is reverted.
- [ ] Every new or changed data source has its terms stated plainly in `docs/reference/sources.json` and `docs/reference/source-licensing.md`, including sources used only for identifiers.
- [ ] The changelog section `## X.Y.Z — YYYY-MM-DD` describes each change for users, names every GitHub issue the release fixes, and cites every ticket merged since the last release. `scripts/check-changelog-coverage.py --tag vX.Y.Z --revision <candidate-sha>` passes without creating a tag.
- [ ] Every public version file says X.Y.Z — the release commit drops the `-dev` suffix Cargo and Python carry between releases, and `check-release-versions.py` refuses the tag if any committed version still carries it. `scripts/check-version-sync.sh` and `scripts/check-release-versions.py --tag vX.Y.Z` pass against the committed stable candidate, without creating a tag.
- [ ] Every change landed through the coordinator's existing landing gate, singly or in a reviewed batch that shares one gate, followed by the full main check through a build job. The final candidate's routine, shipped-feature and required long checks passed.
- [ ] On the final candidate, run `python3 tools/check-replay-live --live` with the authorized source-disable and access settings. Retain every recording's result, the complete summary counts and candidate revision. Repair or re-record every blocked or drifted entry before tagging; name disabled sources explicitly because they remain unverified. See the release process for field-projection limits and credential restrictions.
- [ ] Both the prior release and candidate measurements used the same build host, corrected measurement harness, toolchain, workload and cache settings; retain raw samples and label failed collections incomplete.
- [ ] Every `gh` call in `.github/workflows/release.yml` carries `GH_REPO: ${{ github.repository }}` (or a checkout), and every repository script runs after a checkout — `tests/test_release_workflow_provenance.py` enforces both. Every job that downloads the release while it is still a draft has `contents: write`.
- [ ] Merged ticket branches, worktrees, local branches, stale ticket-number reservations and scratch build folders for this repository are removed. Branches owned by the 1.0 team are left alone. `workspace repos --dirty` is clean for this repository.
- [ ] After every release ticket and the final measurement record has landed, manually run the full untagged `release.yml` rehearsal from the final candidate commit, with its full `candidate_sha` and stable `rehearsal_version`. Every required job, including `rehearsal-complete`, succeeds. Record the run URL and exact SHA in the go request. No tag or publication is created. A later source/workflow change requires a new full rehearsal.
- [ ] Record the rehearsal limits: GitHub release creation/upload/publication, PyPI trusted upload, registry authentication/pushes and `latest` movement, Pages deployment/live-site verification, and the Homebrew push remain unexercised. The rehearsal validates production builds, installed-package and container smokes, generated documentation and publication payloads without their credentials.


### 2. The go request

- [ ] The request names the exact commit to tag, its successful full manual rehearsal, the local gate evidence, and the GitHub issues the release fixes.
- [ ] Ian's reviewer returns GO.
- [ ] Ian says go.

### 3. Tag and release

- [ ] Tag the named commit explicitly: `git tag vX.Y.Z <sha>` and `git push origin vX.Y.Z`. Never tag `HEAD` after main has moved.
- [ ] Watch the release workflow. The order is: version-check, direct qualification, release-gate, create-draft, build, publish-assets, pypi-build, wheel-smoke, tagged documentation publication, docs-live, container-publish, pypi-publish, publish-release (whose Docker work and latest-tag move run before the release goes public), homebrew-tap.
- [ ] If a job fails before pypi-publish, fix the cause, delete the draft release and the tag, and start again from section 1. If a job fails after pypi-publish, re-run only the failed jobs. PyPI never accepts the same version twice.

### 4. Verify the published release

- [ ] PyPI serves X.Y.Z. A clean `uv tool install biomcp-cli==X.Y.Z` (or pip) runs `biomcp --version`.
- [ ] The GitHub release is public with all five archives and their SHA-256 sidecars.
- [ ] Each published binary's `biomcp --version` names X.Y.Z and the tagged commit.
- [ ] `ghcr.io/genomoncology/biomcp:X.Y.Z` and `:latest` both point at the new image for `linux/amd64` and `linux/arm64`.
- [ ] The Homebrew formula names X.Y.Z with the published checksums, and `brew install` runs `biomcp --version`.
- [ ] Each GitHub issue the release fixes is reproduced against the published artifacts and now passes. The evidence goes in the release record.


### 5. After the release

- [ ] Post a short reply on each fixed issue with the evidence, and close it.
- [ ] Write the release record in `sdlc/records/` with the tag, the commit, the release run ID, the release workflow's run time, each archive's download size, the number of full rehearsals the release needed with the cause of each failed one, and the verification evidence.
- [ ] Bump main to the next development version (for example `X.Y.(Z+1)-dev` in Cargo and `X.Y.(Z+1).dev0` in Python) through a ticket branch, so a build from main reports the coming version as a dev build instead of the released number.
- [ ] Add an empty `## Unreleased` section to the changelog.
- [ ] Review the committed `server.json` and submit to the MCP Registry and other directories separately. Record each acceptance.
- [ ] Record any lesson from the release in this checklist or the runbook.

## Worked for 0.9.1

This checklist was first worked for 0.9.1 on 2026-09-30; the ticket for that pass is `sdlc/tickets/1283-close-the-fourth-go-request-round.md`, whose go request cites this file.
