#!/usr/bin/env bash
# Frozen ClinGen ERepo provider captures exercise the shipped CLI without network access.
set -euo pipefail

repo_root="$(cd "${1:-../..}" && pwd)"
binary="${BIOMCP_BIN:-$repo_root/target/spec/biomcp}"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ownership_helper="$script_dir/routine-fixture-ownership.sh"
# shellcheck source=fixture-supervisor.sh
source "$script_dir/fixture-supervisor.sh"
cache_dir="$repo_root/.cache"
kind="run-clingen-erepo"
prefix="spec-run-clingen-erepo."
mkdir -p "$cache_dir"
recover_fixture_orphans "$cache_dir" "$kind" "$prefix"
tmp="$(mktemp -d "$cache_dir/$prefix"XXXXXX)"
owner_arg="$(bash "$ownership_helper" new-owner "$kind" "$tmp")"
server_pid_file="$tmp/server-pid"
trap 'bash "$ownership_helper" cleanup "$repo_root" "$kind" "BIOMCP_RUN_CLINGEN_EREPO"' EXIT

prepare_fixture_supervisor_current_process
REPO_ROOT="$repo_root" PORT_FILE="$tmp/port" REQUESTS="$tmp/requests.jsonl" \
  start_fixture_supervisor "$kind" "$cache_dir" "$tmp" "$prefix" "$server_pid_file" \
  python3 - "$owner_arg" <<'PY' &
import json, os
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

root = Path(os.environ["REPO_ROOT"]) / "testdata/sources/clingen_erepo"
requests = Path(os.environ["REQUESTS"])
requests.touch()
apc_summary = (root / "apc-summary.json").read_bytes()
apc_detail = (root / "apc-detail.json").read_bytes()
pten_gene = json.loads((root / "pten-gene-limit-26.json").read_text())
# The receipted APC page has no p.cspec-svi-text element. Keep that absence
# explicit while still exercising the required HTML request and parser path.
apc_guideline = b"<!doctype html><html><body></body></html>"
summaries = {}
for path in root.glob("*-summary.json"):
    value = json.loads(path.read_text())
    for row in value.get("data", []): summaries[row["caId"]] = value
apc = json.loads(apc_summary)
miss = json.loads((root / "CA001621-summary.json").read_text())
extra = dict(apc["data"][0]); extra["uuid"] = "00000000-0000-0000-0000-000000000002"; extra["docVersion"] = "2.0.0"; extra["versionsList"] = ["2.0.0"]
multiple = {"status":{"code":200}, "metadata":{}, "data":[apc["data"][0], extra]}
class Handler(BaseHTTPRequestHandler):
    def log_message(self, *_): pass
    def send_bytes(self, code, body, content_type="application/json"):
        self.send_response(code); self.send_header("Content-Type", content_type); self.send_header("Content-Length", str(len(body))); self.end_headers(); self.wfile.write(body)
    def send_json(self, code, value): self.send_bytes(code, json.dumps(value).encode())
    def do_GET(self):
        requests.open("a").write(self.path + "\n")
        path, _, query = self.path.partition("?")
        if path == "/evrepo/api/classifications":
            params = dict(part.split("=", 1) for part in query.split("&"))
            start = int(params["matchSkip"]); size = int(params["matchLimit"])
            self.send_json(200, {"@context": pten_gene["@context"], "variantInterpretations": pten_gene["variantInterpretations"][start:start + size]}); return
        if path == "/evrepo/api/summary/classifications":
            caid = next((part[7:] for part in query.split("&") if part.startswith("values=")), "")
            if caid == "CA015543": self.send_bytes(200, apc_summary); return
            if caid == "CA001621": self.send_json(404, miss); return
            self.send_json(200, multiple if caid == "CA-MULTI" else summaries.get(caid, {"status":{"code":200}, "metadata":{}, "data":[]})); return
        if path.endswith("/34ea9707-51d8-44df-818d-f69b075295c5/doc/sepio/version/1.0.0"):
            self.send_bytes(200, apc_detail); return
        if path == "/evrepo/ui/classification/34ea9707-51d8-44df-818d-f69b075295c5" and query == "version=1.0.0":
            self.send_bytes(200, apc_guideline, "text/html; charset=utf-8"); return
        self.send_json(404, {"status":{"code":404}, "metadata":{}, "data":[]})
