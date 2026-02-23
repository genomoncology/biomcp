from __future__ import annotations

import os
import shlex
from collections.abc import AsyncIterator, Callable
from contextlib import asynccontextmanager
from datetime import timedelta

import pytest
from mcp import ClientSession, StdioServerParameters, types
from mcp.client.stdio import stdio_client

DEFAULT_MCP_CMD = "biomcp serve"
SessionFactory = Callable[
    [],
    AsyncIterator[tuple[ClientSession, types.InitializeResult]],
]


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
        or DEFAULT_MCP_CMD
    )
    argv = shlex.split(command)
    if not argv:
        raise pytest.UsageError("MCP command is empty. Set --mcp-cmd or MCP_TEST_CMD.")
    return argv


def _server_parameters(argv: list[str]) -> StdioServerParameters:
    return StdioServerParameters(
        command=argv[0],
        args=argv[1:],
        env=dict(os.environ),
    )


@pytest.fixture
def mcp_session_factory(pytestconfig: pytest.Config) -> SessionFactory:
    argv = _resolve_command(pytestconfig)
    timeout_seconds = float(pytestconfig.getoption("--mcp-timeout"))
    parameters = _server_parameters(argv)

    @asynccontextmanager
    async def _open_session() -> AsyncIterator[tuple[ClientSession, types.InitializeResult]]:
        async with stdio_client(parameters) as (read_stream, write_stream):
            async with ClientSession(
                read_stream,
                write_stream,
                read_timeout_seconds=timedelta(seconds=timeout_seconds),
            ) as session:
                initialize_result = await session.initialize()
                yield session, initialize_result

    return _open_session
