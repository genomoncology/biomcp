#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:?repo root required}"
ROOT="$(cd "$ROOT" && pwd)"
cleanup() {
  bash "$ROOT/spec/fixtures/cleanup-europepmc-cursor-fixture.sh" "$ROOT"
}
trap cleanup EXIT

bash "$ROOT/spec/fixtures/setup-europepmc-cursor-fixture.sh" "$ROOT"
# shellcheck source=/dev/null
. "$ROOT/.cache/spec-europepmc-cursor-env"

BIN="$ROOT/tools/biomcp-ci"
REQUESTS="$BIOMCP_EUROPEPMC_CURSOR_REQUESTS"

run_case() {
  local label="$1" limit="$2" offset="${3:-0}" phrase="${4:-cursor fixture}"
  : >"$REQUESTS"
  "$BIN" --json search article --source europepmc "$phrase" \
    --limit "$limit" --offset "$offset" >"$ROOT/.cache/europepmc-cursor-$label.json"
  python3 - "$label" "$REQUESTS" "$ROOT/.cache/europepmc-cursor-$label.json" <<'PY'
import json
import sys

label, requests_path, output_path = sys.argv[1], sys.argv[2], sys.argv[3]
rows = json.load(open(output_path)).get("results", [])
lines = [line for line in open(requests_path) if line.strip()]
print(json.dumps({
    f"{label}_rows": len(rows),
    f"{label}_search_requests": len(lines),
    f"{label}_first_pmid": rows[0]["pmid"] if rows else None,
    f"{label}_page_parameter_sent": any("page=" in line for line in lines),
    f"{label}_cursormark_sent": all("cursorMark=" in line for line in lines),
    f"{label}_distinct_pmids": len({row["pmid"] for row in rows}),
}))
PY
}

run_case limit50 50 0 "cursor fixture full"
run_case short 50 0 "cursor fixture short"
run_case exhaustion 50 0 "cursor fixture lying"
run_case offset25 10 25 "cursor fixture full"
