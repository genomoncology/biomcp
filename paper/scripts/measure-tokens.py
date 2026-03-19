#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = ["tiktoken"]
# ///

from __future__ import annotations

import json
import sys
from pathlib import Path

import tiktoken

# Paths are relative to the repo root; invoke this script from there.
DEFAULT_INPUT_DIR = "paper/generated/workflows"
DEFAULT_OUTPUT_PATH = "paper/generated/workflows/token-summary.json"


def load_counts(directory: Path, encoding: tiktoken.Encoding) -> tuple[int, int]:
    token_total = 0
    byte_total = 0
    for path in sorted(directory.glob("*.json")):
        text = path.read_text(encoding="utf-8")
        token_total += len(encoding.encode(text))
        byte_total += len(text.encode("utf-8"))
    return token_total, byte_total


def resolve_paths(argv: list[str]) -> tuple[Path, Path]:
    if len(argv) > 3:
        raise SystemExit(
            "Usage: paper/scripts/measure-tokens.py [workflow_dir] [output_path]"
        )

    workflow_dir = Path(argv[1]) if len(argv) > 1 else Path(DEFAULT_INPUT_DIR)
    output_path = Path(argv[2]) if len(argv) > 2 else Path(DEFAULT_OUTPUT_PATH)
    return workflow_dir, output_path


def main(argv: list[str]) -> None:
    workflow_dir, output_path = resolve_paths(argv)
    encoding = tiktoken.get_encoding("cl100k_base")

    records: list[dict[str, object]] = []
    totals = {
        "compact_tokens": 0,
        "naive_tokens": 0,
        "compact_bytes": 0,
        "naive_bytes": 0,
    }

    for workflow_path in sorted(workflow_dir.iterdir()):
        if not workflow_path.is_dir():
            continue

        compact_tokens, compact_bytes = load_counts(workflow_path / "compact", encoding)
        naive_tokens, naive_bytes = load_counts(workflow_path / "naive", encoding)
        record = {
            "workflow": workflow_path.name,
            "compact_tokens": compact_tokens,
            "naive_tokens": naive_tokens,
            "token_reduction_pct": round(100 * (1 - compact_tokens / naive_tokens), 1)
            if naive_tokens
            else None,
            "compact_bytes": compact_bytes,
            "naive_bytes": naive_bytes,
            "byte_reduction_pct": round(100 * (1 - compact_bytes / naive_bytes), 1)
            if naive_bytes
            else None,
        }
        records.append(record)
        totals["compact_tokens"] += compact_tokens
        totals["naive_tokens"] += naive_tokens
        totals["compact_bytes"] += compact_bytes
        totals["naive_bytes"] += naive_bytes

    summary = {
        "tokenizer": "cl100k_base",
        "workflows": records,
        "totals": {
            **totals,
            "token_reduction_pct": round(
                100 * (1 - totals["compact_tokens"] / totals["naive_tokens"]),
                1,
            )
            if totals["naive_tokens"]
            else None,
            "byte_reduction_pct": round(
                100 * (1 - totals["compact_bytes"] / totals["naive_bytes"]),
                1,
            )
            if totals["naive_bytes"]
            else None,
        },
    }

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(summary, indent=2), encoding="utf-8")
    print(f"Wrote token summary to {output_path}")


if __name__ == "__main__":
    main(sys.argv)
