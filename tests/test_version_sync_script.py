from __future__ import annotations

import json
import re
import shutil
import subprocess
import tomllib
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
VERSION_PATTERN = re.compile(r'^version\s*=\s*"([^"]+)"', re.MULTILINE)
CITATION_VERSION_PATTERN = re.compile(r'^version:\s*"?([^"\n]+)"?\s*$', re.MULTILINE)
LOCK_ROOT_VERSION_PATTERN = re.compile(
    r'(name = "biomcp-cli"\nversion = ")([^"]+)(")', re.MULTILINE
)
UV_LOCK_ROOT_VERSION_PATTERN = re.compile(
    r'(name = "biomcp-cli"\nversion = ")([^"]+)(")', re.MULTILINE
)


def _copy_version_sync_fixture(tmp_path: Path) -> Path:
    fixture_root = tmp_path / "repo"
    (fixture_root / "scripts").mkdir(parents=True)
    for relative_path in (
        "Cargo.toml",
        "Cargo.lock",
        "CITATION.cff",
        "manifest.json",
        "pyproject.toml",
        "server.json",
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


def _run_registry_check(repo_root: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["bash", "scripts/check-mcp-registry-server.sh"],
        cwd=repo_root,
        capture_output=True,
        text=True,
        check=False,
    )


def _read_version(path: Path) -> str:
    match = VERSION_PATTERN.search(path.read_text(encoding="utf-8"))
    assert match is not None, f"missing version in {path}"
    return match.group(1)


def _read_manifest_version(path: Path) -> str:
    return json.loads(path.read_text(encoding="utf-8"))["version"]


def _read_server_versions(path: Path) -> tuple[str, str]:
    server = json.loads(path.read_text(encoding="utf-8"))
    package = next(
        item
        for item in server["packages"]
        if item["registryType"] == "pypi" and item["identifier"] == "biomcp-cli"
    )
    return server["version"], package["version"]


def _read_citation_version(path: Path) -> str:
    match = CITATION_VERSION_PATTERN.search(path.read_text(encoding="utf-8"))
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


def _replace_manifest_version(path: Path, new_version: str) -> None:
    manifest = json.loads(path.read_text(encoding="utf-8"))
    manifest["version"] = new_version
    path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")


def _replace_server_version(path: Path, new_version: str) -> None:
    server = json.loads(path.read_text(encoding="utf-8"))
    server["version"] = new_version
    path.write_text(json.dumps(server, indent=2) + "\n", encoding="utf-8")


def _replace_server_package_version(path: Path, new_version: str) -> None:
    server = json.loads(path.read_text(encoding="utf-8"))
    for package in server["packages"]:
        if package["registryType"] == "pypi" and package["identifier"] == "biomcp-cli":
            package["version"] = new_version
            break
    path.write_text(json.dumps(server, indent=2) + "\n", encoding="utf-8")


def _replace_citation_version(path: Path, new_version: str) -> None:
    updated, count = CITATION_VERSION_PATTERN.subn(
        lambda match: match.group(0).replace(match.group(1), new_version, 1),
        path.read_text(encoding="utf-8"),
        count=1,
    )
    assert count == 1, f"missing version in {path}"
    path.write_text(updated, encoding="utf-8")


def test_version_sync_script_passes_when_all_versions_match(tmp_path: Path) -> None:
    repo_root = _copy_version_sync_fixture(tmp_path)
    expected_version = _read_version(repo_root / "Cargo.toml")
    assert _read_manifest_version(repo_root / "manifest.json") == expected_version
    assert _read_server_versions(repo_root / "server.json") == (expected_version, expected_version)
    assert _read_citation_version(repo_root / "CITATION.cff") == expected_version

    result = _run_version_sync_script(repo_root)

    assert result.returncode == 0
    assert result.stdout.strip() == f"Versions in sync: {expected_version}"
    assert result.stderr == ""


def test_registry_check_passes_for_current_metadata(tmp_path: Path) -> None:
    repo_root = _copy_version_sync_fixture(tmp_path)
    for relative_path in (
        "README.md",
        "docs/reference/mcp-server.md",
        "scripts/check-mcp-registry-server.sh",
    ):
        source = REPO_ROOT / relative_path
        target = repo_root / relative_path
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, target)

    result = _run_registry_check(repo_root)

    assert result.returncode == 0
    assert result.stdout.strip() == "MCP registry metadata ok"
    assert result.stderr == ""


def test_registry_check_rejects_wrong_registry_package_identifier(tmp_path: Path) -> None:
    repo_root = _copy_version_sync_fixture(tmp_path)
    for relative_path in (
        "README.md",
        "docs/reference/mcp-server.md",
        "scripts/check-mcp-registry-server.sh",
    ):
        source = REPO_ROOT / relative_path
        target = repo_root / relative_path
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, target)
    server = json.loads((repo_root / "server.json").read_text(encoding="utf-8"))
    server["packages"][0]["identifier"] = "biomcp"
    (repo_root / "server.json").write_text(json.dumps(server), encoding="utf-8")

    result = _run_registry_check(repo_root)

    assert result.returncode == 1
    assert "server.json must include a pypi package entry for biomcp-cli" in result.stderr


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


def test_version_sync_script_reports_manifest_mismatch(tmp_path: Path) -> None:
    repo_root = _copy_version_sync_fixture(tmp_path)
    manifest = repo_root / "manifest.json"
    current_version = _read_version(repo_root / "Cargo.toml")
    mismatched_version = f"{current_version}-manifest-mismatch"
    _replace_manifest_version(manifest, mismatched_version)

    result = _run_version_sync_script(repo_root)

    assert result.returncode == 1
    assert (
        f"Version mismatch: Cargo.toml={current_version}, "
        f"manifest.json={mismatched_version}"
    ) in result.stderr


