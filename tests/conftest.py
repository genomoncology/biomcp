from __future__ import annotations

import os
import shlex
import subprocess
import tempfile
from collections.abc import AsyncIterator, Callable, Iterator
from contextlib import asynccontextmanager
from datetime import timedelta
from pathlib import Path

import pytest
from mcp import ClientSession, StdioServerParameters, types
from mcp.client.stdio import stdio_client

REPO_ROOT = Path(__file__).resolve().parents[1]
SessionFactory = Callable[
    [dict[str, str] | None],
    AsyncIterator[tuple[ClientSession, types.InitializeResult]],
]


def _default_mcp_command() -> str:
    release_bin = REPO_ROOT / "target" / "release" / "biomcp"
    if release_bin.exists():
        return f"{release_bin} serve"
    return "biomcp serve"


def pytest_addoption(parser: pytest.Parser) -> None:
    group = parser.getgroup("mcp")
    group.addoption(
        "--mcp-cmd",
        action="store",
        default=None,
        help="Command used to launch MCP server (default: 'biomcp serve').",
    )
    group.addoption(
        "--mcp-timeout",
        action="store",
        type=float,
        default=20.0,
        help="Timeout in seconds for MCP requests.",
    )


def _resolve_command(pytestconfig: pytest.Config) -> list[str]:
    command = (
        pytestconfig.getoption("--mcp-cmd")
        or os.environ.get("MCP_TEST_CMD")
        or _default_mcp_command()
    )
    argv = shlex.split(command)
    if not argv:
        raise pytest.UsageError("MCP command is empty. Set --mcp-cmd or MCP_TEST_CMD.")
    return argv


def _server_parameters(
    argv: list[str], extra_env: dict[str, str] | None = None
) -> StdioServerParameters:
    env = dict(os.environ)
    if extra_env:
        env.update(extra_env)
    return StdioServerParameters(
        command=argv[0],
        args=argv[1:],
        env=env,
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
    root = Path(tempfile.mkdtemp(prefix="biomcp-study-tests-"))
    try:
        yield {"BIOMCP_STUDY_DIR": _provision_study_fixture(root)}
    finally:
        subprocess.run(["rm", "-rf", str(root)], check=False)


@pytest.fixture
def mcp_session_factory(pytestconfig: pytest.Config) -> SessionFactory:
    argv = _resolve_command(pytestconfig)
    timeout_seconds = float(pytestconfig.getoption("--mcp-timeout"))

    @asynccontextmanager
    async def _open_session(
        extra_env: dict[str, str] | None = None,
    ) -> AsyncIterator[tuple[ClientSession, types.InitializeResult]]:
        parameters = _server_parameters(argv, extra_env)
        async with stdio_client(parameters) as (read_stream, write_stream):
            async with ClientSession(
                read_stream,
                write_stream,
                read_timeout_seconds=timedelta(seconds=timeout_seconds),
            ) as session:
                initialize_result = await session.initialize()
                yield session, initialize_result

    return _open_session
