"""In-process structural-variant article annotation API."""

from __future__ import annotations

from typing import Any

from .parser import annotate, annotate_documents, detect, render_jsonl

_TYPE_EXPORTS = {
    "CopyNumberDirection",
    "Detection",
    "Document",
    "DocumentAnnotations",
    "EventType",
    "GeneRelationship",
    "NormalizedEvent",
    "ParseStatus",
    "Provenance",
    "StructuralEvent",
    "VerbatimSpan",
}

__all__ = [
    *_TYPE_EXPORTS,
    "annotate",
    "annotate_documents",
    "detect",
    "render_jsonl",
]


def __getattr__(name: str) -> Any:
    """Load type-only contracts only when a consumer requests one."""
    if name in _TYPE_EXPORTS:
        from . import types

        value = getattr(types, name)
        globals()[name] = value
        return value
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")
