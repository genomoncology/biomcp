from __future__ import annotations

import pathlib

import pytest


@pytest.fixture(scope="session")
def repo_root() -> pathlib.Path:
    return pathlib.Path(__file__).resolve().parent.parent
