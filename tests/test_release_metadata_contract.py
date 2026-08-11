from __future__ import annotations

import json
import re
import shutil
import subprocess
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
VERSION_FILES = (
    "Cargo.toml",
    "Cargo.lock",
    "pyproject.toml",
    "uv.lock",
    "manifest.json",
    "server.json",
    "CITATION.cff",
    "Formula/biomcp.rb",
)


def _copy_release_metadata_fixture(tmp_path: Path) -> Path:
    repo = tmp_path / "repo"
    for relative in (*VERSION_FILES, "CHANGELOG.md", "scripts/check-version-sync.sh"):
        source = REPO_ROOT / relative
        destination = repo / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, destination)
    subprocess.run(["git", "init", "-q"], cwd=repo, check=True)
    subprocess.run(
        ["git", "config", "user.email", "test@example.com"], cwd=repo, check=True
    )
    subprocess.run(["git", "config", "user.name", "Test User"], cwd=repo, check=True)
    subprocess.run(["git", "add", "."], cwd=repo, check=True)
    subprocess.run(["git", "commit", "-qm", "published release"], cwd=repo, check=True)
    subprocess.run(["git", "tag", "v0.8.25"], cwd=repo, check=True)
    return repo


def _run_version_lock(repo: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["bash", "scripts/check-version-sync.sh"],
        cwd=repo,
        text=True,
        capture_output=True,
        check=False,
    )


def _replace_root_package_version(path: Path, version: str) -> None:
    text = path.read_text(encoding="utf-8")
    updated, replacements = re.subn(
        r'(name = "biomcp-cli"\nversion = ")[^"]+(")',
        rf"\g<1>{version}\2",
        text,
        count=1,
    )
    assert replacements == 1
    path.write_text(updated, encoding="utf-8")


def _set_every_concrete_version(repo: Path, version: str) -> None:
    for relative in ("Cargo.toml", "pyproject.toml"):
        path = repo / relative
        text = path.read_text(encoding="utf-8")
        updated, replacements = re.subn(
            r'(?m)^version = "[^"]+"', f'version = "{version}"', text, count=1
        )
        assert replacements == 1
        path.write_text(updated, encoding="utf-8")
    for relative in ("Cargo.lock", "uv.lock"):
        _replace_root_package_version(repo / relative, version)
    for relative in ("manifest.json", "server.json"):
        path = repo / relative
        metadata = json.loads(path.read_text(encoding="utf-8"))
        metadata["version"] = version
        if relative == "server.json":
            package = next(
                item
                for item in metadata["packages"]
                if item["identifier"] == "biomcp-cli"
            )
            package["version"] = version
        path.write_text(json.dumps(metadata, indent=2) + "\n", encoding="utf-8")
    citation = repo / "CITATION.cff"
    citation.write_text(
        re.sub(
            r"(?m)^version: .*$",
            f"version: {version}",
            citation.read_text(encoding="utf-8"),
            count=1,
        ),
        encoding="utf-8",
    )
    formula = repo / "Formula/biomcp.rb"
    updated, replacements = re.subn(
        r'(?m)^  version "(?:__VERSION__|[^"]+)"$',
        f'  version "{version}"',
        formula.read_text(encoding="utf-8"),
        count=1,
    )
    assert replacements == 1
    formula.write_text(updated, encoding="utf-8")


def test_version_lock_rejects_root_uv_lock_drift(tmp_path: Path) -> None:
    repo = _copy_release_metadata_fixture(tmp_path)
    _replace_root_package_version(repo / "uv.lock", "0.8.24")

    result = _run_version_lock(repo)

    assert result.returncode != 0
    assert "uv.lock" in result.stderr


def test_version_lock_rejects_formula_version_drift(tmp_path: Path) -> None:
    repo = _copy_release_metadata_fixture(tmp_path)
    formula = repo / "Formula/biomcp.rb"
    updated, replacements = re.subn(
        r'(?m)^  version "(?:__VERSION__|[^"]+)"$',
        '  version "0.8.24"',
        formula.read_text(encoding="utf-8"),
        count=1,
    )
    assert replacements == 1
    formula.write_text(updated, encoding="utf-8")

    result = _run_version_lock(repo)

    assert result.returncode != 0
    assert "Formula/biomcp.rb" in result.stderr


def test_version_lock_rejects_dirty_future_release_rewrite(tmp_path: Path) -> None:
    repo = _copy_release_metadata_fixture(tmp_path)
    _set_every_concrete_version(repo, "0.9.0")

    result = _run_version_lock(repo)

    assert result.returncode != 0
    assert "release version changes must be committed" in result.stderr


def test_version_lock_rejects_placeholder_doi(tmp_path: Path) -> None:
    repo = _copy_release_metadata_fixture(tmp_path)

    result = _run_version_lock(repo)

    assert result.returncode != 0
    assert "placeholder DOI" in result.stderr


def test_breaking_unreleased_requires_a_pre_1_0_minor_bump(tmp_path: Path) -> None:
    repo = _copy_release_metadata_fixture(tmp_path)
    (repo / "CHANGELOG.md").write_text(
        "# Changelog\n\n## Unreleased\n\n### Breaking changes\n\n- Changed a public command.\n\n## 0.8.25 — 2026-07-07\n",
        encoding="utf-8",
    )

    result = _run_version_lock(repo)

    assert result.returncode != 0
    assert (
        "breaking changes require a minor version increase before 1.0" in result.stderr
    )


def test_nonbreaking_unreleased_keeps_the_published_pre_1_0_version_valid(
    tmp_path: Path,
) -> None:
    repo = _copy_release_metadata_fixture(tmp_path)
    (repo / "CHANGELOG.md").write_text(
        "# Changelog\n\n## Unreleased\n\n### Fixed\n\n- Corrected a display label.\n\n## 0.8.25 — 2026-07-07\n",
        encoding="utf-8",
    )
    citation = repo / "CITATION.cff"
    citation.write_text(
        re.sub(
            r"(?ms)^preferred-citation:.*", "", citation.read_text(encoding="utf-8")
        ),
        encoding="utf-8",
    )

    result = _run_version_lock(repo)

    assert result.returncode == 0, result.stderr


def test_breaking_unreleased_accepts_the_next_pre_1_0_minor_version(
    tmp_path: Path,
) -> None:
    repo = _copy_release_metadata_fixture(tmp_path)
    _set_every_concrete_version(repo, "0.9.0")
    (repo / "CHANGELOG.md").write_text(
        "# Changelog\n\n## Unreleased\n\n### Breaking changes\n\n- Changed a public command.\n\n## 0.8.25 — 2026-07-07\n",
        encoding="utf-8",
    )
    citation = repo / "CITATION.cff"
    citation.write_text(
        re.sub(
            r"(?ms)^preferred-citation:.*", "", citation.read_text(encoding="utf-8")
        ),
        encoding="utf-8",
    )
    subprocess.run(["git", "add", "."], cwd=repo, check=True)
    subprocess.run(["git", "commit", "-qm", "prepare 0.9.0"], cwd=repo, check=True)

    result = _run_version_lock(repo)

    assert result.returncode == 0, result.stderr
