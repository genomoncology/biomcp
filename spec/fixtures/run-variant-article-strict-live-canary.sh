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
variants = [
    "APC c.847C>T",
    "TP53 c.356C>A",
    "BRCA1 c.788G>T",
    "BRCA1 c.2428A>T",
]
expected_versions = {
    "pubmed": "pubmed-title-abstract-v1",
    "europepmc": "europepmc-title-abstract-v1",
    "semanticscholar": "semantic-scholar-bulk-phrase-v1",
    "pubtator": "pubtator-entity-v1",
}
plans = {}
routes = {}
rows = []
for variant in variants:
    completed = subprocess.run(
        [
            binary,
            "--json",
            "variant",
            "articles",
            variant,
            "--strategy",
            "union",
            "--limit",
            "10",
            "--debug-plan",
        ],
        check=True,
        capture_output=True,
        text=True,
    )
    payload = json.loads(completed.stdout)
    plans[variant] = payload.get("debug_plan", {}).get("provider_queries", [])
    routes[variant] = payload.get("debug_plan", {}).get("routes", [])
    rows.extend(payload.get("results", []))


def strict_by_provider(variant):
    return {
        query.get("provider"): query
        for query in plans[variant]
        if query.get("route") == "strict"
    }


def expected_query(variant, provider):
    gene, alias = variant.split(" ", 1)
    if provider == "pubmed":
        return f'("{gene}"[Title/Abstract] AND "{alias}"[Title/Abstract])'
    if provider == "europepmc":
        return f'TITLE_ABS:"{gene} {alias}"'
    if provider == "semanticscholar":
        return variant
    return None

all_strict_templates_exact = all(
    set(strict_by_provider(variant)) == set(expected_versions)
    and all(
        strict_by_provider(variant)[provider].get("query_alias") == variant
        and strict_by_provider(variant)[provider].get("query_template_version")
        == expected_versions[provider]
        and (
            provider == "pubtator"
            or strict_by_provider(variant)[provider].get("query")
            == expected_query(variant, provider)
        )
        and (provider != "pubtator" or str(strict_by_provider(variant)[provider].get("query", "")).startswith("@VARIANT_"))
        for provider in expected_versions
    )
    for variant in variants
)
brca1_aliases_remain_distinct = (
    strict_by_provider("BRCA1 c.788G>T").get("pubmed", {}).get("query")
    != strict_by_provider("BRCA1 c.2428A>T").get("pubmed", {}).get("query")
)
discovery_route_retained = all(
    any(query.get("route") == "discovery" for query in plans[variant])
    for variant in variants
)
strict_route_executed = all(
    any(
        route.get("route") == "strict"
        and set(expected_versions).issubset(
            {
                provider.get("source")
                for provider in route.get("providers", [])
                if provider.get("calls", 0) > 0
            }
        )
        for route in routes[variant]
    )
    for variant in variants
)
provenance_uses_query_aliases_only = bool(rows) and all(
    all(
        isinstance(provenance.get("query_aliases"), list)
        and bool(provenance["query_aliases"])
        and "observed_alias" not in provenance
        and "verified_alias" not in provenance
        for provenance in row.get("provenance", [])
    )
    for row in rows
)
print(
    json.dumps(
        {
            "all_strict_templates_exact": all_strict_templates_exact,
            "brca1_aliases_remain_distinct": brca1_aliases_remain_distinct,
            "discovery_route_retained": discovery_route_retained,
            "strict_route_executed": strict_route_executed,
            "provenance_uses_query_aliases_only": provenance_uses_query_aliases_only,
        },
        indent=2,
        sort_keys=True,
    )
)
PY
