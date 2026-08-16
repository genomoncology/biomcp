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


def expected_query(query_alias, provider):
    gene, alias = query_alias.split(" ", 1)
    if provider == "pubmed":
        return f'("{gene}"[Title/Abstract] AND "{alias}"[Title/Abstract])'
    if provider == "europepmc":
        return f'TITLE_ABS:"{gene} {alias}"'
    if provider == "semanticscholar":
        return query_alias
    if provider == "pubtator":
        return f"@VARIANT_{query_alias}"
    raise AssertionError(f"unknown strict provider: {provider}")


strict_rows = {
    variant: [query for query in plans[variant] if query.get("route") == "strict"]
    for variant in variants
}


def row_is_exact(variant, row):
    provider = row.get("provider")
    query_alias = row.get("query_alias")
    gene = variant.split(" ", 1)[0]
    return (
        provider in expected_versions
        and isinstance(query_alias, str)
        and query_alias.startswith(f"{gene} ")
        and len(query_alias) > len(gene) + 1
        and row.get("query_template_version") == expected_versions[provider]
        and row.get("query") == expected_query(query_alias, provider)
    )


all_strict_templates_exact = all(
    bool(strict_rows[variant])
    and all(row_is_exact(variant, row) for row in strict_rows[variant])
    for variant in variants
)


def aliases_have_complete_provider_matrix(variant):
    grouped = {}
    for row in strict_rows[variant]:
        grouped.setdefault(row.get("query_alias"), []).append(row.get("provider"))
    return (
        variant in grouped
        and all(
            len(providers) == len(expected_versions)
            and set(providers) == set(expected_versions)
            for providers in grouped.values()
        )
    )


every_alias_has_all_four_providers = all(
    aliases_have_complete_provider_matrix(variant) for variant in variants
)


def original_pubmed_query(variant):
    return next(
        (
            row.get("query")
            for row in strict_rows[variant]
            if row.get("provider") == "pubmed" and row.get("query_alias") == variant
        ),
        None,
    )


brca1_queries = [
    original_pubmed_query("BRCA1 c.788G>T"),
    original_pubmed_query("BRCA1 c.2428A>T"),
]
brca1_aliases_remain_distinct = all(brca1_queries) and len(set(brca1_queries)) == 2
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
            "every_alias_has_all_four_providers": every_alias_has_all_four_providers,
            "brca1_aliases_remain_distinct": brca1_aliases_remain_distinct,
            "discovery_route_retained": discovery_route_retained,
            "strict_route_executed": strict_route_executed,
            "provenance_uses_query_aliases_only": provenance_uses_query_aliases_only,
        },
        indent=2,
        sort_keys=True,
    )
)
checks = (
    all_strict_templates_exact,
    every_alias_has_all_four_providers,
    brca1_aliases_remain_distinct,
    discovery_route_retained,
    strict_route_executed,
    provenance_uses_query_aliases_only,
)
raise SystemExit(0 if all(checks) else 1)
PY
