#!/usr/bin/env python3
"""Inspect staged native archives and binary wheels before registration."""

from __future__ import annotations

import argparse
import json
import subprocess
import tarfile
import zipfile
from pathlib import Path, PurePosixPath

from candidate import canonical_bytes, sha256_file

FORBIDDEN = ("testdata/", "tests/", "spec/fixtures/", ".git/", ".cache/", "sdlc/")


class InspectionError(ValueError):
    pass


def _safe_name(name: str) -> None:
    path = PurePosixPath(name)
    if path.is_absolute() or ".." in path.parts or "\\" in name:
        raise InspectionError(f"unsafe archive member: {name}")
    if any(marker in name for marker in FORBIDDEN):
        raise InspectionError(f"forbidden archive member: {name}")


def inspect_native(path: Path, windows: bool) -> dict[str, object]:
    expected = "biomcp.exe" if windows else "biomcp"
    if windows:
        with zipfile.ZipFile(path) as archive:
            names = archive.namelist()
            for name in names:
                _safe_name(name)
            if names != [expected] or not archive.read(expected):
                raise InspectionError("native archive must contain exactly biomcp.exe")
    else:
        with tarfile.open(path, "r:gz") as archive:
            members = archive.getmembers()
            for member in members:
                _safe_name(member.name)
            if len(members) != 1 or members[0].name != expected or not members[0].isfile():
                raise InspectionError("native archive must contain exactly biomcp")
            if members[0].mode & 0o111 == 0:
                raise InspectionError("native executable lacks execute permission")
    return {"archive_members": 1, "executable_count": 1, "inspected": True}


def inspect_wheel(path: Path, windows: bool) -> dict[str, object]:
    suffix = ".exe" if windows else ""
    with zipfile.ZipFile(path) as archive:
        names = archive.namelist()
        for name in names:
            _safe_name(name)
        scripts = sorted(name for name in names if ".data/scripts/" in name)
        expected = [
            next(name for name in scripts if name.endswith(f"/biomcp-cli{suffix}")),
            next(name for name in scripts if name.endswith(f"/biomcp{suffix}")),
        ]
        if scripts != sorted(expected) or len(scripts) != 2:
            raise InspectionError("wheel must contain exactly biomcp and its compatibility shim")
        if not any(name.endswith(".dist-info/RECORD") for name in names):
            raise InspectionError("wheel lacks RECORD")
        full = archive.read(next(name for name in scripts if name.endswith(f"/biomcp{suffix}")))
        shim = archive.read(next(name for name in scripts if name.endswith(f"/biomcp-cli{suffix}")))
        if not full or not shim or len(shim) >= len(full):
            raise InspectionError("wheel compatibility executable is not a small shim")
    return {
        "archive_members": len(names),
        "executable_count": 2,
        "shim_is_smaller": True,
        "inspected": True,
    }


def smoke(binary: Path, source_sha: str, version: str) -> dict[str, object]:
    probes = [
        (["--version"], True),
        (["--help"], True),
        (["--json", "list"], True),
        (["--json", "not-a-command"], False),
    ]
    for arguments, success in probes:
        result = subprocess.run([binary, *arguments], capture_output=True, text=True, check=False)
        if (result.returncode == 0) != success:
            raise InspectionError(f"smoke failed: {' '.join(arguments)}")
        if arguments[:1] == ["--json"]:
            json.loads(result.stdout)
    version_result = subprocess.run(
        [binary, "--json", "version"], capture_output=True, text=True, check=True
    )
    identity = json.loads(version_result.stdout)
    serialized = json.dumps(identity)
    if version not in serialized or source_sha[:8] not in serialized:
        raise InspectionError("binary identity does not match candidate")
    return {"version_help_json_smoke": True, "binary_sha256": sha256_file(binary)}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--kind", choices=["native", "wheel"], required=True)
    parser.add_argument("--artifact", type=Path, required=True)
    parser.add_argument("--record", type=Path, required=True)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--source-sha", required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--windows", action="store_true")
    args = parser.parse_args()
    record = json.loads(args.record.read_text(encoding="utf-8"))
    if record["sha256"] != sha256_file(args.artifact):
        raise SystemExit("inspection refused changed artifact")
    evidence = (
        inspect_native(args.artifact, args.windows)
        if args.kind == "native"
        else inspect_wheel(args.artifact, args.windows)
    )
    evidence.update(smoke(args.binary, args.source_sha, args.version))
    record["evidence"] = evidence
    args.record.write_bytes(canonical_bytes(record))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
