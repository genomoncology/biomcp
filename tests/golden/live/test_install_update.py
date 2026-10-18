"""Opt-in published-release update proof in a temporary installation.

Run explicitly at release qualification, outside offline gates:
BIOMCP_TEST_LIVE_UPDATE=1 pytest tests/golden/live/test_install_update.py
"""

from __future__ import annotations

import hashlib
import json
import os
import subprocess
import urllib.request
from pathlib import Path

import pytest

from replay_env import REPO_ROOT, ReplayEnvironment

if os.environ.get("BIOMCP_TEST_LIVE_UPDATE") != "1":
    pytest.skip("published-release update requires explicit live opt-in", allow_module_level=True)

pytestmark = [
    pytest.mark.needs_binary,
    pytest.mark.skipif(os.name != "posix", reason="self-update requires Unix"),
]

LATEST_RELEASE = "https://api.github.com/repos/genomoncology/biomcp/releases/latest"
OLD_VERSION = "0.9.2"


def _run(argv: list[str], env: dict[str, str]) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(argv, env=env, capture_output=True, text=True, check=False)
    assert result.returncode == 0, f"{argv}:\n{result.stdout}\n{result.stderr}"
    return result


def test_published_install_checks_and_updates_to_latest_release(
    tmp_path: Path, replay: ReplayEnvironment
) -> None:
    request = urllib.request.Request(LATEST_RELEASE, headers={"User-Agent": "BioMCP-release-test"})
    with urllib.request.urlopen(request, timeout=30) as response:
        latest = json.load(response)
    latest_tag = latest["tag_name"]
    latest_version = latest_tag.removeprefix("v")
    assert latest_version != OLD_VERSION, "published upgrade proof needs a newer release"

    prefix = tmp_path / "installation"
    env = replay.command_env(tmp_path) | {
        "BIOMCP_GITHUB_REPO": "genomoncology/biomcp",
        "BIOMCP_VERSION": OLD_VERSION,
        "BIOMCP_INSTALL_DIR": str(prefix),
    }
    _run(["bash", str(REPO_ROOT / "install.sh")], env)
    executable = prefix / "biomcp"
    receipt = prefix / "biomcp.install.json"
    sibling = prefix / "keep.txt"
    sibling.write_text("unrelated installation content", encoding="utf-8")
    original_binary = executable.read_bytes()
    original_receipt = receipt.read_bytes()
    original_identity = json.loads(original_receipt)
    assert original_identity["version"].removeprefix("v") == OLD_VERSION
    assert original_identity["sha256"] == hashlib.sha256(original_binary).hexdigest()

    checked = _run([str(executable), "update", "--check"], env)
    assert f"Current version: {OLD_VERSION}" in checked.stdout
    assert f"Latest version: {latest_tag}" in checked.stdout
    assert "not up to date (update available)" in checked.stdout
    assert executable.read_bytes() == original_binary
    assert receipt.read_bytes() == original_receipt

    updated = _run([str(executable), "update"], env)
    assert f"Updated BioMCP to {latest_tag}" in updated.stdout
    installed_version = _run([str(executable), "version"], env)
    assert latest_version in installed_version.stdout.split()
    installed_binary = executable.read_bytes()
    assert installed_binary != original_binary
    installed_identity = json.loads(receipt.read_text(encoding="utf-8"))
    assert installed_identity == {
        "schema_version": 1,
        "installer": "biomcp-standalone-installer",
        "state": "installed",
        "executable_path": str(executable.resolve()),
        "version": latest_tag,
        "sha256": hashlib.sha256(installed_binary).hexdigest(),
    }
    assert sibling.read_text(encoding="utf-8") == "unrelated installation content"
    assert not list(prefix.glob(".biomcp-stage-*"))
