#!/usr/bin/env python3
from __future__ import annotations

import argparse
import base64
from collections.abc import Iterator
import hashlib
import json
from pathlib import Path
import re
import subprocess
import tarfile

ROOT = Path(__file__).resolve().parents[1]
INVENTORY = ROOT / "tools" / "zero-coupling-historical.json"
FORBIDDEN = ("bio" + "data", "cfafc69d27c9a2fc" + "74909f21692a418a8b17db83")
TRIAL_SUBJECTS = ("clinical " + "trial", "trial " + "contract")
HANDOFF_MECHANISMS = (
    "check" + "out",
    "path depend" + "ency",
    "patch depend" + "ency",
    "defer" + "red package",
    "gener" + "ated source",
    "rename" + "d handoff",
)
TRIAL_OWNER = re.compile(
    r"(?:clinical[\W_]*trials?|"
    r"(?:shared|external)[\W_]*trials?(?:[\W_]*(?:core|contract|types?|schema|model|domain))?|"
    r"trials?[\W_]*(?:core|contract|types?|schema|model|domain|shared))",
    re.IGNORECASE,
)
INLINE_CARGO_ENTRY = re.compile(
    r"(?m)^\s*(?P<key>\"[^\"\r\n]+\"|'[^'\r\n]+'|[A-Za-z0-9_.-]+)"
    r"\s*=\s*\{(?P<body>[^}]{0,2000})\}"
)
CARGO_ASSIGNMENT = re.compile(
    r"(?m)^\s*(?P<key>\"[^\"\r\n]+\"|'[^'\r\n]+'|[A-Za-z0-9_.-]+)"
    r"\s*=\s*(?P<value>[^\r\n]+)"
)
TOML_TABLE_HEADER = re.compile(r"(?m)^\s*\[(?!\[)(?P<header>[^]\r\n]+)\]\s*(?:#.*)?$")
TOML_ARRAY_HEADER = re.compile(r"(?m)^\s*\[\[(?P<header>[^]\r\n]+)\]\]\s*(?:#.*)?$")
DEPENDENCY_SECTION = re.compile(r"(?i)(?:^|\.)(?:dev-|build-)?dependencies$")
DEPENDENCY_TABLE = re.compile(r"(?i)(?:^|\.)(?:dev-|build-)?dependencies\.(?P<key>.+)$")
TOML_NAME = re.compile(r"(?mi)^\s*name\s*=\s*[\"'](?P<value>[^\"']+)[\"']")
TOML_SOURCE = re.compile(r"(?mi)^\s*source\s*=\s*[\"'](?P<value>[^\"']+)[\"']")
INCLUDE_STATEMENT = re.compile(r"(?is)\binclude\s*!\s*\([^;]{0,2000}\)\s*;")
EXTERNAL_GIT_COMMAND = re.compile(
    r"(?i)\bgit(?:\s+-[^\s]+)*\s+"
    r"(?:clone|submodule\s+add|worktree\s+add)\b"
)


def _text(data: bytes) -> str | None:
    if b"\0" in data:
        return None
    try:
        return data.decode("utf-8")
    except UnicodeDecodeError:
        return None


def _forbidden_text(text: str) -> bool:
    folded = text.casefold()
    if any(value in folded for value in FORBIDDEN):
        return True
    if any(
        f"{subject} {mechanism}" in folded
        for subject in TRIAL_SUBJECTS
        for mechanism in HANDOFF_MECHANISMS
    ):
        return True
    return _has_external_trial_handoff(text)


