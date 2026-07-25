from __future__ import annotations

import hashlib
import os
import shutil
import stat
import subprocess
import tarfile
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]
INSTALLER = REPO_ROOT / "install.sh"
ASSET = "biomcp-linux-x86_64.tar.gz"


def _write_executable(path: Path, content: str) -> None:
    path.write_text(content, encoding="utf-8")
    path.chmod(path.stat().st_mode | stat.S_IXUSR)


def _fixture(tmp_path: Path, sidecar: str | None) -> tuple[Path, str]:
    fixture = tmp_path / "fixture"
    fixture.mkdir()
    binary = fixture / "biomcp"
    _write_executable(binary, "#!/bin/sh\necho 'biomcp 0.0.0'\n")
    archive = fixture / ASSET
    with tarfile.open(archive, "w:gz") as tar:
        tar.add(binary, arcname="biomcp")
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    if sidecar is not None:
        (fixture / "sidecar").write_text(sidecar.format(digest=digest), encoding="utf-8")
    return fixture, digest


def _fake_downloader(path: Path, command: str) -> None:
    if command == "curl":
        args = """while [ "$#" -gt 0 ]; do
  case "$1" in
    -o) destination="$2"; shift 2 ;;
    *) url="$1"; shift ;;
  esac
done"""
    else:
        args = """while [ "$#" -gt 0 ]; do
  case "$1" in
    -qO) destination="$2"; shift 2 ;;
    *) url="$1"; shift ;;
  esac
done"""
    _write_executable(
        path / command,
        f"#!/bin/sh\n{args}\ncase \"$url\" in\n  *.sha256) source=\"$FIXTURE_DIR/sidecar\" ;;\n  *) source=\"$FIXTURE_DIR/{ASSET}\" ;;\nesac\n[ -f \"$source\" ] || exit 22\ncp \"$source\" \"$destination\"\n",
    )


def _tools(tmp_path: Path, downloader: str, sha_tool: str) -> Path:
    tools = tmp_path / "tools"
    tools.mkdir()
    _fake_downloader(tools, downloader)
    for name in ("bash", "uname", "mktemp", "rm", "basename", "tar", "gzip", "mkdir", "chmod", "mv", "head", "cp"):
        target = shutil.which(name)
        assert target, name
        (tools / name).symlink_to(target)
    if sha_tool == "system":
        for name in ("sha256sum", "awk"):
            target = shutil.which(name)
            assert target, name
            (tools / name).symlink_to(target)
    elif sha_tool == "failure":
        _write_executable(tools / "sha256sum", "#!/bin/sh\nexit 1\n")
    elif sha_tool == "openssl":
        target = shutil.which("awk")
        assert target
        (tools / "awk").symlink_to(target)
        _write_executable(
            tools / "openssl",
            "#!/bin/sh\n[ \"$1 $2\" = 'dgst -sha256' ] || exit 1\n"
            "digest=$(/usr/bin/sha256sum \"$3\" | /usr/bin/awk '{print $1}')\n"
            "printf 'SHA2-256 (%s) = %s\\n' \"$3\" \"$digest\"\n",
        )
    return tools


def _run_installer(
    tmp_path: Path, sidecar: str | None, *, downloader: str = "curl", sha_tool: str = "system"
) -> subprocess.CompletedProcess[str]:
    fixture, _ = _fixture(tmp_path, sidecar)
    tools = _tools(tmp_path, downloader, sha_tool)
    install_dir = tmp_path / "install"
    scratch = tmp_path / "scratch"
    scratch.mkdir()
    env = os.environ | {
        "BIOMCP_INSTALL_DIR": str(install_dir),
        "BIOMCP_VERSION": "0.0.0",
        "FIXTURE_DIR": str(fixture),
        "TMPDIR": str(scratch),
        "PATH": str(tools),
    }
    result = subprocess.run(
        ["bash", str(INSTALLER)], text=True, capture_output=True, env=env, check=False
    )
    assert not list(scratch.iterdir())
    return result


@pytest.mark.parametrize(
    "sidecar",
    ["{digest}\n", "{digest}  biomcp-linux-x86_64.tar.gz\n", "{digest} *biomcp-linux-x86_64.tar.gz\n"],
)
def test_installer_accepts_one_valid_checksum_record(tmp_path: Path, sidecar: str) -> None:
    result = _run_installer(tmp_path, sidecar)

    assert result.returncode == 0, result.stderr
    assert "Checksum verified." in result.stdout
    assert (tmp_path / "install" / "biomcp").exists()
    assert "Verified installation: biomcp 0.0.0" in result.stdout


@pytest.mark.parametrize(
    "sidecar",
    [
        None,
        "",
        "not-a-checksum\n",
        "{digest} biomcp-linux-x86_64.tar.gz extra\n",
        "{digest} wrong-name.tar.gz\n",
        "{digest} ./biomcp-linux-x86_64.tar.gz\n",
        "# checksum\n{digest}\n",
        "{digest}\n{digest}\n",
    ],
)
def test_installer_refuses_unproven_checksum_before_destination_change(
    tmp_path: Path, sidecar: str | None
) -> None:
    result = _run_installer(tmp_path, sidecar)

    assert result.returncode != 0
    assert not (tmp_path / "install" / "biomcp").exists()


def test_installer_rejects_mismatched_checksum_before_extraction(tmp_path: Path) -> None:
    result = _run_installer(tmp_path, f"{'0' * 64}\n")

    assert result.returncode != 0
    assert "Checksum verification failed" in result.stderr
    assert not (tmp_path / "install" / "biomcp").exists()


@pytest.mark.parametrize("sha_tool", ["failure", "unavailable"])
def test_installer_refuses_when_checksum_computation_is_unavailable_or_fails(
    tmp_path: Path, sha_tool: str
) -> None:
    result = _run_installer(tmp_path, "{digest}\n", sha_tool=sha_tool)

    assert result.returncode != 0
    assert "Could not compute SHA-256" in result.stderr
    assert not (tmp_path / "install" / "biomcp").exists()


def test_installer_uses_openssl_when_other_checksum_tools_are_unavailable(tmp_path: Path) -> None:
    result = _run_installer(tmp_path, "{digest}\n", sha_tool="openssl")

    assert result.returncode == 0, result.stderr
    assert (tmp_path / "install" / "biomcp").exists()


@pytest.mark.parametrize("downloader", ["curl", "wget"])
def test_installer_handles_checksum_download_failures_for_both_downloaders(
    tmp_path: Path, downloader: str
) -> None:
    result = _run_installer(tmp_path, None, downloader=downloader)

    assert result.returncode != 0
    assert "Could not download release checksum" in result.stderr
    assert not (tmp_path / "install" / "biomcp").exists()
