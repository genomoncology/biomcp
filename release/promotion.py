#!/usr/bin/env python3
"""Preflight and fixture-prove promotion of one sealed BioMCP candidate."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import sys
from pathlib import Path
from typing import Any

from candidate import (
    FINAL_ARTIFACTS,
    CandidateError,
    canonical_bytes,
    load_manifest,
    sha256_file,
)

REQUIRED_CREDENTIALS = (
    "GH_TOKEN",
    "BIOMCP_PYPI_TOKEN",
    "BIOMCP_GHCR_TOKEN",
    "BIOMCP_HOMEBREW_TAP_TOKEN",
)

PUBLIC_CHANNELS = {"github", "pypi", "ghcr", "homebrew"}


class PromotionError(ValueError):
    pass


def require_release_candidate(manifest: dict[str, Any]) -> None:
    if manifest["candidate_kind"] != "release":
        raise PromotionError("development candidate cannot enter promotion")


def _unique_file(root: Path, filename: str) -> Path:
    matches = [
        path
        for path in root.rglob(filename)
        if path.is_file() and not path.is_symlink()
    ]
    if len(matches) != 1:
        raise PromotionError(
            f"expected one private candidate file named {filename}, found {len(matches)}"
        )
    return matches[0]


def validate_manual_smoke(
    value: str, source_sha: str, mcpb_sha256: str
) -> dict[str, str]:
    try:
        record = json.loads(value)
    except json.JSONDecodeError as error:
        raise PromotionError("Windows desktop smoke record is not JSON") from error
    expected = {
        "source_sha": source_sha,
        "mcpb_sha256": mcpb_sha256,
        "result": "passed",
        "client": "Claude Desktop",
    }
    for key, expected_value in expected.items():
        if record.get(key) != expected_value:
            raise PromotionError(f"Windows desktop smoke has wrong {key}")
    if record.get("os") not in {"Windows 10", "Windows 11"}:
        raise PromotionError("Windows desktop smoke requires Windows 10 or 11")
    if (
        not isinstance(record.get("performed_by"), str)
        or not record["performed_by"].strip()
    ):
        raise PromotionError("Windows desktop smoke lacks performer")
    return {
        **expected,
        "os": record["os"],
        "performed_by": record["performed_by"].strip(),
    }


def verify_public_snapshot(
    manifest: dict[str, Any],
    inventory: dict[str, Any],
    public: dict[str, dict[str, Any]],
    fetch: Any,
) -> dict[str, str]:
    """Verify bytes fetched through public versioned endpoints.

    ``fetch`` is deliberately injected. Production passes an HTTPS downloader;
    tests pass a local registry so routine gates never touch public services.
    """
    expected_ids = {
        artifact_id
        for artifact_ids in inventory["channels"].values()
        for artifact_id in artifact_ids
    }
    if set(public) != expected_ids:
        raise PromotionError("public artifact inventory does not match candidate")
    results: dict[str, str] = {}
    for artifact_id in sorted(expected_ids):
        record = manifest["artifacts"][artifact_id]
        endpoint = public[artifact_id]
        channel = endpoint.get("channel")
        if channel not in PUBLIC_CHANNELS or artifact_id not in inventory[
            "channels"
        ].get(channel, []):
            raise PromotionError(f"public channel mismatch: {artifact_id}")
        expected_identity = {
            "filename": record["filename"],
            "version": manifest["version"],
            "source_sha": manifest["source_sha"],
            "target": record["target"],
        }
        for key, value in expected_identity.items():
            if endpoint.get(key) != value:
                raise PromotionError(f"public {artifact_id} has wrong {key}")
        url = endpoint.get("url")
        if not isinstance(url, str) or not url.startswith("https://"):
            raise PromotionError(f"public {artifact_id} lacks an HTTPS versioned URL")
        try:
            data = fetch(url)
        except Exception as error:
            raise PromotionError(f"public download failed: {artifact_id}") from error
        if not isinstance(data, bytes):
            raise PromotionError(
                f"public downloader returned invalid bytes: {artifact_id}"
            )
        digest = hashlib.sha256(data).hexdigest()
        if digest != record["sha256"] or len(data) != record["bytes"]:
            raise PromotionError(f"public byte mismatch: {artifact_id}")
        results[artifact_id] = "passed"
    return results


def validate_live_provider_results(value: dict[str, Any]) -> list[dict[str, str]]:
    limitations: list[dict[str, str]] = []
    for provider, result in sorted(value.items()):
        status = result.get("status") if isinstance(result, dict) else None
        if status == "passed":
            continue
        if (
            status == "unavailable"
            and isinstance(result.get("reason"), str)
            and result["reason"]
        ):
            limitations.append({"provider": provider, "reason": result["reason"]})
            continue
        raise PromotionError(f"live provider contract failed: {provider}")
    return limitations


def legacy_updater_result(
    previous_version: str,
    prior_records: list[dict[str, Any]],
    *,
    legacy_limit_proved: bool,
    installer_upgrade_proved: bool,
) -> str:
    waiver = "legacy-updater-limit-from-v0.8.25"
    if previous_version == "0.8.25":
        if any(waiver in record.get("waivers", []) for record in prior_records):
            raise PromotionError("legacy updater waiver was already used")
        if not legacy_limit_proved or not installer_upgrade_proved:
            raise PromotionError("legacy updater transition proof is incomplete")
        return waiver
    if legacy_limit_proved:
        raise PromotionError("legacy updater waiver applies only after v0.8.25")
    if not installer_upgrade_proved:
        raise PromotionError("previous-version updater proof is incomplete")
    return "previous-version-self-update-passed"


def validate_updater_transition(
    value: str, manifest: dict[str, Any], prior_records: list[dict[str, Any]]
) -> str:
    _, result = normalize_updater_transition(value, manifest, prior_records)
    return result


def normalize_updater_transition(
    value: str, manifest: dict[str, Any], prior_records: list[dict[str, Any]]
) -> tuple[dict[str, Any], str]:
    try:
        record = json.loads(value)
    except json.JSONDecodeError as error:
        raise PromotionError("updater transition record is not JSON") from error
    for key, expected in {
        "source_sha": manifest["source_sha"],
        "version": manifest["version"],
    }.items():
        if record.get(key) != expected:
            raise PromotionError(f"updater transition has wrong {key}")
    previous = record.get("previous_version")
    if not isinstance(previous, str):
        raise PromotionError("updater transition lacks previous version")
    hashes = {
        key: record.get(key)
        for key in ("before_sha256", "after_update_sha256", "after_installer_sha256")
    }
    if any(
        not isinstance(value, str)
        or len(value) != 64
        or any(char not in "0123456789abcdef" for char in value)
        for value in hashes.values()
    ):
        raise PromotionError("updater transition lacks exact executable hashes")
    expected_new_hash = manifest["artifacts"]["native-linux-x86_64"]["evidence"].get(
        "binary_sha256"
    )
    if hashes["after_installer_sha256"] != expected_new_hash:
        raise PromotionError("verified installer produced the wrong executable")
    if previous == "0.8.25":
        if record.get("legacy_update_result") != "failed-without-changing-binary":
            raise PromotionError("legacy updater failure proof is incomplete")
        if hashes["after_update_sha256"] != hashes["before_sha256"]:
            raise PromotionError("legacy updater changed the installed executable")
        result = legacy_updater_result(
            previous,
            prior_records,
            legacy_limit_proved=True,
            installer_upgrade_proved=record.get("verified_installer_result")
            == "passed",
        )
        return {
            "source_sha": record["source_sha"],
            "version": record["version"],
            "previous_version": previous,
            "legacy_update_result": record["legacy_update_result"],
            "verified_installer_result": record["verified_installer_result"],
            **hashes,
        }, result
    if record.get("self_update_result") != "passed":
        raise PromotionError("previous-version self-update proof is incomplete")
    if hashes["after_update_sha256"] != expected_new_hash:
        raise PromotionError("self-update produced the wrong executable")
    result = legacy_updater_result(
        previous,
        prior_records,
        legacy_limit_proved=False,
        installer_upgrade_proved=True,
    )
    return {
        "source_sha": record["source_sha"],
        "version": record["version"],
        "previous_version": previous,
        "self_update_result": record["self_update_result"],
        **hashes,
    }, result


def validate_prior_public_release(
    path: Path, previous_version: str, candidate_version: str
) -> dict[str, Any]:
    try:
        releases = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise PromotionError(
            "public release inventory is absent or malformed"
        ) from error
    if not isinstance(releases, list):
        raise PromotionError("public release inventory must be a list")
    public = [
        release
        for release in releases
        if isinstance(release, dict)
        and release.get("draft") is False
        and release.get("prerelease") is False
        and isinstance(release.get("published_at"), str)
        and release["published_at"]
    ]
    if any(release.get("tag_name") == f"v{candidate_version}" for release in public):
        raise PromotionError("candidate version is already a public release")
    if not public:
        raise PromotionError("public release inventory contains no stable release")
    latest = max(public, key=lambda release: release["published_at"])
    expected_tag = f"v{previous_version}"
    if latest.get("tag_name") != expected_tag:
        raise PromotionError(
            f"{previous_version} is not the actual prior public release"
        )
    if (
        not isinstance(latest.get("id"), int)
        or not isinstance(latest.get("html_url"), str)
        or latest["html_url"]
        != f"https://github.com/genomoncology/biomcp/releases/tag/{expected_tag}"
    ):
        raise PromotionError("actual prior public release identity is incomplete")
    return {
        "id": latest["id"],
        "tag_name": expected_tag,
        "published_at": latest["published_at"],
        "html_url": latest["html_url"],
    }


def preflight(
    manifest_path: Path,
    candidate_root: Path,
    stage_run_id: str,
    policy_path: Path,
    protected_policy_hash: str,
    *,
    require_credentials: bool,
    windows_desktop_smoke: str,
    updater_transition: str,
    public_releases_path: Path,
) -> dict[str, Any]:
    manifest = load_manifest(manifest_path)
    require_release_candidate(manifest)
    if (
        manifest["status"] != "complete"
        or set(manifest["artifacts"]) != FINAL_ARTIFACTS
    ):
        raise PromotionError("promotion requires one complete final candidate")
    if manifest["stage_run_id"] != stage_run_id:
        raise PromotionError("stage run ID does not match candidate")
    checksum_path = manifest_path.with_suffix(manifest_path.suffix + ".sha256")
    expected_line = f"{sha256_file(manifest_path)}  {manifest_path.name}"
    if (
        not checksum_path.is_file()
        or checksum_path.read_text(encoding="utf-8").strip() != expected_line
    ):
        raise PromotionError("candidate manifest checksum is absent or invalid")
    actual_policy_hash = sha256_file(policy_path)
    if (
        manifest["signing_policy_sha256"] != actual_policy_hash
        or protected_policy_hash != actual_policy_hash
    ):
        raise PromotionError("candidate and protected signing policy disagree")
    files: dict[str, str] = {}
    for artifact_id, record in manifest["artifacts"].items():
        path = _unique_file(candidate_root, record["filename"])
        if (
            sha256_file(path) != record["sha256"]
            or path.stat().st_size != record["bytes"]
        ):
            raise PromotionError(f"private candidate bytes changed: {artifact_id}")
        files[artifact_id] = str(path.relative_to(candidate_root))
    if require_credentials:
        missing = [name for name in REQUIRED_CREDENTIALS if not os.environ.get(name)]
        if missing:
            raise PromotionError(f"missing promotion credentials: {', '.join(missing)}")
    version = manifest["version"]
    manual_smoke = validate_manual_smoke(
        windows_desktop_smoke,
        manifest["source_sha"],
        manifest["artifacts"]["mcpb"]["sha256"],
    )
    normalized_updater, updater_result = normalize_updater_transition(
        updater_transition, manifest, []
    )
    prior_public_release = validate_prior_public_release(
        public_releases_path,
        normalized_updater["previous_version"],
        manifest["version"],
    )
    github_ids = sorted(
        artifact_id
        for artifact_id, record in manifest["artifacts"].items()
        if record["kind"] in {"native", "mcpb"}
    )
    return {
        "schema_version": 1,
        "source_sha": manifest["source_sha"],
        "version": version,
        "tag": f"v{version}",
        "stage_run_id": stage_run_id,
        "manifest_sha256": sha256_file(manifest_path),
        "signing_policy_sha256": actual_policy_hash,
        "files": files,
        "manual_windows_desktop_smoke": manual_smoke,
        "updater_transition": normalized_updater,
        "updater_result": updater_result,
        "prior_public_release": prior_public_release,
        "public_release_inventory_sha256": sha256_file(public_releases_path),
        "channels": {
            "github": github_ids,
            "pypi": sorted(
                artifact_id
                for artifact_id, record in manifest["artifacts"].items()
                if record["kind"] == "wheel"
            ),
            "ghcr": ["oci-index"],
            "homebrew": ["homebrew-formula"],
        },
    }


class FixtureRegistry:
    """Filesystem model for versioned writes and mutable-pointer ordering."""

    def __init__(self, root: Path):
        self.root = root
        self.versioned = root / "versioned"
        self.mutable = root / "mutable"
        self.records = root / "records"

    def publish(self, channel: str, version: str, name: str, source: Path) -> str:
        destination = self.versioned / channel / version / name
        destination.parent.mkdir(parents=True, exist_ok=True)
        if destination.exists():
            if sha256_file(destination) == sha256_file(source):
                return "unchanged"
            raise PromotionError(f"public byte conflict: {channel}/{name}")
        shutil.copyfile(source, destination)
        return "created"

    def verify(self, channel: str, version: str, name: str, expected_hash: str) -> None:
        destination = self.versioned / channel / version / name
        if not destination.is_file() or sha256_file(destination) != expected_hash:
            raise PromotionError(f"public verification failed: {channel}/{name}")

    def advance(self, name: str, version: str) -> None:
        self.mutable.mkdir(parents=True, exist_ok=True)
        (self.mutable / name).write_text(version + "\n", encoding="utf-8")

    def partial_record(self, version: str, run_id: str, value: dict[str, Any]) -> Path:
        self.records.mkdir(parents=True, exist_ok=True)
        path = self.records / f"release-record-{version}-partial-{run_id}.json"
        if path.exists():
            raise PromotionError("partial release record already exists")
        path.write_bytes(canonical_bytes(value))
        return path


def fixture_transaction(
    registry: FixtureRegistry,
    inventory: dict[str, Any],
    manifest: dict[str, Any],
    candidate_root: Path,
    *,
    fail_after: int | None = None,
) -> dict[str, Any]:
    writes: list[dict[str, str]] = []
    try:
        count = 0
        for channel, artifact_ids in inventory["channels"].items():
            for artifact_id in artifact_ids:
                record = manifest["artifacts"][artifact_id]
                source = candidate_root / inventory["files"][artifact_id]
                result = registry.publish(
                    channel, inventory["version"], record["filename"], source
                )
                writes.append(
                    {"channel": channel, "artifact": artifact_id, "result": result}
                )
                count += 1
                if fail_after is not None and count == fail_after:
                    raise PromotionError("injected publication failure")
        for channel, artifact_ids in inventory["channels"].items():
            for artifact_id in artifact_ids:
                record = manifest["artifacts"][artifact_id]
                registry.verify(
                    channel, inventory["version"], record["filename"], record["sha256"]
                )
        registry.advance("latest", inventory["version"])
        return {"status": "complete", "writes": writes, **inventory}
    except PromotionError as error:
        record = {
            "status": "partial",
            "error": str(error),
            "writes": writes,
            **inventory,
        }
        registry.partial_record(inventory["version"], inventory["stage_run_id"], record)
        raise


def release_record(
    manifest: dict[str, Any],
    inventory: dict[str, Any],
    public_results: dict[str, Any],
    *,
    live_provider_results: dict[str, Any],
    formula_commit: str,
) -> dict[str, Any]:
    if any(value != "passed" for value in public_results.values()):
        raise PromotionError("public verification is incomplete")
    return {
        "schema_version": 1,
        "status": "complete",
        "source_sha": manifest["source_sha"],
        "version": manifest["version"],
        "stage_run_id": manifest["stage_run_id"],
        "candidate_manifest_sha256": inventory["manifest_sha256"],
        "signing_policy_sha256": inventory["signing_policy_sha256"],
        "candidate_job": f"sealed-candidate-{manifest['stage_run_id']}",
        "artifacts": {
            key: {
                "sha256": value["sha256"],
                "provenance": value["provenance"],
                "evidence": value["evidence"],
            }
            for key, value in manifest["artifacts"].items()
        },
        "public_results": public_results,
        "manual_windows_desktop_smoke": inventory["manual_windows_desktop_smoke"],
        "updater_transition": inventory["updater_transition"],
        "prior_public_release": inventory["prior_public_release"],
        "public_release_inventory_sha256": inventory["public_release_inventory_sha256"],
        "formula_commit": formula_commit,
        "updater_result": inventory["updater_result"],
        "waivers": (
            [inventory["updater_result"]]
            if inventory["updater_result"] == "legacy-updater-limit-from-v0.8.25"
            else []
        ),
        "live_provider_limitations": validate_live_provider_results(
            live_provider_results
        ),
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    promotable = commands.add_parser("require-release")
    promotable.add_argument("--manifest", type=Path, required=True)
    preflight_command = commands.add_parser("preflight")
    preflight_command.add_argument("--manifest", type=Path, required=True)
    preflight_command.add_argument("--candidate-root", type=Path, required=True)
    preflight_command.add_argument("--stage-run-id", required=True)
    preflight_command.add_argument("--policy", type=Path, required=True)
    preflight_command.add_argument("--protected-policy-sha256", required=True)
    preflight_command.add_argument("--output", type=Path, required=True)
    preflight_command.add_argument("--windows-desktop-smoke", required=True)
    preflight_command.add_argument("--updater-transition", required=True)
    preflight_command.add_argument("--public-releases", type=Path, required=True)
    preflight_command.add_argument("--require-credentials", action="store_true")
    manual = commands.add_parser("validate-manual-smoke")
    manual.add_argument("--record", required=True)
    manual.add_argument("--source-sha", required=True)
    manual.add_argument("--mcpb-sha256", required=True)
    updater = commands.add_parser("validate-updater-transition")
    updater.add_argument("--record", required=True)
    updater.add_argument("--manifest", type=Path, required=True)
    record = commands.add_parser("release-record")
    record.add_argument("--manifest", type=Path, required=True)
    record.add_argument("--inventory", type=Path, required=True)
    record.add_argument("--public-results", type=Path, required=True)
    record.add_argument("--live-provider-results", type=Path, required=True)
    record.add_argument("--formula-commit", required=True)
    record.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try:
        if args.command == "require-release":
            require_release_candidate(load_manifest(args.manifest))
        elif args.command == "preflight":
            inventory = preflight(
                args.manifest,
                args.candidate_root,
                args.stage_run_id,
                args.policy,
                args.protected_policy_sha256,
                require_credentials=args.require_credentials,
                windows_desktop_smoke=args.windows_desktop_smoke,
                updater_transition=args.updater_transition,
                public_releases_path=args.public_releases,
            )
            args.output.write_bytes(canonical_bytes(inventory))
        elif args.command == "validate-manual-smoke":
            validate_manual_smoke(args.record, args.source_sha, args.mcpb_sha256)
        elif args.command == "validate-updater-transition":
            print(
                validate_updater_transition(
                    args.record, load_manifest(args.manifest), []
                )
            )
        else:
            manifest = load_manifest(args.manifest)
            inventory = json.loads(args.inventory.read_text(encoding="utf-8"))
            public_results = json.loads(args.public_results.read_text(encoding="utf-8"))
            live_results = json.loads(
                args.live_provider_results.read_text(encoding="utf-8")
            )
            value = release_record(
                manifest,
                inventory,
                public_results,
                live_provider_results=live_results,
                formula_commit=args.formula_commit,
            )
            args.output.write_bytes(canonical_bytes(value))
            args.output.with_suffix(args.output.suffix + ".sha256").write_text(
                f"{sha256_file(args.output)}  {args.output.name}\n", encoding="utf-8"
            )
        return 0
    except (CandidateError, OSError, PromotionError) as error:
        print(f"promotion: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
