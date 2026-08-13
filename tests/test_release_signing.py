from __future__ import annotations

import hashlib
import importlib.util
import json
import subprocess
import sys
from pathlib import Path

import jsonschema
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
signing = _module("release_signing", "release/signing.py")


def _fixture_policy(tmp_path: Path) -> Path:
    fingerprint = "A" * 64
    policy = {
        "schema_version": 1,
        "enabled": True,
        "fixture_only": True,
        "apple": {
            "team_id": "FIXTURE123",
            "identity": "Fixture Developer ID",
            "leaf_sha256": fingerprint,
            "notary_profile": "fixture",
            "notary_service": "https://appstoreconnect.apple.com",
            "network_destinations": ["https://appstoreconnect.apple.com"],
        },
        "windows": {
            "publisher": "Fixture Publisher",
            "leaf_sha256": fingerprint,
            "timestamp_url": "https://timestamp.example.test",
            "timestamp_policy_oid": "1.2.3.4",
        },
        "mcpb": {"subject": "Fixture MCPB", "leaf_sha256": fingerprint},
        "allowed_notary_warnings": [],
    }
    path = tmp_path / "policy.json"
    path.write_text(json.dumps(policy))
    return path


def test_committed_production_policy_is_valid_explicitly_disabled_and_secret_free() -> None:
    policy_path = ROOT / "release/signing-policy.json"
    policy = json.loads(policy_path.read_text())
    schema = json.loads((ROOT / "release/signing-policy.schema.json").read_text())
    jsonschema.Draft202012Validator(schema).validate(policy)
    assert policy == {
        "schema_version": 1,
        "enabled": False,
        "apple": None,
        "windows": None,
        "mcpb": None,
        "allowed_notary_warnings": [],
    }
    assert not any(word in policy_path.read_text().lower() for word in ("password", "private_key", "token"))


def test_production_policy_fails_closed_before_tools_or_network() -> None:
    with pytest.raises(signing.SigningError, match="not provisioned"):
        signing.load_policy(ROOT / "release/signing-policy.json", fixture=False)


def test_fixture_policy_is_rejected_by_production_and_accepted_only_explicitly(tmp_path: Path) -> None:
    path = _fixture_policy(tmp_path)
    with pytest.raises(signing.SigningError, match="rejects fixture"):
        signing.load_policy(path, fixture=False)
    policy, digest = signing.load_policy(path, fixture=True)
    assert policy["fixture_only"]
    assert len(digest) == 64


@pytest.mark.parametrize("target", ["macos-x86_64", "macos-arm64", "macos-universal", "windows-x86_64"])
def test_fixture_finalization_binds_unsigned_and_signed_bytes(tmp_path: Path, target: str) -> None:
    policy_path = _fixture_policy(tmp_path)
    policy, digest = signing.load_policy(policy_path, fixture=True)
    source = tmp_path / "unsigned"
    output = tmp_path / "signed"
    source.write_bytes(b"fixture executable")
    evidence = signing.fixture_finalize(
        source, output, target, "a" * 40, "1.2.3", digest, policy
    )
    assert evidence["unsigned_sha256"] == hashlib.sha256(source.read_bytes()).hexdigest()
    assert evidence["signed_sha256"] == candidate.sha256_file(output)
    assert evidence["fixture_only"] is True
    assert evidence["timestamp_verified"] is True
    if target.startswith("macos"):
        assert evidence["notary_status"] == "Accepted"
        assert evidence["notary_warnings"] == []


def test_wrong_protected_digest_and_release_commit_policy_change_fail(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    repo = tmp_path / "repo"
    (repo / "release").mkdir(parents=True)
    policy = repo / "release/signing-policy.json"
    policy.write_text('{"first":true}\n')
    subprocess.run(["git", "init", "-q"], cwd=repo, check=True)
    subprocess.run(["git", "config", "user.name", "Fixture"], cwd=repo, check=True)
    subprocess.run(["git", "config", "user.email", "fixture@example.test"], cwd=repo, check=True)
    subprocess.run(["git", "add", "."], cwd=repo, check=True)
    subprocess.run(["git", "commit", "-qm", "policy"], cwd=repo, check=True)
    policy.write_text('{"changed":true}\n')
    subprocess.run(["git", "add", "."], cwd=repo, check=True)
    subprocess.run(["git", "commit", "-qm", "release"], cwd=repo, check=True)
    sha = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repo, text=True).strip()
    digest = hashlib.sha256(policy.read_bytes()).hexdigest()
    monkeypatch.setenv("BIOMCP_SIGNING_POLICY_SHA256", "0" * 64)
    with pytest.raises(signing.SigningError, match="digest mismatch"):
        signing.verify_protected_policy(repo, sha, policy, digest)
    monkeypatch.setenv("BIOMCP_SIGNING_POLICY_SHA256", digest)
    with pytest.raises(signing.SigningError, match="changed"):
        signing.verify_protected_policy(repo, sha, policy, digest)


def test_duplicate_output_is_refused_before_signing(tmp_path: Path) -> None:
    source = tmp_path / "source"
    output = tmp_path / "output"
    evidence = tmp_path / "evidence.json"
    source.write_bytes(b"unsigned")
    output.write_bytes(b"existing signed bytes")
    result = subprocess.run(
        [
            sys.executable,
            str(ROOT / "release/signing.py"),
            "--fixture",
            "--policy",
            str(_fixture_policy(tmp_path)),
            "--source",
            str(source),
            "--output",
            str(output),
            "--evidence",
            str(evidence),
            "--target",
            "windows-x86_64",
            "--source-sha",
            "a" * 40,
            "--version",
            "1.2.3",
            "--unsigned-sha256",
            hashlib.sha256(source.read_bytes()).hexdigest(),
        ],
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode != 0
    assert "refusing duplicate signing" in result.stderr
    assert not evidence.exists()
