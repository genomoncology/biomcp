from __future__ import annotations

import os
from pathlib import Path
import shlex
import shutil
import signal
import socket
import subprocess
import time
import urllib.error
import urllib.request

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
        if line.startswith("export "):
            key, value = line.removeprefix("export ").split("=", 1)
            exports[key] = shlex.split(value)[0]
    return exports


def _copy_article_fixture(workspace: Path, *, include_data: bool = True) -> None:
    fixtures = workspace / "spec" / "fixtures"
    fixtures.mkdir(parents=True, exist_ok=True)
    for name in (
        "setup-article-fulltext-source-fixture.sh",
        "cleanup-article-fulltext-source-fixture.sh",
    ):
        shutil.copy2(REPO_ROOT / "spec" / "fixtures" / name, fixtures / name)
    if include_data:
        shutil.copytree(
            REPO_ROOT / "tests" / "fixtures" / "article" / "fulltext",
            workspace / "tests" / "fixtures" / "article" / "fulltext",
        )


def _runner_workspace(
    tmp_path: Path, fail_mustmatch_call: int = 0
) -> tuple[Path, dict[str, str]]:
    workspace = tmp_path / "workspace"
    (workspace / "scripts").mkdir(parents=True)
    shutil.copy2(REPO_ROOT / "scripts" / "run-specs.sh", workspace / "scripts")
    _copy_article_fixture(workspace)

    fixtures = workspace / "spec" / "fixtures"
    for name in (
        "setup-study-spec-fixture.sh",
        "setup-ddinter-spec-fixture.sh",
        "setup-ctgov-intervention-alias-spec-fixture.sh",
        "setup-disease-survival-spec-fixture.sh",
        "cleanup-disease-survival-spec-fixture.sh",
        "setup-variant-identity-spec-fixture.sh",
        "cleanup-variant-identity-spec-fixture.sh",
        "setup-section-outcomes-spec-fixture.sh",
        "cleanup-section-outcomes-spec-fixture.sh",
    ):
        script = fixtures / name
        script.write_text("#!/usr/bin/env bash\nexit 0\n")
        script.chmod(0o755)
    cleanup = fixtures / "cleanup-ctgov-intervention-alias-spec-fixture.sh"
    cleanup.write_text("#!/usr/bin/env bash\nexit 0\n")
    cleanup.chmod(0o755)
    surface_contract = (
        workspace / "tests" / "surface" / "test_parallel_isolation_contract.py"
    )
    surface_contract.parent.mkdir(parents=True, exist_ok=True)
    surface_contract.write_text(
        "import os\n\n"
        "def test_placeholder():\n"
        '    assert os.environ["BIOMCP_PUBTATOR_BASE"] == "caller-pubtator"\n'
        '    assert os.environ["BIOMCP_TEST_UNPACED_ORIGIN"] == "http://127.0.0.1:9999"\n'
    )

    bin_dir = workspace / "bin"
    bin_dir.mkdir()
    biomcp = bin_dir / "biomcp"
    biomcp.write_text("#!/usr/bin/env bash\nexit 0\n")
    biomcp.chmod(0o755)
    mustmatch = bin_dir / "mustmatch"
    mustmatch.write_text(
        "#!/usr/bin/env bash\n"
        'if [ "${1:-}" = --version ]; then echo "mustmatch 1.0.0"; exit 0; fi\n'
        'printf "%s|%s|%s\\n" "$*" "${BIOMCP_PUBTATOR_BASE-}" '
        '"${BIOMCP_TEST_UNPACED_ORIGIN-}" >>"$MUSTMATCH_INVOCATION_LOG"\n'
        'if [ -n "${BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE_PID-}" ]; then\n'
        '  printf "%s\\n" "$BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE_PID" >>"$ARTICLE_SETUP_LOG"\n'
        "fi\n"
        'call="$(wc -l <"$MUSTMATCH_INVOCATION_LOG")"\n'
        'if [ "$call" -eq "${FAIL_MUSTMATCH_CALL:-0}" ]; then exit 7; fi\n'
        "exit 0\n"
    )
    mustmatch.chmod(0o755)
    env = os.environ | {
        "BIOMCP_BIN": str(biomcp),
        "MUSTMATCH_BIN": str(mustmatch),
        "ARTICLE_SETUP_LOG": str(workspace / "article-setup-log"),
        "MUSTMATCH_INVOCATION_LOG": str(workspace / "mustmatch-invocation-log"),
        "FAIL_MUSTMATCH_CALL": str(fail_mustmatch_call),
        "BIOMCP_PUBTATOR_BASE": "caller-pubtator",
        "BIOMCP_TEST_UNPACED_ORIGIN": "http://127.0.0.1:9999",
    }
    return workspace, env


