"""Source readers and parsing helpers for variant structure annotation."""

from __future__ import annotations

import re
import time
import urllib.parse
from typing import Any, Callable

import requests

AA3 = {
    "Ala": "A", "Arg": "R", "Asn": "N", "Asp": "D", "Cys": "C", "Gln": "Q", "Glu": "E", "Gly": "G",
    "His": "H", "Ile": "I", "Leu": "L", "Lys": "K", "Met": "M", "Phe": "F", "Pro": "P", "Ser": "S",
    "Thr": "T", "Trp": "W", "Tyr": "Y", "Val": "V", "Ter": "*", "Sec": "U", "Pyl": "O",
}

HEADERS = {"Accept": "application/json", "User-Agent": "biomcp-spike/variant-structure"}


def now() -> float:
    return time.perf_counter()


def timed(label: str, fn: Callable[[], Any]) -> dict[str, Any]:
    start = now()
    try:
        value = fn()
        return {"label": label, "ok": True, "latency_ms": round((now() - start) * 1000), "value": value}
    except Exception as exc:  # keep experiments moving
        return {"label": label, "ok": False, "latency_ms": round((now() - start) * 1000), "error": str(exc)}


def http_get(url: str, params: dict[str, Any] | None = None) -> Any:
    resp = requests.get(url, params=params, headers=HEADERS, timeout=30)
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
        locations = []
        for prot in row.get("proteins", []) or []:
            for loc in prot.get("entry_protein_locations", []) or []:
                for frag in loc.get("fragments", []) or []:
                    start = frag.get("start")
                    end = frag.get("end")
                    if isinstance(start, int) and isinstance(end, int):
                        locations.append({"start": start, "end": end})
        item = {
            "accession": meta.get("accession"),
            "name": meta.get("name"),
            "type": meta.get("type"),
            "locations": locations,
        }
        domains.append(item)
        if residue is not None and any(loc["start"] <= residue <= loc["end"] for loc in locations):
            overlaps.append(item)
    return {"domain_count": len(domains), "domains_with_locations": sum(bool(d["locations"]) for d in domains), "overlaps": overlaps, "examples": domains[:6]}


def rcsb_coverage(pdb_ids: list[str], accession: str, residue: int | None) -> dict[str, Any]:
    if residue is None or not pdb_ids:
        return {"checked": 0, "covering": []}
    covering = []
    checked = 0
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
    empty = {"source": "cancerhotspots.org", "position_count": None, "same_aa_count": None, "matched_transcript": None}
    if requested is None:
        return {"ok": True, "present": True, "value": empty}
    residue, alt = requested
    for row in rows:
        if str(row.get("residue") or "").strip().upper() != residue:
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
    return {"ok": True, "present": True, "value": empty}
