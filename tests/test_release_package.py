from __future__ import annotations

import importlib.util
import sys
import tarfile
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def _module(name: str, relative: str):
    spec = importlib.util.spec_from_file_location(name, ROOT / relative)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


candidate = _module("candidate", "release/candidate.py")
packaging = _module("release_package", "release/package.py")
inspection = _module("release_inspect", "release/inspect.py")


def test_native_archive_is_deterministic_minimal_and_executable(tmp_path: Path) -> None:
    binary = tmp_path / "input"
    binary.write_bytes(b"full executable")
    first = tmp_path / "one.tar.gz"
    second = tmp_path / "two.tar.gz"
    packaging.native_archive(binary, first, False)
    packaging.native_archive(binary, second, False)
    assert first.read_bytes() == second.read_bytes()
    assert inspection.inspect_native(first, False)["executable_count"] == 1
    with tarfile.open(first, "r:gz") as archive:
        member = archive.getmembers()[0]
        assert member.name == "biomcp"
        assert member.mode == 0o755


def test_windows_native_archive_contains_only_executable(tmp_path: Path) -> None:
    binary = tmp_path / "biomcp.exe"
    binary.write_bytes(b"MZ full")
    archive = tmp_path / "biomcp-windows-x86_64.zip"
    packaging.native_archive(binary, archive, True)
    assert inspection.inspect_native(archive, True)["archive_members"] == 1


def test_wheel_contains_full_binary_and_small_shim_once(tmp_path: Path) -> None:
    full = tmp_path / "biomcp"
    shim = tmp_path / "biomcp-cli"
    full.write_bytes(b"full executable bytes")
    shim.write_bytes(b"shim")
    wheel = tmp_path / "biomcp_cli-1.2.3-py3-none-manylinux_2_28_x86_64.whl"
    packaging.wheel(full, shim, wheel, "1.2.3", "py3-none-manylinux_2_28_x86_64", False)
    evidence = inspection.inspect_wheel(wheel, False)
    assert evidence["executable_count"] == 2
    with zipfile.ZipFile(wheel) as archive:
        assert sum(name.endswith("/biomcp") for name in archive.namelist()) == 1
        assert sum(name.endswith("/biomcp-cli") for name in archive.namelist()) == 1
        assert not any("testdata" in name or "sdlc" in name for name in archive.namelist())


def test_artifact_record_binds_bytes_and_candidate_identity(tmp_path: Path) -> None:
    artifact = tmp_path / "biomcp-linux-x86_64.tar.gz"
    artifact.write_bytes(b"artifact")
    record = packaging.record(
        "native-linux-x86_64", artifact, "a" * 40, "1.2.3", "42", {"inspected": True}
    )
    assert record["filename"] == artifact.name
    assert record["bytes"] == 8
    assert record["source_sha"] == "a" * 40
    assert record["provenance"]["build_count"] == 1
