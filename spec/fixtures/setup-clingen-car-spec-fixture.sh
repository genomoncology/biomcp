#!/usr/bin/env bash
set -euo pipefail

workspace_root="$(cd "${1:-$PWD}" && pwd)"
cache_dir="$workspace_root/.cache"
env_file="$cache_dir/spec-clingen-car-env"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cleanup_script="$script_dir/cleanup-clingen-car-spec-fixture.sh"
mkdir -p "$cache_dir"
bash "$cleanup_script" "$workspace_root"

fixture_root="$(mktemp -d "$cache_dir/spec-clingen-car.XXXXXX")"
ready_file="$fixture_root/base-url"
server_log="$fixture_root/server.log"
request_log="$fixture_root/request.log"
: >"$request_log"

uv run --no-sync python - "$ready_file" "$request_log" <<'PY' >"$server_log" 2>&1 &
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlparse
import json
import sys

READY = Path(sys.argv[1])
REQUEST_LOG = Path(sys.argv[2])
CAIDS = {
    "NM_000038.6:c.847C>G": "CA16023172",
    "NM_000038.6:c.1A>G": "CA015543",
    "NC_000005.9:g.112175951A>G": "CA015543",
    "NM_000051.4:c.7271T>G": "CA151456",
    "NM_007294.4:c.5266dupC": "CA001621",
    "NM_000249.4:c.793C>T": "CA009197",
    "NM_024675.4:c.3113G>A": "CA168760",
    "NM_000314.8:c.388C>G": "CA000498",
    "NM_000546.6:c.215C>G": "CA397844357",
    "NM_004333.6:c.1799T>A": "CA123643",
}

def resolved(hgvs, caid=None):
    return {
        "@id": f"https://fixture.car/allele/{caid or CAIDS.get(hgvs, 'CA123643')}",
        "communityStandardTitle": [hgvs],
        "genomicAlleles": [],
        "transcriptAlleles": [],
        "externalRecords": {"dbSNP": [], "ClinVarVariations": []},
    }

def external_rich(hgvs):
    result = resolved(hgvs)
    result["externalRecords"] = {
        "dbSNP": [{"rs": number} for number in [9, 2, 2, 1, 8, 3, 7, 4, 6, 5]],
        "ClinVarVariations": [{"variationId": number} for number in [19, 12, 12, 11, 18, 13, 17, 14, 16, 15]],
    }
    return result

def send(handler, status, payload):
    body = json.dumps(payload).encode()
    handler.send_response(status)
    handler.send_header("Content-Type", "application/json")
    handler.send_header("Content-Length", str(len(body)))
    handler.send_header("X-CAR-Version", "fixture-617")
    handler.end_headers()
    handler.wfile.write(body)

class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        parsed = urlparse(self.path)
        query = parse_qs(parsed.query)
        hgvs = query.get("hgvs", [""])[0]
        REQUEST_LOG.write_text(REQUEST_LOG.read_text() + f"GET {self.path}\n")
        if parsed.path == "/healthz":
            send(self, 200, {"ok": True})
        elif hgvs == "NM_000001.1:c.1A>G":
            send(self, 200, external_rich(hgvs))
        elif hgvs == "NM_000002.1:c.2A>G":
            send(self, 200, {"@id": "_:CA"})
        elif hgvs == "NM_000003.1:c.3A>G":
            send(self, 200, {"@id": "_:CA", "genomicAlleles": "wrong"})
        elif hgvs == "NM_000004.1:c.4A>G":
            send(self, 503, {"error": "fixture outage"})
        else:
            send(self, 200, resolved(hgvs))

    def do_POST(self):
        parsed = urlparse(self.path)
        body = self.rfile.read(int(self.headers.get("Content-Length", "0"))).decode()
        REQUEST_LOG.write_text(REQUEST_LOG.read_text() + f"POST {self.path} {body!r}\n")
        inputs = [line for line in body.splitlines() if line]
        if "NM_000005.1:c.5A>G" in inputs:
            send(self, 200, [resolved(inputs[0])])
            return
        send(self, 200, [
            {"@id": "_:CA"} if value == "NM_000002.1:c.2A>G" else
            {"@id": "_:CA", "genomicAlleles": "wrong"} if value == "NM_000003.1:c.3A>G" else
            resolved(value)
            for value in inputs
        ])

    def log_message(self, format, *args):
        return

server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
READY.write_text(f"http://127.0.0.1:{server.server_port}\n")
server.serve_forever()
PY
server_pid=$!
for _ in $(seq 1 50); do
  test -s "$ready_file" && break
  kill -0 "$server_pid" 2>/dev/null || { cat "$server_log" >&2; exit 1; }
  sleep 0.1
done
test -s "$ready_file"
base_url="$(<"$ready_file")"
{
  printf 'export BIOMCP_CLINGEN_CAR_BASE=%q\n' "$base_url"
  printf 'export BIOMCP_CACHE_MODE=off\n'
  printf 'export BIOMCP_CLINGEN_CAR_PID=%q\n' "$server_pid"
  printf 'export BIOMCP_CLINGEN_CAR_ROOT=%q\n' "$fixture_root"
  printf 'export BIOMCP_CLINGEN_CAR_REQUEST_LOG=%q\n' "$request_log"
} >"$env_file"
printf '%s\n' "$fixture_root"
