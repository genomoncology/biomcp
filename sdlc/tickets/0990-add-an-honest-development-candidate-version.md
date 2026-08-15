---
flow: build
priority: 10
---

# 0990: Add an honest development candidate version

## Outcome

BioMCP can build and privately stage `0.9.0-dev.1` while its Python wheels use
the valid PEP 440 version `0.9.0.dev1`, without claiming that unpublished bytes
are the public citation, MCP directory, registry, or Homebrew release. A
development candidate is structurally unable to enter promotion.

## Current facts

The repository and every public metadata file currently say `0.8.25`.
`release/candidate.py` accepts loose prerelease SemVer but records only one
version. `release/build_target.py` reuses that Rust version in wheel filenames
and metadata, where `0.9.0-dev.1` is not the intended Python package identity.
`scripts/check-version-sync.sh` requires the Rust package, Python package, and
all public metadata to be identical. The breaking population response change
is described under `New features`, so the pre-1.0 minor-version check does not
see it.

## Scope

- Upgrade the candidate manifest to schema 2. Keep `version` as the canonical
  Rust/Cargo candidate identity and add exact `python_version` and
  `candidate_kind` fields. The only accepted forms are:
  - release: Rust and Python are the same stable `X.Y.Z`;
  - development: Rust is `X.Y.Z-dev.N`, Python is `X.Y.Z.devN`, and `N` is a
    positive decimal integer with no leading zero.
  Every `X`, `Y`, and `Z` component must be either `0` or a decimal integer
  beginning with `1` through `9`. The Python value must equal its canonical
  PEP 440 serialization; do not accept a spelling that package tooling would
  normalize to another string.
  Candidate initialization must read and validate both committed manifests at
  the exact source SHA. Artifact records remain bound to the canonical Rust
  `version`; wheel filename, directory, `METADATA`, and inspection are bound to
  `python_version`.
- Thread both identities through target construction and the five private wheel
  jobs. Native archives, binaries, SBOM identity, OCI, Homebrew staging, MCPB,
  and binary smoke continue to use the Rust candidate version. Wheels use the
  Python version only for their package filename and internal package metadata;
  their installed binaries must still report the exact Rust build identity.
- Reject every development candidate in `release/promotion.py` before manual
  evidence, credential, network, or artifact processing, and independently in
  `release/publish-versioned.sh` before tag, GitHub, PyPI, GHCR, or Homebrew
  operations. A schema-1 manifest is unsupported rather than guessed or
  migrated during release execution.
- Teach `scripts/check-version-sync.sh` the two states. A stable checkout keeps
  the current all-metadata equality rule. A development checkout requires the
  exact Rust/Python mapping, matching Cargo and uv lock roots, and requires
  `manifest.json`, `CITATION.cff`, both `server.json` versions, and any concrete
  formula version to equal the latest reachable stable release tag rather than
  the unpublished development version. It must still reject an uncommitted
  version change and must apply the pre-1.0 breaking-change check to the stable
  base of a development version.
- Move the population response item into an explicit top-level `Breaking
  changes` section under `Unreleased`. Prove that `0.8.26` and
  `0.8.26-dev.1` are rejected while `0.9.0-dev.1` is accepted for this
  changelog.
- Make one isolated version commit that sets Cargo/Cargo.lock to
  `0.9.0-dev.1` and pyproject/uv.lock to `0.9.0.dev1`. Keep the committed
  published `manifest.json`, `CITATION.cff`, and both `server.json` versions at
  `0.8.25`. Keep the Homebrew source template as `__VERSION__`. Update the
  release-process documentation to explain this split plainly.

Tests may change beside the owning release, workflow, version-sync, package,
directory-manifest, citation, and release-metadata code. Do not loosen the
existing stable-release equality checks or public-metadata assertions.

This ticket does not enable signing, stage a candidate, create a tag, publish
an artifact, or change any public mutable pointer. Private run ID and source
SHA distinguish repeated development stages; this ticket does not reserve a
development ordinal across workflow runs. A candidate manifest itself cannot
be reused with another source SHA.

## Acceptance

- Candidate schema and focused tests prove the exact stable and development
  pairs, reject malformed, non-canonical, leading-zero, or mismatched pairs,
  reject schema 1, and bind all artifacts to one source SHA and Rust candidate
  identity.
- All five platform wheels have PEP 440 `0.9.0.dev1` filenames and metadata;
  installed `biomcp` and `biomcp-cli` still pass exact
  `0.9.0-dev.1+g<revision>` identity smoke.
- Both promotion entry points reject a complete development candidate before
  any external or credential-dependent action. Stable schema-2 candidates
  retain the existing promotion behavior.
- The committed checkout passes version synchronization with development
  package versions and published metadata still at `0.8.25`. Mutation tests
  fail for a wrong Python mapping, changed public metadata, an uncommitted
  version bump, and either `0.8.26` form in the presence of the recorded
  breaking change.
- Focused candidate, target/package, promotion, workflow, version-sync,
  directory-manifest, citation, and release-metadata tests pass, followed by
  `make lint` and `git diff --check`.

## Dependencies

Ticket 0989, because development candidates rely on the corrected artifact and
promotion evidence boundary.

## Review

- Design review: accepted after defining canonical numeric forms and removing
  an unenforceable cross-run development-ordinal reservation claim
- Code review: pending
