#!/usr/bin/env python3
"""Inspect staged native archives and binary wheels before registration."""

from __future__ import annotations

import argparse
import base64
import csv
import hashlib
import io
import json
import os
import re
import subprocess
import sys
import tarfile
import tempfile
import zipfile
from email.parser import BytesParser
from pathlib import Path, PurePosixPath
from typing import Any

from candidate import ARTIFACTS, canonical_bytes, sha256_file

FORBIDDEN = ("testdata/", "tests/", "spec/fixtures/", ".git/", ".cache/", "sdlc/")
TARGETS = {
    "x86_64-unknown-linux-gnu": {
        "os": "linux",
        "slug": "linux-x86_64",
        "archive": "biomcp-linux-x86_64.tar.gz",
        "wheel_tag": "py3-none-manylinux_2_28_x86_64",
    },
    "aarch64-unknown-linux-gnu": {
        "os": "linux",
        "slug": "linux-arm64",
        "archive": "biomcp-linux-arm64.tar.gz",
        "wheel_tag": "py3-none-manylinux_2_28_aarch64",
    },
    "x86_64-apple-darwin": {
        "os": "macos",
        "slug": "macos-x86_64",
        "archive": "biomcp-darwin-x86_64.tar.gz",
        "wheel_tag": "py3-none-macosx_14_0_x86_64",
    },
    "aarch64-apple-darwin": {
        "os": "macos",
        "slug": "macos-arm64",
        "archive": "biomcp-darwin-arm64.tar.gz",
        "wheel_tag": "py3-none-macosx_14_0_arm64",
    },
    "x86_64-pc-windows-msvc": {
        "os": "windows",
        "slug": "windows-x86_64",
        "archive": "biomcp-windows-x86_64.zip",
        "wheel_tag": "py3-none-win_amd64",
    },
}
HASH_RE = re.compile(r"^[0-9a-f]{64}$")


class InspectionError(ValueError):
    pass


def _run(arguments: list[str]) -> str:
    result = subprocess.run(arguments, text=True, capture_output=True, check=False)
    if result.returncode:
        raise InspectionError(
            result.stderr.strip() or f"command failed: {arguments[0]}"
        )
    return result.stdout


def _safe_name(name: str) -> None:
    path = PurePosixPath(name)
    if path.is_absolute() or ".." in path.parts or "\\" in name:
        raise InspectionError(f"unsafe archive member: {name}")
    if any(marker in name for marker in FORBIDDEN):
        raise InspectionError(f"forbidden archive member: {name}")


def max_glibc_version(readelf_output: str) -> tuple[int, int]:
    versions = [
        tuple(map(int, match.groups()))
        for match in re.finditer(r"GLIBC_([0-9]+)\.([0-9]+)", readelf_output)
    ]
    return max(versions, default=(0, 0))


def platform_evidence(binary: Path, target: str) -> dict[str, object]:
    settings = TARGETS[target]
    if settings["os"] == "linux":
        maximum = max_glibc_version(_run(["readelf", "--version-info", str(binary)]))
        if maximum > (2, 28):
            raise InspectionError(
                f"binary imports GLIBC_{maximum[0]}.{maximum[1]} above 2.28"
            )
        return {"glibc_max": f"{maximum[0]}.{maximum[1]}", "glibc_floor_checked": True}
    if settings["os"] == "macos":
        output = _run(["otool", "-l", str(binary)])
        versions = [
            tuple(map(int, match.groups()))
            for match in re.finditer(r"\bminos\s+([0-9]+)\.([0-9]+)", output)
        ]
        if not versions or max(versions) > (14, 0):
            raise InspectionError(
                "Mach-O deployment target is absent or above macOS 14.0"
            )
        architectures = _run(["lipo", "-archs", str(binary)]).strip().split()
        expected = "x86_64" if target.startswith("x86_64") else "arm64"
        if architectures != [expected]:
            raise InspectionError(f"unexpected Mach-O architectures: {architectures}")
        return {"deployment_target": "14.0", "architectures": architectures}
    headers = _run(["dumpbin", "/headers", str(binary)])
    imports = _run(["dumpbin", "/imports", str(binary)])
    if "machine (x64)" not in headers.lower() or not imports.strip():
        raise InspectionError("Windows PE header/import inspection failed")
    return {
        "windows_client_floor": "10",
        "windows_server_floor": "2016",
        "pe_headers_checked": True,
        "pe_imports_checked": True,
    }


