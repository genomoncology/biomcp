#!/usr/bin/env python3
"""Prepare every Cargo-owned artifact used by specification pages."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import sys
from typing import Iterable


MCP_EXAMPLE = "rmcp_streamable_http_contract"
FILTERED_TEST_TARGETS = (
    "article_cli_tests_structure",
    "benchmark_cli_structure",
    "cli_line_cap_absorption",
    "health_cli_structure",
    "list_cli_structure",
    "skill_cli_structure",
)


def cargo_artifacts(lines: Iterable[str]) -> dict[str, Path]:
    artifacts: dict[str, Path] = {}
    for line in lines:
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            continue
        if message.get("reason") != "compiler-artifact" or not message.get(
            "executable"
        ):
            continue
        target = message.get("target", {})
        name = target.get("name")
        if isinstance(name, str):
            artifacts[name] = Path(message["executable"]).resolve()
    return artifacts


def run_cargo_json(root: Path, arguments: list[str]) -> dict[str, Path]:
    command = [str(root / "tools" / "with-build-identity"), "cargo", *arguments]
    print(f"spec preparation: {shlex.join(command[2:])}", file=sys.stderr)
    completed = subprocess.run(
        command,
        cwd=root,
        check=True,
        text=True,
        stdout=subprocess.PIPE,
    )
    return cargo_artifacts(completed.stdout.splitlines())


def capture(root: Path, destination: Path, arguments: list[str]) -> None:
    command = ["cargo", *arguments]
    print(f"spec preparation: {shlex.join(command)}", file=sys.stderr)
    with destination.open("w", encoding="utf-8") as output:
        subprocess.run(command, cwd=root, check=True, text=True, stdout=output)


def require_artifact(artifacts: dict[str, Path], name: str, owner: str) -> Path:
    artifact = artifacts.get(name)
    if artifact is None or not artifact.is_file():
        raise SystemExit(f"spec preparation did not produce {owner} artifact {name!r}")
    return artifact


def install_artifact(source: Path, destination: Path) -> Path:
    destination.parent.mkdir(parents=True, exist_ok=True)
    if source.resolve() != destination.resolve():
        shutil.copy2(source, destination)
    destination.chmod(destination.stat().st_mode | 0o111)
    return destination.resolve()


def export_name(target: str) -> str:
    return "BIOMCP_SPEC_TEST_" + target.upper().replace("-", "_")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--mode", required=True)
    parser.add_argument("--profile", default="spec")
    parser.add_argument("--feature-on-bin")
    parser.add_argument("--cargo-feature-arg", action="append", default=[])
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()

    root = Path(__file__).resolve().parents[1]
    artifact_root = root / ".cache" / "spec-artifacts"
    artifact_root.mkdir(parents=True, exist_ok=True)

    routine_mode = args.mode in {"spec", "spec-pr", "spec-contracts"}
    needs_feature_off = routine_mode or args.mode == "verify"
    needs_feature_on = args.mode.startswith("verify") or bool(args.feature_on_bin)
    exports: dict[str, Path | str] = {"BIOMCP_SPEC_ARTIFACT_MODE": args.mode}

    provided_feature_on = (
        Path(args.feature_on_bin).resolve() if args.feature_on_bin else None
    )
    if provided_feature_on is not None:
        if not provided_feature_on.is_file():
            raise SystemExit(
                "provided feature-on BioMCP binary does not exist: "
                f"{provided_feature_on}"
            )
        # Copy before a feature-off build using the same Cargo profile can
        # replace target/<profile>/biomcp.
        exports["BIOMCP_SPEC_FEATURE_ON_BIN"] = install_artifact(
            provided_feature_on, artifact_root / "feature-on" / "biomcp"
        )

    if needs_feature_off:
        if not args.cargo_feature_arg:
            raise SystemExit("spec preparation requires the declared routine Cargo features")
        off_arguments = [
            "build",
            "--locked",
            "--profile",
            args.profile,
            *args.cargo_feature_arg,
            "--bin",
            "biomcp",
            "--example",
            MCP_EXAMPLE,
        ]
        off_arguments.append("--message-format=json")
        off_artifacts = run_cargo_json(root, off_arguments)
        feature_off = install_artifact(
            require_artifact(off_artifacts, "biomcp", "feature-off CLI"),
            artifact_root / "feature-off" / "biomcp",
        )
        exports["BIOMCP_SPEC_FEATURE_OFF_BIN"] = feature_off
        exports["BIOMCP_SPEC_MCP_EXAMPLE_BIN"] = install_artifact(
            require_artifact(off_artifacts, MCP_EXAMPLE, "MCP example"),
            artifact_root / MCP_EXAMPLE,
        )

    if needs_feature_on and provided_feature_on is None:
        on_artifacts = run_cargo_json(
            root,
            [
                "build",
                "--locked",
                "--profile",
                args.profile,
                "--bin",
                "biomcp",
                "--message-format=json",
            ],
        )
        exports["BIOMCP_SPEC_FEATURE_ON_BIN"] = install_artifact(
            require_artifact(on_artifacts, "biomcp", "feature-on CLI"),
            artifact_root / "feature-on" / "biomcp",
        )

    if routine_mode:
        tree_path = artifact_root / "cargo-tree-no-default-features.txt"
        metadata_path = artifact_root / "cargo-metadata.json"
        capture(
            root,
            tree_path,
            [
                "tree",
                "--locked",
                *args.cargo_feature_arg,
                "--edges",
                "normal,build",
            ],
        )
        capture(
            root,
            metadata_path,
            ["metadata", "--locked", "--no-deps", "--format-version", "1"],
        )
        exports["BIOMCP_SPEC_CARGO_TREE"] = tree_path.resolve()
        exports["BIOMCP_SPEC_CARGO_METADATA"] = metadata_path.resolve()

    if needs_feature_on:
        exports["BIOMCP_BIN"] = exports["BIOMCP_SPEC_FEATURE_ON_BIN"]
    else:
        exports["BIOMCP_BIN"] = exports["BIOMCP_SPEC_FEATURE_OFF_BIN"]

    if args.mode.startswith("verify"):
        test_artifacts = run_cargo_json(
            root,
            ["test", "--locked", "--no-run", "--message-format=json"],
        )
        exports["BIOMCP_SPEC_TEST_LIB"] = require_artifact(
            test_artifacts, "biomcp_cli", "library test"
        )
        for target in FILTERED_TEST_TARGETS:
            exports[export_name(target)] = require_artifact(
                test_artifacts, target, "integration test"
            )

    args.output.parent.mkdir(parents=True, exist_ok=True)
    temporary = args.output.with_suffix(".tmp")
    with temporary.open("w", encoding="utf-8") as output:
        for name, value in sorted(exports.items()):
            output.write(f"export {name}={shlex.quote(str(value))}\n")
    os.replace(temporary, args.output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
