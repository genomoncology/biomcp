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
import urllib.parse
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


def _read_record(path: Path) -> dict[str, str]:
    return dict(line.split("=", 1) for line in path.read_text().splitlines())


def _copy_article_fixture(workspace: Path, *, include_data: bool = True) -> None:
    fixtures = workspace / "spec" / "fixtures"
    fixtures.mkdir(parents=True, exist_ok=True)
    for name in (
        "routine-fixture-ownership.sh",
        "fixture-supervisor.py",
        "fixture-supervisor.sh",
        "setup-article-fulltext-source-fixture.sh",
        "cleanup-article-fulltext-source-fixture.sh",
    ):
        shutil.copy2(REPO_ROOT / "spec" / "fixtures" / name, fixtures / name)
    source_root = REPO_ROOT / "testdata" / "sources"
    for path in (
        "europepmc/search_pmid_20516115.json",
        "ncbi_efetch/pmc3040717.xml",
        "ncbi_efetch/pmc6329583.xml",
        "pmc_article/pmc3040717-supplementary-tables-pow.html",
        "pmc_article/pmc3040717.html",
        "pmc_oa/pmc3040717-versions.xml",
        "pmc_oa/pmc3040717.1.json",
        "pmc_oa/pmc3040717.1.xml",
        "pubtator/export_20516115.json",
        "semantic_scholar/pmid20516115-batch.json",
        "semantic_scholar/pmid20516115-citations.json",
        "semantic_scholar/pmid20516115-recommendations.json",
        "semantic_scholar/pmid20516115-references.json",
    ):
        destination = workspace / "testdata" / "sources" / path
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source_root / path, destination)
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
        "setup-disease-survival-spec-fixture.sh",
        "cleanup-disease-survival-spec-fixture.sh",
        "setup-variant-identity-spec-fixture.sh",
        "cleanup-variant-identity-spec-fixture.sh",
        "setup-clingen-car-spec-fixture.sh",
        "cleanup-clingen-car-spec-fixture.sh",
        "setup-section-outcomes-spec-fixture.sh",
        "cleanup-section-outcomes-spec-fixture.sh",
    ):
        script = fixtures / name
        script.write_text("#!/usr/bin/env bash\nexit 0\n")
        script.chmod(0o755)
    ctgov_setup = fixtures / "setup-ctgov-intervention-alias-spec-fixture.sh"
    ctgov_setup.write_text(
        "#!/usr/bin/env bash\n"
        "mkdir -p \"$1/.cache\"\n"
        "printf 'export BIOMCP_CTGOV_BASE=http://127.0.0.1/api/v2\\n' >\"$1/.cache/spec-ctgov-intervention-alias-env\"\n"
        "printf 'export BIOMCP_CTGOV_CDN_BASE=http://127.0.0.1\\n' >>\"$1/.cache/spec-ctgov-intervention-alias-env\"\n"
    )
    ctgov_setup.chmod(0o755)
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
        'if [ -e /proc/$$/fd/8 ]; then printf "inherited-fd-8\\n" >>"$MUSTMATCH_FD_LOG"; fi\n'
        'printf "%s|%s|%s\\n" "$*" "${BIOMCP_PUBTATOR_BASE-}" '
        '"${BIOMCP_TEST_UNPACED_ORIGIN-}" >>"$MUSTMATCH_INVOCATION_LOG"\n'
        'if [ -n "${MUSTMATCH_ACTIVE_DIR:-}" ]; then\n'
        '  mkdir -p "$MUSTMATCH_ACTIVE_DIR"\n'
        '  touch "$MUSTMATCH_ACTIVE_DIR/$$"\n'
        "  trap 'rm -f \"$MUSTMATCH_ACTIVE_DIR/$$\"' EXIT\n"
        "  trap 'exit 143' TERM\n"
        "  trap 'exit 130' INT\n"
        "  trap 'exit 129' HUP\n"
        '  find "$MUSTMATCH_ACTIVE_DIR" -type f | wc -l >>"$MUSTMATCH_CONCURRENCY_LOG"\n'
        '  delay="${MUSTMATCH_DELAY:-0}"\n'
        '  if [[ -n "${MUSTMATCH_SKIP_DELAY_PATTERN:-}" && "$*" == *"$MUSTMATCH_SKIP_DELAY_PATTERN"* ]]; then delay=0; fi\n'
        '  sleep "$delay"\n'
        '  rm -f "$MUSTMATCH_ACTIVE_DIR/$$"\n'
        "fi\n"
        'printf "mustmatch-output:%s\\n" "$*"\n'
        'if [[ "$*" == *"spec/entity/article.md"* ]] && [ -f .cache/spec-article-fulltext-source-ownership ]; then\n'
        "  awk -F= '$1 == \"BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE_PID\" { print $2 }' "
        '    .cache/spec-article-fulltext-source-ownership >>"$ARTICLE_SETUP_LOG"\n'
        "fi\n"
        'call="$(wc -l <"$MUSTMATCH_INVOCATION_LOG")"\n'
        'for failure_pattern in ${FAIL_MUSTMATCH_PATTERNS//,/ }; do\n'
        '  if [[ "$*" == *"$failure_pattern"* ]]; then exit 7; fi\n'
        'done\n'
        'if [ "$call" -eq "${FAIL_MUSTMATCH_CALL:-0}" ]; then exit 7; fi\n'
        "exit 0\n"
    )
    mustmatch.chmod(0o755)
    env = os.environ | {
        "BIOMCP_BIN": str(biomcp),
        "MUSTMATCH_BIN": str(mustmatch),
        "ARTICLE_SETUP_LOG": str(workspace / "article-setup-log"),
        "MUSTMATCH_INVOCATION_LOG": str(workspace / "mustmatch-invocation-log"),
        "MUSTMATCH_FD_LOG": str(workspace / "mustmatch-fd-log"),
        "FAIL_MUSTMATCH_CALL": str(fail_mustmatch_call),
        "BIOMCP_SPEC_WORKERS": "1",
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
    _wait_until(lambda: not Path(f"/proc/{pids[0]}").exists())
    assert not (workspace / "mustmatch-fd-log").exists()
    invocations = [
        line.split("|", 2)
        for line in (workspace / "mustmatch-invocation-log").read_text().splitlines()
    ]
    assert len(invocations) == (3 if mode == "spec-contracts" else 28)
    article_args, article_base, article_origin = invocations[0]
    assert "spec/entity/article.md" in article_args
    assert "spec/entity/author.md" in article_args
    assert "spec/surface/mcp.md" in article_args
    assert article_base.startswith("http://127.0.0.1:")
    assert article_origin == article_base
    for rest_args, rest_base, rest_origin in invocations[1:]:
        assert "spec/entity/article.md" not in rest_args
        assert "spec/entity/author.md" not in rest_args
        assert "spec/surface/mcp.md" not in rest_args
        assert rest_base == "caller-pubtator"
        assert rest_origin == "http://127.0.0.1:9999"
    assert not (workspace / ".cache" / "spec-article-fulltext-source-env").exists()


def test_runner_rejects_caller_ctgov_values_when_fixture_exports_nothing(
    tmp_path: Path,
) -> None:
    workspace, env = _runner_workspace(tmp_path)
    setup = workspace / "spec" / "fixtures" / "setup-ctgov-intervention-alias-spec-fixture.sh"
    setup.write_text("#!/usr/bin/env bash\nexit 0\n")
    setup.chmod(0o755)
    cleanup = workspace / "spec" / "fixtures" / "cleanup-ctgov-intervention-alias-spec-fixture.sh"
    cleanup.write_text("#!/usr/bin/env bash\ntouch \"$1/ctgov-cleaned\"\n")
    cleanup.chmod(0o755)
    env |= {
        "BIOMCP_CTGOV_BASE": "https://clinicaltrials.gov/api/v2",
        "BIOMCP_CTGOV_CDN_BASE": "https://cdn.clinicaltrials.gov",
    }

    result = subprocess.run(
        ["bash", "scripts/run-specs.sh", "spec"],
        cwd=workspace,
        env=env,
        check=False,
        capture_output=True,
        text=True,
    )

    assert result.returncode != 0
    assert "CTGov fixture did not create" in result.stderr
    assert (workspace / "ctgov-cleaned").exists()
    assert not (workspace / "mustmatch-invocation-log").exists()


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
    _wait_until(lambda: not Path(f"/proc/{pid}").exists())
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
    fixture_record = workspace / ".cache" / "spec-article-fulltext-source-ownership"
    try:
        _wait_until(
            lambda: ready.exists() and fixture_env.exists() and fixture_record.exists()
        )
        pid = int(
            _read_record(fixture_record)["BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE_PID"]
        )
        os.kill(runner.pid, termination_signal)
        assert runner.wait(timeout=10) == 128 + termination_signal
        _wait_until(lambda: not Path(f"/proc/{pid}").exists())
        assert not fixture_env.exists()
        assert not fixture_record.exists()
    finally:
        if runner.poll() is None:
            runner.kill()
            runner.wait()


@pytest.mark.parametrize(
    "termination_signal", [signal.SIGINT, signal.SIGTERM, signal.SIGHUP]
)
def test_interrupted_routine_fixture_owns_a_separate_process_group_and_reruns(
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
    fixture_record = workspace / ".cache" / "spec-article-fulltext-source-ownership"
    try:
        _wait_until(
            lambda: ready.exists() and fixture_env.exists() and fixture_record.exists()
        )
        fixture_pid = int(
            _read_record(fixture_record)["BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE_PID"]
        )
        fixture_group = os.getpgid(fixture_pid)
        runner_group = os.getpgid(runner.pid)

        os.kill(runner.pid, termination_signal)
        assert runner.wait(timeout=10) == 128 + termination_signal
        _wait_until(lambda: not Path(f"/proc/{fixture_pid}").exists())
        assert not fixture_env.exists()
        assert not fixture_record.exists()

        successor = subprocess.run(
            ["bash", "scripts/run-specs.sh", "spec-contracts"],
            cwd=workspace,
            env=env | {"BIOMCP_SPEC_RUNNER_HOLD": "0"},
            check=False,
        )
        assert successor.returncode == 0
        successor_invocations = (workspace / "mustmatch-invocation-log").read_text()
        assert "spec/entity/article.md" in successor_invocations
        assert fixture_group != runner_group, (
            "a routine fixture must have its own process group so interruption and stale "
            "recovery can reap its descendants without signaling the coordinator group"
        )
    finally:
        if runner.poll() is None:
            runner.kill()
            runner.wait()
        subprocess.run(
            [
                "bash",
                "spec/fixtures/cleanup-article-fulltext-source-fixture.sh",
                str(workspace),
            ],
            cwd=workspace,
            check=False,
        )


def test_cleanup_refuses_mismatched_fixture_process_group(tmp_path: Path) -> None:
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
    fixture_record = workspace / ".cache" / "spec-article-fulltext-source-ownership"
    record = _read_record(fixture_record)
    fixture_pid = int(record["BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE_PID"])
    sentinel = subprocess.Popen(["sleep", "30"], start_new_session=True)
    try:
        sentinel_group = os.getpgid(sentinel.pid)
        fixture_record.write_text(
            fixture_record.read_text().replace(
                "BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE_PGID="
                f"{record['BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE_PGID']}",
                f"BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE_PGID={sentinel_group}",
            )
        )
        subprocess.run(
            [
                "bash",
                "spec/fixtures/cleanup-article-fulltext-source-fixture.sh",
                str(workspace),
            ],
            cwd=workspace,
            check=True,
        )
        assert sentinel.poll() is None
        assert Path(f"/proc/{fixture_pid}").exists()
    finally:
        if sentinel.poll() is None:
            sentinel.kill()
            sentinel.wait()
        if Path(f"/proc/{fixture_pid}").exists():
            os.killpg(os.getpgid(fixture_pid), signal.SIGKILL)
            _wait_until(lambda: not Path(f"/proc/{fixture_pid}").exists())


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


def test_article_fixture_logs_every_candidate_search_route(tmp_path: Path) -> None:
    workspace = tmp_path / "workspace"
    _copy_article_fixture(workspace)
    subprocess.run(
        ["bash", "spec/fixtures/setup-article-fulltext-source-fixture.sh", str(workspace)],
        cwd=workspace,
        check=True,
    )
    exports = _read_exports(workspace / ".cache" / "spec-article-fulltext-source-env")
    base = exports["BIOMCP_PUBTATOR_BASE"]
    request_log = Path(exports["BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE_REQUEST_LOG"])
    routes = [
        ("/search/", "text", "pubtator"),
        ("/search", "query", "europepmc"),
        ("/esearch.fcgi", "term", "pubmed"),
        ("/graph/v1/paper/search", "query", "semanticscholar"),
        ("/sentences/", "query", "litsense2"),
    ]
    try:
        assert exports["BIOMCP_LITSENSE2_BASE"] == base
        request_log.write_text("", encoding="utf-8")
        for path, parameter, _ in routes:
            query = urllib.parse.urlencode({parameter: "route proof"})
            _status(f"{base}{path}?{query}")
        assert request_log.read_text(encoding="utf-8").splitlines() == [
            f"search:{provider}:route proof" for _, _, provider in routes
        ]
    finally:
        subprocess.run(
            ["bash", "spec/fixtures/cleanup-article-fulltext-source-fixture.sh", str(workspace)],
            cwd=workspace,
            check=True,
        )


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


def test_article_graph_fixture_records_one_requested_page_after_seed_resolution(
    tmp_path: Path,
) -> None:
    workspace = tmp_path / "workspace"
    _copy_article_fixture(workspace)
    subprocess.run(
        ["bash", "spec/fixtures/setup-article-fulltext-source-fixture.sh", str(workspace)],
        cwd=workspace,
        check=True,
    )
    exports = _read_exports(workspace / ".cache" / "spec-article-fulltext-source-env")
    request_log = Path(exports["BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE_REQUEST_LOG"])
    sentinel = "fixture-secret-key-663"
    try:
        for direction in ("citations", "references"):
            request_log.write_text("", encoding="utf-8")
            result = subprocess.run(
                [Path(os.environ["BIOMCP_BIN"]), "--json", "article", direction, "20516115", "--limit", "1", "--offset", "1"],
                env=os.environ | exports | {"BIOMCP_CACHE_DIR": str(tmp_path / "cache"), "S2_API_KEY": sentinel},
                capture_output=True,
                text=True,
                check=True,
            )
            assert request_log.read_text().splitlines() == [
                "s2:seed:x-api-key:present",
                f"s2:graph:{direction}:limit=1:offset=1:x-api-key:present",
            ]
            assert sentinel not in result.stdout
            assert sentinel not in result.stderr
    finally:
        subprocess.run(
            ["bash", "spec/fixtures/cleanup-article-fulltext-source-fixture.sh", str(workspace)],
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
        env = (
            os.environ
            | exports
            | {
                "BIOMCP_CACHE_DIR": str(tmp_path / cache_name),
            }
        )
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
        assert (
            request_log.read_text().splitlines().count("indexing:xml:pubmed-efetch")
            == 1
        )

        run(
            "--json",
            "get",
            "article",
            "22663011",
            "all",
            cache_name="all-cache",
        )
        assert (
            request_log.read_text().splitlines().count("indexing:xml:pubmed-efetch")
            == 2
        )
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


def test_concurrent_routine_runners_serialize_shared_fixtures(tmp_path: Path) -> None:
    workspace, env = _runner_workspace(tmp_path)
    setup = workspace / "spec" / "fixtures" / "setup-variant-identity-spec-fixture.sh"
    setup.write_text(
        "#!/usr/bin/env bash\n"
        "set -euo pipefail\n"
        'active="$1/.cache/variant-fixture-active"\n'
        'if ! mkdir "$active" 2>/dev/null; then touch "$1/fixture-collision"; exit 1; fi\n'
        "sleep 0.2\n"
        'rmdir "$active"\n'
    )
    setup.chmod(0o755)

    runners = [
        subprocess.Popen(
            ["bash", "scripts/run-specs.sh", "spec"], cwd=workspace, env=env
        )
        for _ in range(2)
    ]

    assert all(process.wait(timeout=30) == 0 for process in runners)
    assert not (workspace / "fixture-collision").exists()


def test_runner_defaults_to_four_independent_markdown_workers(tmp_path: Path) -> None:
    workspace, env = _runner_workspace(tmp_path)
    env.pop("BIOMCP_SPEC_WORKERS")
    concurrency_log = workspace / "mustmatch-concurrency-log"
    env |= {
        "MUSTMATCH_ACTIVE_DIR": str(workspace / "mustmatch-active"),
        "MUSTMATCH_CONCURRENCY_LOG": str(concurrency_log),
        "MUSTMATCH_DELAY": "0.1",
    }

    result = subprocess.run(
        ["bash", "scripts/run-specs.sh", "spec"], cwd=workspace, env=env, check=False
    )

    assert result.returncode == 0
    assert max(map(int, concurrency_log.read_text().splitlines())) == 4


def test_runner_can_force_one_markdown_worker_for_diagnosis(tmp_path: Path) -> None:
    workspace, env = _runner_workspace(tmp_path)
    concurrency_log = workspace / "mustmatch-concurrency-log"
    env |= {
        "BIOMCP_SPEC_WORKERS": "1",
        "MUSTMATCH_ACTIVE_DIR": str(workspace / "mustmatch-active"),
        "MUSTMATCH_CONCURRENCY_LOG": str(concurrency_log),
        "MUSTMATCH_DELAY": "0.01",
    }

    result = subprocess.run(
        ["bash", "scripts/run-specs.sh", "spec"], cwd=workspace, env=env, check=False
    )

    assert result.returncode == 0
    assert max(map(int, concurrency_log.read_text().splitlines())) == 1


def test_runner_rejects_invalid_routine_worker_count_before_fixture_setup(
    tmp_path: Path,
) -> None:
    workspace, env = _runner_workspace(tmp_path)
    env["BIOMCP_SPEC_WORKERS"] = "unbounded"

    result = subprocess.run(
        ["bash", "scripts/run-specs.sh", "spec"],
        cwd=workspace,
        env=env,
        check=False,
        capture_output=True,
        text=True,
    )

    assert result.returncode == 2
    assert "BIOMCP_SPEC_WORKERS must be a positive integer" in result.stderr
    assert not (workspace / "article-setup-log").exists()


def test_static_pages_keep_their_single_serial_invocation(tmp_path: Path) -> None:
    workspace, env = _runner_workspace(tmp_path)
    env["BIOMCP_SPEC_WORKERS"] = "unbounded"

    result = subprocess.run(
        ["bash", "scripts/run-specs.sh", "spec-static"],
        cwd=workspace,
        env=env,
        check=False,
    )

    assert result.returncode == 0
    invocations = (workspace / "mustmatch-invocation-log").read_text().splitlines()
    assert len(invocations) == 1
    assert "spec/surface/docker-image.md spec/surface/homebrew.md" in invocations[0]


def test_failing_parallel_pages_are_aggregated_and_stop_new_batches(
    tmp_path: Path,
) -> None:
    workspace, env = _runner_workspace(tmp_path)
    failed_pages = ("spec/entity/disease.md", "spec/entity/disease-survival-fixture.md")
    env |= {
        "BIOMCP_SPEC_WORKERS": "2",
        "FAIL_MUSTMATCH_PATTERNS": ",".join(failed_pages),
    }

    result = subprocess.run(
        ["bash", "scripts/run-specs.sh", "spec"],
        cwd=workspace,
        env=env,
        check=False,
        capture_output=True,
        text=True,
    )

    assert result.returncode != 0
    for failed_page in failed_pages:
        assert f"spec page failed: {failed_page} (exit 7)" in result.stderr
        assert f"mustmatch-output:test {failed_page}" in result.stdout
    invocations = (workspace / "mustmatch-invocation-log").read_text()
    assert "spec/entity/pgx.md" not in invocations
    assert result.stdout.index(failed_pages[0]) < result.stdout.index(failed_pages[1])


def test_interrupt_reaps_parallel_markdown_workers(tmp_path: Path) -> None:
    workspace, env = _runner_workspace(tmp_path)
    active_dir = workspace / "mustmatch-active"
    env |= {
        "BIOMCP_SPEC_WORKERS": "4",
        "MUSTMATCH_ACTIVE_DIR": str(active_dir),
        "MUSTMATCH_CONCURRENCY_LOG": str(workspace / "mustmatch-concurrency-log"),
        "MUSTMATCH_DELAY": "30",
        "MUSTMATCH_SKIP_DELAY_PATTERN": "spec/entity/article.md",
    }
    runner = subprocess.Popen(
        ["bash", "scripts/run-specs.sh", "spec"], cwd=workspace, env=env
    )
    try:
        _wait_until(lambda: active_dir.exists() and len(list(active_dir.iterdir())) >= 2)
        worker_pids = [int(path.name) for path in active_dir.iterdir()]
        runner.terminate()
        assert runner.wait(timeout=10) == 128 + signal.SIGTERM
        _wait_until(
            lambda: all(not Path(f"/proc/{worker_pid}").exists() for worker_pid in worker_pids)
        )
    finally:
        if runner.poll() is None:
            runner.kill()
            runner.wait()


def test_only_owned_article_fixtures_export_unpaced_origin() -> None:
    exporters = {
        path.name
        for path in (REPO_ROOT / "spec" / "fixtures").glob("*.sh")
        if "BIOMCP_TEST_UNPACED_ORIGIN" in path.read_text()
    }
    assert exporters == {
        "run-article-semanticscholar-source-search.sh",
        "run-variant-article-entity-fixture.sh",
        "run-variant-article-identity-fixture.sh",
        "run-variant-articles-live-canary.sh",
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
    records = [
        _read_record(workspace / ".cache" / "spec-article-fulltext-source-ownership")
        for workspace in workspaces
    ]
    roots = {item["BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE_ROOT"] for item in records}
    pids = {item["BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE_PID"] for item in records}
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
        first_pid = records[0]["BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE_PID"]
        second_pid = records[1]["BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE_PID"]
        # Cleanup stops the owned process group before returning, but the
        # detached supervisor can remain briefly visible in procfs until its
        # host parent reaps it.
        _wait_until(lambda: not Path(f"/proc/{first_pid}").exists())
        assert Path(f"/proc/{second_pid}").exists()
        assert (
            _status(f"{exports[1]['BIOMCP_FIGSHARE_BASE']}/graph/v1/author/1716151")
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
