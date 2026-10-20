#!/usr/bin/env python3
"""Require a described changelog bullet for every ticket merged since the prior release.

At a tag, `--tag` is the release bar: every ticket merged since the
previous tag must carry a described bullet in the tagged section. On
a branch, `--branch` is the early form: the tickets landed since the
last tag plus the branch's own ticket need bullets only when their
diffs change shipped behavior, checked against the live section or the
matching dated section of a stable release candidate.
"""

from __future__ import annotations

import argparse
import os
from collections.abc import Callable
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
# A landing branch names its ticket: ticket/2074-fix-the-gate. The
# older tickets/ spelling still runs in CI, so both count.
BRANCH_NAME_TICKET = re.compile(r"^tickets?/([0-9]+)-")
# Shipped behavior (2026-10-14 ruling): a diff that touches these
# paths changes what a release ships; anything else — tests,
# scripts, CI, sdlc records — changes the repository only.
SHIPPED_ROOTS = ("src/", "templates/", "skills/", "docs/", "crates/")
SHIPPED_FILES = ("Cargo.toml", "Cargo.lock", "install.sh", "pyproject.toml")

def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--tag", help="the release bar: cover every ticket since the previous tag")
    mode.add_argument(
        "--branch",
        action="store_true",
        help="the branch form: cover tickets whose diffs change shipped behavior",
    )
    parser.add_argument("--changelog", type=Path, default=Path("CHANGELOG.md"))
    parser.add_argument("--revision", help="untagged release candidate revision (with --tag for its version)")
    return parser.parse_args()


def run_git(*arguments: str) -> str:
    completed = subprocess.run(["git", *arguments], capture_output=True, text=True)
    if completed.returncode != 0:
        raise RuntimeError(completed.stderr.strip() or "git failed")
    return completed.stdout


def version_key(tag: str) -> tuple[int, ...]:
    return tuple(int(part) for part in tag[1:].split("."))


def previous_tag(tag: str, revision: str | None = None) -> str | None:
    tags = run_git("tag", "--merged", revision or tag, "--sort=version:refname").splitlines()
    older = [
        candidate
        for candidate in tags
        if STABLE_TAG.fullmatch(candidate) and version_key(candidate) < version_key(tag)
    ]
    return max(older, key=version_key) if older else None


def landing_subject_tickets(previous: str, tag: str) -> set[str]:
    subjects = run_git("log", "--format=%s", f"{previous}..{tag}").splitlines()
    found: set[str] = set()
    for subject in subjects:
        found |= subject_tickets(subject)
    return found


