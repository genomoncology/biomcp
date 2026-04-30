from __future__ import annotations

import os
import subprocess
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
WRAPPER_SCRIPT = REPO_ROOT / "tools" / "biomcp-ci"
ARCHITECTURE_CONTRACT = REPO_ROOT / "architecture" / "technical" / "spec-v2.md"
STALE_SENTINEL = "STALE_PATH_BIOMCP_EXECUTED"


def _write_executable(path: Path, contents: str) -> None:
    path.write_text(contents, encoding="utf-8")
    path.chmod(0o755)


def _section(path: Path, heading: str) -> str:
    content = path.read_text(encoding="utf-8")
    start = content.index(heading)
    next_heading = content.find("\n## ", start + len(heading))
    if next_heading == -1:
        return content[start:]
    return content[start:next_heading]


def test_biomcp_ci_fails_closed_before_stale_path_biomcp_executes(
    tmp_path: Path,
) -> None:
    stale_dir = tmp_path / "path-bin"
    stale_dir.mkdir()
    stale_bin = stale_dir / "biomcp"
    _write_executable(
        stale_bin,
        "#!/usr/bin/env sh\n"
        f"printf '{STALE_SENTINEL} %s\\n' \"$*\"\n",
    )

    wrapper_env = os.environ.copy()
    wrapper_env.pop("BIOMCP_BIN", None)
    wrapper_env["PATH"] = f"{stale_dir}:{wrapper_env['PATH']}"

    result = subprocess.run(
        ["bash", str(WRAPPER_SCRIPT), "version"],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        check=False,
        env=wrapper_env,
    )

    assert result.returncode != 0, result.stdout
    assert STALE_SENTINEL not in result.stdout
    assert STALE_SENTINEL not in result.stderr
    assert str(stale_bin) in result.stderr
    assert "BIOMCP_BIN" in result.stderr

    wrapper_contract = _section(
        ARCHITECTURE_CONTRACT,
        "## `tools/biomcp-ci` wrapper contract",
    )
    assert "fail closed" in wrapper_contract.lower()
    assert "BIOMCP_BIN" in wrapper_contract
    assert "otherwise resolve `biomcp` from\n  `PATH`" not in wrapper_contract
