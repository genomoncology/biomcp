from __future__ import annotations

import hashlib
import importlib.util
import json
import subprocess
import sys
import tarfile
import zipfile
from pathlib import Path

import pytest

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
release_smoke = _module("release_smoke", "release/smoke.py")


def _sbom(path: Path, source_sha: str = "a" * 40, version: str = "1.2.3") -> Path:
    path.write_text(
        json.dumps(
            {
                "bomFormat": "CycloneDX",
                "specVersion": "1.6",
                "version": 1,
                "metadata": {
                    "component": {"name": "biomcp-cli", "version": version},
                    "source_sha": source_sha,
                },
                "components": [],
            }
        )
    )
    return path


def _lock(path: Path) -> Path:
    path.write_text("")
    return path


def test_native_archive_is_deterministic_minimal_and_executable(tmp_path: Path) -> None:
    binary = tmp_path / "input"
    binary.write_bytes(b"full executable")
    first = tmp_path / "biomcp-linux-x86_64.tar.gz"
    second = tmp_path / "two.tar.gz"
    packaging.native_archive(binary, first, False)
    packaging.native_archive(binary, second, False)
    assert first.read_bytes() == second.read_bytes()
    assert (
        inspection.inspect_native(first, "x86_64-unknown-linux-gnu")["executable_count"]
        == 1
    )
    with tarfile.open(first, "r:gz") as archive:
        member = archive.getmembers()[0]
        assert member.name == "biomcp"
        assert member.mode == 0o755


def test_windows_native_archive_contains_only_executable(tmp_path: Path) -> None:
    binary = tmp_path / "biomcp.exe"
    binary.write_bytes(b"MZ full")
    archive = tmp_path / "biomcp-windows-x86_64.zip"
    packaging.native_archive(binary, archive, True)
    assert (
        inspection.inspect_native(archive, "x86_64-pc-windows-msvc")["archive_members"]
        == 1
    )


def test_wheel_contains_full_binary_and_small_shim_once(tmp_path: Path) -> None:
    full = tmp_path / "biomcp"
    shim = tmp_path / "biomcp-cli"
    full.write_bytes(b"full executable bytes")
    shim.write_bytes(b"shim")
    wheel = tmp_path / "biomcp_cli-1.2.3-py3-none-manylinux_2_28_x86_64.whl"
    packaging.wheel(full, shim, wheel, "1.2.3", "py3-none-manylinux_2_28_x86_64", False)
    evidence = inspection.inspect_wheel(wheel, "wheel-linux-x86_64", "1.2.3")
    assert evidence["executable_count"] == 2
    with zipfile.ZipFile(wheel) as archive:
        assert sum(name.endswith("/biomcp") for name in archive.namelist()) == 1
        assert sum(name.endswith("/biomcp-cli") for name in archive.namelist()) == 1
        assert not any(
            "testdata" in name or "sdlc" in name for name in archive.namelist()
        )


def test_development_wheel_uses_exact_pep440_identity(tmp_path: Path) -> None:
    full = tmp_path / "biomcp"
    shim = tmp_path / "biomcp-cli"
    full.write_bytes(b"full executable bytes")
    shim.write_bytes(b"shim")
    wheel = tmp_path / "biomcp_cli-0.9.0.dev1-py3-none-manylinux_2_28_x86_64.whl"

    packaging.wheel(
        full,
        shim,
        wheel,
        "0.9.0.dev1",
        "py3-none-manylinux_2_28_x86_64",
        False,
    )

    evidence = inspection.inspect_wheel(
        wheel, "wheel-linux-x86_64", "0.9.0.dev1"
    )
    assert evidence["python_version"] == "0.9.0.dev1"
    with zipfile.ZipFile(wheel) as archive:
        metadata = archive.read("biomcp_cli-0.9.0.dev1.dist-info/METADATA")
    assert b"Version: 0.9.0.dev1\n" in metadata


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


