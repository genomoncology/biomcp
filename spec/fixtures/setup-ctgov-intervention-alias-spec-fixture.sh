#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ownership_helper="$script_dir/routine-fixture-ownership.sh"
# shellcheck source=fixture-supervisor.sh
source "$script_dir/fixture-supervisor.sh"

workspace_root="${1:-$PWD}"
cache_dir="$workspace_root/.cache"
env_file="$cache_dir/spec-ctgov-intervention-alias-env"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cleanup_script="$script_dir/cleanup-ctgov-intervention-alias-spec-fixture.sh"

mkdir -p "$cache_dir"

if [ -x "$cleanup_script" ]; then
  bash "$cleanup_script" "$workspace_root"
fi
recover_fixture_orphans "$cache_dir" "ctgov-intervention-alias" "spec-ctgov-intervention-alias."

fixture_root="$(mktemp -d "$cache_dir/spec-ctgov-intervention-alias.XXXXXX")"
owner_arg="$(bash "$ownership_helper" new-owner "ctgov-intervention-alias" "$fixture_root")"
ready_file="$fixture_root/base-url"
server_pid_file="$fixture_root/server-pid"
server_log="$fixture_root/server.log"
request_log="$fixture_root/request.log"
fixture_pgid=""
: >"$request_log"

cleanup_incomplete_setup() {
  if [ -n "$fixture_pgid" ]; then
    kill -TERM -- "-$fixture_pgid" 2>/dev/null || true
    wait "$fixture_pgid" 2>/dev/null || true
  fi
  rm -rf "$fixture_root"
}
trap cleanup_incomplete_setup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
trap 'exit 129' HUP

prepare_fixture_supervisor_owner
start_fixture_supervisor "ctgov-intervention-alias" "$cache_dir" "$fixture_root" "spec-ctgov-intervention-alias." "$server_pid_file" \
  python3 - "$ready_file" "$request_log" "$server_pid_file" "$owner_arg" "$workspace_root" <<'PY' >"$server_log" 2>&1 &
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlparse
import json
import os
import sys

SOURCES = Path(sys.argv[5]) / "testdata/sources"


def source_bytes(path):
    return (SOURCES / path).read_bytes()


def send_json(handler, status, payload):
    body = json.dumps(payload).encode("utf-8")
    handler.send_response(status)
    handler.send_header("Content-Type", "application/json")
    handler.send_header("Content-Length", str(len(body)))
    handler.end_headers()
    handler.wfile.write(body)


def send_text(handler, status, payload):
    body = payload.encode("utf-8")
    handler.send_response(status)
    handler.send_header("Content-Type", "text/plain; charset=utf-8")
    handler.send_header("Content-Length", str(len(body)))
    handler.end_headers()
    handler.wfile.write(body)


def send_bytes(handler, status, payload, content_type="application/octet-stream"):
    handler.send_response(status)
    handler.send_header("Content-Type", content_type)
    handler.send_header("Content-Length", str(len(payload)))
    handler.end_headers()
    handler.wfile.write(payload)


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

VENETOCLAX_STUDY = {
    "protocolSection": {
        "identificationModule": {
            "nctId": "NCT51000001",
            "briefTitle": "Literal Venetoclax Trial",
        },
        "statusModule": {"overallStatus": "RECRUITING"},
        "descriptionModule": {"briefSummary": "Fixture result for the requested intervention."},
        "conditionsModule": {"conditions": ["Chronic Lymphocytic Leukemia"]},
        "designModule": {
            "phases": ["PHASE2"],
            "studyType": "Interventional",
            "enrollmentInfo": {"count": 20},
        },
        "armsInterventionsModule": {
            "interventions": [{"type": "DRUG", "name": "venetoclax"}],
            "armGroups": [],
        },
        "eligibilityModule": {
            "eligibilityCriteria": "Inclusion Criteria: eligible adults may enroll."
        },
    }
}

VENCLEXTA_STUDY = {
    "protocolSection": {
        "identificationModule": {
            "nctId": "NCT51000002",
            "briefTitle": "Venclexta Alias Trial",
        },
        "statusModule": {"overallStatus": "RECRUITING"},
        "descriptionModule": {"briefSummary": "Fixture result for a plausible trade alias."},
        "conditionsModule": {"conditions": ["Acute Myeloid Leukemia"]},
        "designModule": {
            "phases": ["PHASE2"],
            "studyType": "Interventional",
            "enrollmentInfo": {"count": 18},
        },
        "armsInterventionsModule": {
            "interventions": [{"type": "DRUG", "name": "Venclexta"}],
            "armGroups": [],
        },
        "eligibilityModule": {
            "eligibilityCriteria": "Inclusion Criteria: eligible adults may enroll."
        },
    }
}

