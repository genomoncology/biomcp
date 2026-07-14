"""Bounded structural-event detection and occurrence annotation.

This experimental parser normalizes syntax only. It contains no PMIDs, diseases,
or event-to-gene mappings and never infers a gene relationship from notation.
"""

from __future__ import annotations

import json
import re
from collections.abc import Iterable, Iterator
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from .types import (
        CopyNumberDirection,
        Detection,
        Document,
        DocumentAnnotations,
        EventType,
        StructuralEvent,
    )

# Generic syntax/context only. Keep independent scans: overlapping occurrences are
# part of the measured exact-span contract and combined alternations lost matches.
PATTERNS: list[tuple[str, EventType]] = [
    (r"\bt\(\d{1,2};\d{1,2}\)(?:\([pq][\d.]+;[pq][\d.]+\))?", "translocation"),
    (r"\bdel\([^)]+\)", "deletion"),
    (r"\binv\([^)]+\)", "inversion"),
    (r"\bgain\([^)]+\)", "gain"),
    (r"\bamp\([^)]+\)", "amplification"),
    (r"\br\(\d{1,2}\)c\b", "complex_event"),
    (r"\biAMP\d+\b", "amplification"),
    (
        r"\b[A-Z][A-Z0-9]*(?:-[A-Z0-9]+)*::[A-Z][A-Z0-9]*(?:-[A-Z0-9]+)*\b",
        "free_text_structural_variant",
    ),
    (r"\b[Ss]tructural [Vv]ariants?\b", "free_text_structural_variant"),
    (r"\bstructural genome variations?\b", "free_text_structural_variant"),
    (r"\bchromosomal rearrangements?\b", "free_text_structural_variant"),
    (r"\bprimary rearrangement\b", "free_text_structural_variant"),
    (r"\bcopy-number abnormalities\b", "free_text_structural_variant"),
    (r"\bCopy-Number(?= and Structural Variants)", "free_text_structural_variant"),
    (r"\b(?:CNA|CNAs|SV|SVs)\b", "free_text_structural_variant"),
    (
        r"\b(?:complex (?:genomic )?rearrangements?|[Cc]hromoplexy|[Cc]hromothripsis|templated insertions?)\b",
        "complex_event",
    ),
    (r"\bring chromosome \d{1,2}\b", "complex_event"),
    (
        r"\b[Pp]ericentric inversion (?:of|involving) (?:the )?(?:der\(\d{1,2}\)|(?:derivative )?chromosome \d{1,2})",
        "inversion",
    ),
    (
        r"\b(?:codeletion|(?:chr)?\d+[pq][\d.]* deletions?|regions of deletion)\b",
        "deletion",
    ),
    (
        r"\b(?:chromosomal gains|focal gains|whole-arm gains|duplication of \d+[pq]?|of gain)\b",
        "gain",
    ),
    (r"\bintrachromosomal amplification of chromosome \d{1,2}\b", "amplification"),
    (
        r"\b(?:high )?hyperdiploid(?: karyotype)?\b|\bhigh hyperdiploidy\b",
        "ploidy_state",
    ),
    (r"\buniparental disomy\b", "free_text_structural_variant"),
    (r"\b(?:complex )?genomic rearrangements?\b", "complex_event"),
    (r"(?<=chromosomal gains and )losses\b", "deletion"),
    (r"(?i)(?<![A-Za-z])(?:inter-)?chromosomal translocations?", "translocation"),
    (r"(?i)(?<![A-Za-z])chromosomal deletions?", "deletion"),
    (r"(?i)(?<![A-Za-z])(?:[A-Z]-)?chromosome gains?", "gain"),
    (r"(?i)(?<![A-Za-z])chromosomal gains?", "gain"),
    (r"(?i)(?<![A-Za-z])chromosomal amplifications?", "amplification"),
    (r"(?i)(?<![A-Za-z])chromosomal inversions?", "inversion"),
    (r"(?i)(?<![A-Za-z])complex genomic rearrangements?(?![A-Za-z])", "complex_event"),
    (
        r"(?i)(?<![A-Za-z])(?:high )?hyperdiploid(?:y| karyotype)?(?![A-Za-z])",
        "ploidy_state",
    ),
]

