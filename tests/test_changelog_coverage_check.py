from __future__ import annotations

import os
from pathlib import Path
import shlex
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
CHECK = ROOT / "scripts" / "check-changelog-coverage.py"


def _fake_git(directory: Path, responses: list[list[str]]) -> None:
    script = directory / "git"
    body = "#!/usr/bin/env bash\nset -euo pipefail\n"
    body += 'printf \'%s\\n\' "$*" >> "$GIT_FAKE_CALLS"\n'
    body += 'COUNT="$(wc -l < "$GIT_FAKE_CALLS")"\n'
    for index, lines in enumerate(responses, 1):
        body += f"if [ \"$COUNT\" -eq {index} ]; then printf '%s\\n' {shlex.join(lines)}; exit 0; fi\n"
    body += 'echo "unexpected git call" >&2\nexit 1\n'
    script.write_text(body, encoding="utf-8")
    script.chmod(0o755)


def _run(
    tmp_path: Path,
    *,
    changelog: str,
    subjects: list[str],
    tags: list[str] | None = None,
) -> subprocess.CompletedProcess[str]:
    _fake_git(tmp_path, [tags or ["v0.9.0", "v0.9.1"], subjects])
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
    _fake_git(tmp_path, [["v0.9.1"]])
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
