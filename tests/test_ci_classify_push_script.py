"""Behavioral tests for scripts/ci-classify-push.sh.

The 2026-09-29 second review's blocking finding: the contract suite only asserted
substrings of the script, which would pass with broken logic around
them — and a broken classify flow is exactly what once let a failed
tree look green (a failed commit followed by a markdown-only commit
skipped every Rust job). These tests run the real script against
scratch git repositories.
"""

from __future__ import annotations

import os
import subprocess
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPT = REPO_ROOT / "scripts" / "ci-classify-push.sh"


def _run_script(cwd: Path, base_ref: str | None, sha: str, event: str = "push") -> str:
    env = dict(os.environ)
    env["GITHUB_EVENT_NAME"] = event
    env["GITHUB_REF"] = "refs/heads/tickets/scratch"
    if base_ref is not None:
        env["PUSH_BASE_REF"] = base_ref
    else:
        env.pop("PUSH_BASE_REF", None)
    env["PUSH_AFTER"] = sha
    out = Path(cwd / "github_output")
    env["GITHUB_OUTPUT"] = str(out)
    out.write_text("", encoding="utf-8")
    result = subprocess.run(
        ["bash", str(SCRIPT)],
        cwd=cwd,
        env=env,
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 0, result.stderr
    return out.read_text(encoding="utf-8").strip()


@pytest.fixture()
def scratch_repo(tmp_path: Path) -> Path:
    git_env = {
        **os.environ,
        "GIT_AUTHOR_NAME": "t",
        "GIT_AUTHOR_EMAIL": "t@t",
        "GIT_COMMITTER_NAME": "t",
        "GIT_COMMITTER_EMAIL": "t@t",
    }
    repo = tmp_path / "repo"
    repo.mkdir()
    subprocess.run(["git", "init", "-q", "-b", "main", str(repo)], check=True, env=git_env)
    subprocess.run(["git", "-C", str(repo), "commit", "-qm", "empty", "--allow-empty"], check=True, env=git_env)
    # origin/main points at the trunk tip.
    subprocess.run(["git", "-C", str(repo), "branch", "main2"], check=False, env=git_env)
    subprocess.run(["git", "-C", str(repo), "update-ref", "refs/remotes/origin/main", "HEAD"], check=True, env=git_env)
    return repo


def _commit(repo: Path, files: dict[str, str], message: str) -> str:
    git_env = {
        **os.environ,
        "GIT_AUTHOR_NAME": "t",
        "GIT_AUTHOR_EMAIL": "t@t",
        "GIT_COMMITTER_NAME": "t",
        "GIT_COMMITTER_EMAIL": "t@t",
    }
    for name, body in files.items():
        target = repo / name
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(body, encoding="utf-8")
    subprocess.run(["git", "-C", str(repo), "add", "-A"], check=True, env=git_env)
    subprocess.run(["git", "-C", str(repo), "commit", "-qm", message], check=True, env=git_env)
    return subprocess.run(
        ["git", "-C", str(repo), "rev-parse", "HEAD"],
        capture_output=True,
        text=True,
        check=True,
    ).stdout.strip()


def test_docs_only_push_classifies_true(scratch_repo: Path) -> None:
    _commit(scratch_repo, {"sdlc/tickets/x.md": "x", "notes/plan.md": "y"}, "docs")
    sha = _commit(scratch_repo, {"notes/thing.md": "z"}, "more docs")
    assert _run_script(scratch_repo, "origin/main", sha) == "docs_only=true"


def test_changelog_only_push_runs_full_ci(scratch_repo: Path) -> None:
    """CHANGELOG.md is not docs-only: the changelog refresh test is
    marked needs_binary, so only the canonical lane runs it, and a
    changelog-only push must reach that lane (2026-10-07 review,
    finding 4).
    """
    sha = _commit(scratch_repo, {"CHANGELOG.md": "# Changelog\n\n## Unreleased\n"}, "bullets")
    assert _run_script(scratch_repo, "origin/main", sha) == "docs_only=false"


def test_changelog_alongside_docs_still_runs_full_ci(scratch_repo: Path) -> None:
    sha = _commit(
        scratch_repo,
        {"CHANGELOG.md": "# Changelog\n", "sdlc/tickets/x.md": "x"},
        "bullets and ticket",
    )
    assert _run_script(scratch_repo, "origin/main", sha) == "docs_only=false"


@pytest.mark.parametrize(
    "path",
    [
        "README.md",
        "docs/reference/source-licensing.md",
        "docs/charts/bar.md",
        "docs/user-guide/cli-reference.md",
        "skills/oncology-treatment.md",
        "spec/surface/mcp.md",
        "src/cli/list_reference.md",
    ],
)
def test_each_executable_markdown_path_runs_full_ci_alone(
    tmp_path: Path, path: str
) -> None:
    """Each executable-markdown path gets its own fresh repo (the
    second go-request review found the shared-repo form let README's
    earlier commit mask the other paths: the merge-base diff kept
    README in every later case).
    """
    import os

    git_env = {
        **os.environ,
        "GIT_AUTHOR_NAME": "t",
        "GIT_AUTHOR_EMAIL": "t@t",
        "GIT_COMMITTER_NAME": "t",
        "GIT_COMMITTER_EMAIL": "t@t",
    }
    repo = tmp_path / "repo"
    repo.mkdir()
    subprocess.run(["git", "init", "-q", "-b", "main", str(repo)], check=True, env=git_env)
    subprocess.run(["git", "-C", str(repo), "commit", "-qm", "empty", "--allow-empty"], check=True, env=git_env)
    subprocess.run(["git", "-C", str(repo), "update-ref", "refs/remotes/origin/main", "HEAD"], check=True, env=git_env)
    target = repo / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text("changed", encoding="utf-8")
    subprocess.run(["git", "-C", str(repo), "add", "-A"], check=True, env=git_env)
    subprocess.run(["git", "-C", str(repo), "commit", "-qm", "one path"], check=True, env=git_env)
    sha = subprocess.run(
        ["git", "-C", str(repo), "rev-parse", "HEAD"], capture_output=True, text=True, check=True
    ).stdout.strip()
    assert _run_script(repo, "origin/main", sha) == "docs_only=false", path


def test_source_change_classifies_false_even_with_markdown(scratch_repo: Path) -> None:
    sha = _commit(scratch_repo, {"src/foo.md": "x", "sdlc/x.md": "y"}, "mixed")
    assert _run_script(scratch_repo, "origin/main", sha) == "docs_only=false"


def test_empty_diff_and_missing_base_fail_closed(scratch_repo: Path) -> None:
    sha = subprocess.run(
        ["git", "-C", str(scratch_repo), "rev-parse", "HEAD"],
        capture_output=True, text=True, check=True,
    ).stdout.strip()
    # Empty diff (same commit as the base): full CI.
    assert _run_script(scratch_repo, "origin/main", sha) == "docs_only=false"
    # Missing base ref: full CI.
    assert _run_script(scratch_repo, "origin/absent", sha) == "docs_only=false"


def test_pull_request_and_tag_events_run_everything(scratch_repo: Path) -> None:
    sha = _commit(scratch_repo, {"sdlc/x.md": "docs only"}, "docs")
    assert _run_script(scratch_repo, "origin/main", sha, event="pull_request") == "docs_only=false"
    env_tag = dict(os.environ)
    env_tag["GITHUB_REF"] = "refs/tags/v0.9.1"
    # (the helper always sets refs/heads; run the tag case directly)
    out = Path(scratch_repo / "github_output")
    env_tag.update(
        GITHUB_EVENT_NAME="push",
        GITHUB_REF="refs/tags/v0.9.1",
        PUSH_BASE_REF="origin/main",
        PUSH_AFTER=sha,
        GITHUB_OUTPUT=str(out),
    )
    out.write_text("", encoding="utf-8")
    subprocess.run(["bash", str(SCRIPT)], cwd=scratch_repo, env=env_tag, check=True, capture_output=True)
    assert out.read_text(encoding="utf-8").strip() == "docs_only=false"


def test_a_failed_tip_then_markdown_still_classifies_false(scratch_repo: Path) -> None:
    """The production incident shape: a code commit (whatever its CI
    result) followed by a markdown commit must never skip the Rust
    jobs, because the classify base is the merge-base with main.
    """
    _commit(scratch_repo, {"src/lib.rs": "fn a() {}"}, "code")
    sha = _commit(scratch_repo, {"sdlc/only-markdown.md": "x"}, "docs:")
    assert _run_script(scratch_repo, "origin/main", sha) == "docs_only=false"
