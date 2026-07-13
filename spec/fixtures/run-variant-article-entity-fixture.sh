#!/usr/bin/env bash
set -euo pipefail

repo_root="${1:-../..}"
scenario="${2:-all}"
repo_root="$(cd "$repo_root" && pwd)"
fixture_root="${repo_root}/.cache/spec-variant-article-entity-${scenario}"
ready_file="${fixture_root}/ready"
server_py="${fixture_root}/server.py"
request_log="${fixture_root}/requests.log"
rm -rf "$fixture_root"
mkdir -p "$fixture_root"

cat >"$server_py" <<'PY'
import json
import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlparse

ready = Path(sys.argv[1])
request_log = Path(sys.argv[2])

BRAF_PMID = "4260001"
MYD88_PMID = "24534189"


def send_json(handler, status, payload):
    body = json.dumps(payload).encode("utf-8")
    handler.send_response(status)
    handler.send_header("Content-Type", "application/json")
    handler.send_header("Content-Length", str(len(body)))
    handler.end_headers()
    handler.wfile.write(body)


def pubtator_result(pmid, title):
    return {
        "_id": pmid,
        "pmid": pmid,
        "title": title,
        "journal": "BioMCP fixture journal",
        "date": "2024-01-01",
        "score": 42.0,
    }


class Handler(BaseHTTPRequestHandler):
    def log_message(self, *args):
        return

    def do_GET(self):
        parsed = urlparse(self.path)
        params = parse_qs(parsed.query)
        with request_log.open("a", encoding="utf-8") as handle:
            handle.write(f"{parsed.path}?{parsed.query}\n")

        if parsed.path == "/entity/autocomplete/":
            query = params.get("query", [""])[0]
            rows = []
            if query == "BRAF":
                rows.append({"_id": "@GENE_BRAF", "biotype": "Gene", "name": "BRAF"})
            if query in {"BRAF V600E", "V600E", "p.V600E"}:
                rows.append({
                    "_id": "@VARIANT_p.V600V_BRAF_human",
                    "biotype": "Variant",
                    "name": "BRAF p.V600V",
                })
                rows.append({
                    "_id": "@VARIANT_p.V600E_BRAF_human",
                    "biotype": "Variant",
                    "name": "BRAF p.V600E",
                })
            if query == "MYD88":
                rows.append({"_id": "@GENE_MYD88", "biotype": "Gene", "name": "MYD88"})
            if query in {"MYD88 S219C", "S219C", "p.S219C"}:
                rows.append({
                    "_id": "@VARIANT_p.S219C_MYD88_human",
                    "biotype": "Variant",
                    "name": "MYD88 p.S219C",
                })
            send_json(self, 200, rows)
            return

        if parsed.path == "/search/":
            text = params.get("text", [""])[0]
            if text == "@VARIANT_p.V600E_BRAF_human":
                send_json(self, 200, {
                    "results": [pubtator_result(BRAF_PMID, "BRAF V600E entity-annotated fixture article")],
                    "count": 1,
                    "total_pages": 1,
                    "current": 1,
                    "page_size": 25,
                })
                return
            if text == "@VARIANT_p.S219C_MYD88_human":
                send_json(self, 200, {"results": [], "count": 0, "total_pages": 0, "current": 1, "page_size": 25})
                return
            if text == "MYD88 S219C":
                send_json(self, 200, {
                    "results": [pubtator_result(MYD88_PMID, "MYD88 S219C body-only free-text fixture article")],
                    "count": 1,
                    "total_pages": 1,
                    "current": 1,
                    "page_size": 25,
                })
                return
            send_json(self, 200, {"results": [], "count": 0, "total_pages": 0, "current": 1, "page_size": 25})
            return

        if parsed.path.endswith("/esearch.fcgi"):
            send_json(self, 200, {"esearchresult": {"idlist": [], "count": "0"}})
            return

        if parsed.path.endswith("/esummary.fcgi"):
            send_json(self, 200, {"result": {"uids": []}})
            return

        if parsed.path == "/semantic-scholar/graph/v1/paper/search" or parsed.path == "/graph/v1/paper/search":
            send_json(self, 200, {"total": 0, "data": []})
            return

        if parsed.path == "/sentences/" or parsed.path == "/passages/":
            send_json(self, 200, [])
            return

        if parsed.path == "/api/search" or parsed.path == "/search":
            send_json(self, 200, {"results": [], "hitCount": 0})
            return

        send_json(self, 200, {"resultList": {"result": []}, "hitCount": 0})


server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
ready.write_text(f"http://127.0.0.1:{server.server_port}", encoding="utf-8")
server.serve_forever()
PY

uv run --no-sync python3 "$server_py" "$ready_file" "$request_log" &
server_pid=$!
trap 'kill "$server_pid" 2>/dev/null || true' EXIT

for _ in $(seq 1 100); do
  if [ -s "$ready_file" ]; then
    break
  fi
  sleep 0.05
done

base_url="$(cat "$ready_file")"
binary="${BIOMCP_BIN:-$repo_root/target/spec/biomcp}"

export BIOMCP_CACHE_MODE=off
export BIOMCP_CACHE_DIR="$fixture_root/cache"
export BIOMCP_TEST_UNPACED_ORIGIN="$base_url"
export BIOMCP_PUBTATOR_BASE="$base_url"
export BIOMCP_EUROPEPMC_BASE="$base_url"
export BIOMCP_PUBMED_BASE="$base_url/entrez/eutils"
export BIOMCP_S2_BASE="$base_url"
export BIOMCP_LITSENSE2_BASE="$base_url"

case "$scenario" in
  all|braf)
    printf '## BRAF V600E limit 1\n'
    "$binary" variant articles "BRAF V600E" --limit 1
    printf '\n## BRAF V600E limit 3\n'
    "$binary" variant articles "BRAF V600E" --limit 3
    ;;
esac

case "$scenario" in
  all|myd88)
    printf '## MYD88 S219C fallback\n'
    "$binary" variant articles "MYD88 S219C" --limit 3
    ;;
esac

case "$scenario" in
  all|myd88-json)
    json_out="$("$binary" --json variant articles "MYD88 S219C" --limit 3)"
    jq -e '.retrieval_path | test("fallback")' >/dev/null <<<"$json_out"
    jq -e '.results | any(.pmid == "24534189")' >/dev/null <<<"$json_out"
    printf 'JSON fallback path preserved\n'
    jq -r '.results[] | select(.pmid == "24534189") | .pmid' <<<"$json_out"
    ;;
esac
