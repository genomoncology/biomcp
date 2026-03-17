from __future__ import annotations

import re
import shutil
import subprocess
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
VERSION_PATTERN = re.compile(r'^version\s*=\s*"([^"]+)"', re.MULTILINE)
LOCK_ROOT_VERSION_PATTERN = re.compile(
    r'(name = "biomcp-cli"\nversion = ")([^"]+)(")', re.MULTILINE
)


def _copy_version_sync_fixture(tmp_path: Path) -> Path:
    fixture_root = tmp_path / "repo"
    (fixture_root / "scripts").mkdir(parents=True)
    for relative_path in (
        "Cargo.toml",
        "Cargo.lock",
        "pyproject.toml",
        "scripts/check-version-sync.sh",
    ):
        source = REPO_ROOT / relative_path
        target = fixture_root / relative_path
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, target)
    return fixture_root


def _run_version_sync_script(repo_root: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["bash", "scripts/check-version-sync.sh"],
        cwd=repo_root,
        capture_output=True,
        text=True,
        check=False,
    )


def _read_version(path: Path) -> str:
    match = VERSION_PATTERN.search(path.read_text(encoding="utf-8"))
    assert match is not None, f"missing version in {path}"
    return match.group(1)


def _replace_first_version(path: Path, new_version: str) -> None:
    updated, count = VERSION_PATTERN.subn(
        lambda match: match.group(0).replace(match.group(1), new_version, 1),
        path.read_text(encoding="utf-8"),
        count=1,
    )
    assert count == 1, f"missing version in {path}"
    path.write_text(updated, encoding="utf-8")


def _replace_lock_root_version(path: Path, new_version: str) -> None:
    updated, count = LOCK_ROOT_VERSION_PATTERN.subn(
        rf"\g<1>{new_version}\g<3>",
        path.read_text(encoding="utf-8"),
        count=1,
    )
    assert count == 1, f"missing biomcp-cli lockfile version in {path}"
    path.write_text(updated, encoding="utf-8")


def test_version_sync_script_passes_when_all_versions_match(tmp_path: Path) -> None:
    repo_root = _copy_version_sync_fixture(tmp_path)
    expected_version = _read_version(repo_root / "Cargo.toml")

    result = _run_version_sync_script(repo_root)

    assert result.returncode == 0
    assert result.stdout.strip() == f"Versions in sync: {expected_version}"
    assert result.stderr == ""


def test_version_sync_script_reports_pyproject_mismatch(tmp_path: Path) -> None:
    repo_root = _copy_version_sync_fixture(tmp_path)
    pyproject = repo_root / "pyproject.toml"
    current_version = _read_version(repo_root / "Cargo.toml")
    mismatched_version = f"{current_version}-pyproject-mismatch"
    _replace_first_version(pyproject, mismatched_version)

    result = _run_version_sync_script(repo_root)

    assert result.returncode == 1
    assert (
        f"Version mismatch: Cargo.toml={current_version}, "
        f"pyproject.toml={mismatched_version}"
    ) in result.stderr


def test_version_sync_script_reports_cargo_lock_mismatch(tmp_path: Path) -> None:
    repo_root = _copy_version_sync_fixture(tmp_path)
    cargo_lock = repo_root / "Cargo.lock"
    current_version = _read_version(repo_root / "Cargo.toml")
    mismatched_version = f"{current_version}-lock-mismatch"
    _replace_lock_root_version(cargo_lock, mismatched_version)

    result = _run_version_sync_script(repo_root)

    assert result.returncode == 1
    assert (
        f"Version mismatch: Cargo.toml={current_version}, "
        f"Cargo.lock={mismatched_version}"
    ) in result.stderr


def test_version_sync_script_reports_all_mismatches_in_one_run(tmp_path: Path) -> None:
    repo_root = _copy_version_sync_fixture(tmp_path)
    pyproject = repo_root / "pyproject.toml"
    cargo_lock = repo_root / "Cargo.lock"
    current_version = _read_version(repo_root / "Cargo.toml")
    pyproject_mismatch = f"{current_version}-pyproject-mismatch"
    lock_mismatch = f"{current_version}-lock-mismatch"
    _replace_first_version(pyproject, pyproject_mismatch)
    _replace_lock_root_version(cargo_lock, lock_mismatch)

    result = _run_version_sync_script(repo_root)

    assert result.returncode == 1
    assert (
        f"Version mismatch: Cargo.toml={current_version}, "
        f"pyproject.toml={pyproject_mismatch}"
    ) in result.stderr
    assert (
        f"Version mismatch: Cargo.toml={current_version}, "
        f"Cargo.lock={lock_mismatch}"
    ) in result.stderr
