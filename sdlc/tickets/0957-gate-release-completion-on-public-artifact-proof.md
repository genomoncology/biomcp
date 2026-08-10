---
flow: build
priority: 4
deps: ["0690", "0874", "0876", "0877", "0878", "0881", "0882", "0884", "0909", "0910", "0911", "0912", "0913", "0914", "0915", "0916", "0917", "0918", "0919", "0920", "0921", "0922", "0923", "0924", "0925", "0926", "0927", "0928", "0929", "0930", "0931", "0932", "0933", "0934", "0935", "0936", "0937", "0938", "0939", "0940", "0941", "0942", "0943", "0948", "0949", "0950", "0951", "0952", "0953", "0954", "0955", "0956", "0958"]
---
# Gate release completion on public artifact proof

A successful private build is not proof that users can install the release.
The promotion transaction needs one final, bounded reconciliation across every
public channel before mutable pointers move or a release is called complete.
This is the only ticket authorized to remove 0952's disabled guard and add a
callable `promote` workflow mode or public-write permissions. No public BioMCP
release may be cut between 0952 and this ticket.

This gate deliberately does not wait for optional capability tickets 0880,
0883, and 0908, developer-hook ticket 0895, or fixture-supervisor maintenance
0896–0897. Lower-priority audit work 0959–0965 also follows this gate. Every
one of those tickets explicitly depends on 0957, so normal priority ordering
cannot dispatch it before the trust/release sequence. None repairs a false
result or required release/install surface. In contrast, user-output
correctness tickets 0876, 0877, 0881, and 0882 are explicit dependencies. A
blocked follow-on therefore does not misrepresent the release as unsafe or
incomplete.

## Verification contract

After Ian approves promotion and all immutable versioned writes succeed, a
single verifier reads ticket 0952's checksummed candidate manifest and
installs/downloads only from public GitHub Releases, PyPI, GHCR, the existing
Homebrew tap, and the
GitHub MCPB asset. It never substitutes private candidate artifacts. Every
runtime surface must report the same semver and canonical eight-character Git
revision. Its executable SHA-256 or image digest must match the candidate
manifest that binds it to the full source commit; OCI revision labels also use
that full commit. Do not require a binary to report identity it does not embed.
The public MCPB bytes must match 0956's final signed hash, and pinned MCPB
verification must reproduce the candidate certificate chain and fingerprint.
An unsigned, self-signed, expired, differently signed, or unverifiable public
bundle blocks completion even when its inner executable hash is correct.
The verifier independently loads the exact 0958 signing-policy bytes whose
SHA-256 is pinned by the protected `biomcp-release-signing` environment. Every
public signer, publisher, Team ID, certificate fingerprint, timestamp policy,
and notary record must satisfy that policy as well as the candidate manifest;
the manifest cannot authorize a signer by itself.
Every public macOS native/wheel/Homebrew executable and the MCPB universal
member must also match 0958's Developer ID signature and accepted notarization
evidence. Public proof downloads through a quarantine-producing path, extracts
on a clean supported Mac, performs an online Gatekeeper assessment, verifies
the exact executable hash, and runs it. Every Windows native/wheel/MCPB
executable must be the exact Authenticode-signed PE, pass default-policy and
timestamp verification, and run on the supported lane. This is not reported as
proof that Microsoft SmartScreen reputation has already accumulated.

Run bounded public smokes for help/version/list, local stdio MCP initialization
and the seven-tool catalog, fixture-backed PNG, and installer checksum failure
and success. The updater has one explicit transition: when and only when the
immediately previous public version is exactly v0.8.25, run its update against
the new versioned archive, prove the known 8 MiB legacy limit fails without
changing the installed binary, and prove the verified installer upgrades that
installation instead. Record `legacy-updater-limit-from-v0.8.25` in the final
release record. That waiver is rejected if the previous version is not
v0.8.25 or any earlier public release record already used it. Ticket 0910's
candidate tests must separately prove the repaired updater consumes a
checksum-valid over-8-MiB next-version fixture. Starting with the following
release, successful self-update from the immediately previous public CLI is a
mandatory public gate with no waiver. Pull both
container architectures, install all five platform targets for both native
archives and wheels through their public channels, install Homebrew on both
macOS architectures, and inspect/run the three MCPB platform paths. Before a
real promotion is called complete, require Ian's recorded manual smoke of the
exact bundle in Claude Desktop on Windows 10 or 11; automated Windows Server
execution alone is not described as end-to-end desktop proof. `make verify`
remains the explicit live-provider
lane; every unavailable provider is recorded and any release-owned contract
failure blocks completion.

Only after all required checks pass may the protected workflow update
`latest`, docs, installer/updater pointers, and other mutable aliases. It then
attaches one immutable `release-record-X.Y.Z.json` and checksum containing the
source SHA, every artifact hash/digest, candidate job/run identity, SBOM and
provenance references, formula commit, gate results, public smoke results, and
explicit live-provider limitations. A failure uses a unique
`release-record-X.Y.Z-partial-<run-id>.json`; it never overwrites the final name
or advances mutable pointers.

## Done when

- The protected workflow adds promotion only here, after every frontmatter
  dependency is complete. The enablement test fails if the 0952 guard is
  removed without the exact candidate-job and public-verifier gates in this
  ticket.
- Local registry fixtures exercise the complete verifier, wrong/missing bytes,
  stale caches, wrong architecture/SHA/version, updater download failure,
  provider unavailability, partial publication, retry, and mutable-pointer
  ordering without public writes.
- The promotion workflow requires green exact-release-SHA candidate jobs for
  all target, container, Homebrew, and MCPB paths; committed workflow text alone
  is not accepted as execution evidence.
- Public URLs and expected channel inventories derive from the candidate
  manifest, never a hard-coded old release name.
- Operator documentation gives Ian one go/no-go checkpoint and names the
  remaining manual official-registry/directory actions without claiming them
  complete. Ian creates and reviews the single committed release-version
  change (v0.9.0 or later under 0951), lands it on main, and supplies that full
  SHA to `stage`; no factory ticket creates that commit or publishes it.
- Implementing this ticket does not approve, tag, publish, or move a pointer.

## Authorized test changes

Design commits may add the public verifier and local registry fixtures and
restate release smoke, promotion workflow, release-record, updater, installer,
artifact, docs, and operator tests. Real publication and account-bound directory
submissions remain outside the factory flight.

The src line ceiling may not rise.
