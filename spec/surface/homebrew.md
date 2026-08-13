# Homebrew Tap

The Homebrew formula is generated once from the two signed macOS records and
downloads the same immutable GitHub archives as the native channel.

## Formula Verifies Archives And Installed Executables

```bash
cat ../../Formula/biomcp.rb | mustmatch like 'genomoncology/biomcp
biomcp-darwin-arm64.tar.gz
biomcp-darwin-x86_64.tar.gz
sha256
bin.install "biomcp"
bin.install_symlink
Digest::SHA256.file'
```

The renderer requires matching source identity and real signing evidence,
replaces every marker, and records both native archive hashes as upstreams.

```bash
cat ../../release/homebrew.py | mustmatch like 'macOS candidate identities disagree
unsigned Homebrew source artifact
__DARWIN_ARM64_BINARY_SHA256__
unresolved Homebrew formula placeholder
native-macos-arm64
native-macos-x86_64'
```

## Public Tap Moves Only After Verification

Stage mode renders and tests the formula from an exact preseeded archive cache
on both Mac architectures. Promotion first pushes an immutable formula tag,
installs from that public tag on both architectures, and fast-forwards the tap's
main branch only in the final pointer job.

```bash
cat ../../.github/workflows/release.yml ../../release/publish-versioned.sh | mustmatch like 'homebrew-smoke:
HOMEBREW_NO_INSTALL_FROM_API: 1
public-homebrew-smoke:
refs/tags/$tag
advance-mutable-pointers:
merge --ff-only'
```

## Installation Docs Show The Tap Path

```bash
cat ../../README.md ../../docs/getting-started/installation.md | mustmatch like 'brew tap genomoncology/biomcp
brew install biomcp
homebrew-biomcp'
```
