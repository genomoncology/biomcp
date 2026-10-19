"""Variant article source plans and states agree across output formats."""

from __future__ import annotations

import json
import os
from pathlib import Path
import re
import subprocess
import sys

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent / "golden"))
import replay_env  # noqa: E402

pytestmark = [pytest.mark.needs_binary]
REPO_ROOT = replay_env.REPO_ROOT
CAPTURE = REPO_ROOT / "tests/fixtures/article/transcript-aliases/replay"


@pytest.mark.parametrize("authenticated", [False, True])
@pytest.mark.parametrize("scenario", ["populated", "empty_page", "disabled"])
def test_variant_article_markdown_uses_json_source_plan_and_states(
    authenticated: bool, scenario: str, tmp_path: Path, monkeypatch: pytest.MonkeyPatch,
) -> None:
    fixture_root = tmp_path / "fixture"
    fixture_root.mkdir()
    (fixture_root / "src").symlink_to(REPO_ROOT / "src", target_is_directory=True)
    replay_root = fixture_root / "testdata/replay"
    replay_root.mkdir(parents=True)
    (replay_root / "responses").symlink_to(CAPTURE / "responses", target_is_directory=True)
    manifest = json.loads((CAPTURE / "manifest.json").read_text())
    # Authored empty S2 control keeps output parity independent from retry delays.
    # All other replies retain their recorded bytes, and replay misses still fail.
    (replay_root / "s2-empty.json").write_text(json.dumps({"total": 0, "data": []}))
    for route, entry in manifest["routes"].items():
        if route.startswith("GET /s2/") and entry["status"] == 429:
            entry.update(status=200, file="s2-empty.json")
    (replay_root / "manifest.json").write_text(json.dumps(manifest))
    monkeypatch.setattr(replay_env, "REPO_ROOT", fixture_root)
    binary = os.environ.get("BIOMCP_BIN", str(REPO_ROOT / "target/debug/biomcp"))
    offset = 999 if scenario == "empty_page" else 0
    with replay_env.replay_environment(record=False) as replay:
        outputs = {}
        before = replay.request_log_line_count()
        for surface in ("json", "markdown"):
            env = replay.command_env(replay._next_run_dir())
            # Gene alias discovery has its own controls; keep this recorded
            # source-status matrix scoped to the original article routes.
            env["BIOMCP_DISABLE_SOURCES"] = "mygene-info"
            if scenario == "disabled":
                env["BIOMCP_DISABLE_SOURCES"] += ",myvariant-info"
            if authenticated:
                env["S2_API_KEY"] = "fixture-only-key"
            command = [binary, "--no-cache"]
            if surface == "json":
                command.append("--json")
            command += ["variant", "articles", "BRCA1 F1761I", "--verify-identity",
                        "--limit", "10", "--offset", str(offset)]
            result = subprocess.run(command, env=env, cwd=REPO_ROOT,
                                    capture_output=True, text=True, timeout=60)
            assert result.returncode == (1 if scenario == "disabled" else 0), result.stdout + result.stderr
            outputs[surface] = result.stderr if scenario == "disabled" and surface == "markdown" else result.stdout
        requests = replay.request_log_entries(since=before)
        assert not [row for row in requests
                    if row.get("result") in ("miss", "record-error")], requests
    payload = json.loads(outputs["json"])
    markdown = outputs["markdown"]
    print(json.dumps({"authenticated": authenticated, "scenario": scenario,
                      "source_status": payload["source_status"], "markdown": markdown}))
    expected = {(row["route"], row["source"], row["status"])
                for row in payload["source_status"]}
    actual = set(re.findall(r"^\| ([a-z_]+) \| ([a-z0-9_]+) \| ([a-z_]+) \|$",
                            markdown, flags=re.MULTILINE))
    assert actual == expected
    plan = re.search(r"^Source plan: routes \[(.*)\]$", markdown, flags=re.MULTILINE)
    assert plan, markdown
    assert set(plan[1].split(", ")) == {f"{route}/{source}" for route, source, _ in expected}
    assert "Semantic Scholar: disabled" not in markdown
    assert "Source plan: candidates" not in markdown
    if offset:
        assert not payload["results"]
        assert "No articles" in markdown
    elif scenario == "populated":
        assert payload["results"]
    else:
        assert payload["error"]
        assert not requests
