#!/usr/bin/env python3
"""Fail-closed native signing and notarization boundary."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any

from candidate import canonical_bytes, sha256_file


class SigningError(ValueError):
    pass


POLICY_KEYS = {
    "schema_version",
    "enabled",
    "fixture_only",
    "apple",
    "windows",
    "mcpb",
    "development_unsigned_mcpb",
    "allowed_notary_warnings",
}
EXCEPTION_KEYS = {"enabled", "package", "tool_version", "reason", "blocks_promotion"}
FINGERPRINT_RE = re.compile(r"^[0-9A-F]{64}$")


def _require_keys(value: dict[str, Any], expected: set[str], label: str) -> None:
    if set(value) != expected:
        raise SigningError(f"{label} fields are incomplete or unknown")


def _validate_policy(policy: Any, *, fixture: bool, require_mcpb: bool) -> None:
    if not isinstance(policy, dict) or policy.get("schema_version") != 2:
        raise SigningError("unsupported signing policy schema")
    allowed = POLICY_KEYS if "fixture_only" in policy else POLICY_KEYS - {"fixture_only"}
    _require_keys(policy, allowed, "signing policy")
    if not isinstance(policy.get("enabled"), bool):
        raise SigningError("signing policy enabled flag is invalid")
    if fixture:
        if policy.get("fixture_only") is not True:
            raise SigningError("fixture signing requires a fixture-only policy")
    elif policy.get("fixture_only"):
        raise SigningError("production signing rejects fixture policy")
    if policy.get("allowed_notary_warnings") != []:
        raise SigningError("notary warning allowlist must remain empty")
    exception = policy.get("development_unsigned_mcpb")
    if not isinstance(exception, dict):
        raise SigningError("development unsigned MCPB exception is absent")
    _require_keys(exception, EXCEPTION_KEYS, "development unsigned MCPB exception")
    if (
        not isinstance(exception.get("enabled"), bool)
        or exception.get("package") != "@anthropic-ai/mcpb"
        or exception.get("tool_version") != "2.1.2"
        or not isinstance(exception.get("reason"), str)
        or not exception["reason"].strip()
        or exception.get("blocks_promotion") is not True
    ):
        raise SigningError("development unsigned MCPB exception is invalid")
    if not policy["enabled"]:
        raise SigningError("release signing policy is not provisioned")
    apple = policy.get("apple")
    windows = policy.get("windows")
    if not isinstance(apple, dict) or not isinstance(windows, dict):
        raise SigningError("enabled policy lacks native signing identity")
    _require_keys(
        apple,
        {
            "team_id",
            "identity",
            "leaf_sha256",
            "notary_profile",
            "notary_service",
            "network_destinations",
        },
        "Apple signing identity",
    )
    if (
        not re.fullmatch(r"[A-Z0-9]{10}", str(apple.get("team_id", "")))
        or not isinstance(apple.get("identity"), str)
        or not apple["identity"]
        or not FINGERPRINT_RE.fullmatch(str(apple.get("leaf_sha256", "")))
        or not isinstance(apple.get("notary_profile"), str)
        or not apple["notary_profile"]
        or apple.get("notary_service") != "https://appstoreconnect.apple.com"
        or not isinstance(apple.get("network_destinations"), list)
        or not apple["network_destinations"]
        or any(
            not isinstance(url, str) or not re.fullmatch(r"https://[^/]+", url)
            for url in apple["network_destinations"]
        )
    ):
        raise SigningError("Apple signing identity is invalid")
    _require_keys(
        windows,
        {"publisher", "leaf_sha256", "timestamp_url", "timestamp_policy_oid"},
        "Windows signing identity",
    )
    if (
        not isinstance(windows.get("publisher"), str)
        or not windows["publisher"]
        or not FINGERPRINT_RE.fullmatch(str(windows.get("leaf_sha256", "")))
        or not re.fullmatch(r"https://[^/]+(?:/.*)?", str(windows.get("timestamp_url", "")))
        or not re.fullmatch(
            r"[0-9]+(?:\.[0-9]+)+", str(windows.get("timestamp_policy_oid", ""))
        )
    ):
        raise SigningError("Windows signing identity is invalid")
    mcpb = policy.get("mcpb")
    if require_mcpb and not isinstance(mcpb, dict):
        raise SigningError("enabled policy lacks stable MCPB identity")
    if isinstance(mcpb, dict):
        _require_keys(mcpb, {"subject", "leaf_sha256"}, "MCPB signing identity")
        if (
            not isinstance(mcpb.get("subject"), str)
            or not mcpb["subject"]
            or not FINGERPRINT_RE.fullmatch(str(mcpb.get("leaf_sha256", "")))
        ):
            raise SigningError("MCPB signing identity is invalid")
    elif mcpb is not None:
        raise SigningError("MCPB signing identity is invalid")


def load_policy(
    path: Path, *, fixture: bool, require_mcpb: bool = False
) -> tuple[dict[str, Any], str]:
    raw = path.read_bytes()
    try:
        policy = json.loads(raw)
    except json.JSONDecodeError as error:
        raise SigningError(f"invalid signing policy: {error}") from error
    _validate_policy(policy, fixture=fixture, require_mcpb=require_mcpb)
    return policy, hashlib.sha256(raw).hexdigest()


def verify_protected_policy(
    repo: Path, source_sha: str, policy_path: Path, policy_hash: str
) -> None:
    expected = os.environ.get("BIOMCP_SIGNING_POLICY_SHA256", "")
    if expected != policy_hash:
        raise SigningError("protected signing policy digest mismatch")
    relative = policy_path.resolve().relative_to(repo.resolve()).as_posix()
    result = subprocess.run(
        ["git", "show", f"{source_sha}^:{relative}"],
        cwd=repo,
        capture_output=True,
        check=False,
    )
    if result.returncode or result.stdout != policy_path.read_bytes():
        raise SigningError("release commit changed the protected signing policy")


def _run(arguments: list[str]) -> str:
    result = subprocess.run(arguments, text=True, capture_output=True, check=False)
    if result.returncode:
        raise SigningError(result.stderr.strip() or f"command failed: {arguments[0]}")
    return result.stdout


def _base_evidence(
    target: str,
    source_sha: str,
    version: str,
    run_id: str,
    unsigned_hash: str,
    signed_path: Path,
    policy_hash: str,
    fixture: bool,
) -> dict[str, Any]:
    return {
        "schema_version": 1,
        "target": target,
        "source_sha": source_sha,
        "version": version,
        "stage_run_id": run_id,
        "unsigned_sha256": unsigned_hash,
        "signed_sha256": sha256_file(signed_path),
        "signing_policy_sha256": policy_hash,
        "signing_job_id": os.environ.get("GITHUB_JOB", "local-fixture"),
        "fixture_only": fixture,
    }


def fixture_finalize(
    source: Path,
    output: Path,
    target: str,
    source_sha: str,
    version: str,
    run_id: str,
    policy_hash: str,
    policy: dict[str, Any],
) -> dict[str, Any]:
    data = source.read_bytes()
    marker = canonical_bytes(
        {
            "fixture_signature": hashlib.sha256(data + target.encode()).hexdigest(),
            "target": target,
        }
    )
    output.write_bytes(data + marker)
    section = policy["apple" if target.startswith("macos") else "windows"]
    evidence = _base_evidence(
        target,
        source_sha,
        version,
        run_id,
        hashlib.sha256(data).hexdigest(),
        output,
        policy_hash,
        True,
    )
    evidence.update(
        {
            "certificate_fingerprint": section["leaf_sha256"],
            "timestamp_verified": True,
            "chain_verified": True,
        }
    )
    if target.startswith("macos"):
        evidence.update(
            {
                "team_id": section["team_id"],
                "hardened_runtime": True,
                "notary_status": "Accepted",
                "notary_warnings": [],
                "notary_submission_id": "fixture-submission",
                "notary_log_sha256": "0" * 64,
            }
        )
    else:
        evidence.update(
            {
                "publisher": section["publisher"],
                "timestamp_authority": section["timestamp_url"],
                "timestamp_policy_oid": section["timestamp_policy_oid"],
            }
        )
    return evidence


def production_finalize(
    source: Path,
    output: Path,
    target: str,
    source_sha: str,
    version: str,
    run_id: str,
    policy_hash: str,
    policy: dict[str, Any],
) -> dict[str, Any]:
    unsigned_hash = sha256_file(source)
    shutil.copyfile(source, output)
    if target.startswith("macos"):
        apple = policy["apple"]
        _run(
            [
                "codesign", "--force", "--options", "runtime", "--timestamp",
                "--sign", apple["identity"], str(output),
            ]
        )
        _run(["codesign", "--verify", "--strict", "--verbose=4", str(output)])
        with tempfile.TemporaryDirectory() as directory:
            submission = Path(directory) / "submission.zip"
            _run(["ditto", "-c", "-k", "--keepParent", str(output), str(submission)])
            submitted = json.loads(
                _run(
                    [
                        "xcrun", "notarytool", "submit", str(submission), "--wait",
                        "--output-format", "json", "--keychain-profile", apple["notary_profile"],
                    ]
                )
            )
            submission_id = submitted.get("id")
            if submitted.get("status") != "Accepted" or not submission_id:
                raise SigningError("Apple notarization was not accepted")
            log_text = _run(
                [
                    "xcrun", "notarytool", "log", submission_id,
                    "--keychain-profile", apple["notary_profile"],
                ]
            )
            log = json.loads(log_text)
            issues = log.get("issues", [])
            if issues:
                raise SigningError("Apple notary log contains an unapproved warning or error")
            evidence = _base_evidence(
                target,
                source_sha,
                version,
                run_id,
                unsigned_hash,
                output,
                policy_hash,
                False,
            )
            evidence.update(
                {
                    "team_id": apple["team_id"],
                    "certificate_fingerprint": apple["leaf_sha256"],
                    "hardened_runtime": True,
                    "timestamp_verified": True,
                    "chain_verified": True,
                    "notary_status": "Accepted",
                    "notary_warnings": [],
                    "notary_submission_id": submission_id,
                    "notary_zip_sha256": sha256_file(submission),
                    "notary_log_sha256": hashlib.sha256(log_text.encode()).hexdigest(),
                }
            )
            return evidence
    windows = policy["windows"]
    certificate_script = (
        "$matches=@(Get-ChildItem Cert:\\CurrentUser\\My | Where-Object { $_.Subject -eq '"
        + windows["publisher"].replace("'", "''")
        + "' }); if ($matches.Count -ne 1) { throw 'expected one signing certificate' }; "
        "$sha=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($matches[0].RawData)); "
        "if ($sha -ne '"
        + windows["leaf_sha256"]
        + "') { throw 'signing certificate fingerprint mismatch' }; $matches[0].Thumbprint"
    )
    thumbprint = _run(["powershell", "-NoProfile", "-Command", certificate_script]).strip()
    if not thumbprint:
        raise SigningError("Windows signing certificate has no thumbprint")
    _run(
        [
            "signtool", "sign", "/fd", "SHA256", "/td", "SHA256", "/tr",
            windows["timestamp_url"], "/sha1", thumbprint, str(output),
        ]
    )
    _run(["signtool", "verify", "/pa", "/all", "/tw", str(output)])
    evidence = _base_evidence(
        target,
        source_sha,
        version,
        run_id,
        unsigned_hash,
        output,
        policy_hash,
        False,
    )
    evidence.update(
        {
            "publisher": windows["publisher"],
            "certificate_fingerprint": windows["leaf_sha256"],
            "timestamp_authority": windows["timestamp_url"],
            "timestamp_policy_oid": windows["timestamp_policy_oid"],
            "timestamp_verified": True,
            "chain_verified": True,
        }
    )
    return evidence


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--policy", type=Path, default=Path("release/signing-policy.json"))
    parser.add_argument("--repo", type=Path, default=Path.cwd())
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--evidence", type=Path, required=True)
    parser.add_argument("--target", choices=["macos-x86_64", "macos-arm64", "macos-universal", "windows-x86_64"], required=True)
    parser.add_argument("--source-sha", required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--run-id", required=True)
    parser.add_argument("--unsigned-sha256", required=True)
    parser.add_argument("--fixture", action="store_true", help=argparse.SUPPRESS)
    args = parser.parse_args()
    try:
        if sha256_file(args.source) != args.unsigned_sha256:
            raise SigningError("unsigned executable SHA-256 mismatch")
        policy, policy_hash = load_policy(args.policy, fixture=args.fixture)
        if not args.fixture:
            verify_protected_policy(args.repo, args.source_sha, args.policy, policy_hash)
        if args.output.exists():
            raise SigningError("refusing duplicate signing of existing output")
        args.output.parent.mkdir(parents=True, exist_ok=True)
        evidence = (
            fixture_finalize(
                args.source,
                args.output,
                args.target,
                args.source_sha,
                args.version,
                args.run_id,
                policy_hash,
                policy,
            )
            if args.fixture
            else production_finalize(
                args.source,
                args.output,
                args.target,
                args.source_sha,
                args.version,
                args.run_id,
                policy_hash,
                policy,
            )
        )
        args.evidence.write_bytes(canonical_bytes(evidence))
        return 0
    except (OSError, SigningError, json.JSONDecodeError) as error:
        print(f"signing: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
