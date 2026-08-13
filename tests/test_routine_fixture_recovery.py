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
    (
        "cpic",
        "cpic",
        "cleanup-cpic-spec-fixture.sh",
        "spec-cpic",
        "spec-cpic-env",
        "BIOMCP_CPIC_FIXTURE",
    ),
)


def _process_start_identity(pid: int) -> str:
    """Return Linux procfs start time, which changes when a PID is reused."""
    stat_fields = (
        Path(f"/proc/{pid}/stat").read_text().rsplit(")", maxsplit=1)[1].split()
    )
    return stat_fields[19]


@pytest.mark.parametrize(
    "record_kind", ["pid-reused", "disk-token-only", "live-owner", "unexpected-field"]
)
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
        record_text = "".join(f"{key}={value}\n" for key, value in exports.items())
        if record_kind == "unexpected-field":
            record_text += f"{variable_prefix}_UNEXPECTED_FIELD=untrusted\n"
        record_file.write_text(record_text)

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
    (
        "cpic",
        "cpic",
        "setup-cpic-spec-fixture.sh",
        "cleanup-cpic-spec-fixture.sh",
        "spec-cpic-env",
        "BIOMCP_CPIC_FIXTURE_PID",
    ),
)

SUPERVISED_SETUP_FIXTURES = (
    (
        "article-fulltext-source",
        "setup-article-fulltext-source-fixture.sh",
        "cleanup-article-fulltext-source-fixture.sh",
        "BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE",
    ),
    (
        "ctgov-intervention-alias",
        "setup-ctgov-intervention-alias-spec-fixture.sh",
        "cleanup-ctgov-intervention-alias-spec-fixture.sh",
        "BIOMCP_CTGOV_INTERVENTION_ALIAS",
    ),
    (
        "variant-identity",
        "setup-variant-identity-spec-fixture.sh",
        "cleanup-variant-identity-spec-fixture.sh",
        "BIOMCP_VARIANT_IDENTITY",
    ),
    (
        "clingen-cspec",
        "setup-clingen-cspec-spec-fixture.sh",
        "cleanup-clingen-cspec-spec-fixture.sh",
        "BIOMCP_CSPEC_FIXTURE",
    ),
    (
        "cpic",
        "setup-cpic-spec-fixture.sh",
        "cleanup-cpic-spec-fixture.sh",
        "BIOMCP_CPIC_FIXTURE",
    ),
    (
        "complexportal",
        "setup-complexportal-spec-fixture.sh",
        "cleanup-complexportal-spec-fixture.sh",
        "BIOMCP_COMPLEXPORTAL_FIXTURE",
    ),
    (
        "drug-ae-fallback",
        "setup-drug-ae-fallback-spec-fixture.sh",
        "cleanup-drug-ae-fallback-spec-fixture.sh",
        "BIOMCP_DRUG_AE_FALLBACK",
    ),
    (
        "mychem-empty",
        "setup-mychem-empty-spec-fixture.sh",
        "cleanup-mychem-empty-spec-fixture.sh",
        "BIOMCP_MYCHEM_EMPTY",
    ),
    (
        "section-outcomes",
        "setup-section-outcomes-spec-fixture.sh",
        "cleanup-section-outcomes-spec-fixture.sh",
        "BIOMCP_SECTION_OUTCOMES_FIXTURE",
    ),
    (
        "study-download-error",
        "setup-study-download-error-fixture.sh",
        "cleanup-study-download-error-fixture.sh",
        "BIOMCP_STUDY_DOWNLOAD_ERROR",
    ),
    (
        "vaers",
        "setup-vaers-spec-fixture.sh",
        "cleanup-vaers-spec-fixture.sh",
        "BIOMCP_VAERS_FIXTURE",
    ),
    (
        "article-federated-timeout",
        "setup-article-federated-timeout-fixture.sh",
        "cleanup-article-federated-timeout-fixture.sh",
        "BIOMCP_ARTICLE_FEDERATED_TIMEOUT_FIXTURE",
    ),
)

