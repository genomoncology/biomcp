#!/usr/bin/env bash
set -euo pipefail

workspace_root="${1:-$PWD}"
cache_dir="$workspace_root/.cache"
env_file="$cache_dir/spec-vaers-env"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cleanup_script="$script_dir/cleanup-vaers-spec-fixture.sh"
fixture_src="$script_dir/cvx"
fixture_cvx_root="$cache_dir/spec-vaers-cvx"

mkdir -p "$cache_dir"

if [ -x "$cleanup_script" ]; then
  bash "$cleanup_script" "$workspace_root"
fi

rm -rf "$fixture_cvx_root"
cp -R "$fixture_src" "$fixture_cvx_root"
find "$fixture_cvx_root" -type f -exec touch {} +

fixture_root="$(mktemp -d "$cache_dir/spec-vaers.XXXXXX")"
ready_file="$fixture_root/base-url"
server_log="$fixture_root/server.log"

python3 - "$ready_file" "$script_dir/vaers" <<'PY' >"$server_log" 2>&1 &
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


def send_xml(handler, body: bytes):
    handler.send_response(200)
    handler.send_header("Content-Type", "text/html; charset=ISO-8859-1")
    handler.send_header("Content-Length", str(len(body)))
    handler.end_headers()
    handler.wfile.write(body)


def load_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def load_bytes(path: Path) -> bytes:
    return path.read_bytes()


fixture_dir = Path(sys.argv[2])
reactions_response = load_bytes(fixture_dir / "reactions-response.xml")
serious_response = load_bytes(fixture_dir / "serious-response.xml")
age_response = load_bytes(fixture_dir / "age-response.xml")
covid_reactions_response = load_bytes(fixture_dir / "covid-reactions-response.xml")

FAERS_RESULTS = [
    {
        "safetyreportid": "90000001",
        "receivedate": "20250401",
        "serious": "1",
        "patient": {
            "drug": [
                {
                    "medicinalproduct": "COVID-19 VACCINE",
                    "openfda": {
                        "generic_name": ["COVID-19 vaccine"],
                        "brand_name": ["COMIRNATY"],
                    },
                }
            ],
            "reaction": [
                {"reactionmeddrapt": "Pyrexia"},
                {"reactionmeddrapt": "Fatigue"},
            ],
        },
    },
    {
        "safetyreportid": "90000002",
        "receivedate": "20250402",
        "serious": "2",
        "patient": {
            "drug": [
                {
                    "medicinalproduct": "COVID-19 VACCINE",
                    "openfda": {
                        "generic_name": ["COVID-19 vaccine"],
                        "brand_name": ["COMIRNATY"],
                    },
                }
            ],
            "reaction": [
                {"reactionmeddrapt": "Headache"},
            ],
        },
    },
]


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        parsed = urlparse(self.path)
        if parsed.path == "/drug/event.json":
            query = parse_qs(parsed.query)
            limit = int(query.get("limit", ["5"])[0])
            skip = int(query.get("skip", ["0"])[0])
            results = FAERS_RESULTS[skip : skip + limit]
            send_json(
                self,
                200,
                {
                    "meta": {
                        "results": {
                            "skip": skip,
                            "limit": limit,
                            "total": len(FAERS_RESULTS),
                        }
                    },
                    "results": results,
                },
            )
            return

        send_json(self, 404, {"error": "not found"})

    def do_POST(self):
        parsed = urlparse(self.path)
        if parsed.path != "/controller/datarequest/D8":
            send_json(self, 404, {"error": "not found"})
            return

        length = int(self.headers.get("Content-Length", "0"))
        raw_body = self.rfile.read(length).decode("utf-8")
        form = parse_qs(raw_body, keep_blank_values=True)
        request_xml = form.get("request_xml", [""])[0]
        accepted = form.get("accept_datause_restrictions", ["false"])[0]

        if accepted != "true" or not request_xml:
            send_json(self, 400, {"error": "missing request_xml or restrictions flag"})
            return

        if "<name>B_1</name><value>D8.V13-level2</value>" in request_xml:
            if "<name>F_D8.V14</name><value>COVID19</value>" in request_xml:
                send_xml(self, covid_reactions_response)
            else:
                send_xml(self, reactions_response)
            return
        if "<name>B_1</name><value>D8.V10</value>" in request_xml:
            send_xml(self, serious_response)
            return
        if "<name>B_1</name><value>D8.V1</value>" in request_xml:
            send_xml(self, age_response)
            return

        send_json(self, 400, {"error": "unexpected request_xml"})

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

# Wait for the shared loopback VAERS/OpenFDA fixture base URL to answer before exporting it.
printf 'export BIOMCP_VAERS_BASE=%q\n' "$base_url" >"$env_file"
printf 'export BIOMCP_OPENFDA_BASE=%q\n' "$base_url" >>"$env_file"
printf 'export BIOMCP_CVX_DIR=%q\n' "$fixture_cvx_root" >>"$env_file"
printf 'unset OPENFDA_API_KEY\n' >>"$env_file"
printf 'export BIOMCP_VAERS_FIXTURE_PID=%q\n' "$server_pid" >>"$env_file"
printf 'export BIOMCP_VAERS_FIXTURE_ROOT=%q\n' "$fixture_root" >>"$env_file"
printf 'export BIOMCP_VAERS_FIXTURE_READY_FILE=%q\n' "$ready_file" >>"$env_file"

printf '%s\n' "$fixture_root"
