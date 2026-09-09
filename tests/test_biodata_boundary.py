from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CHECKER = ROOT / "tools/check-biodata-boundary.py"


def test_biodata_boundary_accepts_the_dedicated_1_0_line() -> None:
    subprocess.run([sys.executable, CHECKER, "--root", ROOT], check=True)


def test_biodata_boundary_rejects_a_path_dependency(tmp_path: Path) -> None:
    subprocess.run(["git", "init", "-q"], cwd=tmp_path, check=True)
    manifest = (
        (ROOT / "Cargo.toml")
        .read_text(encoding="utf-8")
        .replace(
            'biodata = { git = "https://github.com/genomoncology/biodata", rev = "cfafc69d27c9a2fc74909f21692a418a8b17db83" }',
            'biodata = { path = "../biodata" }',
        )
    )
    (tmp_path / "Cargo.toml").write_text(manifest, encoding="utf-8")
    (tmp_path / "Cargo.lock").write_text(
        (ROOT / "Cargo.lock").read_text(encoding="utf-8"), encoding="utf-8"
    )
    subprocess.run(["git", "add", "Cargo.toml", "Cargo.lock"], cwd=tmp_path, check=True)
    result = subprocess.run(
        [sys.executable, CHECKER, "--root", tmp_path],
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 1
    assert "exact Git-pinned BioData dependency" in result.stdout
