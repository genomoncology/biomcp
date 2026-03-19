#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DATA_FILE="$ROOT/paper/data/workflow-adjudication.json"

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

OUT_DIR="${1:-$ROOT/paper/generated/workflows}"
mkdir -p "$OUT_DIR"

python3 - "$DATA_FILE" "$OUT_DIR" "$BIN" <<'PY'
from __future__ import annotations

import json
import shlex
import subprocess
import sys
import time
from pathlib import Path


def run() -> int:
    data_path = Path(sys.argv[1])
    out_dir = Path(sys.argv[2])
    biomcp_bin = Path(sys.argv[3])

    data = json.loads(data_path.read_text(encoding="utf-8"))
    workflows = data.get("workflows", [])

    if data.get("_is_stub") or not workflows:
        print(
            "Replace paper/data/workflow-adjudication.json with archived release data before running workflows.",
            file=sys.stderr,
        )
        return 1

    timing_rows: list[dict[str, object]] = []
    for workflow in workflows:
        workflow_id = workflow.get("id", "workflow")
        workflow_dir = out_dir / workflow_id
        workflow_dir.mkdir(parents=True, exist_ok=True)

        for mode_key, mode_name in (("compact_commands", "compact"), ("naive_commands", "naive")):
            mode_dir = workflow_dir / mode_name
            mode_dir.mkdir(parents=True, exist_ok=True)
            commands = workflow.get(mode_key, [])

            for index, command in enumerate(commands, start=1):
                argv = shlex.split(command)
                if argv and argv[0] == "biomcp":
                    argv[0] = str(biomcp_bin)

                output_path = mode_dir / f"{index:02d}.json"
                start = time.perf_counter()
                with output_path.open("w", encoding="utf-8") as handle:
                    subprocess.run(argv, check=True, stdout=handle)
                elapsed_s = time.perf_counter() - start
                timing_rows.append(
                    {
                        "workflow": workflow_id,
                        "mode": mode_name,
                        "step": index,
                        "command": command,
                        "output_path": str(output_path),
                        "elapsed_s": round(elapsed_s, 6),
                    }
                )

    (out_dir / "timings.json").write_text(json.dumps(timing_rows, indent=2), encoding="utf-8")
    print(f"Wrote workflow outputs and timings to {out_dir}")
    return 0


raise SystemExit(run())
PY
