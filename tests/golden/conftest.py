"""Golden-suite fixtures: the --update flag and the shared replay server."""

from __future__ import annotations

import os
from typing import Iterator

import sys
from pathlib import Path

import pytest

_HERE = Path(__file__).resolve().parent
if str(_HERE) not in sys.path:
    sys.path.insert(0, str(_HERE))


@pytest.fixture
def replay(request: pytest.FixtureRequest) -> Iterator["object"]:
    """One isolated view of the session's replay server for each golden case.

    Set BIOMCP_GOLDEN_RECORD=1 for a run that records live replies for
    unrecorded requests; without it every request must replay from disk.
    """
    from replay_env import replay_environment

    record = os.environ.get("BIOMCP_GOLDEN_RECORD") == "1"
    with replay_environment(record=record) as environment:
        yield environment


@pytest.fixture(scope="session")
def golden_update(request: pytest.FixtureRequest) -> bool:
    return bool(request.config.getoption("--update"))
