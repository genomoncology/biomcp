---
flow: build
priority: 5
deps: ["0911", "0938", "0952", "0953", "0958"]
---
# Generate and prove the macOS Homebrew formula

Homebrew must consume the same verified macOS artifacts rather than rebuilding
or guessing a checksum. The existing `genomoncology/homebrew-biomcp` tap is the
only supported tap.

## Formula contract

Generate the formula from ticket 0952's staged macOS x86_64 and ARM64 archive
URLs, SHA-256 values, and committed semver. It installs `biomcp` as a
package-managed binary; it does not invoke the standalone installer or write
its ownership receipt, and self-update/uninstall therefore refuse under 0916.
Each archive contains the exact architecture-specific Developer ID-signed and
notarized executable accepted under 0958; formula generation cannot substitute,
strip, or resign it.
The formula test reports the committed version and canonical eight-character
binary revision. The installed executable's SHA-256 must match 0952's candidate
manifest, which binds it to the full source commit. The supported contract is
macOS 14 or later on those two architectures only; do not imply Linuxbrew
support.

Stage the exact formula bytes and expected tap path in the release manifest.
Private candidate proof does not rewrite its final public URLs or fetch them
before they exist. Instead, each macOS job creates an isolated Homebrew cache,
preseeds the exact candidate archive bytes under the cache identity Homebrew
derives from the final immutable versioned URL, disables outbound access, and
installs the staged formula unchanged. A cache miss or attempted network call
fails. Formula bytes and SHA-256 are then the same bytes promotion will write.
Promotion may update the existing tap only after versioned GitHub assets exist
and only through a least-privilege, protected job. A conflicting formula
version fails; an identical replay is a no-op. This ticket neither writes the
public tap nor creates a release.

## Done when

- Formula-generation tests pin both URLs/hashes/version, reject a private or
  mutable URL, prove the offline final-URL cache mechanism, and prove no
  architecture falls through to the other binary.
- Standard `macos-15-intel` and `macos-15` candidate jobs install from a staged local tap,
  execute the formula test and local PNG/MCP smoke, and record the compact
  revision plus executable SHA-256. A quarantined extraction also passes the
  online Gatekeeper assessment defined by 0958.
- Ticket 0957 requires both exact-SHA candidate jobs and then installs from the
  public tap before mutable pointers can advance.
- Installation docs name the existing tap, macOS-only support, `biomcp`
  command, and `biomcp-cli` compatibility alias without conflating package and
  executable names.

## Authorized test changes

Design commits may restate `Formula/biomcp.rb`, formula generation, staged tap
fixtures, candidate workflow, Homebrew release helpers, and installation docs.
No public tap mutation belongs to implementation tests.

The src line ceiling may not rise.
