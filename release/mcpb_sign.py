#!/usr/bin/env python3
"""Sign an MCPB exactly once under the independently protected policy."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any

from candidate import (
    HASH_RE,
    REQUIRED_GATES,
    CandidateError,
    canonical_bytes,
    load_manifest,
    sha256_file,
)
from signing import SigningError, load_policy, verify_protected_policy

EXPECTED_REPOSITORY = "genomoncology/biomcp"
EXPECTED_WORKFLOW_REF = (
    f"{EXPECTED_REPOSITORY}/.github/workflows/release.yml@refs/heads/main"
)
EXPECTED_JOB = "mcpb-artifact"


def _run(arguments: list[str]) -> str:
    result = subprocess.run(arguments, text=True, capture_output=True, check=False)
    if result.returncode:
        raise SigningError(result.stderr.strip() or f"command failed: {arguments[0]}")
    return result.stdout


def fixture_sign(source: Path, output: Path, fingerprint: str) -> dict[str, object]:
    unsigned = source.read_bytes()
    marker = b"MCPB_SIG_V1" + canonical_bytes(
        {"fixture_signature": hashlib.sha256(unsigned).hexdigest(), "fingerprint": fingerprint}
    ) + b"MCPB_SIG_END"
    output.write_bytes(unsigned + marker)
    return {
        "schema_version": 1,
        "unsigned_sha256": hashlib.sha256(unsigned).hexdigest(),
        "signed_sha256": sha256_file(output),
        "certificate_fingerprint": fingerprint,
        "chain_verified": True,
        "eku": "codeSigning",
        "fixture_only": True,
    }


def production_sign(
    source: Path,
    output: Path,
    policy: dict,
    policy_hash: str,
    manifest: dict[str, Any],
) -> dict[str, object]:
    certificate = Path(os.environ.get("BIOMCP_MCPB_CERTIFICATE", ""))
    private_key = Path(os.environ.get("BIOMCP_MCPB_PRIVATE_KEY", ""))
    chain = Path(os.environ.get("BIOMCP_MCPB_CHAIN", ""))
    if not all(path.is_file() for path in (certificate, private_key, chain)):
        raise SigningError("MCPB signing credentials are unavailable")
    text = _run(["openssl", "x509", "-in", str(certificate), "-noout", "-text", "-fingerprint", "-sha256", "-subject"])
    normalized = text.replace(":", "").upper()
    expected = policy["mcpb"]["leaf_sha256"]
    if expected not in normalized or policy["mcpb"]["subject"] not in text:
        raise SigningError("MCPB certificate identity mismatch")
    if "Code Signing" not in text:
        raise SigningError("MCPB certificate lacks Code Signing EKU")
    _run(["openssl", "x509", "-in", str(certificate), "-checkend", "0", "-noout"])
    _run(["openssl", "verify", "-CAfile", str(chain), str(certificate)])
    cert_key = _run(["openssl", "x509", "-in", str(certificate), "-pubkey", "-noout"])
    private_public = _run(["openssl", "pkey", "-in", str(private_key), "-pubout"])
    if cert_key != private_public:
        raise SigningError("MCPB certificate and private key do not match")
    shutil.copyfile(source, output)
    _run(
        [
            "mcpb", "sign", str(output), "--cert", str(certificate), "--key", str(private_key),
            "--intermediate", str(chain),
        ]
    )
    verified = _run(["mcpb", "verify", str(output)])
    if "valid" not in verified.lower() or "self-signed" in verified.lower():
        raise SigningError("MCPB signature verification failed")
    return {
        "schema_version": 1,
        "unsigned_sha256": sha256_file(source),
        "signed_sha256": sha256_file(output),
        "certificate_fingerprint": expected,
        "certificate_subject": policy["mcpb"]["subject"],
        "chain_verified": True,
        "eku": "codeSigning",
        "signing_policy_sha256": policy_hash,
        "source_sha": manifest["source_sha"],
        "version": manifest["version"],
        "python_version": manifest["python_version"],
        "candidate_kind": manifest["candidate_kind"],
        "stage_run_id": manifest["stage_run_id"],
        "signing_job_id": os.environ.get("GITHUB_JOB", "unknown"),
        "fixture_only": False,
    }


def _candidate_base(path: Path) -> dict[str, Any]:
    manifest = load_manifest(path)
    if (
        manifest["status"] != "staging"
        or set(manifest["gates"]) != REQUIRED_GATES
        or manifest["artifacts"] != {}
        or not HASH_RE.fullmatch(str(manifest.get("signing_policy_sha256", "")))
    ):
        raise SigningError("MCPB operation requires the validated candidate-base manifest")
    return manifest


def _job_context(manifest: dict[str, Any], *, fixture: bool) -> dict[str, str]:
    if fixture:
        return {
            "repository": "fixture/biomcp",
            "workflow_ref": "fixture/release.yml@fixture",
            "job": "fixture-mcpb-artifact",
            "run_id": manifest["stage_run_id"],
            "run_attempt": "1",
            "source_sha": manifest["source_sha"],
        }
    context = {
        "repository": os.environ.get("GITHUB_REPOSITORY", ""),
        "workflow_ref": os.environ.get("GITHUB_WORKFLOW_REF", ""),
        "job": os.environ.get("GITHUB_JOB", ""),
        "run_id": os.environ.get("GITHUB_RUN_ID", ""),
        "run_attempt": os.environ.get("GITHUB_RUN_ATTEMPT", ""),
        "source_sha": os.environ.get("GITHUB_SHA", ""),
    }
    if (
        context["repository"] != EXPECTED_REPOSITORY
        or context["workflow_ref"] != EXPECTED_WORKFLOW_REF
        or context["job"] != EXPECTED_JOB
        or context["run_id"] != manifest["stage_run_id"]
        or not context["run_attempt"].isdigit()
        or int(context["run_attempt"]) < 1
        or context["source_sha"] != manifest["source_sha"]
    ):
        raise SigningError("unsigned MCPB GitHub job context does not match candidate")
    return context


def _atomic_evidence(path: Path, evidence: dict[str, Any]) -> None:
    if path.exists():
        raise SigningError("refusing duplicate unsigned MCPB attestation")
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    try:
        with os.fdopen(descriptor, "wb") as handle:
            handle.write(canonical_bytes(evidence))
            handle.flush()
            os.fsync(handle.fileno())
        if path.exists():
            raise SigningError("refusing duplicate unsigned MCPB attestation")
        os.link(temporary, path)
        os.unlink(temporary)
    finally:
        if os.path.exists(temporary):
            os.unlink(temporary)


def attest_unsigned_development(
    source: Path,
    evidence_path: Path,
    manifest: dict[str, Any],
    policy: dict[str, Any],
    policy_hash: str,
    *,
    fixture: bool,
) -> dict[str, Any]:
    if manifest["candidate_kind"] != "development":
        raise SigningError("unsigned MCPB attestation requires a development candidate")
    exception = policy["development_unsigned_mcpb"]
    if exception["enabled"] is not True or exception["blocks_promotion"] is not True:
        raise SigningError("unsigned development MCPB exception is disabled")
    if manifest["signing_policy_sha256"] != policy_hash:
        raise SigningError("candidate signing policy hash mismatch")
    expected_name = f"biomcp-{manifest['version']}.mcpb"
    if not source.is_file() or source.name != expected_name:
        raise SigningError("unsigned development MCPB bytes are absent or misnamed")
    evidence = {
        "schema_version": 1,
        "evidence_type": "unsigned-development-mcpb",
        "archive_sha256": sha256_file(source),
        "source_sha": manifest["source_sha"],
        "version": manifest["version"],
        "python_version": manifest["python_version"],
        "candidate_kind": manifest["candidate_kind"],
        "stage_run_id": manifest["stage_run_id"],
        "signing_policy_sha256": policy_hash,
        "package": exception["package"],
        "tool_version": exception["tool_version"],
        "exception_reason": exception["reason"],
        "outer_signature_status": "unsigned-development",
        "non_promotable": True,
        "github": _job_context(manifest, fixture=fixture),
        "fixture_only": fixture,
    }
    _atomic_evidence(evidence_path, evidence)
    return evidence


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--evidence", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--policy", type=Path, default=Path("release/signing-policy.json"))
    parser.add_argument("--repo", type=Path, default=Path.cwd())
    parser.add_argument("--attest-development", action="store_true")
    parser.add_argument("--fixture", action="store_true", help=argparse.SUPPRESS)
    args = parser.parse_args()
    try:
        manifest = _candidate_base(args.manifest)
        policy, policy_hash = load_policy(
            args.policy,
            fixture=args.fixture,
            require_mcpb=not args.attest_development,
        )
        if manifest["signing_policy_sha256"] != policy_hash:
            raise SigningError("candidate signing policy hash mismatch")
        if not args.fixture:
            verify_protected_policy(
                args.repo, manifest["source_sha"], args.policy, policy_hash
            )
        if args.attest_development:
            if args.output is not None:
                raise SigningError("unsigned MCPB attestation must not copy the archive")
            attest_unsigned_development(
                args.source,
                args.evidence,
                manifest,
                policy,
                policy_hash,
                fixture=args.fixture,
            )
            return 0
        if manifest["candidate_kind"] != "release":
            raise SigningError("stable MCPB signing requires a release candidate")
        if args.output is None:
            raise SigningError("stable MCPB signing requires an output archive")
        if args.output.exists():
            raise SigningError("refusing duplicate MCPB signing")
        if args.fixture:
            evidence = fixture_sign(args.source, args.output, policy["mcpb"]["leaf_sha256"])
        else:
            evidence = production_sign(
                args.source, args.output, policy, policy_hash, manifest
            )
        args.evidence.write_bytes(canonical_bytes(evidence))
        return 0
    except (CandidateError, OSError, SigningError, json.JSONDecodeError) as error:
        print(f"MCPB signing: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