@pytest.mark.parametrize(
    "member, old, new",
    [
        ("METADATA", b"Version: 1.2.3", b"Version: 9.9.9"),
        (
            "WHEEL",
            b"Tag: py3-none-manylinux_2_28_x86_64",
            b"Tag: py3-none-manylinux_2_28_aarch64",
        ),
    ],
)
def test_wheel_identity_metadata_and_record_are_fully_validated(
    tmp_path: Path, member: str, old: bytes, new: bytes
) -> None:
    full = tmp_path / "biomcp"
    shim = tmp_path / "biomcp-cli"
    full.write_bytes(b"full executable bytes")
    shim.write_bytes(b"shim")
    wheel = tmp_path / "biomcp_cli-1.2.3-py3-none-manylinux_2_28_x86_64.whl"
    packaging.wheel(full, shim, wheel, "1.2.3", "py3-none-manylinux_2_28_x86_64", False)
    with zipfile.ZipFile(wheel) as archive:
        entries = {
            info.filename: (info, archive.read(info)) for info in archive.infolist()
        }
    target_name = next(name for name in entries if name.endswith(f"/{member}"))
    info, data = entries[target_name]
    entries[target_name] = (info, data.replace(old, new))
    record_name = next(name for name in entries if name.endswith("/RECORD"))
    rows = [
        f"{name},{packaging._record_hash(data)},{len(data)}\n"
        for name, (_, data) in entries.items()
        if name != record_name
    ]
    record_info, _ = entries[record_name]
    entries[record_name] = (
        record_info,
        ("".join(rows) + f"{record_name},,\n").encode(),
    )
    with zipfile.ZipFile(wheel, "w") as archive:
        for info, data in entries.values():
            archive.writestr(info, data)

    with pytest.raises(inspection.InspectionError):
        inspection.inspect_wheel(wheel, "wheel-linux-x86_64", "1.2.3")


def test_wheel_duplicate_member_is_rejected(tmp_path: Path) -> None:
    full = tmp_path / "biomcp"
    shim = tmp_path / "biomcp-cli"
    full.write_bytes(b"full executable bytes")
    shim.write_bytes(b"shim")
    wheel = tmp_path / "biomcp_cli-1.2.3-py3-none-manylinux_2_28_x86_64.whl"
    packaging.wheel(full, shim, wheel, "1.2.3", "py3-none-manylinux_2_28_x86_64", False)
    with pytest.warns(UserWarning, match="Duplicate"):
        with zipfile.ZipFile(wheel, "a") as archive:
            archive.writestr("biomcp_cli-1.2.3.dist-info/METADATA", b"duplicate")

    with pytest.raises(inspection.InspectionError, match="duplicate"):
        inspection.inspect_wheel(wheel, "wheel-linux-x86_64", "1.2.3")


def test_smoke_requires_exact_structured_identity(tmp_path: Path) -> None:
    fake = tmp_path / "fake-biomcp"
    fake.write_text(
        "#!/usr/bin/env python3\n"
        "import json, sys\n"
        "if sys.argv[1:] == ['--json', 'version']:\n"
        " print(json.dumps({'version':'9.9.9+gdeadbeef','git_revision':'deadbeef',"
        "'note':'1.2.3 aaaaaaaa'}))\n"
        "elif sys.argv[1:] == ['--json', 'not-a-command']:\n"
        " print('{}'); sys.exit(2)\n"
        "elif sys.argv[1:2] == ['--json']:\n"
        " print('{}')\n"
        "else:\n"
        " print('ok')\n"
    )
    fake.chmod(0o755)

    with pytest.raises(inspection.InspectionError, match="identity"):
        inspection.smoke(fake, "a" * 40, "1.2.3")


def test_installed_command_smoke_requires_exact_structured_identity(
    monkeypatch,
) -> None:
    def run(arguments, **_kwargs):
        if arguments[-2:] == ["--json", "not-a-command"]:
            return subprocess.CompletedProcess(arguments, 2, "{}", "")
        if arguments[-2:] == ["--json", "version"]:
            return subprocess.CompletedProcess(
                arguments,
                0,
                json.dumps(
                    {
                        "version": "9.9.9+gdeadbeef",
                        "git_revision": "deadbeef",
                        "note": "1.2.3 aaaaaaaa",
                    }
                ),
                "",
            )
        return subprocess.CompletedProcess(arguments, 0, "{}", "")

    monkeypatch.setattr(release_smoke.subprocess, "run", run)
    with pytest.raises(release_smoke.SmokeError, match="identity"):
        release_smoke.smoke(Path("biomcp"), "1.2.3", "aaaaaaaa")


