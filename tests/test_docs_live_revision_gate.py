from __future__ import annotations

import os
from pathlib import Path
import subprocess
import sys

import pytest


ROOT = Path(__file__).resolve().parents[1]
GATE = ROOT / "scripts" / "check-docs-live-revision.py"
REPOSITORY = "genomoncology/biomcp"
TAG_SHA = "a" * 40
LIVE_SHA = "b" * 40
COMPARE_CALL = f"api repos/{REPOSITORY}/compare/{TAG_SHA}...{LIVE_SHA} --jq .status"


def _fake_gh(directory: Path, *, status: str | None) -> None:
    script = directory / "gh"
    body = "#!/usr/bin/env bash\nset -euo pipefail\n"
    body += 'printf \'%s\\n\' "$*" >> "$GH_FAKE_CALLS"\n'
    if status is None:
        body += 'echo "HTTP 403: rate limited" >&2\nexit 1\n'
    else:
        body += f"printf '%s\\n' '{status}'\n"
    script.write_text(body, encoding="utf-8")
    script.chmod(0o755)


def _run_gate(directory: Path, *, status: str | None) -> subprocess.CompletedProcess[str]:
    _fake_gh(directory, status=status)
    return subprocess.run(
        [
            sys.executable,
            str(GATE),
            "--repository",
            REPOSITORY,
            "--tag-sha",
            TAG_SHA,
            "--live-revision",
            LIVE_SHA,
        ],
        capture_output=True,
        text=True,
        env=os.environ
        | {
            "PATH": f"{directory}:{os.environ.get('PATH', '')}",
            "GH_FAKE_CALLS": str(directory / "gh-calls"),
        },
    )


def _compare_calls(directory: Path) -> list[str]:
    return (directory / "gh-calls").read_text(encoding="utf-8").splitlines()


@pytest.mark.parametrize("status", ["identical", "ahead"])
def test_gate_passes_when_live_equals_or_descends_from_the_tag(
    tmp_path: Path, status: str
) -> None:
    completed = _run_gate(tmp_path, status=status)

    assert completed.returncode == 0, completed.stderr
    assert LIVE_SHA in completed.stdout
    assert _compare_calls(tmp_path) == [COMPARE_CALL]


@pytest.mark.parametrize("status", ["behind", "diverged"])
def test_gate_fails_when_live_is_behind_or_divergent(
    tmp_path: Path, status: str
) -> None:
    completed = _run_gate(tmp_path, status=status)

    assert completed.returncode == 1
    assert status in completed.stderr
    assert _compare_calls(tmp_path) == [COMPARE_CALL]


def test_gate_fails_closed_when_the_compare_call_fails(tmp_path: Path) -> None:
    completed = _run_gate(tmp_path, status=None)

    assert completed.returncode == 1
    assert "live documentation revision check failed" in completed.stderr
    assert _compare_calls(tmp_path) == [COMPARE_CALL]
