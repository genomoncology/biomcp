"""Validate focused Python collection inside the one execution process."""

from __future__ import annotations

import os
from pathlib import Path

import pytest

from biodata_focused_selection import SelectionError, load_selection, validate_python_collection


def pytest_collection_modifyitems(items: list[pytest.Item]) -> None:
    manifest = os.environ.get("BIOMCP_FOCUSED_MANIFEST")
    if manifest is None:
        raise pytest.UsageError("BIOMCP_FOCUSED_MANIFEST is required")
    try:
        selection = load_selection(Path(manifest))
        validate_python_collection(selection, tuple(item.nodeid for item in items))
    except SelectionError as error:
        raise pytest.UsageError(str(error)) from error