DIRECTION: dict[EventType, CopyNumberDirection] = {
    "gain": "gain",
    "amplification": "amplification",
    "deletion": "loss",
}


def _loci(surface: str) -> list[str]:
    if surface.lower().startswith("y-chromosome"):
        return ["Y"]
    notation = re.fullmatch(
        r"(?i)t\((\d{1,2});(\d{1,2})\)(?:\(([pq][\d.]+);([pq][\d.]+)\))?", surface
    )
    if notation:
        first, second, arm1, arm2 = notation.groups()
        return [first + (arm1 or ""), second + (arm2 or "")]
    inside = re.search(r"\((\d{1,2}[pq]?[\d.]*)\)", surface)
    if inside:
        return [inside.group(1)]
    locus = re.match(r"(?i)(?:chr)?(\d+[pq][\d.]*) (?:deletions?|gain)", surface)
    return [locus.group(1)] if locus else []


def _normalized(surface: str, event_type: EventType) -> str:
    if event_type == "free_text_structural_variant":
        if re.fullmatch(r"(?:CNA|CNAs)", surface):
            return "copy-number abnormality"
        if re.fullmatch(r"(?:SV|SVs|[Ss]tructural [Vv]ariants?)", surface):
            return "structural variant"
    if event_type == "ploidy_state":
        return (
            "high hyperdiploidy"
            if surface.lower().startswith("high ")
            else "hyperdiploidy"
        )
    value = surface.lower()
    plural_words = (
        "translocations",
        "deletions",
        "gains",
        "amplifications",
        "inversions",
        "rearrangements",
    )
    return value[:-1] if value.endswith(plural_words) else value


def detect(text: str) -> list[Detection]:
    """Return compact, sorted exact-span predictions for scoring or adapters."""
    unique: dict[tuple[int, int, str], Detection] = {}
    for expression, event_type in PATTERNS:
        for match in re.finditer(expression, text):
            key = (match.start(), match.end(), event_type)
            unique[key] = {
                "start": match.start(),
                "end": match.end(),
                "text": match.group(),
                "event_type": event_type,
                "source": "deterministic",
            }
    return [unique[key] for key in sorted(unique)]


def annotate(
    text: str, pmid: str, source: str = "PubMed title/abstract"
) -> list[StructuralEvent]:
    """Return occurrence records with normalization and no inferred relationships."""
    events: list[StructuralEvent] = []
    title_end = text.find("\n")
    for row in detect(text):
        surface, event_type = row["text"], row["event_type"]
        events.append(
            {
                "event_id": f"{pmid}:{row['start']}:{row['end']}",
                "event_type": event_type,
                "verbatim": {
                    "text": surface,
                    "start": row["start"],
                    "end": row["end"],
                    "offset_unit": "unicode_codepoints",
                },
                "normalized": {
                    "form": _normalized(surface, event_type),
                    "chromosomes_or_loci": _loci(surface),
                    "copy_number_direction": DIRECTION.get(event_type),
                },
                "parse_status": "complete" if _loci(surface) else "partial",
                "provenance": {
                    "source": source,
                    "pmid": pmid,
                    "passage": "title"
                    if title_end >= 0 and row["end"] <= title_end
                    else "abstract",
                },
                "gene_relationships": [],
            }
        )
    return events


def annotate_documents(
    documents: Iterable[Document], source: str = "PubMed title/abstract"
) -> Iterator[DocumentAnnotations]:
    """Annotate an iterable of article records without filesystem or process I/O."""
    for document in documents:
        pmid, text = str(document["pmid"]), str(document["text"])
        yield {
            "pmid": pmid,
            "structural_events": annotate(text, pmid, document.get("source", source)),
        }


def render_jsonl(
    documents: Iterable[Document], source: str = "PubMed title/abstract"
) -> str:
    """Render document annotations in the deterministic CLI JSONL format."""
    lines = [
        json.dumps(row, ensure_ascii=False)
        for row in annotate_documents(documents, source)
    ]
    return "\n".join(lines) + ("\n" if lines else "")
