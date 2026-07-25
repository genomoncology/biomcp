#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:?repo root required}"
CACHE_DIR="$ROOT/.cache"
mkdir -p "$CACHE_DIR"
PORT_FILE="$CACHE_DIR/spec-article-semanticscholar-source-port"
LOG_FILE="$CACHE_DIR/spec-article-semanticscholar-source.log"
PID_FILE="$CACHE_DIR/spec-article-semanticscholar-source.pid"

cleanup() {
  if [[ -f "$PID_FILE" ]]; then
    old_pid="$(cat "$PID_FILE" 2>/dev/null || true)"
    if [[ -n "$old_pid" ]]; then
      kill "$old_pid" 2>/dev/null || true
    fi
  fi
}
trap cleanup EXIT
cleanup
rm -f "$PORT_FILE" "$LOG_FILE" "$PID_FILE"

uv run --no-sync python3 - "$PORT_FILE" >"$LOG_FILE" 2>&1 <<'PY' 8>&- &
import json
import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import urlparse

port_file = Path(sys.argv[1])

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
pid=$!
echo "$pid" >"$PID_FILE"

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
BIOMCP_TEST_UNPACED_ORIGIN="$base" \
S2_API_KEY="" \
  timeout 25s "$ROOT/tools/biomcp-ci" --json search article -k "BRAF melanoma" --source semanticscholar --debug-plan --limit 1
