#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:?repo root required}"
ROOT="$(cd "$ROOT" && pwd)"
CACHE_DIR="$ROOT/.cache"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OWNERSHIP_HELPER="$SCRIPT_DIR/routine-fixture-ownership.sh"
# shellcheck source=fixture-supervisor.sh
source "$SCRIPT_DIR/fixture-supervisor.sh"
mkdir -p "$CACHE_DIR"
ENV_FILE="$CACHE_DIR/spec-europepmc-cursor-env"
bash "$SCRIPT_DIR/cleanup-europepmc-cursor-fixture.sh" "$ROOT"
recover_fixture_orphans "$CACHE_DIR" "europepmc-cursor" "spec-europepmc-cursor."
FIXTURE_ROOT="$(mktemp -d "$CACHE_DIR/spec-europepmc-cursor.XXXXXX")"
OWNER_ARG="$(bash "$OWNERSHIP_HELPER" new-owner "europepmc-cursor" "$FIXTURE_ROOT")"
PORT_FILE="$FIXTURE_ROOT/port"
LOG_FILE="$FIXTURE_ROOT/server.log"
PID_FILE="$FIXTURE_ROOT/server-pid"
REQUESTS_FILE="$FIXTURE_ROOT/requests.log"

prepare_fixture_supervisor_owner
start_fixture_supervisor "europepmc-cursor" "$CACHE_DIR" "$FIXTURE_ROOT" "spec-europepmc-cursor." "$PID_FILE" \
  python3 - "$PORT_FILE" "$OWNER_ARG" "$REQUESTS_FILE" >"$LOG_FILE" 2>&1 <<'PY' &
import json
import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlparse

port_file = Path(sys.argv[1])
requests_file = Path(sys.argv[3])

PAGE_SIZE = 25
HIT_COUNT = 60


def rows_for_page(page_number, count=PAGE_SIZE):
    start = (page_number - 1) * PAGE_SIZE + 1
    return [
        {
            "id": str(41800100 + start + i),
            "pmid": str(41800100 + start + i),
            "title": f"cursor page {page_number} row {i + 1}",
            "journalTitle": "Fixture Journal",
            "firstPublicationDate": "2026-01-01",
            "isOpenAccess": "N",
        }
        for i in range(count)
    ]


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

    def do_POST(self):
        parsed = urlparse(self.path)
        path = parsed.path.rstrip("/") or "/"
        if path == "/graph/v1/paper/batch":
            length = int(self.headers.get("Content-Length") or "0")
            if length:
                self.rfile.read(length)
            self.send_json([None])
            return
        self.send_response(404)
        self.end_headers()

    def do_GET(self):
        parsed = urlparse(self.path)
        query = parse_qs(parsed.query)
        path = parsed.path.rstrip("/") or "/"

        if path == "/search" and "query" in query:
            with requests_file.open("a") as fh:
                fh.write(parsed.query + "\n")
            cursor = query.get("cursorMark", [""])[0]
            # Two corpora: the full one holds 50 rows in two pages plus an
            # exhausted third; the short one holds 40 so a --limit 50 search
            # walks into the exhausted page and stops there.
            phrase = query.get("query", [""])[0]
            short = "short" in phrase
            # The lying corpus reports a hitCount above its real rows, so the
            # walk requests a third page that exists but is exhausted.
            lying = "lying" in phrase
            reported = HIT_COUNT if (lying or not short) else 40
            if cursor == "*":
                payload = {
                    "hitCount": reported,
                    "nextCursorMark": "CUR2",
                    "resultList": {"result": rows_for_page(1)},
                }
            elif cursor == "CUR2":
                payload = {
                    "hitCount": reported,
                    "nextCursorMark": "CUR3",
                    "resultList": {"result": rows_for_page(2, 15 if (short or lying) else PAGE_SIZE)},
                }
            elif cursor == "CUR3":
                # Exhausted: Europe PMC omits nextCursorMark and serves no rows.
                payload = {"hitCount": reported, "resultList": {"result": []}}
            else:
                payload = {"hitCount": reported, "resultList": {"result": []}}
            self.send_json(payload)
            return

        # Keep every other article provider quiet and fast.
        if path == "/search" and "text" in query:
            self.send_json({"results": [], "count": 0, "facets": {}})
            return
        if path == "/entrez/eutils/esearch.fcgi":
            self.send_json({"esearchresult": {"count": "0", "idlist": []}})
            return
        if path == "/entrez/eutils/esummary.fcgi":
            self.send_json({"result": {"uids": []}})
            return
        if path == "/graph/v1/paper/search":
            self.send_json({"total": 0, "data": []})
            return
        if path == "/publications/export/biocjson":
            self.send_json({"documents": []})
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
  echo "europepmc cursor fixture failed to start" >&2
  cat "$LOG_FILE" >&2 || true
  exit 1
fi

port="$(cat "$PORT_FILE")"
base="http://127.0.0.1:$port"
cat >"$ENV_FILE" <<EOF
export BIOMCP_EUROPEPMC_CURSOR_FIXTURE_PID="$pid"
export BIOMCP_EUROPEPMC_CURSOR_FIXTURE_ROOT="$FIXTURE_ROOT"
export BIOMCP_EUROPEPMC_CURSOR_REQUESTS="$REQUESTS_FILE"
export BIOMCP_CACHE_DIR="$FIXTURE_ROOT/cache"
export BIOMCP_PUBTATOR_BASE="$base"
export BIOMCP_EUROPEPMC_BASE="$base"
export BIOMCP_PUBMED_BASE="$base/entrez/eutils"
export BIOMCP_S2_BASE="$base"
export BIOMCP_TEST_UNPACED_ORIGIN="$base"
export BIOMCP_LITSENSE2_BASE="$base"
export S2_API_KEY=""
EOF
bash "$OWNERSHIP_HELPER" write "$ROOT" "europepmc-cursor" "$FIXTURE_ROOT" "$pid" "BIOMCP_EUROPEPMC_CURSOR_FIXTURE" "$OWNER_ARG" >/dev/null
