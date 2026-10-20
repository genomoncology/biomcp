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

SHARED_PYTHON_RUNNER = """
import json, sys, urllib.request
from pathlib import Path
sys.path.insert(0, str(Path.cwd() / 'tests/golden'))
from replay_env import replay_environment, replay_session
with replay_session():
    with replay_environment() as first:
        first_pid, first_origin = first.process.pid, first.origin
        first_dir = first._next_run_dir()
        assert first.request_log_entries() == []
        with urllib.request.urlopen(first.origin + '/healthz', timeout=None) as response:
            assert response.status == 200
        assert first.request_log_line_count() == 1
    with replay_environment() as second:
        assert second.process.pid == first_pid
        assert second.origin == first_origin
        assert second._next_run_dir() != first_dir
        assert second.request_log_entries() == []
    with replay_environment() as replay:
        print(json.dumps({'origin': replay.origin, 'pid': replay.process.pid}), flush=True)
        action = sys.stdin.readline().strip()
        if action == 'timeout':
            replay.run(Path(sys.executable), ('-c', 'import sys; sys.stdin.read()'), timeout=0.1)
"""


def _start_runner(tmp_path: Path, launcher: str) -> subprocess.Popen:
    env = isolated_command_env(tmp_path, {}, "")
    env["TMPDIR"] = str(tmp_path)
    if launcher in ("python", "python-shared"):
        script = SHARED_PYTHON_RUNNER if launcher == "python-shared" else PYTHON_RUNNER
        argv = [sys.executable, "-c", script]
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
    for line in runner.stdout:
        if line.startswith("{"):
            return json.loads(line)
    pytest.fail("runner exited before reporting a ready replay server: " + runner.stderr.read())


def _assert_stopped(identity: dict) -> None:
    port = int(identity["origin"].rsplit(":", 1)[1])
    pause = threading.Event()
    while True:
        try:
            with socket.create_connection(("127.0.0.1", port), timeout=None):
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
        pause.wait(0.02)


