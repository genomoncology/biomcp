from __future__ import annotations

import fcntl
import os
from pathlib import Path
import shlex
import shutil
import signal
import subprocess
import time

import pytest


REPO_ROOT = Path(__file__).resolve().parents[1]
pytestmark = pytest.mark.skipif(
    not Path("/proc").is_dir(),
    reason="routine ownership identity probes require Linux procfs",
)


FIXTURES = (
    (
        "article",
        "article-fulltext-source",
        "cleanup-article-fulltext-source-fixture.sh",
        "spec-article-fulltext-source",
        "spec-article-fulltext-source-env",
        "BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE",
    ),
    (
        "ctgov",
        "ctgov-intervention-alias",
        "cleanup-ctgov-intervention-alias-spec-fixture.sh",
        "spec-ctgov-intervention-alias",
        "spec-ctgov-intervention-alias-env",
        "BIOMCP_CTGOV_INTERVENTION_ALIAS",
    ),
    (
        "disease-survival",
        "disease-survival",
        "cleanup-disease-survival-spec-fixture.sh",
        "spec-disease-survival",
        "spec-disease-survival-env",
        "BIOMCP_DISEASE_SURVIVAL",
    ),
    (
        "variant-identity",
        "variant-identity",
        "cleanup-variant-identity-spec-fixture.sh",
        "spec-variant-identity",
        "spec-variant-identity-env",
        "BIOMCP_VARIANT_IDENTITY",
    ),
    (
        "cspec",
        "clingen-cspec",
        "cleanup-clingen-cspec-spec-fixture.sh",
        "spec-clingen-cspec",
        "spec-clingen-cspec-env",
        "BIOMCP_CSPEC_FIXTURE",
    ),
)


def _process_start_identity(pid: int) -> str:
    """Return Linux procfs start time, which changes when a PID is reused."""
    stat_fields = (
        Path(f"/proc/{pid}/stat").read_text().rsplit(")", maxsplit=1)[1].split()
    )
    return stat_fields[19]


@pytest.mark.parametrize("record_kind", ["pid-reused", "disk-token-only", "live-owner"])
@pytest.mark.parametrize(
    "_name, fixture_kind, cleanup_name, root_prefix, env_name, variable_prefix",
    FIXTURES,
)
def test_cleanup_never_signals_an_unvalidated_ownership_record(
    tmp_path: Path,
    record_kind: str,
    _name: str,
    fixture_kind: str,
    cleanup_name: str,
    root_prefix: str,
    env_name: str,
    variable_prefix: str,
) -> None:
    """A disk record cannot authorize signals to a live unrelated group."""
    workspace = tmp_path / "workspace with space"
    cache = workspace / ".cache"
    cache.mkdir(parents=True)
    forged_root = cache / f"{root_prefix}.forged"
    forged_root.mkdir()
    token = f"ticket-628-{record_kind}"
    owner_root = forged_root
    if record_kind == "live-owner":
        # A live marker with a different root cannot own this record, even when
        # every other record field matches the detached sentinel.
        owner_root = cache / f"{root_prefix}.other"
        owner_root.mkdir()
    owner_arg = f"routine-fixture-owner:{fixture_kind}:{token}:{owner_root}"
    live_owner_arg = owner_arg
    if record_kind == "disk-token-only":
        live_owner_arg = f"fixture-owner {token}"
    sentinel = subprocess.Popen(
        [
            "bash",
            "-c",
            'exec -a "$1" sleep 30',
            "fixture-owner",
            live_owner_arg,
        ],
        start_new_session=True,
    )
    record_file = cache / f"spec-{fixture_kind}-ownership"
    try:
        start_identity = _process_start_identity(sentinel.pid)
        if record_kind == "pid-reused":
            start_identity = str(int(start_identity) + 1)
        exports = {
            f"{variable_prefix}_RECORD_VERSION": "1",
            f"{variable_prefix}_PID": str(sentinel.pid),
            f"{variable_prefix}_PGID": str(os.getpgid(sentinel.pid)),
            f"{variable_prefix}_ROOT": str(forged_root),
            f"{variable_prefix}_PID_START_ID": start_identity,
            f"{variable_prefix}_OWNER_WORKTREE": str(workspace),
            f"{variable_prefix}_OWNER_TOKEN": token,
            f"{variable_prefix}_OWNER_ARG": owner_arg,
        }
        record_file.write_text(
            "".join(f"{key}={value}\n" for key, value in exports.items())
        )

        subprocess.run(
            [
                "bash",
                str(REPO_ROOT / "spec" / "fixtures" / cleanup_name),
                str(workspace),
            ],
            check=True,
        )

        assert sentinel.poll() is None, (
            f"{record_kind} record for {_name} must not signal an unrelated live group"
        )
        assert forged_root.is_dir(), "invalid records must not authorize root deletion"
        assert not record_file.exists()
    finally:
        if sentinel.poll() is None:
            os.killpg(os.getpgid(sentinel.pid), signal.SIGKILL)
            sentinel.wait()


