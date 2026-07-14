"""Shared occurrence-level contracts for structural-variant article annotations."""

from __future__ import annotations

from typing import Literal, NotRequired, TypedDict

EventType = Literal[
    "translocation",
    "deletion",
    "gain",
    "amplification",
    "inversion",
    "complex_event",
    "ploidy_state",
    "free_text_structural_variant",
]
ParseStatus = Literal["complete", "partial", "ambiguous", "verbatim_only"]
CopyNumberDirection = Literal["gain", "amplification", "loss"]
Passage = Literal["title", "abstract"]


class Detection(TypedDict):
    """Compact exact-span parser result used by scorers and adapters."""

    start: int
    end: int
    text: str
    event_type: EventType
    source: Literal["deterministic"]


class VerbatimSpan(TypedDict):
    text: str
    start: int
    end: int
    offset_unit: Literal["unicode_codepoints"]


class NormalizedEvent(TypedDict):
    form: str
    chromosomes_or_loci: list[str]
    copy_number_direction: CopyNumberDirection | None


class Provenance(TypedDict):
    source: str
    pmid: str
    passage: Passage


class EvidenceSpan(TypedDict):
    text: str
    start: int
    end: int
    offset_unit: Literal["unicode_codepoints"]


class GeneRelationship(TypedDict):
    """A sourced relationship; parsers must never infer one from notation alone."""

    relation: str
    genes: list[str]
    evidence_span: EvidenceSpan
    provenance: Provenance


class StructuralEvent(TypedDict):
    event_id: str
    event_type: EventType
    verbatim: VerbatimSpan
    normalized: NormalizedEvent
    parse_status: ParseStatus
    provenance: Provenance
    gene_relationships: list[GeneRelationship]


class Document(TypedDict):
    pmid: str
    text: str
    source: NotRequired[str]


class DocumentAnnotations(TypedDict):
    pmid: str
    structural_events: list[StructuralEvent]
