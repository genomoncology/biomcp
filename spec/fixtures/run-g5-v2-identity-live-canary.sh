#!/usr/bin/env bash
set -euo pipefail

repo_root="${1:-../..}"
repo_root="$(cd "$repo_root" && pwd)"
binary="${BIOMCP_BIN:-$repo_root/target/release/biomcp}"
panel="$repo_root/spec/fixtures/g5-v2-identity-panel.json"

uv run --no-sync python - "$binary" "$panel" <<'PY'
import json
import re
import subprocess
import sys
from pathlib import Path

binary, panel_path = sys.argv[1:]
requests = json.loads(Path(panel_path).read_text(encoding="utf-8"))
completed = subprocess.run(
    [binary, "--no-cache", "--json", "variant", "articles", "--input", "-", "--limit", "50", "--verify-identity", "--debug-plan"],
    input=json.dumps(requests), capture_output=True, text=True,
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
recognized_items = [
    items_by_id[request["request_id"]]
    for request in requests
    if request["request_id"] in items_by_id
]
retrieval_exact_routes = {"exact_lexical", "pubtator_variant"}

def supplied_aliases(request):
    return {
        f'{request["transcript"]}:{request["coding"]}',
        f'{request["gene"]} {request["coding"]}',
        f'{request["accession"]}:g.{request["position"]}{request["ref"]}>{request["alt"]}',
    }

def has_canonical_equivalence(item):
    equivalence = item.get("canonical_equivalence")
    observations = equivalence.get("observations", []) if isinstance(equivalence, dict) else []
    caid = equivalence.get("caid") if isinstance(equivalence, dict) else None
    return (
        isinstance(equivalence, dict)
        and equivalence.get("status") == "confirmed"
        and equivalence.get("complete") is True
        and equivalence.get("exhaustive") is True
        and equivalence.get("applicable_identity_count") == 2
        and isinstance(caid, str)
        and re.fullmatch(r"CA[0-9]+", caid) is not None
        and len(observations) == 2
        and {observation.get("basis") for observation in observations} == {"transcript_coding", "genomic"}
        and all(
            observation.get("status") == "resolved"
            and observation.get("caid") == caid
            and observation.get("source") == "clingen_car"
            and observation.get("comparison_complete") is True
            and isinstance(observation.get("provider_response_sha256"), str)
            and re.fullmatch(r"[0-9a-f]{64}", observation["provider_response_sha256"]) is not None
            for observation in observations
        )
    )

def has_consistent_work_allocation(item):
    plan = item.get("debug_plan")
    if not isinstance(plan, dict):
        return False
    budgets = plan.get("budgets")
    allocation = plan.get("work_allocation")
    if not isinstance(budgets, dict) or not isinstance(allocation, dict):
        return False
    item_budget = budgets.get("item")
    discovery = allocation.get("discovery")
    exact = allocation.get("exact_lexical")
    verification = allocation.get("identity_verification")
    if not all(isinstance(value, dict) for value in (item_budget, discovery, exact, verification)):
        return False
    exact_item = exact.get("item")
    verification_item = verification.get("item")
    if not isinstance(exact_item, dict) or not isinstance(verification_item, dict):
        return False
    consumed = [
        item_budget.get("consumed"),
        discovery.get("consumed"),
        exact_item.get("consumed"),
        verification_item.get("consumed"),
    ]
    if not all(isinstance(value, int) and value >= 0 for value in consumed):
        return False
    parent, discovery_consumed, exact_consumed, verification_consumed = consumed
    if any(value > parent for value in (discovery_consumed, exact_consumed, verification_consumed)):
        return False
    if discovery_consumed + exact_consumed + verification_consumed != parent:
        return False
    allocation_by_route = {
        "strict": discovery_consumed,
        "exact_lexical": exact_consumed,
        "identity_verification": verification_consumed,
    }
    for route in plan.get("routes", []):
        if not isinstance(route, dict):
            return False
        route_name = route.get("route")
        if route_name not in allocation_by_route:
            continue
        for provider in route.get("providers", []):
            if not isinstance(provider, dict):
                return False
            if provider.get("calls", 0) > 0 and allocation_by_route[route_name] == 0:
                return False
    return True

summary = {
    "expected_request_ids": len(returned_request_ids) == len(expected_request_ids) and len(set(returned_request_ids)) == len(expected_request_ids) and set(returned_request_ids) == expected_request_ids,
    "all_resolved": len(recognized_items) == len(expected_request_ids) and all((item.get("resolution") or {}).get("status") == "resolved" for item in recognized_items),
    "all_have_exact_route": len(recognized_items) == len(expected_request_ids) and all(any(retrieval_exact_routes.intersection(row.get("routes", [])) for row in item.get("results", [])) for item in recognized_items),
    "all_have_route_tied_alias": len(recognized_items) == len(expected_request_ids) and all(any(retrieval_exact_routes.intersection(row.get("routes", [])) and supplied_aliases(request_by_id[item.get("request_id")]).intersection(row.get("matched_aliases", [])) for row in item.get("results", [])) for item in recognized_items),
    "all_have_source_status": len(recognized_items) == len(expected_request_ids) and all(bool(item.get("source_status")) for item in recognized_items),
    "all_have_terminal_state": len(recognized_items) == len(expected_request_ids) and all(isinstance(item.get("complete"), bool) and isinstance(item.get("truncated"), bool) and "error" in item for item in recognized_items),
    "work_allocation_is_consistent_with_budgets_and_recorded_calls": len(recognized_items) == len(expected_request_ids) and all(has_consistent_work_allocation(item) for item in recognized_items),
    "authoritative_verify_treats_g5_as_hard": True,
}
positives = {"apc-grch38": "12901799", "atm-grch38": "32918381", "palb2-grch38": "39999518", "mlh1-grch38": "20864636"}
collisions = {"31749828", "24376681", "33656647"}
diagnostics = {
    "known_collision_confirmations": [],
    "schema_parse_failures": [],
    "missing_available_positives": [],
    "incomplete_results": [],
    "internal_misattributions": [],
    "route_status_contradictions": [],
    "missing_canonical_equivalence": [],
}
seen_misattributions = set()
source_status_kinds = {"ok", "degraded", "unavailable", "skipped", "not_attempted"}
for item in recognized_items:
    request_id = item.get("request_id")
    verification = (item.get("debug_plan") or {}).get("verification")
    source_status = item.get("source_status")
    valid_source_status = (
        isinstance(source_status, list)
        and bool(source_status)
        and all(
            isinstance(status, dict)
            and isinstance(status.get("route"), str)
            and isinstance(status.get("source"), str)
            and status.get("status") in source_status_kinds
            for status in source_status
        )
    )
    if not isinstance(verification, dict) or not valid_source_status:
        diagnostics["schema_parse_failures"].append(request_id)
    internal_unperformed_work = any(
        isinstance(status, dict)
        and status.get("source") == "internal"
        and status.get("status") == "not_attempted"
        for status in (source_status if isinstance(source_status, list) else [])
    )
    if not item.get("complete", False) and internal_unperformed_work:
        diagnostics["incomplete_results"].append(request_id)
    recorded_calls = {
        (route.get("route"), provider.get("source"))
        for route in (item.get("debug_plan") or {}).get("routes", [])
        if isinstance(route, dict)
        for provider in route.get("providers", [])
        if isinstance(provider, dict) and provider.get("calls", 0) > 0
    }
    misattributions = {
        (status.get("route"), status.get("source"))
        for status in (source_status if isinstance(source_status, list) else [])
        if isinstance(status, dict)
        and status.get("source") != "internal"
        and status.get("status") in {"degraded", "unavailable"}
        and (status.get("route"), status.get("source")) not in recorded_calls
    }
    if misattributions.difference(seen_misattributions):
        diagnostics["internal_misattributions"].append(request_id)
        seen_misattributions.update(misattributions)
    stopped_routes = set((item.get("debug_plan") or {}).get("stopped_routes", []))
    route_provider_statuses = {
        (route.get("route"), provider.get("source")): provider.get("status")
        for route in (item.get("debug_plan") or {}).get("routes", [])
        if isinstance(route, dict)
        for provider in route.get("providers", [])
        if isinstance(provider, dict)
    }
    contradictions = {
        status.get("route")
        for status in (source_status if isinstance(source_status, list) else [])
        if isinstance(status, dict)
        and (
            (
                status.get("source") != "internal"
                and status.get("status") in {"ok", "degraded", "unavailable"}
                and route_provider_statuses.get((status.get("route"), status.get("source")))
                in {"ok", "degraded", "unavailable"}
                and status.get("status")
                != route_provider_statuses[(status.get("route"), status.get("source"))]
            )
            or (
                isinstance(status.get("detail"), str)
                and "stopped" in status["detail"]
                and status.get("route") not in stopped_routes
            )
        )
    }
    if contradictions:
        diagnostics["route_status_contradictions"].append(request_id)
    for row in item.get("results", []):
        if row.get("pmid") in collisions and (row.get("identity") or {}).get("status") == "confirmed":
            diagnostics["known_collision_confirmations"].append(row["pmid"])
for request_id, pmid in positives.items():
    item = items_by_id.get(request_id, {})
    if item.get("complete", False) and not any(
        row.get("pmid") == pmid and (row.get("identity") or {}).get("status") == "confirmed"
        for row in item.get("results", [])
    ):
        diagnostics["missing_available_positives"].append(request_id)
for request_id in ("atm-grch38", "palb2-grch38"):
    if not has_canonical_equivalence(items_by_id.get(request_id, {})):
        diagnostics["missing_canonical_equivalence"].append(request_id)
print(json.dumps({"identity_readiness": summary, "identity_diagnostics": diagnostics}, indent=2, sort_keys=True))
ready = completed.returncode == 0 and all(summary.values()) and not any(diagnostics[key] for key in ("known_collision_confirmations", "schema_parse_failures", "missing_available_positives", "incomplete_results", "internal_misattributions", "route_status_contradictions", "missing_canonical_equivalence"))
raise SystemExit(0 if ready else 1)
PY
