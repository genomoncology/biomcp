from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
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
promotion = _module("release_promotion", "release/promotion.py")


def _candidate(tmp_path: Path) -> tuple[Path, Path, dict, Path, str]:
    root = tmp_path / "candidate"
    root.mkdir()
    policy = tmp_path / "signing-policy.json"
    policy.write_text('{"schema_version":1,"enabled":true}\n')
    policy_hash = candidate.sha256_file(policy)
    manifest = {
        "schema_version": 2,
        "source_sha": "a" * 40,
        "version": "1.2.3",
        "python_version": "1.2.3",
        "candidate_kind": "release",
        "stage_run_id": "42",
        "status": "complete",
        "created_at": "2026-08-12T00:00:00Z",
        "gates": {name: "passed" for name in candidate.REQUIRED_GATES},
        "pins": {"rust": "1.93.1"},
        "signing_policy_sha256": policy_hash,
        "artifacts": {},
    }
    for artifact_id in sorted(candidate.FINAL_ARTIFACTS):
        kind, target = candidate.ARTIFACTS[artifact_id]
        filename = f"{artifact_id}.bin"
        path = root / filename
        path.write_bytes(artifact_id.encode())
        manifest["artifacts"][artifact_id] = {
            "id": artifact_id,
            "kind": kind,
            "target": target,
            "filename": filename,
            "sha256": candidate.sha256_file(path),
            "bytes": path.stat().st_size,
            "source_sha": manifest["source_sha"],
            "version": manifest["version"],
            "stage_run_id": manifest["stage_run_id"],
            "provenance": {"fixture": True},
            "evidence": {
                "inspected": True,
                "binary_sha256": "e" * 64,
                **(
                    {
                        "outer_signature_status": "signed",
                        "non_promotable": False,
                    }
                    if artifact_id == "mcpb"
                    else {}
                ),
                **(
                    {"python_version": manifest["python_version"]}
                    if kind == "wheel"
                    else {}
                ),
            },
        }
    manifest_path = root / "candidate-manifest.json"
    manifest_path.write_bytes(candidate.canonical_bytes(manifest))
    checksum = candidate.sha256_file(manifest_path)
    manifest_path.with_suffix(".json.sha256").write_text(
        f"{checksum}  candidate-manifest.json\n"
    )
    return root, manifest_path, manifest, policy, policy_hash


def _development_manifest(*, spoof_kind: bool = False) -> dict:
    return {
        "schema_version": 2,
        "source_sha": "a" * 40,
        "version": "0.9.0-dev.1",
        "python_version": "0.9.0.dev1",
        "candidate_kind": "release" if spoof_kind else "development",
        "stage_run_id": "42",
        "status": "complete",
        "created_at": "2026-08-15T00:00:00Z",
        "gates": {},
        "pins": {"rust": "1.93.1"},
        "signing_policy_sha256": None,
        "artifacts": {},
    }


def _development_public_fixture(
    tmp_path: Path, *, spoof_kind: bool = False
) -> tuple[dict, dict, dict, Path, bytes]:
    data = b"development artifact"
    manifest = _development_manifest(spoof_kind=spoof_kind)
    artifact_id = "mcpb"
    filename = "biomcp-dev.mcpb"
    manifest["artifacts"][artifact_id] = {
        "id": artifact_id,
        "kind": "mcpb",
        "target": "darwin-universal,win32-x86_64",
        "filename": filename,
        "sha256": candidate.sha256_file(_write(tmp_path / filename, data)),
        "bytes": len(data),
        "source_sha": manifest["source_sha"],
        "version": manifest["version"],
        "stage_run_id": manifest["stage_run_id"],
        "provenance": {"fixture": True},
        "evidence": {
            "inspected": True,
            "outer_signature_status": "unsigned-development",
            "non_promotable": True,
        },
    }
    inventory = {
        "version": manifest["version"],
        "stage_run_id": manifest["stage_run_id"],
        "files": {artifact_id: filename},
        "channels": {"github": [artifact_id]},
    }
    url = "https://public.invalid/dev/biomcp-dev.mcpb"
    public = {
        artifact_id: {
            "channel": "github",
            "url": url,
            "filename": filename,
            "version": manifest["version"],
            "source_sha": manifest["source_sha"],
            "target": manifest["artifacts"][artifact_id]["target"],
        }
    }
    return manifest, inventory, public, tmp_path, data


def _write(path: Path, data: bytes) -> Path:
    path.write_bytes(data)
    return path


