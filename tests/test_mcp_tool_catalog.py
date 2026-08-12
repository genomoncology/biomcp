from __future__ import annotations

import json
import os
from pathlib import Path
import re
import subprocess


ROOT = Path(__file__).resolve().parents[1]
TOOLS = [
    "biomcp",
    "search",
    "get",
    "variant_normalize_car",
    "variant_erepo",
    "gene_cspec",
    "variant_articles",
]


def test_public_inventory_matches_typed_catalog() -> None:
    catalog = (ROOT / "src/mcp/catalog.rs").read_text(encoding="utf-8")
    names = re.findall(r'name:\s*"([a-z_]+)"', catalog)
    assert names == TOOLS

    manifest = json.loads((ROOT / "manifest.json").read_text(encoding="utf-8"))
    assert [tool["name"] for tool in manifest["tools"]] == TOOLS
    catalog_descriptions = dict(
        re.findall(
            r'name:\s*"([a-z_]+)".*?description:\s*"(.*?)"',
            catalog,
            re.DOTALL,
        )
    )
    assert {tool["name"]: tool["description"] for tool in manifest["tools"]} == (
        catalog_descriptions
    )

    for path in (
        "docs/getting-started/claude-desktop.md",
        "docs/reference/mcp-server.md",
        "docs/blog/we-deleted-35-tools.md",
    ):
        text = (ROOT / path).read_text(encoding="utf-8")
        for name in TOOLS:
            assert name in text, f"{path} omitted {name}"


def test_real_tools_list_stays_within_context_budget() -> None:
    env = dict(os.environ)
    env.setdefault("BIOMCP_BIN", str(ROOT / "target/debug/biomcp"))
    result = subprocess.run(
        ["uv", "run", "--no-sync", "python", "scripts/measure-mcp-tools.py"],
        cwd=ROOT,
        env=env,
        capture_output=True,
        text=True,
        check=True,
    )
    measurements = {
        label: int(value)
        for label, value in re.findall(r"^(.+): (\d+)$", result.stdout, re.MULTILINE)
    }
    assert measurements["tools/list UTF-8 bytes"] <= 16_000
    assert measurements["tools/list cl100k_base tokens"] <= 4_000
    assert measurements["biomcp description UTF-8 bytes"] <= 4_000
