from __future__ import annotations

import os
import shutil
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


def _run_real_nih_spec(
    tmp_path: Path, response: str, *, command_status: int = 0
) -> subprocess.CompletedProcess[str]:
    workspace = tmp_path / "workspace"
    for relative in (
        "scripts/run-specs.sh",
        "spec/entity/nih-reporter-live.md",
        "tools/biomcp-ci",
    ):
        destination = workspace / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(REPO_ROOT / relative, destination)

    binary = tmp_path / "biomcp"
    _write_executable(
        binary,
        "#!/usr/bin/env python3\n"
        "import sys\n"
        f"sys.stdout.write({response!r})\n"
        + (
            "print('provider command failed', file=sys.stderr)\n"
            if command_status
            else ""
        )
        + f"raise SystemExit({command_status})\n",
    )
    env = os.environ | {
        "BIOMCP_BIN": str(binary),
        "MUSTMATCH_BIN": str(Path.home() / ".local/bin/mustmatch"),
    }
    return subprocess.run(
        [
            str(HELPER),
            "nih-reporter",
            "--",
            "bash",
            str(workspace / "scripts/run-specs.sh"),
            "verify-nih-reporter",
        ],
        cwd=workspace,
        capture_output=True,
        text=True,
        check=False,
        env=env,
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
    sentinel_capture = tmp_path / "sentinel-path"
    _write_executable(
        command,
        "#!/usr/bin/env python3\n"
        "import os\n"
        "import pathlib\n"
        "import stat\n"
        f"capture = pathlib.Path({str(sentinel_capture)!r})\n"
        "sentinel = pathlib.Path(os.environ['BIOMCP_VERIFY_LIVE_PENDING_SENTINEL'])\n"
        "assert not sentinel.exists()\n"
        "assert stat.S_IMODE(sentinel.parent.stat().st_mode) == 0o700\n"
        "capture.write_text(str(sentinel), encoding='utf-8')\n"
        "sentinel.write_text('biomcp-nih-reporter-unavailable-v1\\n', encoding='utf-8')\n"
        "print('NIH Reporter funding data is temporarily unavailable.')\n"
        "raise SystemExit(1)\n",
    )

    result = _run("nih-reporter", "--", str(command))

    assert result.returncode == 0
    assert "operator-pending: nih-reporter" in result.stderr
    assert "temporarily unavailable" in result.stdout
    sentinel = Path(sentinel_capture.read_text(encoding="utf-8"))
    assert not sentinel.exists()
    assert not sentinel.parent.exists()


def test_verify_live_does_not_trust_simulated_nih_spec_diagnostics(
    tmp_path: Path,
) -> None:
    command = tmp_path / "nih-runner-unavailable.sh"
    _write_executable(
        command,
        "#!/usr/bin/env bash\n"
        "echo 'NIH Reporter funding data is temporarily unavailable.' >&2\n"
        "echo 'spec page failed: spec/entity/nih-reporter-live.md (exit 1)' >&2\n"
        "exit 1\n",
    )

    result = _run("nih-reporter", "--", str(command))

    assert result.returncode == 1
    assert "product-red: nih-reporter" in result.stderr
    assert "operator-pending" not in result.stderr


def test_real_nih_spec_runner_uses_validated_pending_state(tmp_path: Path) -> None:
    unavailable = _run_real_nih_spec(
        tmp_path / "unavailable",
        '{"funding_note":"NIH Reporter funding data is temporarily unavailable."}',
    )

    assert unavailable.returncode == 0, unavailable.stderr
    assert "operator-pending: nih-reporter" in unavailable.stderr

    negative_cases = (
        ("malformed", "{", 0),
        ("missing-fields", '{"funding":{"query":"Marfan syndrome"}}', 0),
        (
            "command-error",
            '{"funding_note":"NIH Reporter funding data is temporarily unavailable."}',
            7,
        ),
    )
    for name, response, status in negative_cases:
        result = _run_real_nih_spec(
            tmp_path / name,
            response,
            command_status=status,
        )

        assert result.returncode != 0, name
        assert "product-red: nih-reporter" in result.stderr, name
        assert "operator-pending" not in result.stderr, name


def test_verify_live_keeps_exact_nih_line_with_unknown_command_error_red(
    tmp_path: Path,
) -> None:
    command = tmp_path / "nih-command-error.sh"
    _write_executable(
        command,
        "#!/usr/bin/env bash\n"
        "echo 'NIH Reporter funding data is temporarily unavailable.' >&2\n"
        "echo 'command failed before funding validation' >&2\n"
        "exit 1\n",
    )

    result = _run("nih-reporter", "--", str(command))

    assert result.returncode == 1
    assert "product-red: nih-reporter" in result.stderr
    assert "operator-pending" not in result.stderr


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


def test_verify_live_keeps_similar_nih_reporter_messages_red(tmp_path: Path) -> None:
    for message in (
        "NIH Reporter funding response is temporarily unavailable.",
        "NIH Reporter funding data is temporarily unavailable because its table is missing.",
        "NIH Reporter funding result is missing its table.",
    ):
        command = tmp_path / "nih-similar.sh"
        _write_executable(command, f"#!/usr/bin/env bash\necho {message!r} >&2\nexit 1\n")

        result = _run("nih-reporter", "--", str(command))

        assert result.returncode == 1, message
        assert "product-red: nih-reporter" in result.stderr
        assert "operator-pending" not in result.stderr
