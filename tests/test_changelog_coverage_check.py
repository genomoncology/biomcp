from __future__ import annotations

import importlib.util
import os

import pytest
from pathlib import Path
import re
import shlex
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
CHECK = ROOT / "scripts" / "check-changelog-coverage.py"

_SPEC = importlib.util.spec_from_file_location("check_changelog_coverage", CHECK)
_MODULE = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(_MODULE)
described_tickets = _MODULE.described_tickets
record_tickets = _MODULE.record_tickets


def _fake_git(directory: Path, responses: dict[str, list[str]]) -> None:
    """Answer git subprocess calls by leading subcommand, not by order.

    The coverage check runs `git tag`, `git log`, and `git diff` in
    whatever order it needs them, so dispatching on the first argument
    keeps the harness honest when the script changes.
    """
    script = directory / "git"
    body = "#!/usr/bin/env bash\nset -euo pipefail\n"
    body += 'printf \'%s\\n\' "$*" >> "$GIT_FAKE_CALLS"\n'
    body += 'case "$1" in\n'
    for subcommand, lines in responses.items():
        body += f"  {subcommand}) printf '%s\\n' {shlex.join(lines)}; exit 0 ;;\n"
    body += 'esac\necho "unexpected git call: $*" >&2\nexit 1\n'
    script.write_text(body, encoding="utf-8")
    script.chmod(0o755)


def _run(
    tmp_path: Path,
    *,
    changelog: str,
    subjects: list[str],
    records: list[str] | None = None,
) -> subprocess.CompletedProcess[str]:
    _fake_git(
        tmp_path,
        {
            "tag": ["v0.9.0", "v0.9.1"],
            "log": subjects,
            "diff": records or [],
        },
    )
    path = tmp_path / "CHANGELOG.md"
    path.write_text(changelog, encoding="utf-8")
    return subprocess.run(
        [sys.executable, str(CHECK), "--tag", "v0.9.1", "--changelog", str(path)],
        capture_output=True,
        text=True,
        env=os.environ
        | {
            "PATH": f"{tmp_path}:{os.environ.get('PATH', '')}",
            "GIT_FAKE_CALLS": str(tmp_path / "calls"),
        },
    )


def test_version_section_at_end_of_file_passes(tmp_path: Path) -> None:
    result = _run(
        tmp_path,
        subjects=["Merge remote-tracking branch 'origin/tickets/2000-fix'"],
        changelog="# C\n\n## 0.9.1 — 2026-09-23\n\n- Fixed release publication. (2000)\n",
    )
    assert result.returncode == 0, result.stderr


def test_unreleased_fallback_passes(tmp_path: Path) -> None:
    result = _run(
        tmp_path,
        subjects=["Merge branch 'tickets/1234-gate'"],
        changelog="# C\n\n## Unreleased\n\n- Reworked release gates. (1234)\n\n## 0.9.0\n",
    )
    assert result.returncode == 0, result.stderr


def test_only_merge_ticket_subjects_count(tmp_path: Path) -> None:
    result = _run(
        tmp_path,
        subjects=["Mention 1086 in a test", "Merge branch 'tickets/1234-gate'"],
        changelog="# C\n\n## Unreleased\n\n- Reworked release gates. (1234)\n",
    )
    assert result.returncode == 0, result.stderr


def test_bare_ticket_number_does_not_satisfy_coverage(tmp_path: Path) -> None:
    result = _run(
        tmp_path,
        subjects=["Merge branch 'tickets/1234-gate'"],
        changelog="# C\n\n## Unreleased\n\n- 1234\n",
    )
    assert result.returncode == 1
    assert "1234" in result.stderr


def test_bare_ticket_number_list_does_not_satisfy_coverage(tmp_path: Path) -> None:
    result = _run(
        tmp_path,
        subjects=[
            "Merge branch 'tickets/1226-gate'",
            "Merge branch 'tickets/1227-other'",
        ],
        changelog="# C\n\n## Unreleased\n\n- 1226, 1227, 1228\n",
    )
    assert result.returncode == 1
    assert "1226" in result.stderr and "1227" in result.stderr


def test_label_word_bullets_do_not_satisfy_coverage(tmp_path: Path) -> None:
    # "Tickets" and "see" are boilerplate labels, not descriptions:
    # the gate must not count them as covered.
    result = _run(
        tmp_path,
        subjects=[
            "Merge branch 'tickets/1226-gate'",
            "Merge branch 'tickets/1227-other'",
        ],
        changelog="# C\n\n## Unreleased\n\n- Tickets 1226, 1227\n- see 1226\n",
    )
    assert result.returncode == 1
    assert "1226" in result.stderr and "1227" in result.stderr


def test_described_number_only_bullet_after_numbers_removed(tmp_path: Path) -> None:
    # Even with three stray digits gone, at least three word
    # characters of description must remain.
    result = _run(
        tmp_path,
        subjects=["Merge branch 'tickets/1234-gate'"],
        changelog="# C\n\n## Unreleased\n\n- a 1 b 2 c 3 (1234)\n",
    )
    assert result.returncode == 0, result.stderr


def test_record_only_ticket_requires_a_bullet(tmp_path: Path) -> None:
    result = _run(
        tmp_path,
        subjects=["Record ticket 2001 directly"],
        records=["sdlc/records/2001-fix-the-gate.md"],
        changelog="# C\n\n## Unreleased\n\n- Something else entirely. (1234)\n",
    )
    assert result.returncode == 1
    assert "2001" in result.stderr


