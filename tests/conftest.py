from __future__ import annotations

import subprocess
import tempfile
from collections.abc import Iterator
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]


def pytest_addoption(parser: pytest.Parser) -> None:
    group = parser.getgroup("mcp")
    group.addoption(
        "--mcp-cmd",
        action="store",
        default=None,
        help="Deprecated compatibility option; MCP client contracts now run in Rust.",
    )
    group.addoption(
        "--mcp-timeout",
        action="store",
        type=float,
        default=20.0,
        help="Deprecated compatibility option; MCP client contracts now run in Rust.",
    )


def _provision_study_fixture(root: Path) -> str:
    script = REPO_ROOT / "spec" / "fixtures" / "setup-study-spec-fixture.sh"
    subprocess.run(["bash", str(script), str(root)], cwd=REPO_ROOT, check=True)
    result = subprocess.run(
        [
            "bash",
            "-lc",
            "source .cache/spec-study-env && printf '%s' \"$BIOMCP_STUDY_DIR\"",
        ],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    )
    study_dir = result.stdout.strip()
    if not study_dir:
        raise RuntimeError("study fixture did not set BIOMCP_STUDY_DIR")
    return study_dir


@pytest.fixture
def study_fixture_env() -> Iterator[dict[str, str]]:
    with tempfile.TemporaryDirectory(prefix="biomcp-study-tests-") as root_name:
        root = Path(root_name)
        yield {"BIOMCP_STUDY_DIR": _provision_study_fixture(root)}
