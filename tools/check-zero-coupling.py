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
import sys
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
SEGMENTED_RECORD = "sdlc/records/1183-remove-" + "bio" + "data-from-biomcp-0-9.md"
SEGMENT_BEGIN = b"<!-- biomcp-1183-hosted-evidence-begin -->"
SEGMENT_END = b"<!-- biomcp-1183-hosted-evidence-end -->"
PENDING_EVIDENCE = b"\nstatus: pending\n"
HOSTED_JOB = rb"https://github\.com/genomoncology/biomcp/actions/runs/[0-9]+/job/[0-9]+"
CLOSED_EVIDENCE = re.compile(
    rb"\nstatus: closed\n"
    rb"reviewed-sha: [0-9a-f]{40}\n"
    rb"canonical-gates: success " + HOSTED_JOB + rb"\n"
    rb"full-features: success " + HOSTED_JOB + rb"\n"
    rb"windows-contracts: success " + HOSTED_JOB + rb"\n"
    rb"repository-contracts: success " + HOSTED_JOB + rb"\n"
)
SEGMENTED_KEYS = {"version", "mode", "prefix_sha256", "suffix_sha256"}


def _text(data: bytes) -> str | None:
    if b"\0" in data:
        return None
    try:
        return data.decode("utf-8")
    except UnicodeDecodeError:
        return None


DECLARATION_FILES = (
    "sdlc/pm-forbidden-names.json",
    "sdlc/pm-forbidden-names.example.json",
)
LOCAL_DECLARATION_FILE = DECLARATION_FILES[0]


def declared_names(root: Path) -> tuple[str, ...]:
    """Private project names the local forbidden-name declaration carries.

    This repository is public, so no tracked file may spell a real private
    name. The declaration lives in two files: the gitignored
    sdlc/pm-forbidden-names.json holds the real names where they are known,
    and the checked-in sdlc/pm-forbidden-names.example.json holds inert
    placeholders that document the shape. Both are read and unioned, so the
    guard runs everywhere the example ships and tightens wherever the real
    file exists. Each entry may be base64-encoded, the same convention the
    coupling receipts use; a declared value that does not decode is matched
    literally, so a plaintext declaration still guards. An absent or
    malformed file declares nothing.
    """
    names: list[str] = []
    for relative in DECLARATION_FILES:
        _declare_into(names, root, relative)
    return tuple(names)


def _names_from(root: Path, files: tuple[str, ...]) -> tuple[str, ...]:
    """Read the named declaration files and union what they declare."""
    names: list[str] = []
    for relative in files:
        _declare_into(names, root, relative)
    return tuple(names)


