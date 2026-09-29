#!/usr/bin/env python3
"""Require a stable v-prefixed tag to match every committed release version."""

from __future__ import annotations

import argparse
from pathlib import Path
import re
import subprocess
import sys

STABLE_TAG = re.compile(r"^v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$")
VERSION_LINE = re.compile(r'^version\s*=\s*"([^"]+)"', re.MULTILINE)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tag", required=True)
    parser.add_argument(
        "--repo-root", type=Path, default=Path(__file__).resolve().parents[1]
    )
    parser.add_argument("--sync-script", type=Path)
    return parser.parse_args()


def manifest_version(path: Path) -> str:
    try:
        content = path.read_text(encoding="utf-8")
    except OSError as error:
        raise RuntimeError(f"cannot read {path}: {error}") from error
    match = VERSION_LINE.search(content)
    if match is None:
        raise RuntimeError(f"cannot read version from {path}")
    return match.group(1)


def main() -> int:
    args = parse_args()
    match = STABLE_TAG.fullmatch(args.tag)
    if match is None:
        if re.fullmatch(r"v?[0-9]+\.[0-9]+\.[0-9]+[-.]?(?:rc|alpha|beta).*", args.tag):
            message = "pre-release tags are not supported by the release workflow"
        else:
            message = (
                "release tag must start with v and contain a stable semantic version"
            )
        print(f"release version check failed: {message}: {args.tag}", file=sys.stderr)
        return 1

    expected = args.tag[1:]
    try:
        cargo = manifest_version(args.repo_root / "Cargo.toml")
        python = manifest_version(args.repo_root / "pyproject.toml")
    except RuntimeError as error:
        print(f"release version check failed: {error}", file=sys.stderr)
        return 1
    if cargo != expected or python != expected:
        print(
            "release version check failed: "
            f"tag={expected}, Cargo.toml={cargo}, pyproject.toml={python}",
            file=sys.stderr,
        )
        return 1

    sync_script = (
        args.sync_script or args.repo_root / "scripts" / "check-version-sync.sh"
    )
    completed = subprocess.run([str(sync_script)], cwd=args.repo_root)
    if completed.returncode != 0:
        print(
            "release version check failed: check-version-sync.sh failed",
            file=sys.stderr,
        )
        return 1
    print(f"release versions match {args.tag}; check-version-sync.sh passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
