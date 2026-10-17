from pathlib import Path
import subprocess
import shlex
import sys


ROOT = Path(__file__).resolve().parents[1]


def test_python_contract_gate_uses_bounded_file_workers() -> None:
    makefile = (ROOT / "Makefile").read_text()

    assert "PYTEST_WORKERS ?= 4" in makefile
    assert "PYTEST_XDIST_ARGS = -n $(PYTEST_WORKERS) --dist loadfile" in makefile
    assert "pytest tests/ -v $(PYTEST_XDIST_ARGS)" in makefile
    assert "-n auto" not in makefile


def test_ci_reuses_the_canonical_python_contract_gate() -> None:
    workflow = (ROOT / ".github/workflows/ci.yml").read_text()

    # Each lane runs its whole gate through the Makefile target
    # (the lane split cut `make test` into `test-rust` and
    # `test-python` for exactly this); CI never copies a smaller
    # command list. repository-contracts runs the binary-less subset
    # for docs-only pushes (2026-09-29 review); a bare full-suite
    # pytest outside the Make targets would lose the spec-binary
    # environment.
    for gate in ("make lint", "make test-rust", "make test-python", "make spec"):
        assert f"run: {gate}" in workflow
    assert 'uv run --no-sync pytest tests/ -v' not in workflow


def test_python_contract_worker_count_has_a_one_worker_override() -> None:
    dry_run = subprocess.run(
        ["make", "-n", "test-contracts", "PYTEST_WORKERS=1"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout

    assert "pytest tests/ -v -n 1 --dist loadfile" in dry_run


def test_contract_gate_keeps_temporary_paths_in_the_worktree() -> None:
    # The suite's temp-path contract asserts that TMPDIR and pytest's
    # basetemp live inside the worktree; the offline sandbox points
    # TMPDIR at its own /tmp, so the gate itself must carry the
    # routine lane's redirection into the worktree (2148).
    dry_run = subprocess.run(
        ["make", "-n", "test-contracts", "PYTEST_WORKERS=1"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout

    pytest_line = next(
        line for line in dry_run.splitlines() if "pytest tests/" in line
    )
    assert 'TMPDIR="' in pytest_line
    assert ".cache/routine-test-tmp" in pytest_line
    assert '--basetemp "' in pytest_line
    assert ".cache/routine-test-tmp/pytest" in pytest_line


def test_routine_python_selection_excludes_audits_long_and_release_cases(
    tmp_path: Path,
) -> None:
    dry_run = subprocess.run(
        ["make", "-n", "test-contracts-prepared"],
        cwd=ROOT, check=True, capture_output=True, text=True,
    ).stdout
    command = shlex.split(next(
        line for line in dry_run.splitlines() if "pytest tests/" in line
    ))
    selection = command[command.index("-m") + 1]
    (tmp_path / "pytest.ini").write_text(
        "[pytest]\nmarkers =\n    repo_audit\n    long\n    release_gate\n"
    )
    (tmp_path / "test_selection.py").write_text(
        "import pytest\n"
        "def test_routine(): pass\n"
        "@pytest.mark.repo_audit\ndef test_audit(): assert False\n"
        "@pytest.mark.long\ndef test_long(): assert False\n"
        "@pytest.mark.release_gate\ndef test_release(): assert False\n"
    )
    result = subprocess.run(
        [sys.executable, "-m", "pytest", "-q", "-m", selection, str(tmp_path)],
        cwd=tmp_path, capture_output=True, text=True, timeout=15,
    )
    assert result.returncode == 0, result.stdout + result.stderr
    assert "1 passed, 3 deselected" in result.stdout
