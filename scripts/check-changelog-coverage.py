#!/usr/bin/env python3
"""Require a described changelog bullet for every ticket merged since the prior release."""

from __future__ import annotations

import argparse
from pathlib import Path
import re
import subprocess
import sys

MERGE_TICKET = re.compile(r"^Merge .*\btickets/([0-9]+)-")
RECORD_TICKET = re.compile(r"^sdlc/records/([0-9]+)-")
STABLE_TAG = re.compile(r"^v[0-9]+\.[0-9]+\.[0-9]+$")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tag", required=True)
    parser.add_argument("--changelog", type=Path, default=Path("CHANGELOG.md"))
    return parser.parse_args()


def run_git(*arguments: str) -> str:
    completed = subprocess.run(["git", *arguments], capture_output=True, text=True)
    if completed.returncode != 0:
        raise RuntimeError(completed.stderr.strip() or "git failed")
    return completed.stdout


def version_key(tag: str) -> tuple[int, ...]:
    return tuple(int(part) for part in tag[1:].split("."))


def previous_tag(tag: str) -> str | None:
    tags = run_git("tag", "--merged", tag, "--sort=version:refname").splitlines()
    older = [
        candidate
        for candidate in tags
        if STABLE_TAG.fullmatch(candidate) and version_key(candidate) < version_key(tag)
    ]
    return max(older, key=version_key) if older else None


def merge_subject_tickets(previous: str, tag: str) -> set[str]:
    subjects = run_git("log", "--format=%s", f"{previous}..{tag}").splitlines()
    return {
        match.group(1) for subject in subjects if (match := MERGE_TICKET.match(subject))
    }


def record_tickets(previous: str, tag: str) -> set[str]:
    names = run_git(
        "diff", "--name-only", "--diff-filter=A", previous, tag, "--", "sdlc/records/"
    ).splitlines()
    return {match.group(1) for name in names if (match := RECORD_TICKET.match(name))}


def merged_tickets(previous: str, tag: str) -> set[str]:
    # Neither source alone is complete: a record can lag a merge, and a
    # merge subject can be rewritten or absent. Fail closed on the union.
    return merge_subject_tickets(previous, tag) | record_tickets(previous, tag)


def section_text(path: Path, tag: str) -> tuple[str, str]:
    try:
        content = path.read_text(encoding="utf-8")
    except OSError as error:
        raise RuntimeError(f"cannot read {path}: {error}") from error
    version = re.escape(tag.removeprefix("v"))
    headings = (
        (rf"^## {version}(?:\s+—[^\n]*)?\s*$", tag.removeprefix("v")),
        (r"^## Unreleased\s*$", "Unreleased"),
    )
    for heading, label in headings:
        match = re.search(heading + r"([\s\S]*?)(?=^## |\Z)", content, re.MULTILINE)
        if match is not None:
            return label, match.group(1)
    raise RuntimeError(
        f"{path} has neither a {tag.removeprefix('v')} nor an Unreleased section"
    )


def described_tickets(section: str) -> set[str]:
    found: set[str] = set()
    bullets: list[str] = []
    current: list[str] = []
    for line in section.splitlines():
        bullet = re.match(r"^\s*[-*]\s+(.+)$", line)
        if bullet is not None:
            if current:
                bullets.append(" ".join(current))
            current = [bullet.group(1).strip()]
        elif current and (line.startswith("  ") or line.strip()):
            current.append(line.strip())
        elif current:
            bullets.append(" ".join(current))
            current = []
    if current:
        bullets.append(" ".join(current))
    for text in bullets:
        for ticket in re.findall(r"(?<![0-9])([0-9]+)(?![0-9])", text):
            marker = re.compile(
                rf"(?:\(#?{re.escape(ticket)}\)|#{re.escape(ticket)}\b|\b{re.escape(ticket)}\b)"
            )
            remainder = marker.sub("", text)
            # A bare number list is not a description: remove every
            # remaining bare number token and the separators around
            # them, then require at least three word characters of
            # described text.
            remainder = re.sub(r"(?<![0-9])[0-9]+(?![0-9])", "", remainder)
            remainder = remainder.strip(" .,:;-|/")
            if len(re.findall(r"[^\W\d_]", remainder, re.UNICODE)) >= 3:
                found.add(ticket)
    return found


def main() -> int:
    args = parse_args()
    if not STABLE_TAG.fullmatch(args.tag):
        print(
            f"changelog coverage check failed: stable v-prefixed tag required: {args.tag}",
            file=sys.stderr,
        )
        return 1
    try:
        previous = previous_tag(args.tag)
        if previous is None:
            print("no previous release; nothing to cover")
            return 0
        tickets = merged_tickets(previous, args.tag)
        label, section = section_text(args.changelog, args.tag)
        described = described_tickets(section)
    except RuntimeError as error:
        print(f"changelog coverage check failed: {error}", file=sys.stderr)
        return 1
    missing = sorted(tickets - described, key=int)
    if missing:
        print(
            f"CHANGELOG {label} is missing described bullets for tickets merged since {previous}: "
            + ", ".join(missing),
            file=sys.stderr,
        )
        return 1
    print(
        f"changelog {label} covers all {len(tickets)} tickets merged since {previous}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
