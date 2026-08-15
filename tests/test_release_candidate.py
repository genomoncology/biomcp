from __future__ import annotations

import hashlib
import importlib.util
import json
import subprocess
from pathlib import Path

import jsonschema
import pytest

ROOT = Path(__file__).resolve().parents[1]


def _module(name: str, relative: str):
    spec = importlib.util.spec_from_file_location(name, ROOT / relative)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


candidate = _module("candidate", "release/candidate.py")


def _repo(
    tmp_path: Path,
    rust_version: str = "1.2.3",
    python_version: str = "1.2.3",
) -> tuple[Path, str]:
    repo = tmp_path / "repo"
    repo.mkdir()
    (repo / "Cargo.toml").write_text(
        f'[package]\nname="fixture"\nversion = "{rust_version}"\n'
    )
    (repo / "pyproject.toml").write_text(
        f'[project]\nname="fixture"\nversion = "{python_version}"\n'
    )
    subprocess.run(["git", "init", "-q"], cwd=repo, check=True)
    subprocess.run(["git", "config", "user.name", "Fixture"], cwd=repo, check=True)
    subprocess.run(["git", "config", "user.email", "fixture@example.test"], cwd=repo, check=True)
    subprocess.run(["git", "add", "."], cwd=repo, check=True)
    subprocess.run(["git", "commit", "-qm", "fixture"], cwd=repo, check=True)
    sha = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repo, text=True).strip()
    return repo, sha


def _manifest(tmp_path: Path) -> dict:
    repo, sha = _repo(tmp_path)
    return candidate.init_manifest(repo, sha, "42", {"rust": "1.93.1"}, require_main=False)


def _record(manifest: dict, artifact_id: str, path: Path) -> dict:
    kind, target = candidate.ARTIFACTS[artifact_id]
    evidence = {"inspected": True}
    if kind == "wheel":
        evidence["python_version"] = manifest["python_version"]
    return {
        "id": artifact_id,
        "kind": kind,
        "target": target,
        "filename": path.name,
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "bytes": path.stat().st_size,
        "source_sha": manifest["source_sha"],
        "version": manifest["version"],
        "stage_run_id": manifest["stage_run_id"],
        "provenance": {"build_count": 1},
        "evidence": evidence,
    }


def _development_mcpb_outer(manifest: dict, archive_sha256: str) -> dict:
    return {
        "schema_version": 1,
        "evidence_type": "unsigned-development-mcpb",
        "archive_sha256": archive_sha256,
        "source_sha": manifest["source_sha"],
        "version": manifest["version"],
        "python_version": manifest["python_version"],
        "candidate_kind": "development",
        "stage_run_id": manifest["stage_run_id"],
        "signing_policy_sha256": manifest["signing_policy_sha256"],
        "package": "@anthropic-ai/mcpb",
        "tool_version": "2.1.2",
        "exception_reason": "private development desktop testing",
        "outer_signature_status": "unsigned-development",
        "non_promotable": True,
        "github": {
            "repository": "genomoncology/biomcp",
            "workflow_ref": "genomoncology/biomcp/.github/workflows/release.yml@refs/heads/main",
            "job": "mcpb-artifact",
            "run_id": manifest["stage_run_id"],
            "run_attempt": "1",
            "source_sha": manifest["source_sha"],
        },
        "fixture_only": False,
    }


def _stable_mcpb_outer(manifest: dict, signed_sha256: str) -> dict:
    return {
        "schema_version": 1,
        "unsigned_sha256": "e" * 64,
        "signed_sha256": signed_sha256,
        "certificate_fingerprint": "A" * 64,
        "certificate_subject": "CN=BioMCP MCPB Signing",
        "chain_verified": True,
        "eku": "codeSigning",
        "signing_policy_sha256": manifest["signing_policy_sha256"],
        "source_sha": manifest["source_sha"],
        "version": manifest["version"],
        "python_version": manifest["python_version"],
        "candidate_kind": "release",
        "stage_run_id": manifest["stage_run_id"],
        "signing_job_id": "mcpb-artifact",
        "fixture_only": False,
    }


def test_manifest_initializes_from_exact_committed_identity(tmp_path: Path) -> None:
    manifest = _manifest(tmp_path)
    assert manifest["schema_version"] == 2
    assert manifest["version"] == "1.2.3"
    assert manifest["python_version"] == "1.2.3"
    assert manifest["candidate_kind"] == "release"
    assert len(manifest["source_sha"]) == 40
    assert manifest["stage_run_id"] == "42"
    schema = json.loads((ROOT / "release/candidate-manifest.schema.json").read_text())
    jsonschema.Draft202012Validator(schema, format_checker=jsonschema.FormatChecker()).validate(manifest)


