---
flow: build
priority: 10
---
# Restore truthful version, changelog, and citation metadata

The repository is still released as v0.8.25, but post-release work sits under
the v0.8.25 changelog heading and `CITATION.cff` claims a future v0.9.0 with a
placeholder DOI. A release workflow also rewrites version files after checkout.
Those states make source identity and public claims impossible to verify.

This ticket is the fail-closed root of the runnable board: every other active
root depends on it. That ordering is deliberate because the current
release-published workflow can still write public channels before the later
release redesign lands.

## Metadata contract

- Preserve the published v0.8.25 changelog facts. Create one top `Unreleased`
  section and move only entries added after the v0.8.25 tag into it.
- Keep the normal main-branch package version at the latest published version
  until a deliberate release-version commit changes it. Development binaries
  distinguish source using the existing canonical eight-character Git
  revision, not a made-up future package version. Release manifests bind that
  compact runtime identity and executable hash to the full commit SHA.
- Remove placeholder/future DOI and paper-version claims from `CITATION.cff`.
  Cite the latest real software release and omit fields for facts that do not
  exist.
- Cargo metadata/lock, Python metadata/root `uv.lock`, `server.json`,
  `manifest.json`, formula templates, citation metadata, and every other
  machine-readable version surface must agree in a release-version commit.
  Workflows may validate or read that committed version; they never rewrite it.
- A nonempty `Unreleased / Breaking changes` section requires the next release
  version to increase the minor component while major remains zero. This pins
  the v0.9.0-or-later boundary required by tickets 0690 and 0878.
- Immediately replace the current release-published trigger with a read-only,
  manually callable release guard. It has no public-write permissions or
  publishing steps and exits nonzero with the stable message
  `release disabled until ticket 0957 installs the public-artifact gate`.
  Ticket 0952 later replaces this guard with stage-only operation; only 0957
  may enable promotion. The no-release rule is not merely an operator
  convention in the meantime: the committed workflow must make publication
  unavailable.

## Done when

- Tests reconstruct the v0.8.25 tag boundary and prove later entries are under
  `Unreleased` without altering published history.
- A single version-lock check names every governed file, rejects one-file
  drift, a dirty release rewrite, a placeholder DOI, and an uncommitted future
  version.
- Breaking and nonbreaking changelog fixtures prove the pre-1.0 semver rule.
- README, release/operator docs, and citation guidance make no claim that an
  unreleased version or paper already exists.
- Workflow-contract tests prove release events cannot invoke the guard, its
  manual invocation has only read permission and the exact nonzero message,
  and no helper or alternate workflow can publish around it.
- No tag, GitHub release, registry publication, or public asset is created by
  this ticket.

## Authorized test changes

Design commits may restate `CHANGELOG.md`, `CITATION.cff`, version manifests
and locks, version-sync/changelog/citation tests, `.github/workflows/release.yml`,
release guard tests, and release workflow steps that mutate versions. They
must not change product behavior or rewrite v0.8.25 history.

The src line ceiling may not rise.