def inspect_native(path: Path, target: str) -> dict[str, object]:
    settings = TARGETS[target]
    if path.name != settings["archive"]:
        raise InspectionError("native archive filename does not match target")
    windows = settings["os"] == "windows"
    expected = "biomcp.exe" if windows else "biomcp"
    if windows:
        with zipfile.ZipFile(path) as archive:
            names = archive.namelist()
            if len(names) != len(set(names)):
                raise InspectionError("native archive contains duplicate members")
            for name in names:
                _safe_name(name)
            if names != [expected] or not (binary := archive.read(expected)):
                raise InspectionError("native archive must contain exactly biomcp.exe")
    else:
        with tarfile.open(path, "r:gz") as archive:
            members = archive.getmembers()
            for member in members:
                _safe_name(member.name)
            if (
                len(members) != 1
                or members[0].name != expected
                or not members[0].isfile()
            ):
                raise InspectionError("native archive must contain exactly biomcp")
            if members[0].mode & 0o111 == 0:
                raise InspectionError("native executable lacks execute permission")
            extracted = archive.extractfile(members[0])
            if extracted is None or not (binary := extracted.read()):
                raise InspectionError("native archive contains an empty executable")
    return {
        "archive_members": 1,
        "executable_count": 1,
        "assembled_binary_sha256": hashlib.sha256(binary).hexdigest(),
        "inspected": True,
    }


def _wheel_hash(data: bytes) -> str:
    value = (
        base64.urlsafe_b64encode(hashlib.sha256(data).digest()).rstrip(b"=").decode()
    )
    return f"sha256={value}"


def _one_header(data: bytes, name: str, expected: str) -> None:
    values = BytesParser().parsebytes(data).get_all(name, [])
    if values != [expected]:
        raise InspectionError(f"wheel has wrong {name}")


def inspect_wheel(path: Path, artifact_id: str, version: str) -> dict[str, object]:
    kind, target = ARTIFACTS[artifact_id]
    if kind != "wheel" or target not in TARGETS:
        raise InspectionError("wheel identity is not a registered platform target")
    settings = TARGETS[target]
    tag = settings["wheel_tag"]
    expected_filename = f"biomcp_cli-{version}-{tag}.whl"
    if path.name != expected_filename:
        raise InspectionError("wheel filename does not match candidate identity")
    dist = f"biomcp_cli-{version}"
    suffix = ".exe" if settings["os"] == "windows" else ""
    full_name = f"{dist}.data/scripts/biomcp{suffix}"
    shim_name = f"{dist}.data/scripts/biomcp-cli{suffix}"
    metadata_name = f"{dist}.dist-info/METADATA"
    wheel_name = f"{dist}.dist-info/WHEEL"
    record_name = f"{dist}.dist-info/RECORD"
    expected_names = {full_name, shim_name, metadata_name, wheel_name, record_name}
    with zipfile.ZipFile(path) as archive:
        infos = archive.infolist()
        names = [info.filename for info in infos]
        if len(names) != len(set(names)):
            raise InspectionError("wheel contains duplicate members")
        for name in names:
            _safe_name(name)
        if set(names) != expected_names:
            raise InspectionError("wheel member set does not match candidate identity")
        contents = {name: archive.read(name) for name in names}
        modes = {info.filename: info.external_attr >> 16 for info in infos}
    if modes[full_name] & 0o111 == 0 or modes[shim_name] & 0o111 == 0:
        raise InspectionError("wheel executables lack execute permission")
    _one_header(contents[metadata_name], "Name", "biomcp-cli")
    _one_header(contents[metadata_name], "Version", version)
    _one_header(contents[wheel_name], "Root-Is-Purelib", "false")
    _one_header(contents[wheel_name], "Tag", tag)
    try:
        rows = list(
            csv.reader(io.StringIO(contents[record_name].decode("utf-8"), newline=""))
        )
    except (UnicodeDecodeError, csv.Error) as error:
        raise InspectionError("wheel RECORD is malformed") from error
    if any(len(row) != 3 for row in rows) or len(rows) != len(expected_names):
        raise InspectionError("wheel RECORD row shape is invalid")
    indexed = {row[0]: row[1:] for row in rows}
    if len(indexed) != len(rows) or set(indexed) != expected_names:
        raise InspectionError("wheel RECORD member set is invalid")
    for name in expected_names - {record_name}:
        if indexed[name] != [_wheel_hash(contents[name]), str(len(contents[name]))]:
            raise InspectionError(f"wheel RECORD does not bind {name}")
    if indexed[record_name] != ["", ""]:
        raise InspectionError(
            "wheel RECORD must leave only its own hash and size empty"
        )
    full = contents[full_name]
    shim = contents[shim_name]
    if not full or not shim or len(shim) >= len(full):
        raise InspectionError("wheel compatibility executable is not a small shim")
    return {
        "archive_members": len(names),
        "executable_count": 2,
        "assembled_binary_sha256": hashlib.sha256(full).hexdigest(),
        "assembled_shim_sha256": hashlib.sha256(shim).hexdigest(),
        "shim_is_smaller": True,
        "inspected": True,
    }


