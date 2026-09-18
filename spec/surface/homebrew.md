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

## Installation Docs Show The Tap Path

```bash
cat ../../README.md ../../docs/getting-started/installation.md | mustmatch like 'brew tap genomoncology/biomcp
brew install biomcp
homebrew-biomcp'
```
