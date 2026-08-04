#!/usr/bin/env bash
# Runner-owned CPIC fixture: serves receipt-backed bytes and accepts only CPIC plans
# used by the routine PGx document.
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ownership_helper="$script_dir/routine-fixture-ownership.sh"
root="$(cd "${1:-$PWD}" && pwd)"
cache="$root/.cache"
env_file="$cache/spec-cpic-env"
fixture_data="$root/testdata/sources/cpic"
mkdir -p "$cache"
fixture_root="$(mktemp -d "$cache/spec-cpic.XXXXXX")"
owner_arg="$(bash "$ownership_helper" new-owner "cpic" "$fixture_root")"
bash "$script_dir/cleanup-cpic-spec-fixture.sh" "$root"
ready="$fixture_root/origin"
requests="$fixture_root/requests.log"
: >"$requests"

READY="$ready" REQUESTS="$requests" FIXTURE_DATA="$fixture_data" \
  setsid python3 - "$owner_arg" 8>&- <<'PY' >"$fixture_root/server.log" 2>&1 &
import os
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlsplit

ready = Path(os.environ["READY"])
requests = Path(os.environ["REQUESTS"])
data = Path(os.environ["FIXTURE_DATA"])


def expected(path, query):
    if path == "/pair_view" and query == {
        "genesymbol": ["eq.CYP2D6"], "select": ["*"], "limit": ["15"],
        "offset": ["0"], "order": ["cpiclevel.asc,drugname.asc"],
    }:
        return "pair_gene_cyp2d6_20260803.json", "0-14/*"
    if path == "/pair_view" and query == {
        "drugname": ["ilike.*clopidogrel*"], "select": ["*"], "limit": ["15"],
        "offset": ["0"], "order": ["cpiclevel.asc,genesymbol.asc"],
    }:
        return "pair_drug_clopidogrel_20260803.json", "0-1/*"
    if path == "/pair_view" and query == {
        "genesymbol": ["eq.CYP2D6"], "select": ["*"], "limit": ["100"],
        "offset": ["0"], "order": ["cpiclevel.asc,drugname.asc"],
    }:
        return "pair_gene_cyp2d6_20260803.json", "0-78/*"
    if path == "/recommendation_view" and query == {
        "lookupkey->>CYP2D6": ["not.is.null"], "select": ["*"], "limit": ["50"],
    }:
        return "recommendation_cyp2d6_20260803.json", None
    if path == "/population_frequency_view" and query == {
        "genesymbol": ["eq.CYP2D6"], "select": ["*"], "limit": ["30"],
    }:
        return "frequency_cyp2d6_20260803.json", None
    if path == "/guideline_summary_view" and query == {
        "genes": ['cs.[{"symbol":"CYP2D6"}]'], "select": ["*"], "limit": ["40"],
    }:
        return "guideline_cyp2d6_20260803.json", None
    return None


class Handler(BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def do_GET(self):
        parsed = urlsplit(self.path)
        query = parse_qs(parsed.query, keep_blank_values=True)
        with requests.open("a") as log:
            log.write(f"GET {parsed.path}?{parsed.query}\n")
        response = expected(parsed.path, query)
        if response is None:
            self.send_response(404)
            self.end_headers()
            return
        name, content_range = response
        body = (data / name).read_bytes()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        if content_range is not None:
            self.send_header("Content-Range", content_range)
        self.end_headers()
        self.wfile.write(body)


server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
ready.write_text(f"http://127.0.0.1:{server.server_port}")
server.serve_forever()
PY
pid=$!
cleanup_incomplete_setup() {
  kill -TERM -- "-$pid" 2>/dev/null || true
  wait "$pid" 2>/dev/null || true
  rm -rf "$fixture_root"
}
trap cleanup_incomplete_setup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
trap 'exit 129' HUP
for _ in $(seq 1 50); do
  test -s "$ready" && break
  kill -0 "$pid" 2>/dev/null || { cat "$fixture_root/server.log" >&2; exit 1; }
  sleep .1
done
test -s "$ready"
{
  printf 'export BIOMCP_CPIC_BASE=%q\n' "$(<"$ready")"
  printf 'export BIOMCP_CACHE_MODE=off\n'
  printf 'export BIOMCP_CACHE_DIR=%q\n' "$fixture_root/cache"
  printf 'export BIOMCP_CPIC_FIXTURE_REQUESTS=%q\n' "$requests"
} >"$env_file"
bash "$ownership_helper" write "$root" "cpic" "$fixture_root" "$pid" "BIOMCP_CPIC_FIXTURE" "$owner_arg" >/dev/null
trap - EXIT INT TERM HUP
