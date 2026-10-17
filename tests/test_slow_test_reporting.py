"""Runner regressions: timing warns without replacing the assertion result."""

from __future__ import annotations

from pathlib import Path
import os
import subprocess
import sys

import pytest

ROOT = Path(__file__).resolve().parents[1]


@pytest.mark.parametrize("load_available", [False, True])
@pytest.mark.parametrize("workers", [0, 2])
@pytest.mark.parametrize("fails", [False, True])
def test_python_slow_warning_preserves_test_result(
    tmp_path: Path, workers: int, fails: bool, load_available: bool
) -> None:
    conftest = (ROOT / "tests/conftest.py").read_text()
    if not load_available:
        conftest += (
            "\ndef pytest_sessionstart(session):\n"
            "    def unavailable():\n"
            '        raise OSError("load unavailable")\n'
            "    os.getloadavg = unavailable\n"
        )
    (tmp_path / "conftest.py").write_text(conftest)
    (tmp_path / "pytest.ini").write_text("[pytest]\nslow_test_warning_seconds = 0.01\n")
    (tmp_path / "test_case.py").write_text(
        "import threading\n"
        "def test_slow_case():\n"
        "    threading.Event().wait(0.03)\n"
        f"    assert {not fails}\n"
    )
    result = subprocess.run(
        [sys.executable, "-m", "pytest", "-n", str(workers), "-q", str(tmp_path)],
        cwd=tmp_path,
        env={**os.environ, "PYTEST_ADDOPTS": ""},
        capture_output=True,
        text=True,
        timeout=30,
    )
    assert result.returncode == int(fails), result.stdout + result.stderr
    assert "SLOW test_case.py::test_slow_case" in result.stdout
    assert "load (1, 5, 15 min):" in result.stdout
    if not load_available:
        assert "load (1, 5, 15 min): unavailable" in result.stdout


@pytest.mark.parametrize("exit_code", [0, 1])
def test_nextest_load_reporting_preserves_runner_exit(exit_code: int) -> None:
    result = subprocess.run(
        [
            sys.executable,
            str(ROOT / "tools/report-slow-tests.py"),
            sys.executable,
            "-c",
            "import sys; print('        SLOW [> 30.000s] probe test_slow_case', "
            f"file=sys.stderr); sys.exit({exit_code})",
        ],
        capture_output=True,
        text=True,
        timeout=10,
    )
    assert result.returncode == exit_code
    assert "probe test_slow_case" in result.stdout
    assert "load (1, 5, 15 min):" in result.stdout


def test_nextest_reporter_preserves_child_signal() -> None:
    result = subprocess.run(
        [
            sys.executable,
            str(ROOT / "tools/report-slow-tests.py"),
            sys.executable,
            "-c",
            "import os, signal; os.kill(os.getpid(), signal.SIGTERM)",
        ],
        capture_output=True,
        text=True,
        timeout=10,
    )
    assert result.returncode == 143, result.stdout + result.stderr


def test_nextest_reporter_forwards_termination_and_reaps_runner() -> None:
    import signal

    child_pid = None
    with subprocess.Popen(
        [
            sys.executable,
            str(ROOT / "tools/report-slow-tests.py"),
            sys.executable,
            "-c",
            "import os, time; print(os.getpid(), flush=True); time.sleep(20)",
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    ) as process:
        try:
            assert process.stdout is not None
            child_pid = int(process.stdout.readline())
            process.send_signal(signal.SIGTERM)
            stdout, stderr = process.communicate(timeout=3)
            assert process.returncode == 143, stdout + stderr
            with pytest.raises(ProcessLookupError):
                os.kill(child_pid, 0)
        finally:
            if child_pid is not None:
                try:
                    os.kill(child_pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
