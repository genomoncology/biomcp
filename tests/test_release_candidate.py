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
        "evidence": {"inspected": True},
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
