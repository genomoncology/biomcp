"""Replay servers belong to the test run, including an abandoned run."""

from __future__ import annotations

import json
import os
from pathlib import Path
import shutil
import signal
import socket
import subprocess
import sys
import threading
import time
import urllib.request

import pytest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tests/golden"))
from replay_env import isolated_command_env  # noqa: E402

PYTHON_RUNNER = """
import json, sys
from pathlib import Path
sys.path.insert(0, str(Path.cwd() / 'tests/golden'))
from replay_env import replay_environment
with replay_environment() as replay:
    print(json.dumps({'origin': replay.origin, 'pid': replay.process.pid}), flush=True)
    action = sys.stdin.readline().strip()
    if action == 'timeout':
        replay.run(Path(sys.executable), ('-c', 'import sys; sys.stdin.read()'), timeout=0.1)
"""


def _start_runner(tmp_path: Path, launcher: str) -> subprocess.Popen:
    env = isolated_command_env(tmp_path, {}, "")
    env["TMPDIR"] = str(tmp_path)
    if launcher == "python":
        argv = [sys.executable, "-c", PYTHON_RUNNER]
        cwd = ROOT
    else:
        cwd = tmp_path / "spec-runner"
        for directory in ("scripts", "tools", "bin", "testdata"):
            (cwd / directory).mkdir(parents=True, exist_ok=True)
        shutil.copy2(ROOT / "scripts/run-specs.sh", cwd / "scripts/run-specs.sh")
        fixture_dir = cwd / "spec/fixtures"
        fixture_dir.mkdir(parents=True)
        shutil.copy2(ROOT / "spec/fixtures/cache-policy.sh", fixture_dir / "cache-policy.sh")
        (cwd / "tools/replay-server").symlink_to(ROOT / "tools/replay-server")
        (cwd / "src").symlink_to(ROOT / "src", target_is_directory=True)
        (cwd / "testdata/replay").symlink_to(ROOT / "testdata/replay", target_is_directory=True)
        mustmatch = cwd / "bin/mustmatch"
        mustmatch.write_text(
            f"#!{sys.executable}\n"
            "import json, os, sys\n"
            "if '--version' in sys.argv:\n"
            "    print('mustmatch 1.0.0')\n"
            "else:\n"
            "    print(json.dumps({'origin': os.environ['BIOMCP_TEST_UNPACED_ORIGIN']}), flush=True)\n"
            "    sys.stdin.read()\n"
        )
        mustmatch.chmod(0o755)
        env["MUSTMATCH_BIN"] = str(mustmatch)
        env["BIOMCP_BIN"] = sys.executable
        argv = ["bash", "scripts/run-specs.sh", "spec"]
    return subprocess.Popen(
        argv, cwd=cwd, env=env, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
        stderr=subprocess.PIPE, text=True, start_new_session=True,
    )


def _identity(runner: subprocess.Popen) -> dict:
    # Bound readiness without leaving a hung runner behind on a startup error.
    result: list[str] = []

    def read() -> None:
        for line in runner.stdout:
            if line.startswith("{"):
                result.append(line)
                return

    reader = threading.Thread(target=read, daemon=True)
    reader.start()
    reader.join(timeout=10)
    assert result, (
        runner.stderr.read() if runner.poll() is not None
        else "runner did not report a ready replay server"
    )
    return json.loads(result[0])


def _assert_stopped(identity: dict) -> None:
    port = int(identity["origin"].rsplit(":", 1)[1])
    deadline = time.monotonic() + 5
    pause = threading.Event()
    while True:
        try:
            with socket.create_connection(("127.0.0.1", port), timeout=0.2):
                port_closed = False
        except ConnectionRefusedError:
            port_closed = True
        process_gone = True
        if "pid" in identity:
            try:
                os.kill(identity["pid"], 0)
                process_gone = False
            except ProcessLookupError:
                pass
        if port_closed and process_gone:
            return
        assert time.monotonic() < deadline, "replay server survived its test runner"
        pause.wait(0.02)


def _cleanup(runner: subprocess.Popen) -> None:
    # Only this test's new process group is ours, including failed-test orphans.
    try:
        os.killpg(runner.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass
    runner.wait(timeout=5)
    for stream in (runner.stdin, runner.stdout, runner.stderr):
        stream.close()


@pytest.mark.parametrize("launcher", ["python", "bash"])
def test_runner_death_stops_replay_during_an_incomplete_request(
    tmp_path: Path, launcher: str,
) -> None:
    runner = _start_runner(tmp_path, launcher)
    identity = None
    try:
        identity = _identity(runner)
        origin = identity["origin"]
        port = int(origin.rsplit(":", 1)[1])
        with socket.create_connection(("127.0.0.1", port), timeout=5) as pending:
            pending.sendall(
                b"POST /mygene/query HTTP/1.1\r\nHost: localhost\r\n"
                b"Content-Length: 100\r\n\r\nx"
            )
            # A second accepted connection proves the server remains responsive
            # while the preceding request waits for its missing body bytes.
            with urllib.request.urlopen(origin + "/healthz", timeout=5) as probe:
                assert probe.status == 200
            runner.kill()
            assert runner.wait(timeout=5) == -signal.SIGKILL
            _assert_stopped(identity)
    finally:
        _cleanup(runner)
        if identity is not None:
            _assert_stopped(identity)


@pytest.mark.parametrize("action", ["normal", "timeout"])
def test_replay_teardown_stops_server_on_normal_exit_and_command_timeout(
    tmp_path: Path, action: str,
) -> None:
    runner = _start_runner(tmp_path, "python")
    identity = None
    try:
        identity = _identity(runner)
        runner.stdin.write(action + "\n")
        runner.stdin.flush()
        status = runner.wait(timeout=10)
        stderr = runner.stderr.read()
        if action == "normal":
            assert status == 0, stderr
        else:
            assert status != 0
            assert "TimeoutExpired" in stderr
        _assert_stopped(identity)
    finally:
        _cleanup(runner)
        if identity is not None:
            _assert_stopped(identity)
