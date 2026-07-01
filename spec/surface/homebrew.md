# Homebrew Tap

BioMCP ships a single binary, so the Homebrew install path should download the
same released macOS archives that other release channels publish. These checks
keep the formula, release automation, and installation docs aligned with that
binary-download contract.

## Homebrew Formula Downloads Release Binaries

The repository should keep a canonical `biomcp.rb` formula or formula template
that points at the official GitHub release assets for both supported macOS
architectures. The formula must carry SHA256 fields so Homebrew verifies the
archive it installs, and it must install the released `biomcp` executable rather
than rebuilding from source or downloading an unchecked asset.

```bash
sed -n '1,260p' ../../Formula/biomcp.rb | mustmatch like 'genomoncology/biomcp
biomcp-darwin-arm64.tar.gz
biomcp-darwin-x86_64.tar.gz
sha256
bin.install
biomcp'
```

Rendering the template for a release should replace the tag, version, and both
architecture checksums without leaving placeholder text behind.

```bash
TAG=v9.8.7 VERSION=9.8.7 ARM64_SHA256=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa X86_64_SHA256=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb python3 - <<'PY' | mustmatch like 'version "9.8.7"
releases/download/v9.8.7/biomcp-darwin-arm64.tar.gz
sha256 "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
releases/download/v9.8.7/biomcp-darwin-x86_64.tar.gz
sha256 "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
NO_PLACEHOLDERS_LEFT'
import os
from pathlib import Path

formula = Path("../../Formula/biomcp.rb").read_text(encoding="utf-8")
formula = formula.replace("__TAG__", os.environ["TAG"])
formula = formula.replace("__VERSION__", os.environ["VERSION"])
formula = formula.replace("__DARWIN_ARM64_SHA256__", os.environ["ARM64_SHA256"])
formula = formula.replace("__DARWIN_X86_64_SHA256__", os.environ["X86_64_SHA256"])
print(formula)
if "__" not in formula:
    print("NO_PLACEHOLDERS_LEFT")
PY
```

## Release Workflow Updates Or Emits The Tap Formula

Publishing a BioMCP release should also update the separate Homebrew tap. When
the tap token is unavailable, the workflow should still leave operators with the
exact rendered formula to commit manually instead of silently skipping the
Homebrew channel.

```bash
awk '/homebrew|Homebrew|brew|tap|biomcp.rb|HOMEBREW|formula|darwin|sha256|artifact|manual/{print}' ../../.github/workflows/release.yml | mustmatch like 'genomoncology/homebrew-biomcp
HOMEBREW_TAP_TOKEN
biomcp.rb
biomcp-darwin-arm64.tar.gz.sha256
biomcp-darwin-x86_64.tar.gz.sha256
formula
manual'
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
