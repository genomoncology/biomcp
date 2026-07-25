#!/usr/bin/env bash
set -euo pipefail

workspace_root="${1:-$PWD}"
cache_dir="$workspace_root/.cache"
env_file="$cache_dir/spec-mychem-empty-env"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cleanup_script="$script_dir/cleanup-mychem-empty-spec-fixture.sh"

mkdir -p "$cache_dir"

if [ -x "$cleanup_script" ]; then
  bash "$cleanup_script" "$workspace_root"
fi

fixture_root="$(mktemp -d "$cache_dir/spec-mychem-empty.XXXXXX")"
ready_file="$fixture_root/base-url"
server_log="$fixture_root/server.log"
server_pid=""

cleanup_on_error() {
  local status=$?
  if [ "$status" -eq 0 ]; then
    return
  fi

  if [ -n "${server_pid:-}" ] && kill -0 "$server_pid" 2>/dev/null; then
    kill "$server_pid" 2>/dev/null || true
    wait "$server_pid" 2>/dev/null || true
  fi

  case "${fixture_root:-}" in
    "$cache_dir"/spec-mychem-empty.*)
      rm -rf "$fixture_root"
      ;;
  esac
}

trap cleanup_on_error EXIT

python3 - "$ready_file" <<'PY' >"$server_log" 2>&1 8>&- &
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import urlparse
import json
import sys


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
        if parsed.path == "/v1/query":
            send_json(self, 200, {"total": 0, "hits": []})
            return
        send_json(self, 404, {"error": "not found"})

    def log_message(self, format, *args):
        return


ready_path = Path(sys.argv[1])
server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
ready_path.write_text(f"http://127.0.0.1:{server.server_port}\n", encoding="utf-8")
server.serve_forever()
PY
server_pid=$!

for _ in $(seq 1 50); do
  if [ -s "$ready_file" ]; then
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

printf 'export BIOMCP_MYCHEM_BASE=%q\n' "$base_url/v1" >"$env_file"
printf 'export BIOMCP_MYCHEM_EMPTY_PID=%q\n' "$server_pid" >>"$env_file"
printf 'export BIOMCP_MYCHEM_EMPTY_ROOT=%q\n' "$fixture_root" >>"$env_file"
printf 'export BIOMCP_MYCHEM_EMPTY_READY_FILE=%q\n' "$ready_file" >>"$env_file"

trap - EXIT
printf '%s\n' "$fixture_root"
