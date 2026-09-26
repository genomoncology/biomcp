"""Landed tickets must record their review verdicts, and no tracked
file may carry merge-conflict markers.

The first rule catches the class Ian's 2026-09-26 review found: six
landed tickets still said "review: pending" after their work merged.
A ticket counts as landed once `sdlc/records/` holds a record with the
same ticket number, so open tickets without records keep their honest
pending lines. A pending line inside a landed ticket fails unless its
parenthetical scope names the part still pending (a batch, an item),
because landed work must say what happened to it.

The second rule catches what merge 7206240b shipped: conflict markers
committed to main. Scanning happens against `git ls-files`, so the
check fails the suite before a bisect ever lands on the broken commit.
"""

from __future__ import annotations

import re
import subprocess
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]

# A review-status bullet in a ticket's Review section. The optional
# parenthetical scope ("(batch 1)", "(batches 2-3)") names the slice.
REVIEW_LINE = re.compile(
    r"^\s*-\s*(?P<scope>\((?P<scope_text>[^)]*)\))?\s*"
    r"(?P<kind>design\s+re-review|code\s+re-review|design\s+review|code\s+review|re-review|review)"
    r"\s*:\s*(?P<verdict>.*?)\s*$",
    re.IGNORECASE | re.MULTILINE,
)
PENDING = re.compile(r"pending", re.IGNORECASE)
SCOPED = re.compile(r"batch|item|remaining|later|future", re.IGNORECASE)

CONFLICT_MARKER = re.compile(r"^(<{7} |> {7}\|?={7}$|>{7} )", re.MULTILINE)
MARKER_START = re.compile(r"^<{7} ")
MARKER_END = re.compile(r"^>{7} ")
MARKER_SEP = re.compile(r"^={7}$")


def _ticket_number(path: Path) -> str:
    return path.name.split("-", 1)[0]


def _landed_ticket_paths() -> list[Path]:
    records_dir = REPO_ROOT / "sdlc" / "records"
    landed_numbers = {
        _ticket_number(record) for record in records_dir.glob("[0-9]" * 4 + "-*.md")
    }
    tickets = sorted((REPO_ROOT / "sdlc" / "tickets").glob("[0-9]" * 4 + "-*.md"))
    return [t for t in tickets if _ticket_number(t) in landed_numbers]


def _pending_review_lines(ticket: Path) -> list[tuple[int, str]]:
    offenses: list[tuple[int, str]] = []
    text = ticket.read_text(encoding="utf-8")
    for match in REVIEW_LINE.finditer(text):
        if not PENDING.search(match.group("verdict")):
            continue
        scope_text = match.group("scope_text")
        if scope_text and SCOPED.search(scope_text):
            # A pending slice of an otherwise landed ticket: the scope
            # names the part (batch, item) that has not landed yet.
            continue
        line = text.count("\n", 0, match.start()) + 1
        offenses.append((line, match.group(0).strip()))
    return offenses


def test_landed_tickets_record_their_review_verdicts() -> None:
    landed = _landed_ticket_paths()
    assert landed, "no landed tickets found; the scan itself is broken"

    failures: list[str] = []
    for ticket in landed:
        for line, text in _pending_review_lines(ticket):
            failures.append(f"{ticket.relative_to(REPO_ROOT)}:{line}: {text}")

    assert not failures, (
        "landed tickets still say a review is pending "
        "(a scoped '(batches ...)' line is allowed while those batches are open):\n"
        + "\n".join(failures)
    )


def test_pending_review_lines_catch_the_known_shapes() -> None:
    """The matcher itself must catch every stale shape the review found."""
    offenders = {
        "- Code review: pending": True,
        "- Design review: pending": True,
        "- code review: pending": True,
        "- re-review: pending": True,
        "- Design review: REJECT once, re-review pending": True,
        "- Code review (batch 1): ACCEPT": False,
        "- Code review (batches 2-3): pending": False,
        "- Code review: ACCEPT 2026-09-25": False,
        "- Verification: pending the yellow gate": False,
    }
    import tempfile

    for line, should_fail in offenders.items():
        body = f"## Review\n\n{line}\n"
        with tempfile.TemporaryDirectory() as tmp:
            probe = Path(tmp) / "0000-probe.md"
            probe.write_text(body, encoding="utf-8")
            found = _pending_review_lines(probe)
        assert bool(found) == should_fail, f"{line!r}: expected fail={should_fail}"


def _tracked_files() -> list[str]:
    output = subprocess.run(
        ["git", "ls-files", "-z"],
        cwd=REPO_ROOT,
        check=True,
        capture_output=True,
    ).stdout
    return [name.decode("utf-8") for name in output.split(b"\0") if name]


def _conflict_marker_lines(path: Path) -> list[tuple[int, str]]:
    text = path.read_text(encoding="utf-8", errors="replace")
    offenses: list[tuple[int, str]] = []
    for number, line in enumerate(text.splitlines(), start=1):
        if MARKER_START.match(line) or MARKER_END.match(line) or MARKER_SEP.match(line):
            offenses.append((number, line))
    return offenses


def test_no_tracked_file_carries_merge_conflict_markers() -> None:
    failures: list[str] = []
    for name in _tracked_files():
        path = REPO_ROOT / name
        if not path.is_file():
            continue
        for number, line in _conflict_marker_lines(path):
            failures.append(f"{name}:{number}: {line[:60]}")

    assert not failures, (
        "tracked files carry merge-conflict markers "
        "(merge 7206240b shipped these once; bisect lands on a broken commit):\n"
        + "\n".join(failures)
    )


def test_conflict_marker_scan_fails_on_a_synthetic_marker_file(
    tmp_path: Path,
) -> None:
    marked = tmp_path / "marked.md"
    marked.write_text(
        "before\n<<<<<<< HEAD\nours\n=======\ntheirs\n>>>>>>> branch\nafter\n",
        encoding="utf-8",
    )
    offenses = _conflict_marker_lines(marked)
    assert [number for number, _ in offenses] == [2, 4, 6]


def test_conflict_marker_scan_passes_clean_text(tmp_path: Path) -> None:
    clean = tmp_path / "clean.md"
    clean.write_text(
        "# Title\n\nA list:\n\n- item\n\nA table row: a || b\n\n"
        "Python code fences with <<< repeated: <<<<<<\n",
        encoding="utf-8",
    )
    assert _conflict_marker_lines(clean) == []


def test_review_scan_requires_a_records_directory(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """The landed set comes from sdlc/records; without it nothing fails."""
    (tmp_path / "tickets").mkdir()
    (tmp_path / "tickets" / "0001-open.md").write_text(
        "## Review\n\n- Code review: pending\n", encoding="utf-8"
    )
    assert _pending_review_lines(tmp_path / "tickets" / "0001-open.md")