CONTINUATION_REJECTED_STUDY = {
    "protocolSection": {
        "identificationModule": {"nctId": "NCT51000003", "briefTitle": "Rejected Fanout Page Fixture"},
        "statusModule": {"overallStatus": "RECRUITING"},
        "eligibilityModule": {"eligibilityCriteria": "Exclusion Criteria: nextpageproof"},
    }
}

CONTINUATION_QUALIFYING_STUDY = {
    "protocolSection": {
        "identificationModule": {"nctId": "NCT51000004", "briefTitle": "Qualifying Continuation Fixture"},
        "statusModule": {"overallStatus": "RECRUITING"},
        "eligibilityModule": {"eligibilityCriteria": "Inclusion Criteria: nextpageproof"},
    }
}

VENETOCLAX_MYCHEM_RESPONSE = {
    "total": 1,
    "hits": [
        {
            "_id": "DB11581",
            "_score": 42.0,
            "drugbank": {
                "id": "DB11581",
                "name": "venetoclax",
                "synonyms": [
                    "Venclexta",
                    "4-[4-[[2-(4-chlorophenyl)-4,4-dimethylcyclohex-1-enyl]methyl]piperazin-1-yl]benzoic acid",
                    "ABT-199 (venetoclax free base)",
                ],
            },
            "openfda": {
                "generic_name": ["venetoclax"],
                "brand_name": ["Parser Trap", "Venclexta"],
            },
        }
    ],
}

KARMMA_DOCUMENT_BYTES = (
    b"%PDF-1.7\nBioMCP CTGov protocol fixture.\n\x00\xff\r\n%%EOF\n"
)

KARMMA_STUDY = {
    "protocolSection": {
        "identificationModule": {
            "nctId": "NCT03361748",
            "briefTitle": "KarMMa-1 Document Fixture",
        },
        "statusModule": {"overallStatus": "COMPLETED"},
        "conditionsModule": {"conditions": ["Multiple Myeloma"]},
        "designModule": {
            "phases": ["PHASE2"],
            "studyType": "Interventional",
            "enrollmentInfo": {"count": 149},
        },
        "armsInterventionsModule": {"interventions": [], "armGroups": []},
        "eligibilityModule": {
            "minimumAge": "18 Years",
            "maximumAge": "N/A",
            "sex": "ALL",
            "eligibilityCriteria": "Inadequate organ function",
        },
    },
    "documentSection": {
        "largeDocumentModule": {
            "largeDocs": [
                {
                    "typeAbbrev": "Prot_SAP",
                    "hasProtocol": True,
                    "hasSap": True,
                    "hasIcf": False,
                    "label": "Study Protocol and Statistical Analysis Plan",
                    "date": "2019-07-18",
                    "uploadDate": "2024-12-12T10:49",
                    "filename": "Prot_SAP_000.pdf",
                    "size": len(KARMMA_DOCUMENT_BYTES),
                }
            ]
        }
    },
}

CONTACTS_ELIGIBILITY_STUDY = {
    "protocolSection": {
        "identificationModule": {
            "nctId": "NCT41300001",
            "briefTitle": "Central and Site Contact Fixture",
        },
        "statusModule": {"overallStatus": "RECRUITING"},
        "descriptionModule": {
            "briefSummary": "Fixture study for trial contact and eligibility detail."
        },
        "conditionsModule": {"conditions": ["Phelan-McDermid Syndrome"]},
        "designModule": {
            "phases": ["PHASE2"],
            "studyType": "Interventional",
            "enrollmentInfo": {"count": 24},
        },
        "armsInterventionsModule": {"interventions": [], "armGroups": []},
        "eligibilityModule": {
            "minimumAge": "2 Years",
            "maximumAge": "18 Years",
            "sex": "FEMALE",
            "eligibilityCriteria": "Key inclusion: confirmed SHANK3-related neurodevelopmental disorder."
        },
        "contactsLocationsModule": {
            "centralContacts": [
                {
                    "name": "Central Coordinator",
                    "role": "CONTACT",
                    "phone": "555-0100",
                    "email": "central@example.test",
                }
            ],
            "locations": [
                {
                    "facility": "Rare Disease Center",
                    "city": "Ann Arbor",
                    "state": "Michigan",
                    "country": "United States",
                    "status": "RECRUITING",
                    "geoPoint": {"lat": 42.2808, "lon": -83.7430},
                    "contacts": [
                        {
                            "name": "Site Coordinator",
                            "role": "CONTACT",
                            "phone": "555-0199",
                            "email": "site@example.test",
                        }
                    ],
                }
            ],
        },
    }
}


