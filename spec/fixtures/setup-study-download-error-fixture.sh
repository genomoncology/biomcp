#!/usr/bin/env bash
set -euo pipefail

workspace_root="${1:-$PWD}"
cache_dir="$workspace_root/.cache"
env_file="$cache_dir/spec-study-download-error-env"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ownership_helper="$script_dir/routine-fixture-ownership.sh"
# shellcheck source=fixture-supervisor.sh
source "$script_dir/fixture-supervisor.sh"
cleanup_script="$script_dir/cleanup-study-download-error-fixture.sh"

mkdir -p "$cache_dir"

if [ -x "$cleanup_script" ]; then
  bash "$cleanup_script" "$workspace_root"
fi
recover_fixture_orphans "$cache_dir" "study-download-error" "spec-study-download-error."

fixture_root="$(mktemp -d "$cache_dir/spec-study-download-error.XXXXXX")"
owner_arg="$(bash "$ownership_helper" new-owner "study-download-error" "$fixture_root")"
study_root="$fixture_root/download-root"
ready_file="$fixture_root/base-url"
server_pid_file="$fixture_root/server-pid"
server_log="$fixture_root/server.log"

mkdir -p "$study_root"

prepare_fixture_supervisor_owner
start_fixture_supervisor "study-download-error" "$cache_dir" "$fixture_root" "spec-study-download-error." "$server_pid_file" \
  python3 - "$ready_file" "$owner_arg" <<'PY' >"$server_log" 2>&1 &
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
supervisor_pid=$!
for _ in $(seq 1 50); do test -s "$server_pid_file" && break; kill -0 "$supervisor_pid" 2>/dev/null || break; sleep .1; done
test -s "$server_pid_file"
server_pid="$(<"$server_pid_file")"

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

bash "$ownership_helper" write "$workspace_root" "study-download-error" "$fixture_root" "$server_pid" "BIOMCP_STUDY_DOWNLOAD_ERROR" "$owner_arg" >/dev/null
printf '%s\n' "$fixture_root"