def test_final_inspector_owns_complete_wheel_evidence(
    tmp_path: Path, monkeypatch
) -> None:
    full = tmp_path / "biomcp"
    shim = tmp_path / "biomcp-cli"
    full.write_bytes(b"full executable bytes")
    shim.write_bytes(b"shim")
    wheel = tmp_path / "biomcp_cli-1.2.3-py3-none-manylinux_2_28_x86_64.whl"
    packaging.wheel(full, shim, wheel, "1.2.3", "py3-none-manylinux_2_28_x86_64", False)
    output = tmp_path / "wheel.json"
    monkeypatch.setattr(
        inspection,
        "smoke",
        lambda *_: {
            "version_help_json_smoke": True,
            "binary_sha256": hashlib.sha256(full.read_bytes()).hexdigest(),
        },
    )
    monkeypatch.setattr(
        inspection, "platform_evidence", lambda *_: {"glibc_floor_checked": True}
    )

    inspection.finalize_record(
        kind="wheel",
        artifact_id="wheel-linux-x86_64",
        artifact=wheel,
        binary=full,
        source_sha="a" * 40,
        version="1.2.3",
        python_version="1.2.3",
        run_id="42",
        shim=shim,
        sbom=_sbom(tmp_path / "sbom.cdx.json"),
        cargo_lock=_lock(tmp_path / "Cargo.lock"),
        signing_policy=ROOT / "release/signing-policy.json",
        signing_evidence={},
        provenance={"target": "x86_64-unknown-linux-gnu"},
        output=output,
    )

    evidence = json.loads(output.read_text())["evidence"]
    assert evidence["artifact_sha256"] == hashlib.sha256(wheel.read_bytes()).hexdigest()
    assert evidence["archive_members"] == 5
    assert evidence["executable_count"] == 2
    assert evidence["python_version"] == "1.2.3"
    assert evidence["version_help_json_smoke"] is True
    assert evidence["platform"] == {"glibc_floor_checked": True}
    assert (
        evidence["sbom_sha256"]
        == hashlib.sha256((tmp_path / "sbom.cdx.json").read_bytes()).hexdigest()
    )
    assert evidence["binary_sha256"] == hashlib.sha256(full.read_bytes()).hexdigest()
    assert evidence["shim_sha256"] == hashlib.sha256(shim.read_bytes()).hexdigest()
    assert evidence["shim_is_smaller"] is True


def test_failed_final_inspection_leaves_no_success_record(tmp_path: Path) -> None:
    artifact = tmp_path / "broken.tar.gz"
    artifact.write_bytes(b"not an archive")
    binary = tmp_path / "biomcp"
    binary.write_bytes(b"binary")
    output = tmp_path / "native.json"
    output.write_text('{"evidence":{"inspected":true}}\n')

    with pytest.raises((inspection.InspectionError, tarfile.TarError)):
        inspection.finalize_record(
            kind="native",
            artifact_id="native-linux-x86_64",
            artifact=artifact,
            binary=binary,
            source_sha="a" * 40,
            version="1.2.3",
            python_version="1.2.3",
            run_id="42",
            shim=None,
            sbom=_sbom(tmp_path / "sbom.cdx.json"),
            cargo_lock=_lock(tmp_path / "Cargo.lock"),
            signing_policy=ROOT / "release/signing-policy.json",
            signing_evidence={},
            provenance={"target": "x86_64-unknown-linux-gnu"},
            output=output,
        )

    assert not output.exists()


def test_tampered_sbom_leaves_no_success_record(tmp_path: Path, monkeypatch) -> None:
    binary = tmp_path / "biomcp"
    binary.write_bytes(b"binary")
    archive = tmp_path / "biomcp-linux-x86_64.tar.gz"
    packaging.native_archive(binary, archive, False)
    output = tmp_path / "native.json"
    monkeypatch.setattr(
        inspection, "platform_evidence", lambda *_: {"glibc_floor_checked": True}
    )
    monkeypatch.setattr(
        inspection,
        "smoke",
        lambda *_: {
            "version_help_json_smoke": True,
            "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
        },
    )

    with pytest.raises(inspection.InspectionError, match="SBOM"):
        inspection.finalize_record(
            kind="native",
            artifact_id="native-linux-x86_64",
            artifact=archive,
            binary=binary,
            source_sha="a" * 40,
            version="1.2.3",
            python_version="1.2.3",
            run_id="42",
            shim=None,
            sbom=_sbom(tmp_path / "sbom.cdx.json", source_sha="c" * 40),
            cargo_lock=_lock(tmp_path / "Cargo.lock"),
            signing_policy=ROOT / "release/signing-policy.json",
            signing_evidence={},
            provenance={"target": "x86_64-unknown-linux-gnu"},
            output=output,
        )
    assert not output.exists()


