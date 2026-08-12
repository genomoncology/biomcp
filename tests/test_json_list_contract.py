from __future__ import annotations

import json
import os
import subprocess
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[1]
RELEASE_BIN = Path(os.environ.get("BIOMCP_BIN", REPO_ROOT / "target" / "release" / "biomcp"))


def _run_json(*args: str) -> dict[str, Any]:
    assert RELEASE_BIN.exists(), f"missing release binary: {RELEASE_BIN}"
    result = subprocess.run(
        [str(RELEASE_BIN), "--json", *args],
        cwd=REPO_ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return json.loads(result.stdout)


def _entries(value: object) -> list[object]:
    if isinstance(value, list):
        return value
    return []


def _read(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def test_json_list_outputs_are_parseable_reference_objects() -> None:
    root = _run_json("list")
    gene = _run_json("list", "gene")
    variant = _run_json("list", "variant")

    assert any(entity.get("name") == "gene" for entity in _entries(root.get("entities")))
    root_refs = _entries(root.get("entries"))
    assert any("search all" in str(entry) for entry in root_refs)
    assert gene.get("entity") == "gene"
    assert any("get gene <symbol>" in str(command) for command in _entries(gene.get("entries")))
    assert any(
        "variant articles <id>" in str(command)
        and "enum:union|annotation|lexical" in str(command)
        for command in _entries(variant.get("entries"))
    )


def test_public_docs_document_json_list_shape() -> None:
    docs = _read("docs/user-guide/cli-reference.md")
    ux = _read("architecture/ux/cli-reference.md")

    for content in (docs, ux):
        assert "biomcp --json list" in content
        assert "biomcp --json list <entity>" in content
        assert "`entities`" in content
        assert "`entity`" in content
        assert "`entries`" in content
        assert "biomcp cache path" in content
        assert "plain text" in content
