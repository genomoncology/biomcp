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
    "22799487",
    "11805335",
    "20516115",
    "18033691",
    "19142183",
    "19493351",
    "26951660",
    "17427195",
}

found_by_variant = {}
for variant in panel:
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
            "50",
        ],
        check=True,
        capture_output=True,
        text=True,
    )
    payload = json.loads(completed.stdout)
    if not payload.get("complete", False):
        raise SystemExit(f"incomplete live result for {variant}")
    found_by_variant[variant] = {
        str(row.get("pmid", "")).strip()
        for row in payload.get("results", [])
        if str(row.get("pmid", "")).strip()
    }

found_reference = set().union(
    *(found_by_variant[variant] & expected for variant, expected in panel.items())
)
covered_variants = sum(
    bool(found_by_variant[variant] & expected) for variant, expected in panel.items()
)
all_found = set().union(*found_by_variant.values())
print(
    json.dumps(
        {
            "reference_recall_at_least_9_of_12": len(found_reference) >= 9,
            "variant_coverage_at_least_6_of_7": covered_variants >= 6,
            "mlh1_family_pmids_present": {
                "19142183",
                "19493351",
            }.issubset(found_by_variant["MLH1 p.G67E"]),
            "route_specific_pmids_present": route_specific.issubset(all_found),
        },
        indent=2,
        sort_keys=True,
    )
)
PY
