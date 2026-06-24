from __future__ import annotations

import os
import shutil
import subprocess
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPT_PATH = Path("scripts") / "pre-commit-reject-march-artifacts.sh"
ALLOWED_MARCH_PATHS = (
    ".march/code-review-log.md",
)
BAD_MARCH_PATHS = (
    ".march/verify-log.md",
    ".march/blueprint.md",
)


def _git(repo_root: Path, *args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["git", *args],
        cwd=repo_root,
        check=True,
        capture_output=True,
        text=True,
    )


def _write(repo_root: Path, path: str, content: str = "tracked\n") -> None:
    target = repo_root / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content, encoding="utf-8")


def _copy_hook_fixture(tmp_path: Path) -> Path:
    fixture_root = tmp_path / "repo"
    (fixture_root / "scripts").mkdir(parents=True)
    source = REPO_ROOT / SCRIPT_PATH
    assert source.is_file(), f"missing hook helper: {SCRIPT_PATH}"
    shutil.copy2(source, fixture_root / SCRIPT_PATH)
    (fixture_root / ".gitignore").write_text(".march/\n", encoding="utf-8")
    _git(fixture_root, "init")
    _git(fixture_root, "config", "user.email", "tests@example.invalid")
    _git(fixture_root, "config", "user.name", "BioMCP Tests")
    return fixture_root


def _run_hook_script(repo_root: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(repo_root / SCRIPT_PATH)],
        cwd=repo_root,
        capture_output=True,
        text=True,
        check=False,
    )


def test_pre_commit_reject_march_artifacts_script_is_executable() -> None:
    script = REPO_ROOT / SCRIPT_PATH

    assert script.is_file()
    assert os.access(script, os.X_OK)


def test_pre_commit_reject_march_artifacts_passes_with_no_staged_march_paths(
    tmp_path: Path,
) -> None:
    repo_root = _copy_hook_fixture(tmp_path)

    result = _run_hook_script(repo_root)

    assert result.returncode == 0
    assert result.stdout == ""
    assert result.stderr == ""


def test_pre_commit_reject_march_artifacts_allows_allowlisted_paths(
    tmp_path: Path,
) -> None:
    repo_root = _copy_hook_fixture(tmp_path)
    for path in ALLOWED_MARCH_PATHS:
        _write(repo_root, path)
    _git(repo_root, "add", "-f", *ALLOWED_MARCH_PATHS)

    result = _run_hook_script(repo_root)

    assert result.returncode == 0
    assert result.stdout == ""
    assert result.stderr == ""


def test_pre_commit_reject_march_artifacts_rejects_staged_bad_paths(
    tmp_path: Path,
) -> None:
    repo_root = _copy_hook_fixture(tmp_path)
    for path in BAD_MARCH_PATHS:
        _write(repo_root, path)
    _git(repo_root, "add", "-f", *BAD_MARCH_PATHS)

    result = _run_hook_script(repo_root)

    assert result.returncode == 1
    assert result.stdout == ""
    assert "Error: staged non-allowlisted .march artifacts detected:" in result.stderr
    for path in BAD_MARCH_PATHS:
        assert path in result.stderr
    for path in ALLOWED_MARCH_PATHS:
        assert path in result.stderr
    assert "git restore --staged -- <path>" in result.stderr
    assert "git rm --cached -- <path>" in result.stderr


def test_pre_commit_reject_march_artifacts_allows_staged_bad_path_deletion(
    tmp_path: Path,
) -> None:
    repo_root = _copy_hook_fixture(tmp_path)
    _write(repo_root, ".march/verify-log.md")
    _git(repo_root, "add", "-f", ".march/verify-log.md")
    _git(repo_root, "commit", "-m", "Track old March artifact")
    _git(repo_root, "rm", ".march/verify-log.md")

    result = _run_hook_script(repo_root)

    assert result.returncode == 0
    assert result.stdout == ""
    assert result.stderr == ""


def test_pre_commit_reject_march_artifacts_rejects_staged_bad_path_modification(
    tmp_path: Path,
) -> None:
    repo_root = _copy_hook_fixture(tmp_path)
    _write(repo_root, ".march/verify-log.md", "old tracked artifact\n")
    _git(repo_root, "add", "-f", ".march/verify-log.md")
    _git(repo_root, "commit", "-m", "Track old March artifact")
    _write(repo_root, ".march/verify-log.md", "modified artifact\n")
    _git(repo_root, "add", "-f", ".march/verify-log.md")

    result = _run_hook_script(repo_root)

    assert result.returncode == 1
    assert result.stdout == ""
    assert ".march/verify-log.md" in result.stderr


def test_pre_commit_reject_march_artifacts_rejects_rename_into_bad_march_path(
    tmp_path: Path,
) -> None:
    repo_root = _copy_hook_fixture(tmp_path)
    _write(repo_root, "notes.md")
    _git(repo_root, "add", "notes.md")
    _git(repo_root, "commit", "-m", "Track note")
    (repo_root / ".march").mkdir()
    _git(repo_root, "mv", "-f", "notes.md", ".march/blueprint.md")

    result = _run_hook_script(repo_root)

    assert result.returncode == 1
    assert result.stdout == ""
    assert ".march/blueprint.md" in result.stderr
