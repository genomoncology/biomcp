#!/usr/bin/env bash
set -euo pipefail

repo_root="${1:-../..}"
repo_root="$(cd "$repo_root" && pwd)"
binary="${BIOMCP_BIN:-$repo_root/target/release/biomcp}"
cache_dir="$(mktemp -d "${TMPDIR:-/tmp}/biomcp-variant-article-live.XXXXXX")"
trap 'rm -rf "$cache_dir"' EXIT

missing=()
for credential in NCBI_API_KEY S2_API_KEY UMLS_API_KEY; do
    if [[ -z "${!credential:-}" ]]; then
        missing+=("$credential")
    fi
done
if ((${#missing[@]})); then
    jq -n --argjson missing "$(printf '%s\n' "${missing[@]}" | jq -R . | jq -s .)" \
        '{preflight: {required_credentials: {NCBI_API_KEY: "ncbi", S2_API_KEY: "semantic_scholar", UMLS_API_KEY: "umls"}, missing: $missing}}'
    exit 1
fi

BIOMCP_CACHE_DIR="$cache_dir" uv run --no-sync python - "$binary" <<'PY'
import json
import subprocess
import sys

binary = sys.argv[1]
panel = {
    "APC p.E1317Q": {"32461654"},
    "APC p.Q2322R": {"22799487"},
    "ATM p.C2464R": {"11805335"},
    "BRCA1 p.M1783I": {"11410501", "20516115", "21990146"},
    "MLH1 p.G67E": {"18033691", "19142183", "19493351"},
    "MSH2 p.L341P": {"26951660", "31433521"},
    "PTEN p.D326N": {"17427195"},
}
route_specific = {
    "APC p.Q2322R": {"22799487"},
    "ATM p.C2464R": {"11805335"},
    "BRCA1 p.M1783I": {"20516115"},
    "MLH1 p.G67E": {"18033691", "19142183", "19493351"},
    "MSH2 p.L341P": {"26951660"},
    "PTEN p.D326N": {"17427195"},
}


def run(variant, strategy="union"):
    completed = subprocess.run(
        [
            binary, "--no-cache", "--json", "variant", "articles", variant,
            "--strategy", strategy, "--debug-plan", "--limit", "50",
        ],
        capture_output=True,
        text=True,
    )
    try:
        payload = json.loads(completed.stdout)
    except json.JSONDecodeError:
        payload = {}
    return payload, completed.returncode


responses = {variant: run(variant) for variant in panel}
route_probes = {
    variant: {strategy: run(variant, strategy) for strategy in ("annotation", "lexical")}
    for variant in panel
}


def probe_diagnostic(payload, returncode, pmid):
    plan = payload.get("debug_plan", {})
    return {
        "command_exit": returncode,
        "terminal_state": "complete" if payload.get("complete") else "incomplete",
        "route_states": plan.get("routes", []),
        "candidate_routes": [
            row for row in plan.get("candidate_trace", {}).get("candidates", [])
            if row.get("identifier") == pmid
        ],
    }
found_by_variant = {}
routes_by_variant = {}
incomplete_variants = []
diagnostics = []
for variant, expected in panel.items():
    payload, returncode = responses[variant]
    if not payload.get("complete", False):
        incomplete_variants.append(variant)
    rows = payload.get("results", [])
    found_by_variant[variant] = {
        str(row.get("pmid", "")).strip()
        for row in rows
        if str(row.get("pmid", "")).strip()
    }
    routes_by_variant[variant] = {
        str(row.get("pmid", "")).strip(): set(row.get("retrieval_routes", []))
        for row in rows
        if str(row.get("pmid", "")).strip()
    }
    plan = payload.get("debug_plan", {})
    trace = plan.get("candidate_trace", {}).get("candidates", [])
    for pmid in sorted(expected):
        diagnostics.append({
            "variant": variant,
            "pmid": pmid,
            "found": pmid in found_by_variant[variant],
            "candidate_routes": [row for row in trace if row.get("identifier") == pmid],
            "candidate_pool_positions": [
                row["rank_position"] for row in trace
                if row.get("identifier") == pmid and row.get("rank_position") is not None
            ],
            "query_aliases": plan.get("normalized_aliases", {}),
            "provider_queries": plan.get("provider_queries", []),
            "retrieval_routes": sorted(routes_by_variant[variant].get(pmid, set())),
            "route_states": plan.get("routes", []),
            "terminal_state": "complete" if payload.get("complete") else "incomplete",
            "command_exit": returncode,
            "individual_route_probes": {
                strategy: probe_diagnostic(probe, probe_returncode, pmid)
                for strategy, (probe, probe_returncode) in route_probes[variant].items()
            },
        })

found_reference = set().union(
    *(found_by_variant[variant] & expected for variant, expected in panel.items())
)
covered_variants = sum(
    bool(found_by_variant[variant] & expected) for variant, expected in panel.items()
)
recognized_routes = {
    "pubtator_variant",
    "exact_lexical",
    "source_citation",
    "best_effort_free_text",
}
route_specific_rows_are_provenanced = all(
    pmid in found_by_variant[variant]
    and bool(routes_by_variant[variant].get(pmid, set()) & recognized_routes)
    for variant, pmids in route_specific.items()
    for pmid in pmids
)
gates = {
    "reference_recall_at_least_9_of_12": len(found_reference) >= 9,
    "variant_coverage_at_least_6_of_7": covered_variants >= 6,
    "mlh1_family_pmids_present": {"19142183", "19493351"}.issubset(found_by_variant["MLH1 p.G67E"]),
    "route_specific_pmids_present_for_expected_variants": route_specific_rows_are_provenanced,
    "expected_pmid_route_diagnostics_are_binary_attributed": all(
        diagnostic["route_states"] and diagnostic["provider_queries"]
        for diagnostic in diagnostics
    ),
}
payload = {**gates, "incomplete_variants": incomplete_variants, "expected_pmid_diagnostics": diagnostics}
print(json.dumps(payload, indent=2, sort_keys=True))
sys.exit(0 if all(gates.values()) else 1)
PY
