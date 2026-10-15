from __future__ import annotations

import os
from pathlib import Path
import subprocess

import pytest

pytestmark = [pytest.mark.needs_binary]  # docs-only CI excludes this module

ROOT = Path(__file__).resolve().parents[1]
BINARY = Path(os.environ.get("BIOMCP_BIN", ROOT / "target" / "debug" / "biomcp"))


def test_a_lane_command_keeps_its_http_cache_out_of_home(
    tmp_path: Path,
) -> None:
    """The routine lane's cache guard, proven end to end.

    conftest points BIOMCP_CACHE_DIR at this test's own folder, so one
    binary command that builds the HTTP client must leave the user's
    home untouched even though the binary's own fallback would put its
    cache there. MyGene points at a refused loopback port so the
    command needs no network.
    """
    sentinel = tmp_path / "home-sentinel"
    sentinel.mkdir()
    subprocess.run(
        [BINARY, "search", "gene", "BRAF", "--limit", "1"],
        cwd=ROOT,
        env=os.environ
        | {
            "HOME": str(sentinel),
            "XDG_CACHE_HOME": str(sentinel),
            "BIOMCP_MYGENE_BASE": "http://127.0.0.1:9",
        },
        text=True,
        capture_output=True,
    )
    cache_root = os.environ.get("BIOMCP_CACHE_DIR")
    assert cache_root, "the lane's own cache root must be set for every test"
    assert Path(cache_root).is_dir(), "the command never built its cache root"
    assert not list(sentinel.rglob("biomcp")), "the binary used the home cache"
