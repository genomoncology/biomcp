#!/usr/bin/env python3
"""Render the release tap payload from the two macOS archive checksums."""
from __future__ import annotations

import argparse
from pathlib import Path
import re

TEMPLATE = 'class Biomcp < Formula\n  desc "Biomedical MCP CLI and knowledge server"\n  homepage "https://biomcp.org"\n  version "${VERSION}"\n  on_macos do\n    if Hardware::CPU.intel?\n      url "https://github.com/genomoncology/biomcp/releases/download/v${VERSION}/biomcp-darwin-x86_64.tar.gz"\n      sha256 "${X86_SHA256}"\n    else\n      url "https://github.com/genomoncology/biomcp/releases/download/v${VERSION}/biomcp-darwin-arm64.tar.gz"\n      sha256 "${ARM64_SHA256}"\n    end\n  end\n  def install\n    bin.install "biomcp"\n  end\n\n  test do\n    assert_match version.to_s, shell_output("#{bin}/biomcp --version")\n  end\nend\n'


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", required=True)
    parser.add_argument("--checksums", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if not re.fullmatch(r"(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)", args.version):
        parser.error("a stable release version is required")
    replacements = {"${VERSION}": args.version}
    for arch, marker in (("x86_64", "${X86_SHA256}"), ("arm64", "${ARM64_SHA256}")):
        path = args.checksums / f"biomcp-darwin-{arch}.tar.gz.sha256"
        fields = path.read_text(encoding="utf-8").split()
        if len(fields) != 2 or fields[1].removeprefix("*") != f"biomcp-darwin-{arch}.tar.gz" or not re.fullmatch(r"[0-9a-f]{64}", fields[0]):
            parser.error(f"invalid archive checksum: {path.name}")
        replacements[marker] = fields[0]
    formula = TEMPLATE
    for marker, value in replacements.items():
        formula = formula.replace(marker, value)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(formula, encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
