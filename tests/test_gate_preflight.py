"""Gate callers see every absent prerequisite before preparation starts."""
from pathlib import Path
import os
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def test_gate_lists_all_missing_tools_before_python_sync(tmp_path: Path) -> None:
    shutil.copy2(ROOT / "Makefile", tmp_path / "Makefile")
    shutil.copytree(ROOT / "tools", tmp_path / "tools")
    shutil.copytree(ROOT / ".github", tmp_path / ".github")
    for name in ("rust-toolchain.toml", "pyproject.toml", "Cargo.toml"):
        if (ROOT / name).exists():
            shutil.copy2(ROOT / name, tmp_path / name)
    binaries = tmp_path / "bin"
    binaries.mkdir()
    for name in ("bash",):
        (binaries / name).symlink_to(shutil.which(name))
    result = subprocess.run(
        [shutil.which("make"), "landing-gate"], cwd=tmp_path,
        env=os.environ | {"PATH": str(binaries), "HOME": str(tmp_path)},
        capture_output=True, text=True,
    )
    output = result.stdout + result.stderr
    assert result.returncode != 0
    for name in ("python3", "cargo", "uv", "mustmatch", "git", "jq", "curl", "docker", "seq", "sleep", "ps"):
        assert f"missing: {name}" in output, output
    assert "uv sync" not in output


def test_preflight_reports_incompatible_versions_with_missing_tools(tmp_path: Path) -> None:
    shutil.copytree(ROOT / "tools", tmp_path / "tools")
    shutil.copytree(ROOT / ".github", tmp_path / ".github")
    shutil.copy2(ROOT / "Cargo.toml", tmp_path / "Cargo.toml")
    binaries = tmp_path / "bin"
    binaries.mkdir()
    for name, version in (("python3", "Python 3.9.0"), ("jq", "jq-1.5")):
        tool = binaries / name
        tool.write_text(f"#!/bin/sh\nprintf '%s\\n' '{version}'\n")
        tool.chmod(0o755)
    result = subprocess.run(
        [shutil.which("bash"), str(tmp_path / "tools/gate-tools")],
        env=os.environ | {"PATH": str(binaries), "HOME": str(tmp_path)},
        capture_output=True, text=True,
    )
    assert result.returncode != 0
    assert "python3 requires min 3.10; got 3.9.0" in result.stderr
    assert "jq requires min 1.6; got 1.5" in result.stderr
    assert "missing: mustmatch" in result.stderr
