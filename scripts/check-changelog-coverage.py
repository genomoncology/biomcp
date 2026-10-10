#!/usr/bin/env python3
"""Require a described changelog bullet for every ticket merged since the prior release."""

from __future__ import annotations

import argparse
from pathlib import Path
import re
import subprocess
import sys

MERGE_TICKET = re.compile(r"^Merge .*\btickets/([0-9]+)-")
# Landings pushed straight to main carry the ticket in the subject
# itself: "Land 1299: make the article search deadline honest" and
# "Land 2044 and 2045: MCP errors carry no local path" (a landing
# may name several tickets, so no colon is required right after the
# number). Only the leading number counts, so a Land subject that
# mentions another ticket in its text does not pull that ticket in.
LAND_TICKET = re.compile(r"^Land ([0-9]+)(?![0-9])")
# A landing may name its branch instead: "Land tickets/2042-fix".
LAND_BRANCH_TICKET = re.compile(r"^Land tickets/([0-9]+)-")
# Ticket files only: sdlc/tickets/NNNN-*.md. The archive/ and
# drafts/ subdirectories hold dead tickets and never count.
TICKET_FILE = re.compile(r"^sdlc/tickets/([0-9]+)-")
STATUS_LINE = re.compile(r"^Status:\s*(.+)$", re.IGNORECASE)
# Ticket-numbered records only. A date-shaped name (dddd-dd-dd-*)
# is a dated note, never a ticket record, whatever the year; any
# other four-digit number with any slug (digit-start and
# capital-start included) is a ticket record, whatever its number.
RECORD_TICKET = re.compile(r"^sdlc/records/([0-9]{4})-(?![0-9]{2}-[0-9]{2}-)")
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


def landing_subject_tickets(previous: str, tag: str) -> set[str]:
    subjects = run_git("log", "--format=%s", f"{previous}..{tag}").splitlines()
    return {
        match.group(1)
        for subject in subjects
        for match in (
            MERGE_TICKET.match(subject),
            LAND_TICKET.match(subject),
            LAND_BRANCH_TICKET.match(subject),
        )
        if match is not None
    }


def record_tickets(previous: str, tag: str) -> set[str]:
    names = run_git(
        "diff", "--name-only", "--diff-filter=A", previous, tag, "--", "sdlc/records/"
    ).splitlines()
    return {
        match.group(1)
        for name in names
        if (match := RECORD_TICKET.match(name)) is not None
    }


def status_token(content: str) -> str:
    """The status token pm reads from a ticket file.

    Mirrors pm's grammar: the first `Status:` line within the first
    25 lines, emphasis stripped, cut at the first sentence terminator
    (a period between digits is a decimal, not a terminator), trimmed
    and lowercased. Real tickets spell completion "complete.",
    "COMPLETE. Built ..." and "complete. Absorbed by ticket ...".
    """
    for line in content.splitlines()[:25]:
        match = STATUS_LINE.match(line)
        if match is None:
            continue
        raw = match.group(1).strip().replace("*", "")
        for index, char in enumerate(raw):
            if char in ".,;:":
                before = raw[index - 1] if index else ""
                after = raw[index + 1] if index + 1 < len(raw) else ""
                if not (char == "." and before.isdigit() and after.isdigit()):
                    return raw[:index].strip().lower()
        return raw.strip().lower()
    return ""


def completed_status_tickets(previous: str, tag: str) -> set[str]:
    names = run_git(
        "diff", "--name-only", "--diff-filter=A", previous, tag, "--", "sdlc/tickets/"
    ).splitlines()
    found: set[str] = set()
    for name in names:
        match = TICKET_FILE.match(name)
        if match is None:
            continue
        # The status is read at the tag, not at the add commit: a
        # ticket filed mid-cycle and closed later counts once it
        # reads complete at the release.
        if status_token(run_git("show", f"{tag}:{name}")) == "complete":
            found.add(match.group(1))
    return found


def merged_tickets(previous: str, tag: str) -> set[str]:
    # No single source is complete: a landing can be a Land subject,
    # an old-pattern merge subject, or a record file, and a ticket can
    # close with none of them when its work lands as plain commits
    # (2010, absorbed by 1304). A record can lag a landing while a
    # subject can be rewritten or absent. Fail closed on the union,
    # one ticket once across every shape.
    return (
        landing_subject_tickets(previous, tag)
        | record_tickets(previous, tag)
        | completed_status_tickets(previous, tag)
    )


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


# Words that carry no description: a bullet made only of these
# and ticket numbers is a bare list, not coverage.
BOILERPLATE_WORDS = re.compile(
    r"\b(?:tickets?|see|and|or|the|for|with|fix(?:es|ed)?|change(?:s|d)?|update(?:s|d)?|add(?:s|ed)?)\b",
    re.IGNORECASE,
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
            # described text. Label words like "Tickets" or "see" are
            # boilerplate and do not count as description either.
            remainder = re.sub(r"(?<![0-9])[0-9]+(?![0-9])", "", remainder)
            remainder = BOILERPLATE_WORDS.sub(" ", remainder)
            remainder = remainder.strip(" .,:;-|/()")
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
