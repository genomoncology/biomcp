#!/usr/bin/env bash
# Frozen CSpec fixture exercises the public CLI and typed MCP without CSpec network access.
set -euo pipefail

repo_root="${1:-../..}"
repo_root="$(cd "$repo_root" && pwd)"
binary="${BIOMCP_BIN:-$repo_root/target/spec/biomcp}"
tmp="$(mktemp -d)"
trap 'kill "${server_pid:-}" 2>/dev/null || true; rm -rf "$tmp"' EXIT

REPO_ROOT="$repo_root" PORT_FILE="$tmp/port" REQUESTS_FILE="$tmp/requests.json" uv run --no-sync python - <<'PY' &
import json
import os
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

requests = []

def envelope(data):
    return {"status": {"code": 200}, "metadata": {"serviceVersion": "fixture"}, "data": data}

def iri(spec, version):
    return f"http://127.0.0.1:{server.server_port}/cspec/SequenceVariantInterpretation/id/{spec}/version/{version}"

class Handler(BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def send_json(self, value):
        body = json.dumps(value, separators=(",", ":")).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        path, _, _query = self.path.partition("?")
        requests.append(self.path)
        if path == "/cspec/Gene/id/BRAF/SequenceVariantInterpretation/version":
            self.send_json(envelope([
                {"@id": iri("GN004", "1.0.0")},
                {"@id": iri("GN049", "2.1.0")},
            ]))
            return
        if path == "/cspec/Gene/id/ATM/SequenceVariantInterpretation/version":
            self.send_json(envelope([{"@id": iri("GN020", "1.5.1")}]))
            return
        if path == "/cspec/SequenceVariantInterpretation/id/GN020/version/1.5.1":
            self.send_json(envelope({
                "entType": "SequenceVariantInterpretation",
                "entId": "GN020",
                "entContent": {"namespace": "GN020", "version": "1.5"},
                "ld": {"CriteriaCode": [
                    {"entType": "CriteriaCode", "entContent": {
                        "label": "PS3", "sepioID": "SEPIO:0000006", "gene": "ATM",
                        "instructionsToUse": "Use validated functional assays.",
                        "references": ["PMID:123456"], "strengthDescriptor": ["Strong"]
                    }},
                    {"entType": "CriteriaCode", "entContent": {
                        "label": "PM2", "sepioID": "SEPIO:0000007", "gene": "ATM",
                        "additionalComments": "Frequency thresholds are panel source text.",
                        "references": ["PMID:789012"]
                    }}
                ]},
                "ldFor": {"disease": "Ataxia-telangiectasia", "vcep": "ATM VCEP"}
            }))
            return
        self.send_response(404)
        self.end_headers()

server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
Path(os.environ["PORT_FILE"]).write_text(str(server.server_port))
try:
    server.serve_forever()
finally:
    Path(os.environ["REQUESTS_FILE"]).write_text(json.dumps(requests))
PY
server_pid=$!
while [[ ! -s "$tmp/port" ]]; do sleep 0.05; done
export BIOMCP_CLINGEN_CSPEC_BASE="http://127.0.0.1:$(<"$tmp/port")"
export BIOMCP_CACHE_DIR="$tmp/cache"
export BIOMCP_CACHE_MODE=off

braf="$("$binary" --json gene cspec BRAF)"
atm_iri="${BIOMCP_CLINGEN_CSPEC_BASE}/cspec/SequenceVariantInterpretation/id/GN020/version/1.5.1"
selected="$("$binary" --json gene cspec ATM --version "$atm_iri" --limit 1)"
capture_id="$(printf '%s' "$selected" | jq -er '.capture_id')"
raw_sha256="$("$binary" gene cspec document "$capture_id" | sha256sum | cut -d' ' -f1)"
raw_length="$("$binary" gene cspec document "$capture_id" | wc -c | tr -d ' ')"
page_two="$("$binary" --json gene cspec ATM --capture-id "$capture_id" --offset 1 --limit 1)"

BINARY="$binary" ATM_IRI="$atm_iri" MCP_FILE="$tmp/mcp.json" uv run --no-sync python - <<'PY'
import json, os, subprocess
proc = subprocess.Popen([os.environ["BINARY"], "mcp"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
def request(value):
    proc.stdin.write(json.dumps(value) + "\n"); proc.stdin.flush()
    return json.loads(proc.stdout.readline())
request({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"fixture","version":"1"}}})
proc.stdin.write(json.dumps({"jsonrpc":"2.0","method":"notifications/initialized","params":{}}) + "\n"); proc.stdin.flush()
result = request({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"gene_cspec","arguments":{"gene":"ATM","version_iri":os.environ["ATM_IRI"],"limit":1}}})
open(os.environ["MCP_FILE"], "w").write(result["result"]["content"][0]["text"])
proc.terminate(); proc.wait()
PY
mcp="$(<"$tmp/mcp.json")"

kill "$server_pid"
wait "$server_pid" 2>/dev/null || true
server_pid=""
requests="$(<"$tmp/requests.json")"

jq -n --argjson braf "$braf" --argjson selected "$selected" --argjson page_two "$page_two" --argjson mcp "$mcp" --arg raw_sha256 "$raw_sha256" --arg raw_length "$raw_length" --argjson requests "$requests" '{
  braf_keeps_gn004_and_gn049: ([ $braf.manifest[].resource_iri ] | any(endswith("/GN004/version/1.0.0")) and any(endswith("/GN049/version/2.1.0"))),
  atm_full_iri_is_distinct_from_display_version: ($selected.resource_iri | endswith("/GN020/version/1.5.1") and $selected.display_version == "1.5"),
  same_capture_raw_sha256_and_length_match: ($selected.source_sha256 == $raw_sha256 and ($selected.byte_length | tostring) == $raw_length),
  raw_document_does_not_refetch_cspec: ([ $requests[] | select(contains("/GN020/version/1.5.1")) ] | length == 1),
  criteria_pages_are_provider_identity_ordered: ($selected.criteria[0].source_code == "PS3" and $page_two.criteria[0].source_code == "PM2"),
  cli_and_mcp_capture_page_match: ($selected == $mcp),
  criteria_do_not_claim_interpretation: ([ $selected.criteria[] | has("applicability") or has("met") or has("unmet") or has("recommendation") or has("classification") ] | any | not)
}'
