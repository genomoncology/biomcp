from __future__ import annotations

from cache_fixture import set_cache_root

import os
import shutil
import subprocess
import tempfile
from collections.abc import Iterator
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]
_REPLAY_CASE_PATH = pytest.StashKey[Path]()
_REPLAY_CASE_FAILED = pytest.StashKey[bool]()


@pytest.fixture(scope="session", autouse=True)
def replay_server_owner() -> Iterator[None]:
    """Own one lazy replay server per pytest process, including xdist workers."""
    import sys

    golden = str(REPO_ROOT / "tests/golden")
    if golden not in sys.path:
        sys.path.insert(0, golden)
    from replay_env import replay_session

    with replay_session():
        yield


@pytest.fixture(autouse=True)
def replay_case_owner(
    tmp_path: Path, replay_server_owner: None, request: pytest.FixtureRequest,
) -> Iterator[None]:
    """Leave replay evidence in pytest storage for failed cases only."""
    from replay_env import replay_case_directory

    directory = tmp_path / "replay"
    request.node.stash[_REPLAY_CASE_PATH] = directory
    with replay_case_directory(directory):
        yield


@pytest.hookimpl(wrapper=True)
def pytest_runtest_makereport(item: pytest.Item, call: pytest.CallInfo):
    """Keep case storage until setup, call and every fixture teardown finish."""
    report = yield
    if report.failed:
        item.stash[_REPLAY_CASE_FAILED] = True
    if report.when == "teardown":
        directory = item.stash.get(_REPLAY_CASE_PATH, None)
        if directory is not None and not item.stash.get(_REPLAY_CASE_FAILED, False):
            shutil.rmtree(directory, ignore_errors=True)
    return report


def pytest_configure(config: pytest.Config) -> None:
    """Register the needs_binary marker: the test drives a built
    biomcp binary or the spec runner. The docs-only CI lane excludes
    these with -m "not needs_binary"; the canonical-gates lane runs
    the full suite. The release_gate marker holds the checks that
    compile the crate: the routine Python lane deselects them with
    -m "not release_gate", and `make release-gate` runs them alone.
    """
    config.pluginmanager.register(SlowTestReporter(config), "slow-test-reporter")
    config.addinivalue_line("markers", "long: exhaustive release-candidate tests")
    config.addinivalue_line(
        "markers", "needs_binary: requires a built biomcp binary"
    )
    config.addinivalue_line(
        "markers",
        "release_gate: compiles the crate; only make release-gate runs it",
    )

    config.addinivalue_line(
        "markers", "repo_audit: whole-repository audit; required by make repo-audit"
    )


class SlowTestReporter:
    """Report complete test duration on the controller, including xdist reports."""

    def __init__(self, config: pytest.Config) -> None:
        self.config = config
        self.durations: dict[str, float] = {}

    def pytest_runtest_logreport(self, report: pytest.TestReport) -> None:
        self.durations[report.nodeid] = (
            self.durations.get(report.nodeid, 0.0) + report.duration
        )
        if report.when != "teardown":
            return
        duration = self.durations.pop(report.nodeid)
        threshold = float(self.config.getini("slow_test_warning_seconds"))
        terminal = self.config.pluginmanager.get_plugin("terminalreporter")
        if threshold > 0 and duration > threshold and terminal is not None:
            try:
                load = ", ".join(f"{value:.2f}" for value in os.getloadavg())
            except (AttributeError, OSError):
                load = "unavailable"
            terminal.write_line(
                f"SLOW {report.nodeid} ({duration:.3f}s); load (1, 5, 15 min): {load}",
                yellow=True,
            )


@pytest.fixture(autouse=True)
def own_http_cache_dir(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Give every test its own HTTP cache root.

    The shipped binary falls back to the platform cache directory when
    BIOMCP_CACHE_DIR is unset, so without this guard a test that builds
    the HTTP client reads and writes the developer's own cache.
    """
    for key, value in set_cache_root({}, tmp_path / "biomcp-cache").items():
        monkeypatch.setenv(key, value)

def pytest_addoption(parser: pytest.Parser) -> None:
    parser.addini(
        "slow_test_warning_seconds",
        "Warn when a complete test exceeds this duration; never terminate it",
        default="0",
    )
    group = parser.getgroup("mcp")
    group.addoption(
        "--mcp-cmd",
        action="store",
        default=None,
        help="Deprecated compatibility option; MCP client contracts now run in Rust.",
    )
    group.addoption(
        "--mcp-timeout",
        action="store",
        type=float,
        default=20.0,
        help="Deprecated compatibility option; MCP client contracts now run in Rust.",
    )
    golden = parser.getgroup("golden")
    golden.addoption(
        "--update",
        action="store_true",
        help="rewrite the golden output files instead of diffing against them",
    )


def _provision_study_fixture(root: Path) -> str:
    script = REPO_ROOT / "spec" / "fixtures" / "setup-study-spec-fixture.sh"
    subprocess.run(["bash", str(script), str(root)], cwd=REPO_ROOT, check=True)
    result = subprocess.run(
        [
            "bash",
            "-lc",
            "source .cache/spec-study-env && printf '%s' \"$BIOMCP_STUDY_DIR\"",
        ],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    )
    study_dir = result.stdout.strip()
    if not study_dir:
        raise RuntimeError("study fixture did not set BIOMCP_STUDY_DIR")
    return study_dir


@pytest.fixture
def study_fixture_env() -> Iterator[dict[str, str]]:
    with tempfile.TemporaryDirectory(prefix="biomcp-study-tests-") as root_name:
        root = Path(root_name)
        yield {"BIOMCP_STUDY_DIR": _provision_study_fixture(root)}

