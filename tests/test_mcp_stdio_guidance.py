from __future__ import annotations

import os
import subprocess
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]
RELEASE_BIN = REPO_ROOT / "target" / "release" / "biomcp"


@pytest.mark.parametrize("command", ["mcp", "serve"])
def test_stdio_no_input_prints_recovery_guidance(command: str) -> None:
    assert RELEASE_BIN.exists(), f"missing release binary: {RELEASE_BIN}"
    env = dict(os.environ)
    env.pop("RUST_LOG", None)

    result = subprocess.run(
        [str(RELEASE_BIN), command],
        cwd=REPO_ROOT,
        stdin=subprocess.DEVNULL,
        capture_output=True,
        text=True,
        check=False,
        env=env,
    )

    assert result.returncode != 0
    assert result.stdout == ""
    assert "expects an MCP client on stdin" in result.stderr
    assert "biomcp serve-http" in result.stderr
    assert "connection closed: initialized request" not in result.stderr
