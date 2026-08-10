---
flow: build
priority: 8
deps: ["0952"]
---
# Establish protected native code signing and notarization

The public macOS and Windows executables must not leave release agents to
choose whether or how to sign them. Establish one fail-closed finalization seam
before ticket 0953 adds those platform builds. This ticket proves the seam with
fixtures; it does not claim that a production certificate was used or that a
real BioMCP candidate was notarized.

## Finalization contract

The seam accepts one executable plus its target, full source SHA, committed
version, and unsigned SHA-256. It returns one immutable signed executable and a
checksummed evidence record. It never compiles, strips, repacks, or otherwise
changes program code. Ticket 0953 must call it after compilation and before
assembling a native archive or wheel. Ticket 0939's executable `biomcp-cli`
shim goes through the same seam separately before wheel assembly; being small
does not exempt it from platform trust checks. An unsigned macOS or Windows
executable is a private intermediate and cannot be registered as a releasable
artifact.

For each architecture-specific macOS executable:

- use a protected Developer ID Application identity with the expected Team ID,
  certificate fingerprint, valid chain, matching private key, hardened runtime,
  and secure timestamp;
- verify the final signature strictly and record unsigned and signed hashes,
  identity, Team ID, certificate chain/fingerprint, timestamp, and signing-job
  identity;
- put exactly that signed executable in a temporary ZIP, submit it with pinned
  `notarytool`, wait for an accepted result, and inspect the complete notary log
  even when the status is successful; and
- record the ZIP hash, submission ID, status, and log hash beside the exact
  signed-executable hash. An error or an unapproved warning blocks finalization.

The approved notary-warning allowlist starts empty. Any warning blocks unless a
later ticket names its exact stable code and message pattern, explains why it
is safe, and adds a rejecting near-match fixture. An implementation agent may
not add or broaden that policy while finalizing a candidate.

The distributed native archive may remain `tar.gz`; it must contain the exact
accepted signed executable without mutation. Apple does not support stapling a
ticket to a standalone binary or directly to a ZIP, so do not claim a stapled
or offline notarization guarantee. Candidate and public proof instead set
`com.apple.quarantine` on the downloaded archive before extraction and verify
that the attribute propagates, or explicitly simulate the propagated attribute
on the extracted executable when the fixture transport cannot preserve it. On
a clean supported Mac they run `spctl --assess --type execute`, verify the exact
signed hash, and execute the command.
Documentation states that first Gatekeeper assessment can require network
access.

For the Windows x86_64 executable:

- use a protected CA-issued Authenticode code-signing identity whose publisher,
  Code Signing EKU, validity, trust chain, private-key match, and candidate-
  approved fingerprint are checked before signing;
- use SHA-256 and a protected RFC 3161 timestamp service, then require
  `signtool verify /pa /all /tw` on the final PE; and
- record unsigned and signed hashes, publisher, chain/fingerprint, timestamp,
  timestamp authority, and signing-job identity. Every native archive, wheel,
  and MCPB consumer copies the corresponding exact signed PE. The full binary
  and the 0939 shim have separate signed hashes and evidence.

Signing establishes a verifiable publisher but does not promise that a new
binary has accumulated Microsoft SmartScreen reputation. Documentation and
tests must not make that claim.

The macOS universal executable used by ticket 0956 is a separate derived
artifact: `lipo` consumes the verified architecture slices, then this seam
signs and notarizes the universal output after `lipo`. Its own unsigned/signed
hashes and notary evidence are recorded; it is never described as byte-equal
to either thin executable.

## Signer trust policy

Create one reviewed, non-secret `release/signing-policy.json` with a versioned
schema. It is the authority for the exact Apple Team ID and Developer ID leaf
certificate SHA-256 fingerprint, Windows publisher and Authenticode leaf
fingerprint, MCPB signer subject/fingerprint, RFC 3161 timestamp URL and
certificate policy, and the allowed Apple notary service/tool profile and
network destinations. The pinned tool may follow only a notary upload URL it
obtains from that approved Apple service; arbitrary stage input cannot supply
an endpoint. Certificate material, keys, passwords, and tokens never belong in
this file.

The protected environment is named `biomcp-release-signing` and independently
stores `BIOMCP_SIGNING_POLICY_SHA256`. Before credentials or network, stage
hashes the candidate commit's policy bytes and requires that exact protected
digest. Workflow inputs, the candidate manifest, and the signing job cannot set
or override either the policy or expected digest. The manifest records both
after the check; it is evidence, not the trust anchor.

At this ticket's implementation landing the policy may be explicitly
`enabled: false` with no fake identity values; every production signing route
then fails with `release signing policy is not provisioned`. Provisioning or
rotating an identity/endpoint requires a separate reviewed policy commit, Ian's
approval, and a separate protected-environment digest update before staging a
release commit. A candidate whose policy bytes differ from its first parent's
policy is always rejected, so the reviewed provisioning/rotation commit must
already be an ancestor of the later release-version commit. Test policies and
fixture roots are structurally marked non-production and can never match the
protected production digest.

Production signing, timestamp, and Apple notary calls occur only inside the
protected 0952 candidate stage. They are the only permitted external
verification writes during staging and are not publication: no BioMCP asset,
tag, release, package, registry row, tap commit, or mutable pointer is exposed.
Missing, expired, mismatched, or unavailable credentials fail before artifact
registration. Routine and implementation tests use only local fixture
identities and services; private keys and credential values never enter logs,
artifacts, caches, or provenance.

## Done when

- Fixture Mach-O and PE tests cover valid finalization, content tampering,
  wrong target/hash/key/publisher/Team ID/fingerprint/EKU, invalid or expired
  chains, missing/invalid timestamps, duplicate signing, and secret exclusion.
- Notary fixtures cover accepted and rejected submissions, timeout, a success
  with a failing or unread log, every warning under the initially empty
  allowlist, and a signed executable that differs from the submitted ZIP
  contents.
- The interface rejects archive/wheel registration before finalization and
  rejects any mutation or second signing after the signed hash is recorded.
- Workflow-contract tests prove real credentials and external services are
  reachable only in a protected candidate job and missing credentials fail
  before upload or timestamp traffic.
- Policy tests cover unprovisioned state, protected-digest mismatch, workflow
  input override, same-candidate policy change, approved prior rotation,
  endpoint substitution/redirect, fixture-policy refusal, and manifest values
  that disagree with the independently checked policy.
- Ticket 0953 is named as the owner of real five-target integration; this
  ticket neither builds those release targets nor publishes an artifact.

## Authorized test changes

Design commits may add bounded signing/notary helpers, local certificate and
service fixtures, protected-stage workflow structure, evidence schemas, and
release/security documentation. Fixture private keys must be unmistakably
test-only and rejected by production policy.

The src line ceiling may not rise.
