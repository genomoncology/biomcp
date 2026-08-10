---
flow: build
priority: 5
deps: ["0932", "0939", "0952", "0953", "0958"]
---
# Build the declared MCPB desktop bundle

MCPB remains a product goal, but the current metadata and tests do not prove a
portable bundle. Build one bounded desktop asset from already verified native
executables; official directory submission remains Ian-owned.

## Bundle contract

Create `biomcp-X.Y.Z.mcpb` and its SHA-256 from the staged candidate. Use the
pinned official MCPB v0.3 schema/tooling. Combine the verified macOS x86_64 and
ARM64 binaries into one universal executable with `lipo`, and include the
verified Windows x86_64 executable with exact
`server.mcp_config.platform_overrides.win32.command = "server/biomcp.exe"`.
Advertise macOS 14+ and Windows 10+ only because the schema cannot honestly
select the Linux CPU variants in this bundle.

The Windows member is byte-for-byte the final Authenticode-signed PE recorded
by 0958. `lipo` creates a new universal Mach-O, so it does not reuse either
thin executable's final signature. After `lipo`, invoke 0958's protected seam
to sign and notarize that universal executable and record its separate
unsigned/signed hashes and submission evidence before MCPB packing. Validate
both slices before and after `lipo`; packing cannot mutate or resign either
platform executable.

The public asset is signed exactly once with the pinned MCPB signing tool and
a protected CA-issued X.509 code-signing certificate whose extended key usage,
chain, validity, key match, subject, and fingerprint are checked against
0958's independently protected `release/signing-policy.json` before use.
Self-signed certificates are fixture-only. Packing produces a private unsigned
intermediate; a protected candidate signing job appends the MCPB signature,
verifies it immediately, and records the unsigned content hash, final signed
asset hash, certificate chain/fingerprint, and signing-job identity in 0952's
manifest. That signed byte sequence is immutable and is the only candidate
0957 may publish; promotion never repacks or resigns it. Missing/expired/wrong
credentials, an existing signature, or verification failure blocks the
candidate without a public write. No private key enters an artifact or log.

The manifest consumes the seven-tool catalog from 0932 and the committed
version/identity from 0952. The bundle contains no source, second full CLI,
test fixtures, caches, credentials, planning files, absolute workstation
paths, or unverified download step.

## Done when

- Schema validation and archive inspection pin filenames, executable modes,
  platform overrides, catalog metadata, hashes, and absence rules, and reject
  any Linux claim.
- Fixture-certificate tests cover valid signing, content/signature tampering,
  wrong key, missing Code Signing EKU, expired/not-yet-valid chain, duplicate
  signing, self-signed production refusal, fingerprint mismatch, and secret/log
  exclusion. They also reject a manifest-selected signer or endpoint that the
  protected policy did not authorize. The final candidate hash is measured
  only after verification.
- Standard `macos-15-intel`, `macos-15`, and `windows-2022` candidate jobs install the
  unpacked bundle and run initialize/tools-list plus a local fixture-backed
  typed call; each reports the staged version and canonical eight-character
  revision, while its platform executable SHA-256 matches the corresponding
  signed full-SHA candidate-manifest entry. macOS proof also applies quarantine
  and passes 0958's online Gatekeeper assessment; Windows proof verifies the
  Authenticode signature and timestamp without promising SmartScreen reputation.
- Ticket 0957 requires all three exact-SHA jobs and verifies the public release
  asset. The Windows lane proves the executable, bundle selection, and stdio
  process contract; an actual Windows 10/11 Claude Desktop installation and
  official MCP Registry/directory submission remain explicit Ian actions, not
  automated success claims.
- No bundle or directory entry is published by this ticket.

## Authorized test changes

Design commits may restate `manifest.json`, MCPB assembly scripts, pinned schema
validation, archive/native smoke fixtures, signing/verification fixtures,
protected release candidate workflow, and MCP client documentation. Existing
stdio safety and tool annotations remain covered. Real certificate material is
never used by implementation tests.

The src line ceiling may not rise.
