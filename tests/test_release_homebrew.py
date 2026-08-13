from __future__ import annotations

import importlib.util
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
homebrew = _module("release_homebrew", "release/homebrew.py")


def _record(artifact_id: str, architecture: str) -> dict:
    filename = f"biomcp-darwin-{architecture}.tar.gz"
    return {
        "id": artifact_id,
        "kind": "native",
        "target": "aarch64-apple-darwin" if architecture == "arm64" else "x86_64-apple-darwin",
        "filename": filename,
        "sha256": ("a" if architecture == "arm64" else "b") * 64,
        "bytes": 100,
        "source_sha": "c" * 40,
        "version": "1.2.3",
        "stage_run_id": "42",
        "provenance": {},
        "evidence": {
            "binary_sha256": ("d" if architecture == "arm64" else "e") * 64,
            "signing": {"biomcp": {"notary_status": "Accepted"}},
        },
    }


def test_formula_is_derived_from_exact_staged_archives_and_identity() -> None:
    template = (ROOT / "Formula/biomcp.rb").read_text()
    rendered = homebrew.render(
        template,
        _record("native-macos-arm64", "arm64"),
        _record("native-macos-x86_64", "x86_64"),
    )
    assert "__" not in rendered
    assert 'version "1.2.3"' in rendered
    assert "/releases/download/v1.2.3/biomcp-darwin-arm64.tar.gz" in rendered
    assert 'assert_match "cccccccc", output' in rendered
    assert 'bin.install_symlink "biomcp" => "biomcp-cli"' in rendered


def test_formula_refuses_unsigned_or_mixed_candidate_sources() -> None:
    template = (ROOT / "Formula/biomcp.rb").read_text()
    arm = _record("native-macos-arm64", "arm64")
    intel = _record("native-macos-x86_64", "x86_64")
    arm["evidence"]["signing"] = {}
    with pytest.raises(homebrew.FormulaError, match="unsigned"):
        homebrew.render(template, arm, intel)
    arm = _record("native-macos-arm64", "arm64")
    intel["source_sha"] = "f" * 40
    with pytest.raises(homebrew.FormulaError, match="identities disagree"):
        homebrew.render(template, arm, intel)


def test_homebrew_cache_key_uses_final_immutable_url(tmp_path: Path) -> None:
    url = homebrew.final_url("1.2.3", "biomcp-darwin-arm64.tar.gz")
    path = homebrew.homebrew_cache_path(tmp_path, url, "biomcp-darwin-arm64.tar.gz")
    assert path.parent == tmp_path / "downloads"
    assert path.name.endswith("--biomcp-darwin-arm64.tar.gz")
    assert len(path.name.split("--", 1)[0]) == 64


def test_formula_rejects_mutable_or_unsafe_identity() -> None:
    with pytest.raises(homebrew.FormulaError, match="invalid formula version"):
        homebrew.final_url("latest", "biomcp.tar.gz")
    with pytest.raises(homebrew.FormulaError, match="filename"):
        homebrew.final_url("1.2.3", "../biomcp.tar.gz")
