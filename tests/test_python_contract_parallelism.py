from pathlib import Path
import subprocess


ROOT = Path(__file__).resolve().parents[1]


def test_python_contract_gate_uses_bounded_file_workers() -> None:
    makefile = (ROOT / "Makefile").read_text()

    assert "PYTEST_WORKERS ?= 4" in makefile
    assert "PYTEST_XDIST_ARGS = -n $(PYTEST_WORKERS) --dist loadfile" in makefile
    assert "pytest tests/ -v $(PYTEST_XDIST_ARGS)" in makefile
    assert "-n auto" not in makefile


def test_ci_reuses_the_canonical_python_contract_gate() -> None:
    workflow = (ROOT / ".github/workflows/ci.yml").read_text()

    assert "run: make test" in workflow
    assert "uv run --no-sync pytest tests/" not in workflow


def test_python_contract_worker_count_has_a_one_worker_override() -> None:
    dry_run = subprocess.run(
        ["make", "-n", "test-contracts", "PYTEST_WORKERS=1"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout

    assert "pytest tests/ -v -n 1 --dist loadfile" in dry_run
