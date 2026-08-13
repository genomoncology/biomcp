#!/usr/bin/env python3
"""Render the immutable Homebrew formula from staged macOS artifacts."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
from pathlib import Path
from typing import Any

from candidate import ARTIFACTS, canonical_bytes, sha256_file

RELEASE_ROOT = "https://github.com/genomoncology/biomcp/releases/download"


class FormulaError(ValueError):
    pass


def final_url(version: str, filename: str) -> str:
    if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?", version):
        raise FormulaError("invalid formula version")
    if Path(filename).name != filename:
        raise FormulaError("invalid formula artifact filename")
    return f"{RELEASE_ROOT}/v{version}/{filename}"


def homebrew_cache_path(cache: Path, url: str, filename: str) -> Path:
    return cache / "downloads" / f"{hashlib.sha256(url.encode()).hexdigest()}--{filename}"


def render(template: str, arm: dict[str, Any], intel: dict[str, Any]) -> str:
    if arm["version"] != intel["version"] or arm["source_sha"] != intel["source_sha"]:
        raise FormulaError("macOS candidate identities disagree")
    expected = {
        "native-macos-arm64": arm,
        "native-macos-x86_64": intel,
    }
    for artifact_id, record in expected.items():
        if record.get("id") != artifact_id or record.get("kind") != "native":
            raise FormulaError(f"wrong Homebrew source artifact: {artifact_id}")
        if not record.get("evidence", {}).get("signing"):
            raise FormulaError(f"unsigned Homebrew source artifact: {artifact_id}")
    replacements = {
        "__TAG__": f"v{arm['version']}",
        "__VERSION__": arm["version"],
        "__REVISION__": arm["source_sha"][:8],
        "__DARWIN_ARM64_SHA256__": arm["sha256"],
        "__DARWIN_X86_64_SHA256__": intel["sha256"],
        "__DARWIN_ARM64_BINARY_SHA256__": arm["evidence"]["binary_sha256"],
        "__DARWIN_X86_64_BINARY_SHA256__": intel["evidence"]["binary_sha256"],
    }
    result = template
    for marker, value in replacements.items():
        result = result.replace(marker, value)
    if "__" in result:
        raise FormulaError("unresolved Homebrew formula placeholder")
    for record in (arm, intel):
        if final_url(record["version"], record["filename"]) not in result:
            raise FormulaError("formula does not use immutable final archive URL")
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--template", type=Path, default=Path("Formula/biomcp.rb"))
    parser.add_argument("--arm-record", type=Path, required=True)
    parser.add_argument("--intel-record", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--record", type=Path, required=True)
    args = parser.parse_args()
    arm = json.loads(args.arm_record.read_text(encoding="utf-8"))
    intel = json.loads(args.intel_record.read_text(encoding="utf-8"))
    formula = render(args.template.read_text(encoding="utf-8"), arm, intel)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(formula, encoding="utf-8")
    kind, target = ARTIFACTS["homebrew-formula"]
    record = {
        "id": "homebrew-formula",
        "kind": kind,
        "target": target,
        "filename": args.output.name,
        "sha256": sha256_file(args.output),
        "bytes": args.output.stat().st_size,
        "source_sha": arm["source_sha"],
        "version": arm["version"],
        "stage_run_id": arm["stage_run_id"],
        "provenance": {"generator": "release/homebrew.py", "build_count": 1},
        "evidence": {
            "immutable_urls": [
                final_url(arm["version"], arm["filename"]),
                final_url(intel["version"], intel["filename"]),
            ],
            "offline_cache_required": True,
            "candidate_jobs": ["macos-15", "macos-15-intel"],
        },
        "upstream": {
            "native-macos-arm64": arm["sha256"],
            "native-macos-x86_64": intel["sha256"],
        },
    }
    args.record.write_bytes(canonical_bytes(record))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