@pytest.mark.parametrize(
    "_name, _fixture_kind, cleanup_name, _root_prefix, env_name, _variable_prefix",
    FIXTURES,
)
def test_cleanup_discards_a_malformed_ownership_record_without_signaling(
    tmp_path: Path,
    _name: str,
    _fixture_kind: str,
    cleanup_name: str,
    _root_prefix: str,
    env_name: str,
    _variable_prefix: str,
) -> None:
    """Malformed on-disk state is discarded, never evaluated or used for a signal."""
    workspace = tmp_path / "workspace with space"
    cache = workspace / ".cache"
    cache.mkdir(parents=True)
    forged_root = cache / "must-not-delete"
    forged_root.mkdir()
    sentinel = subprocess.Popen(["sleep", "30"], start_new_session=True)
    record_file = cache / f"spec-{_fixture_kind}-ownership"
    try:
        record_file.write_text("this is not an ownership record\n")
        completed = subprocess.run(
            [
                "bash",
                str(REPO_ROOT / "spec" / "fixtures" / cleanup_name),
                str(workspace),
            ],
            check=False,
        )
        assert completed.returncode == 0, "malformed records must be discarded safely"
        assert sentinel.poll() is None, "malformed records must not authorize a signal"
        assert forged_root.is_dir(), (
            "malformed records must not authorize root deletion"
        )
        assert not record_file.exists()
    finally:
        if sentinel.poll() is None:
            os.killpg(os.getpgid(sentinel.pid), signal.SIGKILL)
            sentinel.wait()


SERVER_FIXTURES = (
    (
        "article",
        "article-fulltext-source",
        "setup-article-fulltext-source-fixture.sh",
        "cleanup-article-fulltext-source-fixture.sh",
        "spec-article-fulltext-source-env",
        "BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE_PID",
    ),
    (
        "ctgov",
        "ctgov-intervention-alias",
        "setup-ctgov-intervention-alias-spec-fixture.sh",
        "cleanup-ctgov-intervention-alias-spec-fixture.sh",
        "spec-ctgov-intervention-alias-env",
        "BIOMCP_CTGOV_INTERVENTION_ALIAS_PID",
    ),
    (
        "disease-survival",
        "disease-survival",
        "setup-disease-survival-spec-fixture.sh",
        "cleanup-disease-survival-spec-fixture.sh",
        "spec-disease-survival-env",
        "BIOMCP_DISEASE_SURVIVAL_PID",
    ),
    (
        "variant-identity",
        "variant-identity",
        "setup-variant-identity-spec-fixture.sh",
        "cleanup-variant-identity-spec-fixture.sh",
        "spec-variant-identity-env",
        "BIOMCP_VARIANT_IDENTITY_PID",
    ),
    (
        "cspec",
        "clingen-cspec",
        "setup-clingen-cspec-spec-fixture.sh",
        "cleanup-clingen-cspec-spec-fixture.sh",
        "spec-clingen-cspec-env",
        "BIOMCP_CSPEC_FIXTURE_PID",
    ),
)


def _wait_until(predicate, timeout: float = 10.0) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if predicate():
            return
        time.sleep(0.05)
    assert predicate()


def _read_exports(path: Path) -> dict[str, str]:
    return {
        key: shlex.split(value)[0]
        for line in path.read_text().splitlines()
        if line.startswith("export ")
        for key, value in [line.removeprefix("export ").split("=", 1)]
    }


def _read_record(path: Path) -> dict[str, str]:
    return dict(line.split("=", 1) for line in path.read_text().splitlines())