def _locked_components(lockfile: Path) -> list[dict[str, str]]:
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
    return sorted(packages, key=lambda item: (item["name"], item["version"]))


def validate_sbom(path: Path, lockfile: Path, source_sha: str, version: str) -> str:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise InspectionError("SBOM is absent or malformed") from error
    expected_metadata = {
        "component": {"name": "biomcp-cli", "version": version},
        "source_sha": source_sha,
    }
    if (
        value.get("bomFormat") != "CycloneDX"
        or value.get("specVersion") != "1.6"
        or value.get("version") != 1
        or value.get("metadata") != expected_metadata
        or value.get("components") != _locked_components(lockfile)
    ):
        raise InspectionError("SBOM identity does not match candidate")
    return sha256_file(path)


def _signing_record(
    path: Path,
    binary: Path,
    unsigned_binary: Path,
    target: str,
    source_sha: str,
    version: str,
    policy: dict[str, Any],
    policy_hash: str,
) -> dict[str, Any]:
    try:
        record = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise InspectionError("signing evidence is absent or malformed") from error
    expected = {
        "schema_version": 1,
        "target": TARGETS[target]["slug"],
        "source_sha": source_sha,
        "version": version,
        "signed_sha256": sha256_file(binary),
        "unsigned_sha256": sha256_file(unsigned_binary),
        "signing_policy_sha256": policy_hash,
        "fixture_only": False,
        "timestamp_verified": True,
        "chain_verified": True,
    }
    if any(record.get(key) != value for key, value in expected.items()):
        raise InspectionError(
            "signing evidence identity or hash does not match artifact"
        )
    if (
        not isinstance(record.get("signing_job_id"), str)
        or not record["signing_job_id"]
    ):
        raise InspectionError("signing evidence lacks its protected job identity")
    if TARGETS[target]["os"] == "macos":
        apple = policy["apple"]
        expected_apple = {
            "certificate_fingerprint": apple["leaf_sha256"],
            "team_id": apple["team_id"],
            "hardened_runtime": True,
            "notary_status": "Accepted",
            "notary_warnings": [],
        }
        if any(record.get(key) != value for key, value in expected_apple.items()):
            raise InspectionError(
                "Apple signing identity or notarization evidence is invalid"
            )
        for name in ("notary_log_sha256",):
            if not HASH_RE.fullmatch(str(record.get(name, ""))):
                raise InspectionError(
                    "Apple signing evidence lacks a valid notary hash"
                )
        _run(["codesign", "--verify", "--strict", "--verbose=4", str(binary)])
    else:
        windows = policy["windows"]
        expected_windows = {
            "certificate_fingerprint": windows["leaf_sha256"],
            "publisher": windows["publisher"],
            "timestamp_authority": windows["timestamp_url"],
            "timestamp_policy_oid": windows["timestamp_policy_oid"],
        }
        if any(record.get(key) != value for key, value in expected_windows.items()):
            raise InspectionError(
                "Windows signing identity or timestamp evidence is invalid"
            )
        _run(["signtool", "verify", "/pa", "/all", "/tw", str(binary)])
    return record


