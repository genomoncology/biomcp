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


def _read_record(path: Path) -> dict[str, str]:
    return dict(line.split("=", 1) for line in path.read_text().splitlines())


@pytest.mark.parametrize(
    "termination_signal", [signal.SIGINT, signal.SIGTERM, signal.SIGHUP]
)
@pytest.mark.parametrize("runner_mode", ["spec", "spec-contracts"])
def test_runner_termination_cleans_ctgov_process_group_env_and_port(
    tmp_path: Path, termination_signal: signal.Signals, runner_mode: str
) -> None:
    workspace = tmp_path / "workspace"
    (workspace / "scripts").mkdir(parents=True)
    fixtures = workspace / "spec" / "fixtures"
    fixtures.mkdir(parents=True)
    shutil.copy2(REPO_ROOT / "scripts" / "run-specs.sh", workspace / "scripts")
    for name in (
        "fixture-supervisor.py",
        "fixture-supervisor.sh",
        "routine-fixture-ownership.sh",
        "setup-article-fulltext-source-fixture.sh",
        "cleanup-article-fulltext-source-fixture.sh",
        "setup-ctgov-intervention-alias-spec-fixture.sh",
        "cleanup-ctgov-intervention-alias-spec-fixture.sh",
    ):
        shutil.copy2(REPO_ROOT / "spec" / "fixtures" / name, fixtures / name)
    shutil.copytree(
        REPO_ROOT / "tests" / "fixtures" / "article" / "fulltext",
        workspace / "tests" / "fixtures" / "article" / "fulltext",
    )
    shutil.copytree(
        REPO_ROOT / "testdata" / "sources",
        workspace / "testdata" / "sources",
    )

    for name in (
        "setup-study-spec-fixture.sh",
        "setup-ddinter-spec-fixture.sh",
        "setup-disease-survival-spec-fixture.sh",
        "cleanup-disease-survival-spec-fixture.sh",
        "setup-vaers-spec-fixture.sh",
        "cleanup-vaers-spec-fixture.sh",
        "setup-variant-identity-spec-fixture.sh",
        "cleanup-variant-identity-spec-fixture.sh",
    ):
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
        ["bash", "scripts/run-specs.sh", runner_mode],
        cwd=workspace,
        env=env,
    )
    fixture_env = workspace / ".cache" / "spec-ctgov-intervention-alias-env"
    fixture_record = workspace / ".cache" / "spec-ctgov-intervention-alias-ownership"
    try:
        _wait_until(
            lambda: ready.exists() and fixture_env.exists() and fixture_record.exists()
        )
        exports = _read_exports(fixture_env)
        record = _read_record(fixture_record)
        supervisor_pid = int(record["BIOMCP_CTGOV_INTERVENTION_ALIAS_PID"])
        server_pid = int(record["BIOMCP_CTGOV_INTERVENTION_ALIAS_SERVER_PID"])
        host, port_text = (
            exports["BIOMCP_CTGOV_BASE"]
            .removeprefix("http://")
            .split("/", 1)[0]
            .split(":")
        )
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
            [
                "bash",
                str(fixtures / "cleanup-ctgov-intervention-alias-spec-fixture.sh"),
                str(workspace),
            ],
            check=False,
        )
