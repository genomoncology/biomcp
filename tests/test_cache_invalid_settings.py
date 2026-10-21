"""Bad cache input is reported before a provider request on each public surface."""
from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import sys

import pytest

sys.path.insert(0, str(Path(__file__).parent / "golden"))
from replay_env import REPO_ROOT, replay_environment  # noqa: E402
from test_mcp_2026_protocol import RawStdioMcp  # noqa: E402

pytestmark = [pytest.mark.needs_binary]
BINARY = Path(os.environ.get("BIOMCP_BIN", REPO_ROOT / "target/debug/biomcp"))


@pytest.fixture
def cache_env():
    with replay_environment(record=False) as replay:
        env = replay.command_env(replay._next_run_dir())
        env.pop("BIOMCP_CACHE_MODE", None)
        for setting in ("BIOMCP_CACHE_MAX_SIZE", "BIOMCP_CACHE_MIN_DISK_FREE", "BIOMCP_CACHE_MAX_AGE"):
            env.pop(setting, None)
        yield env, replay


@pytest.mark.parametrize("setting,value", [
    ("BIOMCP_CACHE_MAX_SIZE", "10GB"),
    ("BIOMCP_CACHE_MIN_DISK_FREE", "abc"),
    ("BIOMCP_CACHE_MAX_AGE", "-1"),
    ("cache.toml", "10GB"),
])
@pytest.mark.parametrize("json_output", [False, True])
def test_bad_cache_setting_is_invalid_input(cache_env, setting, value, json_output):
    env, replay = cache_env
    if setting == "cache.toml":
        relative = "Library/Application Support" if sys.platform == "darwin" else "xdg-config"
        config = Path(env["HOME"]) / relative / "biomcp/cache.toml"
        config.parent.mkdir(parents=True, exist_ok=True)
        config.write_text('[cache]\nmax_size = "10GB"\n')
    else:
        env[setting] = value
    before = replay.request_log_line_count()
    result = subprocess.run(
        [str(BINARY), *(["--json"] if json_output else []), "search", "gene", "BRAF", "--limit", "1"],
        cwd=REPO_ROOT, env=env, capture_output=True, text=True, timeout=20,
    )
    assert result.returncode == 2, (result.returncode, result.stdout, result.stderr)
    if json_output:
        error = json.loads(result.stdout)["error"]
        assert error["code"] == "invalid_argument"
        message = error["message"]
    else:
        message = result.stderr
        assert "Invalid argument" in message
    assert setting in message and value in message, message
    assert replay.request_log_entries(since=before) == []


@pytest.mark.timeout(120)
def test_mcp_bad_cache_setting_reports_invalid_input(cache_env, monkeypatch):
    env, replay = cache_env
    env["BIOMCP_CACHE_MAX_SIZE"] = "10GB"
    for key in ("BIOMCP_CACHE_MODE", "BIOMCP_CACHE_MIN_DISK_FREE", "BIOMCP_CACHE_MAX_AGE"):
        monkeypatch.delenv(key, raising=False)
    for key, value in env.items():
        monkeypatch.setenv(key, value)
    before = replay.request_log_line_count()
    client = RawStdioMcp()
    try:
        reply = client.call("bad-cache", "tools/call", {
            "name": "biomcp", "arguments": {"command": "search gene BRAF --limit 1"},
        })
        result = reply["result"]
        assert result["isError"] is True, reply
        message = "\n".join(item.get("text", "") for item in result["content"])
        assert "Invalid argument" in message, reply
        assert "BIOMCP_CACHE_MAX_SIZE" in message and "10GB" in message, reply
        assert replay.request_log_entries(since=before) == []
    finally:
        client.close()
