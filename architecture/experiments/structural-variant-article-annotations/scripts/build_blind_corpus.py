#!/usr/bin/env -S uv run --no-project
"""Build the held-out full-scale corpus from a fixed PubMed source snapshot.

The PMID selection and gold patterns are independent of the candidate parser. Candidate
papers were selected by broad PubMed queries, then the minimal event mentions were
reviewed by category. Generated NCBI payloads remain under ignored work/.
"""

from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "work" / "blind-candidates.json"
OUTPUT = ROOT / "fixtures" / "blind_corpus.json"

SELECTION = {
    "translocation": ["42401675", "42205458", "42026179", "42371798", "42303812"],
    "deletion": ["41921592", "42311136", "41773871", "42079105", "42001523"],
    "gain": ["42216953", "41863841", "41733011", "40257113", "40496546"],
    "amplification": ["42193341", "41926541", "41959452", "40864971", "40540066"],
    "inversion": ["42394944", "42354908", "41547329", "42095083", "41992736"],
    "complex_event": ["42326738", "42442953", "41943674", "41954631", "42302064"],
    "ploidy_state": ["42068673", "42342436", "42382092", "42346857", "41996759"],
    "free_text_structural_variant": [
        "42203030",
        "41869909",
        "42436115",
        "42423801",
        "42421598",
    ],
    "control_pcr_amplification": [
        "42155174",
        "42434337",
        "42119654",
        "42434792",
        "42404976",
    ],
    "control_protein_translocation": [
        "41649879",
        "42184477",
        "42307419",
        "42212830",
        "42177619",
    ],
    "control_nuclear_translocation": [
        "42281240",
        "42100892",
        "42021123",
        "42165632",
        "42144020",
    ],
    "control_molecular_inversion_probe": [
        "41947040",
        "41483127",
        "42124593",
        "41379859",
        "41291966",
    ],
}

# Adjudication vocabulary: broad semantic mentions used to mark gold, deliberately
# separate from the candidate parser's notation/context rules.
RELATIONSHIPS = {
    "42371798": {
        "event_surface": "t(4;14)",
        "relation": "drives_overexpression",
        "genes": ["NSD2"],
        "evidence": "The t(4;14) chromosomal translocation drives overexpression of the histone methyltransferase NSD2",
    }
}

GOLD_PATTERNS: list[tuple[str, str]] = [
    (r"(?i)(?<![A-Za-z])(?:inter-)?chromosomal translocations?", "translocation"),
    (
        r"(?i)(?<![A-Za-z])t\(\d{1,2};\d{1,2}\)(?:\([pq][\d.]+;[pq][\d.]+\))?",
        "translocation",
    ),
    (r"(?i)(?<![A-Za-z])chromosomal deletions?", "deletion"),
    (r"(?i)(?<![A-Za-z0-9])(?:chr)?\d+[pq][\d.]* deletions?", "deletion"),
    (r"(?i)(?<![A-Za-z])(?:[A-Z]-)?chromosome gains?", "gain"),
    (r"(?i)(?<![A-Za-z])chromosomal gains?", "gain"),
    (r"(?i)(?<![A-Za-z])chromosomal amplifications?", "amplification"),
    (r"(?i)(?<![A-Za-z])chromosomal inversions?", "inversion"),
    (
        r"(?i)(?<![A-Za-z])(?:chromothripsis|chromoplexy|complex genomic rearrangements?)(?![A-Za-z])",
        "complex_event",
    ),
    (
        r"(?i)(?<![A-Za-z])(?:high )?hyperdiploid(?:y| karyotype)?(?![A-Za-z])",
        "ploidy_state",
    ),
    (
        r"(?i)(?<![A-Za-z])structural variants?(?![A-Za-z])",
        "free_text_structural_variant",
    ),
    (
        r"(?i)(?<![A-Za-z])chromosomal rearrangements?(?![A-Za-z])",
        "free_text_structural_variant",
    ),
    (
        r"(?<![A-Za-z])[A-Z][A-Z0-9]*(?:-[A-Z0-9]+)*::[A-Z][A-Z0-9]*(?:-[A-Z0-9]+)*(?![A-Za-z])",
        "free_text_structural_variant",
    ),
    (r"(?<![A-Za-z])(?:CNA|CNAs|SV|SVs)(?![A-Za-z])", "free_text_structural_variant"),
]


