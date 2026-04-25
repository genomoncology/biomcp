#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

HELPER_MODULES = {"cbioportal_download", "cbioportal_study", "rate_limit"}
EXEMPT_MODULES = {"cvx", "ddinter", "ema", "gtr", "who_ivd", "who_pq"}
HEALTH_ALIASES = {
    "cbioportal": "cBioPortal",
    "clinicaltrials": "ClinicalTrials.gov",
    "ddinter": "DDInter local data",
    "gprofiler": "g:Profiler",
    "gwas": "GWAS Catalog",
    "ncbi_efetch": "NCBI E-utilities",
    "ncbi_idconv": "NCBI ID Converter",
    "nci_cts": "NCI CTS",
    "pmc_oa": "PMC OA",
    "pubtator": "PubTator3",
    "seer": "SEER Explorer",
    "semantic_scholar": "Semantic Scholar",
    "vaers": "CDC WONDER VAERS",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Audit BioMCP source-module registration against health inventory.",
    )
    parser.add_argument("--sources-dir", type=Path, default=Path("src/sources"))
    parser.add_argument("--sources-mod", type=Path, default=Path("src/sources/mod.rs"))
    parser.add_argument("--health-file", type=Path, default=Path("src/cli/health.rs"))
    parser.add_argument("--json", action="store_true")
    return parser.parse_args()


def normalize_name(value: str) -> str:
    return re.sub(r"[^a-z0-9]+", "", value.lower())


def expected_health_key(module: str) -> str:
    return normalize_name(HEALTH_ALIASES.get(module, module))


def parse_declared_modules(text: str) -> list[str]:
    modules = re.findall(r"pub\(crate\)\s+mod\s+([a-z0-9_]+);", text)
    if not modules:
        raise ValueError("failed to parse declared source modules")
    return sorted(modules)


def parse_health_entries(text: str) -> list[str]:
    match = re.search(
        r"const HEALTH_SOURCES: &\[SourceDescriptor\] = &\[(?P<body>.*?)\n\];",
        text,
        flags=re.DOTALL,
    )
    if match is None:
        raise ValueError("failed to locate HEALTH_SOURCES block")
    entries = re.findall(r'api:\s*"([^"]+)"', match.group("body"))
    if not entries:
        raise ValueError("failed to parse health source entries")
    return entries


def make_payload(sources_dir: Path, sources_mod: Path, health_file: Path) -> dict[str, object]:
    errors: list[str] = []
    try:
        source_files = sorted(
            path.stem
            for path in sources_dir.glob("*.rs")
            if path.name != "mod.rs"
        )
        declared_modules = parse_declared_modules(sources_mod.read_text(encoding="utf-8"))
        health_entries = parse_health_entries(health_file.read_text(encoding="utf-8"))
    except Exception as exc:  # noqa: BLE001
        errors.append(str(exc))
        source_files = []
        declared_modules = []
        health_entries = []

    external_modules = sorted(
        module for module in source_files if module not in HELPER_MODULES
    )
    declared_set = set(declared_modules)
    health_key_to_entry = {normalize_name(entry): entry for entry in health_entries}
    expected_health_keys = {
        expected_health_key(module): module
        for module in external_modules
        if module not in EXEMPT_MODULES
    }

    undeclared_modules = sorted(module for module in external_modules if module not in declared_set)
    missing_health_modules = sorted(
        module
        for module in external_modules
        if module not in EXEMPT_MODULES and expected_health_key(module) not in health_key_to_entry
    )
    orphan_health_entries = sorted(
        entry
        for entry in health_entries
        if normalize_name(entry) not in expected_health_keys
    )

    if errors:
        status = "error"
    elif undeclared_modules or missing_health_modules or orphan_health_entries:
        status = "fail"
    else:
        status = "pass"

    return {
        "status": status,
        "source_files": source_files,
        "declared_modules": declared_modules,
        "health_entries": health_entries,
        "helper_modules": sorted(HELPER_MODULES),
        "exempt_modules": sorted(EXEMPT_MODULES),
        "undeclared_modules": undeclared_modules,
        "missing_health_modules": missing_health_modules,
        "orphan_health_entries": orphan_health_entries,
        "errors": errors,
    }


def main() -> int:
    args = parse_args()
    payload = make_payload(args.sources_dir, args.sources_mod, args.health_file)
    if args.json:
        json.dump(payload, sys.stdout, indent=2)
        sys.stdout.write("\n")
    else:
        print(f"status={payload['status']}")
        print(f"undeclared={','.join(payload['undeclared_modules'])}")
        print(f"missing_health={','.join(payload['missing_health_modules'])}")
        print(f"orphan_health={','.join(payload['orphan_health_entries'])}")
    return 0 if payload["status"] == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
