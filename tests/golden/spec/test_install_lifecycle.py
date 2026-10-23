"""Standalone uninstall operates only on the installation it owns."""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
from pathlib import Path

import pytest

from replay_env import ReplayEnvironment
from test_golden_output import BINARY
from test_public_installer_checksum import _paused_installer, _resume_installer

pytestmark = [
    pytest.mark.needs_binary,
    pytest.mark.skipif(os.name != "posix", reason="standalone uninstall requires Unix"),
]


def _installation(tmp_path: Path) -> tuple[Path, Path, Path]:
    prefix = tmp_path / "installation"
    prefix.mkdir()
    executable = prefix / "biomcp"
    shutil.copy2(BINARY, executable)
    receipt = prefix / "biomcp.install.json"
    receipt.write_text(
        json.dumps(
            {
                "schema_version": 1,
                "installer": "biomcp-standalone-installer",
                "state": "installed",
                "executable_path": str(executable.resolve()),
                "version": "0.0.0",
                "sha256": hashlib.sha256(executable.read_bytes()).hexdigest(),
            }
        ),
        encoding="utf-8",
    )
    sibling = prefix / "keep.txt"
    sibling.write_text("unrelated installation content", encoding="utf-8")
    return executable, receipt, sibling


def test_uninstall_removes_owned_binary_and_receipt_only(
    tmp_path: Path, replay: ReplayEnvironment
) -> None:
    executable, receipt, sibling = _installation(tmp_path)

    completed = subprocess.run(
        [str(executable), "--json", "uninstall"],
        env=replay.command_env(tmp_path),
        capture_output=True,
        text=True,
        check=False,
    )

    assert completed.returncode == 0, completed.stderr
    result = json.loads(completed.stdout)
    assert result["kind"] == "uninstall"
    assert result["status"] == "uninstalled"
    assert result["changed"] is True
    assert not executable.exists()
    assert not receipt.exists()
    assert not (executable.parent / ".biomcp-install.lock").exists()
    assert sibling.read_text(encoding="utf-8") == "unrelated installation content"


@pytest.mark.parametrize("invalid_receipt", ["missing", "malformed", "wrong-checksum"])
def test_uninstall_refuses_unproven_ownership_without_removing_files(
    tmp_path: Path, replay: ReplayEnvironment, invalid_receipt: str
) -> None:
    executable, receipt, sibling = _installation(tmp_path)
    if invalid_receipt == "missing":
        receipt.unlink()
    elif invalid_receipt == "malformed":
        receipt.write_text("{", encoding="utf-8")
    else:
        data = json.loads(receipt.read_text(encoding="utf-8"))
        data["sha256"] = "0" * 64
        receipt.write_text(json.dumps(data), encoding="utf-8")
    binary_before = executable.read_bytes()
    receipt_before = receipt.read_bytes() if receipt.exists() else None

    completed = subprocess.run(
        [str(executable), "uninstall"],
        env=replay.command_env(tmp_path),
        capture_output=True,
        text=True,
        check=False,
    )

    assert completed.returncode != 0
    assert "installer" in completed.stderr
    assert executable.read_bytes() == binary_before
    assert not (executable.parent / ".biomcp-install.lock").exists()
    assert (receipt.read_bytes() if receipt.exists() else None) == receipt_before
    assert sibling.read_text(encoding="utf-8") == "unrelated installation content"


def test_uninstall_never_recovers_a_live_installers_pending_receipt(
    tmp_path: Path, replay: ReplayEnvironment
) -> None:
    executable, receipt, sibling = _installation(tmp_path)
    first = tmp_path / "installer"
    first.mkdir()
    process = _paused_installer(first, executable.parent, phase="pending")
    pending = receipt.read_bytes()
    before_binary = executable.read_bytes()
    try:
        refused = subprocess.run(
            [str(executable), "uninstall"], env=replay.command_env(tmp_path),
            capture_output=True, text=True, timeout=60, check=False,
        )
        assert refused.returncode != 0
        assert "installation lock" in refused.stderr
        assert receipt.read_bytes() == pending
        assert executable.read_bytes() == before_binary
    finally:
        _, stderr = _resume_installer(process)
    assert process.returncode == 0, stderr
    installed = json.loads(receipt.read_text())
    assert installed["state"] == "installed"
    assert installed["sha256"] == hashlib.sha256(executable.read_bytes()).hexdigest()
    assert sibling.exists()
    assert not (executable.parent / ".biomcp-install.lock").exists()


@pytest.mark.parametrize("entry", ["directory", "file", "symlink"])
def test_uninstall_preserves_existing_lock_entries(
    tmp_path: Path, replay: ReplayEnvironment, entry: str
) -> None:
    executable, receipt, _ = _installation(tmp_path)
    lock = executable.parent / ".biomcp-install.lock"
    target = tmp_path / "target"
    target.mkdir()
    if entry == "directory":
        lock.mkdir()
    elif entry == "file":
        lock.write_text("untouched")
    else:
        lock.symlink_to(target, target_is_directory=True)
    before = receipt.read_bytes()
    refused = subprocess.run(
        [str(executable), "uninstall"], env=replay.command_env(tmp_path),
        capture_output=True, text=True, timeout=60, check=False,
    )
    assert refused.returncode != 0
    assert "installation lock" in refused.stderr
    assert receipt.read_bytes() == before
    assert executable.exists()
    assert lock.is_symlink() if entry == "symlink" else lock.exists()
    assert target.is_dir()
