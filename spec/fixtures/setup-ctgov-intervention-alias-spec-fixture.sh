#!/usr/bin/env bash
set -euo pipefail

workspace_root="${1:-$PWD}"
cache_dir="$workspace_root/.cache"
env_file="$cache_dir/spec-ctgov-intervention-alias-env"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cleanup_script="$script_dir/cleanup-ctgov-intervention-alias-spec-fixture.sh"

mkdir -p "$cache_dir"

if [ -x "$cleanup_script" ]; then
  bash "$cleanup_script" "$workspace_root"
fi

fixture_root="$(mktemp -d "$cache_dir/spec-ctgov-intervention-alias.XXXXXX")"
ready_file="$fixture_root/base-url"
server_log="$fixture_root/server.log"

uv run --no-sync python - "$ready_file" <<'PY' >"$server_log" 2>&1 &
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


NCT02136914_STUDY = {
    "protocolSection": {
        "identificationModule": {
            "nctId": "NCT02136914",
            "briefTitle": "ADS-5102 for Levodopa Induced Dyskinesia",
        },
        "statusModule": {"overallStatus": "COMPLETED"},
        "descriptionModule": {
            "briefSummary": "A study of investigational extended-release capsules for levodopa induced dyskinesia."
        },
        "conditionsModule": {"conditions": ["Parkinson Disease", "Dyskinesia"]},
        "designModule": {
            "phases": ["PHASE3"],
            "studyType": "Interventional",
            "enrollmentInfo": {"count": 126},
        },
        "armsInterventionsModule": {
            "interventions": [
                {
                    "type": "DRUG",
                    "name": "ADS-5102",
                    "description": "Oral capsules administered once nightly at bedtime.",
                    "armGroupLabels": ["ADS-5102"],
                    "otherNames": ["amantadine HCl extended release"],
                }
            ],
            "armGroups": [
                {
                    "label": "ADS-5102",
                    "type": "EXPERIMENTAL",
                    "description": "Investigational active treatment arm.",
                    "interventionNames": [],
                }
            ],
        },
    }
}

SHELL_SAFE_STUDY = {
    "protocolSection": {
        "identificationModule": {
            "nctId": "NCT35700001",
            "briefTitle": "Shell Safety Fixture",
        },
        "statusModule": {"overallStatus": "RECRUITING"},
        "descriptionModule": {"briefSummary": "Fixture study for source-derived command text."},
        "conditionsModule": {
            "conditions": ["quoted $(touch /tmp/biomcp-357-pwned) \"condition\""]
        },
        "designModule": {
            "phases": ["PHASE1"],
            "studyType": "Interventional",
            "enrollmentInfo": {"count": 1},
        },
        "armsInterventionsModule": {
            "interventions": [
                {
                    "type": "DRUG",
                    "name": "SAFE-357",
                    "description": "Fixture intervention for command escaping.",
                    "armGroupLabels": ["SAFE-357"],
                    "otherNames": ["alias $(touch /tmp/biomcp-357-pwned) \"dose\""],
                }
            ],
            "armGroups": [],
        },
    }
}

STUDIES = {
    "nct02136914": NCT02136914_STUDY,
    "nct35700001": SHELL_SAFE_STUDY,
}


def study_payload_for_request(parsed, study):
    payload = json.loads(json.dumps(study))
    fields = ",".join(parse_qs(parsed.query).get("fields", []))
    requested_fields = {field.strip() for field in fields.split(",") if field.strip()}
    if "InterventionOtherName" not in requested_fields:
        interventions = payload["protocolSection"]["armsInterventionsModule"]["interventions"]
        for intervention in interventions:
            intervention.pop("otherNames", None)
    return payload


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        parsed = urlparse(self.path)
        if parsed.path.startswith("/api/v2/studies/"):
            nct_id = parsed.path.rsplit("/", 1)[-1].lower()
            if nct_id in STUDIES:
                send_json(self, 200, study_payload_for_request(parsed, STUDIES[nct_id]))
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

printf 'export BIOMCP_CTGOV_BASE=%q\n' "$base_url/api/v2" >"$env_file"
printf 'export BIOMCP_CACHE_MODE=off\n' >>"$env_file"
printf 'export BIOMCP_CTGOV_INTERVENTION_ALIAS_PID=%q\n' "$server_pid" >>"$env_file"
printf 'export BIOMCP_CTGOV_INTERVENTION_ALIAS_ROOT=%q\n' "$fixture_root" >>"$env_file"
printf 'export BIOMCP_CTGOV_INTERVENTION_ALIAS_READY_FILE=%q\n' "$ready_file" >>"$env_file"

printf '%s\n' "$fixture_root"
