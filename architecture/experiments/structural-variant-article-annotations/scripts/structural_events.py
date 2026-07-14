#!/usr/bin/env -S uv run --no-project
"""Annotate local article JSON with experimental structural events."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

from structural_variant_annotations import render_jsonl


def _documents(payload: Any) -> list[dict[str, str]]:
    if isinstance(payload, dict) and isinstance(payload.get("documents"), list):
        return payload["documents"]
    if isinstance(payload, list):
        return payload
    raise ValueError("input must be a document list or an object with a documents list")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--input", type=Path, required=True, help="local JSON corpus/document file"
    )
    parser.add_argument("--output", type=Path, help="JSONL output (stdout by default)")
    args = parser.parse_args()

    rendered = render_jsonl(_documents(json.loads(args.input.read_text())))
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered)
    else:
        sys.stdout.write(rendered)


if __name__ == "__main__":
    main()
