"""Contract for tools/with-build-identity (ticket 2024 reopening).

The wrapper handed off with os.execvpe, which on Windows exits 0 at
once while the child keeps running, so a release build step passed
with no binary. These tests pin the wait-and-propagate hand-off and
the env export mode the manylinux legs pipe into docker --env-file.
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
WRAPPER = REPO_ROOT / "tools" / "with-build-identity"


def test_the_wrapper_source_runs_the_child_to_completion() -> None:
    source = WRAPPER.read_text(encoding="utf-8")
    assert "execvpe(" not in source, (
        "an exec hand-off replaces this process, which on Windows "
        "exits 0 at once while the child keeps running; the build step "
        "then passes with no binary (ticket 2024)"
    )
    assert "check=False" in source, (
        "the child's exit code decides the wrapper's exit, not an exception"
    )
    assert "env=identity_environment(root)" in source, (
        "the child must run under the identity environment"
    )
    assert "SystemExit(completed.returncode)" in source, (
        "the wrapper must exit with the child's exit code"
    )


def test_the_wrapper_passes_the_child_exit_code_through(tmp_path: Path) -> None:
    # 42 fits in the one byte POSIX exit codes carry, so the same
    # expectation holds on every platform.
    result = subprocess.run(
        [
            sys.executable,
            str(WRAPPER),
            sys.executable,
            "-c",
            "raise SystemExit(42)",
        ],
        cwd=tmp_path,
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 42


def test_the_wrapper_env_export_mode_still_exports_the_identity(
    tmp_path: Path,
) -> None:
    """The manylinux legs run `tools/with-build-identity env`, grep the
    BIOMCP_BUILD_ lines into a file, and hand it to docker through
    --env-file, so the export mode must still run its child under the
    identity environment."""
    root = tmp_path / "fixture"
    root.mkdir()
    (root / "Cargo.toml").write_text(
        '[package]\nname = "identity-fixture"\nversion = "0.1.0"\n', encoding="utf-8"
    )
    for args in (
        ["git", "init", "-q"],
        ["git", "config", "user.email", "identity@example.invalid"],
        ["git", "config", "user.name", "Identity Test"],
        ["git", "add", "."],
        ["git", "commit", "-qm", "initial"],
    ):
        subprocess.run(args, cwd=root, check=True, capture_output=True)
    result = subprocess.run(
        [sys.executable, str(WRAPPER), "env"],
        cwd=root,
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 0
    exported = {
        line.split("=", 1)[0] for line in result.stdout.splitlines() if "=" in line
    }
    assert {
        "BIOMCP_BUILD_GIT_SHA",
        "BIOMCP_BUILD_VERSION",
        "BIOMCP_BUILD_DATE",
    } <= exported
