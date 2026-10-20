"""Variant literature callers can distinguish configured disables from outages."""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

import pytest

from test_mcp_2026_protocol import RawStdioMcp

_GOLDEN = Path(__file__).resolve().parent / "golden"
if str(_GOLDEN) not in sys.path:
    sys.path.insert(0, str(_GOLDEN))

from replay_env import REPO_ROOT, replay_environment  # noqa: E402

pytestmark = [pytest.mark.needs_binary]


@pytest.mark.parametrize("surface", ["cli", pytest.param("mcp", marks=pytest.mark.timeout(120))])
@pytest.mark.parametrize("disabled", ["all", "myvariant-info"])
def test_variant_literature_names_configured_resolution_disable(
    surface: str, disabled: str, monkeypatch: pytest.MonkeyPatch,
) -> None:
    binary = Path(os.environ.get("BIOMCP_BIN", REPO_ROOT / "target/debug/biomcp"))
    sources = json.loads((REPO_ROOT / "docs/reference/sources.json").read_text())
    disabled_ids = ",".join(row["id"] for row in sources) if disabled == "all" else disabled
    with replay_environment(record=False) as replay:
        env = replay.command_env(replay._next_run_dir())
        env["BIOMCP_DISABLE_SOURCES"] = disabled_ids
        before = replay.request_log_line_count()
        if surface == "cli":
            command = [
                str(binary), "--json", "variant", "articles", "BRAF V600E",
                "--limit", "1",
            ]
            result = subprocess.run(
                command, env=env, capture_output=True, text=True, timeout=30,
                cwd=REPO_ROOT,
            )
            assert result.returncode != 0, result.stdout + result.stderr
            payload = json.loads(result.stdout)
        else:
            with monkeypatch.context() as isolated:
                for key in set(os.environ) - set(env):
                    isolated.delenv(key)
                for key, value in env.items():
                    isolated.setenv(key, value)
                client = RawStdioMcp()
                try:
                    result = client.call(
                        "disabled-variant", "tools/call",
                        {"name": "variant_articles", "arguments": {
                            "items": [{"gene": "BRAF", "protein": "V600E"}],
                            "limit": 1,
                        }},
                    )["result"]
                finally:
                    client.close()
            assert result["isError"] is True, result
            batch = json.loads(result["content"][0]["text"])
            payload = batch["items"][0]
        requests = replay.request_log_entries(since=before)
        print(json.dumps({"surface": surface, "disabled": disabled,
                          "payload": payload, "requests": requests}))
        assert requests == [], requests
        assert payload["complete"] is False
        assert payload["truncated"] is True
        assert payload["results"] == []
        resolution = next(row for row in payload["source_status"]
                          if row["route"] == "resolution" and row["source"] == "myvariant")
        assert "source_disabled" in resolution["reason_codes"], resolution
        assert "provider_error" not in resolution["reason_codes"], resolution
        for text in (resolution["detail"], payload["error"]["message"]):
            assert "BIOMCP_DISABLE_SOURCES" in text, text
            assert "myvariant-info" in text, text
