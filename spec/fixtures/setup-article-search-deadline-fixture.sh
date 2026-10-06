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
ENV_FILE="$CACHE_DIR/spec-article-search-deadline-env"
bash "$SCRIPT_DIR/cleanup-article-search-deadline-fixture.sh" "$ROOT"
recover_fixture_orphans "$CACHE_DIR" "article-search-deadline" "spec-article-search-deadline."
FIXTURE_ROOT="$(mktemp -d "$CACHE_DIR/spec-article-search-deadline.XXXXXX")"
OWNER_ARG="$(bash "$OWNERSHIP_HELPER" new-owner "article-search-deadline" "$FIXTURE_ROOT")"
PORT_FILE="$FIXTURE_ROOT/port"
LOG_FILE="$FIXTURE_ROOT/server.log"
PID_FILE="$FIXTURE_ROOT/server-pid"

prepare_fixture_supervisor_owner
start_fixture_supervisor "article-search-deadline" "$CACHE_DIR" "$FIXTURE_ROOT" "spec-article-search-deadline." "$PID_FILE" \
  python3 - "$PORT_FILE" "$OWNER_ARG" >"$LOG_FILE" 2>&1 <<'PY' &
import json
import sys
import threading
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

        # Europe PMC search outlasts the whole-search deadline: the reply is
        # gated on an event nothing ever sets, so the hold carries no clock
        # assumption and dies with the fixture process at cleanup. The
        # "pagebound" marker answers page one fast and holds every later
        # page, so a single-backend plan expires mid-pagination with rows
        # already fetched (ticket 1299). The "silence" marker makes every
        # other source fail fast so nothing answers and the terminal
        # deadline error is the only output.
        if path == "/search" and "query" in query:
            search = query.get("query", [""])[0]
            if "pagebound" in search:
                page = int(query.get("page", ["1"])[0])
                if page < 2:
                    self.send_json({
                        "version": "6.9",
                        "hitCount": 200,
                        "request": {"query": search},
                        "resultList": {"result": [
                            {
                                "id": f"41800{page:02d}{i:02d}",
                                "pmid": str(41800000 + page * 100 + i),
                                "title": f"deadline single-source page one row {i}",
                                "journalTitle": "Fixture Journal",
                                "firstPublicationDate": "2026-01-01",
                                "citedByCount": 0,
                                "isOpenAccess": "N",
                            }
                            for i in range(25)
                        ]},
                    })
                    return
            threading.Event().wait()
            return

        wants_silence = any("silence" in value[0] for value in query.values() if value)
        if wants_silence:
            self.send_response(503)
            self.send_header("Content-Length", "0")
            self.end_headers()
            return

        # PubTator3 search returns one usable row, then an empty page.
        if path == "/search" and "text" in query:
            page = query.get("page", ["1"])[0]
            if page == "1":
                self.send_json({
                    "results": [{
                        "_id": "pt-418",
                        "pmid": 41800011,
                        "title": "deadline-bound federation PubTator row",
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

        # PubMed ESearch/ESummary returns one row quickly.
        if path == "/entrez/eutils/esearch.fcgi":
            self.send_json({"esearchresult": {"count": "1", "idlist": ["41800012"]}})
            return
        if path == "/entrez/eutils/esummary.fcgi":
            self.send_json({"result": {"uids": ["41800012"], "41800012": {
                "uid": "41800012",
                "title": "deadline-bound federation PubMed row",
                "sortpubdate": "2026/01/02 00:00",
                "pubdate": "2026 Jan 2",
                "fulljournalname": "Fixture Journal",
                "source": "Fixture Journal",
            }}})
            return

        # Semantic Scholar search returns one row quickly; batch enrichment
        # answers nulls so a healthy page finishes before the deadline too.
        if path == "/graph/v1/paper/search":
            self.send_json({
                "total": 1,
                "data": [{
                    "paperId": "fixture-deadline-s2-paper",
                    "externalIds": {"PubMed": "41800013"},
                    "title": "deadline-bound federation Semantic Scholar row",
                    "venue": "Fixture Journal",
                    "year": 2026,
                    "citationCount": 7,
                    "influentialCitationCount": 1,
                    "abstract": "deadline fixture abstract."
                }]
            })
            return

        # PubTator export answers no document: the metadata fallback is cheap.
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
  echo "article search deadline fixture failed to start" >&2
  cat "$LOG_FILE" >&2 || true
  exit 1
fi

port="$(cat "$PORT_FILE")"
base="http://127.0.0.1:$port"
cat >"$ENV_FILE" <<EOF
export BIOMCP_ARTICLE_SEARCH_DEADLINE_FIXTURE_PID="$pid"
export BIOMCP_ARTICLE_SEARCH_DEADLINE_FIXTURE_ROOT="$FIXTURE_ROOT"
export BIOMCP_CACHE_DIR="$FIXTURE_ROOT/cache"
export BIOMCP_PUBTATOR_BASE="$base"
export BIOMCP_EUROPEPMC_BASE="$base"
export BIOMCP_PUBMED_BASE="$base/entrez/eutils"
export BIOMCP_S2_BASE="$base"
export BIOMCP_TEST_UNPACED_ORIGIN="$base"
export BIOMCP_LITSENSE2_BASE="$base"
export BIOMCP_TEST_ARTICLE_SEARCH_DEADLINE_MS="8000"
export S2_API_KEY=""
EOF
bash "$OWNERSHIP_HELPER" write "$ROOT" "article-search-deadline" "$FIXTURE_ROOT" "$pid" "BIOMCP_ARTICLE_SEARCH_DEADLINE_FIXTURE" "$OWNER_ARG" >/dev/null
