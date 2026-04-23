from __future__ import annotations

import os
import shutil
import subprocess
import tomllib
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]
LINT_SCRIPT = REPO_ROOT / "bin" / "lint"
PYPROJECT_FILE = REPO_ROOT / "pyproject.toml"
RUFF_GATE_PROBE = (
    REPO_ROOT
    / "architecture"
    / "experiments"
    / "_ruff_gate_probe"
    / "unused_import_probe.py"
)


def _copy_lint_fixture(tmp_path: Path) -> Path:
    fixture_root = tmp_path / "repo"
    (fixture_root / "bin").mkdir(parents=True)
    (fixture_root / "docs").mkdir()
    shutil.copy2(LINT_SCRIPT, fixture_root / "bin" / "lint")
    subprocess.run(["git", "init"], cwd=fixture_root, check=True, capture_output=True)
    return fixture_root


def _track_files(repo_root: Path, files: dict[str, str]) -> None:
    for relative_path, contents in files.items():
        path = repo_root / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(contents, encoding="utf-8")
    subprocess.run(["git", "add", "."], cwd=repo_root, check=True)


def _write_executable(path: Path, contents: str) -> None:
    path.write_text(contents, encoding="utf-8")
    path.chmod(0o755)


def _run_lint(
    repo_root: Path, *, env: dict[str, str] | None = None
) -> subprocess.CompletedProcess[str]:
    full_env = None if env is None else {"PATH": "", **env}
    return subprocess.run(
        ["bash", "bin/lint"],
        cwd=repo_root,
        capture_output=True,
        text=True,
        check=False,
        env=full_env,
    )


def _require_ruff() -> str:
    ruff = shutil.which("ruff")
    if ruff is None:
        pytest.skip("ruff not installed")
    return ruff


@pytest.mark.parametrize("relative_path", ["README.md", "docs/install.md"])
def test_lint_rejects_deprecated_public_doc_tokens(
    tmp_path: Path, relative_path: str
) -> None:
    repo_root = _copy_lint_fixture(tmp_path)
    _track_files(repo_root, {relative_path: "pip install biomcp-python\n"})

    result = _run_lint(repo_root)

    assert result.returncode == 1
    assert f"{relative_path}:1:pip install biomcp-python" in result.stdout
    assert "[FAIL] deprecated public-doc install string scan" in result.stdout


def test_lint_ignores_historical_biomcp_python_mentions_in_changelog(
    tmp_path: Path,
) -> None:
    repo_root = _copy_lint_fixture(tmp_path)
    _track_files(repo_root, {"CHANGELOG.md": "Historical note: biomcp-python hotfix.\n"})

    result = _run_lint(repo_root)

    assert result.returncode == 0
    assert "[PASS] deprecated public-doc install string scan" in result.stdout
    assert "CHANGELOG.md" not in result.stdout


def test_lint_reports_each_offending_public_doc_line_once(tmp_path: Path) -> None:
    repo_root = _copy_lint_fixture(tmp_path)
    _track_files(repo_root, {"README.md": "pip install biomcp-python\n"})

    result = _run_lint(repo_root)

    offending_line = "README.md:1:pip install biomcp-python"
    assert result.returncode == 1
    assert result.stdout.count(offending_line) == 1


def test_lint_requires_cargo_deny_for_rust_repos(tmp_path: Path) -> None:
    repo_root = _copy_lint_fixture(tmp_path)
    _track_files(
        repo_root,
        {
            "Cargo.toml": '[package]\nname = "fixture"\nversion = "0.1.0"\nedition = "2024"\n'
        },
    )
    tool_dir = tmp_path / "tools"
    tool_dir.mkdir()
    _write_executable(tool_dir / "cargo", "#!/usr/bin/env bash\nexit 0\n")

    result = _run_lint(
        repo_root,
        env={"PATH": f"{tool_dir}:/usr/bin:/bin"},
    )

    assert result.returncode == 1
    assert "cargo install cargo-deny --locked" in result.stdout
    assert "[FAIL] Rust license lint (cargo-deny missing)" in result.stdout


