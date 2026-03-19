#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DATA_FILE="$ROOT/paper/data/normalization-benchmark.json"

discover_biomcp() {
  if [[ -n "${BIOMCP_BIN:-}" && -x "${BIOMCP_BIN}" ]]; then
    printf '%s\n' "${BIOMCP_BIN}"
    return 0
  fi

  if [[ -x "$ROOT/target/release/biomcp" ]]; then
    printf '%s\n' "$ROOT/target/release/biomcp"
    return 0
  fi

  if command -v biomcp >/dev/null 2>&1; then
    command -v biomcp
    return 0
  fi

  return 1
}

if ! BIN="$(discover_biomcp)"; then
  echo "biomcp binary not found; set BIOMCP_BIN, build ./target/release/biomcp, or install biomcp on PATH" >&2
  exit 1
fi

OUT_DIR="${1:-$ROOT/paper/generated/normalization}"
mkdir -p "$OUT_DIR"

python3 - "$DATA_FILE" "$OUT_DIR" "$BIN" <<'PY'
from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path


def slugify(text: str) -> str:
    slug = re.sub(r"[^A-Za-z0-9]+", "-", text.strip()).strip("-").lower()
    return slug or "case"


def run() -> int:
    data_path = Path(sys.argv[1])
    out_dir = Path(sys.argv[2])
    biomcp_bin = sys.argv[3]

    data = json.loads(data_path.read_text(encoding="utf-8"))
    if data.get("_is_stub"):
        print(
            "Replace paper/data/normalization-benchmark.json with archived release data before running normalization.",
            file=sys.stderr,
        )
        return 1

    manifests: list[dict[str, str]] = []
    groups = [
        (
            "gene_aliases",
            data.get("gene_aliases", {}).get("results", []),
            lambda row: [biomcp_bin, "--json", "search", "gene", "-q", row["query"], "--limit", "10"],
        ),
        (
            "drug_brands",
            data.get("drug_brands", {}).get("results", []),
            lambda row: [biomcp_bin, "--json", "search", "drug", "-q", row["query"], "--limit", "10"],
        ),
        (
            "variant_input_parsing",
            data.get("variant_input_parsing", {}).get("results", []),
            lambda row: [biomcp_bin, "--json", "get", "variant", row["query"]],
        ),
    ]

    if not any(rows for _, rows, _ in groups):
        print(
            "Replace paper/data/normalization-benchmark.json with archived release data before running normalization.",
            file=sys.stderr,
        )
        return 1

    for group_name, rows, command_builder in groups:
        group_dir = out_dir / group_name
        group_dir.mkdir(parents=True, exist_ok=True)
        for index, row in enumerate(rows, start=1):
            output_path = group_dir / f"{index:02d}-{slugify(row['query'])}.json"
            command = command_builder(row)
            with output_path.open("w", encoding="utf-8") as handle:
                subprocess.run(command, check=True, stdout=handle)
            manifests.append(
                {
                    "group": group_name,
                    "query": row["query"],
                    "command": " ".join(command),
                    "output_path": str(output_path),
                }
            )

    (out_dir / "manifest.json").write_text(json.dumps(manifests, indent=2), encoding="utf-8")
    print(f"Wrote normalization outputs to {out_dir}")
    return 0


raise SystemExit(run())
PY
