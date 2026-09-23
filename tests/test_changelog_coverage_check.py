from __future__ import annotations

import os
from pathlib import Path
import shlex
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[1]
GATE = ROOT / "scripts" / "check-changelog-coverage.py"
REPOSITORY = "genomoncology/biomcp"


def _fake_gh(directory: Path, *, responses: list[list[str]]) -> None:
    """Serve queued stdout lines in call order, one gh api invocation per response."""
    script = directory / "gh"
    body = "#!/usr/bin/env bash\nset -euo pipefail\n"
    body += 'printf \'%s\\n\' "$*" >> "$GH_FAKE_CALLS"\n'
    body += 'COUNT="$(( $(wc -l < "$GH_FAKE_CALLS") ))"\n'
    for index, lines in enumerate(responses, start=1):
        body += (
            f'if [ "$COUNT" -eq {index} ]; then '
            f"printf '%s\\n' {shlex.join(lines)}; exit 0; fi\n"
        )
    body += 'echo "unexpected gh call" >&2\nexit 1\n'
    script.write_text(body, encoding="utf-8")
    script.chmod(0o755)


def _run_gate(
    directory: Path,
    *,
    responses: list[list[str]],
    changelog: str,
) -> subprocess.CompletedProcess[str]:
    _fake_gh(directory, responses=responses)
    changelog_path = directory / "CHANGELOG.md"
    changelog_path.write_text(changelog, encoding="utf-8")
    return subprocess.run(
        [
            sys.executable,
            str(GATE),
            "--repository",
            REPOSITORY,
            "--tag",
            "v0.9.1",
            "--changelog",
            str(changelog_path),
        ],
        capture_output=True,
        text=True,
        env=os.environ
        | {
            "PATH": f"{directory}:{os.environ.get('PATH', '')}",
            "GH_FAKE_CALLS": str(directory / "gh-calls"),
        },
    )


COVERED = """# Changelog

## Unreleased

### Fixes

- Fixed the wheel binary. (1230)
- Let the fallback degrade. (1231)

## 0.9.0 — 2026-09-16
"""

MISSING = """# Changelog

## Unreleased

### Fixes

- Something else. (1200)

## 0.9.0 — 2026-09-16
"""


def test_gate_passes_when_unreleased_names_every_merged_ticket(tmp_path: Path) -> None:
    result = _run_gate(
        tmp_path,
        responses=[
            ["v0.9.1", "v0.9.0", "v0.8.25"],
            [
                "Restore panic unwinding (1230)",
                "Merge branch 'tickets/1231-ca-fallback-warn' (1231)",
            ],
        ],
        changelog=COVERED,
    )
    assert result.returncode == 0, result.stderr
    assert "covers all 2 tickets" in result.stdout


def test_gate_fails_when_a_merged_ticket_is_missing_from_unreleased(tmp_path: Path) -> None:
    result = _run_gate(
        tmp_path,
        responses=[
            ["v0.9.1", "v0.9.0", "v0.8.25"],
            ["Restore panic unwinding (1230)", "Let the fallback degrade (1231)"],
        ],
        changelog=MISSING,
    )
    assert result.returncode == 1
    assert "missing tickets merged since v0.9.0: 1230, 1231" in result.stderr


def test_gate_passes_when_there_is_no_previous_release(tmp_path: Path) -> None:
    result = _run_gate(
        tmp_path,
        responses=[["v0.9.1"]],
        changelog=COVERED,
    )
    assert result.returncode == 0, result.stderr
    assert "no previous release" in result.stdout


def test_gate_ignores_releases_newer_than_the_tag(tmp_path: Path) -> None:
    result = _run_gate(
        tmp_path,
        responses=[
            ["v0.10.0", "v0.9.2", "v0.9.1", "v0.9.0"],
            ["Restore panic unwinding (1230)", "Let the fallback degrade (1231)"],
        ],
        changelog=COVERED,
    )
    assert result.returncode == 0, result.stderr


def test_gate_fails_closed_when_gh_fails(tmp_path: Path) -> None:
    result = _run_gate(
        tmp_path,
        responses=[],
        changelog=COVERED,
    )
    assert result.returncode == 1
    assert "changelog coverage check failed" in result.stderr


def test_gate_fails_when_unreleased_section_is_absent(tmp_path: Path) -> None:
    result = _run_gate(
        tmp_path,
        responses=[
            ["v0.9.1", "v0.9.0"],
            ["Restore panic unwinding (1230)"],
        ],
        changelog="# Changelog\n\n## 0.9.0 — 2026-09-16\n",
    )
    assert result.returncode == 1
    assert "no Unreleased section" in result.stderr
