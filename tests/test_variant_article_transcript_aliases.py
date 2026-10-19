"""Recorded transcript aliases remain visibly distinct from verified evidence."""

from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import sys

import pytest

from test_mcp_2026_protocol import RawStdioMcp

sys.path.insert(0, str(Path(__file__).resolve().parent / "golden"))
from replay_env import REPO_ROOT, isolated_command_env, replay_environment  # noqa: E402

pytestmark = [pytest.mark.needs_binary]
CAPTURE = REPO_ROOT / "tests/fixtures/article/transcript-aliases/replay"
UNKNOWN_TRANSCRIPT = "source transcript not stated by the provider"


@pytest.mark.parametrize("surface", ["json", "markdown", "mcp"])
def test_recorded_alias_only_paper_is_not_presented_as_verified_evidence(
    surface: str, tmp_path: Path, monkeypatch: pytest.MonkeyPatch,
) -> None:
    fixture_root = tmp_path / "fixture"
    fixture_root.mkdir()
    (fixture_root / "src").symlink_to(REPO_ROOT / "src", target_is_directory=True)
    (fixture_root / "testdata").mkdir()
    (fixture_root / "testdata/replay").symlink_to(CAPTURE, target_is_directory=True)
    with replay_environment(root=fixture_root) as replay:
        ready = replay.greeting
        assert ready.startswith("READY "), ready
        origin = ready.split(" ", 1)[1]
        providers = replay.provider_env
        env = isolated_command_env(tmp_path / "isolated", providers, origin)
        # This replay owns transcript provenance; MyGene expansion has separate controls.
        env["BIOMCP_DISABLE_SOURCES"] = "mygene-info"
        binary = str(os.environ.get("BIOMCP_BIN", REPO_ROOT / "target/debug/biomcp"))
        if surface == "mcp":
            with monkeypatch.context() as isolated:
                for key in set(os.environ) - set(env):
                    isolated.delenv(key)
                for key, value in env.items():
                    isolated.setenv(key, value)
                client = RawStdioMcp()
                try:
                    response = client.call("transcript-alias", "tools/call", {
                        "name": "variant_articles", "arguments": {
                            "items": [{"gene": "BRCA1", "protein": "F1761I"}],
                            "verify_identity": True, "limit": 10,
                        },
                    })["result"]
                finally:
                    client.close()
            assert response.get("isError") is not True, response
            payload = json.loads(response["content"][0]["text"])["items"][0]
        else:
            command = [binary, "--no-cache"]
            if surface == "json":
                command.append("--json")
            command += ["variant", "articles", "BRCA1 F1761I", "--strategy",
                        "union", "--verify-identity", "--limit", "10"]
            result = subprocess.run(command, env=env, cwd=REPO_ROOT,
                                    capture_output=True, text=True, timeout=60)
            assert result.returncode == 0, result.stdout + result.stderr
            if surface == "markdown":
                payload = result.stdout
            else:
                payload = json.loads(result.stdout)
        requests = replay.request_log_entries()
        misses = [row for row in requests if row.get("result") in ("miss", "record-error")]
        assert not misses, misses
        if surface == "markdown":
            print(json.dumps({"surface": surface, "output": payload}))
            assert "unverified alias match" in payload.lower()
            assert "BRCA1 F71I" in payload
            assert UNKNOWN_TRANSCRIPT in payload
        else:
            row = next(row for row in payload["results"] if row["pmid"] == "34829431")
            print(json.dumps({"surface": surface, "row": row}))
            assert row["identity"]["status"] == "unverified"
            assert row["matched_aliases"] == ["BRCA1 F71I"]
            assert row.get("match_reason") == "unverified_alias_match"
            assert row.get("alias_matches") == [{
                "alias": "BRCA1 F71I", "source_transcript": None,
                "reason": UNKNOWN_TRANSCRIPT,
            }]
            known = [fact for article in payload["results"]
                     for fact in article["alias_matches"]
                     if fact["alias"] == "BRCA1 F1714I"]
            assert known
            assert {fact["source_transcript"] for fact in known} == {"NM_007297.3"}