def test_lint_runs_cargo_deny_license_and_advisory_checks_when_present(
    tmp_path: Path,
) -> None:
    repo_root = _copy_lint_fixture(tmp_path)
    _track_files(
        repo_root,
        {
            "Cargo.toml": '[package]\nname = "fixture"\nversion = "0.1.0"\nedition = "2024"\n'
        },
    )
    tool_dir = tmp_path / "tools"
    tool_dir.mkdir()
    log_file = tmp_path / "cargo-deny.log"
    _write_executable(
        tool_dir / "cargo",
        "#!/usr/bin/env bash\nif [ \"$1\" = \"deny\" ]; then\n  shift\n  exec cargo-deny \"$@\"\nfi\nexit 0\n",
    )
    _write_executable(
        tool_dir / "cargo-deny",
        "#!/usr/bin/env bash\nprintf '%s\\n' \"$*\" >> \"$CARGO_DENY_LOG\"\n",
    )

    result = _run_lint(
        repo_root,
        env={
            "CARGO_DENY_LOG": str(log_file),
            "PATH": f"{tool_dir}:/usr/bin:/bin",
        },
    )

    assert result.returncode == 0
    assert log_file.read_text(encoding="utf-8") == "check licenses\ncheck advisories\n"
    assert "[PASS] Rust license lint (cargo deny check licenses)" in result.stdout
    assert "[PASS] Rust advisory lint (cargo deny check advisories)" in result.stdout


def test_lint_fails_when_cargo_deny_advisory_check_fails(tmp_path: Path) -> None:
    repo_root = _copy_lint_fixture(tmp_path)
    _track_files(
        repo_root,
        {
            "Cargo.toml": '[package]\nname = "fixture"\nversion = "0.1.0"\nedition = "2024"\n'
        },
    )
    tool_dir = tmp_path / "tools"
    tool_dir.mkdir()
    log_file = tmp_path / "cargo-deny.log"
    _write_executable(
        tool_dir / "cargo",
        "#!/usr/bin/env bash\nif [ \"$1\" = \"deny\" ]; then\n  shift\n  exec cargo-deny \"$@\"\nfi\nexit 0\n",
    )
    _write_executable(
        tool_dir / "cargo-deny",
        "#!/usr/bin/env bash\nprintf '%s\\n' \"$*\" >> \"$CARGO_DENY_LOG\"\nif [ \"$*\" = \"check advisories\" ]; then\n  exit 1\nfi\n",
    )

    result = _run_lint(
        repo_root,
        env={
            "CARGO_DENY_LOG": str(log_file),
            "PATH": f"{tool_dir}:/usr/bin:/bin",
        },
    )

    assert result.returncode == 1
    assert log_file.read_text(encoding="utf-8") == "check licenses\ncheck advisories\n"
    assert "[PASS] Rust license lint (cargo deny check licenses)" in result.stdout
    assert "[FAIL] Rust advisory lint (cargo deny check advisories)" in result.stdout


def test_repo_ruff_excludes_architecture_experiments_probe() -> None:
    pyproject = tomllib.loads(PYPROJECT_FILE.read_text(encoding="utf-8"))
    probe = RUFF_GATE_PROBE.read_text(encoding="utf-8")

    assert "architecture/experiments/**" in pyproject["tool"]["ruff"]["extend-exclude"]
    assert "import os" in probe


def test_lint_ignores_experiment_probe_with_repo_ruff_config(tmp_path: Path) -> None:
    _require_ruff()
    repo_root = _copy_lint_fixture(tmp_path)
    _track_files(
        repo_root,
        {
            "pyproject.toml": PYPROJECT_FILE.read_text(encoding="utf-8"),
            "architecture/experiments/_ruff_gate_probe/unused_import_probe.py": (
                RUFF_GATE_PROBE.read_text(encoding="utf-8")
            ),
        },
    )

    result = _run_lint(repo_root, env={"PATH": os.environ.get("PATH", "")})

    assert result.returncode == 0
    assert "[PASS] Python lint (ruff check .)" in result.stdout
    assert "unused_import_probe.py" not in result.stdout


def test_lint_still_fails_for_production_python_with_repo_ruff_config(
    tmp_path: Path,
) -> None:
    _require_ruff()
    repo_root = _copy_lint_fixture(tmp_path)
    _track_files(
        repo_root,
        {
            "pyproject.toml": PYPROJECT_FILE.read_text(encoding="utf-8"),
            "scripts/production_probe.py": "import os\n",
        },
    )

    result = _run_lint(repo_root, env={"PATH": os.environ.get("PATH", "")})

    assert result.returncode == 1
    assert "[FAIL] Python lint (ruff check .)" in result.stdout
    assert "scripts/production_probe.py" in result.stdout
    assert "F401" in result.stdout
