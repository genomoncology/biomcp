from __future__ import annotations

import subprocess
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
HELPER = REPO_ROOT / "tools" / "biomcp-verify-live"


def _write_executable(path: Path, contents: str) -> None:
    path.write_text(contents, encoding="utf-8")
    path.chmod(0o755)


def _run(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(HELPER), *args],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        check=False,
    )


def test_verify_live_passes_through_success(tmp_path: Path) -> None:
    command = tmp_path / "ok.sh"
    _write_executable(
        command,
        "#!/usr/bin/env bash\n"
        "echo live-ok\n"
        "echo live-note >&2\n",
    )

    result = _run("cpic", "--", str(command))

    assert result.returncode == 0
    assert "live-ok" in result.stdout
    assert "live-note" in result.stderr
    assert "operator-pending" not in result.stderr


def test_verify_live_classifies_cpic_web_anon_permission_denial(tmp_path: Path) -> None:
    command = tmp_path / "cpic-auth.sh"
    _write_executable(
        command,
        "#!/usr/bin/env bash\n"
        "echo 'HTTP 401: permission denied to set role \"web_anon\"' >&2\n"
        "exit 1\n",
    )

    result = _run("cpic", "--", str(command))

    assert result.returncode == 0
    assert "operator-pending: cpic" in result.stderr
    assert "web_anon" in result.stderr


def test_verify_live_keeps_unknown_cpic_failure_red(tmp_path: Path) -> None:
    command = tmp_path / "cpic-regression.sh"
    _write_executable(
        command,
        "#!/usr/bin/env bash\n"
        "echo 'CPIC returned malformed recommendations table' >&2\n"
        "exit 1\n",
    )

    result = _run("cpic", "--", str(command))

    assert result.returncode == 1
    assert "product-red: cpic" in result.stderr
    assert "operator-pending" not in result.stderr


def test_verify_live_classifies_nih_reporter_funding_unavailable(tmp_path: Path) -> None:
    command = tmp_path / "nih-unavailable.sh"
    _write_executable(
        command,
        "#!/usr/bin/env bash\n"
        "echo 'NIH Reporter funding data is temporarily unavailable.'\n"
        "exit 1\n",
    )

    result = _run("nih-reporter", "--", str(command))

    assert result.returncode == 0
    assert "operator-pending: nih-reporter" in result.stderr
    assert "temporarily unavailable" in result.stdout


def test_verify_live_keeps_unknown_nih_reporter_failure_red(tmp_path: Path) -> None:
    command = tmp_path / "nih-regression.sh"
    _write_executable(
        command,
        "#!/usr/bin/env bash\n"
        "echo 'NIH Reporter rendered malformed funding JSON' >&2\n"
        "exit 1\n",
    )

    result = _run("nih-reporter", "--", str(command))

    assert result.returncode == 1
    assert "product-red: nih-reporter" in result.stderr
    assert "operator-pending" not in result.stderr
