#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
import shlex
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Sequence

PYTEST_ITEM_SUFFIX_RE = re.compile(r" \(line \d+\) \[[^\]]+\]$")


@dataclass(frozen=True)
class CollectionFailure:
    command: list[str]
    exit_code: int
    stdout: str
    stderr: str
    target_files: list[str]
    errors: list[str] = field(default_factory=list)


@dataclass(frozen=True)
class CollectionResult:
    items: list[str]
    command: list[str]
    target_files: list[str]
    failure: CollectionFailure | None = None


@dataclass(frozen=True)
class SmokeTargetError:
    rule: str
    smoke_target: str
    node_id: str
    section: str
    message: str
    candidates: list[str] = field(default_factory=list)


@dataclass(frozen=True)
class SmokeTargetResolution:
    resolved: list[str]
    errors: list[SmokeTargetError]
    collection: CollectionResult


def canonical_section_id(node_id: str) -> str:
    path, separator, item_name = node_id.partition("::")
    if not separator:
        return node_id
    return f"{path}{separator}{PYTEST_ITEM_SUFFIX_RE.sub('', item_name)}"


def parse_make_variable_values(makefile_path: Path, variable_name: str) -> list[str]:
    assignment_re = re.compile(rf"^{re.escape(variable_name)}\s*[:+?]?=")
    try:
        lines = makefile_path.read_text(encoding="utf-8").splitlines()
    except FileNotFoundError:
        return []

    for index, line in enumerate(lines):
        match = assignment_re.match(line)
        if match is None:
            continue

        value_lines = [line[match.end() :]]
        while value_lines[-1].rstrip().endswith("\\") and index + 1 < len(lines):
            index += 1
            value_lines.append(lines[index])
        return re.findall(r'"([^"]+)"', "\n".join(value_lines))

    return []


def target_spec_path(target: str) -> str | None:
    spec_path, separator, _item_name = target.partition("::")
    if not separator or not spec_path:
        return None
    return spec_path


