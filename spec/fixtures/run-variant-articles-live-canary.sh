#!/usr/bin/env bash
set -euo pipefail

repo_root="${1:-../..}"
repo_root="$(cd "$repo_root" && pwd)"
binary="${BIOMCP_BIN:-$repo_root/target/release/biomcp}"
fixture_root="$(mktemp -d "${TMPDIR:-/tmp}/biomcp-variant-article-corpus.XXXXXX")"
trap 'rm -rf "$fixture_root"' EXIT

uv run --no-sync python - "$repo_root" "$binary" "$fixture_root" <<'PY'
import hashlib
import json
import os
import subprocess
import sys
import threading
import urllib.error
import urllib.parse
import urllib.request
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

repo = Path(sys.argv[1])
binary = sys.argv[2]
work = Path(sys.argv[3])
source_root = repo / "testdata/sources"
mapping = json.loads((source_root / "variant_articles_683/panel-landmark-map.json").read_text())
receipts = json.loads((source_root / "capture-receipts.json").read_text())
receipt_by_path = {
    row["path"]: row["receipt"]
    for row in receipts["entries"]
    if row.get("classification") == "real_and_receipted" and row.get("receipt")
}

decoded = {}
route_bodies = {}
for row in mapping["landmarks"]:
    path = row["capture_path"]
    body = (source_root / path).read_bytes()
    receipt = receipt_by_path[path]
    assert hashlib.sha256(body).hexdigest() == receipt["sha256"]
    assert receipt["request"] == row["safe_request"]
    assert hashlib.sha256(row["safe_request"].encode()).hexdigest() == row["request_sha256"]
    payload = json.loads(body)
    if row["provider"] == "pubmed":
        pmids = set(payload.get("esearchresult", {}).get("idlist", []))
    else:
        pmids = {str(value["pmid"]) for value in payload["resultList"]["result"] if value.get("pmid")}
        parsed = urllib.parse.urlsplit(row["safe_request"])
        route_bodies["/search?" + parsed.query] = body
    decoded[(row["variant"], row["landmark_pmid"])] = row["landmark_pmid"] in pmids
    assert decoded[(row["variant"], row["landmark_pmid"])] is row["present"]

requests = []
unknown = []
class Handler(BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass
    def do_GET(self):
        requests.append(self.path)
        body = route_bodies.get(self.path)
        if body is None:
            unknown.append(self.path)
            body = b'{"error":"unknown captured route"}'
            self.send_response(404)
        else:
            self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
thread = threading.Thread(target=server.serve_forever, daemon=True)
thread.start()
base = f"http://127.0.0.1:{server.server_port}"
env = os.environ | {
    "BIOMCP_EUROPEPMC_BASE": base,
    "BIOMCP_TEST_UNPACED_ORIGIN": base,
    "BIOMCP_CACHE_MODE": "off",
    "BIOMCP_CACHE_DIR": str(work / "cache"),
}

cli_checks = []
seen_requests = set()
for safe_request in sorted({
    row["safe_request"] for row in mapping["landmarks"] if row["provider"] == "europepmc"
}):
    parsed = urllib.parse.urlsplit(safe_request)
    provider_query = urllib.parse.parse_qs(parsed.query)["query"][0]
    keyword = provider_query.removesuffix(' AND NOT PUB_TYPE:"retracted publication"')
    expected = {
        row["landmark_pmid"]
        for row in mapping["landmarks"]
        if row["safe_request"] == safe_request and row["present"]
    }
    json_run = subprocess.run(
        [binary, "--no-cache", "--json", "search", "article", "-k", keyword,
         "--source", "europepmc", "--limit", "25"],
        capture_output=True, text=True, env=env,
    )
    markdown_run = subprocess.run(
        [binary, "--no-cache", "search", "article", "-k", keyword,
         "--source", "europepmc", "--limit", "25"],
        capture_output=True, text=True, env=env,
    )
    payload = json.loads(json_run.stdout) if json_run.returncode == 0 else {}
    found = {str(row.get("pmid")) for row in payload.get("results", []) if row.get("pmid")}
    target = "/search?" + parsed.query
    seen_requests.add(target)
    cli_checks.append(
        json_run.returncode == 0
        and markdown_run.returncode == 0
        and expected <= found
        and all(pmid in markdown_run.stdout for pmid in expected)
        and payload.get("source_plan", {}).get("candidate_sources") == ["europepmc"]
        and payload.get("source_plan", {}).get("enrichment_sources") == []
    )

# Prove that dispatch is exact: a request absent from the corpus is refused.
try:
    urllib.request.urlopen(base + "/unknown-corpus-route", timeout=2)
except urllib.error.HTTPError as error:
    strict_unknown_rejected = error.code == 404
else:
    strict_unknown_rejected = False
server.shutdown()
thread.join()

present = {key for key, value in decoded.items() if value}
covered = {variant for variant, _ in present}
route_specific = {
    (row["variant"], row["landmark_pmid"])
    for row in mapping["landmarks"]
    if row["present"] and row["internal_route"] is not None
}
gates = {
    "reference_recall_at_least_9_of_12": len(present) >= 9,
    "variant_coverage_at_least_6_of_7": len(covered) >= 6,
    "mlh1_family_pmids_present": {
        ("MLH1 p.G67E", "19142183"), ("MLH1 p.G67E", "19493351")
    } <= present,
    "route_specific_pmids_present_for_expected_variants": route_specific <= present,
    "expected_pmid_route_diagnostics_are_binary_attributed": all(
        (not row["present"] and row["internal_route"] is None and row.get("absence_evidence"))
        or (row["present"] and row["internal_route"] in mapping["derived_internal_routes"])
        for row in mapping["landmarks"]
    ),
    "production_cli_consumed_exact_europepmc_captures": (
        all(cli_checks) and seen_requests <= set(requests)
    ),
    "compact_json_and_markdown_rendering_preserve_landmarks": all(cli_checks),
    "strict_unknown_route_rejected": strict_unknown_rejected and unknown == ["/unknown-corpus-route"],
    "terminal_states_and_work_are_pinned_by_corpus_map": (
        mapping["capture_count"] == 10
        and {row["state"] for row in mapping["state_evidence"]}
        >= {"positive", "empty", "degraded", "not_attempted"}
        and {row.get("route") for row in mapping["state_evidence"] if row["state"] == "not_attempted"}
        == {"car", "ldh"}
    ),
}
print(json.dumps(gates, indent=2, sort_keys=True))
if not all(gates.values()):
    print(json.dumps({"requests": requests, "unknown": unknown, "cli_checks": cli_checks}, indent=2), file=sys.stderr)
sys.exit(0 if all(gates.values()) else 1)
PY
