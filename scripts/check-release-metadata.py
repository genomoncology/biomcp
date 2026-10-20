#!/usr/bin/env python3
"""Run the existing release metadata checks before a candidate enters a gate."""

from __future__ import annotations

import argparse
import importlib.util
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PYTHON_TESTS = (
    "tests/test_docs_changelog_refresh.py",
    "tests/test_version_sync_script.py",
    "tests/test_release_versions_check.py",
    "tests/test_release_metadata_contract.py",
    "tests/test_changelog_coverage_check.py",
    "tests/test_citation_contract.py",
    "tests/test_release_notes.py",
    "tests/test_release_draft_notes.py",
    "tests/test_cli_surface_contract_ratchet.py::"
    "test_cli_surface_contract_compares_release_process_versions_to_metadata",
)
RUST_TESTS = (
    "binary(package_build_identity) | "
    "(binary(cli_global_modifiers) & test(json_display_version)) | "
    "test(cli::system::tests::version_) | "
    "test(cli::system::tests::clap_version_) | "
    "test(cli::update::tests::release_metadata_) | "
    "test(cli::update::tests::failed_staged_version_) | "
    "test(cli::update::tests::owned_update_)"
)


def runbook_check() -> int:
    spec = importlib.util.spec_from_file_location(
        "quality_ratchet", ROOT / "tools/check-quality-ratchet.py"
    )
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    result = module.check_release_process_versions_match_package_metadata(ROOT)
    print(json.dumps(result, indent=2), flush=True)
    return int(result["status"] != "pass")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate-sha", required=True, help="full checked-out commit SHA")
    parser.add_argument("--version", required=True, help="proposed stable package version")
    parser.add_argument("--tag", required=True, help="proposed v-prefixed release tag")
    args = parser.parse_args()
    commands = (
        ("candidate version", [
            sys.executable, "scripts/check-release-versions.py",
            "--candidate-sha", args.candidate_sha,
            "--workflow-sha", args.candidate_sha, "--rehearsal-version", args.version,
        ]),
        ("tag version", [
            sys.executable, "scripts/check-release-versions.py", "--tag", args.tag,
        ]),
        ("changelog coverage", [
            sys.executable, "scripts/check-changelog-coverage.py",
            "--tag", args.tag, "--revision", args.candidate_sha,
        ]),
        ("runbook metadata", None),
        ("Python metadata contracts", ["uv", "run", "--no-sync", "pytest", "-q", *PYTHON_TESTS]),
        ("Rust metadata contracts", [
            "tools/with-build-identity", "cargo", "nextest", "run",
            "--no-default-features", "--lib", "--test", "package_build_identity",
            "--test", "cli_global_modifiers", "-E", RUST_TESTS,
        ]),
    )
    failures = []
    for name, command in commands:
        print(f"\n=== {name} ===", flush=True)
        try:
            status = (
                runbook_check() if command is None else
                subprocess.run(command, cwd=ROOT, check=False).returncode
            )
        except Exception as error:
            print(f"{name}: {error}", file=sys.stderr, flush=True)
            status = 1
        if status:
            failures.append(name)
    if failures:
        print("\nFailed release metadata groups: " + "; ".join(failures), flush=True)
        return 1
    print("\nAll release metadata checks passed.", flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