def test_preflight_rejects_development_before_other_inputs_or_artifacts(
    tmp_path: Path,
) -> None:
    manifest_path = tmp_path / "candidate-manifest.json"
    manifest_path.write_text(json.dumps(_development_manifest()))

    with pytest.raises(promotion.PromotionError, match="development candidate"):
        promotion.preflight(
            manifest_path,
            tmp_path / "missing-candidate-root",
            "wrong-run",
            tmp_path / "missing-policy",
            "not-a-hash",
            require_credentials=True,
            windows_desktop_smoke="not json",
            updater_transition="not json",
            public_releases_path=tmp_path / "missing-public-releases",
        )


@pytest.mark.parametrize("spoof_kind", [False, True])
def test_updater_and_release_record_functions_validate_candidate_first(
    spoof_kind: bool,
) -> None:
    manifest = _development_manifest(spoof_kind=spoof_kind)
    expected = "version pair" if spoof_kind else "development candidate"

    with pytest.raises((candidate.CandidateError, promotion.PromotionError), match=expected):
        promotion.validate_updater_transition("not json", manifest, [])
    with pytest.raises((candidate.CandidateError, promotion.PromotionError), match=expected):
        promotion.release_record(
            manifest,
            {},
            {"already": "failed"},
            live_provider_results={"bad": None},
            formula_commit="not-a-commit",
        )


@pytest.mark.parametrize("spoof_kind", [False, True])
def test_normalize_updater_rejects_candidate_before_parsing_malformed_input(
    spoof_kind: bool,
) -> None:
    expected = "version pair" if spoof_kind else "development candidate"
    with pytest.raises((candidate.CandidateError, promotion.PromotionError), match=expected):
        promotion.normalize_updater_transition(
            "not json", _development_manifest(spoof_kind=spoof_kind), []
        )


@pytest.mark.parametrize("spoof_kind", [False, True])
def test_public_verifier_rejects_candidate_before_fetch(
    tmp_path: Path, spoof_kind: bool
) -> None:
    manifest, inventory, public, _, data = _development_public_fixture(
        tmp_path, spoof_kind=spoof_kind
    )
    fetched: list[str] = []

    def fetch(url: str) -> bytes:
        fetched.append(url)
        return data

    expected = "version pair" if spoof_kind else "development candidate"
    with pytest.raises((candidate.CandidateError, promotion.PromotionError), match=expected):
        promotion.verify_public_snapshot(manifest, inventory, public, fetch)
    assert fetched == []


@pytest.mark.parametrize("spoof_kind", [False, True])
def test_fixture_transaction_rejects_candidate_without_registry_effects(
    tmp_path: Path, spoof_kind: bool
) -> None:
    manifest, inventory, _, candidate_root, _ = _development_public_fixture(
        tmp_path, spoof_kind=spoof_kind
    )
    registry = promotion.FixtureRegistry(tmp_path / "registry")
    expected = "version pair" if spoof_kind else "development candidate"

    with pytest.raises((candidate.CandidateError, promotion.PromotionError), match=expected):
        promotion.fixture_transaction(registry, inventory, manifest, candidate_root)

    assert not registry.root.exists()


@pytest.mark.parametrize(
    ("command", "extra_arguments"),
    [
        ("validate-updater-transition", ["--record", "not json"]),
        (
            "release-record",
            [
                "--inventory",
                "missing-inventory.json",
                "--public-results",
                "missing-public.json",
                "--live-provider-results",
                "missing-live.json",
                "--formula-commit",
                "not-a-commit",
                "--output",
                "release-record.json",
            ],
        ),
    ],
)
@pytest.mark.parametrize("spoof_kind", [False, True])
def test_all_promotion_commands_reject_development_before_other_inputs_or_output(
    tmp_path: Path,
    command: str,
    extra_arguments: list[str],
    spoof_kind: bool,
) -> None:
    manifest_path = tmp_path / "candidate-manifest.json"
    manifest_path.write_text(
        json.dumps(_development_manifest(spoof_kind=spoof_kind))
    )
    result = subprocess.run(
        [
            sys.executable,
            str(ROOT / "release/promotion.py"),
            command,
            "--manifest",
            str(manifest_path),
            *extra_arguments,
        ],
        cwd=tmp_path,
        text=True,
        capture_output=True,
        check=False,
    )
    assert result.returncode == 2
    assert result.stdout == ""
    expected = "version pair" if spoof_kind else "development candidate"
    assert expected in result.stderr
    assert not (tmp_path / "release-record.json").exists()


