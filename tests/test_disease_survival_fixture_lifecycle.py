from __future__ import annotations

import os
import re
import shutil
import signal
import subprocess
import time
import urllib.error
import urllib.request
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]
pytestmark = pytest.mark.skipif(
    not Path("/proc").is_dir(),
    reason="fixture owner-death probes require Linux procfs",
)


def _wait_until(predicate, timeout: float = 10.0) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if predicate():
            return
        time.sleep(0.05)
    assert predicate()


def _read_record(path: Path) -> dict[str, str]:
    return dict(line.split("=", 1) for line in path.read_text().splitlines())


def _healthz_is_unavailable(url: str) -> bool:
    try:
        with urllib.request.urlopen(url, timeout=1):
            return False
    except urllib.error.HTTPError:
        return False
    except OSError:
        return True


def _heartbeat_advances(path: Path) -> bool:
    before = path.read_text()
    time.sleep(0.2)
    return path.read_text() != before


def _disease_workspace(tmp_path: Path) -> Path:
    workspace = tmp_path / "workspace"
    workspace.mkdir()
    for name in ("spec", "testdata", "tests"):
        (workspace / name).symlink_to(REPO_ROOT / name, target_is_directory=True)
    return workspace


def _owner_identity_shell() -> str:
    return (
        'owner_stat="$(cat /proc/$$/stat)"; '
        'owner_rest="${owner_stat#*) }"; '
        'read -r -a owner_fields <<<"$owner_rest"; '
        'export ROUTINE_FIXTURE_OWNER_PID="$$"; '
        'export ROUTINE_FIXTURE_OWNER_START_ID="${owner_fields[19]}"; '
    )


def test_disease_survival_server_and_root_die_with_sigkilled_owner(
    tmp_path: Path,
) -> None:
    """A killed routine coordinator leaves no disease-survival server or root."""
    workspace = _disease_workspace(tmp_path)
    setup = REPO_ROOT / "spec" / "fixtures" / "setup-disease-survival-spec-fixture.sh"
    cleanup = (
        REPO_ROOT / "spec" / "fixtures" / "cleanup-disease-survival-spec-fixture.sh"
    )
    ready = workspace / "owner-ready"
    owner = subprocess.Popen(
        [
            "bash",
            "-c",
            _owner_identity_shell()
            + 'bash "$1" "$2"; touch "$3"; while :; do sleep 1; done',
            "fixture-owner",
            str(setup),
            str(workspace),
            str(ready),
        ],
        start_new_session=True,
    )
    record_path = workspace / ".cache" / "spec-disease-survival-ownership"
    try:
        _wait_until(lambda: ready.exists() and record_path.exists())
        record = _read_record(record_path)
        fixture_root = Path(record["BIOMCP_DISEASE_SURVIVAL_ROOT"])
        healthz_url = (fixture_root / "base-url").read_text().strip() + "/healthz"

        owner.kill()
        assert owner.wait(timeout=10) == -signal.SIGKILL

        _wait_until(lambda: _healthz_is_unavailable(healthz_url))
        _wait_until(lambda: not fixture_root.exists())
    finally:
        if owner.poll() is None:
            owner.kill()
            owner.wait()
        subprocess.run(["bash", str(cleanup), str(workspace)], check=False)