def test_manifest_accepts_exact_canonical_development_identity(tmp_path: Path) -> None:
    repo, sha = _repo(tmp_path, "0.9.0-dev.1", "0.9.0.dev1")

    manifest = candidate.init_manifest(
        repo, sha, "42", {"rust": "1.93.1"}, require_main=False
    )

    assert manifest["version"] == "0.9.0-dev.1"
    assert manifest["python_version"] == "0.9.0.dev1"
    assert manifest["candidate_kind"] == "development"


@pytest.mark.parametrize(
    ("rust_version", "python_version"),
    [
        ("01.2.3", "01.2.3"),
        ("1.02.3", "1.02.3"),
        ("1.2.03", "1.2.03"),
        ("1.2.3-dev.0", "1.2.3.dev0"),
        ("1.2.3-dev.01", "1.2.3.dev1"),
        ("1.2.3-dev.1", "1.2.3.dev01"),
        ("1.2.3-dev.1", "1.2.3.dev2"),
        ("1.2.3-rc.1", "1.2.3rc1"),
    ],
)
def test_manifest_rejects_noncanonical_or_mismatched_identity(
    tmp_path: Path, rust_version: str, python_version: str
) -> None:
    repo, sha = _repo(tmp_path, rust_version, python_version)

    with pytest.raises(candidate.CandidateError, match="candidate version"):
        candidate.init_manifest(
            repo, sha, "42", {"rust": "1.93.1"}, require_main=False
        )


def test_schema_one_manifest_is_rejected(tmp_path: Path) -> None:
    manifest = _manifest(tmp_path)
    manifest["schema_version"] = 1

    with pytest.raises(candidate.CandidateError, match="unsupported"):
        candidate.validate_manifest(manifest)


def test_init_rejects_dirty_checkout_existing_tag_and_short_sha(tmp_path: Path) -> None:
    repo, sha = _repo(tmp_path)
    with pytest.raises(candidate.CandidateError, match="full commit SHA"):
        candidate.init_manifest(repo, sha[:8], "42", {"rust": "pinned"}, require_main=False)
    subprocess.run(["git", "tag", "v1.2.3"], cwd=repo, check=True)
    with pytest.raises(candidate.CandidateError, match="already tagged"):
        candidate.init_manifest(repo, sha, "42", {"rust": "pinned"}, require_main=False)
    subprocess.run(["git", "tag", "-d", "v1.2.3"], cwd=repo, check=True, capture_output=True)
    (repo / "dirty").write_text("x")
    with pytest.raises(candidate.CandidateError, match="clean checkout"):
        candidate.init_manifest(repo, sha, "42", {"rust": "pinned"}, require_main=False)


def test_registration_is_exact_idempotent_and_conflicts_fail(tmp_path: Path) -> None:
    manifest = _manifest(tmp_path)
    artifact = tmp_path / "biomcp-linux-x86_64.tar.gz"
    artifact.write_bytes(b"one immutable archive")
    record = _record(manifest, "native-linux-x86_64", artifact)
    candidate.register_artifact(manifest, record, artifact)
    candidate.register_artifact(manifest, record, artifact)
    assert len(manifest["artifacts"]) == 1
    changed = {**record, "provenance": {"build_count": 2}}
    with pytest.raises(candidate.CandidateError, match="artifact conflict"):
        candidate.register_artifact(manifest, changed, artifact)


def test_registration_rejects_unknown_rebuilt_tampered_and_fixture_artifacts(tmp_path: Path) -> None:
    manifest = _manifest(tmp_path)
    artifact = tmp_path / "biomcp-linux-x86_64.tar.gz"
    artifact.write_bytes(b"archive")
    record = _record(manifest, "native-linux-x86_64", artifact)
    for key, value, message in [
        ("id", "unknown", "unregistered artifact"),
        ("sha256", "0" * 64, "does not match registered bytes"),
        ("source_sha", "1" * 40, "wrong source_sha"),
    ]:
        with pytest.raises(candidate.CandidateError, match=message):
            candidate.register_artifact(manifest, {**record, key: value}, artifact)
    with pytest.raises(candidate.CandidateError, match="fixture-only"):
        candidate.register_artifact(
            manifest,
            {**record, "evidence": {"fixture_only": True}},
            artifact,
        )


