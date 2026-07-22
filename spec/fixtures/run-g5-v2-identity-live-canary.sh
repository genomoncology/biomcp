#!/usr/bin/env bash
set -euo pipefail

repo_root="${1:-../..}"
repo_root="$(cd "$repo_root" && pwd)"
binary="${BIOMCP_BIN:-$repo_root/target/release/biomcp}"
panel="$repo_root/spec/fixtures/g5-v2-identity-panel.json"

uv run --no-sync python - "$binary" "$panel" <<'PY'
import json
import subprocess
import sys
from pathlib import Path

binary, panel_path = sys.argv[1:]
requests = json.loads(Path(panel_path).read_text(encoding="utf-8"))
completed = subprocess.run(
    [binary, "--no-cache", "--json", "variant", "articles", "--input", "-", "--limit", "50", "--verify-identity", "--debug-plan"],
    input=json.dumps(requests), capture_output=True, text=True,
)
try:
    payload = json.loads(completed.stdout)
except json.JSONDecodeError as error:
    print(json.dumps({"error": f"BioMCP returned non-JSON output: {error}"}, indent=2))
    raise SystemExit(1) from error
items = payload.get("items", [])
request_by_id = {request["request_id"]: request for request in requests}
expected_request_ids = set(request_by_id)
returned_request_ids = [item.get("request_id") for item in items]
items_by_id = {item.get("request_id"): item for item in items}
recognized_items = [items_by_id[request_id] for request_id in expected_request_ids if request_id in items_by_id]
retrieval_exact_routes = {"exact_lexical", "pubtator_variant"}

def supplied_aliases(request):
    return {
        f'{request["transcript"]}:{request["coding"]}',
        f'{request["gene"]} {request["coding"]}',
        f'{request["accession"]}:g.{request["position"]}{request["ref"]}>{request["alt"]}',
    }

summary = {
    "expected_request_ids": len(returned_request_ids) == 7 and len(set(returned_request_ids)) == 7 and set(returned_request_ids) == expected_request_ids,
    "total": len(items),
    "resolved": sum((item.get("resolution") or {}).get("status") == "resolved" for item in recognized_items),
    "with_exact_route": sum(any(retrieval_exact_routes.intersection(row.get("routes", [])) for row in item.get("results", [])) for item in recognized_items),
    "with_route_tied_alias": sum(any(retrieval_exact_routes.intersection(row.get("routes", [])) and supplied_aliases(request_by_id[item.get("request_id")]).intersection(row.get("matched_aliases", [])) for row in item.get("results", [])) for item in recognized_items),
    "with_source_status": sum(bool(item.get("source_status")) for item in recognized_items),
    "with_terminal_state": sum(isinstance(item.get("complete"), bool) and isinstance(item.get("truncated"), bool) and "error" in item for item in recognized_items),
}
positives = {"apc-grch38": "12901799", "atm-grch38": "32918381", "palb2-grch38": "39999518", "mlh1-grch38": "20864636"}
collisions = {"31749828", "24376681", "33656647"}
diagnostics = {
    "known_collision_confirmations": [],
    "schema_parse_failures": [],
    "missing_available_positives": [],
    "unavailable_outages": [],
}
for item in recognized_items:
    request_id = item.get("request_id")
    verification = (item.get("debug_plan") or {}).get("verification")
    if not isinstance(verification, dict):
        diagnostics["schema_parse_failures"].append(request_id)
    if not item.get("complete", False):
        diagnostics["unavailable_outages"].append(request_id)
    for row in item.get("results", []):
        if row.get("pmid") in collisions and (row.get("identity") or {}).get("status") == "confirmed":
            diagnostics["known_collision_confirmations"].append(row["pmid"])
for request_id, pmid in positives.items():
    item = items_by_id.get(request_id, {})
    if item.get("complete", False) and not any(
        row.get("pmid") == pmid and (row.get("identity") or {}).get("status") == "confirmed"
        for row in item.get("results", [])
    ):
        diagnostics["missing_available_positives"].append(request_id)
print(json.dumps({"identity_readiness": summary, "identity_diagnostics": diagnostics}, indent=2, sort_keys=True))
ready = completed.returncode == 0 and summary["expected_request_ids"] and all(value == 7 for key, value in summary.items() if key != "expected_request_ids") and not any(diagnostics[key] for key in ("known_collision_confirmations", "schema_parse_failures", "missing_available_positives"))
raise SystemExit(0 if ready else 1)
PY