def test_failed_platform_reinspection_leaves_no_success_record(
    tmp_path: Path, monkeypatch
) -> None:
    binary = tmp_path / "biomcp"
    binary.write_bytes(b"binary")
    archive = tmp_path / "biomcp-linux-x86_64.tar.gz"
    packaging.native_archive(binary, archive, False)
    output = tmp_path / "native.json"
    monkeypatch.setattr(
        inspection,
        "platform_evidence",
        lambda *_: (_ for _ in ()).throw(
            inspection.InspectionError("platform mismatch")
        ),
    )

    with pytest.raises(inspection.InspectionError, match="platform mismatch"):
        inspection.finalize_record(
            kind="native",
            artifact_id="native-linux-x86_64",
            artifact=archive,
            binary=binary,
            source_sha="a" * 40,
            version="1.2.3",
            python_version="1.2.3",
            run_id="42",
            shim=None,
            sbom=_sbom(tmp_path / "sbom.cdx.json"),
            cargo_lock=_lock(tmp_path / "Cargo.lock"),
            signing_policy=ROOT / "release/signing-policy.json",
            signing_evidence={},
            provenance={"target": "x86_64-unknown-linux-gnu"},
            output=output,
        )
    assert not output.exists()


def test_tampered_signing_evidence_leaves_no_success_record(
    tmp_path: Path, monkeypatch
) -> None:
    full = tmp_path / "biomcp"
    shim = tmp_path / "biomcp-cli"
    full.write_bytes(b"signed full executable")
    shim.write_bytes(b"signed shim")
    wheel = tmp_path / "biomcp_cli-1.2.3-py3-none-macosx_14_0_x86_64.whl"
    packaging.wheel(full, shim, wheel, "1.2.3", "py3-none-macosx_14_0_x86_64", False)
    policy = tmp_path / "policy.json"
    policy.write_text(
        json.dumps(
            {
                "schema_version": 1,
                "enabled": True,
                "apple": {
                    "team_id": "ABCDEFGHIJ",
                    "identity": "Developer ID Application: Example",
                    "leaf_sha256": "A" * 64,
                    "notary_profile": "biomcp-release",
                    "notary_service": "https://appstoreconnect.apple.com",
                    "network_destinations": ["https://appstoreconnect.apple.com"],
                },
                "windows": {
                    "publisher": "CN=Example",
                    "leaf_sha256": "B" * 64,
                    "timestamp_url": "https://timestamp.example.com",
                    "timestamp_policy_oid": "1.2.3",
                },
                "mcpb": {"subject": "CN=Example", "leaf_sha256": "C" * 64},
                "allowed_notary_warnings": [],
            }
        )
    )
    policy_hash = hashlib.sha256(policy.read_bytes()).hexdigest()

    unsigned_full = tmp_path / "unsigned-biomcp"
    unsigned_shim = tmp_path / "unsigned-biomcp-cli"
    unsigned_full.write_bytes(b"unsigned full executable")
    unsigned_shim.write_bytes(b"unsigned shim")

    def signing_record(path: Path, signed_hash: str, unsigned_hash: str) -> Path:
        path.write_text(
            json.dumps(
                {
                    "schema_version": 1,
                    "target": "macos-x86_64",
                    "source_sha": "a" * 40,
                    "version": "1.2.3",
                    "unsigned_sha256": unsigned_hash,
                    "signed_sha256": signed_hash,
                    "signing_policy_sha256": policy_hash,
                    "signing_job_id": "signed-artifacts",
                    "fixture_only": False,
                    "certificate_fingerprint": "A" * 64,
                    "team_id": "ABCDEFGHIJ",
                    "hardened_runtime": True,
                    "timestamp_verified": True,
                    "chain_verified": True,
                    "notary_status": "Accepted",
                    "notary_warnings": [],
                    "notary_log_sha256": "c" * 64,
                }
            )
        )
        return path

    monkeypatch.setattr(
        inspection, "platform_evidence", lambda *_: {"deployment_target": "14.0"}
    )
    output = tmp_path / "wheel.json"
    with pytest.raises(inspection.InspectionError, match="signing evidence"):
        inspection.finalize_record(
            kind="wheel",
            artifact_id="wheel-macos-x86_64",
            artifact=wheel,
            binary=full,
            source_sha="a" * 40,
            version="1.2.3",
            python_version="1.2.3",
            run_id="42",
            shim=shim,
            sbom=_sbom(tmp_path / "sbom.cdx.json"),
            cargo_lock=_lock(tmp_path / "Cargo.lock"),
            signing_policy=policy,
            signing_evidence={
                "biomcp": signing_record(
                    tmp_path / "full-signing.json",
                    "0" * 64,
                    hashlib.sha256(unsigned_full.read_bytes()).hexdigest(),
                ),
                "biomcp-cli": signing_record(
                    tmp_path / "shim-signing.json",
                    hashlib.sha256(shim.read_bytes()).hexdigest(),
                    hashlib.sha256(unsigned_shim.read_bytes()).hexdigest(),
                ),
            },
            provenance={"target": "x86_64-apple-darwin"},
            output=output,
            unsigned_binary=unsigned_full,
            unsigned_shim=unsigned_shim,
        )
    assert not output.exists()
