#!/usr/bin/env python3
"""Compile one release target once, finalize executables, and package it twice."""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from pathlib import Path

import package as package_artifact
from candidate import canonical_bytes, sha256_file

TARGETS = {
    "x86_64-unknown-linux-gnu": {
        "slug": "linux-x86_64",
        "archive": "biomcp-linux-x86_64.tar.gz",
        "wheel": "manylinux_2_28_x86_64",
        "os": "linux",
    },
    "aarch64-unknown-linux-gnu": {
        "slug": "linux-arm64",
        "archive": "biomcp-linux-arm64.tar.gz",
        "wheel": "manylinux_2_28_aarch64",
        "os": "linux",
    },
    "x86_64-apple-darwin": {
        "slug": "macos-x86_64",
        "archive": "biomcp-darwin-x86_64.tar.gz",
        "wheel": "macosx_14_0_x86_64",
        "os": "macos",
    },
    "aarch64-apple-darwin": {
        "slug": "macos-arm64",
        "archive": "biomcp-darwin-arm64.tar.gz",
        "wheel": "macosx_14_0_arm64",
        "os": "macos",
    },
    "x86_64-pc-windows-msvc": {
        "slug": "windows-x86_64",
        "archive": "biomcp-windows-x86_64.zip",
        "wheel": "win_amd64",
        "os": "windows",
    },
}


class BuildError(ValueError):
    pass


def _run(arguments: list[str], *, env: dict[str, str] | None = None) -> str:
    result = subprocess.run(arguments, text=True, capture_output=True, env=env, check=False)
    if result.returncode:
        raise BuildError(result.stderr.strip() or f"command failed: {arguments[0]}")
    return result.stdout


def max_glibc_version(readelf_output: str) -> tuple[int, int]:
    versions = [
        tuple(map(int, match.groups()))
        for match in re.finditer(r"GLIBC_([0-9]+)\.([0-9]+)", readelf_output)
    ]
    return max(versions, default=(0, 0))


def platform_evidence(binary: Path, target: str) -> dict[str, object]:
    settings = TARGETS[target]
    if settings["os"] == "linux":
        output = _run(["readelf", "--version-info", str(binary)])
        maximum = max_glibc_version(output)
        if maximum > (2, 28):
            raise BuildError(f"binary imports GLIBC_{maximum[0]}.{maximum[1]} above 2.28")
        return {"glibc_max": f"{maximum[0]}.{maximum[1]}", "glibc_floor_checked": True}
    if settings["os"] == "macos":
        output = _run(["otool", "-l", str(binary)])
        versions = [
            tuple(map(int, match.groups()))
            for match in re.finditer(r"\bminos\s+([0-9]+)\.([0-9]+)", output)
        ]
        if not versions or max(versions) > (14, 0):
            raise BuildError("Mach-O deployment target is absent or above macOS 14.0")
        architectures = _run(["lipo", "-archs", str(binary)]).strip().split()
        expected = "x86_64" if target.startswith("x86_64") else "arm64"
        if architectures != [expected]:
            raise BuildError(f"unexpected Mach-O architectures: {architectures}")
        return {"deployment_target": "14.0", "architectures": architectures}
    headers = _run(["dumpbin", "/headers", str(binary)])
    imports = _run(["dumpbin", "/imports", str(binary)])
    if "machine (x64)" not in headers.lower() or not imports.strip():
        raise BuildError("Windows PE header/import inspection failed")
    return {
        "windows_client_floor": "10",
        "windows_server_floor": "2016",
        "pe_headers_checked": True,
        "pe_imports_checked": True,
    }


def sbom(lockfile: Path, output: Path, source_sha: str, version: str) -> str:
    packages = []
    current: dict[str, str] = {}
    for line in lockfile.read_text(encoding="utf-8").splitlines():
        if line == "[[package]]":
            if {"name", "version"} <= current.keys():
                packages.append(current)
            current = {}
        elif match := re.match(r'^(name|version|source) = "([^"]+)"$', line):
            current[match.group(1)] = match.group(2)
    if {"name", "version"} <= current.keys():
        packages.append(current)
    value = {
        "bomFormat": "CycloneDX",
        "specVersion": "1.6",
        "version": 1,
        "metadata": {"component": {"name": "biomcp-cli", "version": version}, "source_sha": source_sha},
        "components": sorted(packages, key=lambda item: (item["name"], item["version"])),
    }
    output.write_bytes(canonical_bytes(value))
    return sha256_file(output)