def _declare_into(names: list[str], root: Path, relative: str) -> None:
    try:
        declared = json.loads((root / relative).read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return
    entries = declared.get("forbiddenNames") if isinstance(declared, dict) else None
    if not isinstance(entries, list):
        return
    for entry in entries:
        if not isinstance(entry, str) or not entry:
            continue
        try:
            name = base64.b64decode(entry.encode("ascii"), validate=True).decode("utf-8")
        except (UnicodeError, ValueError):
            name = entry
        name = name.casefold()
        if name not in names:
            names.append(name)


def local_declaration_names(root: Path) -> tuple[str, ...]:
    """Names only the gitignored local declaration carries.

    Split out of declared_names so a run that requires the real names can
    tell an inert guard (example placeholders only) from a live one.
    """
    return _names_from(root, (LOCAL_DECLARATION_FILE,))


def _forbidden_text(text: str, private: tuple[str, ...] = ()) -> bool:
    folded = text.casefold()
    if any(value in folded for value in FORBIDDEN + private):
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


def _matches(data: bytes, private: tuple[str, ...] = ()) -> bool:
    text = _text(data)
    return text is not None and _forbidden_text(text, private)


def _path_matches(path: str, private: tuple[str, ...] = ()) -> bool:
    return _forbidden_text(path, private)


def _allowlist(path: Path = INVENTORY) -> dict[str, object]:
    encoded = json.loads(path.read_text(encoding="utf-8"))
    return {
        base64.b64decode(name).decode("utf-8"): digest
        for name, digest in encoded.items()
    }


def _segment_parts(data: bytes) -> tuple[bytes, bytes, bytes] | None:
    if data.count(SEGMENT_BEGIN) != 1 or data.count(SEGMENT_END) != 1:
        return None
    begin = data.index(SEGMENT_BEGIN)
    end = data.index(SEGMENT_END)
    block_start = begin + len(SEGMENT_BEGIN)
    if block_start >= end:
        return None
    return data[:begin], data[block_start:end], data[end + len(SEGMENT_END) :]


def _valid_evidence(block: bytes) -> bool:
    return block == PENDING_EVIDENCE or CLOSED_EVIDENCE.fullmatch(block) is not None


def _entry_allows(name: str, data: bytes, entry: object) -> bool:
    if isinstance(entry, str):
        return re.fullmatch(r"[0-9a-f]{64}", entry) is not None and (
            hashlib.sha256(data).hexdigest() == entry
        )
    if name != SEGMENTED_RECORD or not isinstance(entry, dict):
        return False
    if set(entry) != SEGMENTED_KEYS:
        return False
    if entry.get("version") != 1 or entry.get("mode") != "hosted-evidence-v1":
        return False
    prefix_digest = entry.get("prefix_sha256")
    suffix_digest = entry.get("suffix_sha256")
    if not isinstance(prefix_digest, str) or not isinstance(suffix_digest, str):
        return False
    if not re.fullmatch(r"[0-9a-f]{64}", prefix_digest) or not re.fullmatch(
        r"[0-9a-f]{64}", suffix_digest
    ):
        return False
    parts = _segment_parts(data)
    if parts is None:
        return False
    prefix, block, suffix = parts
    return (
        hashlib.sha256(prefix).hexdigest() == prefix_digest
        and hashlib.sha256(suffix).hexdigest() == suffix_digest
        and _valid_evidence(block)
    )


def closure_diff_is_block_only(before: bytes, after: bytes) -> bool:
    before_parts = _segment_parts(before)
    after_parts = _segment_parts(after)
    if before_parts is None or after_parts is None:
        return False
    before_prefix, before_block, before_suffix = before_parts
    after_prefix, after_block, after_suffix = after_parts
    return (
        before_prefix == after_prefix
        and before_suffix == after_suffix
        and before_block == PENDING_EVIDENCE
        and CLOSED_EVIDENCE.fullmatch(after_block) is not None
    )


def scan_files(
    root: Path,
    names: list[str],
    inventory: Path = INVENTORY,
    private: tuple[str, ...] | None = None,
) -> list[str]:
    if private is None:
        private = declared_names(root)
    allowed = _allowlist(inventory)
    violations: list[str] = []
    seen_allowed: set[str] = set()
    for name in names:
        data = (root / name).read_bytes()
        if not (_path_matches(name, private) or _matches(data, private)):
            continue
        if _entry_allows(name, data, allowed.get(name)):
            seen_allowed.add(name)
        else:
            violations.append(name)
    missing = sorted(set(allowed).difference(seen_allowed))
    violations.extend(f"missing-or-renamed:{name}" for name in missing)
    return sorted(violations)


def scan_archive(path: Path, private: tuple[str, ...] = ()) -> list[str]:
    violations: list[str] = []
    with tarfile.open(path, "r:gz") as archive:
        for member in archive.getmembers():
            if not member.isfile():
                continue
            source = archive.extractfile(member)
            if _path_matches(member.name, private) or (
                source is not None and _matches(source.read(), private)
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


def missing_local_names_error(root: Path) -> str | None:
    """Loud failure sentence when a required local declaration is absent.

    A run with `--require-local-names` (continuous integration) must not
    pass silently while guarding only inert example placeholders: without
    the real names the scan cannot catch a planted private name, so the
    checker refuses to pretend it guarded anything (ticket 2038 finding 7,
    2035 #11).
    """
    if local_declaration_names(root):
        return None
    return (
        f"the local forbidden-name declaration {LOCAL_DECLARATION_FILE} is "
        "missing or declares no names, so this run cannot guard private "
        "project names. Copy sdlc/pm-forbidden-names.example.json to "
        f"{LOCAL_DECLARATION_FILE}, list the real private project names, "
        "and re-run."
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--archive", type=Path)
    parser.add_argument("--inventory", type=Path, default=INVENTORY)
    parser.add_argument(
        "--require-local-names",
        action="store_true",
        help="fail when the gitignored local declaration is absent "
        "(continuous integration); local runs print a note instead",
    )
    args = parser.parse_args(argv)
    # The declaration resolves under the scanned tree (`--root`), not beside
    # the script: a scan of a fixture or a second checkout must read that
    # tree's own gitignored declaration (ticket 2044).
    if args.require_local_names:
        message = missing_local_names_error(args.root)
        if message is not None:
            print(message)
            return 2
    elif not local_declaration_names(args.root):
        # Say it plainly instead of passing silently: an inert guard is a
        # fact the operator should see (ticket 2038 finding 7, 2035 #11).
        print(
            f"note: no local forbidden-name declaration at "
            f"{LOCAL_DECLARATION_FILE}; guarding only the tracked example "
            "names",
            file=sys.stderr,
        )
    private = declared_names(args.root)
    violations = (
        scan_archive(args.archive, private)
        if args.archive
        else scan_files(args.root, tracked(args.root), args.inventory, private)
    )
    if violations:
        print("forbidden coupling:\n" + "\n".join(violations))
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
