#!/usr/bin/env -S uv run --no-project
"""Build the reviewed compact corpus and resolve exact character spans."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "work" / "source_documents.json"
OUTPUT = ROOT / "fixtures" / "corpus.json"

# (surface, event type, normalized form, chromosomes/loci)
GOLD: dict[str, list[tuple[str, str, str, list[str]]]] = {
    "30709865": [
        ("uniparental disomy", "free_text_structural_variant", "uniparental disomy", []),
    ],
    "35637217": [
        ("Structural variants", "free_text_structural_variant", "structural variant", []),
        ("structural variants", "free_text_structural_variant", "structural variant", []),
        ("SV", "free_text_structural_variant", "structural variant", []),
        ("t(4;14)", "translocation", "t(4;14)", ["4", "14"]),
        ("t(11;14)", "translocation", "t(11;14)", ["11", "14"]),
        ("chromosomal rearrangements", "free_text_structural_variant", "chromosomal rearrangement", []),
        ("complex rearrangements", "complex_event", "complex rearrangement", []),
        ("chromoplexy", "complex_event", "chromoplexy", []),
        ("Chromothripsis", "complex_event", "chromothripsis", []),
        ("chromothripsis", "complex_event", "chromothripsis", []),
        ("templated insertions", "complex_event", "templated insertion", []),
    ],
    "37449980": [
        ("Copy-Number", "free_text_structural_variant", "copy-number abnormality", ["1"]),
        ("Structural Variants", "free_text_structural_variant", "structural variant", ["1"]),
        ("copy-number abnormalities", "free_text_structural_variant", "copy-number abnormality", ["1"]),
        ("structural variants", "free_text_structural_variant", "structural variant", ["1"]),
        ("CNA", "free_text_structural_variant", "copy-number abnormality", ["1"]),
        ("CNAs", "free_text_structural_variant", "copy-number abnormality", ["1"]),
        ("SV", "free_text_structural_variant", "structural variant", ["1"]),
        ("SVs", "free_text_structural_variant", "structural variant", ["1"]),
        ("gain(1q)", "gain", "gain(1q)", ["1q"]),
        ("focal gains", "gain", "gain", []),
        ("whole-arm gains", "gain", "gain", ["1q"]),
        ("regions of deletion", "deletion", "deletion", []),
        ("of gain", "gain", "gain", []),
        ("chromothripsis", "complex_event", "chromothripsis", []),
        ("templated insertion", "complex_event", "templated insertion", []),
    ],
    "39796213": [
        ("ring chromosome 21", "complex_event", "ring chromosome 21", ["21"]),
        ("r(21)c", "complex_event", "r(21)c", ["21"]),
        ("intrachromosomal amplification of chromosome 21", "amplification", "amp(21)", ["21"]),
        ("iAMP21", "amplification", "amp(21)", ["21"]),
        ("WWOX::PAX5", "free_text_structural_variant", "WWOX::PAX5 fusion", []),
        ("13q12.2 deletion", "deletion", "del(13q12.2)", ["13q12.2"]),
        ("chromothripsis", "complex_event", "chromothripsis", ["21"]),
    ],
    "34885058": [
        ("structural genome variations", "free_text_structural_variant", "structural variant", []),
        ("chromosomal gains", "gain", "gain", []),
        ("losses", "deletion", "copy-number loss", []),
        ("complex genomic rearrangements", "complex_event", "complex genomic rearrangement", []),
        ("chromothripsis", "complex_event", "chromothripsis", []),
        ("templated insertions", "complex_event", "templated insertion", []),
        ("chromoplexy", "complex_event", "chromoplexy", []),
    ],
    "42426366": [
        ("TCF3::HLF", "free_text_structural_variant", "TCF3::HLF fusion", []),
        ("t(17;19)(q22;p13)", "translocation", "t(17;19)(q22;p13)", ["17q22", "19p13"]),
        ("high hyperdiploid karyotype", "ploidy_state", "high hyperdiploidy", []),
        ("duplication of 1q", "gain", "gain(1q)", ["1q"]),
        ("high hyperdiploidy", "ploidy_state", "high hyperdiploidy", []),
    ],
    "41935330": [
        ("BCR::ABL1", "free_text_structural_variant", "BCR::ABL1 fusion", []),
        ("pericentric inversion of der(9)", "inversion", "inv(der(9))", ["9"]),
        ("codeletion", "deletion", "codeletion", []),
        ("t(9;22)(q34;q11.2)", "translocation", "t(9;22)(q34;q11.2)", ["9q34", "22q11.2"]),
        ("primary rearrangement", "free_text_structural_variant", "chromosomal rearrangement", ["9", "22"]),
        ("Pericentric inversion of chromosome 9", "inversion", "inv(9)", ["9"]),
        ("pericentric inversion involving the derivative chromosome 9", "inversion", "inv(der(9))", ["9"]),
        ("pericentric inversion of the derivative chromosome 9", "inversion", "inv(der(9))", ["9"]),
        ("complex rearrangements", "complex_event", "complex rearrangement", []),
        ("inv(9)", "inversion", "inv(9)", ["9"]),
    ],
    "42379467": [
        ("del(17p)", "deletion", "del(17p)", ["17p"]),
    ],
    "42366058": [
        ("PML::RARA", "free_text_structural_variant", "PML::RARA fusion", []),
        ("BCR::ABL1", "free_text_structural_variant", "BCR::ABL1 fusion", []),
        ("t(15;17)", "translocation", "t(15;17)", ["15", "17"]),
        ("t(9;22)", "translocation", "t(9;22)", ["9", "22"]),
    ],
}

RELATIONSHIPS = {
    "42426366": [
        {
            "event_surface": "t(17;19)(q22;p13)", "relation": "produces_fusion",
            "genes": ["TCF3", "HLF"],
            "evidence": "The t(17;19)(q22;p13) translocation, generating the TCF3::HLF fusion oncogene",
        }
    ],
    "41935330": [
        {
            "event_surface": "t(9;22)(q34;q11.2)", "relation": "produces_fusion",
            "genes": ["BCR", "ABL1"],
            "evidence": "the BCR::ABL1 fusion gene resulting from the t(9;22)(q34;q11.2) translocation",
        },
        {
            "event_surface": "pericentric inversion involving the derivative chromosome 9",
            "relation": "co_deletes",
            "genes": ["BCR", "ABL1"],
            "evidence": "an acquired pericentric inversion involving the derivative chromosome 9, associated with codeletion of BCR and ABL1 sequences",
        },
    ],
    "39796213": [
        {
            "event_surface": "WWOX::PAX5", "relation": "fusion_partners",
            "genes": ["WWOX", "PAX5"],
            "evidence": "the novel WWOX::PAX5",
        }
    ],
    "42366058": [
        {
            "event_surface": "PML::RARA", "relation": "fusion_partners",
            "genes": ["PML", "RARA"],
            "evidence": "Independent PML::RARA-positive acute promyelocytic leukemia",
        },
        {
            "event_surface": "BCR::ABL1", "relation": "fusion_partners",
            "genes": ["BCR", "ABL1"],
            "evidence": "during BCR::ABL1-positive chronic myeloid leukemia",
        },
    ],
}


def occurrences(text: str, surface: str) -> list[int]:
    starts, offset = [], 0
    while (found := text.find(surface, offset)) >= 0:
        end = found + len(surface)
        left_ok = not surface[0].isalnum() or found == 0 or not text[found - 1].isalnum()
        right_ok = not surface[-1].isalnum() or end == len(text) or not text[end].isalnum()
        if left_ok and right_ok:
            starts.append(found)
        offset = end
    return starts


def main() -> None:
    source = json.loads(SOURCE.read_text())
    corpus = {"schema_version": 1, "offset_unit": "unicode_codepoints", "documents": []}
    for record in source:
        pmid = record["pmid"]
        text = f"{record['title']}\n{record['abstract']}"
        gold = []
        for surface, kind, normalized, loci in GOLD.get(pmid, []):
            starts = occurrences(text, surface)
            if not starts:
                raise ValueError(f"{pmid}: missing gold surface {surface!r}")
            for start in starts:
                gold.append({
                    "id": f"{pmid}-e{len(gold) + 1}", "start": start,
                    "end": start + len(surface), "text": surface, "normalized": normalized,
                    "event_type": kind, "chromosome_or_locus": loci,
                })
        relationships = []
        for relation in RELATIONSHIPS.get(pmid, []):
            evidence = relation["evidence"]
            starts = occurrences(text, evidence)
            if len(starts) != 1:
                raise ValueError(f"{pmid}: relationship evidence not unique: {evidence!r}")
            evidence_end = starts[0] + len(evidence)
            event_ids = [
                event["id"] for event in gold
                if event["text"] == relation["event_surface"]
                and starts[0] <= event["start"] < evidence_end
            ]
            if not event_ids:
                raise ValueError(f"{pmid}: relation event missing: {relation['event_surface']}")
            relationships.append({
                "event_id": event_ids[0], "relation": relation["relation"],
                "genes": relation["genes"], "evidence_start": starts[0],
                "evidence_end": evidence_end, "evidence_text": evidence,
                "provenance": {"source": "PubMed title/abstract", "pmid": pmid},
            })
        gold.sort(key=lambda event: (event["start"], event["end"]))
        corpus["documents"].append({
            "pmid": pmid, "label": "positive" if gold else "control",
            "title": record["title"], "text": text, "gold_events": gold,
            "gold_gene_relationships": relationships,
        })
    OUTPUT.parent.mkdir(exist_ok=True)
    OUTPUT.write_text(json.dumps(corpus, indent=2, ensure_ascii=False) + "\n")
    positives = sum(bool(doc["gold_events"]) for doc in corpus["documents"])
    events = sum(len(doc["gold_events"]) for doc in corpus["documents"])
    print(f"wrote {len(corpus['documents'])} documents, {positives} positives, {events} events")


if __name__ == "__main__":
    main()
