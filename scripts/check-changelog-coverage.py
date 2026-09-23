#!/usr/bin/env python3
"""Require the CHANGELOG Unreleased section to name every ticket merged since the previous release."""

from __future__ import annotations

import argparse
import re
import subprocess
import sys

TICKET_PATTERN = re.compile(r"(?<![0-9])(1[0-9]{3})(?![0-9])")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", required=True)
    parser.add_argument("--tag", required=True)
    parser.add_argument("--changelog", default="CHANGELOG.md")
    return parser.parse_args()


def run_gh(*arguments: str) -> str:
    completed = subprocess.run(
        ["gh", "api", *arguments],
        capture_output=True,
        text=True,
    )
    if completed.returncode != 0:
        raise RuntimeError(completed.stderr.strip() or "gh api failed")
    return completed.stdout


def version_key(tag: str) -> list[int]:
    return [int(part) for part in re.findall(r"[0-9]+", tag)]


def previous_release_tag(repository: str, tag: str) -> str | None:
    """Return the newest published release tag strictly older than this tag."""
    listing = run_gh(f"repos/{repository}/releases", "--jq", ".[].tag_name")
    older = [candidate for candidate in listing.splitlines()
             if candidate.strip() and candidate != tag
             and version_key(candidate) < version_key(tag)]
    if not older:
        return None
    return max(older, key=version_key)


def merged_tickets(repository: str, previous: str, tag: str) -> set[str]:
    """Collect ticket references from every commit between the previous release and this tag."""
    listing = run_gh(
        f"repos/{repository}/compare/{previous}...{tag}",
        "--jq",
        ".commits[].commit.message",
    )
    tickets: set[str] = set()
    for message in listing.splitlines():
        tickets.update(TICKET_PATTERN.findall(message))
    return tickets


def unreleased_text(changelog_path: str) -> str:
    try:
        content = open(changelog_path, encoding="utf-8").read()
    except OSError as error:
        raise RuntimeError(f"cannot read {changelog_path}: {error}") from error
    match = re.search(r"^## Unreleased\s*$([\s\S]*?)(?=^## )", content, re.MULTILINE)
    if match is None:
        raise RuntimeError(f"{changelog_path} has no Unreleased section")
    return match.group(1)


def main() -> int:
    arguments = parse_args()
    try:
        previous = previous_release_tag(arguments.repository, arguments.tag)
        if previous is None:
            print("no previous release; nothing to cover")
            return 0
        tickets = merged_tickets(arguments.repository, previous, arguments.tag)
        if not tickets:
            print(f"no ticket-bearing commits between {previous} and {arguments.tag}")
            return 0
        recorded = unreleased_text(arguments.changelog)
        missing = sorted(ticket for ticket in tickets
                         if not re.search(rf"(?<![0-9]){ticket}(?![0-9])", recorded))
    except RuntimeError as error:
        print(f"changelog coverage check failed: {error}", file=sys.stderr)
        return 1
    if missing:
        print(
            "CHANGELOG Unreleased is missing tickets merged since "
            f"{previous}: {', '.join(missing)}",
            file=sys.stderr,
        )
        return 1
    print(f"changelog covers all {len(tickets)} tickets merged since {previous}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
