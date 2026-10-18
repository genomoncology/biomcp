from __future__ import annotations

import os
import json
from pathlib import Path
import subprocess
import sys

import pytest

sys.path.insert(0, str(Path(__file__).parent / "golden"))
from replay_env import retained_cache_env  # noqa: E402

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
        env=retained_cache_env(
            sentinel, {"BIOMCP_MYGENE_BASE": "http://127.0.0.1:9"}, "http://127.0.0.1:9",
            cache_dir=Path(os.environ["BIOMCP_CACHE_DIR"]),
        ),
        text=True,
        capture_output=True,
    )
    cache_root = os.environ.get("BIOMCP_CACHE_DIR")
    assert not list(sentinel.rglob("biomcp")), "the binary used the home cache"
    assert cache_root, "the lane's own cache root must be set for every test"
    assert Path(cache_root).is_dir(), "the command never built its cache root"


def test_a_lane_cache_uses_its_own_disk_floor(tmp_path: Path) -> None:
    result = subprocess.run(
        [BINARY, "--json", "cache", "stats"], cwd=ROOT,
        env=os.environ | {"HOME": str(tmp_path), "XDG_CONFIG_HOME": str(tmp_path / "config")},
        text=True, capture_output=True, check=True,
    )
    payload = json.loads(result.stdout)
    assert payload["min_disk_free"] == "1 B"
    assert payload["min_disk_free_origin"] == "env"
