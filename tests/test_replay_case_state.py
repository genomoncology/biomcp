"""A failed pytest replay case leaves usable evidence after its runner exits."""
from __future__ import annotations

import json
import os
from pathlib import Path
import sqlite3
import subprocess
import sys

import pytest

ROOT = Path(__file__).resolve().parents[1]


@pytest.mark.needs_binary
def test_failed_replay_case_keeps_cache_and_log_while_passing_case_cleans_up(
    tmp_path: Path,
) -> None:
    project = tmp_path / "runner"
    project.mkdir()
    (project / "conftest.py").symlink_to(ROOT / "tests/conftest.py")
    (project / "test_cases.py").write_text('''
import json, os, subprocess
from pathlib import Path
import pytest
from replay_env import replay_environment, retained_cache_env

@pytest.mark.parametrize("fails", [False, True], ids=["passing", "failing"])
def test_replayed_case(tmp_path, fails):
    with replay_environment() as replay:
        run = replay._next_run_dir()
        result = subprocess.run(
            [os.environ["BIOMCP_BIN"], "search", "gene", "BRAF", "--limit", "1"],
            env=retained_cache_env(run, replay.provider_env, replay.origin),
            capture_output=True, text=True, timeout=120,
        )
        assert result.returncode == 0, result.stderr
        assert replay.request_log_entries()
        (Path(__file__).parent / ("failing.json" if fails else "passing.json")).write_text(
            json.dumps({"work": str(replay.work_dir), "run": str(run)}))
        assert not fails, "deliberate retained-state control"
''')
    env = os.environ.copy()
    env["BIOMCP_BIN"] = str(Path(env.get("BIOMCP_BIN", ROOT / "target/debug/biomcp")).resolve())
    env["PYTHONPATH"] = os.pathsep.join((str(ROOT / "tests"), str(ROOT / "tests/golden")))
    result = subprocess.run(
        [sys.executable, "-m", "pytest", "-c", str(ROOT / "pyproject.toml"),
         str(project / "test_cases.py"), "--basetemp", str(project / "cases"), "-q"],
        env=env, capture_output=True, text=True, timeout=180,
    )
    (project / "pytest.log").write_text(result.stdout + result.stderr)
    assert result.returncode == 1, result.stdout + result.stderr
    assert "1 failed, 1 passed" in result.stdout, result.stdout + result.stderr
    assert "deliberate retained-state control" in result.stdout
    failed = json.loads((project / "failing.json").read_text())
    passed = json.loads((project / "passing.json").read_text())
    assert Path(failed["run"]).is_relative_to(project / "cases")
    assert Path(failed["run"]).is_dir(), "failed replay command state was deleted"
    database = Path(failed["run"]) / "cache/biomcp.db"
    assert database.is_file(), "failed replay cache database was deleted"
    with sqlite3.connect(database) as connection:
        assert connection.execute("select count(*) from entries").fetchone()[0] > 0
    entries = [json.loads(line) for line in
               (Path(failed["work"]) / "request.log").read_text().splitlines()]
    assert entries and all(entry["result"] == "hit" for entry in entries)
    assert not Path(passed["work"]).exists(), "passing replay case was retained"
