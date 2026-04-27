#!/usr/bin/env bash
set -euo pipefail

workspace_root="${1:-$PWD}"
cache_dir="$workspace_root/.cache"
env_file="$cache_dir/spec-complexportal-env"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cleanup_script="$script_dir/cleanup-complexportal-spec-fixture.sh"

mkdir -p "$cache_dir"

if [ -x "$cleanup_script" ]; then
  bash "$cleanup_script" "$workspace_root"
fi

fixture_root="$(mktemp -d "$cache_dir/spec-complexportal.XXXXXX")"
ready_file="$fixture_root/base-url"
server_log="$fixture_root/server.log"
request_log="$fixture_root/requests.log"
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
    "$cache_dir"/spec-complexportal.*)
      rm -rf "$fixture_root"
      ;;
  esac
}

trap cleanup_on_error EXIT

python3 - "$ready_file" "$request_log" <<'PY' >"$server_log" 2>&1 &
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlparse
import json
import sys


EXPECTED_FILTER = 'species_f:("Homo sapiens")'


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
        query = parse_qs(parsed.query, keep_blank_values=True)
        number = query.get("number", [""])[0]
        filters = query.get("filters", [""])[0]

        with Path(sys.argv[2]).open("a", encoding="utf-8") as log:
            log.write(f"GET {parsed.path} number={number} filters={filters}\n")

        if parsed.path != "/search/P15056" or number != "25" or filters != EXPECTED_FILTER:
            send_json(
                self,
                400,
                {
                    "error": "unexpected ComplexPortal fixture request",
                    "path": parsed.path,
                    "number": number,
                    "filters": filters,
                },
            )
            return

        send_json(
            self,
            200,
            {
                "elements": [
                    {
                        "complexAC": "CPX-12345",
                        "complexName": "Fixture BRAF signalosome",
                        "description": "Deterministic ComplexPortal fixture for BioMCP protein spec.",
                        "predictedComplex": False,
                        "interactors": [
                            {
                                "identifier": "P15056",
                                "name": "BRAF",
                                "stochiometry": "minValue: 1, maxValue: 1",
                                "interactorType": "protein",
                            },
                            {
                                "identifier": "Q02750",
                                "name": "MAP2K1",
                                "stochiometry": "minValue: 1, maxValue: 1",
                                "interactorType": "protein",
                            },
                            {
                                "identifier": "P36507",
                                "name": "MAP2K2",
                                "stochiometry": "minValue: 1, maxValue: 1",
                                "interactorType": "protein",
                            },
                            {
                                "identifier": "P27361",
                                "name": "MAPK3",
                                "stochiometry": "minValue: 1, maxValue: 1",
                                "interactorType": "protein",
                            },
                            {
                                "identifier": "P28482",
                                "name": "MAPK1",
                                "stochiometry": "minValue: 1, maxValue: 1",
                                "interactorType": "protein",
                            },
                            {
                                "identifier": "P04049",
                                "name": "RAF1",
                                "stochiometry": "minValue: 1, maxValue: 1",
                                "interactorType": "protein",
                            },
                            {
                                "identifier": "CHEBI:15422",
                                "name": "ATP",
                                "stochiometry": "minValue: 1, maxValue: 1",
                                "interactorType": "small molecule",
                            },
                        ],
                    },
                    {
                        "complexAC": "CPX-99999",
                        "complexName": "Fixture non-participant control",
                        "description": "Mentions P15056 only in prose and must be filtered out.",
                        "predictedComplex": True,
                        "interactors": [
                            {
                                "identifier": "Q9Y243",
                                "name": "AKT3",
                                "stochiometry": "minValue: 1, maxValue: 1",
                                "interactorType": "protein",
                            }
                        ],
                    },
                ]
            },
        )

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

# Wait for the loopback ComplexPortal fixture to answer before exporting it.
for _ in $(seq 1 50); do
  if curl -fsS "$base_url/search/P15056?number=25&filters=species_f:%28%22Homo%20sapiens%22%29" >/dev/null 2>&1; then
    break
  fi
  if ! kill -0 "$server_pid" 2>/dev/null; then
    cat "$server_log" >&2
    exit 1
  fi
  sleep 0.1
done
curl -fsS "$base_url/search/P15056?number=25&filters=species_f:%28%22Homo%20sapiens%22%29" >/dev/null 2>&1
: >"$request_log"

printf 'export BIOMCP_COMPLEXPORTAL_BASE=%q\n' "$base_url" >"$env_file"
printf 'export BIOMCP_COMPLEXPORTAL_FIXTURE_PID=%q\n' "$server_pid" >>"$env_file"
printf 'export BIOMCP_COMPLEXPORTAL_FIXTURE_ROOT=%q\n' "$fixture_root" >>"$env_file"
printf 'export BIOMCP_COMPLEXPORTAL_FIXTURE_READY_FILE=%q\n' "$ready_file" >>"$env_file"
printf 'export BIOMCP_COMPLEXPORTAL_FIXTURE_REQUEST_LOG=%q\n' "$request_log" >>"$env_file"

trap - EXIT
printf '%s\n' "$fixture_root"
