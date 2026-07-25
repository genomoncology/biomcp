#!/usr/bin/env bash
set -euo pipefail

workspace_root="${1:-$PWD}"
cache_dir="$workspace_root/.cache"
env_file="$cache_dir/spec-drug-ae-fallback-env"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cleanup_script="$script_dir/cleanup-drug-ae-fallback-spec-fixture.sh"

mkdir -p "$cache_dir"

if [ -x "$cleanup_script" ]; then
  bash "$cleanup_script" "$workspace_root"
fi

fixture_root="$(mktemp -d "$cache_dir/spec-drug-ae-fallback.XXXXXX")"
ready_file="$fixture_root/base-url"
server_log="$fixture_root/server.log"

python3 - "$ready_file" <<'PY' >"$server_log" 2>&1 8>&- &
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlparse
import json
import sys


def send_json(handler, status, payload):
    body = json.dumps(payload).encode("utf-8")
    handler.send_response(status)
    handler.send_header("Content-Type", "application/json")
    handler.send_header("Content-Length", str(len(body)))
    handler.end_headers()
    handler.wfile.write(body)


DARAXONRASIB_STUDIES = {
    "studies": [
        {
            "protocolSection": {
                "identificationModule": {
                    "nctId": "NCT05379985",
                    "briefTitle": "Daraxonrasib First-in-Human Study",
                }
            },
            "hasResults": True,
            "resultsSection": {
                "adverseEventsModule": {
                    "seriousEvents": [
                        {
                            "term": "Rash",
                            "stats": [{"groupId": "g1", "numAffected": 2, "numAtRisk": 10}],
                        },
                        {
                            "term": "Fatigue",
                            "stats": [{"groupId": "g1", "numAffected": 1, "numAtRisk": 10}],
                        },
                    ],
                    "otherEvents": [
                        {
                            "term": "Rash",
                            "stats": [{"groupId": "g1", "numAffected": 4, "numAtRisk": 10}],
                        },
                        {
                            "term": "Nausea",
                            "stats": [{"groupId": "g1", "numAffected": 5, "numAtRisk": 10}],
                        },
                    ],
                }
            },
        },
        {
            "protocolSection": {
                "identificationModule": {
                    "nctId": "NCT00000002",
                    "briefTitle": "Daraxonrasib Expansion Cohort",
                }
            },
            "hasResults": True,
            "resultsSection": {
                "adverseEventsModule": {
                    "seriousEvents": [],
                    "otherEvents": [
                        {
                            "term": "Rash",
                            "stats": [{"groupId": "g2", "numAffected": 3, "numAtRisk": 12}],
                        },
                        {
                            "term": "Diarrhea",
                            "stats": [{"groupId": "g2", "numAffected": 2, "numAtRisk": 12}],
                        },
                    ],
                }
            },
        },
    ],
    "nextPageToken": None,
}

EMPTY_CT_STUDIES = {"studies": [], "nextPageToken": None}


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        parsed = urlparse(self.path)
        query = parse_qs(parsed.query)

        if parsed.path == "/drug/event.json":
            search = query.get("search", [""])[0].lower().replace("\\", "")
            if any(name in search for name in ("daraxonrasib", "rmc-6236", "ctgov-empty")):
                send_json(
                    self,
                    404,
                    {"error": {"code": "NOT_FOUND", "message": "No matches found!"}},
                )
                return
            if "faers-empty" in search:
                send_json(
                    self,
                    200,
                    {"meta": {"results": {"skip": 0, "limit": 5, "total": 0}}, "results": []},
                )
                return
            send_json(
                self,
                404,
                {"error": {"code": "NOT_FOUND", "message": "No matches found!"}},
            )
            return

        if parsed.path == "/api/v2/studies":
            intervention = query.get("query.intr", [""])[0].strip().lower()
            if intervention in {"daraxonrasib", "rmc-6236"}:
                send_json(self, 200, DARAXONRASIB_STUDIES)
                return
            if intervention == "ctgov-empty":
                send_json(self, 200, EMPTY_CT_STUDIES)
                return
            if intervention == "faers-empty":
                send_json(self, 200, DARAXONRASIB_STUDIES)
                return
            send_json(self, 200, EMPTY_CT_STUDIES)
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

printf 'export BIOMCP_OPENFDA_BASE=%q\n' "$base_url" >"$env_file"
printf 'export BIOMCP_CTGOV_BASE=%q\n' "$base_url/api/v2" >>"$env_file"
printf 'unset OPENFDA_API_KEY\n' >>"$env_file"
printf 'export BIOMCP_DRUG_AE_FALLBACK_PID=%q\n' "$server_pid" >>"$env_file"
printf 'export BIOMCP_DRUG_AE_FALLBACK_ROOT=%q\n' "$fixture_root" >>"$env_file"
printf 'export BIOMCP_DRUG_AE_FALLBACK_READY_FILE=%q\n' "$ready_file" >>"$env_file"

printf '%s\n' "$fixture_root"