def loci_for(surface: str) -> list[str]:
    if surface.lower().startswith("y-chromosome"):
        return ["Y"]
    notation = re.fullmatch(
        r"(?i)t\((\d{1,2});(\d{1,2})\)(?:\(([pq][\d.]+);([pq][\d.]+)\))?", surface
    )
    if not notation:
        return []
    first, second, arm1, arm2 = notation.groups()
    return [first + (arm1 or ""), second + (arm2 or "")]


def normalized(surface: str, event_type: str) -> str:
    value = surface.lower()
    if event_type == "free_text_structural_variant":
        return "structural variant"
    if event_type == "ploidy_state":
        return "high hyperdiploidy" if value.startswith("high ") else "hyperdiploidy"
    plural_words = (
        "translocations",
        "deletions",
        "gains",
        "amplifications",
        "inversions",
        "rearrangements",
    )
    return value[:-1] if value.endswith(plural_words) else value


def main() -> None:
    records = {row["pmid"]: row for row in json.loads(SOURCE.read_text())}
    documents: list[dict[str, Any]] = []
    seen: set[str] = set()
    for category, pmids in SELECTION.items():
        is_control = category.startswith("control_")
        for pmid in pmids:
            if pmid in seen:
                raise ValueError(f"duplicate PMID in selection: {pmid}")
            seen.add(pmid)
            source = records[pmid]
            text = f"{source['title']}\n{source['abstract']}"
            events = []
            if not is_control:
                matches: dict[tuple[int, int, str], re.Match[str]] = {}
                for expression, event_type in GOLD_PATTERNS:
                    for match in re.finditer(expression, text):
                        matches[(match.start(), match.end(), event_type)] = match
                for number, ((start, end, event_type), match) in enumerate(
                    sorted(matches.items()), 1
                ):
                    surface = match.group()
                    events.append(
                        {
                            "id": f"{pmid}-e{number}",
                            "start": start,
                            "end": end,
                            "text": surface,
                            "normalized": normalized(surface, event_type),
                            "event_type": event_type,
                            "chromosome_or_locus": loci_for(surface),
                            "provenance": {
                                "source": "PubMed title/abstract",
                                "pmid": pmid,
                                "passage": "title"
                                if end <= len(source["title"])
                                else "abstract",
                            },
                        }
                    )
                if not events:
                    raise ValueError(
                        f"positive selection has no adjudicated events: {pmid}"
                    )
            relationships = []
            if pmid in RELATIONSHIPS:
                relation = RELATIONSHIPS[pmid]
                evidence_start = text.index(relation["evidence"])
                evidence_end = evidence_start + len(relation["evidence"])
                event = next(
                    row
                    for row in events
                    if row["text"] == relation["event_surface"]
                    and evidence_start <= row["start"] < evidence_end
                )
                relationships.append(
                    {
                        "event_id": event["id"],
                        "relation": relation["relation"],
                        "genes": relation["genes"],
                        "evidence_start": evidence_start,
                        "evidence_end": evidence_end,
                        "evidence_text": relation["evidence"],
                        "provenance": {"source": "PubMed title/abstract", "pmid": pmid},
                    }
                )
            documents.append(
                {
                    "pmid": pmid,
                    "selection_category": category,
                    "label": "control" if is_control else "positive",
                    "title": source["title"],
                    "text": text,
                    "gold_events": events,
                    "gold_gene_relationships": relationships,
                }
            )
    corpus = {
        "schema_version": 1,
        "split": "blind_full_scale",
        "offset_unit": "unicode_codepoints",
        "selection": "fixed disjoint PubMed query sample; 5 papers/event family and 5 papers/control trap",
        "documents": documents,
    }
    OUTPUT.write_text(json.dumps(corpus, indent=2, ensure_ascii=False) + "\n")
    counts: dict[str, int] = {}
    for doc in documents:
        for event in doc["gold_events"]:
            kind = event["event_type"]
            counts[kind] = counts.get(kind, 0) + 1
    print(
        f"wrote {len(documents)} documents, "
        f"{sum(doc['label'] == 'positive' for doc in documents)} positives, "
        f"{sum(len(doc['gold_events']) for doc in documents)} events: {counts}"
    )


if __name__ == "__main__":
    main()
