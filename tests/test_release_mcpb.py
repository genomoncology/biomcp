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


def _bundle(tmp_path: Path) -> tuple[Path, bytes, bytes]:
    manifest = mcpb.render_manifest(json.loads((ROOT / "manifest.json").read_text()), "1.2.3")
    macos = b"universal Mach-O"
    windows = b"MZ signed PE"
    bundle = tmp_path / "biomcp-1.2.3.mcpb"
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


def test_mcpb_tool_install_is_exact_version_and_integrity_pinned() -> None:
    script = (ROOT / "release/install-mcpb-tool.sh").read_text()
    assert "version=2.1.2" in script
    assert "sha512-goRbBC8ySo7SWb7tRzr+tL6FxDc4JPTRCdgfD2omba7freofvjq5rom1lBnYHZHo6Mizs1jAHJeN53aZbDoy8A==" in script
    assert "npm install --global --ignore-scripts" in script
    assert "mcpb --version" in script
