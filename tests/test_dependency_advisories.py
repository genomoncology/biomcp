"""Check the policy and lock inputs passed to the authoritative scanner."""
from __future__ import annotations

import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tomllib

import pytest

from test_lint import REPO_ROOT, _write_executable


def _fixture(tmp_path: Path) -> tuple[Path, Path, dict[str, str]]:
    root = tmp_path / "repo"
    root.mkdir()
    shutil.copy2(REPO_ROOT / "tools/check-dependency-advisories", root / "check")
    shutil.copy2(REPO_ROOT / "deny.toml", root / "deny.toml")
    for name in ("Cargo.lock", "uv.lock"):
        (root / name).write_text('[[package]]\nname = "fixture"\nversion = "1.0.0"\n')
    scanner = root / "scanner"
    _write_executable(scanner, f"#!{sys.executable}\n" + '''import json, os, pathlib, sys, tomllib
config = pathlib.Path(next(arg.split('=', 1)[1] for arg in sys.argv if arg.startswith('--config=')))
pathlib.Path(os.environ['SCAN_LOG']).write_text(json.dumps({'args': sys.argv[1:], 'policy': tomllib.loads(config.read_text())}))
sys.exit(int(os.environ.get('SCAN_EXIT', '0')))
''')
    return root, scanner, os.environ | {"SCAN_LOG": str(tmp_path / "scan.json")}


def _run(root: Path, scanner: Path, env: dict[str, str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run([sys.executable, "check", str(scanner)], cwd=root,
                          env=env, capture_output=True, text=True, check=False)


def test_scanner_policy_contains_exactly_reviewed_ids_and_reasons(tmp_path: Path) -> None:
    root, scanner, env = _fixture(tmp_path)
    result = _run(root, scanner, env)
    assert result.returncode == 0, result.stdout + result.stderr
    captured = json.loads(Path(env["SCAN_LOG"]).read_text())
    reviewed = tomllib.loads((root / "deny.toml").read_text())["advisories"]["ignore"]
    assert captured["policy"] == {"IgnoredVulns": reviewed}
    assert captured["args"][:4] == ["scan", "source", "--lockfile=Cargo.lock", "--lockfile=uv.lock"]
    assert len(captured["args"]) == 5
    assert not Path(captured["args"][-1].split("=", 1)[1]).exists()


@pytest.mark.parametrize("bad_input", ["missing-cargo", "missing-uv", "malformed-cargo", "empty-uv", "invalid-package", "invalid-policy", "broadened-exception"])
def test_scanner_does_not_run_with_incomplete_locks_or_invalid_policy(tmp_path: Path, bad_input: str) -> None:
    root, scanner, env = _fixture(tmp_path)
    if bad_input.startswith("missing-"):
        (root / ("Cargo.lock" if bad_input.endswith("cargo") else "uv.lock")).unlink()
    elif bad_input == "malformed-cargo":
        (root / "Cargo.lock").write_text("[broken")
    elif bad_input == "empty-uv":
        (root / "uv.lock").write_text("version = 1\n")
    elif bad_input == "invalid-package":
        (root / "uv.lock").write_text('[[package]]\nname = "fixture"\n')
    elif bad_input == "invalid-policy":
        (root / "deny.toml").write_text("[broken")
    else:
        (root / "deny.toml").write_text('[advisories]\nignore = [{id = "RUSTSEC-2024-0384", reason = "reviewed", version = "*"}]\n')
    result = _run(root, scanner, env)
    assert result.returncode != 0
    assert "Dependency advisory check failed:" in result.stderr
    assert "Traceback" not in result.stderr
    assert not Path(env["SCAN_LOG"]).exists()


def test_unrelated_scanner_finding_still_blocks_with_reviewed_exceptions(tmp_path: Path) -> None:
    root, scanner, env = _fixture(tmp_path)
    env["SCAN_EXIT"] = "1"
    result = _run(root, scanner, env)
    assert result.returncode == 1
    assert Path(env["SCAN_LOG"]).exists()
