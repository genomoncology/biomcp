"""Exercise pinned scanner installation failures without downloading executables."""
from __future__ import annotations

import os
from pathlib import Path
import shutil
import subprocess

import pytest

from test_lint import REPO_ROOT, _write_executable


def _fixture(tmp_path: Path) -> tuple[Path, dict[str, str]]:
    root = tmp_path / "repo"
    (root / "tools").mkdir(parents=True)
    shutil.copy2(REPO_ROOT / "tools/bootstrap-lint-tools", root / "tools/bootstrap-lint-tools")
    subprocess.run(["git", "init", "-q"], cwd=root, check=True)
    commands = tmp_path / "commands"
    commands.mkdir()
    _write_executable(commands / "uname", '#!/usr/bin/env bash\ncase "$1" in -s) echo "$TEST_SYSTEM" ;; -m) echo "$TEST_ARCH" ;; esac\n')
    _write_executable(commands / "curl", '#!/usr/bin/env bash\nprintf "%s\\n" "$*" >> "$CURL_LOG"\nif [ "${CURL_STATUS:-22}" != 0 ]; then exit "${CURL_STATUS:-22}"; fi\nwhile [ "$#" -gt 0 ]; do if [ "$1" = -o ]; then shift; printf invalid > "$1"; fi; shift; done\n')
    env = os.environ | {
        "PATH": str(commands) + os.pathsep + os.environ["PATH"],
        "TEST_SYSTEM": "Linux", "TEST_ARCH": "x86_64",
        "CURL_LOG": str(tmp_path / "curl.log"),
        "BIOMCP_TOOL_DIR": str(tmp_path / "installed"),
    }
    return root, env


