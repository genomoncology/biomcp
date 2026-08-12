from __future__ import annotations

import os
from pathlib import Path
import subprocess

import pytest


ROOT = Path(__file__).resolve().parents[1]


@pytest.mark.skipif(os.name == "nt", reason="Unix socket contract")
def test_socket_case_ignores_a_long_ambient_tmpdir(tmp_path: Path) -> None:
    long_tmp = tmp_path / ("ambient-" + "x" * 90) / ("nested-" + "y" * 90)
    long_tmp.mkdir(parents=True)
    result = subprocess.run(
        [
            "cargo",
            "test",
            "--locked",
            "--no-default-features",
            "--lib",
            "cache::clear::tests::clear_rejects_special_file_before_mutation",
        ],
        cwd=ROOT,
        env=os.environ | {"TMPDIR": str(long_tmp)},
        text=True,
        capture_output=True,
    )
    assert result.returncode == 0, result.stdout + result.stderr


def test_socket_fixture_uses_unique_short_owned_root() -> None:
    source = (ROOT / "src/cache/clear.rs").read_text()
    assert '.prefix("biomcp-sock-")' in source
    assert '.tempdir_in("/tmp")' in source
    assert "socket_path.as_os_str().len() < 100" in source