def _has_external_trial_handoff(text: str) -> bool:
    for entry in INLINE_CARGO_ENTRY.finditer(text):
        key_and_body = entry.group("key") + " " + entry.group("body")
        if TRIAL_OWNER.search(key_and_body) and re.search(
            r"(?i)\b(?:path|git|rev)\s*=", entry.group("body")
        ):
            return True

    for is_array, header, body in _toml_sections(text):
        dependency = DEPENDENCY_TABLE.search(header)
        if dependency:
            if TRIAL_OWNER.search(dependency.group("key")):
                return True
            for assignment in CARGO_ASSIGNMENT.finditer(body):
                field = assignment.group("key").strip("\"'").casefold()
                if field in {"package", "path", "git"} and TRIAL_OWNER.search(
                    assignment.group("value")
                ):
                    return True
        if DEPENDENCY_SECTION.search(header):
            for assignment in CARGO_ASSIGNMENT.finditer(body):
                if TRIAL_OWNER.search(
                    assignment.group("key") + " " + assignment.group("value")
                ):
                    return True
        header_folded = header.casefold()
        if header_folded.startswith("patch.") or header_folded.startswith("replace"):
            if TRIAL_OWNER.search(header):
                return True
            for assignment in CARGO_ASSIGNMENT.finditer(body):
                if TRIAL_OWNER.search(
                    assignment.group("key") + " " + assignment.group("value")
                ):
                    return True
        if is_array and header_folded == "package":
            name = TOML_NAME.search(body)
            source = TOML_SOURCE.search(body)
            if (
                name
                and source
                and TRIAL_OWNER.search(name.group("value"))
                and source.group("value").casefold().startswith("git+")
            ):
                return True

    for statement in INCLUDE_STATEMENT.findall(text):
        if (
            TRIAL_OWNER.search(statement)
            and re.search(r"(?i)\benv\s*!\s*\(", statement)
            and re.search(r"(?i)generated(?:[._/-]|\b)", statement)
        ):
            return True

    return any(
        EXTERNAL_GIT_COMMAND.search(line) and TRIAL_OWNER.search(line)
        for line in text.splitlines()
    )


def _toml_sections(text: str) -> Iterator[tuple[bool, str, str]]:
    headers = [
        (match.start(), match.end(), False, match.group("header"))
        for match in TOML_TABLE_HEADER.finditer(text)
    ]
    headers.extend(
        (match.start(), match.end(), True, match.group("header"))
        for match in TOML_ARRAY_HEADER.finditer(text)
    )
    headers.sort()
    for index, (_, end, is_array, header) in enumerate(headers):
        body_end = headers[index + 1][0] if index + 1 < len(headers) else len(text)
        yield is_array, header.strip(), text[end:body_end]


def _matches(data: bytes) -> bool:
    text = _text(data)
    return text is not None and _forbidden_text(text)


def _path_matches(path: str) -> bool:
    return _forbidden_text(path)


def _allowlist(path: Path = INVENTORY) -> dict[str, str]:
    encoded = json.loads(path.read_text(encoding="utf-8"))
    return {
        base64.b64decode(name).decode("utf-8"): digest
        for name, digest in encoded.items()
    }


def scan_files(root: Path, names: list[str], inventory: Path = INVENTORY) -> list[str]:
    allowed = _allowlist(inventory)
    violations: list[str] = []
    seen_allowed: set[str] = set()
    for name in names:
        data = (root / name).read_bytes()
        if not (_path_matches(name) or _matches(data)):
            continue
        digest = hashlib.sha256(data).hexdigest()
        if allowed.get(name) == digest:
            seen_allowed.add(name)
        else:
            violations.append(name)
    missing = sorted(set(allowed).difference(seen_allowed))
    violations.extend(f"missing-or-renamed:{name}" for name in missing)
    return sorted(violations)


def scan_archive(path: Path) -> list[str]:
    violations: list[str] = []
    with tarfile.open(path, "r:gz") as archive:
        for member in archive.getmembers():
            if not member.isfile():
                continue
            source = archive.extractfile(member)
            if _path_matches(member.name) or (
                source is not None and _matches(source.read())
            ):
                violations.append(member.name)
    return sorted(violations)


def tracked(root: Path) -> list[str]:
    output = subprocess.run(
        ["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard"],
        cwd=root,
        check=True,
        capture_output=True,
    ).stdout
    return [
        value.decode("utf-8")
        for value in output.split(b"\0")
        if value and (root / value.decode("utf-8")).is_file()
    ]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--archive", type=Path)
    parser.add_argument("--inventory", type=Path, default=INVENTORY)
    args = parser.parse_args()
    violations = (
        scan_archive(args.archive)
        if args.archive
        else scan_files(args.root, tracked(args.root), args.inventory)
    )
    if violations:
        print("forbidden coupling:\n" + "\n".join(violations))
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
