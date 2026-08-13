#!/usr/bin/env python3
"""Prepare and inspect the BioMCP desktop bundle."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import stat
import zipfile
from pathlib import Path, PurePosixPath
from typing import Any

from candidate import ARTIFACTS, canonical_bytes, sha256_file

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
    record_command.add_argument("--source-sha", required=True)
    record_command.add_argument("--version", required=True)
    record_command.add_argument("--run-id", required=True)
    record_command.add_argument("--macos-sha256", required=True)
    record_command.add_argument("--windows-sha256", required=True)
    record_command.add_argument("--macos-arm-record", type=Path, required=True)
    record_command.add_argument("--macos-intel-record", type=Path, required=True)
    record_command.add_argument("--windows-record", type=Path, required=True)
    args = parser.parse_args()
    if args.command == "prepare":
        prepare(args.template, args.version, args.macos, args.windows, args.output)
        return 0
    evidence = inspect_bundle(
        args.bundle, args.macos_sha256, args.windows_sha256, args.version
    )
    signing = json.loads(args.signing_evidence.read_text(encoding="utf-8"))
    if signing.get("fixture_only") or signing.get("signed_sha256") != sha256_file(args.bundle):
        raise McpbError("MCPB production signature evidence is absent or stale")
    evidence["signing"] = signing
    arm = json.loads(args.macos_arm_record.read_text(encoding="utf-8"))
    intel = json.loads(args.macos_intel_record.read_text(encoding="utf-8"))
    windows = json.loads(args.windows_record.read_text(encoding="utf-8"))
    kind, target = ARTIFACTS["mcpb"]
    record = {
        "id": "mcpb",
        "kind": kind,
        "target": target,
        "filename": args.bundle.name,
        "sha256": sha256_file(args.bundle),
        "bytes": args.bundle.stat().st_size,
        "source_sha": args.source_sha,
        "version": args.version,
        "stage_run_id": args.run_id,
        "provenance": {"packer": "@anthropic-ai/mcpb@2.1.2", "build_count": 1},
        "evidence": evidence,
        "upstream": {
            "native-macos-arm64": arm["sha256"],
            "native-macos-x86_64": intel["sha256"],
            "native-windows-x86_64": windows["sha256"],
        },
    }
    args.record.write_bytes(canonical_bytes(record))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