STUDIES = {
    "nct02136914": NCT02136914_STUDY,
    "nct03361748": KARMMA_STUDY,
    "nct35700001": SHELL_SAFE_STUDY,
    "nct41300001": CONTACTS_ELIGIBILITY_STUDY,
    "nct51000001": VENETOCLAX_STUDY,
    "nct51000002": VENCLEXTA_STUDY,
    "nct51000003": CONTINUATION_REJECTED_STUDY,
    "nct51000004": CONTINUATION_QUALIFYING_STUDY,
}
CTGOV_SEARCH = {
    "melanoma": source_bytes("ctgov/search_melanoma_recruiting_limit3_20260811.json"),
    "phelan-50": source_bytes("ctgov/search_phelan_limit50_20260811.json"),
    "phelan-5": source_bytes("ctgov/search_phelan_limit5_20260811.json"),
    "phelan-next": source_bytes("ctgov/search_phelan_next_20260811.json"),
    "mutation": source_bytes("ctgov/search_nsclc_egfr_l858r_20260811.json"),
    "keytruda": source_bytes("ctgov/search_keytruda_limit3_20260811.json"),
    "age-count": source_bytes("ctgov/search_age_count_20260811.json"),
}
CTGOV_DETAIL = {
    "nct02576665": source_bytes("ctgov/get_nct02576665_20260811.json"),
    "nct06382129": source_bytes("ctgov/get_nct06382129_20260811.json"),
    "nct06604689": source_bytes("ctgov/get_nct06604689_20260811.json"),
}


def study_payload_for_request(parsed, study):
    payload = json.loads(json.dumps(study))
    fields = ",".join(parse_qs(parsed.query).get("fields", []))
    requested_fields = {field.strip() for field in fields.split(",") if field.strip()}
    if "InterventionOtherName" not in requested_fields:
        interventions = payload["protocolSection"]["armsInterventionsModule"]["interventions"]
        for intervention in interventions:
            intervention.pop("otherNames", None)
    if "LargeDocumentModule" not in requested_fields:
        payload.pop("documentSection", None)
    return payload


