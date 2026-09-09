#!/usr/bin/env python3
from __future__ import annotations

import argparse
import base64
import hashlib
import json
from pathlib import Path
import subprocess
import tarfile

ROOT = Path(__file__).resolve().parents[1]
INVENTORY = ROOT / "tools" / "zero-coupling-historical.json"
FORBIDDEN = ("bio" + "data", "cfafc69d27c9a2fc" + "74909f21692a418a8b17db83")
TRIAL_SUBJECTS = ("clinical " + "trial", "trial " + "contract")
HANDOFF_MECHANISMS = (
    "check" + "out",
    "path depend" + "ency",
    "patch depend" + "ency",
    "defer" + "red package",
    "gener" + "ated source",
    "rename" + "d handoff",
)


def _text(data: bytes) -> str | None:
    if b"\0" in data:
        return None
    try:
        return data.decode("utf-8")
    except UnicodeDecodeError:
        return None


def _forbidden_text(text: str) -> bool:
    folded = text.casefold()
    if any(value in folded for value in FORBIDDEN):
        return True
    if any(
        f"{subject} {mechanism}" in folded
        for subject in TRIAL_SUBJECTS
        for mechanism in HANDOFF_MECHANISMS
    ):
        return True
    return False


def _matches(data: bytes) -> bool:
    text = _text(data)
    return text is not None and _forbidden_text(text)


def _path_matches(path: str) -> bool:
    return _forbidden_text(path)


def _allowlist(path: Path = INVENTORY) -> dict[str, str]:
    encoded = json.loads(path.read_text(encoding="utf-8"))
    return {
        base64.b64decode(name).decode("utf-8"): digest
        for name, digest in encoded.items()
    }


def scan_files(root: Path, names: list[str], inventory: Path = INVENTORY) -> list[str]:
    allowed = _allowlist(inventory)
    violations: list[str] = []
    seen_allowed: set[str] = set()
    for name in names:
        data = (root / name).read_bytes()
        if not (_path_matches(name) or _matches(data)):
            continue
        digest = hashlib.sha256(data).hexdigest()
        if allowed.get(name) == digest:
            seen_allowed.add(name)
        else:
            violations.append(name)
    missing = sorted(set(allowed).difference(seen_allowed))
    violations.extend(f"missing-or-renamed:{name}" for name in missing)
    return sorted(violations)


def scan_archive(path: Path) -> list[str]:
    violations: list[str] = []
    with tarfile.open(path, "r:gz") as archive:
        for member in archive.getmembers():
            if not member.isfile():
                continue
            source = archive.extractfile(member)
            if _path_matches(member.name) or (
                source is not None and _matches(source.read())
            ):
                violations.append(member.name)
    return sorted(violations)


def tracked(root: Path) -> list[str]:
    output = subprocess.run(
        ["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard"],
        cwd=root,
        check=True,
        capture_output=True,
    ).stdout
    return [
        value.decode("utf-8")
        for value in output.split(b"\0")
        if value and (root / value.decode("utf-8")).is_file()
    ]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--archive", type=Path)
    parser.add_argument("--inventory", type=Path, default=INVENTORY)
    args = parser.parse_args()
    violations = (
        scan_archive(args.archive)
        if args.archive
        else scan_files(args.root, tracked(args.root), args.inventory)
    )
    if violations:
        print("forbidden coupling:\n" + "\n".join(violations))
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
