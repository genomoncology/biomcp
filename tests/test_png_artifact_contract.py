from __future__ import annotations

from pathlib import Path
import tomllib


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
