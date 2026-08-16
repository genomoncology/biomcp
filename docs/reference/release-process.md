# Release Process

BioMCP uses one manually started workflow with two deliberately separate modes.
The repository currently records v0.8.25 as the latest published release. The
workflow code does not approve or start a release on its own.

The Rust package can move ahead as a private development candidate without changing public release claims. The current candidate uses `0.9.0-dev.4`, while Python packaging uses its canonical PEP 440 equivalent, `0.9.0.dev4`. The committed citation, MCP directory manifests, and other public metadata continue to identify v0.8.25 until a reviewed stable release commit updates them together. Development candidates may be staged privately but are rejected by promotion and publication.

A schema-2 signing policy can allow one narrow development exception: the outer MCPB archive may remain unsigned for private desktop testing only. The macOS executable inside must still carry its real Developer ID signature and accepted notarization, and the Windows executable must still carry its real Authenticode signature and timestamp. The candidate records the unsigned outer archive, exact exception, and non-promotable status. Stable 0.9.0 still requires a valid outer MCPB signature and real MCPB identity. Unpacking and executing the archive on the three hosted runners proves the archive and signed inner binaries; it does not prove installation compatibility with a particular Claude Desktop build, which remains a separate manual check.

## Go/no-go checkpoint

Ian creates and reviews the single commit that changes the public version to
v0.9.0 or later. All committed version fields, changelog text, citation data,
and install metadata must agree. From that full commit SHA:

1. Run `Release candidate` in `stage` mode. This builds one private candidate,
   signs its native executables and MCPB bundle, and seals a checksummed
   manifest. It does not tag, publish, or update a tap.
2. Review that exact successful run, its 13 registered artifacts, signing and
   notarization evidence, SBOM, provenance, and live-provider result.
3. Record the exact MCPB SHA-256 from Ian's Claude Desktop smoke on Windows 10
   or 11. Record the immediately previous public version's updater and verified
   installer result, including executable hashes before the attempt, after the
   updater, and after the installer. These records must identify the same source
   commit and the final hash must match the sealed Linux executable.
4. Make one go/no-go decision. On “go,” run `promote` with the source SHA,
   successful stage run ID, and both records. The protected
   `biomcp-release-promotion` environment supplies publisher credentials and a
   separately pinned signing-policy hash.

Promotion writes versioned GitHub, PyPI, GHCR, and Homebrew objects first. It
then downloads or installs from those public locations on all five platform
targets, both container architectures, both Homebrew runners, and all three
MCPB paths. The public installer and live provider contracts are also checked.
Only the last job marks the GitHub release latest, adds the GHCR `latest` tag,
and advances the Homebrew tap's main branch. Failed attempts retain a unique
partial record and do not move those pointers. Replaying identical versioned
bytes is a no-op; conflicting bytes stop promotion.

## Provisioning required before a real release

- Provision reviewed Apple and Windows identities for a development candidate; a stable candidate also requires the real MCPB identity and working outer-signature path. Enable the top-level `release/signing-policy.json` only with those real identities and pin its SHA-256 in the protected environment. Because the protected-policy check compares the staging commit with its parent, the identity and policy activation must land in a predecessor commit, followed by a staging commit that does not change the policy bytes.
- Provision the protected signing and promotion environments, required
  publisher tokens, Apple notarization credentials, Windows signing material,
  and the MCPB certificate chain.
- Confirm the existing `genomoncology/homebrew-biomcp` tap and its protected
  main branch accept only the workflow's final fast-forward.

## Separate manual directory actions

The release workflow does not publish `server.json` to the official MCP
Registry or submit BioMCP to third-party directories. After public promotion,
an operator reviews the committed registry metadata and performs each official
submission separately. Record acceptance before describing any directory as
updated.
