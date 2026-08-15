---
flow: build
priority: 10
---

# 0991: Support a non-promotable development MCPB

## Outcome

Once real Apple and Windows signing identities are provisioned, BioMCP can
privately stage `0.9.0-dev.1` with signed and notarized native executables
inside an unsigned outer MCPB archive. The candidate and its evidence state
that exception plainly, and no stable release or public promotion can use it.

## Current facts

Ticket 0990 makes every development candidate non-promotable. Native macOS and
Windows artifacts still require real platform signatures. The pinned official
MCPB tool is `@anthropic-ai/mcpb` 2.1.2; its outer signing path is not usable
for this development flight, so requiring an MCPB certificate would prevent
private desktop testing without improving the already signed inner executable
bytes. This is consistent with upstream MCPB issue 278 and the official
development-bundle allowance. The current schema-1 signing policy treats
Apple, Windows, and MCPB
identities as one indivisible requirement, and the workflow always signs and
verifies the outer MCPB.

The repository currently has no `biomcp-release-signing` GitHub environment,
no visible signing secrets, and a deliberately disabled policy with null
identities. Code must not fabricate or substitute fixture identities. A real
stage remains operationally blocked until Ian provisions the protected
environment, real Apple Developer ID/notarization access, and real Windows
Authenticode material.

## Scope

- Upgrade the signing-policy schema to 2. Keep Apple, Windows, and stable MCPB
  identities explicit. Add one exact `development_unsigned_mcpb` object with:
  `enabled`, package `@anthropic-ai/mcpb`, tool version `2.1.2`, a non-empty
  reason, and `blocks_promotion: true`. Commit that exception with
  `development_unsigned_mcpb.enabled: true` while top-level production signing
  remains `enabled: false` and real identities remain null. Fixture policies
  may exercise it only through the existing explicit fixture path.
- Separate native policy validation from stable outer-MCPB identity
  validation. An enabled production policy always requires valid Apple and
  Windows identities for non-Linux executable construction. A stable candidate
  additionally requires the real MCPB identity and the existing signed,
  chain-verified evidence. A development candidate may omit only the outer
  MCPB identity when the exact schema-2 exception is enabled; it may not omit,
  fake, or downgrade either native platform signature.
- Add a fail-closed operation that attests an unsigned development MCPB after
  packing. It consumes the exact validated schema-2 candidate-base manifest
  and binds the archive SHA-256; source SHA; Rust and Python versions;
  candidate kind; stage run ID; signing-policy hash; pinned tool/package;
  exception reason; and `non_promotable: true`. Production job identity is the
  exact GitHub repository, workflow reference, job name, run ID, run attempt,
  and source SHA, all required from GitHub Actions context. Fixture context is
  accepted only with the explicit fixture flag. The attester never copies or
  mutates the packed archive: it atomically writes one evidence file, refuses
  an existing evidence file, and record construction rehashes the archive.
  It must reject stable versions, disabled or malformed policies, fixture
  evidence in production, changed policy bytes, duplicate evidence, missing or
  mismatched job context, and any archive mutation. It reuses
  `verify_protected_policy()` so the protected digest must match and committed
  policy bytes must equal `source_sha^`.
- Make MCPB record construction and inspection accept exactly two states:
  stable outer signature evidence under a real MCPB identity, or the exact
  unsigned-development attestation above. Store an explicit
  `outer_signature_status` and `non_promotable` result in the artifact evidence.
  The latter is valid only when the candidate version is `X.Y.Z-dev.N`; stable
  records and stable candidate registration must reject it.
- Make MCPB record construction consume the validated candidate-base manifest,
  the universal macOS signing/notarization evidence, the Windows native record
  with its nested Authenticode evidence, and both thin macOS upstream records.
  Validate each against the exact bundled executable bytes and cross-check
  source SHA, Rust version, stage run, signing-policy hash, certificate
  identities, chain/timestamp/notary results, non-fixture status, and upstream
  hashes. Swapped or stale evidence, or valid evidence for different bytes,
  source, version, run, or policy, must fail without a record.
- In the private stage workflow, derive the candidate kind from the same
  canonical candidate-base manifest rather than a second shell or workflow
  derivation. The MCPB job downloads and validates that manifest, uses its
  versions, source, run, kind, and policy hash, and remains protected by the
  `biomcp-release-signing` environment. Stable candidates keep the current
  sign-and-verify sequence. Development candidates pack the final outer MCPB
  once, create the unsigned attestation, inspect it, and smoke both signed inner
  executables on the two macOS runners and Windows runner. Do not call or claim
  outer `mcpb sign`/`mcpb verify` success for that branch. Every smoke must
  require the recorded development exception before unpacking, then retain
  `codesign`/Gatekeeper or `signtool` verification of the selected inner
  executable and exact binary identity smoke.
- Update release documentation to say this exception exists only for private
  development desktop testing. Stable 0.9.0 remains blocked on a valid outer
  signature path and real MCPB identity, in addition to the native signing
  prerequisites. Do not describe unsigned outer bytes as signed or releasable.
  Also state that unzip-and-execute runner smoke proves the archive and inner
  binaries, not installation compatibility with a particular Claude Desktop
  build; that remains a separate manual compatibility check.
- Document the activation order forced by the protected-policy rule: real
  identities and the enabled top-level policy land in a predecessor commit,
  followed by a staging commit that does not change policy bytes. The final
  implementation report must repeat this requirement.

Existing signing, MCPB, package-inspection, candidate, and release-workflow
tests may be extended. This ticket changes repository support only. It does not
enable the production policy, invent identity values, create GitHub
environments or secrets, start a stage, tag, publish, or change public metadata.

## Acceptance

- Schema and policy tests reject unknown, incomplete, disabled, fixture, or
  widened exceptions and retain all existing stable signing requirements.
- Red-green tests prove a development attestation binds exact bytes, identity,
  policy, complete GitHub job context, candidate manifest, and tool version;
  every stable, stale, spoofed, mutated, duplicate, or cross-run case fails
  without success evidence.
- MCPB artifact tests prove the stable signed path remains unchanged, the
  development record says `outer_signature_status: unsigned-development` and
  `non_promotable: true`, and neither evidence form can be relabeled as the
  other candidate kind.
- Record tests tamper with universal macOS and Windows signing evidence and
  prove that valid evidence for different bytes, source, version, stage run, or
  policy cannot enter the sealed MCPB record.
- Workflow tests prove all three development smokes use the sealed unsigned
  outer archive only after checking its exception evidence, while still
  verifying the real inner native signatures. Stable workflow assertions keep
  outer signing and verification mandatory.
- Focused signing, MCPB, package, candidate, and workflow tests pass, followed
  by `make lint` and `git diff --check`.
- The final report names the missing GitHub environment and real Apple/Windows
  identities as an external staging blocker, repeats the predecessor-policy
  commit requirement, and distinguishes runner smoke from manual Claude
  Desktop installation; no fixture or unsigned native candidate is staged.

## Dependencies

Tickets 0988 through 0990, because the private candidate depends on trusted
hosted checks, truthful package evidence, and structural development
non-promotion.

## Review

- Design review: accepted after requiring durable exact inner-signature
  evidence, canonical candidate and GitHub job context, atomic attestation
  ownership, protected-policy activation order, and honest desktop limitations
- Code review: pending