def collect_pytest_items(root_dir: Path, spec_paths: list[str]) -> CollectionResult:
    target_files = list(dict.fromkeys(spec_paths))
    command = [
        sys.executable,
        "-m",
        "pytest",
        "-o",
        "addopts=",
        "--mustmatch-lang",
        "bash",
        "--collect-only",
        "-q",
        *target_files,
    ]
    proc = subprocess.run(
        command,
        cwd=root_dir,
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        failure = CollectionFailure(
            command=command,
            exit_code=proc.returncode,
            stdout=proc.stdout,
            stderr=proc.stderr,
            target_files=target_files,
            errors=[f"collection failed for {', '.join(target_files)}"],
        )
        return CollectionResult(
            items=[],
            command=command,
            target_files=target_files,
            failure=failure,
        )

    prefixes = tuple(f"{spec_path}::" for spec_path in target_files)
    items = [
        line
        for raw_line in proc.stdout.splitlines()
        if (line := raw_line.strip()).startswith(prefixes)
    ]
    return CollectionResult(items=items, command=command, target_files=target_files)


def resolve_smoke_targets(root_dir: Path, targets: list[str]) -> SmokeTargetResolution:
    errors: list[SmokeTargetError] = []
    target_paths: list[str] = []
    for target in targets:
        spec_path = target_spec_path(target)
        if spec_path is None:
            errors.append(
                SmokeTargetError(
                    rule="smoke-target-invalid",
                    smoke_target=target,
                    node_id=target,
                    section=target,
                    message=f"'{target}' is not a section-qualified pytest target",
                )
            )
            continue
        target_paths.append(spec_path)

    collection = (
        collect_pytest_items(root_dir, target_paths)
        if target_paths
        else _empty_collection()
    )
    if collection.failure is not None:
        return SmokeTargetResolution(resolved=[], errors=errors, collection=collection)

    collected_items = set(collection.items)
    items_by_section: dict[str, list[str]] = {}
    for item in collection.items:
        items_by_section.setdefault(canonical_section_id(item), []).append(item)

    resolved: list[str] = []
    for target in targets:
        spec_path = target_spec_path(target)
        if spec_path is None:
            continue

        node_id = canonical_section_id(target)
        _path, _separator, section = node_id.partition("::")

        if target in collected_items:
            resolved.append(target)
            continue

        if _has_pytest_item_suffix(target):
            errors.append(
                SmokeTargetError(
                    rule="smoke-target-not-collectable",
                    smoke_target=target,
                    node_id=node_id,
                    section=section or node_id,
                    message=f"smoke target '{target}' is not collectable; the line-qualified pytest item may be stale",
                )
            )
            continue

        candidates = items_by_section.get(node_id, [])
        if len(candidates) == 1:
            resolved.append(candidates[0])
            continue

        if len(candidates) > 1:
            errors.append(
                SmokeTargetError(
                    rule="smoke-target-ambiguous",
                    smoke_target=target,
                    node_id=node_id,
                    section=section or node_id,
                    message=f"smoke target '{target}' maps to multiple collectable pytest items",
                    candidates=candidates,
                )
            )
            continue

        errors.append(
            SmokeTargetError(
                rule="smoke-target-not-collectable",
                smoke_target=target,
                node_id=node_id,
                section=section or node_id,
                message=f"smoke target '{target}' is not collectable",
            )
        )

    return SmokeTargetResolution(
        resolved=resolved, errors=errors, collection=collection
    )


def _empty_collection() -> CollectionResult:
    return CollectionResult(items=[], command=[], target_files=[])


def _has_pytest_item_suffix(target: str) -> bool:
    _path, separator, item_name = target.partition("::")
    return bool(separator and PYTEST_ITEM_SUFFIX_RE.search(item_name))


def _parse_args(argv: Sequence[str] | None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Resolve stable SPEC_SMOKE_ARGS entries to collectable pytest item IDs.",
    )
    parser.add_argument("--root-dir", type=Path, default=Path.cwd())
    parser.add_argument("--makefile", type=Path)
    parser.add_argument("--makefile-variable")
    parser.add_argument("targets", nargs="*")
    args = parser.parse_args(argv)

    if args.makefile_variable and args.targets:
        parser.error("use either --makefile-variable or positional targets, not both")
    if bool(args.makefile) != bool(args.makefile_variable):
        parser.error("--makefile and --makefile-variable must be provided together")
    return args


def _targets_from_args(args: argparse.Namespace) -> list[str]:
    if args.makefile_variable:
        return parse_make_variable_values(args.makefile, args.makefile_variable)
    return list(args.targets)


def _write_diagnostics(resolution: SmokeTargetResolution) -> None:
    failure = resolution.collection.failure
    if failure is not None:
        print(
            "smoke-target-collection-failed: pytest collect-only failed",
            file=sys.stderr,
        )
        print(f"command: {shlex.join(failure.command)}", file=sys.stderr)
        print(f"exit code: {failure.exit_code}", file=sys.stderr)
        print(f"target files: {', '.join(failure.target_files)}", file=sys.stderr)
        if failure.stdout:
            print("stdout:", file=sys.stderr)
            print(
                failure.stdout,
                file=sys.stderr,
                end="" if failure.stdout.endswith("\n") else "\n",
            )
        if failure.stderr:
            print("stderr:", file=sys.stderr)
            print(
                failure.stderr,
                file=sys.stderr,
                end="" if failure.stderr.endswith("\n") else "\n",
            )

    for error in resolution.errors:
        print(f"{error.rule}: {error.message}", file=sys.stderr)
        if error.candidates:
            for candidate in error.candidates:
                print(f"  candidate: {candidate}", file=sys.stderr)


def main(argv: Sequence[str] | None = None) -> int:
    args = _parse_args(argv)
    targets = _targets_from_args(args)
    if not targets:
        print("no smoke targets provided", file=sys.stderr)
        return 2

    resolution = resolve_smoke_targets(args.root_dir, targets)
    if resolution.collection.failure is not None or resolution.errors:
        _write_diagnostics(resolution)
        return 1

    for node_id in resolution.resolved:
        print(node_id)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
