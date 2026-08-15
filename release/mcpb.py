#!/usr/bin/env python3
"""Prepare and inspect the BioMCP desktop bundle."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import stat
import sys
import tempfile
import zipfile
from pathlib import Path, PurePosixPath
from typing import Any

from candidate import (
    ARTIFACTS,
    HASH_RE,
    REQUIRED_GATES,
    CandidateError,
    canonical_bytes,
    load_manifest,
    sha256_file,
)
from signing import SigningError, load_policy

EXPECTED_MEMBERS = {"manifest.json", "server/biomcp", "server/biomcp.exe"}


class McpbError(ValueError):
    pass


def render_manifest(template: dict[str, Any], version: str) -> dict[str, Any]:
    manifest = json.loads(json.dumps(template))
    if manifest.get("manifest_version") != "0.3":
        raise McpbError("MCPB manifest must use schema 0.3")
    manifest["version"] = version
    server = manifest.get("server", {})
    config = server.get("mcp_config", {})
    if server.get("type") != "binary" or server.get("entry_point") != "server/biomcp":
        raise McpbError("MCPB default server must be the bundled macOS executable")
    if config.get("command") != "server/biomcp" or config.get("args") != ["serve"]:
        raise McpbError("MCPB default command must launch biomcp serve")
    override = config.get("platform_overrides", {}).get("win32", {})
    if override != {"command": "server/biomcp.exe"}:
        raise McpbError("MCPB Windows command override is not exact")
    if manifest.get("compatibility", {}).get("platforms") != ["darwin", "win32"]:
        raise McpbError("MCPB may advertise only macOS and Windows")
    tools = manifest.get("tools")
    if not isinstance(tools, list) or len(tools) != 7 or len({tool["name"] for tool in tools}) != 7:
        raise McpbError("MCPB manifest must carry the authoritative seven-tool catalog")
    return manifest


def prepare(
    template: Path, version: str, macos: Path, windows: Path, output: Path
) -> None:
    if output.exists():
        raise McpbError("MCPB preparation output already exists")
    (output / "server").mkdir(parents=True)
    manifest = render_manifest(json.loads(template.read_text(encoding="utf-8")), version)
    (output / "manifest.json").write_bytes(canonical_bytes(manifest))
    shutil.copyfile(macos, output / "server/biomcp")
    shutil.copyfile(windows, output / "server/biomcp.exe")
    os.chmod(output / "server/biomcp", 0o755)
    os.chmod(output / "server/biomcp.exe", 0o755)


def inspect_bundle(
    bundle: Path, macos_sha256: str, windows_sha256: str, version: str
) -> dict[str, Any]:
    with zipfile.ZipFile(bundle) as archive:
        names = set(archive.namelist())
        for name in names:
            path = PurePosixPath(name)
            if path.is_absolute() or ".." in path.parts or "\\" in name:
                raise McpbError(f"unsafe MCPB member: {name}")
        if names != EXPECTED_MEMBERS:
            raise McpbError(f"unexpected MCPB members: {sorted(names ^ EXPECTED_MEMBERS)}")
        manifest = json.loads(archive.read("manifest.json"))
        render_manifest(manifest, version)
        macos = archive.read("server/biomcp")
        windows = archive.read("server/biomcp.exe")
        if sha256_file_bytes(macos) != macos_sha256 or sha256_file_bytes(windows) != windows_sha256:
            raise McpbError("MCPB executable hash mismatch")
        for name in ("server/biomcp", "server/biomcp.exe"):
            mode = archive.getinfo(name).external_attr >> 16
            if mode & stat.S_IXUSR == 0:
                raise McpbError(f"MCPB executable mode missing: {name}")
    serialized = json.dumps(manifest)
    forbidden = ("/home/", "/Users/", "testdata", "sdlc/", ".cache", "private_key")
    if any(value in serialized for value in forbidden):
        raise McpbError("MCPB manifest contains a forbidden local value")
    return {
        "schema": "0.3",
        "members": sorted(names),
        "tools": [tool["name"] for tool in manifest["tools"]],
        "platforms": ["darwin", "win32"],
        "macos_sha256": macos_sha256,
        "windows_sha256": windows_sha256,
        "inspected": True,
    }


def sha256_file_bytes(data: bytes) -> str:
    import hashlib

    return hashlib.sha256(data).hexdigest()


def _load_record(path: Path, label: str) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise McpbError(f"{label} is absent or malformed") from error
    if not isinstance(value, dict):
        raise McpbError(f"{label} is not an object")
    return value


def _candidate_base(path: Path) -> dict[str, Any]:
    manifest = load_manifest(path)
    if (
        manifest["status"] != "staging"
        or set(manifest["gates"]) != REQUIRED_GATES
        or manifest["artifacts"] != {}
        or not HASH_RE.fullmatch(str(manifest.get("signing_policy_sha256", "")))
    ):
        raise McpbError("MCPB record requires the validated candidate-base manifest")
    return manifest


def _validate_native_record(
    record: dict[str, Any],
    artifact_id: str,
    binary: Path,
    manifest: dict[str, Any],
    policy: dict[str, Any],
    policy_hash: str,
) -> str:
    kind, target = ARTIFACTS[artifact_id]
    expected = {
        "id": artifact_id,
        "kind": kind,
        "target": target,
        "source_sha": manifest["source_sha"],
        "version": manifest["version"],
        "stage_run_id": manifest["stage_run_id"],
    }
    if any(record.get(key) != value for key, value in expected.items()):
        raise McpbError(f"{artifact_id} record identity does not match candidate")
    if not HASH_RE.fullmatch(str(record.get("sha256", ""))):
        raise McpbError(f"{artifact_id} record has an invalid archive hash")
    evidence = record.get("evidence")
    if not isinstance(evidence, dict) or evidence.get("binary_sha256") != sha256_file(binary):
        raise McpbError(f"{artifact_id} record does not bind its executable bytes")
    signing = evidence.get("signing", {}).get("biomcp")
    if not isinstance(signing, dict):
        raise McpbError(f"{artifact_id} lacks native signing evidence")
    slug = "macos-arm64" if artifact_id.endswith("arm64") else (
        "macos-x86_64" if "macos" in artifact_id else "windows-x86_64"
    )
    signed_expected = {
        "schema_version": 1,
        "target": slug,
        "source_sha": manifest["source_sha"],
        "version": manifest["version"],
        "stage_run_id": manifest["stage_run_id"],
        "signed_sha256": sha256_file(binary),
        "signing_policy_sha256": policy_hash,
        "fixture_only": False,
        "timestamp_verified": True,
        "chain_verified": True,
    }
    if any(signing.get(key) != value for key, value in signed_expected.items()):
        raise McpbError(f"{artifact_id} native signing evidence is stale or mismatched")
    if (
        not HASH_RE.fullmatch(str(signing.get("unsigned_sha256", "")))
        or not isinstance(signing.get("signing_job_id"), str)
        or not signing["signing_job_id"]
    ):
        raise McpbError(f"{artifact_id} native signing job identity is absent")
    section = policy["windows"] if "windows" in artifact_id else policy["apple"]
    identity = {
        "certificate_fingerprint": section["leaf_sha256"],
        **(
            {
                "publisher": section["publisher"],
                "timestamp_authority": section["timestamp_url"],
                "timestamp_policy_oid": section["timestamp_policy_oid"],
            }
            if "windows" in artifact_id
            else {
                "team_id": section["team_id"],
                "hardened_runtime": True,
                "notary_status": "Accepted",
                "notary_warnings": [],
            }
        ),
    }
    if any(signing.get(key) != value for key, value in identity.items()):
        raise McpbError(f"{artifact_id} native certificate evidence is invalid")
    if "windows" not in artifact_id:
        if (
            not HASH_RE.fullmatch(str(signing.get("notary_log_sha256", "")))
            or not isinstance(signing.get("notary_submission_id"), str)
            or not signing["notary_submission_id"]
        ):
            raise McpbError(f"{artifact_id} notarization evidence is invalid")
    return str(record["sha256"])


def _validate_universal_signing(
    path: Path,
    macos_hash: str,
    manifest: dict[str, Any],
    policy: dict[str, Any],
    policy_hash: str,
) -> dict[str, Any]:
    signing = _load_record(path, "universal macOS signing evidence")
    expected = {
        "schema_version": 1,
        "target": "macos-universal",
        "source_sha": manifest["source_sha"],
        "version": manifest["version"],
        "stage_run_id": manifest["stage_run_id"],
        "signed_sha256": macos_hash,
        "signing_policy_sha256": policy_hash,
        "certificate_fingerprint": policy["apple"]["leaf_sha256"],
        "team_id": policy["apple"]["team_id"],
        "hardened_runtime": True,
        "timestamp_verified": True,
        "chain_verified": True,
        "notary_status": "Accepted",
        "notary_warnings": [],
        "fixture_only": False,
    }
    if any(signing.get(key) != value for key, value in expected.items()):
        raise McpbError("universal macOS signing evidence is stale or mismatched")
    if (
        not HASH_RE.fullmatch(str(signing.get("unsigned_sha256", "")))
        or not HASH_RE.fullmatch(str(signing.get("notary_log_sha256", "")))
        or not isinstance(signing.get("notary_submission_id"), str)
        or not signing["notary_submission_id"]
        or not isinstance(signing.get("signing_job_id"), str)
        or not signing["signing_job_id"]
    ):
        raise McpbError("universal macOS notarization evidence is incomplete")
    return signing


def _validate_outer_evidence(
    path: Path,
    bundle: Path,
    manifest: dict[str, Any],
    policy: dict[str, Any],
    policy_hash: str,
) -> tuple[dict[str, Any], str, bool]:
    evidence = _load_record(path, "MCPB outer evidence")
    bundle_hash = sha256_file(bundle)
    if manifest["candidate_kind"] == "release":
        mcpb_identity = policy.get("mcpb")
        expected = {
            "schema_version": 1,
            "signed_sha256": bundle_hash,
            "certificate_fingerprint": mcpb_identity["leaf_sha256"],
            "certificate_subject": mcpb_identity["subject"],
            "chain_verified": True,
            "eku": "codeSigning",
            "signing_policy_sha256": policy_hash,
            "source_sha": manifest["source_sha"],
            "version": manifest["version"],
            "python_version": manifest["python_version"],
            "candidate_kind": "release",
            "stage_run_id": manifest["stage_run_id"],
            "fixture_only": False,
        }
        if any(evidence.get(key) != value for key, value in expected.items()):
            raise McpbError("stable MCPB signature evidence is absent or stale")
        if not isinstance(evidence.get("signing_job_id"), str) or not evidence["signing_job_id"]:
            raise McpbError("stable MCPB signing job identity is absent")
        return evidence, "signed", False
    exception = policy["development_unsigned_mcpb"]
    if exception["enabled"] is not True or exception["blocks_promotion"] is not True:
        raise McpbError("unsigned development MCPB exception is disabled")
    expected = {
        "schema_version": 1,
        "evidence_type": "unsigned-development-mcpb",
        "archive_sha256": bundle_hash,
        "source_sha": manifest["source_sha"],
        "version": manifest["version"],
        "python_version": manifest["python_version"],
        "candidate_kind": "development",
        "stage_run_id": manifest["stage_run_id"],
        "signing_policy_sha256": policy_hash,
        "package": exception["package"],
        "tool_version": exception["tool_version"],
        "exception_reason": exception["reason"],
        "outer_signature_status": "unsigned-development",
        "non_promotable": True,
        "fixture_only": False,
    }
    expected_fields = set(expected) | {"github"}
    if set(evidence) != expected_fields or any(
        evidence.get(key) != value for key, value in expected.items()
    ):
        raise McpbError("unsigned development MCPB attestation is absent or stale")
    github = evidence.get("github")
    if (
        not isinstance(github, dict)
        or github.get("repository") != "genomoncology/biomcp"
        or github.get("workflow_ref")
        != "genomoncology/biomcp/.github/workflows/release.yml@refs/heads/main"
        or github.get("job") != "mcpb-artifact"
        or github.get("run_id") != manifest["stage_run_id"]
        or not str(github.get("run_attempt", "")).isdigit()
        or int(github["run_attempt"]) < 1
        or github.get("source_sha") != manifest["source_sha"]
    ):
        raise McpbError("unsigned development MCPB job context is invalid")
    return evidence, "unsigned-development", True


def _atomic_record(path: Path, record: dict[str, Any]) -> None:
    if path.exists():
        raise McpbError("refusing to replace an existing MCPB record")
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    try:
        with os.fdopen(descriptor, "wb") as handle:
            handle.write(canonical_bytes(record))
            handle.flush()
            os.fsync(handle.fileno())
        os.link(temporary, path)
        os.unlink(temporary)
    finally:
        if os.path.exists(temporary):
            os.unlink(temporary)


def record_bundle(
    *,
    bundle: Path,
    record_path: Path,
    manifest_path: Path,
    policy_path: Path,
    outer_evidence_path: Path,
    universal_signing_path: Path,
    macos_arm_record_path: Path,
    macos_intel_record_path: Path,
    windows_record_path: Path,
    macos_arm_binary: Path,
    macos_intel_binary: Path,
    windows_binary: Path,
) -> dict[str, Any]:
    manifest = _candidate_base(manifest_path)
    policy, policy_hash = load_policy(
        policy_path,
        fixture=False,
        require_mcpb=manifest["candidate_kind"] == "release",
    )
    if manifest["signing_policy_sha256"] != policy_hash:
        raise McpbError("candidate signing policy hash does not match policy bytes")
    try:
        with zipfile.ZipFile(bundle) as archive:
            macos_hash = sha256_file_bytes(archive.read("server/biomcp"))
            windows_hash = sha256_file_bytes(archive.read("server/biomcp.exe"))
    except (OSError, KeyError, zipfile.BadZipFile) as error:
        raise McpbError("MCPB archive is absent or malformed") from error
    bundle_evidence = inspect_bundle(
        bundle, macos_hash, windows_hash, manifest["version"]
    )
    universal = _validate_universal_signing(
        universal_signing_path, macos_hash, manifest, policy, policy_hash
    )
    arm = _load_record(macos_arm_record_path, "macOS arm64 native record")
    intel = _load_record(macos_intel_record_path, "macOS x86_64 native record")
    windows = _load_record(windows_record_path, "Windows native record")
    upstream = {
        "native-macos-arm64": _validate_native_record(
            arm,
            "native-macos-arm64",
            macos_arm_binary,
            manifest,
            policy,
            policy_hash,
        ),
        "native-macos-x86_64": _validate_native_record(
            intel,
            "native-macos-x86_64",
            macos_intel_binary,
            manifest,
            policy,
            policy_hash,
        ),
        "native-windows-x86_64": _validate_native_record(
            windows,
            "native-windows-x86_64",
            windows_binary,
            manifest,
            policy,
            policy_hash,
        ),
    }
    if windows_hash != sha256_file(windows_binary):
        raise McpbError("bundled Windows executable differs from its native record")
    outer, status, non_promotable = _validate_outer_evidence(
        outer_evidence_path, bundle, manifest, policy, policy_hash
    )
    evidence = {
        **bundle_evidence,
        "universal_macos_signing": universal,
        "outer": outer,
        "outer_signature_status": status,
        "non_promotable": non_promotable,
    }
    kind, target = ARTIFACTS["mcpb"]
    record = {
        "id": "mcpb",
        "kind": kind,
        "target": target,
        "filename": bundle.name,
        "sha256": sha256_file(bundle),
        "bytes": bundle.stat().st_size,
        "source_sha": manifest["source_sha"],
        "version": manifest["version"],
        "stage_run_id": manifest["stage_run_id"],
        "provenance": {"packer": "@anthropic-ai/mcpb@2.1.2", "build_count": 1},
        "evidence": evidence,
        "upstream": upstream,
    }
    _atomic_record(record_path, record)
    return record


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    prepare_command = commands.add_parser("prepare")
    prepare_command.add_argument("--template", type=Path, default=Path("manifest.json"))
    prepare_command.add_argument("--version", required=True)
    prepare_command.add_argument("--macos", type=Path, required=True)
    prepare_command.add_argument("--windows", type=Path, required=True)
    prepare_command.add_argument("--output", type=Path, required=True)
    record_command = commands.add_parser("record")
    record_command.add_argument("--bundle", type=Path, required=True)
    record_command.add_argument("--record", type=Path, required=True)
    record_command.add_argument("--signing-evidence", type=Path, required=True)
    record_command.add_argument("--manifest", type=Path, required=True)
    record_command.add_argument("--policy", type=Path, default=Path("release/signing-policy.json"))
    record_command.add_argument("--universal-signing-evidence", type=Path, required=True)
    record_command.add_argument("--macos-arm-record", type=Path, required=True)
    record_command.add_argument("--macos-intel-record", type=Path, required=True)
    record_command.add_argument("--windows-record", type=Path, required=True)
    record_command.add_argument("--macos-arm-binary", type=Path, required=True)
    record_command.add_argument("--macos-intel-binary", type=Path, required=True)
    record_command.add_argument("--windows-binary", type=Path, required=True)
    args = parser.parse_args()
    if args.command == "prepare":
        prepare(args.template, args.version, args.macos, args.windows, args.output)
        return 0
    record_bundle(
        bundle=args.bundle,
        record_path=args.record,
        manifest_path=args.manifest,
        policy_path=args.policy,
        outer_evidence_path=args.signing_evidence,
        universal_signing_path=args.universal_signing_evidence,
        macos_arm_record_path=args.macos_arm_record,
        macos_intel_record_path=args.macos_intel_record,
        windows_record_path=args.windows_record,
        macos_arm_binary=args.macos_arm_binary,
        macos_intel_binary=args.macos_intel_binary,
        windows_binary=args.windows_binary,
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (CandidateError, McpbError, OSError, SigningError, json.JSONDecodeError) as error:
        print(f"MCPB: {error}", file=sys.stderr)
        raise SystemExit(2) from error
