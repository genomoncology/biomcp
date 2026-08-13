#!/usr/bin/env python3
"""Deterministically assemble one BioMCP native archive or binary wheel."""

from __future__ import annotations

import argparse
import base64
import csv
import gzip
import hashlib
import io
import tarfile
import zipfile
from pathlib import Path

from candidate import ARTIFACTS, canonical_bytes, sha256_file

EPOCH = (1980, 1, 1, 0, 0, 0)


def _zip_entry(archive: zipfile.ZipFile, name: str, data: bytes, mode: int) -> None:
    info = zipfile.ZipInfo(name, EPOCH)
    info.compress_type = zipfile.ZIP_DEFLATED
    info.external_attr = mode << 16
    archive.writestr(info, data)


def native_archive(binary: Path, output: Path, windows: bool) -> None:
    data = binary.read_bytes()
    if windows:
        with zipfile.ZipFile(output, "w") as archive:
            _zip_entry(archive, "biomcp.exe", data, 0o100755)
        return
    tar_buffer = io.BytesIO()
    with tarfile.open(fileobj=tar_buffer, mode="w") as archive:
        info = tarfile.TarInfo("biomcp")
        info.size = len(data)
        info.mode = 0o755
        info.mtime = 0
        archive.addfile(info, io.BytesIO(data))
    with output.open("wb") as raw:
        with gzip.GzipFile(filename="", mode="wb", fileobj=raw, mtime=0) as compressed:
            compressed.write(tar_buffer.getvalue())


def _record_hash(data: bytes) -> str:
    encoded = base64.urlsafe_b64encode(hashlib.sha256(data).digest()).rstrip(b"=").decode()
    return f"sha256={encoded}"


def wheel(full: Path, shim: Path, output: Path, version: str, tag: str, windows: bool) -> None:
    dist = f"biomcp_cli-{version}"
    suffix = ".exe" if windows else ""
    entries: list[tuple[str, bytes, int]] = [
        (f"{dist}.data/scripts/biomcp{suffix}", full.read_bytes(), 0o100755),
        (f"{dist}.data/scripts/biomcp-cli{suffix}", shim.read_bytes(), 0o100755),
        (
            f"{dist}.dist-info/METADATA",
            (
                "Metadata-Version: 2.3\n"
                "Name: biomcp-cli\n"
                f"Version: {version}\n"
                "Summary: Biomedical MCP command-line interface\n"
                "License: MIT\n"
                "Requires-Python: >=3.10\n"
            ).encode(),
            0o100644,
        ),
        (
            f"{dist}.dist-info/WHEEL",
            f"Wheel-Version: 1.0\nGenerator: biomcp-release\nRoot-Is-Purelib: false\nTag: {tag}\n".encode(),
            0o100644,
        ),
    ]
    rows = [[name, _record_hash(data), str(len(data))] for name, data, _ in entries]
    record_name = f"{dist}.dist-info/RECORD"
    record = io.StringIO(newline="")
    writer = csv.writer(record, lineterminator="\n")
    writer.writerows([*rows, [record_name, "", ""]])
    entries.append((record_name, record.getvalue().encode(), 0o100644))
    with zipfile.ZipFile(output, "w") as archive:
        for name, data, mode in entries:
            _zip_entry(archive, name, data, mode)


def record(
    artifact_id: str,
    artifact: Path,
    source_sha: str,
    version: str,
    run_id: str,
    evidence: dict[str, object],
) -> dict[str, object]:
    kind, target = ARTIFACTS[artifact_id]
    return {
        "id": artifact_id,
        "kind": kind,
        "target": target,
        "filename": artifact.name,
        "sha256": sha256_file(artifact),
        "bytes": artifact.stat().st_size,
        "source_sha": source_sha,
        "version": version,
        "stage_run_id": run_id,
        "provenance": {"builder": "release/package.py", "build_count": 1},
        "evidence": evidence,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--kind", choices=["native", "wheel"], required=True)
    parser.add_argument("--artifact-id", choices=sorted(ARTIFACTS), required=True)
    parser.add_argument("--biomcp", type=Path, required=True)
    parser.add_argument("--shim", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--record", type=Path, required=True)
    parser.add_argument("--source-sha", required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--run-id", required=True)
    parser.add_argument("--wheel-tag")
    parser.add_argument("--windows", action="store_true")
    args = parser.parse_args()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    if args.kind == "native":
        native_archive(args.biomcp, args.output, args.windows)
    else:
        if args.shim is None or args.wheel_tag is None:
            parser.error("wheel requires --shim and --wheel-tag")
        wheel(
            args.biomcp,
            args.shim,
            args.output,
            args.version,
            args.wheel_tag,
            args.windows,
        )
    value = record(
        args.artifact_id,
        args.output,
        args.source_sha,
        args.version,
        args.run_id,
        {"package_inspection": "pending"},
    )
    args.record.write_bytes(canonical_bytes(value))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