def _run(root: Path, env: dict[str, str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(["bash", "tools/bootstrap-lint-tools", "--advisories"],
                          cwd=root, env=env, capture_output=True, text=True, check=False)


@pytest.mark.parametrize("system,arch,asset", [
    ("Linux", "x86_64", "linux_amd64"), ("Linux", "aarch64", "linux_arm64"),
    ("Darwin", "arm64", "darwin_arm64"), ("Darwin", "x86_64", "darwin_amd64"),
])
def test_advisory_bootstrap_uses_pinned_official_asset_and_fails_on_download_error(
    tmp_path: Path, system: str, arch: str, asset: str,
) -> None:
    root, env = _fixture(tmp_path)
    env |= {"TEST_SYSTEM": system, "TEST_ARCH": arch}
    result = _run(root, env)
    assert result.returncode != 0
    command = Path(env["CURL_LOG"]).read_text()
    assert f"https://github.com/google/osv-scanner/releases/download/v2.7.0/osv-scanner_{asset}" in command
    assert "--proto =https --tlsv1.2" in command
    assert not (Path(env["BIOMCP_TOOL_DIR"]) / "osv-scanner").exists()


def test_advisory_bootstrap_rejects_a_download_with_the_wrong_digest(tmp_path: Path) -> None:
    root, env = _fixture(tmp_path)
    env["CURL_STATUS"] = "0"
    result = _run(root, env)
    assert result.returncode != 0
    assert "checksum mismatch for osv-scanner" in result.stderr
    assert not (Path(env["BIOMCP_TOOL_DIR"]) / "osv-scanner").exists()


def test_advisory_bootstrap_reuses_pinned_version_without_network(tmp_path: Path) -> None:
    root, env = _fixture(tmp_path)
    tool_dir = Path(env["BIOMCP_TOOL_DIR"])
    tool_dir.mkdir()
    _write_executable(tool_dir / "osv-scanner", "#!/usr/bin/env bash\necho osv-scanner version: 2.7.0\n")
    result = _run(root, env)
    assert result.returncode == 0, result.stdout + result.stderr
    assert result.stdout.strip() == str(tool_dir)
    assert not Path(env["CURL_LOG"]).exists()


def test_advisory_bootstrap_rejects_an_unsupported_platform(tmp_path: Path) -> None:
    root, env = _fixture(tmp_path)
    env["TEST_ARCH"] = "unsupported"
    result = _run(root, env)
    assert result.returncode != 0
    assert "No pinned advisory scanner" in result.stderr
    assert not Path(env["CURL_LOG"]).exists()


def _cargo_fixture(tmp_path: Path) -> tuple[Path, dict[str, str]]:
    root, env = _fixture(tmp_path)
    policy = root / ".github/workflows/ci.yml"
    policy.parent.mkdir(parents=True)
    shutil.copyfile(REPO_ROOT / ".github/workflows/ci.yml", policy)
    tools = Path(env["BIOMCP_TOOL_DIR"])
    tools.mkdir()
    for name, version in [("shellcheck", "0.10.0"), ("actionlint", "1.7.12")]:
        _write_executable(tools / name, f"#!/bin/sh\necho {name} {version}\n")
    return root, env


@pytest.mark.parametrize("system,arch,target", [
    ("Linux", "x86_64", "x86_64-unknown-linux-musl"),
    ("Darwin", "arm64", "aarch64-apple-darwin"),
])
@pytest.mark.parametrize("cached_version", [None, "0.19.40"])
def test_cargo_deny_download_matches_policy_and_rejects_lookalike_cached_versions(
    tmp_path: Path, system: str, arch: str, target: str, cached_version: str | None,
) -> None:
    root, env = _cargo_fixture(tmp_path)
    env |= {"TEST_SYSTEM": system, "TEST_ARCH": arch}
    if cached_version:
        _write_executable(Path(env["BIOMCP_TOOL_DIR"]) / "cargo-deny",
                          f"#!/bin/sh\necho cargo-deny {cached_version}\n")
    result = subprocess.run(["bash", "tools/bootstrap-lint-tools"], cwd=root, env=env,
                            capture_output=True, text=True)
    assert result.returncode != 0
    command = Path(env["CURL_LOG"]).read_text()
    assert f"https://github.com/EmbarkStudios/cargo-deny/releases/download/0.19.4/cargo-deny-0.19.4-{target}.tar.gz" in command
    assert "--proto =https --tlsv1.2" in command


def test_cargo_deny_rejects_corrupt_download_and_unreviewed_policy(tmp_path: Path) -> None:
    root, env = _cargo_fixture(tmp_path)
    env["CURL_STATUS"] = "0"
    result = subprocess.run(["bash", "tools/bootstrap-lint-tools"], cwd=root, env=env,
                            capture_output=True, text=True)
    assert result.returncode != 0
    assert "checksum mismatch for cargo-deny" in result.stderr
    assert not (Path(env["BIOMCP_TOOL_DIR"]) / "cargo-deny").exists()
    policy = root / ".github/workflows/ci.yml"
    policy.write_text(policy.read_text().replace("CARGO_DENY_VERSION: 0.19.4", "CARGO_DENY_VERSION: 0.19.5"))
    Path(env["CURL_LOG"]).unlink()
    result = subprocess.run(["bash", "tools/bootstrap-lint-tools"], cwd=root, env=env,
                            capture_output=True, text=True)
    assert result.returncode != 0
    assert "No pinned cargo-deny archive for policy 0.19.5" in result.stderr
    assert not Path(env["CURL_LOG"]).exists()


def test_cargo_deny_reuses_only_the_exact_policy_version_without_network(tmp_path: Path) -> None:
    root, env = _cargo_fixture(tmp_path)
    _write_executable(Path(env["BIOMCP_TOOL_DIR"]) / "cargo-deny",
                      "#!/bin/sh\necho cargo-deny 0.19.4\n")
    result = subprocess.run(["bash", "tools/bootstrap-lint-tools"], cwd=root, env=env,
                            capture_output=True, text=True)
    assert result.returncode == 0, result.stdout + result.stderr
    assert result.stdout.strip() == env["BIOMCP_TOOL_DIR"]
    assert not Path(env["CURL_LOG"]).exists()
