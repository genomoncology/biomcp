#!/usr/bin/env python3
"""Fail-closed release candidate manifest operations.

This module never publishes. It binds private candidate bytes to one committed
version, full source SHA, and GitHub stage run before promotion exists.
"""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any

SHA_RE = re.compile(r"^[0-9a-f]{40}$")
NUMERIC_COMPONENT = r"(?:0|[1-9][0-9]*)"
STABLE_VERSION_RE = re.compile(
    rf"^(?P<base>{NUMERIC_COMPONENT}\.{NUMERIC_COMPONENT}\.{NUMERIC_COMPONENT})$"
)
DEVELOPMENT_VERSION_RE = re.compile(
    rf"^(?P<base>{NUMERIC_COMPONENT}\.{NUMERIC_COMPONENT}\.{NUMERIC_COMPONENT})"
    r"-dev\.(?P<ordinal>[1-9][0-9]*)$"
)
RUN_RE = re.compile(r"^[1-9][0-9]*$")
HASH_RE = re.compile(r"^[0-9a-f]{64}$")

ARTIFACTS: dict[str, tuple[str, str]] = {
    "native-linux-x86_64": ("native", "x86_64-unknown-linux-gnu"),
    "wheel-linux-x86_64": ("wheel", "x86_64-unknown-linux-gnu"),
    "native-linux-arm64": ("native", "aarch64-unknown-linux-gnu"),
    "wheel-linux-arm64": ("wheel", "aarch64-unknown-linux-gnu"),
    "native-macos-x86_64": ("native", "x86_64-apple-darwin"),
    "wheel-macos-x86_64": ("wheel", "x86_64-apple-darwin"),
    "native-macos-arm64": ("native", "aarch64-apple-darwin"),
    "wheel-macos-arm64": ("wheel", "aarch64-apple-darwin"),
    "native-windows-x86_64": ("native", "x86_64-pc-windows-msvc"),
    "wheel-windows-x86_64": ("wheel", "x86_64-pc-windows-msvc"),
    "oci-index": ("oci", "linux/amd64,linux/arm64"),
    "homebrew-formula": ("homebrew", "macos"),
    "mcpb": ("mcpb", "darwin-universal,win32-x86_64"),
}

BASELINE_ARTIFACTS = {"native-linux-x86_64", "wheel-linux-x86_64"}
PLATFORM_ARTIFACTS = {
    artifact_id
    for artifact_id, (kind, _) in ARTIFACTS.items()
    if kind in {"native", "wheel"}
}
CONTAINER_ARTIFACTS = PLATFORM_ARTIFACTS | {"oci-index"}
DELIVERY_ARTIFACTS = CONTAINER_ARTIFACTS | {"homebrew-formula"}
FINAL_ARTIFACTS = set(ARTIFACTS)
REQUIRED_GATES = {"lint", "test", "spec", "full-feature-check"}


class CandidateError(ValueError):
    pass


