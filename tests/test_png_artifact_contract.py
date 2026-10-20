from __future__ import annotations

from pathlib import Path
import shutil
import subprocess
import tomllib
import pytest

pytestmark = [pytest.mark.needs_binary]  # docs-only CI excludes this module


ROOT = Path(__file__).resolve().parents[1]


def test_png_is_in_the_public_default_feature_set() -> None:
    cargo = tomllib.loads((ROOT / "Cargo.toml").read_text())
    assert "charts-png" in cargo["features"]["default"]


def test_public_build_commands_do_not_disable_default_features() -> None:
    for relative in (
        "Dockerfile",
        "pyproject.toml",
        "scripts/release-smoke.sh",
        ".github/workflows/ci.yml",
    ):
        source = (ROOT / relative).read_text()
        public_lines = [line for line in source.splitlines() if "build --release" in line or "maturin" in line]
        assert all("--no-default-features" not in line for line in public_lines), relative


def test_full_feature_gate_runs_the_local_artifact_smoke() -> None:
    makefile = (ROOT / "Makefile").read_text()
    smoke = (ROOT / "tools/check-png-artifact").read_text()
    assert "tools/check-png-artifact target/release/biomcp" in makefile
    assert "default.png" in smoke and "scaled.png" in smoke
    assert 'signature = b"\\x89PNG\\r\\n\\x1a\\n"' in smoke


def test_png_smoke_reports_failed_command_output(tmp_path: Path) -> None:
    (tmp_path / "tools").mkdir()
    shutil.copy2(ROOT / "tools/check-png-artifact", tmp_path / "tools/check-png-artifact")
    fixture = tmp_path / "spec/fixtures/setup-study-spec-fixture.sh"
    fixture.parent.mkdir(parents=True)
    fixture.write_text('mkdir -p "$1/.cache"\necho "BIOMCP_STUDY_DIR=$1/studies" > "$1/.cache/spec-study-env"\n')
    binary = tmp_path / "biomcp"
    binary.write_text(
        "#!/bin/sh\necho '{\"error\":\"chart stdout failure\"}'\n"
        "echo 'chart stderr failure' >&2\nexit 7\n"
    )
    binary.chmod(0o755)
    subprocess.run(["git", "init", "-q", str(tmp_path)], check=True)
    result = subprocess.run(
        ["bash", "tools/check-png-artifact", str(binary)], cwd=tmp_path,
        capture_output=True, text=True, timeout=10,
    )
    assert result.returncode == 7, result.stdout + result.stderr
    assert "chart stdout failure" in result.stdout + result.stderr
    assert "chart stderr failure" in result.stdout + result.stderr
