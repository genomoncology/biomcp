#!/usr/bin/env bash
set -euo pipefail

workspace_root="${1:-$PWD}"
cache_dir="$workspace_root/.cache"
env_file="$cache_dir/spec-section-outcomes-env"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cleanup_script="$script_dir/cleanup-section-outcomes-spec-fixture.sh"

mkdir -p "$cache_dir"
bash "$cleanup_script" "$workspace_root"

fixture_root="$(mktemp -d "$cache_dir/spec-section-outcomes.XXXXXX")"
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
    "$cache_dir"/spec-section-outcomes.*) rm -rf "$fixture_root" ;;
  esac
}
trap cleanup_on_error EXIT

python3 - "$ready_file" 8>&- <<'PY' >"$server_log" 2>&1 &
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, unquote, urlparse
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
        path = unquote(urlparse(self.path).path)
        if path == "/v1/query":
            query = parse_qs(urlparse(self.path).query).get("q", [""])[0]
            if "dbsnp.rsid:rs589000" in query:
                send_json(
                    self,
                    200,
                    {
                        "total": 1,
                        "hits": [
                            {
                                "_id": "rs589000",
                                "_score": 10.0,
                                "dbsnp": {"rsid": "rs589000"},
                                "dbnsfp": {
                                    "genename": "BRAF",
                                    "hgvsp": "p.V600E",
                                },
                            }
                        ],
                    },
                )
                return
            if "dbsnp.rsid:rs589001" in query:
                send_json(
                    self,
                    200,
                    {
                        "total": 1,
                        "hits": [
                            {
                                "_id": "rs589001",
                                "_score": 10.0,
                                "dbsnp": {"rsid": "rs589001"},
                            }
                        ],
                    },
                )
                return
            send_json(
                self,
                200,
                {
                    "total": 1,
                    "hits": [
                        {
                            "_id": "fixture-drug",
                            "_score": 10.0,
                            "drugbank": {
                                "id": "DBFIXTURE",
                                "name": "fixture-drug",
                                "synonyms": [],
                                "drug_interactions": [],
                            },
                        }
                    ],
                },
            )
            return
        if path == "/v1/variant/chr7:g.140453136A>T":
            send_json(
                self,
                200,
                {
                    "_id": "chr7:g.140453136A>T",
                    "_score": 10.0,
                    "dbnsfp": {"genename": "BRAF", "hgvsp": "p.V600E"},
                },
            )
            return
        if path == "/drug/drugsfda.json":
            send_json(
                self,
                200,
                {
                    "meta": {"results": {"skip": 0, "limit": 8, "total": 0}},
                    "results": [],
                },
            )
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
curl -fsS "$base_url/v1/query?q=fixture-drug" >/dev/null

printf 'export BIOMCP_MYCHEM_BASE=%q\n' "$base_url/v1" >"$env_file"
printf 'export BIOMCP_MYVARIANT_BASE=%q\n' "$base_url/v1" >>"$env_file"
printf 'export BIOMCP_OPENFDA_BASE=%q\n' "$base_url" >>"$env_file"
printf 'export BIOMCP_SECTION_OUTCOMES_FIXTURE_PID=%q\n' "$server_pid" >>"$env_file"
printf 'export BIOMCP_SECTION_OUTCOMES_FIXTURE_ROOT=%q\n' "$fixture_root" >>"$env_file"
printf 'export BIOMCP_SECTION_OUTCOMES_FIXTURE_READY_FILE=%q\n' "$ready_file" >>"$env_file"

trap - EXIT
printf '%s\n' "$fixture_root"
