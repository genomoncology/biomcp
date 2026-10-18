---
priority: 1
---

# 2215 — Finish the 0.9.3 release and open the next development version

Status: OPEN.

Milestone: 0.9.4

## Outcome

Main reports the next development version instead of the released number, the changelog has an empty Unreleased section with the conventional headings, and a short release record preserves the actual 0.9.3 publication evidence.

- Evidence: PM authorized this work after tagged release run `39150372750` succeeded on `5641de7d7b79b53872f8599dbb4dba22e611dcaf`. Current main still carries stable0.9.3 and begins the changelog with that release. PM released the main hold and owns GitHub issue replies with Ian.
- Changes: `Cargo.toml` `Cargo.lock` `pyproject.toml` `uv.lock` `manifest.json` `CITATION.cff` `server.json` `Formula/biomcp.rb` `CHANGELOG.md` `sdlc/records/2215-0-9-3-release.md` `docs/reference/release-process.md`. Use the existing post-release version convention and version tools; narrow any unneeded version-file edits after reading their rules. No release workflow, publisher, tag or product behavior change.
- Keeps: released tag and artifacts unchanged. Public text names only BioMCP and anonymous build-host labels. No new reporting framework or benchmark collection.
- Proof: inspect current manifests/changelog before edits; existing version-sync and development-version contracts pass through a build job, and a built CLI identifies the coming development version. Verify the short record against GitHub API evidence: tag and commit, publishing run and elapsed duration, five archive sizes, full rehearsal count with each failed rehearsal cause, and PM verification. Record PM verification explicitly as reported, rather than claiming a new independent install. Fresh review and normal landing/main checks follow.

PM verification: GitHub release is public with five archives/checksums; PyPI serves0.9.3 and `uvx` runs `biomcp0.9.3`; version and latest container tags share the amd64/arm64 index; Homebrew formula names0.9.3. PM owns external GitHub replies. No MCP Registry submission or external directory change is authorized here.

## Progress

- 2026-10-18 started

Developer outcome before editing: current Cargo, Python and lock versions are0.9.3 and the changelog starts with0.9.3, so main has no next-dev state. Existing2132 convention sets Cargo0.9.4-dev/Python0.9.4.dev0 while registry, citation and server records keep latest stable0.9.3; preserve the existing formula placeholder convention. Actual claim narrows to `Cargo.toml`, `Cargo.lock`, `pyproject.toml`, `uv.lock`, `CHANGELOG.md` and `sdlc/records/2215-0-9-3-release.md`. Reuse version-sync/development-version contracts and a built CLI version check through one build job; add no redundant test. Release record must distinguish full workflow-dispatch attempts from tagged or container-only runs, retain each failed attempt cause and cite the source of reported publication verification.

Focused check `2215/1792348190229-38824` passed version-sync, all32 existing version/identity contracts in2.94s and formatting, then the existing quality check correctly rejected two literal current-version facts at `docs/reference/release-process.md:57`. Expand only that version-fact prose file before correction: describe the current development Cargo0.9.4-dev/Python0.9.4.dev0 identities and preserve the stable registry/citation explanation. No check, product code or test change; reuse the32 passing contracts and rerun affected quality plus remaining build checks.
- 2026-10-18 next: Developer stopped and pushed once at effe9f05ad9b60c0159a5aaf58282bffe8a61e92. Version sync,32 existing version/identity contracts and formatting passed job1792348190229-38824; its quality failure identified the corrected current-version prose. Version sync and corrected quality passed1792348291161-46891. Tracked text,diff and CLI build passed1792348326368-52731; banner biomcp0.9.4-dev+geffe9f05. Final direct banner assertion/diff passed1792348381030-55399. Two checking-command errors (script invocation and missing scratch tee directory) are excluded from product failures; no build/contract repeats. Fresh review now runs before solo landing of this version/config update, then full actual-main check; start the first six independent0.9.4 lanes afterward.

PM decision needed: existing Unreleased-description contract and branch/main changelog coverage require a2215 description for this version/docs update, while the approved outcome requires an empty Unreleased section. Prefer one Internal after-release bookkeeping bullet to a new version-only gate exception; PM ruling is pending. Reviewer has made no edit and source remains effe9f05.