def finalize(
    source: Path,
    output: Path,
    evidence: Path,
    target_slug: str,
    source_sha: str,
    version: str,
    repo: Path,
) -> dict[str, object]:
    command = [
        sys.executable,
        str(repo / "release/signing.py"),
        "--repo",
        str(repo),
        "--source",
        str(source),
        "--output",
        str(output),
        "--evidence",
        str(evidence),
        "--target",
        target_slug,
        "--source-sha",
        source_sha,
        "--version",
        version,
        "--unsigned-sha256",
        sha256_file(source),
    ]
    _run(command)
    return json.loads(evidence.read_text(encoding="utf-8"))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", choices=sorted(TARGETS), required=True)
    parser.add_argument("--source-sha", required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--run-id", required=True)
    parser.add_argument("--repo", type=Path, default=Path.cwd())
    parser.add_argument("--dist", type=Path, required=True)
    parser.add_argument("--skip-build", action="store_true", help=argparse.SUPPRESS)
    args = parser.parse_args()
    settings = TARGETS[args.target]
    args.dist.mkdir(parents=True, exist_ok=True)
    environment = os.environ.copy()
    if settings["os"] == "macos":
        environment["MACOSX_DEPLOYMENT_TARGET"] = "14.0"
    if not args.skip_build:
        _run(
            [
                "cargo", "build", "--release", "--locked", "--all-features",
                "--target", args.target, "--bin", "biomcp", "--bin", "biomcp-cli",
            ],
            env=environment,
        )
    suffix = ".exe" if settings["os"] == "windows" else ""
    target_dir = args.repo / "target" / args.target / "release"
    full = target_dir / f"biomcp{suffix}"
    shim = target_dir / f"biomcp-cli{suffix}"
    signing: dict[str, object] = {}
    if settings["os"] != "linux":
        signed_full = args.dist / f"signed-biomcp{suffix}"
        signed_shim = args.dist / f"signed-biomcp-cli{suffix}"
        signing["biomcp"] = finalize(
            full,
            signed_full,
            args.dist / "biomcp-signing.json",
            str(settings["slug"]),
            args.source_sha,
            args.version,
            args.repo,
        )
        signing["biomcp-cli"] = finalize(
            shim,
            signed_shim,
            args.dist / "shim-signing.json",
            str(settings["slug"]),
            args.source_sha,
            args.version,
            args.repo,
        )
        full, shim = signed_full, signed_shim
    floor = platform_evidence(full, args.target)
    sbom_hash = sbom(args.repo / "Cargo.lock", args.dist / "sbom.cdx.json", args.source_sha, args.version)
    native_path = args.dist / str(settings["archive"])
    wheel_path = args.dist / f"biomcp_cli-{args.version}-py3-none-{settings['wheel']}.whl"
    package_artifact.native_archive(full, native_path, settings["os"] == "windows")
    package_artifact.wheel(
        full, shim, wheel_path, args.version, f"py3-none-{settings['wheel']}", settings["os"] == "windows"
    )
    for kind, artifact_id, artifact_path in (
        ("native", f"native-{settings['slug']}", native_path),
        ("wheel", f"wheel-{settings['slug']}", wheel_path),
    ):
        evidence = {
            "inspected": True,
            "platform": floor,
            "sbom_sha256": sbom_hash,
            "binary_sha256": sha256_file(full),
            "shim_sha256": sha256_file(shim) if kind == "wheel" else None,
            "signing": signing,
        }
        record = package_artifact.record(
            artifact_id, artifact_path, args.source_sha, args.version, args.run_id, evidence
        )
        record["provenance"].update(
            {
                "target": args.target,
                "rust": os.environ.get("RUST_TOOLCHAIN", "unknown"),
                "source_sha": args.source_sha,
            }
        )
        (args.dist / f"{kind}.json").write_bytes(canonical_bytes(record))
    if settings["os"] != "linux":
        full.unlink()
        shim.unlink()
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (BuildError, OSError, json.JSONDecodeError) as error:
        print(f"release build: {error}", file=sys.stderr)
        raise SystemExit(2) from error
