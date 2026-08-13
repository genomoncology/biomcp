#!/usr/bin/env python3
"""Inspect a private two-platform OCI layout and emit its candidate record."""

from __future__ import annotations

import argparse
import hashlib
import json
import tarfile
from pathlib import Path
from typing import Any

from candidate import ARTIFACTS, canonical_bytes, sha256_file

PLATFORMS = {("linux", "amd64"), ("linux", "arm64")}
REQUIRED_LABELS = {
    "org.opencontainers.image.source",
    "org.opencontainers.image.revision",
    "org.opencontainers.image.version",
    "org.opencontainers.image.licenses",
    "org.opencontainers.image.created",
}


class ContainerError(ValueError):
    pass


def _json_member(archive: tarfile.TarFile, name: str) -> dict[str, Any]:
    try:
        member = archive.getmember(name)
        handle = archive.extractfile(member)
        if handle is None:
            raise KeyError(name)
        return json.load(handle)
    except (KeyError, json.JSONDecodeError) as error:
        raise ContainerError(f"invalid OCI member: {name}") from error


def _blob_name(digest: str) -> str:
    algorithm, separator, value = digest.partition(":")
    if algorithm != "sha256" or separator != ":" or len(value) != 64:
        raise ContainerError(f"invalid OCI digest: {digest}")
    return f"blobs/sha256/{value}"


def inspect_layout(path: Path, source_sha: str, version: str) -> dict[str, Any]:
    with tarfile.open(path, "r:*") as archive:
        index = _json_member(archive, "index.json")
        manifests = index.get("manifests", [])
        actual = {
            (item.get("platform", {}).get("os"), item.get("platform", {}).get("architecture"))
            for item in manifests
        }
        if actual != PLATFORMS or len(manifests) != 2:
            raise ContainerError(f"OCI index platforms mismatch: {sorted(actual)}")
        platform_digests: dict[str, str] = {}
        for descriptor in manifests:
            manifest = _json_member(archive, _blob_name(descriptor["digest"]))
            config = _json_member(archive, _blob_name(manifest["config"]["digest"]))
            runtime = config.get("config", {})
            labels = runtime.get("Labels", {})
            if not REQUIRED_LABELS <= labels.keys():
                raise ContainerError("OCI config lacks required labels")
            if labels["org.opencontainers.image.revision"] != source_sha:
                raise ContainerError("OCI revision label does not match candidate")
            if labels["org.opencontainers.image.version"] != version:
                raise ContainerError("OCI version label does not match candidate")
            user = str(runtime.get("User", ""))
            if not user or user in {"0", "root", "0:0"}:
                raise ContainerError("OCI runtime user must be non-root")
            if runtime.get("ExposedPorts"):
                raise ContainerError("BioMCP OCI image must not expose a service port")
            entrypoint = runtime.get("Entrypoint", [])
            if entrypoint != ["biomcp"]:
                raise ContainerError("OCI entrypoint must be biomcp")
            platform = descriptor["platform"]["architecture"]
            platform_digests[platform] = descriptor["digest"]
        index_digest = "sha256:" + hashlib.sha256(canonical_bytes(index)).hexdigest()
        return {
            "platforms": sorted(f"{os_name}/{arch}" for os_name, arch in actual),
            "platform_digests": platform_digests,
            "index_digest": index_digest,
            "non_root": True,
            "ports": [],
            "entrypoint": ["biomcp"],
            "labels_checked": True,
        }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--layout", type=Path, required=True)
    parser.add_argument("--record", type=Path, required=True)
    parser.add_argument("--source-sha", required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--run-id", required=True)
    parser.add_argument("--amd64-sha256", required=True)
    parser.add_argument("--arm64-sha256", required=True)
    parser.add_argument("--sbom", type=Path, required=True)
    args = parser.parse_args()
    evidence = inspect_layout(args.layout, args.source_sha, args.version)
    kind, target = ARTIFACTS["oci-index"]
    record = {
        "id": "oci-index",
        "kind": kind,
        "target": target,
        "filename": args.layout.name,
        "sha256": sha256_file(args.layout),
        "bytes": args.layout.stat().st_size,
        "source_sha": args.source_sha,
        "version": args.version,
        "stage_run_id": args.run_id,
        "provenance": {
            "builder": "docker buildx",
            "build_count": 1,
            "base": "debian:bookworm-slim@sha256:abd67ffcfa541b485a3dff59865ab629aa048a6c613e639d36e7456b0b229241",
            "sbom_sha256": sha256_file(args.sbom),
        },
        "evidence": evidence,
        "upstream": {
            "native-linux-x86_64": args.amd64_sha256,
            "native-linux-arm64": args.arm64_sha256,
        },
    }
    args.record.write_bytes(canonical_bytes(record))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