def test_version_sync_script_reports_citation_mismatch(tmp_path: Path) -> None:
    repo_root = _copy_version_sync_fixture(tmp_path)
    citation = repo_root / "CITATION.cff"
    current_version = _read_version(repo_root / "Cargo.toml")
    mismatched_version = f"{current_version}-citation-mismatch"
    _replace_citation_version(citation, mismatched_version)

    result = _run_version_sync_script(repo_root)

    assert result.returncode == 1
    assert (
        f"Version mismatch: Cargo.toml={current_version}, "
        f"CITATION.cff={mismatched_version}"
    ) in result.stderr


def test_version_sync_script_reports_server_json_mismatch(tmp_path: Path) -> None:
    repo_root = _copy_version_sync_fixture(tmp_path)
    server = repo_root / "server.json"
    current_version = _read_version(repo_root / "Cargo.toml")
    mismatched_version = f"{current_version}-server-mismatch"
    _replace_server_version(server, mismatched_version)

    result = _run_version_sync_script(repo_root)

    assert result.returncode == 1
    assert (
        f"Version mismatch: Cargo.toml={current_version}, "
        f"server.json={mismatched_version}"
    ) in result.stderr


def test_version_sync_script_reports_server_package_mismatch(tmp_path: Path) -> None:
    repo_root = _copy_version_sync_fixture(tmp_path)
    server = repo_root / "server.json"
    current_version = _read_version(repo_root / "Cargo.toml")
    mismatched_version = f"{current_version}-server-package-mismatch"
    _replace_server_package_version(server, mismatched_version)

    result = _run_version_sync_script(repo_root)

    assert result.returncode == 1
    assert (
        f"Version mismatch: Cargo.toml={current_version}, "
        f"server.json biomcp-cli={mismatched_version}"
    ) in result.stderr


def test_version_sync_script_reports_all_mismatches_in_one_run(tmp_path: Path) -> None:
    repo_root = _copy_version_sync_fixture(tmp_path)
    pyproject = repo_root / "pyproject.toml"
    cargo_lock = repo_root / "Cargo.lock"
    manifest = repo_root / "manifest.json"
    citation = repo_root / "CITATION.cff"
    server = repo_root / "server.json"
    current_version = _read_version(repo_root / "Cargo.toml")
    pyproject_mismatch = f"{current_version}-pyproject-mismatch"
    lock_mismatch = f"{current_version}-lock-mismatch"
    manifest_mismatch = f"{current_version}-manifest-mismatch"
    citation_mismatch = f"{current_version}-citation-mismatch"
    server_mismatch = f"{current_version}-server-mismatch"
    server_package_mismatch = f"{current_version}-server-package-mismatch"
    _replace_first_version(pyproject, pyproject_mismatch)
    _replace_lock_root_version(cargo_lock, lock_mismatch)
    _replace_manifest_version(manifest, manifest_mismatch)
    _replace_citation_version(citation, citation_mismatch)
    _replace_server_version(server, server_mismatch)
    _replace_server_package_version(server, server_package_mismatch)

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
    assert (
        f"Version mismatch: Cargo.toml={current_version}, "
        f"manifest.json={manifest_mismatch}"
    ) in result.stderr
    assert (
        f"Version mismatch: Cargo.toml={current_version}, "
        f"CITATION.cff={citation_mismatch}"
    ) in result.stderr
    assert (
        f"Version mismatch: Cargo.toml={current_version}, "
        f"server.json={server_mismatch}"
    ) in result.stderr
    assert (
        f"Version mismatch: Cargo.toml={current_version}, "
        f"server.json biomcp-cli={server_package_mismatch}"
    ) in result.stderr


def test_manifest_and_citation_versions_match_repo_metadata() -> None:
    cargo = tomllib.loads((REPO_ROOT / "Cargo.toml").read_text(encoding="utf-8"))
    pyproject = tomllib.loads((REPO_ROOT / "pyproject.toml").read_text(encoding="utf-8"))

    assert cargo["package"]["version"] == "0.8.25"
    assert pyproject["project"]["version"] == "0.8.25"
    assert _read_manifest_version(REPO_ROOT / "manifest.json") == cargo["package"]["version"]
    assert _read_manifest_version(REPO_ROOT / "manifest.json") == pyproject["project"]["version"]
    assert _read_server_versions(REPO_ROOT / "server.json") == (
        cargo["package"]["version"],
        pyproject["project"]["version"],
    )
    assert _read_citation_version(REPO_ROOT / "CITATION.cff") == cargo["package"]["version"]
    assert _read_citation_version(REPO_ROOT / "CITATION.cff") == pyproject["project"]["version"]


def test_uv_lock_matches_release_version_and_excludes_mustmatch_package() -> None:
    uv_lock = (REPO_ROOT / "uv.lock").read_text(encoding="utf-8")

    root_match = UV_LOCK_ROOT_VERSION_PATTERN.search(uv_lock)

    assert root_match is not None, "missing biomcp-cli package entry in uv.lock"
    assert root_match.group(2) == "0.8.25"
    assert 'name = "mustmatch"' not in uv_lock
    assert "mustmatch" + "==0.0.4" not in uv_lock
    assert 'specifier = "==0.0.4"' not in uv_lock