def _cleanup(runner: subprocess.Popen) -> None:
    # Only this test's new process group is ours, including failed-test orphans.
    try:
        os.killpg(runner.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass
    runner.wait()
    for stream in (runner.stdin, runner.stdout, runner.stderr):
        stream.close()


@pytest.mark.parametrize("launcher", ["python", "python-shared", "bash"])
@pytest.mark.timeout(120)
def test_runner_death_stops_replay_during_an_incomplete_request(
    tmp_path: Path, launcher: str,
) -> None:
    runner = _start_runner(tmp_path, launcher)
    identity = None
    try:
        identity = _identity(runner)
        origin = identity["origin"]
        port = int(origin.rsplit(":", 1)[1])
        with socket.create_connection(("127.0.0.1", port), timeout=None) as pending:
            pending.sendall(
                b"POST /mygene/query HTTP/1.1\r\nHost: localhost\r\n"
                b"Content-Length: 100\r\n\r\nx"
            )
            # A second accepted connection proves the server remains responsive
            # while the preceding request waits for its missing body bytes.
            with urllib.request.urlopen(origin + "/healthz", timeout=None) as probe:
                assert probe.status == 200
            runner.kill()
            assert runner.wait() == -signal.SIGKILL
            _assert_stopped(identity)
    finally:
        _cleanup(runner)


@pytest.mark.parametrize("action", ["normal", "timeout"])
@pytest.mark.parametrize("launcher", ["python", "python-shared"])
@pytest.mark.timeout(120)
def test_replay_teardown_stops_server_on_normal_exit_and_command_timeout(
    tmp_path: Path, action: str, launcher: str,
) -> None:
    runner = _start_runner(tmp_path, launcher)
    identity = None
    try:
        identity = _identity(runner)
        runner.stdin.write(action + "\n")
        runner.stdin.flush()
        status = runner.wait()
        stderr = runner.stderr.read()
        if action == "normal":
            assert status == 0, stderr
        else:
            assert status != 0
            assert "TimeoutExpired" in stderr
        _assert_stopped(identity)
    finally:
        _cleanup(runner)


def test_shared_replay_restores_fixture_roots_and_isolates_request_logs(tmp_path: Path) -> None:
    from replay_env import replay_environment

    roots = []
    for name in ("first", "second"):
        root = tmp_path / name
        replay_dir = root / "testdata/replay"
        replay_dir.mkdir(parents=True)
        (replay_dir / "reply.txt").write_text(name)
        (replay_dir / "manifest.json").write_text(json.dumps({
            "schema_version": 1,
            "routes": {"GET /mygene/ownership-probe": {
                "status": 200, "file": "reply.txt",
                "headers": {"content-type": "text/plain"},
            }},
        }))
        roots.append(root)
    identities = []
    views = []
    for root in roots:
        with replay_environment(root=root) as replay:
            views.append(replay)
            identities.append(replay.process.pid)
            assert replay.request_log_entries() == []
            with urllib.request.urlopen(replay.origin + "/mygene/ownership-probe", timeout=5) as response:
                assert response.read().decode() == root.name
            assert replay.request_log_line_count() == 1
            assert replay.misses() == []
    assert len(set(identities)) == 1
    assert [view.request_log_line_count() for view in views] == [1, 1]
    assert [len(view.request_log_entries()) for view in views] == [1, 1]
    with replay_environment() as replay:
        assert replay.process.pid == identities[0]
        assert replay.request_log_entries() == []
        with pytest.raises(urllib.error.HTTPError) as error:
            urllib.request.urlopen(replay.origin + "/mygene/ownership-probe", timeout=5)
        assert error.value.code == 404
        assert len(replay.misses()) == 1


def test_shared_replay_records_only_explicit_contexts_against_local_upstream(
    tmp_path: Path,
) -> None:
    from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

    calls = []

    class Upstream(BaseHTTPRequestHandler):
        def do_GET(self):
            calls.append(self.path)
            body = b'{"local": true}'
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def log_message(self, *_args):
            pass

    upstream = ThreadingHTTPServer(("127.0.0.1", 0), Upstream)
    thread = threading.Thread(target=upstream.serve_forever, daemon=True)
    thread.start()
    root = tmp_path / "fixture"
    (root / "src").mkdir(parents=True)
    (root / "src/probe.rs").write_text(
        f'const PROBE_BASE: &str = "http://127.0.0.1:{upstream.server_port}";\n'
        'const PROBE_BASE_ENV: &str = "BIOMCP_PROBE_BASE";\n'
    )
    (root / "testdata/replay").mkdir(parents=True)
    (root / "testdata/replay/manifest.json").write_text(
        json.dumps({"schema_version": 1, "routes": {}})
    )
    script = """
import json, sys, urllib.error, urllib.request
from pathlib import Path
sys.path.insert(0, str(Path.cwd() / 'tests/golden'))
import replay_env
replay_env.REPO_ROOT = Path(sys.argv[1])
results = []
with replay_env.replay_session():
    for record, query in [(False, 'one'), (True, 'one'), (False, 'two')]:
        with replay_env.replay_environment(record=record) as replay:
            try:
                response = urllib.request.urlopen(replay.origin + '/probe/value?' + query, timeout=5)
            except urllib.error.HTTPError as error:
                response = error
            with response:
                response.read()
                status = response.status
            results.append({'pid': replay.process.pid, 'status': status,
                            'logs': replay.request_log_entries(), 'record': replay.record})
print(json.dumps(results))
"""
    try:
        result = subprocess.run(
            [sys.executable, "-c", script, str(root)], cwd=ROOT,
            env=isolated_command_env(tmp_path / "command", {}, ""),
            capture_output=True, text=True, timeout=20,
        )
        assert result.returncode == 0, result.stderr
        results = json.loads(result.stdout)
        assert len({row["pid"] for row in results}) == 1
        assert [row["status"] for row in results] == [404, 200, 404]
        assert [row["record"] for row in results] == [False, True, False]
        assert [len(row["logs"]) for row in results] == [1, 1, 1]
        assert [row["logs"][0]["result"] for row in results] == ["miss", "recorded", "miss"]
        assert calls == ["/value?one"]
    finally:
        upstream.shutdown()
        upstream.server_close()
        thread.join(timeout=5)


def test_stored_root_object_routes_preserve_their_source_identity(tmp_path: Path) -> None:
    from replay_env import replay_environment

    root = tmp_path / "fixture"
    root.mkdir()
    (root / "src").symlink_to(ROOT / "src", target_is_directory=True)
    directory = root / "testdata/replay"
    directory.mkdir(parents=True)
    (directory / "reply.json").write_bytes(b'{"recorded": true}')
    path = "/PMC3467229.1/object.json"
    base = "https://pmc-oa-opendata.s3.amazonaws.com"
    cases = [
        ("valid", "pmc-oa", base + path + "?case=valid", 200),
        ("provider", "unknown", base + path + "?case=provider", 404),
        ("origin", "pmc-oa", "https://example.invalid" + path + "?case=origin", 404),
        ("path", "pmc-oa", base + "/different.json?case=path", 404),
        ("query", "pmc-oa", base + path + "?case=different", 404),
    ]
    routes = {
        f"GET {path}?case={case}": {
            "upstream": upstream,
            "status": 200,
            "file": "reply.json",
            "headers": {"content-type": "application/json"},
            "recording_provenance": {"provider": provider},
        }
        for case, provider, upstream, _ in cases
    }
    (directory / "manifest.json").write_text(
        json.dumps({"schema_version": 1, "routes": routes})
    )
    with replay_environment(root=root) as replay:
        for case, _, _, expected in [*cases, ("absent", "", "", 404)]:
            try:
                response = urllib.request.urlopen(
                    replay.origin + path + "?case=" + case, timeout=5
                )
            except urllib.error.HTTPError as error:
                response = error
            with response:
                body = response.read()
                assert response.status == expected, (case, body)
                if expected == 200:
                    assert body == b'{"recorded": true}'
