#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ownership_helper="$script_dir/routine-fixture-ownership.sh"
# shellcheck source=fixture-supervisor.sh
source "$script_dir/fixture-supervisor.sh"

workspace_root="$(realpath -e "${1:-$PWD}")"
cache_dir="$workspace_root/.cache"
env_file="$cache_dir/spec-provider-contract-env"
cleanup_script="$script_dir/cleanup-provider-contract-spec-fixture.sh"
mkdir -p "$cache_dir"
cache_dir="$(realpath -e "$cache_dir")"
bash "$cleanup_script" "$workspace_root"
recover_provider_contract_orphans "$cache_dir"

fixture_root="$(mktemp -d "$cache_dir/spec-provider-contract.XXXXXX")"
owner_arg="$(bash "$ownership_helper" new-owner "provider-contract" "$fixture_root")"
ready_file="$fixture_root/base-url"
server_pid_file="$fixture_root/server-pid"
server_log="$fixture_root/server.log"
request_log="$fixture_root/request.log"
ema_dir="$fixture_root/ema-human"
who_dir="$fixture_root/who-pq"
: >"$request_log"
cp -R "$script_dir/ema-human" "$ema_dir"
cp -R "$script_dir/who-pq" "$who_dir"
find "$ema_dir" "$who_dir" -type f -exec touch {} +
prepare_fixture_supervisor_owner

start_fixture_supervisor "$cache_dir" "$fixture_root" "spec-provider-contract." "$server_pid_file" \
  python3 - "$workspace_root" "$ready_file" "$request_log" "$owner_arg" <<'PY' >"$server_log" 2>&1 &
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlparse
import json
import sys

ROOT = Path(sys.argv[1])
READY = Path(sys.argv[2])
REQUEST_LOG = Path(sys.argv[3])
SOURCES = ROOT / "testdata/sources"


def fixture(path):
    return (SOURCES / path).read_bytes()


MYCHEM = {
    "Keytruda": fixture("mychem/query_keytruda_get_20260811.json"),
    "pembrolizumab": fixture("mychem/query_pembrolizumab_get_20260811.json"),
    "trastuzumab": fixture("mychem/query_trastuzumab_search_20260811.json"),
    'drugcentral.drug_use.indication.concept_name:"Marfan syndrome"': fixture(
        "mychem/query_marfan_indication_20260811.json"
    ),
    "imatinib": fixture("mychem/query_imatinib_get_20260811.json"),
    "warfarin": fixture("mychem/query_warfarin_get_20260811.json"),
}
OPENFDA_LABEL = fixture("openfda/label_keytruda_20260811.json")
OPENFDA_DRUGSFDA = fixture("openfda/drugsfda_imatinib_20260811.json")


def send(handler, status, body):
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
            send(self, 200, b'{"status":"ok"}')
            return
        if parsed.path == "/mychem/v1/query":
            query = parse_qs(parsed.query).get("q", [""])[0]
            if query == "fixture-provider-failure":
                send(self, 503, b'{"error":"synthetic provider failure"}')
                return
            body = MYCHEM.get(query)
            if body is not None:
                send(self, 200, body)
                return
        if parsed.path == "/openfda/drug/label.json":
            search = parse_qs(parsed.query).get("search", [""])[0].lower()
            if "keytruda" in search or "pembrolizumab" in search:
                send(self, 200, OPENFDA_LABEL)
                return
        if parsed.path == "/openfda/drug/drugsfda.json":
            send(self, 200, OPENFDA_DRUGSFDA)
            return

        send(self, 404, b'{"error":"fixture route not found"}')

    def log_message(self, _format, *_args):
        return


server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
READY.write_text(f"http://127.0.0.1:{server.server_port}\n", encoding="utf-8")
server.serve_forever()
PY
supervisor_pid=$!
server_pid=""
cleanup_incomplete_setup() {
  if [[ -s "$server_pid_file" ]]; then
    server_pid="$(cat "$server_pid_file")"
    [[ "$server_pid" =~ ^[1-9][0-9]*$ ]] && kill -TERM -- "-$server_pid" 2>/dev/null || true
    wait "$supervisor_pid" 2>/dev/null || true
  else
    kill -TERM "$supervisor_pid" 2>/dev/null || true
    wait "$supervisor_pid" 2>/dev/null || true
    rm -rf "$fixture_root"
  fi
}
trap cleanup_incomplete_setup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
trap 'exit 129' HUP

for _ in $(seq 1 50); do
  [[ -s "$server_pid_file" ]] && server_pid="$(cat "$server_pid_file")"
  [[ -s "$ready_file" && "$server_pid" =~ ^[1-9][0-9]*$ ]] && break
  kill -0 "$supervisor_pid" 2>/dev/null || { cat "$server_log" >&2; exit 1; }
  sleep 0.1
done
test -s "$ready_file"
[[ "$server_pid" =~ ^[1-9][0-9]*$ ]]
base_url="$(cat "$ready_file")"
for _ in $(seq 1 50); do
  if curl --fail --silent "$base_url/healthz" >/dev/null; then break; fi
  kill -0 "$server_pid" 2>/dev/null || { cat "$server_log" >&2; exit 1; }
  sleep 0.1
done
curl --fail --silent "$base_url/healthz" >/dev/null

{
  printf 'export BIOMCP_MYCHEM_BASE=%q\n' "$base_url/mychem/v1"
  printf 'export BIOMCP_OPENFDA_BASE=%q\n' "$base_url/openfda"
  printf 'export BIOMCP_EMA_DIR=%q\n' "$ema_dir"
  printf 'export BIOMCP_WHO_DIR=%q\n' "$who_dir"
  printf 'export BIOMCP_CACHE_MODE=off\n'
  printf 'export BIOMCP_PROVIDER_CONTRACT_READY_FILE=%q\n' "$ready_file"
  printf 'export BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG=%q\n' "$request_log"
} >"$env_file"

bash "$ownership_helper" write "$workspace_root" "provider-contract" "$fixture_root" "$server_pid" "BIOMCP_PROVIDER_CONTRACT" "$owner_arg" >/dev/null
trap - EXIT INT TERM HUP
printf '%s\n' "$fixture_root"
