#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["requests"]
# ///
"""Small-scale spike measurements for variant -> protein structure annotation.

This is intentionally exploratory. It does not share production code; it records
source reachability, latency, and whether the minimal fields needed for a future
BioMCP contract are available for BRAF V600E, TP53 R175H, and ROS1 G2032R.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import time
import urllib.parse
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import requests

ROOT = Path(__file__).resolve().parents[4]
DEFAULT_BIOMCP_BIN = ROOT / "target" / "debug" / "biomcp"
OUT_DIR = ROOT / "architecture" / "experiments" / "variant-structure-annotation" / "results"

VARIANTS = [
    {"gene": "BRAF", "change": "V600E", "label": "BRAF V600E", "accession": "P15056"},
    {"gene": "TP53", "change": "R175H", "label": "TP53 R175H", "accession": "P04637"},
    {"gene": "ROS1", "change": "G2032R", "label": "ROS1 G2032R", "accession": "P08922"},
]

AA3 = {
    "Ala": "A", "Arg": "R", "Asn": "N", "Asp": "D", "Cys": "C", "Gln": "Q", "Glu": "E", "Gly": "G",
    "His": "H", "Ile": "I", "Leu": "L", "Lys": "K", "Met": "M", "Phe": "F", "Pro": "P", "Ser": "S",
    "Thr": "T", "Trp": "W", "Tyr": "Y", "Val": "V", "Ter": "*", "Sec": "U", "Pyl": "O",
}


def now() -> float:
    return time.perf_counter()


def timed(label: str, fn):
    start = now()
    try:
        value = fn()
        return {"label": label, "ok": True, "latency_ms": round((now() - start) * 1000), "value": value}
    except Exception as exc:  # keep experiments moving
        return {"label": label, "ok": False, "latency_ms": round((now() - start) * 1000), "error": str(exc)}


def http_get(url: str, params: dict[str, Any] | None = None) -> Any:
    resp = requests.get(url, params=params, headers={"Accept": "application/json", "User-Agent": "biomcp-spike/variant-structure"}, timeout=30)
    resp.raise_for_status()
    return resp.json()


def first_scalar(value: Any) -> str | None:
    if isinstance(value, list):
        return first_scalar(value[0]) if value else None
    if isinstance(value, str) and value.strip():
        return value.strip()
    return None


def all_scalars(value: Any) -> list[str]:
    if value is None:
        return []
    if isinstance(value, list):
        out: list[str] = []
        for item in value:
            out.extend(all_scalars(item))
        return out
    if isinstance(value, str) and value.strip():
        return [value.strip()]
    return []


def parse_hgvsp_position(hgvsp: str | None) -> int | None:
    if not hgvsp:
        return None
    # Handles p.V600E, p.Val600Glu, NP_...:p.Val600Glu
    token = hgvsp.split(":")[-1].removeprefix("p.")
    m = re.search(r"(?:[A-Z][a-z]{2}|[A-Z]|\*)?(\d+)", token)
    return int(m.group(1)) if m else None


def normalize_change(change: str) -> str:
    change = change.split(":")[-1].removeprefix("p.")
    for aa3, aa1 in AA3.items():
        change = change.replace(aa3, aa1)
    return change


def requested_position_from_hgvsp(hgvsp_values: list[str], requested_change: str) -> int | None:
    requested = normalize_change(requested_change)
    for hgvsp in hgvsp_values:
        if normalize_change(hgvsp) == requested:
            return parse_hgvsp_position(hgvsp)
    return parse_hgvsp_position(requested_change)


def myvariant_hit(gene: str, change: str) -> dict[str, Any]:
    q = f'dbnsfp.genename:{gene} AND dbnsfp.hgvsp:"p.{change}"'
    fields = "_id,dbnsfp.genename,dbnsfp.hgvsp,dbnsfp.hgvsc,dbsnp.rsid,clinvar.variant_id,clinvar.rcv"
    data = http_get("https://myvariant.info/v1/query", {"q": q, "fields": fields, "size": 5})
    hits = data.get("hits", [])
    if not hits:
        return {"query": q, "hit_count": 0}
    hit = hits[0]
    dbnsfp = hit.get("dbnsfp") or {}
    hgvsp_values = all_scalars(dbnsfp.get("hgvsp"))
    hgvsc_values = all_scalars(dbnsfp.get("hgvsc"))
    positions = sorted({p for p in (parse_hgvsp_position(v) for v in hgvsp_values) if p is not None})
    requested_position = requested_position_from_hgvsp(hgvsp_values, change)
    requested_matches = [v for v in hgvsp_values if normalize_change(v) == normalize_change(change)]
    return {
        "query": q,
        "hit_count": len(hits),
        "id": hit.get("_id"),
        "rsid": first_scalar((hit.get("dbsnp") or {}).get("rsid")),
        "gene_values": sorted(set(all_scalars(dbnsfp.get("genename")))),
        "hgvsp_values": hgvsp_values[:12],
        "hgvsc_count": len(hgvsc_values),
        "hgvsc_examples": hgvsc_values[:8],
        "protein_positions": positions,
        "requested_position": requested_position,
        "requested_hgvsp_matches": requested_matches[:8],
        "position_consistent": len(positions) == 1,
        "clinvar_variant_id": (hit.get("clinvar") or {}).get("variant_id"),
    }


def uniprot_record(accession: str) -> dict[str, Any]:
    return http_get(f"https://rest.uniprot.org/uniprotkb/{accession}.json")


def uniprot_summary(record: dict[str, Any]) -> dict[str, Any]:
    refs = record.get("uniProtKBCrossReferences") or []
    pdb = []
    af = []
    for ref in refs:
        db = ref.get("database")
        rid = ref.get("id")
        if not rid:
            continue
        props = {p.get("key"): p.get("value") for p in ref.get("properties", []) if p.get("key")}
        if db == "PDB":
            pdb.append({"id": rid, "method": props.get("Method"), "resolution": props.get("Resolution"), "chains": props.get("Chains")})
        elif db == "AlphaFoldDB":
            af.append({"id": rid})
    return {
        "accession": record.get("primaryAccession"),
        "entry": record.get("uniProtkbId"),
        "length": ((record.get("sequence") or {}).get("length")),
        "pdb_count": len(pdb),
        "pdb_examples": pdb[:5],
        "alphafold_ids": [x["id"] for x in af],
    }


def interpro_domains(accession: str, residue: int | None) -> dict[str, Any]:
    data = http_get(f"https://www.ebi.ac.uk/interpro/api/entry/interpro/protein/uniprot/{accession}/", {"page_size": 25})
    domains = []
    overlaps = []
    for row in data.get("results", []):
        meta = row.get("metadata") or {}
        entry_acc = meta.get("accession")
        name = meta.get("name")
        typ = meta.get("type")
        locations = []
        for prot in row.get("proteins", []) or []:
            for loc in prot.get("entry_protein_locations", []) or []:
                for frag in loc.get("fragments", []) or []:
                    start = frag.get("start")
                    end = frag.get("end")
                    if isinstance(start, int) and isinstance(end, int):
                        locations.append({"start": start, "end": end})
        item = {"accession": entry_acc, "name": name, "type": typ, "locations": locations}
        domains.append(item)
        if residue is not None and any(loc["start"] <= residue <= loc["end"] for loc in locations):
            overlaps.append(item)
    return {"domain_count": len(domains), "domains_with_locations": sum(bool(d["locations"]) for d in domains), "overlaps": overlaps, "examples": domains[:6]}


def rcsb_coverage(pdb_ids: list[str], accession: str, residue: int | None) -> dict[str, Any]:
    if residue is None or not pdb_ids:
        return {"checked": 0, "covering": []}
    covering = []
    checked = 0
    # RCSB mapping API is a useful probe, but not needed for the minimal contract.
    for pdb_id in pdb_ids[:5]:
        checked += 1
        url = f"https://data.rcsb.org/rest/v1/core/polymer_entity/{pdb_id}/1"
        try:
            data = http_get(url)
        except Exception as exc:
            covering.append({"pdb_id": pdb_id, "ok": False, "error": str(exc)})
            continue
        refs = data.get("rcsb_polymer_entity_container_identifiers", {}).get("reference_sequence_identifiers") or []
        auth_ranges = data.get("rcsb_polymer_entity_container_identifiers", {}).get("auth_asym_ids") or []
        matches_accession = any(ref.get("database_accession") == accession for ref in refs)
        # Core polymer_entity endpoint does not reliably expose residue-level UniProt ranges in a compact stable field.
        covering.append({"pdb_id": pdb_id, "ok": True, "matches_accession": matches_accession, "chain_ids": auth_ranges})
    return {"checked": checked, "covering_probe": covering}


def residue_and_alt(change: str) -> tuple[str, str] | None:
    change = normalize_change(change)
    alt = change[-1:] if change else ""
    residue = change[:-1]
    if not alt or not residue or not any(ch.isdigit() for ch in residue):
        return None
    return residue.upper(), alt.upper()


def cancerhotspots_probe(gene: str, change: str) -> dict[str, Any]:
    rows = http_get(
        "https://www.cancerhotspots.org/api/hotspots/single/byGene/"
        + urllib.parse.quote(gene.strip(), safe="")
    )
    requested = residue_and_alt(change)
    if requested is None:
        return {"ok": True, "present": True, "value": {"source": "cancerhotspots.org", "position_count": None, "same_aa_count": None, "matched_transcript": None}}
    residue, alt = requested
    for row in rows:
        row_residue = str(row.get("residue") or "").strip().upper()
        if row_residue != residue:
            continue
        aa_counts = row.get("variantAminoAcid") or {}
        if alt not in aa_counts:
            continue
        return {
            "ok": True,
            "present": True,
            "value": {
                "source": "cancerhotspots.org",
                "position_count": row.get("tumorCount"),
                "same_aa_count": aa_counts.get(alt),
                "matched_transcript": row.get("transcriptId"),
            },
        }
    return {"ok": True, "present": True, "value": {"source": "cancerhotspots.org", "position_count": None, "same_aa_count": None, "matched_transcript": None}}


def run_existing_cli() -> dict[str, Any]:
    bin_path = Path(os.environ.get("BIOMCP_BIN", DEFAULT_BIOMCP_BIN))
    rows = []
    for v in VARIANTS:
        entry = {"variant": v["label"], "accession": v["accession"]}
        for name, cmd in {
            "variant_all": [str(bin_path), "--json", "--no-cache", "get", "variant", v["label"], "all"],
            "protein_structures": [str(bin_path), "--json", "--no-cache", "get", "protein", v["accession"], "structures"],
            "protein_domains": [str(bin_path), "--json", "--no-cache", "get", "protein", v["accession"], "domains"],
        }.items():
            def call(cmd=cmd):
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


def run_direct_join(with_rcsb: bool = False) -> dict[str, Any]:
    rows = []
    for v in VARIANTS:
        row_start = now()
        row: dict[str, Any] = {"variant": v["label"], "gene": v["gene"], "accession": v["accession"]}
        with ThreadPoolExecutor(max_workers=3) as pool:
            mv_fut = pool.submit(timed, "myvariant", lambda v=v: myvariant_hit(v["gene"], v["change"]))
            up_fut = pool.submit(timed, "uniprot", lambda v=v: uniprot_summary(uniprot_record(v["accession"])))
            ch_fut = pool.submit(timed, "cancerhotspots", lambda v=v: cancerhotspots_probe(v["gene"], v["change"]))
            mv = mv_fut.result()
            row["myvariant"] = mv
            residue = None
            if mv["ok"]:
                residue = mv["value"].get("requested_position")
            row["residue"] = residue
            ip_fut = pool.submit(timed, "interpro", lambda v=v, residue=residue: interpro_domains(v["accession"], residue))
            up = up_fut.result()
            row["uniprot"] = up
            row["cancerhotspots"] = ch_fut.result()
            ip = ip_fut.result()
            row["interpro"] = ip
        if with_rcsb:
            pdb_ids = []
            if up["ok"]:
                pdb_ids = [p["id"] for p in up["value"].get("pdb_examples", [])]
            row["structure_reference_probe"] = timed("rcsb_coverage", lambda pdb_ids=pdb_ids, residue=residue, v=v: {
                "alphafold_url": f"https://alphafold.ebi.ac.uk/entry/{v['accession']}",
                "rcsb_search_url": "https://www.rcsb.org/search?query=" + urllib.parse.quote(v["accession"]),
                "rcsb_probe": rcsb_coverage(pdb_ids, v["accession"], residue),
            })
        row["total_latency_ms"] = round((now() - row_start) * 1000)
        rows.append(row)
    return {"approach": "direct_source_join" + ("_with_structure_links" if with_rcsb else ""), "variants": rows}


def summarize(result: dict[str, Any]) -> dict[str, Any]:
    summary = {"approach": result["approach"], "n": len(result["variants"]), "variants": []}
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
                "total_latency_ms": row.get("total_latency_ms") or sum(v.get("latency_ms", 0) for k, v in row.items() if isinstance(v, dict) and "latency_ms" in v),
            })
    return summary


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--approach", choices=["cli", "direct", "links", "all"], default="all")
    args = parser.parse_args()
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    runs = []
    if args.approach in {"cli", "all"}:
        runs.append(run_existing_cli())
    if args.approach in {"direct", "all"}:
        runs.append(run_direct_join(False))
    if args.approach in {"links", "all"}:
        runs.append(run_direct_join(True))
    for result in runs:
        name = result["approach"]
        (OUT_DIR / f"{name}.json").write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
        (OUT_DIR / f"{name}_summary.json").write_text(json.dumps(summarize(result), indent=2, sort_keys=True) + "\n")
        print(json.dumps(summarize(result), indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
