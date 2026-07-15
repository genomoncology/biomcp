#!/usr/bin/env bash
set -euo pipefail

workspace_root="${1:-$PWD}"
cache_dir="$workspace_root/.cache"
env_file="$cache_dir/spec-myvariant-env"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cleanup_script="$script_dir/cleanup-myvariant-spec-fixture.sh"

mkdir -p "$cache_dir"
bash "$cleanup_script" "$workspace_root"

fixture_root="$(mktemp -d "$cache_dir/spec-myvariant.XXXXXX")"
ready_file="$fixture_root/base-url"
server_log="$fixture_root/server.log"
request_log="$fixture_root/request.log"
: >"$request_log"

uv run --no-sync python - "$workspace_root" "$ready_file" "$request_log" <<'PY' >"$server_log" 2>&1 &
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, unquote, urlparse
import json
import sys

ROOT = Path(sys.argv[1])
READY = Path(sys.argv[2])
REQUEST_LOG = Path(sys.argv[3])
FIXTURE_DIR = ROOT / "testdata/sources/myvariant"
BAYESDEL_HIT = json.loads((FIXTURE_DIR / "get_braf_bayesdel.json").read_text(encoding="utf-8"))
GERP_SEARCH = json.loads((FIXTURE_DIR / "search_gerp_min.json").read_text(encoding="utf-8"))
EMPTY_SEARCH = {"took": 1, "total": 0, "max_score": None, "hits": []}


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

        path = unquote(parsed.path)
        if path == "/healthz":
            send_json(self, 200, {"status": "ok"})
            return
        if path == "/variant/chr7:g.140453136A>T":
            send_json(self, 200, BAYESDEL_HIT)
            return
        if path == "/query":
            query = parse_qs(parsed.query).get("q", [""])[0]
            payload = GERP_SEARCH if "dbnsfp.gerp++.rs:[4 TO *]" in query else EMPTY_SEARCH
            send_json(self, 200, payload)
            return

        send_json(self, 404, {"error": "fixture path not found"})

    def log_message(self, format, *args):
        return


server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
READY.write_text(f"http://127.0.0.1:{server.server_port}\n", encoding="utf-8")
server.serve_forever()
PY
server_pid=$!

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
  if curl --fail --silent "$base_url/healthz" >/dev/null; then
    break
  fi
  sleep 0.1
done
curl --fail --silent "$base_url/healthz" >/dev/null

{
  printf 'export BIOMCP_MYVARIANT_BASE=%q\n' "$base_url"
  printf 'export BIOMCP_CACHE_MODE=off\n'
  printf 'export BIOMCP_MYVARIANT_FIXTURE_PID=%q\n' "$server_pid"
  printf 'export BIOMCP_MYVARIANT_FIXTURE_ROOT=%q\n' "$fixture_root"
  printf 'export BIOMCP_MYVARIANT_FIXTURE_REQUEST_LOG=%q\n' "$request_log"
} >"$env_file"

printf '%s\n' "$fixture_root"
