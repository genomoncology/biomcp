"""Merge-conflict markers must never reach a tracked file.

A past merge (7206240b) shipped them once; the scan keeps bisect off a
broken commit. The review-verdict checks that once shared this file
moved to the ticket tool, which validates its own records.
"""

from __future__ import annotations

import re
import subprocess
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]

MARKER_START = re.compile(r"^<{7} ")
MARKER_END = re.compile(r"^>{7} ")
MARKER_SEP = re.compile(r"^={7}$")


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
