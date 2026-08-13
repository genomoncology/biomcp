#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:?repo root required}"
CACHE_DIR="$ROOT/.cache"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OWNERSHIP_HELPER="$SCRIPT_DIR/routine-fixture-ownership.sh"
# shellcheck source=fixture-supervisor.sh
source "$SCRIPT_DIR/fixture-supervisor.sh"
mkdir -p "$CACHE_DIR"
KIND="run-article-semanticscholar-source"
PREFIX="spec-run-article-semanticscholar-source."
recover_fixture_orphans "$CACHE_DIR" "$KIND" "$PREFIX"
FIXTURE_ROOT="$(mktemp -d "$CACHE_DIR/$PREFIX"XXXXXX)"
OWNER_ARG="$(bash "$OWNERSHIP_HELPER" new-owner "$KIND" "$FIXTURE_ROOT")"
PORT_FILE="$FIXTURE_ROOT/port"
LOG_FILE="$FIXTURE_ROOT/server.log"
REQUEST_FILE="$FIXTURE_ROOT/requests"
PID_FILE="$FIXTURE_ROOT/server-pid"

cleanup() {
  bash "$OWNERSHIP_HELPER" cleanup "$ROOT" "$KIND" "BIOMCP_RUN_ARTICLE_SEMANTICSCHOLAR_SOURCE"
}
trap cleanup EXIT

prepare_fixture_supervisor_current_process
start_fixture_supervisor "$KIND" "$CACHE_DIR" "$FIXTURE_ROOT" "$PREFIX" "$PID_FILE" \
  python3 - "$PORT_FILE" "$REQUEST_FILE" "$OWNER_ARG" >"$LOG_FILE" 2>&1 <<'PY' &
import json
import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import urlparse

port_file = Path(sys.argv[1])
request_file = Path(sys.argv[2])

class Handler(BaseHTTPRequestHandler):
    def log_message(self, fmt, *args):
        return

    def send_json(self, payload):
        body = json.dumps(payload).encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        parsed = urlparse(self.path)
        request_file.write_text(request_file.read_text() + parsed.path + "\n" if request_file.exists() else parsed.path + "\n")
        if parsed.path == "/graph/v1/paper/search":
            self.send_json({
                "total": 1,
                "data": [{
                    "paperId": "fixture-semantic-scholar-paper",
                    "externalIds": {"PubMed": "41800002", "DOI": "10.5555/semantic-fixture"},
                    "title": "Semantic Scholar selectable source fixture",
                    "venue": "Fixture Journal",
                    "year": 2026,
                    "citationCount": 7,
                    "influentialCitationCount": 1,
                    "abstract": "BRAF melanoma Semantic Scholar source-only fixture abstract."
                }]
            })
            return
        self.send_response(404)
        self.end_headers()

server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
port_file.write_text(str(server.server_address[1]))
server.serve_forever()
PY
supervisor_pid=$!
for _ in $(seq 1 50); do test -s "$PID_FILE" && break; kill -0 "$supervisor_pid" 2>/dev/null || break; sleep .1; done
test -s "$PID_FILE"
pid="$(<"$PID_FILE")"
bash "$OWNERSHIP_HELPER" write "$ROOT" "$KIND" "$FIXTURE_ROOT" "$pid" "BIOMCP_RUN_ARTICLE_SEMANTICSCHOLAR_SOURCE" "$OWNER_ARG" >/dev/null

for _ in $(seq 1 100); do
  if [[ -s "$PORT_FILE" ]]; then
    break
  fi
  sleep 0.05
done
if [[ ! -s "$PORT_FILE" ]]; then
  echo "Semantic Scholar source fixture failed to start" >&2
  cat "$LOG_FILE" >&2 || true
  exit 1
fi

base="http://127.0.0.1:$(cat "$PORT_FILE")"
BIOMCP_CACHE_DIR="$ROOT/.cache/biomcp-article-semanticscholar-source" \
BIOMCP_S2_BASE="$base" \
BIOMCP_PUBTATOR_BASE="$base" \
BIOMCP_EUROPEPMC_BASE="$base" \
BIOMCP_PUBMED_BASE="$base" \
BIOMCP_LITSENSE2_BASE="$base" \
BIOMCP_TEST_UNPACED_ORIGIN="$base" \
S2_API_KEY="" \
  timeout 25s "$ROOT/tools/biomcp-ci" --json search article -k "BRAF melanoma" --source semanticscholar --debug-plan --limit 1
test "$(cat "$REQUEST_FILE")" = "/graph/v1/paper/search"
