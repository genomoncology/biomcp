"""Exercise advisory selection through lint and the real Make gate recipes."""
from __future__ import annotations

import os
from pathlib import Path
import shutil
import subprocess

import pytest

from test_lint import REPO_ROOT, _advisory_fixture, _run_lint, _write_executable


def _git(root: Path, *args: str) -> str:
    result = subprocess.run(
        ["git", "-c", "user.name=Fixture", "-c", "user.email=fixture@example.org", *args],
        cwd=root, check=True, capture_output=True, text=True,
    )
    return result.stdout.strip()


@pytest.mark.parametrize("context", ["missing", "equal", "ahead", "divergent", "diff-error", "untracked-input"])
def test_landing_runs_advisories_when_comparison_is_unreliable(tmp_path: Path, context: str) -> None:
    root, env, log = _advisory_fixture(tmp_path)
    if context == "missing":
        _git(root, "update-ref", "-d", "refs/remotes/origin/main")
    elif context == "equal":
        _git(root, "update-ref", "refs/remotes/origin/main", "HEAD")
    elif context == "ahead":
        _git(root, "update-ref", "refs/remotes/origin/main", "HEAD")
        _git(root, "checkout", "--detach", "HEAD^")
    elif context == "divergent":
        candidate = _git(root, "rev-parse", "HEAD")
        _git(root, "checkout", "--detach", "HEAD^")
        _git(root, "commit", "--allow-empty", "-qm", "Other line")
        _git(root, "update-ref", "refs/remotes/origin/main", "HEAD")
        _git(root, "checkout", "--detach", candidate)
    elif context == "diff-error":
        real_git = shutil.which("git")
        assert real_git
        _write_executable(tmp_path / "tools/git", '#!/usr/bin/env bash\nif [ "$1" = diff ]; then exit 128; fi\nexec "$REAL_GIT" "$@"\n')
        env["REAL_GIT"] = real_git
    else:
        _git(root, "rm", "--cached", "uv.lock")
    result = _run_lint(root, env=env)
    assert result.returncode == 0, result.stdout + result.stderr
    assert log.read_text() == "check licenses\ncheck advisories\n"


@pytest.mark.parametrize("state", ["staged", "committed", "deleted"])
def test_landing_runs_advisories_for_every_tracked_change_state(tmp_path: Path, state: str) -> None:
    root, env, log = _advisory_fixture(tmp_path)
    if state == "deleted":
        (root / "Cargo.lock").unlink()
    else:
        (root / "Cargo.lock").write_text("changed\n")
        _git(root, "add", "Cargo.lock")
        if state == "committed":
            _git(root, "commit", "-qm", "Change dependencies")
    result = _run_lint(root, env=env)
    assert result.returncode == 0, result.stdout + result.stderr
    assert log.read_text() == "check licenses\ncheck advisories\n"


def _make_fixture(root: Path, env: dict[str, str]) -> None:
    # Retain production gate orchestration and lint. Replace unrelated heavy
    # boundaries with recorders so these controls compile no native code.
    makefile = (REPO_ROOT / "Makefile").read_text()
    makefile += '\n' + '\n'.join(
        f'{target}:\n\t@echo {target} >> "$$GATE_LOG"'
        for target in ("test", "test-long", "repo-audit", "full-feature-check", "spec", "installed-wheel-smoke")
    ) + '\n'
    (root / "Makefile").write_text(makefile)
    _write_executable(root / "tools/bootstrap-lint-tools", '#!/usr/bin/env bash\nprintf "%s\\n" "$LINT_TOOL_DIR"\n')
    _write_executable(root / "tools/check-quality-ratchet.sh", '#!/usr/bin/env bash\necho ratchet >> "$GATE_LOG"\n')
    _write_executable(Path(env["PATH"].split(":")[0]) / "uv", '#!/usr/bin/env bash\necho "uv $*" >> "$GATE_LOG"\n')
    _write_executable(root / "tools/run-offline", '#!/usr/bin/env bash\nshift\nexec "$@"\n')
    env["LINT_TOOL_DIR"] = env["PATH"].split(":")[0]
    env["GATE_LOG"] = str(root.parent / "gates.log")


@pytest.mark.parametrize("target,scope,dependency_changed,required", [
    ("lint", "", False, True), ("landing-gate", "", False, False),
    ("landing-gate", "", True, True),
    ("main-gate", "landing", False, True), ("release-gate", "landing", False, True),
])
@pytest.mark.parametrize("advisory_fails", [False, True])
def test_gate_entry_points_preserve_advisory_policy_and_failures(
    tmp_path: Path, target: str, scope: str, dependency_changed: bool,
    required: bool, advisory_fails: bool,
) -> None:
    root, env, log = _advisory_fixture(tmp_path)
    _make_fixture(root, env)
    if dependency_changed:
        (root / "Cargo.lock").write_text("changed\n")
    if advisory_fails:
        with (tmp_path / "tools/cargo-deny").open("a") as output:
            output.write('if [ "$*" = "check advisories" ]; then exit 1; fi\n')
    env["BIOMCP_ADVISORY_SCOPE"] = scope
    env = {key: value for key, value in os.environ.items() if key not in ("MAKEFLAGS", "MAKEOVERRIDES", "MAKELEVEL", "MAKEFILES")} | env
    result = subprocess.run(
        ["make", target, f"BIOMCP_ADVISORY_SCOPE={scope}"], cwd=root,
        env=env, text=True, capture_output=True, check=False,
    )
    assert (result.returncode != 0) == (required and advisory_fails), result.stdout + result.stderr
    assert log.read_text() == "check licenses\n" + ("check advisories\n" if required else "")
    assert "[PASS] Rust lint (cargo fmt --check)" in result.stdout
    assert "[PASS] Rust lint (cargo clippy --all-targets -- -D warnings)" in result.stdout
    assert "[PASS] Rust license lint" in result.stdout
    if not (required and advisory_fails):
        gates = Path(env["GATE_LOG"]).read_text()
        assert "ratchet\n" in gates
        if target != "lint":
            assert "test\n" in gates and "spec\n" in gates
