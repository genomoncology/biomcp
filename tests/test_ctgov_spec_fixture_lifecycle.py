from __future__ import annotations

import os
from pathlib import Path
import shlex
import shutil
import signal
import socket
import subprocess
import time

import pytest


REPO_ROOT = Path(__file__).resolve().parents[1]


def _wait_until(predicate, timeout: float = 10.0) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if predicate():
            return
        time.sleep(0.05)
    assert predicate()


def _read_exports(path: Path) -> dict[str, str]:
    exports: dict[str, str] = {}
    for line in path.read_text().splitlines():
        key, value = line.removeprefix("export ").split("=", 1)
        exports[key] = shlex.split(value)[0]
    return exports


@pytest.mark.parametrize("termination_signal", [signal.SIGINT, signal.SIGTERM, signal.SIGHUP])
def test_runner_termination_cleans_ctgov_process_group_env_and_port(
    tmp_path: Path, termination_signal: signal.Signals
) -> None:
    workspace = tmp_path / "workspace"
    (workspace / "scripts").mkdir(parents=True)
    fixtures = workspace / "spec" / "fixtures"
    fixtures.mkdir(parents=True)
    shutil.copy2(REPO_ROOT / "scripts" / "run-specs.sh", workspace / "scripts")
    for name in (
        "setup-ctgov-intervention-alias-spec-fixture.sh",
        "cleanup-ctgov-intervention-alias-spec-fixture.sh",
    ):
        shutil.copy2(REPO_ROOT / "spec" / "fixtures" / name, fixtures / name)

    for name in ("setup-study-spec-fixture.sh",):
        script = fixtures / name
        script.write_text("#!/usr/bin/env bash\nexit 0\n")
        script.chmod(0o755)

    bin_dir = workspace / "bin"
    bin_dir.mkdir()
    biomcp = bin_dir / "biomcp"
    biomcp.write_text("#!/usr/bin/env bash\nexit 0\n")
    biomcp.chmod(0o755)
    mustmatch = bin_dir / "mustmatch"
    mustmatch.write_text(
        '#!/usr/bin/env bash\nif [ "${1:-}" = --version ]; then echo "mustmatch 1.0.0"; exit 0; fi\nexit 0\n'
    )
    mustmatch.chmod(0o755)

    ready = workspace / "runner-ready"
    env = os.environ | {
        "BIOMCP_BIN": str(biomcp),
        "MUSTMATCH_BIN": str(mustmatch),
        "BIOMCP_SPEC_RUNNER_READY_FILE": str(ready),
        "BIOMCP_SPEC_RUNNER_HOLD": "1",
    }
    runner = subprocess.Popen(
        ["bash", "scripts/run-specs.sh", "spec-contracts"],
        cwd=workspace,
        env=env,
    )
    fixture_env = workspace / ".cache" / "spec-ctgov-intervention-alias-env"
    try:
        _wait_until(lambda: ready.exists() and fixture_env.exists())
        exports = _read_exports(fixture_env)
        supervisor_pid = int(exports["BIOMCP_CTGOV_INTERVENTION_ALIAS_PID"])
        server_pid = int(exports["BIOMCP_CTGOV_INTERVENTION_ALIAS_SERVER_PID"])
        host, port_text = exports["BIOMCP_CTGOV_BASE"].removeprefix("http://").split("/", 1)[0].split(":")
        port = int(port_text)

        os.kill(runner.pid, termination_signal)
        assert runner.wait(timeout=10) == 128 + termination_signal

        _wait_until(lambda: not Path(f"/proc/{supervisor_pid}").exists())
        _wait_until(lambda: not Path(f"/proc/{server_pid}").exists())
        assert not fixture_env.exists()
        with socket.socket() as client:
            client.settimeout(0.5)
            assert client.connect_ex((host, port)) != 0
    finally:
        if runner.poll() is None:
            runner.kill()
            runner.wait()
        subprocess.run(
            ["bash", str(fixtures / "cleanup-ctgov-intervention-alias-spec-fixture.sh"), str(workspace)],
            check=False,
        )
