from __future__ import annotations

import hashlib
import importlib.util
import json
import sys
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
signing = _module("signing", "release/signing.py")
mcpb_sign = _module("mcpb_sign", "release/mcpb_sign.py")
mcpb = _module("release_mcpb", "release/mcpb.py")


def _bundle(
    tmp_path: Path,
    version: str = "1.2.3",
    macos: bytes = b"universal Mach-O",
    windows: bytes = b"MZ signed PE",
) -> tuple[Path, bytes, bytes]:
    manifest = mcpb.render_manifest(
        json.loads((ROOT / "manifest.json").read_text()), version
    )
    bundle = tmp_path / f"biomcp-{version}.mcpb"
    with zipfile.ZipFile(bundle, "w") as archive:
        for name, data, mode in (
            ("manifest.json", candidate.canonical_bytes(manifest), 0o100644),
            ("server/biomcp", macos, 0o100755),
            ("server/biomcp.exe", windows, 0o100755),
        ):
            info = zipfile.ZipInfo(name)
            info.external_attr = mode << 16
            archive.writestr(info, data)
    return bundle, macos, windows


def _policy(tmp_path: Path, *, fixture: bool = False, mcpb_identity: bool = True) -> tuple[Path, dict, str]:
    fingerprint = "A" * 64
    value = {
        "schema_version": 2,
        "enabled": True,
        **({"fixture_only": True} if fixture else {}),
        "apple": {
            "team_id": "ABCDEFGHIJ",
            "identity": "Developer ID Application: Example",
            "leaf_sha256": fingerprint,
            "notary_profile": "biomcp-release",
            "notary_service": "https://appstoreconnect.apple.com",
            "network_destinations": ["https://appstoreconnect.apple.com"],
        },
        "windows": {
            "publisher": "CN=Example",
            "leaf_sha256": "B" * 64,
            "timestamp_url": "https://timestamp.example.com",
            "timestamp_policy_oid": "1.2.3.4",
        },
        "mcpb": (
            {"subject": "CN=Example MCPB", "leaf_sha256": "C" * 64}
            if mcpb_identity
            else None
        ),
        "development_unsigned_mcpb": {
            "enabled": True,
            "package": "@anthropic-ai/mcpb",
            "tool_version": "2.1.2",
            "reason": "private development desktop testing",
            "blocks_promotion": True,
        },
        "allowed_notary_warnings": [],
    }
    path = tmp_path / "policy.json"
    path.write_bytes(candidate.canonical_bytes(value))
    return path, value, candidate.sha256_file(path)


def _candidate_base(
    tmp_path: Path,
    policy_hash: str,
    *,
    development: bool = True,
) -> tuple[Path, dict]:
    value = {
        "schema_version": 2,
        "source_sha": "a" * 40,
        "version": "0.9.0-dev.1" if development else "0.9.0",
        "python_version": "0.9.0.dev1" if development else "0.9.0",
        "candidate_kind": "development" if development else "release",
        "stage_run_id": "42",
        "status": "staging",
        "created_at": "2026-08-15T12:00:00Z",
        "gates": {name: "passed" for name in candidate.REQUIRED_GATES},
        "pins": {"mcpb": "2.1.2"},
        "signing_policy_sha256": policy_hash,
        "artifacts": {},
    }
    candidate.validate_manifest(value)
    path = tmp_path / "candidate-manifest.json"
    path.write_bytes(candidate.canonical_bytes(value))
    return path, value


def _github_context(monkeypatch: pytest.MonkeyPatch, manifest: dict) -> None:
    values = {
        "GITHUB_REPOSITORY": "genomoncology/biomcp",
        "GITHUB_WORKFLOW_REF": "genomoncology/biomcp/.github/workflows/release.yml@refs/heads/main",
        "GITHUB_JOB": "mcpb-artifact",
        "GITHUB_RUN_ID": manifest["stage_run_id"],
        "GITHUB_RUN_ATTEMPT": "3",
        "GITHUB_SHA": manifest["source_sha"],
    }
    for name, value in values.items():
        monkeypatch.setenv(name, value)


