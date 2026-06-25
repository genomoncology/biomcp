#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["requests"]
# ///
"""CLI wrapper for variant -> protein-structure spike measurements."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

EXPERIMENT_DIR = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(EXPERIMENT_DIR))

from variant_structure_annotation import run_direct_join, run_existing_cli, write_result  # noqa: E402


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--approach", choices=["cli", "direct", "links", "all"], default="all")
    args = parser.parse_args()

    runs = []
    if args.approach in {"cli", "all"}:
        runs.append(run_existing_cli())
    if args.approach in {"direct", "all"}:
        runs.append(run_direct_join())
    if args.approach in {"links", "all"}:
        runs.append(run_direct_join(with_rcsb=True))

    for result in runs:
        print(json.dumps(write_result(result), indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
