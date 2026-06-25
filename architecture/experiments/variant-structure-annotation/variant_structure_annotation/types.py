"""Shared types for the variant -> protein-structure spike."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True)
class VariantSpec:
    """Input needed to join variant, protein, domain, and structure sources."""

    gene: str
    change: str
    label: str
    accession: str


@dataclass(frozen=True)
class TimedResult:
    """Result envelope used by benchmark and regression summaries."""

    label: str
    ok: bool
    latency_ms: int
    value: Any | None = None
    error: str | None = None

    def to_dict(self) -> dict[str, Any]:
        out: dict[str, Any] = {
            "label": self.label,
            "ok": self.ok,
            "latency_ms": self.latency_ms,
        }
        if self.ok:
            out["value"] = self.value
        else:
            out["error"] = self.error
        return out
