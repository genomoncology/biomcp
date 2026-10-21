"""Runner regressions: timing warns without replacing the assertion result."""

from __future__ import annotations

from hang_guard import HANG_TIMEOUT

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
    plugins = ["-p", "conftest"]
    if not load_available:
        plugins += ["-p", "unavailable_load"]
        (tmp_path / "unavailable_load.py").write_text(
            "import os\n\ndef pytest_sessionstart(session):\n"
            "    def unavailable():\n"
            '        raise OSError("load unavailable")\n'
            "    os.getloadavg = unavailable\n"
        )
    (tmp_path / "pytest.ini").write_text("[pytest]\nslow_test_warning_seconds = 0.01\n")
    (tmp_path / "test_case.py").write_text(
        "import threading\n"
        "def test_slow_case():\n"
        "    threading.Event().wait(0.03)\n"
        f"    assert {not fails}\n"
    )
    # Load the real harness in place so all support dependencies stay together.
    pythonpath = os.pathsep.join(
        filter(None, [str(ROOT / "tests"), os.environ.get("PYTHONPATH")])
    )
    result = subprocess.run(
        [
            sys.executable, "-m", "pytest", *plugins,
            "-n", str(workers), "-q", str(tmp_path),
        ],
        cwd=tmp_path,
        env={**os.environ, "PYTEST_ADDOPTS": "", "PYTHONPATH": pythonpath},
        capture_output=True,
        text=True,
        timeout=HANG_TIMEOUT,
    )
    assert result.returncode == int(fails), result.stdout + result.stderr
    assert "WARNING slow test test_case.py::test_slow_case" in result.stdout
    assert "load (1 min):" in result.stdout
    if not load_available:
        assert "load (1 min): unavailable" in result.stdout


@pytest.mark.parametrize("exit_code", [0, 1])
@pytest.mark.parametrize("messages", [
    "PASS [6.000s] probe test_slow_case",
    "SLOW [> 6.000s] probe test_slow_case\\nSLOW [> 10.000s] probe test_slow_case\\nPASS [11.000s] probe test_slow_case",
    "SLOW [> 5.000s] (───) probe test_slow_case\\nPASS [6.006s] (1/1) probe test_slow_case",
])
def test_nextest_load_reporting_preserves_runner_exit(exit_code: int, messages: str) -> None:
    result = subprocess.run(
        [
            sys.executable,
            str(ROOT / "tools/report-slow-tests.py"),
            sys.executable,
            "-c",
            f"import sys; print('{messages}', "
            f"file=sys.stderr); sys.exit({exit_code})",
        ],
        capture_output=True,
        text=True,
        timeout=HANG_TIMEOUT,
    )
    assert result.returncode == exit_code
    assert "probe test_slow_case" in result.stdout
    assert "load (1 min):" in result.stdout
    assert result.stdout.count("WARNING slow test") == 1


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
        timeout=HANG_TIMEOUT,
    )
    assert result.returncode == 143, result.stdout + result.stderr


@pytest.mark.timeout(HANG_TIMEOUT)
def test_nextest_reporter_forwards_termination_and_reaps_runner() -> None:
    import signal

    child_pid = None
    with subprocess.Popen(
        [
            sys.executable,
            str(ROOT / "tools/report-slow-tests.py"),
            sys.executable,
            "-c",
            "import os, signal\n"
            "def terminate(signum, frame):\n"
            "    print('SIGTERM forwarded', flush=True)\n"
            "    signal.signal(signal.SIGTERM, signal.SIG_DFL)\n"
            "    os.kill(os.getpid(), signal.SIGTERM)\n"
            "signal.signal(signal.SIGTERM, terminate)\n"
            "print(os.getpid(), flush=True)\n"
            "signal.pause()\n",
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        start_new_session=True,
    ) as process:
        try:
            assert process.stdout is not None
            child_pid = int(process.stdout.readline())
            process.send_signal(signal.SIGTERM)
            # Observe forwarding and exit; the test watchdog only bounds hangs.
            assert process.stdout.readline() == "SIGTERM forwarded\n"
            stdout, stderr = process.communicate()
            assert process.returncode == 143, stdout + stderr
            with pytest.raises(ProcessLookupError):
                os.kill(child_pid, 0)
            child_pid = None
        finally:
            # Include a runner whose PID readiness was never forwarded.
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            process.wait()