SUPERVISED_RUN_WRAPPERS = (
    (
        "run-article-semanticscholar-source",
        "run-article-semanticscholar-source-search.sh",
        "BIOMCP_RUN_ARTICLE_SEMANTICSCHOLAR_SOURCE",
    ),
    ("run-clingen-erepo", "run-clingen-erepo-fixture.sh", "BIOMCP_RUN_CLINGEN_EREPO"),
    (
        "run-section-outcome-mcp",
        "run-section-outcome-mcp.sh",
        "BIOMCP_RUN_SECTION_OUTCOME_MCP",
    ),
    (
        "run-variant-article-entity",
        "run-variant-article-entity-fixture.sh",
        "BIOMCP_RUN_VARIANT_ARTICLE_ENTITY",
    ),
    (
        "run-variant-article-identity",
        "run-variant-article-identity-fixture.sh",
        "BIOMCP_RUN_VARIANT_ARTICLE_IDENTITY",
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
    "fixture_kind, setup_name, cleanup_name, variable_prefix",
    SUPERVISED_SETUP_FIXTURES,
)
def test_adopted_setup_fixture_dies_with_its_real_exported_owner(
    tmp_path: Path,
    fixture_kind: str,
    setup_name: str,
    cleanup_name: str,
    variable_prefix: str,
) -> None:
    workspace = tmp_path / "workspace with space"
    workspace.mkdir()
    for name in ("spec", "testdata", "tests"):
        (workspace / name).symlink_to(REPO_ROOT / name, target_is_directory=True)
    ready = workspace / "owner-ready"
    owner = subprocess.Popen(
        [
            "bash",
            "-c",
            'stat="$(< /proc/$$/stat)"; rest="${stat#*) }"; '
            'read -r -a fields <<<"$rest"; '
            'export ROUTINE_FIXTURE_OWNER_PID="$$" ROUTINE_FIXTURE_OWNER_START_ID="${fields[19]}"; '
            'bash "$1" "$2" >/dev/null; touch "$3"; while :; do sleep 1; done',
            "fixture-owner",
            str(REPO_ROOT / "spec" / "fixtures" / setup_name),
            str(workspace),
            str(ready),
        ],
        start_new_session=True,
    )
    record_path = workspace / ".cache" / f"spec-{fixture_kind}-ownership"
    try:
        _wait_until(lambda: ready.exists() and record_path.exists())
        record = _read_record(record_path)
        server_pid = int(record[f"{variable_prefix}_PID"])
        fixture_root = Path(record[f"{variable_prefix}_ROOT"])
        owner.kill()
        assert owner.wait(timeout=10) == -signal.SIGKILL
        _wait_until(lambda: not Path(f"/proc/{server_pid}").exists())
        _wait_until(lambda: not fixture_root.exists())
    finally:
        if owner.poll() is None:
            owner.kill()
            owner.wait()
        subprocess.run(
            [
                "bash",
                str(REPO_ROOT / "spec" / "fixtures" / cleanup_name),
                str(workspace),
            ],
            check=False,
        )


@pytest.mark.parametrize(
    "fixture_kind, wrapper_name, variable_prefix", SUPERVISED_RUN_WRAPPERS
)
def test_run_wrapper_sigkill_reaps_server_group_and_owned_root(
    tmp_path: Path,
    fixture_kind: str,
    wrapper_name: str,
    variable_prefix: str,
) -> None:
    workspace = tmp_path / "workspace"
    workspace.mkdir()
    for name in ("spec", "testdata", "tools"):
        (workspace / name).symlink_to(REPO_ROOT / name, target_is_directory=True)
    fake_binary = tmp_path / "fake-biomcp"
    fake_binary.write_text(
        "#!/usr/bin/env bash\n"
        "set -euo pipefail\n"
        "if [[ ${1:-} == serve-http ]]; then\n"
        "  while [[ $# -gt 0 ]]; do\n"
        "    if [[ $1 == --port ]]; then port=$2; shift 2; else shift; fi\n"
        "  done\n"
        "  exec python3 - \"$port\" <<'PY'\n"
        "from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer\n"
        "import sys\n"
        "class Handler(BaseHTTPRequestHandler):\n"
        "    def log_message(self, *_): pass\n"
        "    def do_GET(self):\n"
        "        self.send_response(200 if self.path == '/readyz' else 404); self.end_headers()\n"
        "ThreadingHTTPServer(('127.0.0.1', int(sys.argv[1])), Handler).serve_forever()\n"
        "PY\n"
        "fi\n"
        "sleep 60\n",
    )
    fake_binary.chmod(0o755)
    wrapper = subprocess.Popen(
        ["bash", str(REPO_ROOT / "spec" / "fixtures" / wrapper_name), str(workspace)],
        env=os.environ
        | {
            "BIOMCP_BIN": str(fake_binary),
            "BIOMCP_SPEC_MCP_EXAMPLE_BIN": str(fake_binary),
        },
        start_new_session=True,
    )
    record_path = workspace / ".cache" / f"spec-{fixture_kind}-ownership"
    try:
        _wait_until(lambda: record_path.exists() or wrapper.poll() is not None)
        assert record_path.exists(), f"{wrapper_name} exited before exporting ownership"
        record = _read_record(record_path)
        server_pid = int(record[f"{variable_prefix}_PID"])
        fixture_root = Path(record[f"{variable_prefix}_ROOT"])
        wrapper.kill()
        assert wrapper.wait(timeout=10) == -signal.SIGKILL
        _wait_until(lambda: not Path(f"/proc/{server_pid}").exists())
        _wait_until(lambda: not fixture_root.exists())
    finally:
        if wrapper.poll() is None:
            wrapper.kill()
            wrapper.wait()
        if record_path.exists():
            subprocess.run(
                [
                    "bash",
                    str(
                        REPO_ROOT / "spec" / "fixtures" / "routine-fixture-ownership.sh"
                    ),
                    "cleanup",
                    str(workspace),
                    fixture_kind,
                    variable_prefix,
                ],
                check=False,
            )


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


def test_runner_reaps_owned_lock_holder_before_acquiring_routine_lock(
    tmp_path: Path,
) -> None:
    """A successor reaps an authenticated stale group before touching its lock."""
    workspace = tmp_path / "workspace"
    fixtures = workspace / "spec" / "fixtures"
    fixtures.mkdir(parents=True)
    (workspace / "scripts").mkdir()
    (workspace / ".cache").mkdir()
    shutil.copy2(REPO_ROOT / "scripts" / "run-specs.sh", workspace / "scripts")
    for name in (
        "routine-fixture-ownership.sh",
        "fixture-supervisor.py",
        "fixture-supervisor.sh",
        "setup-article-fulltext-source-fixture.sh",
        "cleanup-article-fulltext-source-fixture.sh",
        "cleanup-ctgov-intervention-alias-spec-fixture.sh",
    ):
        shutil.copy2(REPO_ROOT / "spec" / "fixtures" / name, fixtures / name)
    (workspace / "tests").symlink_to(REPO_ROOT / "tests", target_is_directory=True)
    shutil.copytree(
        REPO_ROOT / "testdata" / "sources",
        workspace / "testdata" / "sources",
    )
    for name in ("setup-study-spec-fixture.sh", "setup-ddinter-spec-fixture.sh"):
        script = fixtures / name
        script.write_text("#!/usr/bin/env bash\nexit 0\n")
        script.chmod(0o755)

    bin_dir = workspace / "bin"
    bin_dir.mkdir()
    for name, body in {
        "biomcp": "#!/usr/bin/env bash\nexit 0\n",
        "mustmatch": (
            "#!/usr/bin/env bash\n"
            'if [ "${1:-}" = --version ]; then echo "mustmatch 1.0.0"; fi\n'
        ),
    }.items():
        command = bin_dir / name
        command.write_text(body)
        command.chmod(0o755)

    lock_path = workspace / ".cache" / "spec-routine-fixtures.lock"
    fixture_root = workspace / ".cache" / "spec-article-fulltext-source.stale"
    fixture_root.mkdir()
    active_root = workspace / ".cache" / "spec-ctgov-intervention-alias.active"
    active_root.mkdir()
    ownership = fixtures / "routine-fixture-ownership.sh"
    owner_arg = subprocess.run(
        [
            "bash",
            str(ownership),
            "new-owner",
            "article-fulltext-source",
            str(fixture_root),
        ],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    stale = subprocess.Popen(
        [
            "bash",
            "-c",
            'exec 8>"$1"; flock 8; while :; do sleep 1; done',
            "fixture-owner",
            str(lock_path),
            owner_arg,
        ],
        start_new_session=True,
    )
    active_owner_arg = subprocess.run(
        [
            "bash",
            str(ownership),
            "new-owner",
            "ctgov-intervention-alias",
            str(active_root),
        ],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    active = subprocess.Popen(
        ["bash", "-c", 'exec -a "$1" sleep 30', "fixture-owner", active_owner_arg],
        start_new_session=True,
    )
    runner: subprocess.Popen[bytes] | None = None
    try:
        _wait_until(lambda: os.path.exists(f"/proc/{stale.pid}/fd/8"))
        subprocess.run(
            [
                "bash",
                str(ownership),
                "write",
                str(workspace),
                "article-fulltext-source",
                str(fixture_root),
                str(stale.pid),
                "BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE",
                owner_arg,
            ],
            check=True,
        )
        subprocess.run(
            [
                "bash",
                str(ownership),
                "write",
                str(workspace),
                "ctgov-intervention-alias",
                str(active_root),
                str(active.pid),
                "BIOMCP_CTGOV_INTERVENTION_ALIAS",
                active_owner_arg,
            ],
            check=True,
        )
        runner = subprocess.Popen(
            ["bash", "scripts/run-specs.sh", "spec-contracts"],
            cwd=workspace,
            env=os.environ
            | {
                "BIOMCP_BIN": str(bin_dir / "biomcp"),
                "MUSTMATCH_BIN": str(bin_dir / "mustmatch"),
            },
        )
        assert runner.wait(timeout=3) == 0
        _wait_until(lambda: stale.poll() is not None)
        assert not (
            workspace / ".cache" / "spec-article-fulltext-source-ownership"
        ).exists()
        assert active.poll() is None
        assert (
            workspace / ".cache" / "spec-ctgov-intervention-alias-ownership"
        ).exists()
    finally:
        if runner is not None and runner.poll() is None:
            runner.kill()
            runner.wait()
        if stale.poll() is None:
            os.killpg(os.getpgid(stale.pid), signal.SIGKILL)
            stale.wait()
        for cleanup_name in (
            "cleanup-article-fulltext-source-fixture.sh",
            "cleanup-ctgov-intervention-alias-spec-fixture.sh",
        ):
            subprocess.run(
                ["bash", str(fixtures / cleanup_name), str(workspace)],
                check=False,
            )
        if active.poll() is None:
            os.killpg(os.getpgid(active.pid), signal.SIGKILL)
            active.wait()


@pytest.mark.parametrize(
    "_name, fixture_kind, setup_name, cleanup_name, _env_name, pid_key",
    SERVER_FIXTURES[1:],
)
def test_routine_fixture_setup_does_not_depend_on_uv(
    tmp_path: Path,
    _name: str,
    fixture_kind: str,
    setup_name: str,
    cleanup_name: str,
    _env_name: str,
    pid_key: str,
) -> None:
    """Stdlib fixtures start and clean up without uv or leaked owned state."""
    workspace = tmp_path / "workspace"
    workspace.mkdir()
    for name in ("spec", "testdata", "tests"):
        (workspace / name).symlink_to(REPO_ROOT / name, target_is_directory=True)
    no_uv = tmp_path / "no-uv"
    no_uv.mkdir()
    uv = no_uv / "uv"
    uv.write_text(
        "#!/usr/bin/env bash\necho 'uv must not run fixture servers' >&2\nexit 97\n"
    )
    uv.chmod(0o755)
    env = os.environ | {"PATH": f"{no_uv}:{os.environ['PATH']}"}
    record_path = workspace / ".cache" / f"spec-{fixture_kind}-ownership"

    try:
        subprocess.run(
            ["bash", str(REPO_ROOT / "spec" / "fixtures" / setup_name), str(workspace)],
            check=True,
            env=env,
        )
        record = _read_record(record_path)
        server_pid = int(record[pid_key])
        fixture_root = Path(record[f"{pid_key.removesuffix('_PID')}_ROOT"])
        cleanup = subprocess.run(
            [
                "bash",
                str(REPO_ROOT / "spec" / "fixtures" / cleanup_name),
                str(workspace),
            ],
            check=False,
            env=env,
        )
        assert cleanup.returncode == 0
        _wait_until(lambda: not Path(f"/proc/{server_pid}").exists())
        assert not record_path.exists()
        assert not fixture_root.exists()
    finally:
        if record_path.exists():
            subprocess.run(
                [
                    "bash",
                    str(REPO_ROOT / "spec" / "fixtures" / cleanup_name),
                    str(workspace),
                ],
                check=False,
                env=env,
            )


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
        "fixture-supervisor.py",
        "fixture-supervisor.sh",
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
