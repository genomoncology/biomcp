from __future__ import annotations

import json
import os
from pathlib import Path
import re
import runpy
import subprocess
import sys

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


def test_installed_binary_prints_the_complete_mcp_catalog(tmp_path: Path) -> None:
    binary = Path(os.environ.get("BIOMCP_BIN", ROOT / "target" / "debug" / "biomcp"))
    assert binary.exists(), f"missing biomcp binary: {binary}"

    result = subprocess.run(
        [str(binary), "mcp", "tools"],
        cwd=tmp_path,
        capture_output=True,
        text=True,
        check=False,
    )

    assert result.returncode == 0, result.stderr
    tools = json.loads(result.stdout)
    assert [tool["name"] for tool in tools] == TOOLS
    for tool in tools:
        assert tool["description"].strip()
        assert isinstance(tool["inputSchema"], dict)
        assert tool["annotations"]["readOnlyHint"] is True


def test_measurement_reads_catalog_from_a_direct_binary_command(tmp_path: Path) -> None:
    binary = tmp_path / "biomcp"
    catalog = [
        {
            "name": "biomcp",
            "description": "Read-only command",
            "inputSchema": {},
            "annotations": {"readOnlyHint": True},
        }
    ]
    binary.write_text(
        "\n".join(
            [
                f"#!{sys.executable}",
                "import json",
                "import sys",
                "if sys.argv[1:] != ['mcp', 'tools']:",
                "    raise SystemExit(f'unexpected arguments: {sys.argv[1:]}')",
                f"print(json.dumps({catalog!r}))",
            ]
        ),
        encoding="utf-8",
    )
    binary.chmod(0o755)

    result = subprocess.run(
        ["uv", "run", "--no-sync", "python", "scripts/measure-mcp-tools.py"],
        cwd=ROOT,
        env=os.environ | {"BIOMCP_BIN": str(binary)},
        capture_output=True,
        text=True,
        check=False,
    )

    assert result.returncode == 0, result.stderr
    serialized = json.dumps(catalog, ensure_ascii=False, separators=(",", ":"))
    assert "tools: biomcp" in result.stdout
    assert f"tools/list UTF-8 bytes: {len(serialized.encode())}" in result.stdout
    assert "tools/list cl100k_base tokens: " in result.stdout
    assert "biomcp description UTF-8 bytes: 15" in result.stdout


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
