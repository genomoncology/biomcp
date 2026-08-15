#!/usr/bin/env python3
"""Measure BioMCP's real local tools/list context footprint."""

from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import subprocess

import tiktoken


ROOT = Path(__file__).resolve().parents[1]
TOKENIZER = "cl100k_base"
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
    process = subprocess.Popen(
        [str(_binary()), "serve"],
        cwd=ROOT,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1,
    )

    def request(message: dict[str, object]) -> dict[str, object]:
        assert process.stdin is not None
        assert process.stdout is not None
        process.stdin.write(json.dumps(message, separators=(",", ":")) + "\n")
        process.stdin.flush()
        line = process.stdout.readline()
        if not line:
            raise RuntimeError("BioMCP closed before returning an MCP response")
        return json.loads(line)

    try:
        request(
            {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {"name": "measure-mcp-tools", "version": "1"},
                },
            }
        )
        assert process.stdin is not None
        process.stdin.write(
            '{"jsonrpc":"2.0","method":"notifications/initialized"}\n'
        )
        process.stdin.flush()
        tools = request(
            {"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}
        )["result"]["tools"]
        serialized = json.dumps(tools, ensure_ascii=False, separators=(",", ":"))
        tokens = len(encoding.encode(serialized))
        raw = next(tool for tool in tools if tool["name"] == "biomcp")
        print(f"tools: {', '.join(tool['name'] for tool in tools)}")
        print(f"tools/list UTF-8 bytes: {len(serialized.encode())}")
        print(f"tools/list cl100k_base tokens: {tokens}")
        print(f"biomcp description UTF-8 bytes: {len(raw['description'].encode())}")
    finally:
        process.kill()
        process.wait()


if __name__ == "__main__":
    main()
