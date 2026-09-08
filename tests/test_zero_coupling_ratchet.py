from __future__ import annotations

import base64
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import tarfile


ROOT = Path(__file__).resolve().parents[1]
TOOL = ROOT / "tools" / "check-zero-coupling.py"
TOKEN = "bio" + "data"


def _module():
    spec = importlib.util.spec_from_file_location("zero_coupling", TOOL)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _inventory(path: Path, entries: dict[str, bytes]) -> Path:
    content = {
        base64.b64encode(name.encode()).decode(): hashlib.sha256(data).hexdigest()
        for name, data in entries.items()
    }
    path.write_text(json.dumps(content), encoding="utf-8")
    return path


def test_tracked_text_scan_is_default_deny_and_case_insensitive(tmp_path: Path) -> None:
    checker = _module()
    inventory = _inventory(tmp_path / "inventory.json", {})
    cases = {
        "Cargo.toml": f'[dependencies]\n{TOKEN}={{path="../private"}}',
        "src/lib.rs": f"use {TOKEN}::Trial;",
        "tests/value.rs": f"fn {TOKEN}_handoff() {{}}",
        "docs/claim.md": f"owned by {TOKEN}",
        "sdlc/tickets/active.md": f"waits on {TOKEN}",
        "Cargo.lock": f"source = 'git+https://example.test/{TOKEN}'",
        "unknown.xyzzy": TOKEN.swapcase(),
        "NO_EXTENSION": TOKEN,
        "sdlc/records/new.md": TOKEN,
        "generated/output.rs": TOKEN,
        "revision.txt": "cfafc69d27c9a2fc" + "74909f21692a418a8b17db83",
    }
    for name, text in cases.items():
        path = tmp_path / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
    assert checker.scan_files(tmp_path, list(cases), inventory) == sorted(cases)


def test_exact_historical_digest_passes_but_mutation_and_rename_fail(tmp_path: Path) -> None:
    checker = _module()
    name = "sdlc/records/known.md"
    data = f"historical {TOKEN}".encode()
    path = tmp_path / name
    path.parent.mkdir(parents=True)
    path.write_bytes(data)
    inventory = _inventory(tmp_path / "inventory.json", {name: data})
    assert checker.scan_files(tmp_path, [name], inventory) == []
    path.write_bytes(data + b" extra")
    assert checker.scan_files(tmp_path, [name], inventory)
    renamed = "sdlc/records/renamed.md"
    path.rename(tmp_path / renamed)
    assert checker.scan_files(tmp_path, [renamed], inventory)


def test_archive_scan_rejects_packaged_only_text_and_skips_binary(tmp_path: Path) -> None:
    checker = _module()
    archive_path = tmp_path / "package.crate"
    with tarfile.open(archive_path, "w:gz") as archive:
        for name, data in {
            "pkg/only-in-package.txt": TOKEN.encode(),
            "pkg/binary.bin": b"\0\xff" + TOKEN.encode(),
        }.items():
            info = tarfile.TarInfo(name)
            info.size = len(data)
            archive.addfile(info, io.BytesIO(data))
    assert checker.scan_archive(archive_path) == ["pkg/only-in-package.txt"]


def test_repository_historical_inventory_is_exact_and_current() -> None:
    checker = _module()
    assert checker.scan_files(ROOT, checker.tracked(ROOT)) == []
