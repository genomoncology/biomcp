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

_GIT_ENV = {
    **{
        k: v
        for k, v in os.environ.items()
        if k.startswith("GIT_") or k in ("PATH", "HOME")
    },
    "GIT_AUTHOR_NAME": "t",
    "GIT_AUTHOR_EMAIL": "t@t",
    "GIT_COMMITTER_NAME": "t",
    "GIT_COMMITTER_EMAIL": "t@t",
}


def _run_script(
    cwd: Path,
    base_ref: str | None,
    sha: str,
    event: str = "push",
    ref: str = "refs/heads/tickets/scratch",
    push_before: str | None = None,
    extra_env: dict[str, str] | None = None,
) -> str:
    env = dict(os.environ)
    env["GITHUB_EVENT_NAME"] = event
    env["GITHUB_REF"] = ref
    if base_ref is not None:
        env["PUSH_BASE_REF"] = base_ref
    else:
        env.pop("PUSH_BASE_REF", None)
    env["PUSH_AFTER"] = sha
    if push_before is not None:
        env["PUSH_BEFORE"] = push_before
    else:
        env.pop("PUSH_BEFORE", None)
    env.update(extra_env or {})
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
    # The script prints which route decided and why; tests read the
    # print through a scratch file next to the fake output.
    (cwd / "script_stdout").write_text(result.stdout, encoding="utf-8")
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


# --- the green-tip route. ---


def _fake_gh_bin(tmp_path: Path) -> Path:
    """A gh on PATH that prints FAKE_GH_RESPONSE, records its argv in
    FAKE_GH_ARGS, and exits FAKE_GH_EXIT. The script asks for the
    previous tip's ci.yml push run through gh api --jq, so the fake
    prints the already-filtered status|conclusion line.
    """
    bin_dir = tmp_path / "fake-gh-bin"
    bin_dir.mkdir(exist_ok=True)
    gh = bin_dir / "gh"
    gh.write_text(
        "#!/usr/bin/env bash\n"
        'printf \'%s\\n\' "${FAKE_GH_RESPONSE-}"\n'
        'printf \'%s\\n\' "$*" >> "${FAKE_GH_ARGS-/dev/null}"\n'
        'exit "${FAKE_GH_EXIT-0}"\n',
        encoding="utf-8",
    )
    gh.chmod(0o755)
    return bin_dir


def _record_push_over_code_tip(scratch_repo: Path) -> tuple[str, str]:
    """The pm landing shape: a code commit (the previous tip), then a
    record commit that touches only sdlc/. The merge-base route says
    full CI (the code changed since main), so only the green-tip
    route can skip. Returns (previous tip, pushed tip).
    """
    code_tip = _commit(scratch_repo, {"src/lib.rs": "fn a() {}"}, "code")
    record_tip = _commit(
        scratch_repo,
        {"sdlc/tickets/2069-record-the-baseline.md": "Status: OPEN.\n"},
        "Record 2069 review: accept",
    )
    return code_tip, record_tip


def _green_tip_env(
    tmp_path: Path, response: str, exit_code: int = 0
) -> tuple[dict[str, str], Path]:
    gh_dir = _fake_gh_bin(tmp_path)
    args_file = tmp_path / "gh_args"
    env = {
        "PATH": f"{gh_dir}:{os.environ['PATH']}",
        "GITHUB_REPOSITORY": "example/biomcp",
        "FAKE_GH_RESPONSE": response,
        "FAKE_GH_EXIT": str(exit_code),
        "FAKE_GH_ARGS": str(args_file),
    }
    return env, args_file


@pytest.mark.parametrize(
    "ref",
    ["refs/heads/ticket/2086-ci-record-skip", "refs/heads/tickets/2086"],
)
def test_sdlc_only_push_over_green_tip_classifies_true(
    scratch_repo: Path, tmp_path: Path, ref: str
) -> None:
    """The route the ticket buys: a record push whose previous tip's
    ci.yml push run concluded success skips the Rust gates. Both
    ticket-branch spellings count.
    """
    code_tip, record_tip = _record_push_over_code_tip(scratch_repo)
    env, args_file = _green_tip_env(tmp_path, "completed|success")
    assert (
        _run_script(scratch_repo, "origin/main", record_tip, ref=ref, push_before=code_tip, extra_env=env)
        == "docs_only=true"
    ), ref
    asked = args_file.read_text(encoding="utf-8")
    assert f"head_sha={code_tip}" in asked, "the run query must target the previous tip"
    assert "event=push" in asked, "only a push run proves the tip"
    stdout = (scratch_repo / "script_stdout").read_text(encoding="utf-8")
    assert "green-tip" in stdout, "the log must name the deciding route"
    assert "concluded success" in stdout, "the log must say why it skipped"


