from __future__ import annotations

import json
import os
from pathlib import Path
import re
import runpy
import subprocess

import pytest


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


@pytest.mark.parametrize(
    ("cache_contents", "message"),
    [(None, "cache is unavailable"), (b"altered", "cache failed validation")],
)
def test_mcp_measurement_fails_closed_without_valid_committed_tokenizer_cache(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
    cache_contents: bytes | None,
    message: str,
) -> None:
    measurement = runpy.run_path(str(ROOT / "scripts/measure-mcp-tools.py"))
    encoding = measurement["_encoding"]
    cache_file = tmp_path / "tokenizer-cache"
    if cache_contents is not None:
        cache_file.write_bytes(cache_contents)
    monkeypatch.setitem(
        encoding.__globals__,
        "TOKENIZER_CACHE_FILE",
        cache_file,
    )

    with pytest.raises(SystemExit, match=message):
        encoding()


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


def test_real_tools_list_stays_within_context_budget(tmp_path: Path) -> None:
    env = dict(os.environ)
    env.setdefault("BIOMCP_BIN", str(ROOT / "target/debug/biomcp"))
    user_tokenizer_cache = tmp_path / "empty-user-tokenizer-cache"
    user_tokenizer_cache.mkdir()
    env["TIKTOKEN_CACHE_DIR"] = str(user_tokenizer_cache)
    result = subprocess.run(
        ["uv", "run", "--no-sync", "python", "scripts/measure-mcp-tools.py"],
        cwd=ROOT,
        env=env,
        capture_output=True,
        text=True,
        check=True,
    )
    assert list(user_tokenizer_cache.iterdir()) == []
    measurements = {
        label: int(value)
        for label, value in re.findall(r"^(.+): (\d+)$", result.stdout, re.MULTILINE)
    }
    assert measurements["tools/list UTF-8 bytes"] <= 16_000
    assert measurements["tools/list cl100k_base tokens"] <= 4_000
    assert measurements["biomcp description UTF-8 bytes"] <= 4_000
