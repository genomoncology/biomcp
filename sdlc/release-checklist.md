# Release checklist

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
- [ ] Every `gh` call in `.github/workflows/release.yml` carries `GH_REPO: ${{ github.repository }}` (or a checkout), and every repository script runs after a checkout — `tests/test_release_workflow_provenance.py` enforces both. Every job that downloads the release while it is still a draft has `contents: write`.
- [ ] Merged ticket branches, worktrees and local branches are removed. Branches owned by the 1.0 team are left alone. `workspace repos --dirty` is clean for this repository.
- [ ] The tag push has been rehearsed on a scratch repository (a copy of this repository, private, with the release workflow's PyPI job pointed at TestPyPI and the image name pointed at the scratch repository). The rehearsal runs to the end — draft creation, draft download, container publish, the latest move, the public flip and the Homebrew skip — and the run URL goes in the go request. A failed rehearsal is fixed and repeated; the real tag never cuts first.

### 2. The go request

- [ ] The request names the exact commit to tag, its green main CI run, the gate outputs, and the GitHub issues the release fixes.
- [ ] Ian's reviewer returns GO.
- [ ] Ian says go.

### 3. Tag and release

- [ ] Tag the named commit explicitly: `git tag vX.Y.Z <sha>` and `git push origin vX.Y.Z`. Never tag `HEAD` after main has moved.
- [ ] Watch the release workflow. The order is: version-check, create-draft, build, pypi-build, wheel-smoke, docs-live, container-publish, pypi-publish, publish-release (whose Docker work and latest-tag move run before the release goes public), homebrew-tap.
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

## Worked for 0.9.1

This checklist was first worked for 0.9.1 on 2026-09-30; the ticket for that pass is `sdlc/tickets/1283-close-the-fourth-go-request-round.md`, whose go request cites this file.
