# Homebrew Tap

BioMCP ships a single binary, so the Homebrew install path should download the
same released macOS archives that other release channels publish. These checks
keep the formula, release automation, and installation docs aligned with that
binary-download contract.

## Homebrew Formula Downloads Release Binaries

The repository should keep a canonical `biomcp.rb` formula or formula template
that points at the official GitHub release assets for both supported macOS
architectures. The formula must carry SHA256 fields so Homebrew verifies the
archive it installs instead of rebuilding from source or downloading an
unchecked asset.

```bash
find ../.. -path '../../.git' -prune -o -name biomcp.rb -type f -print | sort | xargs -r sed -n '1,220p' | mustmatch like 'genomoncology/biomcp
biomcp-darwin-arm64.tar.gz
biomcp-darwin-x86_64.tar.gz
sha256'
```

## Release Workflow Updates Or Emits The Tap Formula

Publishing a BioMCP release should also update the separate Homebrew tap. When
the tap token is unavailable, the workflow should still leave operators with the
exact rendered formula to commit manually instead of silently skipping the
Homebrew channel.

```bash
awk '/homebrew|Homebrew|brew|tap|biomcp.rb|HOMEBREW|formula/{print}' ../../.github/workflows/release.yml | mustmatch like 'genomoncology/homebrew-biomcp
HOMEBREW_TAP_TOKEN
biomcp.rb
sha256
formula'
```

## Installation Docs Show The Brew Tap Path

The installation guide should show the two commands Mac users need: adding the
BioMCP tap and installing the formula. It should also make the separate tap
repository prerequisite visible so a missing tap is not confused with a BioMCP
binary problem.

```bash
cat ../../docs/getting-started/installation.md | mustmatch like 'brew tap genomoncology/biomcp
brew install biomcp
homebrew-biomcp'
```