def canonical_bytes(value: Any) -> bytes:
    return (json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n").encode()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _git(repo: Path, *args: str) -> str:
    result = subprocess.run(
        ["git", *args], cwd=repo, text=True, capture_output=True, check=False
    )
    if result.returncode:
        raise CandidateError(result.stderr.strip() or f"git {' '.join(args)} failed")
    return result.stdout.strip()


def _manifest_version(repo: Path, sha: str, path: str) -> str:
    text = _git(repo, "show", f"{sha}:{path}")
    match = re.search(r'^version\s*=\s*"([^"]+)"', text, re.MULTILINE)
    if not match:
        raise CandidateError(f"candidate commit has no version in {path}")
    return match.group(1)


def candidate_kind(version: str, python_version: str) -> str:
    stable = STABLE_VERSION_RE.fullmatch(version)
    if stable and python_version == version:
        return "release"
    development = DEVELOPMENT_VERSION_RE.fullmatch(version)
    if development:
        expected_python = (
            f"{development.group('base')}.dev{development.group('ordinal')}"
        )
        if python_version == expected_python:
            return "development"
    raise CandidateError("candidate version pair is invalid or non-canonical")


def committed_versions(repo: Path, sha: str) -> tuple[str, str, str]:
    version = _manifest_version(repo, sha, "Cargo.toml")
    python_version = _manifest_version(repo, sha, "pyproject.toml")
    return version, python_version, candidate_kind(version, python_version)


def _atomic_json(path: Path, value: Any) -> None:
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


def load_manifest(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise CandidateError(f"cannot read candidate manifest: {error}") from error
    validate_manifest(value)
    return value


def validate_manifest(value: Any) -> None:
    if not isinstance(value, dict) or value.get("schema_version") != 2:
        raise CandidateError("unsupported candidate manifest schema")
    if not SHA_RE.fullmatch(str(value.get("source_sha", ""))):
        raise CandidateError("candidate manifest requires a full lowercase source SHA")
    version = str(value.get("version", ""))
    python_version = str(value.get("python_version", ""))
    kind = candidate_kind(version, python_version)
    if value.get("candidate_kind") != kind:
        raise CandidateError("candidate kind does not match its version pair")
    if not RUN_RE.fullmatch(str(value.get("stage_run_id", ""))):
        raise CandidateError("candidate manifest requires a numeric stage run ID")
    if value.get("status") not in {"staging", "complete"}:
        raise CandidateError("invalid candidate status")
    if not isinstance(value.get("gates"), dict) or any(
        result != "passed" for result in value["gates"].values()
    ):
        raise CandidateError("candidate gates may record only passed results")
    if not isinstance(value.get("pins"), dict) or not all(value["pins"].values()):
        raise CandidateError("candidate tool pins must be non-empty")
    policy_hash = value.get("signing_policy_sha256")
    if policy_hash is not None and not HASH_RE.fullmatch(str(policy_hash)):
        raise CandidateError("invalid signing policy hash")
    artifacts = value.get("artifacts")
    if not isinstance(artifacts, dict):
        raise CandidateError("candidate artifacts must be an object")
    for artifact_id, record in artifacts.items():
        validate_artifact(value, artifact_id, record)


def validate_artifact(manifest: dict[str, Any], artifact_id: str, record: Any) -> None:
    if artifact_id not in ARTIFACTS or not isinstance(record, dict):
        raise CandidateError(f"unregistered artifact: {artifact_id}")
    kind, target = ARTIFACTS[artifact_id]
    expected = {
        "id": artifact_id,
        "kind": kind,
        "target": target,
        "source_sha": manifest["source_sha"],
        "version": manifest["version"],
        "stage_run_id": manifest["stage_run_id"],
    }
    for key, expected_value in expected.items():
        if record.get(key) != expected_value:
            raise CandidateError(f"artifact {artifact_id} has wrong {key}")
    filename = record.get("filename")
    if not isinstance(filename, str) or not filename or Path(filename).name != filename:
        raise CandidateError(f"artifact {artifact_id} has unsafe filename")
    if not HASH_RE.fullmatch(str(record.get("sha256", ""))):
        raise CandidateError(f"artifact {artifact_id} has invalid SHA-256")
    if not isinstance(record.get("bytes"), int) or record["bytes"] < 1:
        raise CandidateError(f"artifact {artifact_id} has invalid size")
    if not isinstance(record.get("provenance"), dict) or not isinstance(
        record.get("evidence"), dict
    ):
        raise CandidateError(f"artifact {artifact_id} lacks evidence")
    if (
        kind == "wheel"
        and record["evidence"].get("python_version") != manifest["python_version"]
    ):
        raise CandidateError(f"artifact {artifact_id} has wrong python_version")
    if record["evidence"].get("fixture_only"):
        raise CandidateError(f"artifact {artifact_id} uses fixture-only evidence")
    if artifact_id == "mcpb":
        expected_status = (
            "signed" if manifest["candidate_kind"] == "release" else "unsigned-development"
        )
        expected_non_promotable = manifest["candidate_kind"] == "development"
        if (
            record["evidence"].get("outer_signature_status") != expected_status
            or record["evidence"].get("non_promotable")
            is not expected_non_promotable
        ):
            raise CandidateError("MCPB evidence does not match candidate kind")
    upstream = record.get("upstream", {})
    if not isinstance(upstream, dict) or any(
        key not in manifest["artifacts"]
        or manifest["artifacts"][key]["sha256"] != digest
        for key, digest in upstream.items()
    ):
        raise CandidateError(f"artifact {artifact_id} has invalid upstream hashes")


def init_manifest(
    repo: Path,
    sha: str,
    run_id: str,
    pins: dict[str, str],
    *,
    require_main: bool = True,
) -> dict[str, Any]:
    if not SHA_RE.fullmatch(sha) or _git(repo, "rev-parse", sha) != sha:
        raise CandidateError("stage requires an existing full commit SHA")
    if require_main:
        _git(repo, "merge-base", "--is-ancestor", sha, "origin/main")
    if not RUN_RE.fullmatch(run_id):
        raise CandidateError("stage run ID must be a positive integer")
    if _git(repo, "status", "--porcelain"):
        raise CandidateError("stage requires a clean checkout")
    version, python_version, kind = committed_versions(repo, sha)
    if _git(repo, "tag", "--list", f"v{version}"):
        raise CandidateError(f"v{version} is already tagged")
    if not pins or any(not key or not value for key, value in pins.items()):
        raise CandidateError("stage requires non-empty tool and action pins")
    return {
        "schema_version": 2,
        "source_sha": sha,
        "version": version,
        "python_version": python_version,
        "candidate_kind": kind,
        "stage_run_id": run_id,
        "status": "staging",
        "created_at": dt.datetime.now(dt.timezone.utc).isoformat().replace("+00:00", "Z"),
        "gates": {},
        "pins": dict(sorted(pins.items())),
        "signing_policy_sha256": None,
        "artifacts": {},
    }


def register_artifact(
    manifest: dict[str, Any], record: dict[str, Any], artifact_path: Path
) -> None:
    artifact_id = str(record.get("id", ""))
    if artifact_id in manifest["artifacts"]:
        if manifest["artifacts"][artifact_id] == record:
            return
        raise CandidateError(f"artifact conflict: {artifact_id}")
    if not artifact_path.is_file():
        raise CandidateError(f"missing artifact bytes: {artifact_path}")
    if record.get("filename") != artifact_path.name:
        raise CandidateError("artifact filename does not match registered bytes")
    if record.get("sha256") != sha256_file(artifact_path):
        raise CandidateError("artifact SHA-256 does not match registered bytes")
    if record.get("bytes") != artifact_path.stat().st_size:
        raise CandidateError("artifact size does not match registered bytes")
    candidate = {**manifest, "artifacts": {**manifest["artifacts"], artifact_id: record}}
    validate_artifact(candidate, artifact_id, record)
    manifest["artifacts"][artifact_id] = record


def finalize_manifest(manifest: dict[str, Any], required: set[str]) -> None:
    missing_gates = REQUIRED_GATES - set(manifest["gates"])
    if missing_gates:
        raise CandidateError(f"missing candidate gates: {', '.join(sorted(missing_gates))}")
    actual = set(manifest["artifacts"])
    if actual != required:
        raise CandidateError(
            f"candidate artifact set mismatch: missing={sorted(required - actual)} "
            f"unexpected={sorted(actual - required)}"
        )
    manifest["status"] = "complete"
    validate_manifest(manifest)


def _load_json_argument(value: str) -> Any:
    path = Path(value)
    text = path.read_text(encoding="utf-8") if path.is_file() else value
    return json.loads(text)


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    init = commands.add_parser("init")
    init.add_argument("--repo", type=Path, default=Path.cwd())
    init.add_argument("--sha", required=True)
    init.add_argument("--run-id", required=True)
    init.add_argument("--pins", required=True, help="JSON object or JSON file")
    init.add_argument("--output", type=Path, required=True)
    init.add_argument("--allow-non-main", action="store_true", help=argparse.SUPPRESS)
    gate = commands.add_parser("record-gate")
    gate.add_argument("--manifest", type=Path, required=True)
    gate.add_argument("--name", choices=sorted(REQUIRED_GATES), required=True)
    policy = commands.add_parser("bind-signing-policy")
    policy.add_argument("--manifest", type=Path, required=True)
    policy.add_argument("--policy", type=Path, required=True)
    register = commands.add_parser("register")
    register.add_argument("--manifest", type=Path, required=True)
    register.add_argument("--record", required=True, help="JSON object or JSON file")
    register.add_argument("--artifact", type=Path, required=True)
    verify = commands.add_parser("verify")
    verify.add_argument("--manifest", type=Path, required=True)
    finalize = commands.add_parser("finalize")
    finalize.add_argument("--manifest", type=Path, required=True)
    finalize.add_argument(
        "--set",
        choices=["baseline", "platforms", "container", "delivery", "final"],
        default="final",
    )
    checksum = commands.add_parser("checksum")
    checksum.add_argument("--manifest", type=Path, required=True)
    return parser


def main(argv: list[str] | None = None) -> int:
    args = _parser().parse_args(argv)
    try:
        if args.command == "init":
            pins = _load_json_argument(args.pins)
            if not isinstance(pins, dict):
                raise CandidateError("pins must be a JSON object")
            manifest = init_manifest(
                args.repo.resolve(),
                args.sha,
                args.run_id,
                pins,
                require_main=not args.allow_non_main,
            )
            _atomic_json(args.output, manifest)
        elif args.command == "record-gate":
            manifest = load_manifest(args.manifest)
            if manifest["status"] != "staging":
                raise CandidateError("complete candidate cannot be changed")
            manifest["gates"][args.name] = "passed"
            _atomic_json(args.manifest, manifest)
        elif args.command == "bind-signing-policy":
            manifest = load_manifest(args.manifest)
            if manifest["status"] != "staging":
                raise CandidateError("complete candidate cannot be changed")
            manifest["signing_policy_sha256"] = sha256_file(args.policy)
            _atomic_json(args.manifest, manifest)
        elif args.command == "register":
            manifest = load_manifest(args.manifest)
            if manifest["status"] != "staging":
                raise CandidateError("complete candidate cannot be changed")
            record = _load_json_argument(args.record)
            if not isinstance(record, dict):
                raise CandidateError("artifact record must be a JSON object")
            register_artifact(manifest, record, args.artifact)
            _atomic_json(args.manifest, manifest)
        elif args.command == "verify":
            load_manifest(args.manifest)
        elif args.command == "finalize":
            manifest = load_manifest(args.manifest)
            required = {
                "baseline": BASELINE_ARTIFACTS,
                "platforms": PLATFORM_ARTIFACTS,
                "container": CONTAINER_ARTIFACTS,
                "delivery": DELIVERY_ARTIFACTS,
                "final": FINAL_ARTIFACTS,
            }[args.set]
            finalize_manifest(manifest, required)
            _atomic_json(args.manifest, manifest)
        elif args.command == "checksum":
            load_manifest(args.manifest)
            print(f"{sha256_file(args.manifest)}  {args.manifest.name}")
        return 0
    except (CandidateError, json.JSONDecodeError) as error:
        print(f"candidate: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
