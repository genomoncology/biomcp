---
priority: 1
---

# 2215 — Finish the 0.9.3 release and open the next development version

Status: OPEN.

Milestone: 0.9.4

## Outcome

Main reports the next development version instead of the released number, the changelog has an empty Unreleased section with the conventional headings, and a short release record preserves the actual 0.9.3 publication evidence.

- Evidence: PM authorized this work after tagged release run `39150372750` succeeded on `5641de7d7b79b53872f8599dbb4dba22e611dcaf`. Current main still carries stable0.9.3 and begins the changelog with that release. PM released the main hold and owns GitHub issue replies with Ian.
- Changes: `Cargo.toml` `Cargo.lock` `pyproject.toml` `uv.lock` `manifest.json` `CITATION.cff` `server.json` `Formula/biomcp.rb` `CHANGELOG.md` `sdlc/records/2215-0-9-3-release.md`. Use the existing post-release version convention and version tools; narrow any unneeded version-file edits after reading their rules. No release workflow, publisher, tag or product behavior change.
- Keeps: released tag and artifacts unchanged. Public text names only BioMCP and anonymous build-host labels. No new reporting framework or benchmark collection.
- Proof: inspect current manifests/changelog before edits; existing version-sync and development-version contracts pass through a build job, and a built CLI identifies the coming development version. Verify the short record against GitHub API evidence: tag and commit, publishing run and elapsed duration, five archive sizes, full rehearsal count with each failed rehearsal cause, and PM verification. Record PM verification explicitly as reported, rather than claiming a new independent install. Fresh review and normal landing/main checks follow.

PM verification: GitHub release is public with five archives/checksums; PyPI serves0.9.3 and `uvx` runs `biomcp0.9.3`; version and latest container tags share the amd64/arm64 index; Homebrew formula names0.9.3. PM owns external GitHub replies. No MCP Registry submission or external directory change is authorized here.