def test_disease_survival_setup_reaps_ppid_one_marker_orphan(tmp_path: Path) -> None:
    """Fixture startup collects one matching PPID-1 disease-survival orphan."""
    workspace = _disease_workspace(tmp_path)
    setup = REPO_ROOT / "spec" / "fixtures" / "setup-disease-survival-spec-fixture.sh"
    cleanup = (
        REPO_ROOT / "spec" / "fixtures" / "cleanup-disease-survival-spec-fixture.sh"
    )
    stale_root = workspace / ".cache" / "spec-disease-survival.orphan"
    stale_root.mkdir(parents=True)
    stale_owner_arg = (
        f"routine-fixture-owner:disease-survival:{'a' * 32}:{stale_root.resolve()}"
    )
    stale_heartbeat = workspace / "stale-server-heartbeat"
    decoy_root = workspace / ".cache" / "spec-disease-survival.decoy"
    decoy_root.mkdir()
    decoy_heartbeat = workspace / "decoy-server-heartbeat"
    stale_pid_file = workspace / "stale-server-pid"
    decoy_pid_file = workspace / "decoy-server-pid"
    owner = subprocess.Popen(
        [
            "bash",
            "-c",
            (
                "setsid bash -c 'while :; do date +%s%N >\"$1\"; sleep 0.05; done' "
                'fixture-server "$3" "$1/base-url" "$2" & stale="$!"; '
                "setsid bash -c 'while :; do date +%s%N >\"$1\"; sleep 0.05; done' "
                'fixture-server "$5" "$4/base-url" & decoy="$!"; '
                'printf "%s\\n" "$stale" >"$6"; '
                'printf "%s\\n" "$decoy" >"$7"; wait'
            ),
            "fixture-owner",
            str(stale_root),
            stale_owner_arg,
            str(stale_heartbeat),
            str(decoy_root),
            str(decoy_heartbeat),
            str(stale_pid_file),
            str(decoy_pid_file),
        ],
        start_new_session=True,
    )
    stale_pid: int | None = None
    decoy_pid: int | None = None
    try:
        _wait_until(
            lambda: stale_pid_file.exists()
            and decoy_pid_file.exists()
            and stale_heartbeat.exists()
            and decoy_heartbeat.exists()
        )
        stale_pid = int(stale_pid_file.read_text().strip())
        decoy_pid = int(decoy_pid_file.read_text().strip())
        _wait_until(lambda: _heartbeat_advances(stale_heartbeat))
        _wait_until(lambda: _heartbeat_advances(decoy_heartbeat))
        owner.kill()
        assert owner.wait(timeout=10) == -signal.SIGKILL

        def is_ppid_one(pid: int) -> bool:
            return (
                Path(f"/proc/{pid}/status")
                .read_text()
                .split("PPid:\t", 1)[1]
                .splitlines()[0]
                .strip()
                == "1"
            )

        _wait_until(lambda: is_ppid_one(stale_pid))
        _wait_until(lambda: is_ppid_one(decoy_pid))

        result = subprocess.run(
            ["bash", str(setup), str(workspace)],
            check=False,
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0
        _wait_until(lambda: not stale_root.exists())
        assert not _heartbeat_advances(stale_heartbeat), (
            "the matching PPID-1 fixture must stop after recovery"
        )
        assert _heartbeat_advances(decoy_heartbeat), (
            "a PPID-1 process with only a similarly named path is not an authenticated "
            "disease-survival fixture"
        )
        assert decoy_root.exists()
        assert any(
            re.search(r"\b1\b", line)
            and re.search(r"disease\W+survival", line)
            and ("reap" in line or "collect" in line)
            for line in result.stderr.lower().splitlines()
        ), "one log event must identify the collected disease-survival orphan count"
    finally:
        if owner.poll() is None:
            owner.kill()
            owner.wait()
        for pid in (stale_pid, decoy_pid):
            if pid is not None:
                try:
                    os.killpg(os.getpgid(pid), signal.SIGKILL)
                except ProcessLookupError:
                    pass
        shutil.rmtree(stale_root, ignore_errors=True)
        shutil.rmtree(decoy_root, ignore_errors=True)
        subprocess.run(["bash", str(cleanup), str(workspace)], check=False)


def test_real_runner_exports_owner_identity_to_nested_fixture_setup(
    tmp_path: Path,
) -> None:
    """Nested setup receives the real runner PID and immutable procfs identity."""
    workspace = tmp_path / "workspace"
    fixtures = workspace / "spec" / "fixtures"
    fixtures.mkdir(parents=True)
    (workspace / "scripts").mkdir()
    shutil.copy2(REPO_ROOT / "scripts" / "run-specs.sh", workspace / "scripts")
    identity_file = workspace / "nested-owner-identity"
    (fixtures / "setup-article-fulltext-source-fixture.sh").write_text(
        "#!/usr/bin/env bash\n"
        "set -euo pipefail\n"
        ': "${ROUTINE_FIXTURE_OWNER_PID:?}"\n'
        ': "${ROUTINE_FIXTURE_OWNER_START_ID:?}"\n'
        '[[ "$ROUTINE_FIXTURE_OWNER_PID" == "$PPID" ]]\n'
        'owner_stat="$(cat /proc/$ROUTINE_FIXTURE_OWNER_PID/stat)"\n'
        'owner_rest="${owner_stat#*) }"\n'
        'read -r -a owner_fields <<<"$owner_rest"\n'
        '[[ "${owner_fields[19]}" == "$ROUTINE_FIXTURE_OWNER_START_ID" ]]\n'
        'printf "validated\\n" >"${OWNER_IDENTITY_FILE:?}"\n'
    )
    for name in (
        "cleanup-article-fulltext-source-fixture.sh",
        "setup-study-spec-fixture.sh",
        "setup-ddinter-spec-fixture.sh",
        "cleanup-ctgov-intervention-alias-spec-fixture.sh",
    ):
        (fixtures / name).write_text("#!/usr/bin/env bash\nexit 0\n")
    (fixtures / "setup-ctgov-intervention-alias-spec-fixture.sh").write_text(
        "#!/usr/bin/env bash\n"
        'mkdir -p "$1/.cache"\n'
        'printf "export BIOMCP_CTGOV_BASE=http://127.0.0.1/api/v2\\n" '
        '>"$1/.cache/spec-ctgov-intervention-alias-env"\n'
        'printf "export BIOMCP_CTGOV_CDN_BASE=http://127.0.0.1\\n" '
        '>>"$1/.cache/spec-ctgov-intervention-alias-env"\n'
    )
    for script in fixtures.iterdir():
        script.chmod(0o755)

    bin_dir = workspace / "bin"
    bin_dir.mkdir()
    for name, body in {
        "biomcp": "#!/usr/bin/env bash\nexit 0\n",
        "mustmatch": (
            "#!/usr/bin/env bash\n"
            'if [[ ${1:-} == --version ]]; then echo "mustmatch 1.0.0"; fi\n'
        ),
    }.items():
        command = bin_dir / name
        command.write_text(body)
        command.chmod(0o755)

    completed = subprocess.run(
        ["bash", "scripts/run-specs.sh", "spec-contracts"],
        cwd=workspace,
        env=os.environ
        | {
            "BIOMCP_BIN": str(bin_dir / "biomcp"),
            "MUSTMATCH_BIN": str(bin_dir / "mustmatch"),
            "OWNER_IDENTITY_FILE": str(identity_file),
        },
        check=False,
    )

    assert completed.returncode == 0
    assert identity_file.read_text() == "validated\n"


def test_real_bounded_runner_timeout_reaps_disease_server_and_root(
    tmp_path: Path,
) -> None:
    """A real bounded routine run leaves no disease fixture after it is killed."""
    workspace = tmp_path / "workspace"
    fixtures = workspace / "spec" / "fixtures"
    fixtures.mkdir(parents=True)
    (workspace / "scripts").mkdir()
    shutil.copy2(REPO_ROOT / "scripts" / "run-specs.sh", workspace / "scripts")
    for name in (
        "fixture-supervisor.py",
        "fixture-supervisor.sh",
        "routine-fixture-ownership.sh",
        "setup-disease-survival-spec-fixture.sh",
        "cleanup-disease-survival-spec-fixture.sh",
    ):
        source = REPO_ROOT / "spec" / "fixtures" / name
        if source.exists():
            shutil.copy2(source, fixtures / name)
    shutil.copytree(
        REPO_ROOT / "testdata" / "sources", workspace / "testdata" / "sources"
    )

    for name in (
        "setup-article-fulltext-source-fixture.sh",
        "cleanup-article-fulltext-source-fixture.sh",
        "setup-study-spec-fixture.sh",
        "setup-ddinter-spec-fixture.sh",
        "setup-variant-identity-spec-fixture.sh",
        "cleanup-variant-identity-spec-fixture.sh",
    ):
        script = fixtures / name
        script.write_text("#!/usr/bin/env bash\nexit 0\n")
        script.chmod(0o755)
    ctgov_setup = fixtures / "setup-ctgov-intervention-alias-spec-fixture.sh"
    ctgov_setup.write_text(
        "#!/usr/bin/env bash\n"
        'mkdir -p "$1/.cache"\n'
        'printf "export BIOMCP_CTGOV_BASE=http://127.0.0.1/api/v2\\n" '
        '>"$1/.cache/spec-ctgov-intervention-alias-env"\n'
        'printf "export BIOMCP_CTGOV_CDN_BASE=http://127.0.0.1\\n" '
        '>>"$1/.cache/spec-ctgov-intervention-alias-env"\n'
    )
    ctgov_setup.chmod(0o755)
    ctgov_cleanup = fixtures / "cleanup-ctgov-intervention-alias-spec-fixture.sh"
    ctgov_cleanup.write_text("#!/usr/bin/env bash\nexit 0\n")
    ctgov_cleanup.chmod(0o755)

    bin_dir = workspace / "bin"
    bin_dir.mkdir()
    for name, body in {
        "biomcp": "#!/usr/bin/env bash\nexit 0\n",
        "mustmatch": (
            "#!/usr/bin/env bash\n"
            'if [[ ${1:-} == --version ]]; then echo "mustmatch 1.0.0"; fi\n'
        ),
    }.items():
        command = bin_dir / name
        command.write_text(body)
        command.chmod(0o755)

    ready = workspace / "runner-ready"
    timed_run = subprocess.Popen(
        ["timeout", "--signal=KILL", "3s", "bash", "scripts/run-specs.sh", "spec"],
        cwd=workspace,
        env=os.environ
        | {
            "BIOMCP_BIN": str(bin_dir / "biomcp"),
            "MUSTMATCH_BIN": str(bin_dir / "mustmatch"),
            "BIOMCP_SPEC_RUNNER_READY_FILE": str(ready),
            "BIOMCP_SPEC_RUNNER_HOLD": "1",
        },
    )
    record_path = workspace / ".cache" / "spec-disease-survival-ownership"
    try:
        _wait_until(lambda: ready.exists() and record_path.exists())
        record = _read_record(record_path)
        fixture_root = Path(record["BIOMCP_DISEASE_SURVIVAL_ROOT"])
        healthz_url = (fixture_root / "base-url").read_text().strip() + "/healthz"

        assert timed_run.wait(timeout=10) == -signal.SIGKILL
        _wait_until(lambda: _healthz_is_unavailable(healthz_url))
        _wait_until(lambda: not fixture_root.exists())
    finally:
        if timed_run.poll() is None:
            timed_run.kill()
            timed_run.wait()
        subprocess.run(
            [
                "bash",
                str(fixtures / "cleanup-disease-survival-spec-fixture.sh"),
                str(workspace),
            ],
            check=False,
        )
