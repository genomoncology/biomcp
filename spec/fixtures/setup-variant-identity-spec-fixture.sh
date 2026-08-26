#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ownership_helper="$script_dir/routine-fixture-ownership.sh"
# shellcheck source=fixture-supervisor.sh
source "$script_dir/fixture-supervisor.sh"

workspace_root="${1:-$PWD}"
cache_dir="$workspace_root/.cache"
env_file="$cache_dir/spec-variant-identity-env"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cleanup_script="$script_dir/cleanup-variant-identity-spec-fixture.sh"

mkdir -p "$cache_dir"
bash "$cleanup_script" "$workspace_root"
recover_fixture_orphans "$cache_dir" "variant-identity" "spec-variant-identity."

fixture_root="$(mktemp -d "$cache_dir/spec-variant-identity.XXXXXX")"
owner_arg="$(bash "$ownership_helper" new-owner "variant-identity" "$fixture_root")"
ready_file="$fixture_root/base-url"
server_log="$fixture_root/server.log"
request_log="$fixture_root/request.log"
server_pid_file="$fixture_root/server-pid"
: >"$request_log"

prepare_fixture_supervisor_owner
start_fixture_supervisor "variant-identity" "$cache_dir" "$fixture_root" "spec-variant-identity." "$server_pid_file" \
  python3 - "$workspace_root" "$ready_file" "$request_log" "$owner_arg" <<'PY' >"$server_log" 2>&1 &
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlparse
import json
import sys

ROOT = Path(sys.argv[1])
READY = Path(sys.argv[2])
REQUEST_LOG = Path(sys.argv[3])
SEARCH_RESPONSE = json.loads(
    (ROOT / "testdata/sources/myvariant/search_brca1_contradictory_protein.json").read_text(
        encoding="utf-8"
    )
)
BRAF_MISSENSE_RESPONSE = (ROOT / "testdata/sources/myvariant/search_braf_missense_20260805.json").read_bytes()
BRAF_REVEL_RESPONSE = (ROOT / "testdata/sources/myvariant/search_braf_revel_20260805.json").read_bytes()
BRAF_V600E_RESPONSE = (ROOT / "testdata/sources/myvariant/search_braf_v600e_20260806.json").read_bytes()
BRAF_V600E_GRCH38_RESPONSE = (ROOT / "testdata/sources/myvariant/get_braf_v600e_grch38_20260806.json").read_bytes()
BRAF_V600E_GRCH37_RESPONSE = (ROOT / "testdata/sources/myvariant/get_braf_v600e.json").read_bytes()
GRID1_GRCH37_RESPONSE = (ROOT / "testdata/sources/myvariant/get_grid1_grch37_20260806.json").read_bytes()
GRID1_GRCH38_RESPONSE = (ROOT / "testdata/sources/myvariant/get_grid1_grch38_20260806.json").read_bytes()
PTEN_GRCH38_RESPONSE = (ROOT / "testdata/sources/myvariant/get_pten_grch38_20260806.json").read_bytes()
PTEN_DELETION_GRCH38_RESPONSE = (ROOT / "testdata/sources/myvariant/get_pten_deletion_grch38_20260806.json").read_bytes()
SMARCA4_REPEAT_RESPONSE = json.dumps({
    "_id": "chr19:g.11106928AAG[1]",
    "dbsnp": {"rsid": "rs876657378"},
    "clinvar": {"gene": {"symbol": "SMARCA4"}},
}).encode("utf-8")
MYD88_L265P_RESPONSE = (ROOT / "testdata/sources/myvariant/search_myd88_l265p_20260806.json").read_bytes()
CANCERHOTSPOTS_RESPONSES = {
    "/api/hotspots/single/byGene/BRAF": (ROOT / "testdata/sources/cancerhotspots/by_gene_braf_20260805.json").read_bytes(),
    "/api/hotspots/single/byGene/MYD88": (ROOT / "testdata/sources/cancerhotspots/by_gene_myd88_20260805.json").read_bytes(),
    "/api/hotspots/single/byGene/NOTAREALGENE664": (ROOT / "testdata/sources/cancerhotspots/by_gene_empty_20260805.json").read_bytes(),
}


