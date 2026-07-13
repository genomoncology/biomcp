#!/usr/bin/env bash
set -euo pipefail

workspace_root="${1:-$PWD}"
cache_dir="$workspace_root/.cache"
env_file="$cache_dir/spec-disgenet-credential-env"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cleanup_script="$script_dir/cleanup-disgenet-credential-spec-fixture.sh"

mkdir -p "$cache_dir"

if [[ -x "$cleanup_script" ]]; then
  bash "$cleanup_script" "$workspace_root"
fi

fixture_root="$(mktemp -d "$cache_dir/spec-disgenet-credential.XXXXXX")"
ready_file="$fixture_root/base-url"
server_log="$fixture_root/server.log"

uv run --no-sync python - "$workspace_root" "$ready_file" <<'PY' >"$server_log" 2>&1 &
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import urlparse
import json
import sys

ROOT = Path(sys.argv[1])
READY = Path(sys.argv[2])
BRAF = json.loads(
    (ROOT / "testdata/sources/mygene/get_braf.json").read_text(encoding="utf-8")
)


def send_json(handler, status, payload):
    body = json.dumps(payload).encode("utf-8")
    handler.send_response(status)
    handler.send_header("Content-Type", "application/json")
    handler.send_header("Content-Length", str(len(body)))
    handler.end_headers()
    handler.wfile.write(body)


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        path = urlparse(self.path).path
        if path == "/healthz":
            send_json(self, 200, {"status": "ok"})
            return
        if path == "/mygene/query":
            send_json(self, 200, BRAF)
            return
        if path == "/disgenet/api/v1/gda/summary":
            send_json(self, 403, {"message": "credential rejected"})
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
  if uv run --no-sync python - "$base_url/healthz" <<'PY' >/dev/null 2>&1
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

uv run --no-sync python - "$base_url/healthz" <<'PY' >/dev/null
from urllib.request import urlopen
import sys

with urlopen(sys.argv[1], timeout=1) as response:
    if response.status != 200:
        raise SystemExit(1)
PY

{
  printf 'export BIOMCP_MYGENE_BASE=%q\n' "$base_url/mygene"
  printf 'export BIOMCP_DISGENET_BASE=%q\n' "$base_url/disgenet"
  printf 'export BIOMCP_OPENTARGETS_BASE=%q\n' "$base_url/unused-opentargets"
  printf 'export DISGENET_API_KEY=%q\n' "fixture-disgenet-rejected-key-not-a-secret"
  printf 'export BIOMCP_CACHE_MODE=off\n'
  printf 'export BIOMCP_DISGENET_CREDENTIAL_PID=%q\n' "$server_pid"
  printf 'export BIOMCP_DISGENET_CREDENTIAL_ROOT=%q\n' "$fixture_root"
} >"$env_file"

printf '%s\n' "$fixture_root"