def test_registration_rejects_wheel_for_wrong_python_identity(tmp_path: Path) -> None:
    repo, sha = _repo(tmp_path, "0.9.0-dev.1", "0.9.0.dev1")
    manifest = candidate.init_manifest(
        repo, sha, "42", {"rust": "1.93.1"}, require_main=False
    )
    artifact = tmp_path / "biomcp_cli-0.9.0.dev2-py3-none-manylinux.whl"
    artifact.write_bytes(b"wrong Python wheel")
    record = _record(manifest, "wheel-linux-x86_64", artifact)
    record["evidence"]["python_version"] = "0.9.0.dev2"

    with pytest.raises(candidate.CandidateError, match="wrong python_version"):
        candidate.register_artifact(manifest, record, artifact)


@pytest.mark.parametrize(
    ("rust_version", "python_version", "status", "non_promotable"),
    [
        ("0.9.0-dev.1", "0.9.0.dev1", "signed", False),
        ("0.9.0", "0.9.0", "unsigned-development", True),
    ],
)
def test_mcpb_registration_rejects_outer_evidence_relabeling(
    tmp_path: Path,
    rust_version: str,
    python_version: str,
    status: str,
    non_promotable: bool,
) -> None:
    repo, sha = _repo(tmp_path, rust_version, python_version)
    manifest = candidate.init_manifest(
        repo, sha, "42", {"rust": "1.93.1"}, require_main=False
    )
    manifest["signing_policy_sha256"] = "f" * 64
    artifact = tmp_path / f"biomcp-{rust_version}.mcpb"
    artifact.write_bytes(b"MCPB archive")
    record = _record(manifest, "mcpb", artifact)
    record["evidence"].update(
        outer_signature_status=status,
        non_promotable=non_promotable,
    )

    with pytest.raises(candidate.CandidateError, match="candidate kind"):
        candidate.register_artifact(manifest, record, artifact)


def test_real_development_mcpb_evidence_cannot_be_relabelled_as_stable(
    tmp_path: Path,
) -> None:
    dev_root = tmp_path / "dev"
    stable_root = tmp_path / "stable"
    dev_root.mkdir()
    stable_root.mkdir()
    dev_repo, dev_sha = _repo(dev_root, "0.9.0-dev.1", "0.9.0.dev1")
    development = candidate.init_manifest(
        dev_repo, dev_sha, "42", {"rust": "1.93.1"}, require_main=False
    )
    development["signing_policy_sha256"] = "f" * 64
    stable_repo, stable_sha = _repo(stable_root, "0.9.0", "0.9.0")
    stable = candidate.init_manifest(
        stable_repo, stable_sha, "42", {"rust": "1.93.1"}, require_main=False
    )
    stable["signing_policy_sha256"] = "f" * 64
    artifact = tmp_path / "biomcp-0.9.0.mcpb"
    artifact.write_bytes(b"development outer archive")
    record = _record(stable, "mcpb", artifact)
    record["evidence"].update(
        outer_signature_status="signed",
        non_promotable=False,
        outer=_development_mcpb_outer(development, record["sha256"]),
    )

    with pytest.raises(candidate.CandidateError, match="stable MCPB outer evidence"):
        candidate.validate_artifact(stable, "mcpb", record)


@pytest.mark.parametrize(
    ("path", "replacement"),
    [
        (("archive_sha256",), "0" * 64),
        (("python_version",), "0.9.0.dev2"),
        (("signing_policy_sha256",), "0" * 64),
        (("github", "run_id"), "43"),
        (("github", "run_attempt"), "0"),
    ],
)
def test_development_mcpb_nested_evidence_is_exact(
    tmp_path: Path, path: tuple[str, ...], replacement: str
) -> None:
    repo, sha = _repo(tmp_path, "0.9.0-dev.1", "0.9.0.dev1")
    manifest = candidate.init_manifest(
        repo, sha, "42", {"rust": "1.93.1"}, require_main=False
    )
    manifest["signing_policy_sha256"] = "f" * 64
    artifact = tmp_path / "biomcp-0.9.0-dev.1.mcpb"
    artifact.write_bytes(b"development archive")
    record = _record(manifest, "mcpb", artifact)
    outer = _development_mcpb_outer(manifest, record["sha256"])
    target = outer
    for key in path[:-1]:
        target = target[key]
    target[path[-1]] = replacement
    record["evidence"].update(
        outer_signature_status="unsigned-development",
        non_promotable=True,
        outer=outer,
    )

    with pytest.raises(candidate.CandidateError, match="development MCPB outer evidence"):
        candidate.validate_artifact(manifest, "mcpb", record)