def subject_tickets(subject: str) -> set[str]:
    """The tickets a landing subject names, across every shape."""
    return {
        match.group(1)
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
    """The status token read from a ticket file.

    Uses the ticket status grammar: the first `Status:` line within the first
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


def belongs_to_release_line(content: str | None, version: str) -> bool:
    """A copied record for another release line is not a landing here."""
    line = re.match(r"v(\d+)\.(\d+)", version)
    milestone = re.search(r"^Milestone:\s*(\S+)", content or "", re.MULTILINE)
    return line is None or milestone is None or milestone.group(1).startswith(
        f"{line.group(1)}.{line.group(2)}"
    )


def completed_status_tickets(previous: str, tag: str, version: str | None = None) -> set[str]:
    names = run_git(
        "diff", "--name-only", "--diff-filter=A", previous, tag, "--", "sdlc/tickets/"
    ).splitlines()
    # The tag's own release line: a v0.9.2 tag covers milestone 0.9.x
    # tickets only. Carried tickets from another line (the 1.0
    # planning records landed mid-cycle) complete on their own line
    # and never demand a bullet here.
    found: set[str] = set()
    for name in names:
        match = TICKET_FILE.match(name)
        if match is None:
            continue
        content = run_git("show", f"{tag}:{name}")
        # The status is read at the tag, not at the add commit: a
        # ticket filed mid-cycle and closed later counts once it
        # reads complete at the release.
        if status_token(content) != "complete":
            continue
        if not belongs_to_release_line(content, version or tag):
            continue
        found.add(match.group(1))
    return found


def merged_tickets(previous: str, tag: str, version: str | None = None) -> set[str]:
    # No single source is complete: a landing can be a Land subject,
    # an old-pattern merge subject, or a record file, and a ticket can
    # close with none of them when its work lands as plain commits
    # (2010, absorbed by 1304). A record can lag a landing while a
    # subject can be rewritten or absent. Fail closed on the union,
    # one ticket once across every shape.
    read_ticket = tag_ticket_reader(tag)
    records = {
        ticket for ticket in record_tickets(previous, tag)
        if belongs_to_release_line(read_ticket(ticket), version or tag)
    }
    candidates = (
        landing_subject_tickets(previous, tag)
        | records
        | completed_status_tickets(previous, tag, version)
    )
    release = version or tag
    required: set[str] = set()
    for ticket in candidates:
        content = read_ticket(ticket)
        milestone = re.search(r"^Milestone:\s*(\S+)", content or "", re.MULTILINE)
        # Prior-release housekeeping can close after that release's tag.
        # Missing or nonstandard records still demand a description.
        if milestone is not None and STABLE_TAG.fullmatch("v" + milestone.group(1)):
            if version_key("v" + milestone.group(1)) < version_key(release):
                if not landed_in_milestone(ticket, release.removeprefix("v"), previous, tag):
                    continue
        required.add(ticket)
    return required


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


def latest_tag(head: str = "HEAD") -> str | None:
    """The newest stable tag an object's history carries."""
    tags = run_git("tag", "--merged", head, "--sort=version:refname").splitlines()
    stable = [tag for tag in tags if STABLE_TAG.fullmatch(tag)]
    return max(stable, key=version_key) if stable else None


def landing_files(previous: str, head: str) -> dict[str, set[str]]:
    """The files each landing since `previous` changed, by ticket.

    A landing is a commit whose subject names its ticket, the same
    subject shapes the tag run reads. A landing is charged with the
    diff against its first parent, `git diff --name-only <sha>^1
    <sha>`, so a merge landing is charged with its branch's own
    files alone: a `diff-tree -m --first-parent` also adds the diff
    against the second parent, which charged a stale branch's
    landing with everything main gained since it forked (the 2116
    landing, ticket 2125). A ticket that closed with no landing
    subject (record or status only) has no diff here and never
    needs a bullet on a branch; the tag run still demands it at the
    release.
    """
    files: dict[str, set[str]] = {}
    log = run_git("log", "--format=%H%x09%s", f"{previous}..{head}")
    for line in log.splitlines():
        if not line:
            continue
        sha, _, subject = line.partition("\t")
        for ticket in subject_tickets(subject):
            changed = run_git("diff", "--name-only", f"{sha}^1", sha).splitlines()
            files.setdefault(ticket, set()).update(changed)
    return files


def branch_diff_files(base_ref: str = "origin/main") -> set[str]:
    """The branch's own contribution: the diff since the merge-base
    with the trunk."""
    merge_base = run_git("merge-base", base_ref, "HEAD").strip()
    if not merge_base:
        raise RuntimeError(f"no merge-base with {base_ref}")
    return set(run_git("diff", "--name-only", merge_base, "HEAD").splitlines())


def touches_shipped_behavior(files: set[str]) -> bool:
    return any(
        path.startswith(SHIPPED_ROOTS) or path in SHIPPED_FILES for path in files
    )


LIVE_HEADING = re.compile(r"^## (Unreleased|[0-9]+\.[0-9]+\.[0-9]+)\s*$", re.MULTILINE)


def live_section_text(path: Path) -> tuple[str, str] | None:
    """The dev cycle's section, or None when the changelog has none.

    A released section carries its date (`## 0.9.2 — 2026-10-11`),
    so a heading with a suffix is closed; a leading bare version heading
    is the release under way, and `## Unreleased` is the same thing
    before its number is chosen.
    """
    try:
        content = path.read_text(encoding="utf-8")
    except OSError as error:
        raise RuntimeError(f"cannot read {path}: {error}") from error
    first = re.search(r"^## .+$", content, re.MULTILINE)
    heading = LIVE_HEADING.fullmatch(first.group(0)) if first is not None else None
    if heading is None:
        return None
    body = re.search(
        re.escape(heading.group(0)) + r"([\s\S]*?)(?=^## |\Z)", content, re.MULTILINE
    )
    assert body is not None  # the heading just matched, so its body exists
    return heading.group(1), body.group(1)


def branch_section_text(path: Path) -> tuple[str, str] | None:
    """Use the live section, or the stable candidate's matching closed section."""
    section = live_section_text(path)
    if section is not None:
        return section
    manifest = Path("Cargo.toml")
    if not manifest.is_file():
        return None
    package = re.search(
        r"(?ms)^\[package\]\s*\n(.*?)(?=^\[|\Z)",
        manifest.read_text(encoding="utf-8"),
    )
    version = re.search(
        r'^version\s*=\s*"([0-9]+\.[0-9]+\.[0-9]+)"\s*(?:#.*)?$',
        package.group(1) if package is not None else "",
        re.MULTILINE,
    )
    if version is None:
        return None
    content = path.read_text(encoding="utf-8")
    first = re.search(r"^## (.+)$", content, re.MULTILINE)
    if first is None or re.fullmatch(
        re.escape(version.group(1)) + r"\s+—\s+[0-9]{4}-[0-9]{2}-[0-9]{2}\s*",
        first.group(1),
    ) is None:
        return None
    return section_text(path, "v" + version.group(1))


def branch_main(changelog: Path) -> int:
    """The branch form of the gate: fail a branch that changes
    shipped behavior without a described bullet in the current section.
    """
    ref_name = os.environ.get("GITHUB_REF_NAME") or run_git("branch", "--show-current").strip()
    previous = latest_tag()
    if previous is None:
        print("no previous release; nothing to cover")
        return 0
    tickets_files = landing_files(previous, "HEAD")
    branch_match = BRANCH_NAME_TICKET.match(ref_name)
    if branch_match is not None:
        tickets_files.setdefault(branch_match.group(1), set()).update(
            branch_diff_files()
        )
    needing = sorted(
        (
            ticket
            for ticket, files in tickets_files.items()
            if touches_shipped_behavior(files)
        ),
        key=int,
    )
    section = branch_section_text(changelog)
    described = described_tickets(section[1]) if section is not None else set()
    if not needing:
        print(f"no ticket since {previous} changed shipped behavior")
        return 0
    if section is None:
        print(
            "the changelog has no Unreleased or under-way version section, or "
            "matching stable candidate section, to "
            f"carry bullets for: {', '.join(needing)}",
            file=sys.stderr,
        )
        return 1
    label, body = section
    missing = sorted(set(needing) - described, key=int)
    if missing:
        print(
            f"CHANGELOG {label} is missing described bullets for tickets that "
            f"changed shipped behavior since {previous}: " + ", ".join(missing),
            file=sys.stderr,
        )
        return 1
    print(
        f"changelog {label} covers all {len(needing)} tickets that changed "
        f"shipped behavior since {previous}"
    )
    return 0


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
    fence = ""
    ticket_group = r"\(\s*[0-9]+(?:\s*(?:,|and)\s*[0-9]+)*\s*\)"
    joined = re.compile(ticket_group + r"[ .,;]*[-*]\s+\S.*" + ticket_group + r"[ .,;]*$")
    for line in section.splitlines():
        marker = re.match(r"^ {0,3}(`{3,}|~{3,})(.*)$", line)
        in_code = bool(fence)
        if marker is not None:
            token, tail = marker.groups()
            if not fence:
                fence = token
            elif token[0] == fence[0] and len(token) >= len(fence) and not tail.strip():
                fence = ""
            in_code = True
        bullet = re.match(r"^\s*[-*]\s+(.+)$", line)
        # Literal Markdown examples are not entries. Require both closing
        # ticket groups so a numbered value followed by a dash stays prose.
        visible = re.sub(r"(`+).*?\1", "", line)
        if not in_code and (bullet is not None or current) and joined.search(visible):
            raise RuntimeError("joined changelog entries: put each bullet on its own line")
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
        # A bullet covers a ticket when the bullet's closing group
        # names it. The closing group sits at the bullet's end and
        # reads (NNNN), (NNNN, MMMM) or (NNNN and MMMM); a number in
        # the prose ("tracks 2039") is not coverage.
        stripped = text.strip().rstrip(" .,;")
        closing = ""
        if stripped.endswith(")") and "(" in stripped:
            opening = stripped.rindex("(")
            closing = stripped[opening:]
        group = (
            re.findall(r"[0-9]+", closing)
            if closing
            and re.fullmatch(
                r"\(\s*[0-9]+(?:\s*(?:,|and)\s*[0-9]+)*\s*\)", closing
            )
            else []
        )
        for ticket in group:
            remainder = re.sub(rf"\b{re.escape(ticket)}\b", "", text)
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


# A cited ticket whose file is missing or milestone-less cannot prove
# another release: the milestone convention arrived mid-cycle and
# some early tickets carry no ticket file at all, so the tag run
# tolerates both and fails only a ticket whose file names another
# milestone.
NO_TICKET_FILE = "no ticket file"
NO_MILESTONE = "no milestone"


def cited_tickets(section: str) -> set[str]:
    """Ticket numbers a changelog section cites in parentheses."""
    found: set[str] = set()
    for group in re.findall(r"\(([^)]+)\)", section):
        for token in re.split(r",|and", group):
            token = token.strip()
            if token.isdigit():
                found.add(token)
    return found


def tag_ticket_reader(tag: str) -> Callable[[str], str | None]:
    """Read a cited ticket's file as it stands at `tag`.

    Ticket files move after a release, so the gate reads the tagged
    tree, not the working tree.
    """
    paths = {
        match.group(1): name
        for name in run_git(
            "ls-tree", "--name-only", tag, "--", "sdlc/tickets/"
        ).splitlines()
        if (match := TICKET_FILE.match(name)) is not None
    }

    def read(ticket: str) -> str | None:
        path = paths.get(ticket)
        return run_git("show", f"{tag}:{path}") if path is not None else None

    return read


def landed_in_milestone(
    ticket: str, milestone: str, previous: str, head: str
) -> bool:
    """Keep a landed batch in its release when unfinished work moves later."""
    log = run_git("log", "--format=%H%x09%s", f"{previous}..{head}")
    for line in log.splitlines():
        revision, _, subject = line.partition("\t")
        if ticket not in subject_tickets(subject):
            continue
        content = tag_ticket_reader(revision)(ticket)
        if content is not None and re.search(
            rf"^Milestone:\s*{re.escape(milestone)}\s*$", content, re.MULTILINE
        ):
            return True
    return False


def milestone_problems(
    section: str, milestone: str, read_ticket: Callable[[str], str | None],
    *, previous: str | None = None, head: str = "HEAD",
) -> dict[str, str]:
    """Why each ticket a section cites is not a ticket of `milestone`.

    The value is NO_TICKET_FILE when no ticket file exists, NO_MILESTONE
    when the file carries no Milestone line, and the milestone the file
    carries otherwise. Only the last value proves the cited ticket
    belongs to another release, unless a landing since `previous` records
    its batch in this milestone. Missing records still fail closed.
    """
    problems: dict[str, str] = {}
    for ticket in sorted(cited_tickets(section), key=int):
        content = read_ticket(ticket)
        if content is None:
            problems[ticket] = NO_TICKET_FILE
            continue
        found = re.search(r"^Milestone:\s*(\S+)", content, re.MULTILINE)
        if found is None:
            problems[ticket] = NO_MILESTONE
        elif found.group(1) != milestone:
            if previous is None or not landed_in_milestone(ticket, milestone, previous, head):
                problems[ticket] = found.group(1)
    return problems


def main() -> int:
    args = parse_args()
    if args.branch:
        if args.revision:
            print("--revision requires --tag", file=sys.stderr)
            return 1
        try:
            return branch_main(args.changelog)
        except RuntimeError as error:
            print(f"changelog coverage check failed: {error}", file=sys.stderr)
            return 1
    if not STABLE_TAG.fullmatch(args.tag):
        print(
            f"changelog coverage check failed: stable v-prefixed tag required: {args.tag}",
            file=sys.stderr,
        )
        return 1
    try:
        revision = args.revision or args.tag
        previous = previous_tag(args.tag, revision)
        if previous is None:
            print("no previous release; nothing to cover")
            return 0
        tickets = merged_tickets(previous, revision, args.tag)
        label, section = section_text(args.changelog, args.tag)
        described = described_tickets(section)
        problems = milestone_problems(
            section, args.tag.removeprefix("v"), tag_ticket_reader(revision),
            previous=previous, head=revision,
        )
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
    conflicts = {
        ticket: problem
        for ticket, problem in sorted(problems.items(), key=lambda item: int(item[0]))
        if problem not in (NO_TICKET_FILE, NO_MILESTONE)
    }
    if conflicts:
        print(
            f"CHANGELOG {label} cites tickets filed for another milestone: "
            + ", ".join(f"{ticket} ({problem})" for ticket, problem in conflicts.items()),
            file=sys.stderr,
        )
        return 1
    print(
        f"changelog {label} covers all {len(tickets)} tickets merged since {previous}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