def _manual_inputs(manifest: dict) -> tuple[str, str]:
    desktop = json.dumps(
        {
            "performed_by": " Ian Maurer ",
            "os": "Windows 11",
            "client": "Claude Desktop",
            "result": "passed",
            "mcpb_sha256": manifest["artifacts"]["mcpb"]["sha256"],
            "source_sha": manifest["source_sha"],
        }
    )
    updater = json.dumps(
        {
            "verified_installer_result": "passed",
            "legacy_update_result": "failed-without-changing-binary",
            "after_installer_sha256": manifest["artifacts"]["native-linux-x86_64"][
                "evidence"
            ]["binary_sha256"],
            "after_update_sha256": "d" * 64,
            "before_sha256": "d" * 64,
            "previous_version": "0.8.25",
            "version": manifest["version"],
            "source_sha": manifest["source_sha"],
        }
    )
    return desktop, updater


def _public_releases(tmp_path: Path, latest: str = "0.8.25") -> Path:
    path = tmp_path / "public-releases.json"
    path.write_text(
        json.dumps(
            [
                {
                    "id": 825,
                    "tag_name": f"v{latest}",
                    "draft": False,
                    "prerelease": False,
                    "published_at": "2026-07-07T12:00:00Z",
                    "html_url": f"https://github.com/genomoncology/biomcp/releases/tag/v{latest}",
                }
            ]
        )
    )
    return path


def _preflight(
    path: Path,
    root: Path,
    manifest: dict,
    policy: Path,
    policy_hash: str,
    stage_run_id: str = "42",
) -> dict:
    desktop, updater = _manual_inputs(manifest)
    return promotion.preflight(
        path,
        root,
        stage_run_id,
        policy,
        policy_hash,
        require_credentials=False,
        windows_desktop_smoke=desktop,
        updater_transition=updater,
        public_releases_path=_public_releases(root.parent),
    )


def test_preflight_validates_and_binds_normalized_manual_inputs(tmp_path: Path) -> None:
    root, path, manifest, policy, policy_hash = _candidate(tmp_path)
    desktop, updater = _manual_inputs(manifest)

    inventory = promotion.preflight(
        path,
        root,
        "42",
        policy,
        policy_hash,
        require_credentials=False,
        windows_desktop_smoke=desktop,
        updater_transition=updater,
        public_releases_path=_public_releases(tmp_path),
    )

    assert inventory["manual_windows_desktop_smoke"]["performed_by"] == "Ian Maurer"
    assert inventory["updater_transition"]["previous_version"] == "0.8.25"
    assert inventory["updater_result"] == "legacy-updater-limit-from-v0.8.25"
    with pytest.raises(promotion.PromotionError, match="Windows desktop smoke"):
        promotion.preflight(
            path,
            root,
            "42",
            policy,
            policy_hash,
            require_credentials=False,
            windows_desktop_smoke="{}",
            updater_transition=updater,
            public_releases_path=_public_releases(tmp_path),
        )


def test_preflight_rejects_legacy_waiver_unless_0825_is_actual_latest_public_release(
    tmp_path: Path,
) -> None:
    root, path, manifest, policy, policy_hash = _candidate(tmp_path)
    desktop, updater = _manual_inputs(manifest)
    with pytest.raises(promotion.PromotionError, match="actual prior public release"):
        promotion.preflight(
            path,
            root,
            "42",
            policy,
            policy_hash,
            require_credentials=False,
            windows_desktop_smoke=desktop,
            updater_transition=updater,
            public_releases_path=_public_releases(tmp_path, "0.8.26"),
        )


def test_preflight_cli_binds_verified_actual_prior_public_release(
    tmp_path: Path,
) -> None:
    root, path, manifest, policy, policy_hash = _candidate(tmp_path)
    desktop, updater = _manual_inputs(manifest)
    output = tmp_path / "inventory.json"
    subprocess.run(
        [
            sys.executable,
            str(ROOT / "release/promotion.py"),
            "preflight",
            "--manifest",
            str(path),
            "--candidate-root",
            str(root),
            "--stage-run-id",
            "42",
            "--policy",
            str(policy),
            "--protected-policy-sha256",
            policy_hash,
            "--windows-desktop-smoke",
            desktop,
            "--updater-transition",
            updater,
            "--public-releases",
            str(_public_releases(tmp_path)),
            "--output",
            str(output),
        ],
        cwd=ROOT,
        check=True,
    )
    assert (
        json.loads(output.read_text())["prior_public_release"]["tag_name"] == "v0.8.25"
    )