server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
Path(os.environ["PORT_FILE"]).write_text(str(server.server_port))
server.serve_forever()
PY
supervisor_pid=$!
for _ in $(seq 1 50); do test -s "$server_pid_file" && break; kill -0 "$supervisor_pid" 2>/dev/null || break; sleep .1; done
test -s "$server_pid_file"
server_pid="$(<"$server_pid_file")"
bash "$ownership_helper" write "$repo_root" "$kind" "$tmp" "$server_pid" "BIOMCP_RUN_CLINGEN_EREPO" "$owner_arg" >/dev/null
while [[ ! -s "$tmp/port" ]]; do sleep 0.05; done
fixture_port="$(<"$tmp/port")"
export BIOMCP_CLINGEN_EREPO_BASE="http://127.0.0.1:$fixture_port"
export BIOMCP_CACHE_MODE=off

markdown="$($binary variant erepo CA015543)"
summary="$($binary --json variant erepo CA015543)"
detail="$($binary --json variant erepo CA015543 --detail)"
pten="$($binary --json variant erepo CA000498)"
miss="$($binary --json variant erepo CA001621)"
multiple="$($binary --json variant erepo CA-MULTI)"
printf '["CA015543","CA001621","CA015543"]' > "$tmp/caids.json"
batch="$($binary --json variant erepo --input "$tmp/caids.json")"
gene="$($binary --json variant erepo --gene PTEN --limit 25 --offset 0)"
gene_second="$($binary --json variant erepo --gene PTEN --limit 25 --offset 25)"
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
result = request({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"variant_erepo","arguments":{"gene":"PTEN","limit":25}}})
open(os.environ["MCP_FILE"] + ".gene", "w").write(result["result"]["content"][0]["text"])
proc.terminate(); proc.wait()
PY
mcp="$(<"$tmp/mcp.json")"
gene_mcp="$(<"$tmp/mcp.json.gene")"
requests="$(<"$tmp/requests.jsonl")"
jq -n --arg markdown "$markdown" --argjson summary "$summary" --argjson detail "$detail" --argjson pten "$pten" --argjson miss "$miss" --argjson multiple "$multiple" --argjson batch "$batch" --argjson mcp "$mcp" --argjson gene "$gene" --argjson gene_second "$gene_second" --argjson gene_mcp "$gene_mcp" --argjson ambiguous "$ambiguous" --arg requests "$requests" '{
  plain_cli_reports_summary: ($markdown | contains("ClinGen ERepo expert assertions") and contains("Classification: Pathogenic")),
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
  gene_page_is_bounded_and_truthful: ($gene.returned == 25 and $gene.has_more and $gene.total == null and ($gene.results | length) == 25 and all($gene.results[]; (.hgvs | length) <= 3)),
  gene_second_page_is_reachable: ($gene_second.offset == 25 and $gene_second.returned == 1 and ($gene_second.has_more | not)),
  gene_cli_and_mcp_have_same_contract: ($gene == $gene_mcp),
  summary_and_detail_bounds_are_reported: ($detail.items[0].assertions[0].detail.body_bytes > 0),
  detail_cli_consumes_selected_source_plan: ($requests | contains("/evrepo/api/summary/classification/34ea9707-51d8-44df-818d-f69b075295c5/doc/sepio/version/1.0.0")),
  detail_cli_consumes_guideline_plan: ($requests | contains("/evrepo/ui/classification/34ea9707-51d8-44df-818d-f69b075295c5?version=1.0.0")),
  receipted_summary_and_detail_drive_cli: ($summary.items[0].assertions[0].assertion_id == "34ea9707-51d8-44df-818d-f69b075295c5" and $detail.items[0].assertions[0].detail.body_sha256 == "f6b1e4bfd2359a4d648626a87d487c4d92e5f2cc723de9347139218c03abad46"),
  provider_at_id_is_preserved_in_detail: ($detail.items[0].assertions[0].detail.provider_at_id == "https://cgerepoapi/evrepo/api/summary/classification/34ea9707-51d8-44df-818d-f69b075295c5/doc/sepio/version/1.0.0")
}'