def _native_record(
    path: Path,
    artifact_id: str,
    binary: Path,
    manifest: dict,
    policy: dict,
    policy_hash: str,
) -> Path:
    macos = "macos" in artifact_id
    slug = (
        "macos-arm64"
        if artifact_id.endswith("arm64")
        else "macos-x86_64" if macos else "windows-x86_64"
    )
    section = policy["apple" if macos else "windows"]
    signing_evidence = {
        "schema_version": 1,
        "target": slug,
        "source_sha": manifest["source_sha"],
        "version": manifest["version"],
        "stage_run_id": manifest["stage_run_id"],
        "unsigned_sha256": "d" * 64,
        "signed_sha256": candidate.sha256_file(binary),
        "signing_policy_sha256": policy_hash,
        "signing_job_id": "signed-artifacts",
        "fixture_only": False,
        "certificate_fingerprint": section["leaf_sha256"],
        "timestamp_verified": True,
        "chain_verified": True,
    }
    if macos:
        signing_evidence.update(
            {
                "team_id": section["team_id"],
                "hardened_runtime": True,
                "notary_status": "Accepted",
                "notary_warnings": [],
                "notary_submission_id": "native-submission",
                "notary_log_sha256": "e" * 64,
            }
        )
    else:
        signing_evidence.update(
            {
                "publisher": section["publisher"],
                "timestamp_authority": section["timestamp_url"],
                "timestamp_policy_oid": section["timestamp_policy_oid"],
            }
        )
    kind, target = candidate.ARTIFACTS[artifact_id]
    value = {
        "id": artifact_id,
        "kind": kind,
        "target": target,
        "filename": f"{artifact_id}.archive",
        "sha256": hashlib.sha256(artifact_id.encode()).hexdigest(),
        "bytes": 100,
        "source_sha": manifest["source_sha"],
        "version": manifest["version"],
        "stage_run_id": manifest["stage_run_id"],
        "provenance": {"build_count": 1},
        "evidence": {
            "binary_sha256": candidate.sha256_file(binary),
            "signing": {"biomcp": signing_evidence},
        },
    }
    path.write_bytes(candidate.canonical_bytes(value))
    return path


