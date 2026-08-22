#!/usr/bin/env python3
"""Measure BioMCP's real local tools/list context footprint."""

from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

import tiktoken


ROOT = Path(__file__).resolve().parents[1]
TOKENIZER = "cl100k_base"
TOOLS_LIST_BYTE_CEILING = 22_600
TOOLS_LIST_TOKEN_CEILING = 5_800
BIOMCP_DESCRIPTION_BYTE_CEILING = 4_000
TOKENIZER_CACHE = ROOT / "benchmarks" / "output-footprint" / "tokenizer-cache"
TOKENIZER_CACHE_FILE = TOKENIZER_CACHE / "9b5ad71b2ce5302211f9c61530b329a4922fc6a4"
TOKENIZER_CACHE_SHA256 = (
    "223921b76ee99bde995b7ff738513eef100fb51d18c93597a113bcffe865b2a7"
)


def _encoding() -> tiktoken.Encoding:
    try:
        digest = hashlib.sha256(TOKENIZER_CACHE_FILE.read_bytes()).hexdigest()
    except OSError as error:
        raise SystemExit(
            f"committed {TOKENIZER} tokenizer cache is unavailable: {error}"
        ) from error
    if digest != TOKENIZER_CACHE_SHA256:
        raise SystemExit(f"committed {TOKENIZER} tokenizer cache failed validation")
    os.environ["TIKTOKEN_CACHE_DIR"] = str(TOKENIZER_CACHE)
    return tiktoken.get_encoding(TOKENIZER)


def _binary() -> Path:
    configured = os.environ.get("BIOMCP_BIN")
    candidates = [Path(configured)] if configured else []
    candidates.extend([ROOT / "target/debug/biomcp", ROOT / "target/release/biomcp"])
    for candidate in candidates:
        if candidate.is_file():
            return candidate
    raise SystemExit("BioMCP binary not found; run `cargo build --bin biomcp` first")


def main() -> None:
    encoding = _encoding()
    result = subprocess.run(
        [str(_binary()), "mcp", "tools"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    )
    tools = json.loads(result.stdout)
    serialized = json.dumps(tools, ensure_ascii=False, separators=(",", ":"))
    tokens = len(encoding.encode(serialized))
    byte_count = len(serialized.encode())
    raw = next(tool for tool in tools if tool["name"] == "biomcp")
    raw_description_bytes = len(raw["description"].encode())
    print(f"tools: {', '.join(tool['name'] for tool in tools)}")
    print(f"tools/list UTF-8 bytes: {byte_count}")
    print(f"tools/list cl100k_base tokens: {tokens}")
    print(f"biomcp description UTF-8 bytes: {raw_description_bytes}")

    exceeded = False
    for label, measured, ceiling in (
        ("tools/list UTF-8 bytes", byte_count, TOOLS_LIST_BYTE_CEILING),
        ("tools/list cl100k_base tokens", tokens, TOOLS_LIST_TOKEN_CEILING),
        (
            "biomcp description UTF-8 bytes",
            raw_description_bytes,
            BIOMCP_DESCRIPTION_BYTE_CEILING,
        ),
    ):
        if measured > ceiling:
            print(f"{label}: {measured} (ceiling: {ceiling:,})", file=sys.stderr)
            exceeded = True

    if exceeded:
        print("largest tool descriptions:", file=sys.stderr)
        for tool in sorted(
            tools,
            key=lambda tool: len(tool["description"].encode()),
            reverse=True,
        ):
            description_bytes = len(tool["description"].encode())
            print(
                f"- {tool['name']}: {description_bytes:,} UTF-8 bytes",
                file=sys.stderr,
            )
        raise SystemExit(1)


if __name__ == "__main__":
    main()
