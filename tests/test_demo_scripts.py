from __future__ import annotations

import os
from pathlib import Path
import subprocess

import pytest

ROOT = Path(__file__).resolve().parents[1]


@pytest.mark.parametrize("script", ["geneagent-demo.sh", "genegpt-demo.sh"])
def test_demo_reports_failed_command_streams(tmp_path: Path, script: str) -> None:
    binary = tmp_path / "biomcp"
    binary.write_text(
        "#!/bin/sh\nprintf '%s\\n' '{\"error\":\"command stdout failure\"}'\n"
        "echo 'command stderr failure' >&2\nexit 7\n"
    )
    binary.chmod(0o755)
    result = subprocess.run(
        ["bash", str(ROOT / "scripts" / script)],
        env=dict(os.environ, BIOMCP_BIN=str(binary)),
        capture_output=True, text=True, timeout=10,
    )
    assert result.returncode == 7, result.stdout + result.stderr
    assert "command stdout failure" in result.stdout + result.stderr
    assert "command stderr failure" in result.stdout + result.stderr
