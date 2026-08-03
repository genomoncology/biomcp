#!/usr/bin/env python3
"""Audit provenance receipts for committed source-test captures."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import re
import sys
from pathlib import Path
from urllib.parse import parse_qsl, urlsplit

MANIFEST_NAME = "capture-receipts.json"
CLASSIFICATIONS = (
    "real_and_receipted",
    "synthetic_and_ineligible",
    "pending_verification",
)
REQUIRED_RECEIPT_FIELDS = (
    "provider",
    "request",
    "captured_at",
    "sha256",
    "minimization_or_redaction",
    "provider_origin_statement",
)
SHA256_RE = re.compile(r"[0-9a-f]{64}\Z")
RFC3339_UTC_RE = re.compile(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z\Z")
UNSAFE_REQUEST_FIELDS = {
    "access_token",
    "api_key",
    "apikey",
    "auth",
    "authorization",
    "awsaccesskeyid",
    "client_secret",
    "credential",
    "key",
    "password",
    "secret",
    "signature",
    "sig",
    "token",
}


def invalid_request(request: object) -> bool:
    if not isinstance(request, str):
        return True
    parsed = urlsplit(request)
    if parsed.scheme not in {"http", "https"} or not parsed.netloc:
        return True
    if parsed.username is not None or parsed.password is not None or parsed.fragment:
        return True
    return any(
        key.lower() in UNSAFE_REQUEST_FIELDS
        or key.lower().startswith(("x-amz-", "x-goog-"))
        for component in (parsed.query, parsed.fragment)
        for key, _ in parse_qsl(component, keep_blank_values=True)
    )


def invalid_utc_timestamp(value: object) -> bool:
    if not isinstance(value, str) or not RFC3339_UTC_RE.fullmatch(value):
        return True
    try:
        parsed = dt.datetime.fromisoformat(value.removesuffix("Z") + "+00:00")
    except ValueError:
        return True
    return parsed.utcoffset() != dt.timedelta(0)


def audit(root: Path) -> dict[str, object]:
    manifest_path = root / MANIFEST_NAME
    errors: list[str] = []
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ValueError(f"cannot read {MANIFEST_NAME}: {error}") from error

    entries = manifest.get("entries") if isinstance(manifest, dict) else None
    if (
        not isinstance(manifest, dict)
        or manifest.get("schema_version") != 1
        or not isinstance(entries, list)
    ):
        raise ValueError("manifest requires schema_version 1 and entries array")

    files = sorted(
        path.relative_to(root).as_posix()
        for path in root.rglob("*")
        if path.is_file() and path != manifest_path
    )
    discovered = set(files)
    by_path: dict[str, dict[str, object]] = {}
    classifications = dict.fromkeys(CLASSIFICATIONS, 0)
    for entry in entries:
        if not isinstance(entry, dict) or not isinstance(entry.get("path"), str):
            errors.append("entry requires path")
            continue
        path = entry["path"]
        if path in by_path:
            errors.append(f"duplicate entry: {path}")
            continue
        by_path[path] = entry

        classification = entry.get("classification")
        if classification not in classifications:
            errors.append(f"{path}: invalid classification")
            continue
        classifications[classification] += 1
        receipt = entry.get("receipt")
        if classification == "real_and_receipted":
            if not isinstance(receipt, dict):
                errors.append(f"{path}: receipt required")
                continue
            for field in REQUIRED_RECEIPT_FIELDS:
                if not isinstance(receipt.get(field), str) or not receipt[field]:
                    errors.append(f"{path}: receipt {field} required")
            if invalid_request(receipt.get("request")):
                errors.append(f"{path}: receipt request is unsafe")
            if invalid_utc_timestamp(receipt.get("captured_at")):
                errors.append(f"{path}: receipt captured_at must be RFC3339 UTC")
            digest = receipt.get("sha256")
            if not isinstance(digest, str) or not SHA256_RE.fullmatch(digest):
                errors.append(f"{path}: receipt sha256 must be lowercase SHA-256")
            elif path in discovered:
                actual = hashlib.sha256((root / path).read_bytes()).hexdigest()
                if digest != actual:
                    errors.append(f"{path}: receipt sha256 does not match raw bytes")
        else:
            if (
                not isinstance(entry.get("ineligible_reason"), str)
                or not entry["ineligible_reason"]
            ):
                errors.append(f"{path}: ineligible_reason required")
            if receipt is not None:
                errors.append(f"{path}: non-real classification cannot carry receipt")

    missing = sorted(discovered - set(by_path))
    orphaned = sorted(set(by_path) - discovered)
    errors.extend(f"missing entry: {path}" for path in missing)
    errors.extend(f"orphan entry: {path}" for path in orphaned)
    if errors:
        raise ValueError("\n".join(errors))

    corrections = manifest.get("historical_corrections")
    if not isinstance(corrections, list):
        raise ValueError("historical_corrections must be an array")
    return {
        "audited_files": len(files),
        "classified_files": len(by_path),
        "classifications": classifications,
        "confirmed_byte_unfaithful": manifest.get("confirmed_byte_unfaithful"),
        "historical_corrections": corrections,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()
    try:
        report = audit(args.root)
    except ValueError as error:
        print(error, file=sys.stderr)
        return 1
    if args.json:
        print(json.dumps(report, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
