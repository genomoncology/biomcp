---
flow: build
priority: 9
deps: ["0911", "0934", "0936", "0937", "0938", "0939", "0940", "0942", "0951"]
---
# Stage one pinned candidate and prepare fail-closed promotion

The current release workflow starts after a GitHub Release is already public,
uses broad top-level write permissions, mutates versions after checkout, and
publishes versioned and mutable container tags together. Release control needs
a reversible stage before any public write and a separately approved promotion
of the exact staged bytes. This ticket prepares the transaction, but it must
not make promotion callable before the downstream artifact and public-verifier
work exists.

## Transaction contract

Replace the publish-event workflow with one callable mode and one disabled
framework:

1. `stage` accepts a full main-branch commit SHA whose committed semver is not
   already released. It checks a clean checkout, canonical `make lint`,
   `make test`, and `make spec`, builds each artifact registered at that commit
   exactly once from that SHA, smokes it, records hashes/provenance, and stores
   it in a private, immutable candidate run. At this ticket's landing the
   registered set is exactly the final-form Linux x86_64 native archive and
   Linux x86_64 wheel; their feature, executable-count, checksum, provenance,
   and inspection contract is shared with 0953. Ticket 0958 first installs the
   protected macOS/Windows finalization seam; tickets 0953–0956 then add the
   other native/wheel, container, formula, and MCPB builders to the same stage
   protocol. They consume an upstream registered artifact or private build
   intermediate by its recorded hash and never compile or assemble the same
   output twice within a run. An old run from an earlier source SHA is never
   extended or promoted. Stage creates no tag, release, package, image, tap
   commit, docs deployment, updater pointer, or registry row.
2. Promotion helpers and offline transaction tests may be prepared for the
   eventual protected job, but the committed workflow has no `promote` input,
   no promotion job, and no public-write permission. Any direct helper
   invocation exits nonzero before credentials or network with the stable
   message `promotion disabled until the 0957 public-artifact gate is installed`.
   Ticket 0957 alone may expose the protected mode after it wires every
   candidate and public verification requirement.

The prepared contract for that later mode accepts only the stage run ID,
re-verifies candidate SHA, committed version, hashes, gate results,
tag/release immutability settings, and every required credential before the
first public write. The immutable `vX.Y.Z` tag is created at that exact SHA,
and every versioned channel consumes staged bytes without checkout or rebuild.

Actions use full commit pins; Rust, targets, uv, maturin, protoc, packaging
tools, runner images where controllable, and container base digests are exact
inputs recorded in provenance. Permissions are absent at workflow scope and
granted per job at the least level needed. Untrusted workflow inputs never
enter shell or tag names without typed validation.

Protected Authenticode timestamping and Apple code-signing/notary submission
under 0958 are allowed candidate-verification calls. They do not publish a
BioMCP release surface, and their returned evidence is private candidate data.
All other public writes remain forbidden during stage.

An existing tag/version/artifact with different bytes fails. An identical
replay is a documented no-op. A partial promotion stops, emits a uniquely named
partial record, and does not move `latest`, docs, installer, updater, or other
mutable pointers. Versioned public artifacts are never overwritten or deleted
to hide a partial attempt.

## Done when

- Offline workflow-contract tests prove stage has no public-write permissions
  or commands, the workflow has no callable promotion route, and every direct
  promotion-helper invocation fails at the disabled guard before side effects.
- Fixture registry servers prove event SHA cannot override candidate/tag SHA,
  no build command occurs during promotion, preflight precedes tag creation,
  byte conflicts fail, identical replay is idempotent, and every injected
  failure preserves truthful partial state.
- A release manifest binds full SHA, semver, artifact hashes, action/tool/base
  pins, SBOM/provenance references, gate results, and stage run identity.
- Baseline-stage tests prove the exact two Linux x86_64 artifacts are built
  once, inspected, and represented once; an unknown, duplicate, rebuilt, or
  unregistered artifact fails. Contract fixtures prove later builders can add
  manifest entries without replacing an upstream artifact's bytes or hash.
- Operator and architecture docs describe the two modes and Ian-owned approval.
- Those docs explicitly prohibit any public BioMCP release after 0952 and
  before 0957 lands; a staged candidate is not a releasable candidate yet.
- Implementing or testing this ticket does not create a tag or publish anything.

## Authorized test changes

Design commits may replace `.github/workflows/release.yml`, add bounded release
disabled transaction helpers and fixture registries, and restate release provenance,
permission, replay, version, architecture, and operator documentation tests.
Public provider calls remain outside deterministic proof.

The src line ceiling may not rise.