def _validate_signing_policy(policy: dict[str, Any]) -> None:
    if (
        policy.get("schema_version") != 1
        or policy.get("enabled") is not True
        or policy.get("fixture_only") is True
        or policy.get("allowed_notary_warnings") != []
    ):
        raise InspectionError("production signing policy schema is invalid")
    apple = policy.get("apple")
    windows = policy.get("windows")
    mcpb = policy.get("mcpb")
    if not all(isinstance(section, dict) for section in (apple, windows, mcpb)):
        raise InspectionError("production signing policy lacks a release identity")
    if (
        not re.fullmatch(r"[A-Z0-9]{10}", str(apple.get("team_id", "")))
        or not apple.get("identity")
        or not re.fullmatch(r"[0-9A-F]{64}", str(apple.get("leaf_sha256", "")))
        or not apple.get("notary_profile")
        or apple.get("notary_service") != "https://appstoreconnect.apple.com"
        or not isinstance(apple.get("network_destinations"), list)
        or not apple["network_destinations"]
        or any(
            not isinstance(url, str) or not re.fullmatch(r"https://[^/]+", url)
            for url in apple["network_destinations"]
        )
    ):
        raise InspectionError("production Apple signing policy identity is invalid")
    if (
        not windows.get("publisher")
        or not re.fullmatch(r"[0-9A-F]{64}", str(windows.get("leaf_sha256", "")))
        or not str(windows.get("timestamp_url", "")).startswith("https://")
        or not re.fullmatch(
            r"[0-9]+(?:\.[0-9]+)+", str(windows.get("timestamp_policy_oid", ""))
        )
    ):
        raise InspectionError("production Windows signing policy identity is invalid")
    if not mcpb.get("subject") or not re.fullmatch(
        r"[0-9A-F]{64}", str(mcpb.get("leaf_sha256", ""))
    ):
        raise InspectionError("production MCPB signing policy identity is invalid")


def validate_signing(
    *,
    target: str,
    binary: Path,
    shim: Path | None,
    unsigned_binary: Path | None,
    unsigned_shim: Path | None,
    source_sha: str,
    version: str,
    policy_path: Path,
    evidence_paths: dict[str, Path],
) -> dict[str, Any]:
    try:
        raw = policy_path.read_bytes()
        policy = json.loads(raw)
    except (OSError, json.JSONDecodeError) as error:
        raise InspectionError("signing policy is absent or malformed") from error
    if policy.get("schema_version") != 1:
        raise InspectionError("unsupported signing policy schema")
    if TARGETS[target]["os"] == "linux":
        if evidence_paths or unsigned_binary is not None or unsigned_shim is not None:
            raise InspectionError("Linux artifacts must not claim signing evidence")
        return {}
    _validate_signing_policy(policy)
    if unsigned_binary is None or (shim is not None) != (unsigned_shim is not None):
        raise InspectionError("unsigned executable set does not match signed artifacts")
    expected_paths = {"biomcp"} | ({"biomcp-cli"} if shim is not None else set())
    if set(evidence_paths) != expected_paths:
        raise InspectionError(
            "signing evidence set does not match packaged executables"
        )
    policy_hash = hashlib.sha256(raw).hexdigest()
    result = {
        "biomcp": _signing_record(
            evidence_paths["biomcp"],
            binary,
            unsigned_binary,
            target,
            source_sha,
            version,
            policy,
            policy_hash,
        )
    }
    if shim is not None:
        result["biomcp-cli"] = _signing_record(
            evidence_paths["biomcp-cli"],
            shim,
            unsigned_shim,
            target,
            source_sha,
            version,
            policy,
            policy_hash,
        )
    return result


def smoke(binary: Path, source_sha: str, version: str) -> dict[str, object]:
    probes = [
        (["--version"], True),
        (["--help"], True),
        (["--json", "list"], True),
        (["--json", "not-a-command"], False),
    ]
    for arguments, success in probes:
        result = subprocess.run(
            [binary, *arguments], capture_output=True, text=True, check=False
        )
        if (result.returncode == 0) != success:
            raise InspectionError(f"smoke failed: {' '.join(arguments)}")
        if arguments[:1] == ["--json"]:
            json.loads(result.stdout)
    version_result = subprocess.run(
        [binary, "--json", "version"], capture_output=True, text=True, check=True
    )
    identity = json.loads(version_result.stdout)
    revision = source_sha[:8]
    if (
        not isinstance(identity, dict)
        or identity.get("version") != f"{version}+g{revision}"
        or identity.get("git_revision") != revision
    ):
        raise InspectionError("binary identity does not match candidate")
    return {"version_help_json_smoke": True, "binary_sha256": sha256_file(binary)}


def _atomic_record(path: Path, value: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    try:
        with os.fdopen(descriptor, "wb") as handle:
            handle.write(canonical_bytes(value))
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temporary, path)
    finally:
        if os.path.exists(temporary):
            os.unlink(temporary)