def _development_record_inputs(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> dict[str, Path]:
    policy_path, policy, policy_hash = _policy(tmp_path, mcpb_identity=False)
    manifest_path, manifest = _candidate_base(tmp_path, policy_hash)
    _github_context(monkeypatch, manifest)
    arm = tmp_path / "macos-arm64"
    intel = tmp_path / "macos-x86_64"
    windows = tmp_path / "biomcp.exe"
    arm.write_bytes(b"signed arm64")
    intel.write_bytes(b"signed x86_64")
    windows.write_bytes(b"signed Windows")
    bundle, macos_bytes, _ = _bundle(
        tmp_path,
        manifest["version"],
        b"signed universal macOS",
        windows.read_bytes(),
    )
    universal = {
        "schema_version": 1,
        "target": "macos-universal",
        "source_sha": manifest["source_sha"],
        "version": manifest["version"],
        "stage_run_id": manifest["stage_run_id"],
        "unsigned_sha256": "f" * 64,
        "signed_sha256": hashlib.sha256(macos_bytes).hexdigest(),
        "signing_policy_sha256": policy_hash,
        "signing_job_id": "mcpb-artifact",
        "fixture_only": False,
        "certificate_fingerprint": policy["apple"]["leaf_sha256"],
        "team_id": policy["apple"]["team_id"],
        "hardened_runtime": True,
        "timestamp_verified": True,
        "chain_verified": True,
        "notary_status": "Accepted",
        "notary_warnings": [],
        "notary_submission_id": "universal-submission",
        "notary_log_sha256": "9" * 64,
    }
    universal_path = tmp_path / "universal-signing.json"
    universal_path.write_bytes(candidate.canonical_bytes(universal))
    outer_path = tmp_path / "mcpb-signing.json"
    mcpb_sign.attest_unsigned_development(
        bundle,
        outer_path,
        manifest,
        policy,
        policy_hash,
        fixture=False,
    )
    return {
        "bundle": bundle,
        "record_path": tmp_path / "mcpb.json",
        "manifest_path": manifest_path,
        "policy_path": policy_path,
        "outer_evidence_path": outer_path,
        "universal_signing_path": universal_path,
        "macos_arm_record_path": _native_record(
            tmp_path / "arm.json",
            "native-macos-arm64",
            arm,
            manifest,
            policy,
            policy_hash,
        ),
        "macos_intel_record_path": _native_record(
            tmp_path / "intel.json",
            "native-macos-x86_64",
            intel,
            manifest,
            policy,
            policy_hash,
        ),
        "windows_record_path": _native_record(
            tmp_path / "windows.json",
            "native-windows-x86_64",
            windows,
            manifest,
            policy,
            policy_hash,
        ),
        "macos_arm_binary": arm,
        "macos_intel_binary": intel,
        "windows_binary": windows,
    }


def test_manifest_is_v03_seven_tools_and_exact_platform_selection() -> None:
    manifest = mcpb.render_manifest(json.loads((ROOT / "manifest.json").read_text()), "1.2.3")
    assert manifest["manifest_version"] == "0.3"
    assert manifest["server"]["mcp_config"]["command"] == "server/biomcp"
    assert manifest["server"]["mcp_config"]["platform_overrides"]["win32"] == {
        "command": "server/biomcp.exe"
    }
    assert len(manifest["tools"]) == 7
    assert manifest["compatibility"]["platforms"] == ["darwin", "win32"]


def test_bundle_inspection_pins_members_modes_and_executable_hashes(tmp_path: Path) -> None:
    bundle, macos, windows = _bundle(tmp_path)
    evidence = mcpb.inspect_bundle(
        bundle, hashlib.sha256(macos).hexdigest(), hashlib.sha256(windows).hexdigest(), "1.2.3"
    )
    assert evidence["members"] == ["manifest.json", "server/biomcp", "server/biomcp.exe"]
    assert evidence["inspected"] is True


def test_bundle_rejects_wrong_hash_and_linux_claim(tmp_path: Path) -> None:
    bundle, macos, windows = _bundle(tmp_path)
    with pytest.raises(mcpb.McpbError, match="hash mismatch"):
        mcpb.inspect_bundle(bundle, "0" * 64, hashlib.sha256(windows).hexdigest(), "1.2.3")
    manifest = json.loads((ROOT / "manifest.json").read_text())
    manifest["compatibility"]["platforms"].append("linux")
    with pytest.raises(mcpb.McpbError, match="only macOS and Windows"):
        mcpb.render_manifest(manifest, "1.2.3")


def test_fixture_signature_is_post_pack_and_cannot_register_as_production(tmp_path: Path) -> None:
    bundle, _, _ = _bundle(tmp_path)
    signed = tmp_path / "signed.mcpb"
    evidence = mcpb_sign.fixture_sign(bundle, signed, "A" * 64)
    assert signed.read_bytes().startswith(bundle.read_bytes())
    assert evidence["unsigned_sha256"] == candidate.sha256_file(bundle)
    assert evidence["signed_sha256"] == candidate.sha256_file(signed)
    assert evidence["fixture_only"] is True


def test_unsigned_development_attestation_binds_candidate_policy_job_and_bytes(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    _, policy, policy_hash = _policy(tmp_path, mcpb_identity=False)
    _, manifest = _candidate_base(tmp_path, policy_hash)
    _github_context(monkeypatch, manifest)
    bundle = tmp_path / f"biomcp-{manifest['version']}.mcpb"
    bundle.write_bytes(b"one packed development archive")
    original = bundle.read_bytes()
    evidence_path = tmp_path / "unsigned-attestation.json"

    evidence = mcpb_sign.attest_unsigned_development(
        bundle,
        evidence_path,
        manifest,
        policy,
        policy_hash,
        fixture=False,
    )

    assert bundle.read_bytes() == original
    assert evidence_path.read_bytes() == candidate.canonical_bytes(evidence)
    assert evidence["archive_sha256"] == candidate.sha256_file(bundle)
    assert evidence["candidate_kind"] == "development"
    assert evidence["outer_signature_status"] == "unsigned-development"
    assert evidence["non_promotable"] is True
    assert evidence["github"] == {
        "repository": "genomoncology/biomcp",
        "workflow_ref": "genomoncology/biomcp/.github/workflows/release.yml@refs/heads/main",
        "job": "mcpb-artifact",
        "run_id": "42",
        "run_attempt": "3",
        "source_sha": "a" * 40,
    }


def test_production_attestation_reuses_protected_policy_verification(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    policy_path, _, policy_hash = _policy(tmp_path, mcpb_identity=False)
    manifest_path, manifest = _candidate_base(tmp_path, policy_hash)
    _github_context(monkeypatch, manifest)
    bundle = tmp_path / f"biomcp-{manifest['version']}.mcpb"
    bundle.write_bytes(b"packed archive")
    evidence = tmp_path / "evidence.json"
    calls = []
    monkeypatch.setattr(
        mcpb_sign,
        "verify_protected_policy",
        lambda repo, source_sha, policy, digest: calls.append(
            (repo, source_sha, policy, digest)
        ),
    )
    monkeypatch.setattr(
        sys,
        "argv",
        [
            "mcpb_sign.py",
            "--attest-development",
            "--repo",
            str(tmp_path),
            "--manifest",
            str(manifest_path),
            "--policy",
            str(policy_path),
            "--source",
            str(bundle),
            "--evidence",
            str(evidence),
        ],
    )

    assert mcpb_sign.main() == 0
    assert calls == [(tmp_path, manifest["source_sha"], policy_path, policy_hash)]
    assert json.loads(evidence.read_text())["fixture_only"] is False


@pytest.mark.parametrize(
    ("field", "value"),
    [
        ("GITHUB_REPOSITORY", "attacker/biomcp"),
        ("GITHUB_WORKFLOW_REF", "genomoncology/biomcp/.github/workflows/other.yml@main"),
        ("GITHUB_JOB", "another-job"),
        ("GITHUB_RUN_ID", "43"),
        ("GITHUB_RUN_ATTEMPT", "0"),
        ("GITHUB_SHA", "b" * 40),
    ],
)
def test_unsigned_development_attestation_rejects_mismatched_job_context(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    field: str,
    value: str,
) -> None:
    _, policy, policy_hash = _policy(tmp_path, mcpb_identity=False)
    _, manifest = _candidate_base(tmp_path, policy_hash)
    _github_context(monkeypatch, manifest)
    monkeypatch.setenv(field, value)
    bundle = tmp_path / f"biomcp-{manifest['version']}.mcpb"
    bundle.write_bytes(b"archive")
    evidence = tmp_path / "evidence.json"

    with pytest.raises(signing.SigningError, match="job context"):
        mcpb_sign.attest_unsigned_development(
            bundle, evidence, manifest, policy, policy_hash, fixture=False
        )
    assert not evidence.exists()


def test_unsigned_attestation_rejects_stable_disabled_stale_and_duplicate(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    _, policy, policy_hash = _policy(tmp_path, mcpb_identity=False)
    _, manifest = _candidate_base(tmp_path, policy_hash)
    _github_context(monkeypatch, manifest)
    bundle = tmp_path / f"biomcp-{manifest['version']}.mcpb"
    bundle.write_bytes(b"archive")
    evidence = tmp_path / "evidence.json"

    stable = {**manifest, "version": "0.9.0", "python_version": "0.9.0", "candidate_kind": "release"}
    with pytest.raises(signing.SigningError, match="development candidate"):
        mcpb_sign.attest_unsigned_development(
            bundle, evidence, stable, policy, policy_hash, fixture=False
        )
    disabled = json.loads(json.dumps(policy))
    disabled["development_unsigned_mcpb"]["enabled"] = False
    with pytest.raises(signing.SigningError, match="exception is disabled"):
        mcpb_sign.attest_unsigned_development(
            bundle, evidence, manifest, disabled, policy_hash, fixture=False
        )
    with pytest.raises(signing.SigningError, match="policy hash mismatch"):
        mcpb_sign.attest_unsigned_development(
            bundle, evidence, manifest, policy, "0" * 64, fixture=False
        )

    mcpb_sign.attest_unsigned_development(
        bundle, evidence, manifest, policy, policy_hash, fixture=False
    )
    with pytest.raises(signing.SigningError, match="duplicate"):
        mcpb_sign.attest_unsigned_development(
            bundle, evidence, manifest, policy, policy_hash, fixture=False
        )


def test_development_record_binds_inner_signatures_and_marks_outer_non_promotable(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    inputs = _development_record_inputs(tmp_path, monkeypatch)

    record = mcpb.record_bundle(**inputs)

    assert record["evidence"]["outer_signature_status"] == "unsigned-development"
    assert record["evidence"]["non_promotable"] is True
    assert record["evidence"]["outer"]["fixture_only"] is False
    assert set(record["upstream"]) == {
        "native-macos-arm64",
        "native-macos-x86_64",
        "native-windows-x86_64",
    }
    assert inputs["record_path"].read_bytes() == candidate.canonical_bytes(record)


def test_stable_outer_signature_validation_remains_mandatory(tmp_path: Path) -> None:
    _, policy, policy_hash = _policy(tmp_path)
    _, manifest = _candidate_base(tmp_path, policy_hash, development=False)
    bundle, _, _ = _bundle(tmp_path, manifest["version"])
    evidence = {
        "schema_version": 1,
        "unsigned_sha256": "d" * 64,
        "signed_sha256": candidate.sha256_file(bundle),
        "certificate_fingerprint": policy["mcpb"]["leaf_sha256"],
        "certificate_subject": policy["mcpb"]["subject"],
        "chain_verified": True,
        "eku": "codeSigning",
        "signing_policy_sha256": policy_hash,
        "source_sha": manifest["source_sha"],
        "version": manifest["version"],
        "python_version": manifest["python_version"],
        "candidate_kind": "release",
        "stage_run_id": manifest["stage_run_id"],
        "signing_job_id": "mcpb-artifact",
        "fixture_only": False,
    }
    evidence_path = tmp_path / "stable-signing.json"
    evidence_path.write_bytes(candidate.canonical_bytes(evidence))

    validated, status, non_promotable = mcpb._validate_outer_evidence(
        evidence_path, bundle, manifest, policy, policy_hash
    )
    assert validated == evidence
    assert status == "signed"
    assert non_promotable is False

    evidence["chain_verified"] = False
    evidence_path.write_bytes(candidate.canonical_bytes(evidence))
    with pytest.raises(mcpb.McpbError, match="signature evidence"):
        mcpb._validate_outer_evidence(
            evidence_path, bundle, manifest, policy, policy_hash
        )


@pytest.mark.parametrize(
    ("input_name", "json_path", "replacement", "message"),
    [
        ("universal_signing_path", ("source_sha",), "b" * 40, "universal macOS"),
        (
            "windows_record_path",
            ("evidence", "signing", "biomcp", "stage_run_id"),
            "43",
            "native signing evidence",
        ),
        (
            "macos_arm_record_path",
            ("evidence", "signing", "biomcp", "signed_sha256"),
            "0" * 64,
            "native signing evidence",
        ),
    ],
)
def test_mcpb_record_rejects_swapped_or_stale_inner_evidence_without_output(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    input_name: str,
    json_path: tuple[str, ...],
    replacement: str,
    message: str,
) -> None:
    inputs = _development_record_inputs(tmp_path, monkeypatch)
    path = inputs[input_name]
    value = json.loads(path.read_text())
    target = value
    for key in json_path[:-1]:
        target = target[key]
    target[json_path[-1]] = replacement
    path.write_bytes(candidate.canonical_bytes(value))

    with pytest.raises(mcpb.McpbError, match=message):
        mcpb.record_bundle(**inputs)
    assert not inputs["record_path"].exists()


def test_mcpb_record_rehashes_archive_after_unsigned_attestation(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    inputs = _development_record_inputs(tmp_path, monkeypatch)
    with inputs["bundle"].open("ab") as bundle:
        bundle.write(b"mutated after attestation")

    with pytest.raises(mcpb.McpbError, match="attestation"):
        mcpb.record_bundle(**inputs)
    assert not inputs["record_path"].exists()


def test_mcpb_tool_install_is_exact_version_and_integrity_pinned() -> None:
    script = (ROOT / "release/install-mcpb-tool.sh").read_text()
    assert "version=2.1.2" in script
    assert "sha512-goRbBC8ySo7SWb7tRzr+tL6FxDc4JPTRCdgfD2omba7freofvjq5rom1lBnYHZHo6Mizs1jAHJeN53aZbDoy8A==" in script
    assert "npm install --global --ignore-scripts" in script
    assert "mcpb --version" in script
