"""Shared cache-root policy for Python fixtures."""
from __future__ import annotations

from os import PathLike
from typing import MutableMapping, TypeVar

Env = TypeVar("Env", bound=MutableMapping[str, str])


def set_cache_root(env: Env, root: str | PathLike[str]) -> Env:
    """Set isolation and its disk floor, preserving the caller's cache mode."""
    env["BIOMCP_CACHE_DIR"] = str(root)
    env["BIOMCP_CACHE_MIN_DISK_FREE"] = "1B"
    return env