def finalize_record(
    *,
    kind: str,
    artifact_id: str,
    artifact: Path,
    binary: Path,
    source_sha: str,
    version: str,
    run_id: str,
    shim: Path | None,
    sbom: Path,
    cargo_lock: Path,
    signing_policy: Path,
    signing_evidence: dict[str, Path],
    provenance: dict[str, object],
    output: Path,
    unsigned_binary: Path | None = None,
    unsigned_shim: Path | None = None,
) -> dict[str, object]:
    """Independently inspect final bytes and atomically write their success record."""
    output.unlink(missing_ok=True)
    target_kind, target = ARTIFACTS[artifact_id]
    if target_kind != kind or target not in TARGETS:
        raise InspectionError("artifact kind does not match registered identity")
    archive_evidence = (
        inspect_native(artifact, target)
        if kind == "native"
        else inspect_wheel(artifact, artifact_id, version)
    )
    if (kind == "wheel") != (shim is not None):
        raise InspectionError("shim path does not match artifact kind")
    binary_hash = sha256_file(binary)
    shim_hash = sha256_file(shim) if shim is not None else None
    if archive_evidence["assembled_binary_sha256"] != binary_hash:
        raise InspectionError(
            "assembled binary hash does not match inspected executable"
        )
    if shim is not None and archive_evidence["assembled_shim_sha256"] != shim_hash:
        raise InspectionError("assembled shim hash does not match inspected executable")
    evidence = {
        **archive_evidence,
        "platform": platform_evidence(binary, target),
        "sbom_sha256": validate_sbom(sbom, cargo_lock, source_sha, version),
        "binary_sha256": binary_hash,
        "signing": validate_signing(
            target=target,
            binary=binary,
            shim=shim,
            unsigned_binary=unsigned_binary,
            unsigned_shim=unsigned_shim,
            source_sha=source_sha,
            version=version,
            policy_path=signing_policy,
            evidence_paths=signing_evidence,
        ),
        **smoke(binary, source_sha, version),
        "artifact_sha256": sha256_file(artifact),
    }
    if shim_hash is not None:
        evidence["shim_sha256"] = shim_hash
    record: dict[str, object] = {
        "id": artifact_id,
        "kind": kind,
        "target": target,
        "filename": artifact.name,
        "sha256": evidence["artifact_sha256"],
        "bytes": artifact.stat().st_size,
        "source_sha": source_sha,
        "version": version,
        "stage_run_id": run_id,
        "provenance": {"builder": "release/package.py", "build_count": 1, **provenance},
        "evidence": evidence,
    }
    _atomic_record(output, record)
    return record


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--kind", choices=["native", "wheel"], required=True)
    parser.add_argument("--artifact-id", choices=sorted(ARTIFACTS), required=True)
    parser.add_argument("--artifact", type=Path, required=True)
    parser.add_argument("--record", type=Path, required=True)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--shim", type=Path)
    parser.add_argument("--sbom", type=Path, required=True)
    parser.add_argument("--cargo-lock", type=Path, required=True)
    parser.add_argument("--signing-policy", type=Path, required=True)
    parser.add_argument("--binary-signing-evidence", type=Path)
    parser.add_argument("--shim-signing-evidence", type=Path)
    parser.add_argument("--unsigned-binary", type=Path)
    parser.add_argument("--unsigned-shim", type=Path)
    parser.add_argument("--source-sha", required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--run-id", required=True)
    parser.add_argument("--provenance", required=True)
    args = parser.parse_args()
    signing_evidence = {}
    if args.binary_signing_evidence is not None:
        signing_evidence["biomcp"] = args.binary_signing_evidence
    if args.shim_signing_evidence is not None:
        signing_evidence["biomcp-cli"] = args.shim_signing_evidence
    try:
        finalize_record(
            kind=args.kind,
            artifact_id=args.artifact_id,
            artifact=args.artifact,
            binary=args.binary,
            source_sha=args.source_sha,
            version=args.version,
            run_id=args.run_id,
            shim=args.shim,
            sbom=args.sbom,
            cargo_lock=args.cargo_lock,
            signing_policy=args.signing_policy,
            signing_evidence=signing_evidence,
            provenance=json.loads(args.provenance),
            output=args.record,
            unsigned_binary=args.unsigned_binary,
            unsigned_shim=args.unsigned_shim,
        )
        return 0
    except (
        InspectionError,
        OSError,
        KeyError,
        json.JSONDecodeError,
        subprocess.SubprocessError,
        tarfile.TarError,
        zipfile.BadZipFile,
    ) as error:
        print(f"inspection: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
