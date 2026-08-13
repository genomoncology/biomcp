#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:?repo root required}"
CACHE_DIR="$ROOT/.cache"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OWNERSHIP_HELPER="$SCRIPT_DIR/routine-fixture-ownership.sh"
# shellcheck source=fixture-supervisor.sh
source "$SCRIPT_DIR/fixture-supervisor.sh"
mkdir -p "$CACHE_DIR"
ENV_FILE="$CACHE_DIR/spec-article-federated-timeout-env"
bash "$SCRIPT_DIR/cleanup-article-federated-timeout-fixture.sh" "$ROOT"
recover_fixture_orphans "$CACHE_DIR" "article-federated-timeout" "spec-article-federated-timeout."
FIXTURE_ROOT="$(mktemp -d "$CACHE_DIR/spec-article-federated-timeout.XXXXXX")"
OWNER_ARG="$(bash "$OWNERSHIP_HELPER" new-owner "article-federated-timeout" "$FIXTURE_ROOT")"
PORT_FILE="$FIXTURE_ROOT/port"
LOG_FILE="$FIXTURE_ROOT/server.log"
PID_FILE="$FIXTURE_ROOT/server-pid"

prepare_fixture_supervisor_owner
start_fixture_supervisor "article-federated-timeout" "$CACHE_DIR" "$FIXTURE_ROOT" "spec-article-federated-timeout." "$PID_FILE" \
  python3 - "$PORT_FILE" "$OWNER_ARG" >"$LOG_FILE" 2>&1 <<'PY' &
import json
import sys
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlparse

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
        query = parse_qs(parsed.query)
        path = parsed.path.rstrip("/") or "/"

        # Europe PMC search is the intentionally slow federated leg.
        if path == "/search" and "query" in query:
            time.sleep(65)
            self.send_json({
                "hitCount": 0,
                "resultList": {"result": []},
            })
            return

        # PubTator3 search returns one usable row, then an empty page.
        if path == "/search" and "text" in query:
            page = query.get("page", ["1"])[0]
            if page == "1":
                self.send_json({
                    "results": [{
                        "_id": "pt-418",
                        "pmid": 41800001,
                        "title": "BRAF melanoma bounded federation fixture",
                        "journal": "Fixture Journal",
                        "date": "2026-01-01",
                        "score": 42.0,
                    }],
                    "count": 1,
                    "total_pages": 1,
                    "current": 1,
                    "page_size": 25,
                    "facets": {},
                })
            else:
                self.send_json({
                    "results": [],
                    "count": 1,
                    "total_pages": 1,
                    "current": int(page),
                    "page_size": 25,
                    "facets": {},
                })
            return

        # PubMed ESearch/ESummary returns no rows quickly.
        if path == "/entrez/eutils/esearch.fcgi":
            self.send_json({"esearchresult": {"count": "0", "idlist": []}})
            return
        if path == "/entrez/eutils/esummary.fcgi":
            self.send_json({"result": {"uids": []}})
            return

        # Semantic Scholar search returns no rows quickly.
        if path == "/graph/v1/paper/search":
            self.send_json({"total": 0, "data": []})
            return

        # LitSense2 sentence search returns no rows quickly.
        if path == "/sentences":
            self.send_json([])
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

for _ in $(seq 1 100); do
  if [[ -s "$PORT_FILE" ]]; then
    break
  fi
  sleep 0.05
done
if [[ ! -s "$PORT_FILE" ]]; then
  echo "article federated timeout fixture failed to start" >&2
  cat "$LOG_FILE" >&2 || true
  exit 1
fi

port="$(cat "$PORT_FILE")"
base="http://127.0.0.1:$port"
cat >"$ENV_FILE" <<EOF
export BIOMCP_ARTICLE_FEDERATED_TIMEOUT_FIXTURE_PID="$pid"
export BIOMCP_ARTICLE_FEDERATED_TIMEOUT_FIXTURE_ROOT="$FIXTURE_ROOT"
export BIOMCP_PUBTATOR_BASE="$base"
export BIOMCP_EUROPEPMC_BASE="$base"
export BIOMCP_PUBMED_BASE="$base/entrez/eutils"
export BIOMCP_S2_BASE="$base"
export BIOMCP_TEST_UNPACED_ORIGIN="$base"
export BIOMCP_LITSENSE2_BASE="$base"
export S2_API_KEY=""
EOF
bash "$OWNERSHIP_HELPER" write "$ROOT" "article-federated-timeout" "$FIXTURE_ROOT" "$pid" "BIOMCP_ARTICLE_FEDERATED_TIMEOUT_FIXTURE" "$OWNER_ARG" >/dev/null
