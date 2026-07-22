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

binary, panel_path = sys.argv[1:]
requests = json.load(open(panel_path, encoding="utf-8"))
completed = subprocess.run(
    [binary, "--no-cache", "--json", "variant", "articles", "--input", "-", "--limit", "50"],
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
print(json.dumps({"identity_readiness": summary}, indent=2, sort_keys=True))
ready = completed.returncode == 0 and summary["expected_request_ids"] and all(value == 7 for key, value in summary.items() if key != "expected_request_ids")
raise SystemExit(0 if ready else 1)
PY