@pytest.mark.parametrize("mutation", ["missing", "extra"])
def test_mcpb_nested_evidence_rejects_missing_and_extra_fields(
    tmp_path: Path, mutation: str
) -> None:
    repo, sha = _repo(tmp_path, "0.9.0-dev.1", "0.9.0.dev1")
    manifest = candidate.init_manifest(
        repo, sha, "42", {"rust": "1.93.1"}, require_main=False
    )
    manifest["signing_policy_sha256"] = "f" * 64
    artifact = tmp_path / "biomcp-0.9.0-dev.1.mcpb"
    artifact.write_bytes(b"development archive")
    record = _record(manifest, "mcpb", artifact)
    outer = _development_mcpb_outer(manifest, record["sha256"])
    if mutation == "missing":
        outer.pop("tool_version")
    else:
        outer["unreviewed_exception"] = True
    record["evidence"].update(
        outer_signature_status="unsigned-development",
        non_promotable=True,
        outer=outer,
    )

    with pytest.raises(candidate.CandidateError, match="development MCPB outer evidence"):
        candidate.validate_artifact(manifest, "mcpb", record)


@pytest.mark.parametrize(
    ("field", "replacement"),
    [
        ("signed_sha256", "0" * 64),
        ("python_version", "0.9.1"),
        ("signing_policy_sha256", "0" * 64),
        ("fixture_only", True),
        ("certificate_fingerprint", "a" * 64),
    ],
)
def test_stable_mcpb_nested_signature_evidence_is_exact(
    tmp_path: Path, field: str, replacement: object
) -> None:
    repo, sha = _repo(tmp_path, "0.9.0", "0.9.0")
    manifest = candidate.init_manifest(
        repo, sha, "42", {"rust": "1.93.1"}, require_main=False
    )
    manifest["signing_policy_sha256"] = "f" * 64
    artifact = tmp_path / "biomcp-0.9.0.mcpb"
    artifact.write_bytes(b"signed stable archive")
    record = _record(manifest, "mcpb", artifact)
    outer = _stable_mcpb_outer(manifest, record["sha256"])
    record["evidence"].update(
        outer_signature_status="signed",
        non_promotable=False,
        outer=outer,
    )
    candidate.validate_artifact(manifest, "mcpb", record)
    outer[field] = replacement

    with pytest.raises(candidate.CandidateError, match="stable MCPB outer evidence"):
        candidate.validate_artifact(manifest, "mcpb", record)


def test_mcpb_filename_is_canonical_for_candidate_version(tmp_path: Path) -> None:
    repo, sha = _repo(tmp_path, "0.9.0", "0.9.0")
    manifest = candidate.init_manifest(
        repo, sha, "42", {"rust": "1.93.1"}, require_main=False
    )
    manifest["signing_policy_sha256"] = "f" * 64
    artifact = tmp_path / "renamed.mcpb"
    artifact.write_bytes(b"signed stable archive")
    record = _record(manifest, "mcpb", artifact)
    record["evidence"].update(
        outer_signature_status="signed",
        non_promotable=False,
        outer=_stable_mcpb_outer(manifest, record["sha256"]),
    )

    with pytest.raises(candidate.CandidateError, match="filename"):
        candidate.validate_artifact(manifest, "mcpb", record)


def test_finalize_requires_gates_and_exact_registered_set(tmp_path: Path) -> None:
    manifest = _manifest(tmp_path)
    with pytest.raises(candidate.CandidateError, match="missing candidate gates"):
        candidate.finalize_manifest(manifest, candidate.BASELINE_ARTIFACTS)
    manifest["gates"] = {name: "passed" for name in candidate.REQUIRED_GATES}
    with pytest.raises(candidate.CandidateError, match="artifact set mismatch"):
        candidate.finalize_manifest(manifest, candidate.BASELINE_ARTIFACTS)


def test_old_three_gate_candidate_cannot_finalize(tmp_path: Path) -> None:
    manifest = _manifest(tmp_path)
    manifest["gates"] = {name: "passed" for name in ("lint", "test", "spec")}
    with pytest.raises(candidate.CandidateError, match="full-feature-check"):
        candidate.finalize_manifest(manifest, candidate.BASELINE_ARTIFACTS)