REQUEST_LOG = Path(sys.argv[2])
SERVER_PID_FILE = Path(sys.argv[3])
SERVER_PID_FILE.write_text(f"{os.getpid()}\n", encoding="utf-8")


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        parsed = urlparse(self.path)
        query = parse_qs(parsed.query)
        if parsed.path == "/large-docs/48/NCT03361748/Prot_SAP_000.pdf":
            send_bytes(self, 200, KARMMA_DOCUMENT_BYTES, "application/pdf")
            return
        if parsed.path == "/v1/query":
            if " ".join(query.get("q", [])).strip().lower() == "venetoclax":
                send_json(self, 200, VENETOCLAX_MYCHEM_RESPONSE)
            else:
                send_json(self, 200, {"total": 0, "hits": []})
            return
        if parsed.path == "/api/v2/studies":
            with REQUEST_LOG.open("a", encoding="utf-8") as log:
                log.write(f"{self.path}\n")
            intervention = " ".join(query.get("query.intr", [])).strip()
            condition = " ".join(query.get("query.cond", [])).strip()
            page_size = query.get("pageSize", [""])[0]
            requested_facility = " ".join(query.get("query.locn", [])).lower()
            if "university of michigan" in requested_facility:
                send_json(self, 200, {"studies": [], "totalCount": 0})
                return
            if condition == "melanoma" and query.get("filter.overallStatus") == ["RECRUITING"]:
                send_bytes(self, 200, CTGOV_SEARCH["melanoma"], "application/json")
                return
            if condition == "Phelan-McDermid Syndrome":
                if query.get("pageToken"):
                    send_bytes(self, 200, CTGOV_SEARCH["phelan-next"], "application/json")
                elif page_size == "50":
                    send_bytes(self, 200, CTGOV_SEARCH["phelan-50"], "application/json")
                else:
                    send_bytes(self, 200, CTGOV_SEARCH["phelan-5"], "application/json")
                return
            if condition == "non-small cell lung cancer" and "EGFR L858R" in " ".join(query.get("query.term", [])):
                send_bytes(self, 200, CTGOV_SEARCH["mutation"], "application/json")
                return
            is_quoted_literal = (
                len(intervention) >= 2
                and intervention.startswith('"')
                and intervention.endswith('"')
            )
            literal_intervention = intervention[1:-1] if is_quoted_literal else intervention
            continuation_proof = "nextpageproof" in " ".join(query.get("query.term", [])).lower()
            if continuation_proof and is_quoted_literal and literal_intervention == "venetoclax":
                if "venetoclax-criteria-page-2" in query.get("pageToken", []):
                    send_json(self, 200, {"studies": [], "totalCount": 1})
                else:
                    send_json(self, 200, {"studies": [CONTINUATION_REJECTED_STUDY], "totalCount": 2, "nextPageToken": "venetoclax-criteria-page-2"})
                return
            if continuation_proof and is_quoted_literal and literal_intervention == "Venclexta":
                if "venclexta-criteria-page-2" in query.get("pageToken", []):
                    send_json(self, 200, {"studies": [CONTINUATION_QUALIFYING_STUDY], "totalCount": 2})
                else:
                    send_json(self, 200, {"studies": [CONTINUATION_REJECTED_STUDY], "totalCount": 2, "nextPageToken": "venclexta-criteria-page-2"})
                return
            if is_quoted_literal and literal_intervention == "venetoclax":
                send_json(self, 200, {"studies": [VENETOCLAX_STUDY], "totalCount": 1})
                return
            if is_quoted_literal and literal_intervention == "Venclexta":
                if "venclexta-page-2" in query.get("pageToken", []):
                    send_json(
                        self,
                        200,
                        {"studies": [VENETOCLAX_STUDY], "totalCount": 2},
                    )
                else:
                    send_json(
                        self,
                        200,
                        {
                            "studies": [VENETOCLAX_STUDY, VENCLEXTA_STUDY],
                            "totalCount": 2,
                            "nextPageToken": "venclexta-page-2",
                        },
                    )
                return
            if is_quoted_literal and literal_intervention in {
                "Keytruda",
                "pembrolizumab",
                "ABP 234",
                "GME751",
                "Lambrolizumab",
            }:
                send_bytes(self, 200, CTGOV_SEARCH["keytruda"], "application/json")
                return
            if intervention:
                send_text(
                    self,
                    400,
                    "Error parsing query in Intervention / treatment: invalid expression",
                )
                return
            if (
                page_size == "1"
                and not condition
                and not query.get("query.term")
                and not requested_facility
                and not query.get("filter.geo")
            ):
                send_bytes(self, 200, CTGOV_SEARCH["age-count"], "application/json")
                return
            send_json(self, 200, {"studies": [study_payload_for_request(parsed, CONTACTS_ELIGIBILITY_STUDY)], "totalCount": 1})
            return
        if parsed.path.startswith("/api/v2/studies/"):
            with REQUEST_LOG.open("a", encoding="utf-8") as log:
                log.write(f"{self.path}\n")
            nct_id = parsed.path.rsplit("/", 1)[-1].lower()
            if nct_id in CTGOV_DETAIL:
                send_bytes(self, 200, CTGOV_DETAIL[nct_id], "application/json")
                return
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
supervisor_pid=$!

for _ in $(seq 1 50); do
  if [ -s "$ready_file" ]; then
    break
  fi
  if ! kill -0 "$supervisor_pid" 2>/dev/null; then
    cat "$server_log" >&2
    exit 1
  fi
  sleep 0.1
done

test -s "$ready_file"
test -s "$server_pid_file"
fixture_pgid="$(<"$server_pid_file")"
base_url="$(cat "$ready_file")"

printf 'export BIOMCP_CTGOV_BASE=%q\n' "$base_url/api/v2" >"$env_file"
printf 'export BIOMCP_CTGOV_CDN_BASE=%q\n' "$base_url" >>"$env_file"
printf 'export BIOMCP_CTGOV_INTERVENTION_ALIAS_MYCHEM_BASE=%q\n' "$base_url/v1" >>"$env_file"
printf 'export BIOMCP_CACHE_MODE=off\n' >>"$env_file"
printf 'export BIOMCP_CTGOV_INTERVENTION_ALIAS_ROOT=%q\n' "$fixture_root" >>"$env_file"
printf 'export BIOMCP_CTGOV_INTERVENTION_ALIAS_READY_FILE=%q\n' "$ready_file" >>"$env_file"
printf 'export BIOMCP_CTGOV_INTERVENTION_ALIAS_REQUEST_LOG=%q\n' "$request_log" >>"$env_file"

bash "$ownership_helper" write "$workspace_root" "ctgov-intervention-alias" "$fixture_root" "$fixture_pgid" "BIOMCP_CTGOV_INTERVENTION_ALIAS" "$owner_arg" >/dev/null
trap - EXIT INT TERM HUP
printf '%s\n' "$fixture_root"
