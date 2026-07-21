#!/usr/bin/env bash
set -euo pipefail

repo_root="${1:-../..}"
repo_root="$(cd "$repo_root" && pwd)"
binary="${BIOMCP_BIN:-$repo_root/target/release/biomcp}"

uv run --no-sync python - "$binary" <<'PY'
import json
import subprocess
import sys

binary = sys.argv[1]
requests = [
    {
        "request_id": "apc-grch38",
        "gene": "APC",
        "transcript": "NM_000038.6",
        "coding": "c.847C>T",
        "protein": "p.Arg283Ter",
        "accession": "NC_000005.10",
        "build": "GRCh38",
        "position": 112815507,
        "ref": "C",
        "alt": "T",
    },
    {
        "request_id": "atm-grch38",
        "gene": "ATM",
        "transcript": "NM_000051.4",
        "coding": "c.1066-6T>G",
        "accession": "NC_000011.10",
        "build": "GRCh38",
        "position": 108248927,
        "ref": "T",
        "alt": "G",
    },
    {
        "request_id": "brca1-grch38",
        "gene": "BRCA1",
        "transcript": "NM_007294.4",
        "coding": "c.2428A>T",
        "protein": "p.Asn810Tyr",
        "accession": "NC_000017.11",
        "build": "GRCh38",
        "position": 43093103,
        "ref": "T",
        "alt": "A",
    },
    {
        "request_id": "mlh1-grch38",
        "gene": "MLH1",
        "transcript": "NM_000249.4",
        "coding": "c.2246T>C",
        "protein": "p.Leu749Pro",
        "accession": "NC_000003.12",
        "build": "GRCh38",
        "position": 37050628,
        "ref": "T",
        "alt": "C",
    },
    {
        "request_id": "palb2-grch38",
        "gene": "PALB2",
        "transcript": "NM_024675.4",
        "coding": "c.3350+5G>A",
        "accession": "NC_000016.10",
        "build": "GRCh38",
        "position": 23607859,
        "ref": "C",
        "alt": "T",
    },
    {
        "request_id": "pten-grch38",
        "gene": "PTEN",
        "transcript": "NM_000314.8",
        "coding": "c.517C>T",
        "protein": "p.Arg173Cys",
        "accession": "NC_000010.11",
        "build": "GRCh38",
        "position": 87952142,
        "ref": "C",
        "alt": "T",
    },
    {
        "request_id": "tp53-grch38",
        "gene": "TP53",
        "transcript": "NM_000546.6",
        "coding": "c.356C>G",
        "protein": "p.Ala119Gly",
        "accession": "NC_000017.11",
        "build": "GRCh38",
        "position": 7676013,
        "ref": "G",
        "alt": "C",
    },
]

completed = subprocess.run(
    [
        binary,
        "--no-cache",
        "--json",
        "variant",
        "articles",
        "--input",
        "-",
        "--limit",
        "50",
    ],
    input=json.dumps(requests),
    capture_output=True,
    text=True,
)
try:
    payload = json.loads(completed.stdout)
except json.JSONDecodeError as error:
    print(json.dumps({"error": f"BioMCP returned non-JSON output: {error}"}, indent=2))
    raise SystemExit(1) from error

items = payload.get("items", [])
request_by_id = {request["request_id"]: request for request in requests}
expected_request_ids = set(request_by_id)
returned_request_ids = [item.get("request_id") for item in items]
items_by_id = {item.get("request_id"): item for item in items}
recognized_items = [items_by_id[request_id] for request_id in expected_request_ids if request_id in items_by_id]
retrieval_exact_routes = {"exact_lexical", "pubtator_variant"}


def supplied_aliases(request):
    aliases = {
        f'{request["transcript"]}:{request["coding"]}',
        f'{request["gene"]} {request["coding"]}',
        f'{request["accession"]}:g.{request["position"]}{request["ref"]}>{request["alt"]}',
    }
    return aliases


resolved = sum(
    (item.get("resolution") or {}).get("status") == "resolved" for item in recognized_items
)
with_exact_route = sum(
    any(
        retrieval_exact_routes.intersection(row.get("routes", []))
        for row in item.get("results", [])
    )
    for item in recognized_items
)
with_route_tied_alias = sum(
    any(
        retrieval_exact_routes.intersection(row.get("routes", []))
        and supplied_aliases(request_by_id[item.get("request_id")]).intersection(
            row.get("matched_aliases", [])
        )
        for row in item.get("results", [])
    )
    for item in recognized_items
)
with_source_status = sum(bool(item.get("source_status")) for item in recognized_items)
with_terminal_state = sum(
    isinstance(item.get("complete"), bool)
    and isinstance(item.get("truncated"), bool)
    and "error" in item
    for item in recognized_items
)
summary = {
    "expected_request_ids": (
        len(returned_request_ids) == 7
        and len(set(returned_request_ids)) == 7
        and set(returned_request_ids) == expected_request_ids
    ),
    "total": len(items),
    "resolved": resolved,
    "with_exact_route": with_exact_route,
    "with_route_tied_alias": with_route_tied_alias,
    "with_source_status": with_source_status,
    "with_terminal_state": with_terminal_state,
}
print(json.dumps({"identity_readiness": summary}, indent=2, sort_keys=True))
ready = (
    completed.returncode == 0
    and summary["expected_request_ids"]
    and all(value == 7 for key, value in summary.items() if key != "expected_request_ids")
)
raise SystemExit(0 if ready else 1)
PY
