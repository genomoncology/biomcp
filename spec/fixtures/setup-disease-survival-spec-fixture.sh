#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ownership_helper="$script_dir/routine-fixture-ownership.sh"

workspace_root="${1:-$PWD}"
cache_dir="$workspace_root/.cache"
env_file="$cache_dir/spec-disease-survival-env"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cleanup_script="$script_dir/cleanup-disease-survival-spec-fixture.sh"

mkdir -p "$cache_dir"

if [[ -x "$cleanup_script" ]]; then
  bash "$cleanup_script" "$workspace_root"
fi

fixture_root="$(mktemp -d "$cache_dir/spec-disease-survival.XXXXXX")"
owner_arg="$(bash "$ownership_helper" new-owner "disease-survival" "$fixture_root")"
ready_file="$fixture_root/base-url"
server_log="$fixture_root/server.log"
request_log="$fixture_root/request.log"
: >"$request_log"

setsid python3 - "$workspace_root" "$ready_file" "$request_log" "$owner_arg" 8>&- <<'PY' >"$server_log" 2>&1 &
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import urlparse
import json
import sys

ROOT = Path(sys.argv[1])
READY = Path(sys.argv[2])
REQUEST_LOG = Path(sys.argv[3])

SITE_CATALOG = json.loads((ROOT / "testdata/sources/seer/site_catalog.json").read_text(encoding="utf-8"))
SURVIVAL_PAYLOAD_TEXT = (ROOT / "testdata/sources/seer/survival_payload_97.json").read_text(encoding="utf-8")

CML_HIT = {
    "_id": "MONDO:0011996",
    "mondo": {
        "name": "chronic myeloid leukemia",
        "definition": "A myeloid leukemia fixture for deterministic disease survival specs.",
        "synonym": ["CML", "chronic myelogenous leukemia"],
        "parents": [],
        "xrefs": {"ncit": "C3174"},
    },
    "disease_ontology": {
        "xrefs": {"ncit": "C3174"},
    },
    "hpo": {
        "phenotype_related_to_disease": [
            {"hpo_id": "HP:0001878", "evidence": "IEA", "hp_freq": "Occasional"}
        ]
    },
}


def send_json(handler, status, payload):
    body = json.dumps(payload).encode("utf-8")
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
        if parsed.path == "/mydisease/query":
            send_json(self, 200, {"total": 1, "hits": [CML_HIT]})
            return
        if parsed.path == "/mydisease/disease/MONDO:0011996":
            send_json(self, 200, CML_HIT)
            return
        if parsed.path == "/seer/get_var_formats.php":
            send_json(self, 200, SITE_CATALOG)
            return
        if parsed.path == "/seer/render_region_5.php":
            send_json(self, 200, SURVIVAL_PAYLOAD_TEXT)
            return

        send_json(self, 404, {"error": "fixture path not found"})

    def log_message(self, format, *args):
        return


server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
READY.write_text(f"http://127.0.0.1:{server.server_port}\n", encoding="utf-8")
server.serve_forever()
PY
server_pid=$!
cleanup_incomplete_setup() {
  kill -TERM -- "-$server_pid" 2>/dev/null || true
  wait "$server_pid" 2>/dev/null || true
  rm -rf "$fixture_root"
}
trap cleanup_incomplete_setup EXIT
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
  printf 'export BIOMCP_MYDISEASE_BASE=%q\n' "$base_url/mydisease"
  printf 'export BIOMCP_SEER_BASE=%q\n' "$base_url/seer"
  printf 'export BIOMCP_DGIDB_BASE=%q\n' "$base_url/unused-dgidb"
  printf 'export BIOMCP_OPENTARGETS_BASE=%q\n' "$base_url/unused-opentargets"
  printf 'export BIOMCP_CACHE_MODE=off\n'
  printf 'export BIOMCP_DISEASE_SURVIVAL_READY_FILE=%q\n' "$ready_file"
  printf 'export BIOMCP_DISEASE_SURVIVAL_REQUEST_LOG=%q\n' "$request_log"
} >"$env_file"

bash "$ownership_helper" write "$workspace_root" "disease-survival" "$fixture_root" "$server_pid" "BIOMCP_DISEASE_SURVIVAL" "$owner_arg" >/dev/null
trap - EXIT INT TERM HUP
printf '%s\n' "$fixture_root"