def test_record_only_ticket_passes_with_a_described_bullet(tmp_path: Path) -> None:
    result = _run(
        tmp_path,
        subjects=["Record ticket 2001 directly"],
        records=["sdlc/records/2001-fix-the-gate.md"],
        changelog="# C\n\n## Unreleased\n\n- Fixed the coverage gate. (2001)\n",
    )
    assert result.returncode == 0, result.stderr


def test_union_of_merge_subjects_and_records_requires_both_bullets(
    tmp_path: Path,
) -> None:
    result = _run(
        tmp_path,
        subjects=["Merge branch 'tickets/1234-gate'"],
        records=["sdlc/records/2002-record-only.md"],
        changelog="# C\n\n## Unreleased\n\n- Reworked the gates. (1234)\n",
    )
    assert result.returncode == 1
    assert "2002" in result.stderr


def test_union_passes_when_both_have_bullets(tmp_path: Path) -> None:
    result = _run(
        tmp_path,
        subjects=["Merge branch 'tickets/1234-gate'"],
        records=["sdlc/records/2002-record-only.md"],
        changelog="# C\n\n## Unreleased\n\n- Reworked the gates. (1234)\n- Widened the ticket scan. (2002)\n",
    )
    assert result.returncode == 0, result.stderr


def test_record_discovery_uses_real_git_history(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """The records scan runs against a real repository, not a canned response."""
    import subprocess

    def git(*args: str, cwd: Path) -> None:
        subprocess.run(["git", *args], cwd=cwd, check=True, capture_output=True)

    repo = tmp_path / "repo"
    (repo / "sdlc" / "records").mkdir(parents=True)
    (repo / "CHANGELOG.md").write_text("# Changelog\n", encoding="utf-8")
    git("init", "-q", cwd=repo)
    git("config", "user.email", "t@example.com", cwd=repo)
    git("config", "user.name", "t", cwd=repo)
    git("add", "-A", cwd=repo)
    git("commit", "-q", "-m", "base", cwd=repo)
    git("tag", "v0.9.0", cwd=repo)
    (repo / "sdlc" / "records" / "2001-real-history.md").write_text(
        "x\n", encoding="utf-8"
    )
    git("add", "-A", cwd=repo)
    git("commit", "-q", "-m", "ticket 2001", cwd=repo)
    git("tag", "v0.9.1", cwd=repo)

    monkeypatch.chdir(repo)
    tickets = record_tickets("v0.9.0", "v0.9.1")
    assert "2001" in tickets


def test_non_ticket_record_files_do_not_count(tmp_path: Path) -> None:
    # Only sdlc/records/NNNN-*.md names map to tickets; a note file
    # in the same directory must not.
    result = _run(
        tmp_path,
        subjects=["Merge branch 'tickets/1234-gate'"],
        records=["sdlc/records/release-notes-only.md"],
        changelog="# C\n\n## Unreleased\n\n- Reworked the gates. (1234)\n",
    )
    assert result.returncode == 0, result.stderr


def test_internal_only_described_bullet_passes(tmp_path: Path) -> None:
    result = _run(
        tmp_path,
        subjects=["Merge branch 'tickets/1234-gate'"],
        changelog="# C\n\n## Unreleased\n\n- Internal only: release contract tests. (1234)\n",
    )
    assert result.returncode == 0, result.stderr


def test_missing_both_sections_fails_clearly(tmp_path: Path) -> None:
    result = _run(
        tmp_path,
        subjects=["Merge branch 'tickets/1234-gate'"],
        changelog="# C\n\n## 0.9.0\n",
    )
    assert result.returncode == 1
    assert "neither a 0.9.1 nor an Unreleased section" in result.stderr


def test_no_previous_release_passes_without_git_log(tmp_path: Path) -> None:
    _fake_git(tmp_path, {"tag": ["v0.9.1"]})
    path = tmp_path / "CHANGELOG.md"
    path.write_text("## Unreleased\n", encoding="utf-8")
    result = subprocess.run(
        [sys.executable, str(CHECK), "--tag", "v0.9.1", "--changelog", str(path)],
        capture_output=True,
        text=True,
        env=os.environ
        | {
            "PATH": f"{tmp_path}:{os.environ.get('PATH', '')}",
            "GIT_FAKE_CALLS": str(tmp_path / "calls"),
        },
    )
    assert result.returncode == 0, result.stderr


def test_every_real_unreleased_bullet_describes_its_ticket() -> None:
    content = (ROOT / "CHANGELOG.md").read_text(encoding="utf-8")
    match = re.search(r"^## Unreleased\s*([\s\S]*?)(?=^## |\Z)", content, re.MULTILINE)
    assert match is not None
    section = match.group(1)
    described = described_tickets(section)
    mentioned = set(re.findall(r"\((\d{4})\)", section))
    assert mentioned, "the Unreleased section has no ticket markers"
    missing = sorted(mentioned - described)
    assert not missing, f"bullets for {missing} carry no described text"


def test_described_tickets_rejects_a_number_only_bullet_directly() -> None:
    assert described_tickets("- 1226, 1227, 1228") == set()
    assert described_tickets("- Fixed the wheel floor check (1246)") == {"1246"}


def test_described_tickets_rejects_label_only_bullets_directly() -> None:
    assert described_tickets("- Tickets 1226, 1227") == set()
    assert described_tickets("- see 1226") == set()
    assert described_tickets("- and fixes 1226") == set()
    assert described_tickets("- The changes 1226") == set()
    assert described_tickets("- Fixed 1226") == set()
    assert described_tickets("- Updated 1226") == set()
    assert described_tickets("- Added 1226") == set()
    # Real description words survive the stoplist.
    assert described_tickets("- Restored container publication (1219)") == {"1219"}