def test_preflight_resolves_every_byte_and_derives_channels_from_manifest(
    tmp_path: Path,
) -> None:
    root, path, manifest, policy, policy_hash = _candidate(tmp_path)
    inventory = _preflight(path, root, manifest, policy, policy_hash)
    assert set(inventory["files"]) == candidate.FINAL_ARTIFACTS
    assert inventory["tag"] == "v1.2.3"
    assert len(inventory["channels"]["pypi"]) == 5
    assert "mcpb" in inventory["channels"]["github"]
    assert inventory["channels"]["ghcr"] == ["oci-index"]


def test_preflight_rejects_stage_policy_checksum_and_byte_substitution(
    tmp_path: Path,
) -> None:
    root, path, manifest, policy, policy_hash = _candidate(tmp_path)
    with pytest.raises(promotion.PromotionError, match="stage run ID"):
        _preflight(path, root, manifest, policy, policy_hash, "43")
    with pytest.raises(promotion.PromotionError, match="policy"):
        _preflight(path, root, manifest, policy, "0" * 64)
    (root / manifest["artifacts"]["mcpb"]["filename"]).write_bytes(b"changed")
    with pytest.raises(promotion.PromotionError, match="bytes changed"):
        _preflight(path, root, manifest, policy, policy_hash)


def test_versioned_replay_is_noop_conflict_fails_and_latest_waits_for_verification(
    tmp_path: Path,
) -> None:
    root, path, manifest, policy, policy_hash = _candidate(tmp_path)
    inventory = _preflight(path, root, manifest, policy, policy_hash)
    registry = promotion.FixtureRegistry(tmp_path / "public")
    result = promotion.fixture_transaction(registry, inventory, manifest, root)
    assert result["status"] == "complete"
    assert (registry.mutable / "latest").read_text().strip() == "1.2.3"
    replay = promotion.fixture_transaction(registry, inventory, manifest, root)
    assert all(write["result"] == "unchanged" for write in replay["writes"])
    public = (
        registry.versioned
        / "github"
        / "1.2.3"
        / manifest["artifacts"]["mcpb"]["filename"]
    )
    public.write_bytes(b"conflict")
    with pytest.raises(promotion.PromotionError, match="conflict"):
        promotion.fixture_transaction(registry, inventory, manifest, root)


def test_partial_failure_records_truth_and_never_advances_mutable_pointer(
    tmp_path: Path,
) -> None:
    root, path, manifest, policy, policy_hash = _candidate(tmp_path)
    inventory = _preflight(path, root, manifest, policy, policy_hash)
    registry = promotion.FixtureRegistry(tmp_path / "public")
    with pytest.raises(promotion.PromotionError, match="injected"):
        promotion.fixture_transaction(registry, inventory, manifest, root, fail_after=2)
    assert not (registry.mutable / "latest").exists()
    partial = registry.records / "release-record-1.2.3-partial-42.json"
    assert json.loads(partial.read_text())["status"] == "partial"


def test_manual_desktop_smoke_is_bound_to_exact_source_and_bundle() -> None:
    value = json.dumps(
        {
            "source_sha": "a" * 40,
            "mcpb_sha256": "b" * 64,
            "result": "passed",
            "client": "Claude Desktop",
            "os": "Windows 11",
            "performed_by": "Ian Maurer",
        }
    )
    assert (
        promotion.validate_manual_smoke(value, "a" * 40, "b" * 64)["os"] == "Windows 11"
    )
    with pytest.raises(promotion.PromotionError, match="mcpb_sha256"):
        promotion.validate_manual_smoke(value, "a" * 40, "c" * 64)


def _public_snapshot(
    manifest: dict, inventory: dict, root: Path
) -> tuple[dict, dict[str, bytes]]:
    public = {}
    downloads = {}
    for channel, artifact_ids in inventory["channels"].items():
        for artifact_id in artifact_ids:
            record = manifest["artifacts"][artifact_id]
            url = f"https://public.invalid/{channel}/{manifest['version']}/{record['filename']}"
            public[artifact_id] = {
                "channel": channel,
                "url": url,
                "filename": record["filename"],
                "version": manifest["version"],
                "source_sha": manifest["source_sha"],
                "target": record["target"],
            }
            downloads[url] = (root / inventory["files"][artifact_id]).read_bytes()
    return public, downloads


