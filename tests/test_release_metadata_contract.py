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


def _set_development_package_versions(
    repo: Path, rust_version: str, python_version: str
) -> None:
    for relative, version in (
        ("Cargo.toml", rust_version),
        ("pyproject.toml", python_version),
    ):
        path = repo / relative
        updated, replacements = re.subn(
            r'(?m)^version = "[^"]+"',
            f'version = "{version}"',
            path.read_text(encoding="utf-8"),
            count=1,
        )
        assert replacements == 1
        path.write_text(updated, encoding="utf-8")
    _replace_root_package_version(repo / "Cargo.lock", rust_version)
    _replace_root_package_version(repo / "uv.lock", python_version)


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
    citation = repo / "CITATION.cff"
    citation.write_text(
        citation.read_text(encoding="utf-8") + "doi: 10.5281/zenodo.XXXXXXX\n",
        encoding="utf-8",
    )

    result = _run_version_lock(repo)

    assert result.returncode != 0
    assert "placeholder DOI" in result.stderr


def test_breaking_unreleased_requires_a_pre_1_0_minor_bump(tmp_path: Path) -> None:
    repo = _copy_release_metadata_fixture(tmp_path)
    _set_development_package_versions(repo, "0.8.25-dev.1", "0.8.25.dev1")
    (repo / "CHANGELOG.md").write_text(
        "# Changelog\n\n## Unreleased\n\n### Breaking changes\n\n- Changed a public command.\n\n## 0.8.25 — 2026-07-07\n",
        encoding="utf-8",
    )
    subprocess.run(["git", "add", "."], cwd=repo, check=True)
    subprocess.run(["git", "commit", "-qm", "bad development base"], cwd=repo, check=True)

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


def test_breaking_unreleased_rejects_patch_stable_and_development_versions(
    tmp_path: Path,
) -> None:
    stable = _copy_release_metadata_fixture(tmp_path / "stable")
    _set_every_concrete_version(stable, "0.8.26")
    subprocess.run(["git", "add", "."], cwd=stable, check=True)
    subprocess.run(["git", "commit", "-qm", "prepare 0.8.26"], cwd=stable, check=True)
    stable_result = _run_version_lock(stable)
    assert stable_result.returncode != 0
    assert "breaking changes require a minor version increase" in stable_result.stderr

    development = _copy_release_metadata_fixture(tmp_path / "development")
    _set_development_package_versions(development, "0.8.26-dev.1", "0.8.26.dev1")
    subprocess.run(["git", "add", "."], cwd=development, check=True)
    subprocess.run(
        ["git", "commit", "-qm", "prepare 0.8.26 development"],
        cwd=development,
        check=True,
    )
    development_result = _run_version_lock(development)
    assert development_result.returncode != 0
    assert (
        "breaking changes require a minor version increase"
        in development_result.stderr
    )


def test_development_candidate_rejects_wrong_python_mapping(tmp_path: Path) -> None:
    repo = _copy_release_metadata_fixture(tmp_path)
    _set_development_package_versions(repo, "0.9.0-dev.1", "0.9.0.dev2")
    subprocess.run(["git", "add", "."], cwd=repo, check=True)
    subprocess.run(["git", "commit", "-qm", "wrong mapping"], cwd=repo, check=True)

    result = _run_version_lock(repo)

    assert result.returncode != 0
    assert "Cargo.toml mapping=0.9.0.dev1" in result.stderr


def test_population_response_change_is_explicitly_breaking() -> None:
    changelog = (REPO_ROOT / "CHANGELOG.md").read_text(encoding="utf-8")
    unreleased = changelog.split("## Unreleased", 1)[1].split("\n## ", 1)[0]
    breaking = unreleased.split("### Breaking changes", 1)[1].split("\n### ", 1)[0]
    features = unreleased.split("### New features", 1)[1].split("\n### ", 1)[0]
    marker = "Replaced the legacy MyVariant/ExAC variant-detail population fields"
    assert marker in breaking
    assert marker not in features
