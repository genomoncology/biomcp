from __future__ import annotations

import hashlib
import json
import os
import shutil
import stat
import subprocess
import tarfile
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]
INSTALLER = REPO_ROOT / "install.sh"
DOCS_INSTALLER = REPO_ROOT / "docs/install.sh"
ASSET = "biomcp-linux-x86_64.tar.gz"


def test_root_installer_is_the_canonical_deployed_copy() -> None:
    assert DOCS_INSTALLER.read_bytes() == INSTALLER.read_bytes()


def test_ci_and_release_gate_installer_identity_before_docs_or_release() -> None:
    ci = (REPO_ROOT / ".github/workflows/ci.yml").read_text(encoding="utf-8")
    release = (REPO_ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
    check = "cmp --silent install.sh docs/install.sh"
    assert check in ci
    assert ci.index(check) < ci.index("bash scripts/check-version-sync.sh")
    assert check in release
    assert release.index(check) < release.index("release-disabled.sh")


def test_public_installer_verifier_compares_deployed_bytes() -> None:
    verifier = (REPO_ROOT / "scripts/verify-public-installer.sh").read_text(
        encoding="utf-8"
    )
    assert "https://biomcp.org/install.sh" in verifier
    assert "cmp --silent" in verifier


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
    for name in ("bash", "uname", "mktemp", "rm", "basename", "tar", "gzip", "mkdir", "chmod", "mv", "head", "cp", "sync"):
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
    tmp_path: Path,
    sidecar: str | None,
    *,
    downloader: str = "curl",
    sha_tool: str = "system",
    existing_destination: str | None = None,
) -> subprocess.CompletedProcess[str]:
    fixture, _ = _fixture(tmp_path, sidecar)
    tools = _tools(tmp_path, downloader, sha_tool)
    install_dir = tmp_path / "install"
    if existing_destination is not None:
        install_dir.mkdir()
        (install_dir / "biomcp").write_text(existing_destination, encoding="utf-8")
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
    receipt = json.loads((tmp_path / "install" / "biomcp.install.json").read_text())
    assert receipt == {
        "schema_version": 1,
        "installer": "biomcp-standalone-installer",
        "state": "installed",
        "executable_path": str((tmp_path / "install" / "biomcp").resolve()),
        "version": "0.0.0",
        "sha256": hashlib.sha256((tmp_path / "install" / "biomcp").read_bytes()).hexdigest(),
    }
    assert not list((tmp_path / "install").glob(".biomcp-stage.*"))


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


def test_installer_accepts_uppercase_checksum_hex(tmp_path: Path) -> None:
    fixture, digest = _fixture(tmp_path, None)
    (fixture / "sidecar").write_text(f"{digest.upper()}\n", encoding="utf-8")
    tools = _tools(tmp_path, "curl", "system")
    install_dir = tmp_path / "install"
    scratch = tmp_path / "scratch"
    scratch.mkdir()
    result = subprocess.run(
        ["bash", str(INSTALLER)],
        text=True,
        capture_output=True,
        env=os.environ
        | {
            "BIOMCP_INSTALL_DIR": str(install_dir),
            "BIOMCP_VERSION": "0.0.0",
            "FIXTURE_DIR": str(fixture),
            "TMPDIR": str(scratch),
            "PATH": str(tools),
        },
        check=False,
    )

    assert result.returncode == 0, result.stderr
    assert (install_dir / "biomcp").exists()
    assert not list(scratch.iterdir())


def test_installer_rejects_mismatched_checksum_without_touching_existing_destination(
    tmp_path: Path,
) -> None:
    original = "known-good-existing-binary\n"
    result = _run_installer(
        tmp_path, f"{'0' * 64}\n", existing_destination=original
    )

    assert result.returncode != 0
    assert "Checksum verification failed" in result.stderr
    assert (tmp_path / "install" / "biomcp").read_text(encoding="utf-8") == original


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


