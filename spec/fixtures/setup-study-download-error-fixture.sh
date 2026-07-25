#!/usr/bin/env bash
set -euo pipefail

workspace_root="${1:-$PWD}"
cache_dir="$workspace_root/.cache"
env_file="$cache_dir/spec-study-download-error-env"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cleanup_script="$script_dir/cleanup-study-download-error-fixture.sh"

mkdir -p "$cache_dir"

if [ -x "$cleanup_script" ]; then
  bash "$cleanup_script" "$workspace_root"
fi

fixture_root="$(mktemp -d "$cache_dir/spec-study-download-error.XXXXXX")"
study_root="$fixture_root/download-root"
ready_file="$fixture_root/base-url"
server_log="$fixture_root/server.log"

mkdir -p "$study_root"

python3 - "$ready_file" <<'PY' >"$server_log" 2>&1 8>&- &
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
import sys


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == "/missing_study.tar.gz":
            body = (
                b'<?xml version="1.0" encoding="UTF-8"?>'
                b"<Error><Code>AccessDenied</Code><Message>Access Denied</Message></Error>"
            )
            self.send_response(403)
            self.send_header("Content-Type", "application/xml")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return

        body = b"not found"
        self.send_response(404)
        self.send_header("Content-Type", "text/plain; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

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

printf 'export BIOMCP_CBIOPORTAL_DATAHUB_BASE=%q\n' "$base_url" >"$env_file"
printf 'export BIOMCP_STUDY_DIR=%q\n' "$study_root" >>"$env_file"
printf 'export BIOMCP_STUDY_DOWNLOAD_ERROR_PID=%q\n' "$server_pid" >>"$env_file"
printf 'export BIOMCP_STUDY_DOWNLOAD_ERROR_ROOT=%q\n' "$fixture_root" >>"$env_file"
printf 'export BIOMCP_STUDY_DOWNLOAD_ERROR_READY_FILE=%q\n' "$ready_file" >>"$env_file"

printf '%s\n' "$fixture_root"
