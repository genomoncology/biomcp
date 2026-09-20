# Release Process

BioMCP has one tag-driven release workflow. It runs when a GitHub release is
published or when an operator supplies a `tag` through `workflow_dispatch`.
Either trigger publishes directly; there is no private workflow phase followed
by a separate publication phase.

The repository currently records v0.9.0 as the latest published release. The
Rust package can move ahead as a private development candidate without changing
that public claim. The current candidate uses `1.0.0-dev.1`, while Python
packaging uses its canonical PEP 440 equivalent, `1.0.0.dev1`. The committed
citation, MCP directory manifests, and other public metadata continue to
identify v0.9.0 until a reviewed stable release commit updates them together.
Development-candidate metadata is not a releasable tag.

## Before triggering publication

The release decision and stable metadata update happen before this workflow is
triggered. For v1.0.0, review the exact commit that changes all committed version
fields, changelog text, citation data, and install metadata together. Run the
repository release gates and any separately required manual confidence checks
against that commit. Then create and publish the GitHub release for its tag, or
manually dispatch the workflow with that exact tag.

## What the workflow publishes

The workflow checks out the published GitHub release or a manual `tag` input. It
builds five platform archives and their `.sha256` sidecars and uploads them to
the GitHub release. Four platform jobs build wheels; the `pypi-publish` job
publishes them through the protected `pypi` environment. After the native build
jobs finish, `homebrew-tap` downloads the published macOS checksums and updates
the `genomoncology/homebrew-biomcp` formula when `HOMEBREW_TAP_TOKEN` is
configured.

The workflow does not run the repository's release gates and does not use the
candidate tooling under `release/`. A successful trigger therefore means that
publication has been attempted, not that a private candidate has merely been
prepared for later approval.

## Provisioning required before a real release

- Configure the protected `pypi` environment for trusted publication.
- Ensure the GitHub release exists for the selected tag so archives and
  checksums have a release upload target.
- Configure `HOMEBREW_TAP_TOKEN` when the workflow must update the Homebrew tap;
  without it, that update is skipped.

## Separate manual directory actions

The release workflow does not publish `server.json` to the official MCP
Registry or submit BioMCP to third-party directories. After publication, an
operator reviews the committed registry metadata and performs each official
submission separately. Record acceptance before describing any directory as
updated.
