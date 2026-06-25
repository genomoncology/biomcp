"""Reusable orchestration for the variant -> protein-structure spike."""

from __future__ import annotations

import json
import os
import subprocess
import urllib.parse
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from typing import Any, Iterable

from .sources import (
    cancerhotspots_probe,
    interpro_domains,
    myvariant_hit,
    now,
    rcsb_coverage,
    timed,
    uniprot_record,
    uniprot_summary,
)
from .types import VariantSpec

ROOT = Path(__file__).resolve().parents[4]
DEFAULT_BIOMCP_BIN = ROOT / "target" / "debug" / "biomcp"
OUT_DIR = ROOT / "architecture" / "experiments" / "variant-structure-annotation" / "results"

DEFAULT_VARIANTS = [
    VariantSpec(gene="BRAF", change="V600E", label="BRAF V600E", accession="P15056"),
    VariantSpec(gene="TP53", change="R175H", label="TP53 R175H", accession="P04637"),
    VariantSpec(gene="ROS1", change="G2032R", label="ROS1 G2032R", accession="P08922"),
]


def _variants_or_default(variants: Iterable[VariantSpec] | None) -> list[VariantSpec]:
    return list(variants) if variants is not None else list(DEFAULT_VARIANTS)


def run_existing_cli(variants: Iterable[VariantSpec] | None = None, biomcp_bin: Path | str | None = None) -> dict[str, Any]:
    bin_path = Path(biomcp_bin or os.environ.get("BIOMCP_BIN", DEFAULT_BIOMCP_BIN))
    rows = []
    for v in _variants_or_default(variants):
        entry = {"variant": v.label, "accession": v.accession}
        for name, cmd in {
            "variant_all": [str(bin_path), "--json", "--no-cache", "get", "variant", v.label, "all"],
            "protein_structures": [str(bin_path), "--json", "--no-cache", "get", "protein", v.accession, "structures"],
            "protein_domains": [str(bin_path), "--json", "--no-cache", "get", "protein", v.accession, "domains"],
        }.items():
            def call(cmd: list[str] = cmd) -> dict[str, Any]:
                proc = subprocess.run(cmd, cwd=ROOT, text=True, capture_output=True, timeout=180)
                if proc.returncode != 0:
                    raise RuntimeError(proc.stderr[-600:] or proc.stdout[-600:])
                data = json.loads(proc.stdout)
                return {
                    "keys": sorted(data.keys()),
                    "has_hgvsp": bool(data.get("hgvs_p")),
                    "has_cancerhotspots": bool(data.get("cancerhotspots")),
                    "structure_count": data.get("structure_count"),
                    "structures_len": len(data.get("structures") or []),
                    "domains_len": len(data.get("domains") or []),
                    "domain_has_locations": any("start" in d or "locations" in d for d in (data.get("domains") or [])),
                }
            entry[name] = timed(name, call)
        rows.append(entry)
    return {"approach": "existing_cli_composition", "variants": rows}


def run_direct_join(variants: Iterable[VariantSpec] | None = None, with_rcsb: bool = False) -> dict[str, Any]:
    rows = []
    for v in _variants_or_default(variants):
        row_start = now()
        row: dict[str, Any] = {"variant": v.label, "gene": v.gene, "accession": v.accession}
        with ThreadPoolExecutor(max_workers=3) as pool:
            mv_fut = pool.submit(timed, "myvariant", lambda v=v: myvariant_hit(v.gene, v.change))
            up_fut = pool.submit(timed, "uniprot", lambda v=v: uniprot_summary(uniprot_record(v.accession)))
            ch_fut = pool.submit(timed, "cancerhotspots", lambda v=v: cancerhotspots_probe(v.gene, v.change))
            mv = mv_fut.result()
            row["myvariant"] = mv
            residue = mv["value"].get("requested_position") if mv["ok"] else None
            row["residue"] = residue
            ip_fut = pool.submit(timed, "interpro", lambda v=v, residue=residue: interpro_domains(v.accession, residue))
            up = up_fut.result()
            row["uniprot"] = up
            row["cancerhotspots"] = ch_fut.result()
            row["interpro"] = ip_fut.result()
        if with_rcsb:
            pdb_ids = []
            if up["ok"]:
                pdb_ids = [p["id"] for p in up["value"].get("pdb_examples", [])]
            row["structure_reference_probe"] = timed("rcsb_coverage", lambda pdb_ids=pdb_ids, residue=residue, v=v: {
                "alphafold_url": f"https://alphafold.ebi.ac.uk/entry/{v.accession}",
                "rcsb_search_url": "https://www.rcsb.org/search?query=" + urllib.parse.quote(v.accession),
                "rcsb_probe": rcsb_coverage(pdb_ids, v.accession, residue),
            })
        row["total_latency_ms"] = round((now() - row_start) * 1000)
        rows.append(row)
    return {"approach": "direct_source_join" + ("_with_structure_links" if with_rcsb else ""), "variants": rows}


def summarize(result: dict[str, Any]) -> dict[str, Any]:
    summary: dict[str, Any] = {"approach": result["approach"], "n": len(result["variants"]), "variants": []}
    for row in result["variants"]:
        if result["approach"] == "existing_cli_composition":
            summary["variants"].append({
                "variant": row["variant"],
                "variant_ok": row["variant_all"]["ok"],
                "structures_ok": row["protein_structures"]["ok"],
                "domains_ok": row["protein_domains"]["ok"],
                "domain_locations_exposed": row["protein_domains"].get("value", {}).get("domain_has_locations"),
                "total_latency_ms": sum(row[k]["latency_ms"] for k in ["variant_all", "protein_structures", "protein_domains"]),
            })
        else:
            ip = row.get("interpro", {})
            up = row.get("uniprot", {})
            mv = row.get("myvariant", {})
            summary["variants"].append({
                "variant": row["variant"],
                "residue": row.get("residue"),
                "myvariant_ok": mv.get("ok"),
                "position_consistent": (mv.get("value") or {}).get("position_consistent"),
                "uniprot_ok": up.get("ok"),
                "pdb_count": (up.get("value") or {}).get("pdb_count"),
                "alphafold_ids": (up.get("value") or {}).get("alphafold_ids"),
                "interpro_ok": ip.get("ok"),
                "overlap_count": len((ip.get("value") or {}).get("overlaps") or []),
                "overlap_names": [d.get("name") for d in ((ip.get("value") or {}).get("overlaps") or [])],
                "cancerhotspots_present": ((row.get("cancerhotspots", {}).get("value") or {}).get("present")),
                "total_latency_ms": row.get("total_latency_ms") or sum(v.get("latency_ms", 0) for v in row.values() if isinstance(v, dict) and "latency_ms" in v),
            })
    return summary


def write_result(result: dict[str, Any], out_dir: Path = OUT_DIR) -> dict[str, Any]:
    out_dir.mkdir(parents=True, exist_ok=True)
    name = result["approach"]
    summary = summarize(result)
    (out_dir / f"{name}.json").write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    (out_dir / f"{name}_summary.json").write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n")
    return summary