@pytest.mark.parametrize(
    ("response", "exit_code", "why"),
    [
        ("completed|failure", 0, "the previous tip's run failed"),
        ("completed|cancelled", 0, "the previous tip's run was cancelled"),
        ("none", 0, "the previous tip has no ci.yml push run"),
        ("in_progress|null", 0, "the previous tip's run is still in progress"),
        ("queued|null", 0, "the previous tip's run is queued"),
        ("", 1, "the gh api call errored"),
    ],
)
def test_anything_but_a_green_tip_runs_full_ci(
    scratch_repo: Path, tmp_path: Path, response: str, exit_code: int, why: str
) -> None:
    code_tip, record_tip = _record_push_over_code_tip(scratch_repo)
    env, _ = _green_tip_env(tmp_path, response, exit_code)
    assert (
        _run_script(
            scratch_repo, "origin/main", record_tip,
            ref="refs/heads/ticket/2086-ci-record-skip",
            push_before=code_tip, extra_env=env,
        )
        == "docs_only=false"
    ), why


def test_second_route_counts_only_sdlc_paths(scratch_repo: Path, tmp_path: Path) -> None:
    """notes/ is in the merge-base allow list but not here: the
    previous tip's green run proved a tree without the notes change,
    and only sdlc/ is what pm writes onto a branch.
    """
    code_tip = _commit(scratch_repo, {"src/lib.rs": "fn a() {}"}, "code")
    tip = _commit(
        scratch_repo,
        {"sdlc/tickets/x.md": "x", "notes/plan.md": "y"},
        "record plus notes",
    )
    env, _ = _green_tip_env(tmp_path, "completed|success")
    assert (
        _run_script(
            scratch_repo, "origin/main", tip,
            ref="refs/heads/ticket/2086-ci-record-skip",
            push_before=code_tip, extra_env=env,
        )
        == "docs_only=false"
    )
    stdout = (scratch_repo / "script_stdout").read_text(encoding="utf-8")
    assert "outside sdlc/" in stdout


def test_force_push_before_not_ancestor_runs_full_ci(
    scratch_repo: Path, tmp_path: Path
) -> None:
    """A rewritten history: the recorded before is a real commit but
    not an ancestor of the pushed tip, so the diff between them is
    not the push.
    """
    code_tip, record_tip = _record_push_over_code_tip(scratch_repo)
    subprocess.run(
        ["git", "-C", str(scratch_repo), "checkout", "-q", "-b", "side", "main"],
        check=True, env=_GIT_ENV,
    )
    diverged = _commit(scratch_repo, {"side.txt": "diverged"}, "diverged")
    subprocess.run(
        ["git", "-C", str(scratch_repo), "checkout", "-q", "main"],
        check=True, env=_GIT_ENV,
    )
    env, _ = _green_tip_env(tmp_path, "completed|success")
    assert (
        _run_script(
            scratch_repo, "origin/main", record_tip,
            ref="refs/heads/ticket/2086-ci-record-skip",
            push_before=diverged, extra_env=env,
        )
        == "docs_only=false"
    )
    stdout = (scratch_repo / "script_stdout").read_text(encoding="utf-8")
    assert "not an ancestor" in stdout


@pytest.mark.parametrize(
    "ref",
    ["refs/heads/main", "refs/heads/feature/x"],
)
def test_non_ticket_branch_never_uses_the_green_tip_route(
    scratch_repo: Path, tmp_path: Path, ref: str
) -> None:
    code_tip, record_tip = _record_push_over_code_tip(scratch_repo)
    env, args_file = _green_tip_env(tmp_path, "completed|success")
    assert (
        _run_script(
            scratch_repo, "origin/main", record_tip,
            ref=ref, push_before=code_tip, extra_env=env,
        )
        == "docs_only=false"
    ), ref
    assert not args_file.exists(), "the route must not even ask for a run off a ticket branch"


def test_all_zero_before_runs_full_ci(scratch_repo: Path, tmp_path: Path) -> None:
    """A new branch's first push reports all zeros as its before."""
    _, record_tip = _record_push_over_code_tip(scratch_repo)
    env, args_file = _green_tip_env(tmp_path, "completed|success")
    assert (
        _run_script(
            scratch_repo, "origin/main", record_tip,
            ref="refs/heads/ticket/2086-ci-record-skip",
            push_before="0" * 40, extra_env=env,
        )
        == "docs_only=false"
    )
    assert not args_file.exists(), "an all-zeros before must not be queried"


def test_missing_before_env_runs_full_ci(scratch_repo: Path, tmp_path: Path) -> None:
    """No PUSH_BEFORE at all (the old env): the route cannot run and
    nothing may fail.
    """
    _, record_tip = _record_push_over_code_tip(scratch_repo)
    env, _ = _green_tip_env(tmp_path, "completed|success")
    assert (
        _run_script(
            scratch_repo, "origin/main", record_tip,
            ref="refs/heads/ticket/2086-ci-record-skip",
            extra_env=env,
        )
        == "docs_only=false"
    )


def test_agents_md_only_push_runs_full_ci(scratch_repo: Path) -> None:
    """AGENTS.md is executable markdown: spec/surface/mcp.md's
    "Root Agent Guide Declares The Contract" block reads it, so an
    AGENTS.md-only push must run the spec pages. Commit 393737eae
    classified docs-only and turned main red until ec4029345
    (amendment 2026-10-14).
    """
    sha = _commit(scratch_repo, {"AGENTS.md": "# BioMCP agent guide\n"}, "trim the guide")
    assert _run_script(scratch_repo, "origin/main", sha) == "docs_only=false"
