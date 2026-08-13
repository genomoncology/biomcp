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
from pathlib import Path

from candidate import canonical_bytes, sha256_file
from signing import SigningError, load_policy, verify_protected_policy


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
    source: Path, output: Path, policy: dict, policy_hash: str
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
        "signing_job_id": os.environ.get("GITHUB_JOB", "unknown"),
        "fixture_only": False,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--evidence", type=Path, required=True)
    parser.add_argument("--policy", type=Path, default=Path("release/signing-policy.json"))
    parser.add_argument("--repo", type=Path, default=Path.cwd())
    parser.add_argument("--source-sha", required=True)
    parser.add_argument("--fixture", action="store_true", help=argparse.SUPPRESS)
    args = parser.parse_args()
    try:
        policy, policy_hash = load_policy(args.policy, fixture=args.fixture)
        if args.output.exists():
            raise SigningError("refusing duplicate MCPB signing")
        if args.fixture:
            evidence = fixture_sign(args.source, args.output, policy["mcpb"]["leaf_sha256"])
        else:
            verify_protected_policy(args.repo, args.source_sha, args.policy, policy_hash)
            evidence = production_sign(args.source, args.output, policy, policy_hash)
        args.evidence.write_bytes(canonical_bytes(evidence))
        return 0
    except (OSError, SigningError, json.JSONDecodeError) as error:
        print(f"MCPB signing: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
