#!/usr/bin/env bash
# Frozen ClinGen ERepo provider captures exercise the shipped CLI without network access.
set -euo pipefail

repo_root="${1:-../..}"
repo_root="$(cd "$repo_root" && pwd)"
binary="${BIOMCP_BIN:-$repo_root/target/spec/biomcp}"
tmp="$(mktemp -d)"
trap 'kill "${server_pid:-}" 2>/dev/null || true; rm -rf "$tmp"' EXIT

REPO_ROOT="$repo_root" PORT_FILE="$tmp/port" uv run --no-sync python - <<'PY' &
import json, os
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

root = Path(os.environ["REPO_ROOT"]) / "testdata/sources/clingen_erepo"
summaries = {}
for path in root.glob("*-summary.json"):
    value = json.loads(path.read_text())
    for row in value.get("data", []): summaries[row["caId"]] = value
apc = summaries["CA015543"]
miss = json.loads((root / "CA001621-summary.json").read_text())
extra = dict(apc["data"][0]); extra["uuid"] = "00000000-0000-0000-0000-000000000002"; extra["docVersion"] = "2.0.0"; extra["versionsList"] = ["2.0.0"]
multiple = {"status":{"code":200}, "metadata":{}, "data":[apc["data"][0], extra]}
detail = json.loads((root / "apc-detail.json").read_text())
class Handler(BaseHTTPRequestHandler):
    def log_message(self, *_): pass
    def send(self, code, value):
        body = json.dumps(value).encode(); self.send_response(code); self.send_header("Content-Type", "application/json"); self.send_header("Content-Length", str(len(body))); self.end_headers(); self.wfile.write(body)
    def do_GET(self):
        path, _, query = self.path.partition("?")
        if path == "/evrepo/api/summary/classifications":
            caid = next((part[7:] for part in query.split("&") if part.startswith("values=")), "")
            if caid == "CA001621": self.send(404, miss); return
            self.send(200, multiple if caid == "CA-MULTI" else summaries.get(caid, {"status":{"code":200}, "metadata":{}, "data":[]})); return
        if path.endswith("/34ea9707-51d8-44df-818d-f69b075295c5/doc/sepio/version/1.0.0"):
            value = json.loads(json.dumps(detail)); value["data"]["@id"] = f"http://127.0.0.1:{self.server.server_port}{path}"; self.send(200, value); return
        self.send(404, {"status":{"code":404}, "metadata":{}, "data":[]})
server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
Path(os.environ["PORT_FILE"]).write_text(str(server.server_port))
server.serve_forever()
PY
server_pid=$!
while [[ ! -s "$tmp/port" ]]; do sleep 0.05; done
export BIOMCP_CLINGEN_EREPO_BASE="http://127.0.0.1:$(<"$tmp/port")"
export BIOMCP_CACHE_MODE=off

summary="$($binary --json variant erepo CA015543)"
detail="$($binary --json variant erepo CA015543 --detail)"
pten="$($binary --json variant erepo CA000498)"
miss="$($binary --json variant erepo CA001621)"
multiple="$($binary --json variant erepo CA-MULTI)"
printf '["CA015543","CA001621","CA015543"]' > "$tmp/caids.json"
batch="$($binary --json variant erepo --input "$tmp/caids.json")"
if "$binary" --json variant erepo CA-MULTI --detail >/dev/null 2>&1; then ambiguous=false; else ambiguous=true; fi
BINARY="$binary" MCP_FILE="$tmp/mcp.json" uv run --no-sync python - <<'PY'
import json, os, subprocess
proc = subprocess.Popen([os.environ["BINARY"], "mcp"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
def request(value):
    proc.stdin.write(json.dumps(value) + "\n"); proc.stdin.flush()
    return json.loads(proc.stdout.readline())
request({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"fixture","version":"1"}}})
proc.stdin.write(json.dumps({"jsonrpc":"2.0","method":"notifications/initialized","params":{}}) + "\n"); proc.stdin.flush()
result = request({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"variant_erepo","arguments":{"caids":["CA015543","CA001621","CA015543"]}}})
open(os.environ["MCP_FILE"], "w").write(result["result"]["content"][0]["text"])
proc.terminate(); proc.wait()
PY
mcp="$(<"$tmp/mcp.json")"
jq -n --argjson summary "$summary" --argjson detail "$detail" --argjson pten "$pten" --argjson miss "$miss" --argjson multiple "$multiple" --argjson batch "$batch" --argjson mcp "$mcp" --argjson ambiguous "$ambiguous" '{
  apc_summary_preserves_source_facts: ($summary.items[0].assertions[0].classification == "Pathogenic"),
  plain_ps4_has_no_explicit_strength: (($summary.items[0].assertions[0].criteria[] | select(.source_token == "PS4").explicit_strength) == null),
  default_strength_is_not_applied_strength: (($detail.items[0].assertions[0].detail.criteria[] | select(.code == "PS4").default_strength) == "Pathogenic Strong"),
  comment_strength_is_not_applied_strength: (($summary.items[0].assertions[0].criteria[] | select(.source_token == "PS4").explicit_strength) == null and any($detail.items[0].assertions[0].detail.criteria[]; .comments[] | contains("PS4_VeryStrong"))),
  met_and_unmet_are_independent: ([ $summary.items[0].assertions[0].criteria[].status ] | index("met") and index("unmet")),
  missing_unmet_coverage_is_not_empty: ($pten.items[0].assertions[0].unmet_codes_state == "not_provided"),
  healthy_exact_miss_is_empty_and_complete: ($miss.items[0].assertions == [] and $miss.complete),
  assertions_are_uuid_then_version_ordered: ($multiple.items[0].assertions | map(.assertion_id) == ["00000000-0000-0000-0000-000000000002", "34ea9707-51d8-44df-818d-f69b075295c5"]),
  multiple_assertions_require_explicit_selection: $ambiguous,
  selected_detail_keeps_version_and_citation_locator: ($detail.items[0].assertions[0].detail.source_url | endswith("/version/1.0.0") and any($detail.items[0].assertions[0].detail.criteria[]; any(.pmids[]; .pmid == 12901799 and (.locator | startswith("/evidenceLine/"))))),
  batch_preserves_order_and_duplicates: ($batch.items | map(.caid) == ["CA015543", "CA001621", "CA015543"]),
  cli_and_mcp_have_same_contract: ($batch == $mcp),
  summary_and_detail_bounds_are_reported: ($detail.items[0].assertions[0].detail.body_bytes > 0)
}'
