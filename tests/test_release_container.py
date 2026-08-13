from __future__ import annotations

import hashlib
import importlib.util
import io
import sys
import tarfile
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
container = _module("release_container", "release/container.py")


def _layout(tmp_path: Path, *, user: str = "65532:65532", platforms=("amd64", "arm64")) -> Path:
    tmp_path.mkdir(parents=True, exist_ok=True)
    blobs: dict[str, bytes] = {}

    def blob(value: dict) -> str:
        data = candidate.canonical_bytes(value)
        digest = hashlib.sha256(data).hexdigest()
        blobs[digest] = data
        return f"sha256:{digest}"

    manifests = []
    for architecture in platforms:
        config_digest = blob(
            {
                "config": {
                    "User": user,
                    "Entrypoint": ["biomcp"],
                    "Labels": {
                        "org.opencontainers.image.source": "https://github.com/genomoncology/biomcp",
                        "org.opencontainers.image.revision": "a" * 40,
                        "org.opencontainers.image.version": "1.2.3",
                        "org.opencontainers.image.licenses": "MIT",
                        "org.opencontainers.image.created": "2026-08-12T00:00:00Z",
                    },
                }
            }
        )
        manifest_digest = blob({"schemaVersion": 2, "config": {"digest": config_digest}, "layers": []})
        manifests.append(
            {
                "digest": manifest_digest,
                "platform": {"os": "linux", "architecture": architecture},
            }
        )
    index = {"schemaVersion": 2, "manifests": manifests}
    path = tmp_path / "biomcp.oci.tar"
    with tarfile.open(path, "w") as archive:
        for name, data in {"index.json": candidate.canonical_bytes(index), **{f"blobs/sha256/{key}": value for key, value in blobs.items()}}.items():
            info = tarfile.TarInfo(name)
            info.size = len(data)
            archive.addfile(info, io.BytesIO(data))
    return path


def test_oci_layout_requires_exact_platforms_labels_nonroot_and_no_ports(tmp_path: Path) -> None:
    evidence = container.inspect_layout(_layout(tmp_path), "a" * 40, "1.2.3")
    assert evidence["platforms"] == ["linux/amd64", "linux/arm64"]
    assert evidence["non_root"] is True
    assert evidence["ports"] == []


def test_oci_layout_rejects_root_and_missing_architecture(tmp_path: Path) -> None:
    with pytest.raises(container.ContainerError, match="non-root"):
        container.inspect_layout(_layout(tmp_path / "root", user="0"), "a" * 40, "1.2.3")
    with pytest.raises(container.ContainerError, match="platforms mismatch"):
        container.inspect_layout(
            _layout(tmp_path / "one", platforms=("amd64",)), "a" * 40, "1.2.3"
        )


def test_dockerfile_only_copies_staged_bytes_and_context_excludes_source() -> None:
    dockerfile = (ROOT / "Dockerfile").read_text()
    dockerignore = (ROOT / ".dockerignore").read_text().splitlines()
    assert "cargo build" not in dockerfile
    assert "COPY --chmod=0755 dist/container/${TARGETARCH}/biomcp" in dockerfile
    assert "USER 65532:65532" in dockerfile
    assert "EXPOSE" not in dockerfile
    assert dockerignore == ["**", "!Dockerfile", "!dist/", "!dist/container/", "!dist/container/**"]
