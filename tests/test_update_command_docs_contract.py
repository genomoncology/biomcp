"""Contracts for the fail-closed standalone updater surface."""

from __future__ import annotations

import os
import subprocess
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]


def _read(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def _release_bin() -> Path:
    configured = os.environ.get("BIOMCP_BIN")
    return Path(configured) if configured else REPO_ROOT / "target/release/biomcp"


def _run(*args: str) -> str:
    binary = _release_bin()
    assert binary.exists(), f"missing release binary: {binary}"
    return subprocess.run(
        [str(binary), *args], cwd=REPO_ROOT, check=True, capture_output=True, text=True
    ).stdout


def test_update_help_has_no_unverified_install_escape_hatch() -> None:
    help_text = _run("update", "--help")
    assert "--check" in help_text
    assert "SHA256" in help_text
    assert "allow-missing-checksum" not in help_text


def test_list_and_docs_describe_only_verified_standalone_updates() -> None:
    surfaces = [
        _run("list"),
        _read("src/cli/list_reference.md"),
        _read("docs/user-guide/cli-reference.md"),
        _read("docs/troubleshooting.md"),
        _read("architecture/ux/cli-reference.md"),
        _read("README.md"),
    ]
    for surface in surfaces:
        assert "allow-missing-checksum" not in surface
    assert all("checksum" in surface.lower() or "SHA256" in surface for surface in surfaces)


def test_update_architecture_names_ownership_and_windows_installer_path() -> None:
    docs = _read("docs/user-guide/cli-reference.md") + _read("docs/troubleshooting.md")
    assert "standalone installer" in docs.lower()
    assert "Windows" in docs
    assert "biomcp.install.json" in docs