def test_complete_public_verifier_rejects_missing_stale_and_wrong_identity(
    tmp_path: Path,
) -> None:
    root, path, manifest, policy, policy_hash = _candidate(tmp_path)
    inventory = _preflight(path, root, manifest, policy, policy_hash)
    public, downloads = _public_snapshot(manifest, inventory, root)
    assert (
        len(
            promotion.verify_public_snapshot(
                manifest, inventory, public, downloads.__getitem__
            )
        )
        == 13
    )

    missing = dict(public)
    missing.pop("mcpb")
    with pytest.raises(promotion.PromotionError, match="inventory"):
        promotion.verify_public_snapshot(
            manifest, inventory, missing, downloads.__getitem__
        )

    stale = dict(downloads)
    stale[public["mcpb"]["url"]] = b"old cached bundle"
    with pytest.raises(promotion.PromotionError, match="byte mismatch"):
        promotion.verify_public_snapshot(manifest, inventory, public, stale.__getitem__)

    wrong = {key: dict(value) for key, value in public.items()}
    wrong["native-linux-arm64"]["target"] = "x86_64-unknown-linux-gnu"
    with pytest.raises(promotion.PromotionError, match="wrong target"):
        promotion.verify_public_snapshot(
            manifest, inventory, wrong, downloads.__getitem__
        )

    wrong_version = {key: dict(value) for key, value in public.items()}
    wrong_version["mcpb"]["version"] = "1.2.2"
    with pytest.raises(promotion.PromotionError, match="wrong version"):
        promotion.verify_public_snapshot(
            manifest, inventory, wrong_version, downloads.__getitem__
        )

    unavailable = dict(downloads)
    unavailable.pop(public["mcpb"]["url"])
    with pytest.raises(promotion.PromotionError, match="download failed"):
        promotion.verify_public_snapshot(
            manifest, inventory, public, unavailable.__getitem__
        )


def test_live_provider_and_one_time_legacy_updater_rules(tmp_path: Path) -> None:
    assert promotion.validate_live_provider_results(
        {
            "NCI": {"status": "unavailable", "reason": "planned maintenance"},
            "PubMed": {"status": "passed"},
        }
    ) == [{"provider": "NCI", "reason": "planned maintenance"}]
    with pytest.raises(promotion.PromotionError, match="contract failed"):
        promotion.validate_live_provider_results({"NCI": {"status": "failed"}})
    assert (
        promotion.legacy_updater_result(
            "0.8.25", [], legacy_limit_proved=True, installer_upgrade_proved=True
        )
        == "legacy-updater-limit-from-v0.8.25"
    )
    with pytest.raises(promotion.PromotionError, match="already used"):
        promotion.legacy_updater_result(
            "0.8.25",
            [{"waivers": ["legacy-updater-limit-from-v0.8.25"]}],
            legacy_limit_proved=True,
            installer_upgrade_proved=True,
        )

    _, _, updater_manifest, _, _ = _candidate(tmp_path)
    transition = json.dumps(
        {
            "source_sha": updater_manifest["source_sha"],
            "version": updater_manifest["version"],
            "previous_version": "0.8.25",
            "legacy_update_result": "failed-without-changing-binary",
            "verified_installer_result": "passed",
            "before_sha256": "d" * 64,
            "after_update_sha256": "d" * 64,
            "after_installer_sha256": "e" * 64,
        }
    )
    assert promotion.validate_updater_transition(transition, updater_manifest, []) == (
        "legacy-updater-limit-from-v0.8.25"
    )
    with pytest.raises(promotion.PromotionError, match="failure proof"):
        promotion.validate_updater_transition(
            transition.replace("failed-without-changing-binary", "changed-binary"),
            updater_manifest,
            [],
        )


def test_final_record_requires_every_public_result_and_records_limitations(
    tmp_path: Path,
) -> None:
    root, path, manifest, policy, policy_hash = _candidate(tmp_path)
    inventory = _preflight(path, root, manifest, policy, policy_hash)
    results = {artifact_id: "passed" for artifact_id in candidate.FINAL_ARTIFACTS}
    record = promotion.release_record(
        manifest,
        inventory,
        results,
        live_provider_results={
            "NCI": {"status": "unavailable", "reason": "maintenance"}
        },
        formula_commit="c" * 40,
    )
    assert record["candidate_job"] == "sealed-candidate-42"
    assert record["prior_public_release"]["tag_name"] == "v0.8.25"
    assert record["live_provider_limitations"][0]["provider"] == "NCI"
    assert record["waivers"] == ["legacy-updater-limit-from-v0.8.25"]
    with pytest.raises(promotion.PromotionError, match="incomplete"):
        promotion.release_record(
            manifest,
            inventory,
            {**results, "mcpb": "failed"},
            live_provider_results={},
            formula_commit="c" * 40,
        )