def test_installer_smokes_destination_stage_before_replacing_existing_binary(tmp_path: Path) -> None:
    fixture, digest = _fixture(tmp_path, None)
    (fixture / "sidecar").write_text(f"{digest}\n")
    binary = fixture / "biomcp"
    _write_executable(binary, "#!/bin/sh\nprintf '%s\\n' \"$0\" >> \"$SMOKE_LOG\"\necho 'biomcp 0.0.0'\n")
    archive = fixture / ASSET
    with tarfile.open(archive, "w:gz") as tar:
        tar.add(binary, arcname="biomcp")
    (fixture / "sidecar").write_text(f"{hashlib.sha256(archive.read_bytes()).hexdigest()}\n")
    tools = _tools(tmp_path, "curl", "system")
    install_dir = tmp_path / "install"
    install_dir.mkdir()
    old = install_dir / "biomcp"
    _write_executable(old, "#!/bin/sh\necho 'biomcp old'\n")
    scratch = tmp_path / "scratch"
    scratch.mkdir()
    smoke_log = tmp_path / "smoke.log"
    result = subprocess.run(
        ["bash", str(INSTALLER)], text=True, capture_output=True,
        env=os.environ | {"BIOMCP_INSTALL_DIR": str(install_dir), "BIOMCP_VERSION": "0.0.0", "FIXTURE_DIR": str(fixture), "TMPDIR": str(scratch), "PATH": str(tools), "SMOKE_LOG": str(smoke_log)},
        check=False,
    )
    assert result.returncode == 0, result.stderr
    assert "/.biomcp-stage." in smoke_log.read_text()
    assert json.loads((install_dir / "biomcp.install.json").read_text())["state"] == "installed"


def test_installer_never_edits_shell_startup_files(tmp_path: Path) -> None:
    fixture, _ = _fixture(tmp_path, "{digest}\n")
    tools = _tools(tmp_path, "curl", "system")
    home = tmp_path / "home"
    home.mkdir()
    sentinels = [home / name for name in (".bashrc", ".bash_profile", ".zshrc", ".profile")]
    for path in sentinels:
        path.write_text("sentinel\n")
        path.chmod(0o640)
    before = [(path.read_bytes(), stat.S_IMODE(path.stat().st_mode)) for path in sentinels]
    scratch = tmp_path / "scratch"
    scratch.mkdir()
    result = subprocess.run(
        ["bash", str(INSTALLER)], text=True, capture_output=True,
        env=os.environ | {"HOME": str(home), "SHELL": "/bin/zsh", "BIOMCP_INSTALL_DIR": str(home / ".local/bin"), "BIOMCP_VERSION": "0.0.0", "FIXTURE_DIR": str(fixture), "TMPDIR": str(scratch), "PATH": str(tools)},
        check=False,
    )
    assert result.returncode == 0, result.stderr
    assert before == [(path.read_bytes(), stat.S_IMODE(path.stat().st_mode)) for path in sentinels]
    assert result.stderr.count("export PATH=") == 1


def test_installer_refuses_symlink_destination_without_changing_target(tmp_path: Path) -> None:
    fixture, _ = _fixture(tmp_path, "{digest}\n")
    tools = _tools(tmp_path, "curl", "system")
    install_dir = tmp_path / "install"
    install_dir.mkdir()
    target = tmp_path / "target"
    target.write_text("untouched")
    (install_dir / "biomcp").symlink_to(target)
    scratch = tmp_path / "scratch"
    scratch.mkdir()
    result = subprocess.run(
        ["bash", str(INSTALLER)], text=True, capture_output=True,
        env=os.environ | {"BIOMCP_INSTALL_DIR": str(install_dir), "BIOMCP_VERSION": "0.0.0", "FIXTURE_DIR": str(fixture), "TMPDIR": str(scratch), "PATH": str(tools)},
        check=False,
    )
    assert result.returncode != 0
    assert target.read_text() == "untouched"