@pytest.mark.parametrize(
    "name, fixture_kind, setup_name, cleanup_name, env_name, pid_key", SERVER_FIXTURES
)
def test_routine_background_server_is_group_owned_and_drops_fd_8(
    tmp_path: Path,
    name: str,
    fixture_kind: str,
    setup_name: str,
    cleanup_name: str,
    env_name: str,
    pid_key: str,
) -> None:
    """Every server survives coordinator death safely: isolated group, no flock fd."""
    workspace = tmp_path / "workspace with space"
    workspace.mkdir()
    # The fixtures use repository-relative test data. A worktree symlink keeps
    # the probe isolated while presenting the same marker and data layout.
    for name_to_link in ("spec", "testdata", "tests"):
        (workspace / name_to_link).symlink_to(
            REPO_ROOT / name_to_link, target_is_directory=True
        )

    lock_path = workspace / ".cache" / "spec-routine-fixtures.lock"
    lock_path.parent.mkdir()
    lock = lock_path.open("w")
    try:
        subprocess.run(
            ["bash", str(REPO_ROOT / "spec" / "fixtures" / setup_name), str(workspace)],
            check=True,
            pass_fds=(lock.fileno(),),
            preexec_fn=lambda: os.dup2(lock.fileno(), 8),
        )
        exports = _read_exports(workspace / ".cache" / env_name)
        record = _read_record(workspace / ".cache" / f"spec-{fixture_kind}-ownership")
        assert pid_key not in exports, "ownership metadata must not be runner-sourced"
        leader_pid = int(record[pid_key])

        assert os.getpgid(leader_pid) == leader_pid, (
            f"{name} server must lead an isolated process group for stale recovery"
        )
        fixture_fd_8 = Path(f"/proc/{leader_pid}/fd/8")
        assert not fixture_fd_8.exists() or not os.path.samefile(
            fixture_fd_8, lock_path
        ), f"{name} server must not inherit the coordinator routine-lock descriptor"
    finally:
        subprocess.run(
            [
                "bash",
                str(REPO_ROOT / "spec" / "fixtures" / cleanup_name),
                str(workspace),
            ],
            check=False,
        )
        lock.close()


def test_sigkill_orphan_releases_routine_lock_before_stale_recovery(
    tmp_path: Path,
) -> None:
    """A successor can acquire the original lock before reaping SIGKILL orphans."""
    workspace = tmp_path / "workspace with space"
    fixtures = workspace / "spec" / "fixtures"
    fixtures.mkdir(parents=True)
    (workspace / ".cache").mkdir()
    for name in (
        "routine-fixture-ownership.sh",
        "setup-article-fulltext-source-fixture.sh",
        "cleanup-article-fulltext-source-fixture.sh",
        "setup-ctgov-intervention-alias-spec-fixture.sh",
        "cleanup-ctgov-intervention-alias-spec-fixture.sh",
    ):
        shutil.copy2(REPO_ROOT / "spec" / "fixtures" / name, fixtures / name)
    (workspace / "tests").symlink_to(REPO_ROOT / "tests", target_is_directory=True)

    ready = workspace / "ready"
    lock_path = workspace / ".cache" / "spec-routine-fixtures.lock"
    runner = subprocess.Popen(
        [
            "bash",
            "-c",
            'exec 8>"$1"; flock 8; bash "$2" "$4"; bash "$3" "$4"; touch "$5"; while :; do sleep 1 8>&-; done',
            "coordinator",
            str(lock_path),
            str(fixtures / "setup-article-fulltext-source-fixture.sh"),
            str(fixtures / "setup-ctgov-intervention-alias-spec-fixture.sh"),
            str(workspace),
            str(ready),
        ],
    )
    try:
        _wait_until(lambda: ready.exists() or runner.poll() is not None)
        assert ready.exists(), "coordinator must finish fixture setup before SIGKILL"
        runner.kill()
        assert runner.wait(timeout=10) == -signal.SIGKILL

        with lock_path.open("w") as successor_lock:
            fcntl.flock(successor_lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
            # Recovery happens only after the successor owns this same inode.
            subprocess.run(
                [
                    "bash",
                    str(fixtures / "cleanup-article-fulltext-source-fixture.sh"),
                    str(workspace),
                ],
                check=True,
            )
            subprocess.run(
                [
                    "bash",
                    str(fixtures / "cleanup-ctgov-intervention-alias-spec-fixture.sh"),
                    str(workspace),
                ],
                check=True,
            )
    finally:
        if runner.poll() is None:
            runner.kill()
            runner.wait()
        for name in (
            "cleanup-article-fulltext-source-fixture.sh",
            "cleanup-ctgov-intervention-alias-spec-fixture.sh",
        ):
            subprocess.run(["bash", str(fixtures / name), str(workspace)], check=False)