def send_json(handler, status, payload):
    body = payload if isinstance(payload, bytes) else json.dumps(payload).encode("utf-8")
    handler.send_response(status)
    handler.send_header("Content-Type", "application/json")
    handler.send_header("Content-Length", str(len(body)))
    handler.end_headers()
    handler.wfile.write(body)


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        parsed = urlparse(self.path)
        with REQUEST_LOG.open("a", encoding="utf-8") as log:
            log.write(f"GET {self.path}\n")

        if parsed.path == "/healthz":
            send_json(self, 200, {"status": "ok"})
            return
        if parsed.path == "/v1/variant/chr7:g.140753336A%3ET":
            if parse_qs(parsed.query).get("assembly") == ["hg38"]:
                send_json(self, 200, BRAF_V600E_GRCH38_RESPONSE)
                return
            send_json(self, 404, {"code": 404, "success": False, "error": "Not Found."})
            return
        if parsed.path == "/v1/variant/chr7:g.140453136A%3ET":
            if parse_qs(parsed.query).get("assembly") == ["hg19"]:
                send_json(self, 200, BRAF_V600E_GRCH37_RESPONSE)
                return
            send_json(self, 404, {"code": 404, "success": False, "error": "Not Found."})
            return
        if parsed.path == "/v1/variant/chr10:g.87933119A%3EC":
            if parse_qs(parsed.query).get("assembly") == ["hg19"]:
                send_json(self, 200, GRID1_GRCH37_RESPONSE)
                return
            if parse_qs(parsed.query).get("assembly") == ["hg38"]:
                send_json(self, 200, GRID1_GRCH38_RESPONSE)
                return
            send_json(self, 404, {"code": 404, "success": False, "error": "Not Found."})
            return
        if parsed.path == "/v1/variant/chr10:g.87925512G%3EA":
            if parse_qs(parsed.query).get("assembly") == ["hg38"]:
                send_json(self, 200, PTEN_GRCH38_RESPONSE)
                return
            send_json(self, 404, {"code": 404, "success": False, "error": "Not Found."})
            return
        if parsed.path == "/v1/variant/chr10:g.87925512del":
            if parse_qs(parsed.query).get("assembly") == ["hg38"]:
                send_json(self, 200, PTEN_DELETION_GRCH38_RESPONSE)
                return
            send_json(self, 404, {"code": 404, "success": False, "error": "Not Found."})
            return
        if parsed.path == "/v1/variant/chr19:g.11106928AAG%5B1%5D":
            send_json(self, 200, SMARCA4_REPEAT_RESPONSE)
            return

        if parsed.path == "/v1/query":
            query = parse_qs(parsed.query).get("q", [""])[0]
            expected_proteins = ('dbnsfp.hgvsp:"p.M1783I"', 'dbnsfp.hgvsp:"p.M16I"')
            if query == "dbsnp.rsid:rs876657378":
                send_json(self, 200, {"total": 1, "hits": [json.loads(SMARCA4_REPEAT_RESPONSE)]})
                return
            if "dbnsfp.genename:BRCA1" in query and any(
                protein in query for protein in expected_proteins
            ):
                send_json(self, 200, SEARCH_RESPONSE)
                return
            if (
                "dbnsfp.genename:BRAF" in query
                and "snpeff.ann.effect:*missense_variant*" in query
            ):
                send_json(self, 200, BRAF_MISSENSE_RESPONSE)
                return
            if "dbnsfp.genename:BRAF" in query and "_exists_:dbnsfp.revel.score" in query:
                send_json(self, 200, BRAF_REVEL_RESPONSE)
                return
            if "dbnsfp.genename:BRAF" in query and 'dbnsfp.hgvsp:"p.V600E"' in query:
                send_json(self, 200, BRAF_V600E_RESPONSE)
                return
            if "dbnsfp.genename:MYD88" in query and 'dbnsfp.hgvsp:"p.L265P"' in query:
                send_json(self, 200, MYD88_L265P_RESPONSE)
                return
            send_json(self, 400, {"error": "unexpected fixture query"})
            return

        if parsed.path in CANCERHOTSPOTS_RESPONSES:
            send_json(self, 200, CANCERHOTSPOTS_RESPONSES[parsed.path])
            return

        send_json(self, 404, {"error": "fixture path not found"})

    def log_message(self, format, *args):
        return


server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
READY.write_text(f"http://127.0.0.1:{server.server_port}\n", encoding="utf-8")
server.serve_forever()
PY
supervisor_pid=$!
for _ in $(seq 1 50); do test -s "$server_pid_file" && break; kill -0 "$supervisor_pid" 2>/dev/null || break; sleep .1; done
test -s "$server_pid_file"
server_pid="$(<"$server_pid_file")"
cleanup_partial_setup() {
  kill -TERM -- "-$server_pid" 2>/dev/null || true
  wait "$supervisor_pid" 2>/dev/null || true
  rm -rf "$fixture_root"
}
trap cleanup_partial_setup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
trap 'exit 129' HUP

for _ in $(seq 1 50); do
  if [[ -s "$ready_file" ]]; then
    break
  fi
  if ! kill -0 "$server_pid" 2>/dev/null; then
    cat "$server_log" >&2
    exit 1
  fi
  sleep 0.1
done

test -s "$ready_file"
base_url="$(cat "$ready_file")"

for _ in $(seq 1 50); do
  if python3 - "$base_url/healthz" <<'PY' >/dev/null 2>&1
from urllib.request import urlopen
import sys

with urlopen(sys.argv[1], timeout=1) as response:
    if response.status != 200:
        raise SystemExit(1)
PY
  then
    break
  fi
  if ! kill -0 "$server_pid" 2>/dev/null; then
    cat "$server_log" >&2
    exit 1
  fi
  sleep 0.1
done

python3 - "$base_url/healthz" <<'PY' >/dev/null
from urllib.request import urlopen
import sys

with urlopen(sys.argv[1], timeout=1) as response:
    if response.status != 200:
        raise SystemExit(1)
PY

{
  printf 'export BIOMCP_MYVARIANT_BASE=%q\n' "$base_url/v1"
  printf 'export BIOMCP_CANCERHOTSPOTS_BASE=%q\n' "$base_url"
  printf 'export BIOMCP_CACHE_MODE=off\n'
  printf 'export BIOMCP_VARIANT_IDENTITY_REQUEST_LOG=%q\n' "$request_log"
} >"$env_file"

bash "$ownership_helper" write "$workspace_root" "variant-identity" "$fixture_root" "$server_pid" "BIOMCP_VARIANT_IDENTITY" "$owner_arg" >/dev/null
trap - EXIT INT TERM HUP
printf '%s\n' "$fixture_root"