@pytest.mark.parametrize("mode", ["spec", "spec-pr", "spec-contracts"])
def test_runner_starts_one_article_fixture_and_cleans_it(
    mode: str, tmp_path: Path
) -> None:
    workspace, env = _runner_workspace(tmp_path)
    result = subprocess.run(
        ["bash", "scripts/run-specs.sh", mode], cwd=workspace, env=env, check=False
    )
    assert result.returncode == 0
    pids = (workspace / "article-setup-log").read_text().splitlines()
    assert len(pids) == 1
    assert not Path(f"/proc/{pids[0]}").exists()
    invocations = [
        line.split("|", 2)
        for line in (workspace / "mustmatch-invocation-log").read_text().splitlines()
    ]
    assert len(invocations) == (2 if mode == "spec-contracts" else 3)
    article_args, article_base, article_origin = invocations[0]
    assert "spec/entity/article.md" in article_args
    assert "spec/entity/author.md" in article_args
    assert article_base.startswith("http://127.0.0.1:")
    assert article_origin == article_base
    for rest_args, rest_base, rest_origin in invocations[1:]:
        assert "spec/entity/article.md" not in rest_args
        assert "spec/entity/author.md" not in rest_args
        assert rest_base == "caller-pubtator"
        assert rest_origin == "http://127.0.0.1:9999"
    assert not (workspace / ".cache" / "spec-article-fulltext-source-env").exists()


@pytest.mark.parametrize("fail_mustmatch_call", [1, 2])
def test_runner_cleans_article_fixture_after_child_failure(
    fail_mustmatch_call: int, tmp_path: Path
) -> None:
    workspace, env = _runner_workspace(tmp_path, fail_mustmatch_call)
    result = subprocess.run(
        ["bash", "scripts/run-specs.sh", "spec-contracts"],
        cwd=workspace,
        env=env,
        check=False,
    )
    assert result.returncode != 0
    pid = (workspace / "article-setup-log").read_text().strip()
    assert not Path(f"/proc/{pid}").exists()
    assert not (workspace / ".cache" / "spec-article-fulltext-source-env").exists()


@pytest.mark.parametrize(
    "termination_signal", [signal.SIGINT, signal.SIGTERM, signal.SIGHUP]
)
def test_runner_signal_cleans_article_fixture(
    termination_signal: signal.Signals, tmp_path: Path
) -> None:
    workspace, env = _runner_workspace(tmp_path)
    ready = workspace / "runner-ready"
    env |= {
        "BIOMCP_SPEC_RUNNER_READY_FILE": str(ready),
        "BIOMCP_SPEC_RUNNER_HOLD": "1",
    }
    runner = subprocess.Popen(
        ["bash", "scripts/run-specs.sh", "spec-contracts"], cwd=workspace, env=env
    )
    fixture_env = workspace / ".cache" / "spec-article-fulltext-source-env"
    try:
        _wait_until(lambda: ready.exists() and fixture_env.exists())
        pid = int(
            _read_exports(fixture_env)["BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE_PID"]
        )
        os.kill(runner.pid, termination_signal)
        assert runner.wait(timeout=10) == 128 + termination_signal
        _wait_until(lambda: not Path(f"/proc/{pid}").exists())
        assert not fixture_env.exists()
    finally:
        if runner.poll() is None:
            runner.kill()
            runner.wait()


def test_setup_failure_cleans_started_process_and_root(tmp_path: Path) -> None:
    workspace = tmp_path / "workspace"
    _copy_article_fixture(workspace, include_data=False)
    result = subprocess.run(
        [
            "bash",
            "spec/fixtures/setup-article-fulltext-source-fixture.sh",
            str(workspace),
        ],
        cwd=workspace,
        text=True,
        capture_output=True,
        check=False,
    )
    assert result.returncode != 0
    assert not (workspace / ".cache" / "spec-article-fulltext-source-env").exists()
    assert not [
        path
        for path in (workspace / ".cache").glob("spec-article-fulltext-source.*")
        if path.is_dir()
    ]


def _status(url: str) -> int:
    try:
        with urllib.request.urlopen(url) as response:
            return response.status
    except urllib.error.HTTPError as error:
        return error.code


def test_metadata_resets_only_cold_storage_download_state(tmp_path: Path) -> None:
    workspace = tmp_path / "workspace"
    _copy_article_fixture(workspace)
    subprocess.run(
        [
            "bash",
            "spec/fixtures/setup-article-fulltext-source-fixture.sh",
            str(workspace),
        ],
        cwd=workspace,
        check=True,
    )
    fixture_env = workspace / ".cache" / "spec-article-fulltext-source-env"
    exports = _read_exports(fixture_env)
    base = exports["BIOMCP_FIGSHARE_BASE"]
    assert exports["BIOMCP_TEST_UNPACED_ORIGIN"] == base
    request_log = Path(exports["BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE_REQUEST_LOG"])
    try:
        for _ in range(2):
            assert _status(f"{base}/v2/articles/22474830") == 200
            assert (
                _status(f"{base}/figshare/files/39926330/cold-storage-supplement.pdf")
                == 202
            )
            assert (
                _status(f"{base}/figshare/files/39926330/cold-storage-supplement.pdf")
                == 200
            )
        before = request_log.read_text()
        assert _status(f"{base}/v2/articles/99999999") == 200
        assert request_log.read_text() == before
    finally:
        subprocess.run(
            [
                "bash",
                "spec/fixtures/cleanup-article-fulltext-source-fixture.sh",
                str(workspace),
            ],
            cwd=workspace,
            check=True,
        )


