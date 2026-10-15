"""Golden-suite fixtures: the --update flag and the shared replay server."""

from __future__ import annotations

import os
from typing import Iterator

import pytest

from replay_env import ReplayEnvironment, replay_environment


def pytest_addoption(parser: pytest.Parser) -> None:
    group = parser.getgroup("golden")
    group.addoption(
        "--update",
        action="store_true",
        help="rewrite the golden output files instead of diffing against them",
    )


@pytest.fixture(scope="session")
def replay(request: pytest.FixtureRequest) -> Iterator[ReplayEnvironment]:
    """One replay server for the whole golden run.

    Set BIOMCP_GOLDEN_RECORD=1 for a run that records live replies for
    unrecorded requests; without it every request must replay from disk.
    """
    record = os.environ.get("BIOMCP_GOLDEN_RECORD") == "1"
    with replay_environment(record=record) as environment:
        yield environment


@pytest.fixture(scope="session")
def golden_update(request: pytest.FixtureRequest) -> bool:
    return bool(request.config.getoption("--update"))