def test_article_indexing_request_is_opt_in_and_all_includes_it(tmp_path: Path) -> None:
    workspace = tmp_path / "workspace"
    _copy_article_fixture(workspace)
    subprocess.run(
        [
            "bash",
            "spec/fixtures/setup-article-fulltext-source-fixture.sh",
            str(workspace),
        ],
        cwd=workspace,
        check=True,
    )
    fixture_env = workspace / ".cache" / "spec-article-fulltext-source-env"
    exports = _read_exports(fixture_env)
    request_log = Path(exports["BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE_REQUEST_LOG"])
    binary = Path(os.environ["BIOMCP_BIN"])

    def run(*args: str, cache_name: str) -> None:
        env = os.environ | exports | {
            "BIOMCP_CACHE_DIR": str(tmp_path / cache_name),
        }
        subprocess.run([binary, *args], env=env, check=True, capture_output=True)

    try:
        run("--json", "get", "article", "22663011", cache_name="detail-cache")
        run(
            "--json",
            "search",
            "article",
            "-a",
            "Williams LS",
            "--source",
            "pubmed",
            "--limit",
            "1",
            cache_name="search-cache",
        )
        run(
            "--json",
            "article",
            "batch",
            "22663011",
            cache_name="batch-cache",
        )
        assert "indexing:xml:pubmed-efetch" not in request_log.read_text()

        run(
            "--json",
            "get",
            "article",
            "22663011",
            "indexing",
            cache_name="indexing-cache",
        )
        assert request_log.read_text().splitlines().count(
            "indexing:xml:pubmed-efetch"
        ) == 1

        run(
            "--json",
            "get",
            "article",
            "22663011",
            "all",
            cache_name="all-cache",
        )
        assert request_log.read_text().splitlines().count(
            "indexing:xml:pubmed-efetch"
        ) == 2
    finally:
        subprocess.run(
            [
                "bash",
                "spec/fixtures/cleanup-article-fulltext-source-fixture.sh",
                str(workspace),
            ],
            cwd=workspace,
            check=True,
        )


def test_only_owned_article_fixtures_export_unpaced_origin() -> None:
    exporters = {
        path.name
        for path in (REPO_ROOT / "spec" / "fixtures").glob("*.sh")
        if "BIOMCP_TEST_UNPACED_ORIGIN" in path.read_text()
    }
    assert exporters == {
        "run-article-semanticscholar-source-search.sh",
        "run-variant-article-entity-fixture.sh",
        "setup-article-federated-timeout-fixture.sh",
        "setup-article-fulltext-source-fixture.sh",
    }


def test_concurrent_workspaces_have_distinct_article_fixture_ownership(
    tmp_path: Path,
) -> None:
    workspaces = [tmp_path / "one", tmp_path / "two"]
    for workspace in workspaces:
        _copy_article_fixture(workspace)
    setups = [
        subprocess.Popen(
            [
                "bash",
                "spec/fixtures/setup-article-fulltext-source-fixture.sh",
                str(workspace),
            ],
            cwd=workspace,
        )
        for workspace in workspaces
    ]
    assert [process.wait(timeout=10) for process in setups] == [0, 0]
    exports = [
        _read_exports(workspace / ".cache" / "spec-article-fulltext-source-env")
        for workspace in workspaces
    ]
    roots = {item["BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE_ROOT"] for item in exports}
    pids = {item["BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE_PID"] for item in exports}
    bases = {item["BIOMCP_FIGSHARE_BASE"] for item in exports}
    assert len(roots) == len(pids) == len(bases) == 2
    try:
        for item in exports:
            base = item["BIOMCP_FIGSHARE_BASE"]
            host, port_text = base.removeprefix("http://").split(":")
            with socket.create_connection((host, int(port_text)), timeout=1):
                pass
            assert _status(f"{base}/graph/v1/author/search?query=Name") == 200
            assert _status(f"{base}/graph/v1/author/1716151") == 200
        subprocess.run(
            [
                "bash",
                "spec/fixtures/cleanup-article-fulltext-source-fixture.sh",
                str(workspaces[0]),
            ],
            cwd=workspaces[0],
            check=True,
        )
        first_pid = exports[0]["BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE_PID"]
        second_pid = exports[1]["BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE_PID"]
        assert not Path(f"/proc/{first_pid}").exists()
        assert Path(f"/proc/{second_pid}").exists()
        assert (
            _status(f'{exports[1]["BIOMCP_FIGSHARE_BASE"]}/graph/v1/author/1716151')
            == 200
        )
    finally:
        for workspace in workspaces:
            subprocess.run(
                [
                    "bash",
                    "spec/fixtures/cleanup-article-fulltext-source-fixture.sh",
                    str(workspace),
                ],
                cwd=workspace,
                check=True,
            )
