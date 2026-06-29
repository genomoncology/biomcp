from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[2]


def _read(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def _source_inventory() -> list[dict[str, Any]]:
    value = json.loads(_read("docs/reference/sources.json"))
    assert isinstance(value, list)
    return [entry for entry in value if isinstance(entry, dict)]


def _source_by_id(source_id: str) -> dict[str, Any]:
    for entry in _source_inventory():
        if entry.get("id") == source_id:
            return entry
    raise AssertionError(f"missing source inventory entry: {source_id}")


def _mentions_boundary(content: str, *term_groups: tuple[str, ...]) -> bool:
    lower = content.lower()
    return all(any(term in lower for term in group) for group in term_groups)


def test_variant_guide_documents_transcript_normalization_proxy_boundary() -> None:
    guide = _read("docs/user-guide/variant.md")

    for phrase in (
        "biomcp variant normalize all NM_000248.3:c.135del",
        "biomcp variant normalize all 'NM_004448.2:c.829G>T'",
        "Mutalyzer",
        "VariantValidator",
    ):
        assert phrase in guide

    assert _mentions_boundary(
        guide,
        ("not", "without", "avoid", "reject"),
        ("parse", "parsing", "extract"),
        ("report prose", "report text", "messy report"),
    )
    assert _mentions_boundary(
        guide,
        ("not", "without", "avoid", "reject"),
        ("choose", "select", "guess", "infer"),
        ("transcript", "transcripts"),
    )
    assert _mentions_boundary(
        guide,
        ("not", "without", "avoid", "reject"),
        ("classify", "classification", "clinical meaning", "clinical interpretation"),
        ("variant", "variants"),
    )


def test_cli_references_expose_variant_normalize_command_shape() -> None:
    for path in (
        "docs/user-guide/cli-reference.md",
        "architecture/ux/cli-reference.md",
    ):
        content = _read(path)
        assert "variant normalize <service> <transcript_hgvs>" in content
        assert "NM_000248.3:c.135del" in content
        assert "NM_004448.2:c.829G>T" in content


def test_data_source_reference_documents_normalization_upstreams() -> None:
    data_sources = _read("docs/reference/data-sources.md")

    for phrase in (
        "Mutalyzer",
        "VariantValidator",
        "variant normalize",
        "https://mutalyzer.nl/api",
        "/normalize/{description}",
        "https://rest.variantvalidator.org",
        "/VariantValidator/variantvalidator/{genome_build}/{variant_description}/{select_transcripts}",
        "TranscriptVersionWarning",
    ):
        assert phrase in data_sources

    assert re.search(
        r"\|[^\n]*Mutalyzer[^\n]*\|[^\n]*variant normalize[^\n]*\|[^\n]*No[^\n]*\|",
        data_sources,
    )
    assert re.search(
        r"\|[^\n]*VariantValidator[^\n]*\|[^\n]*variant normalize[^\n]*\|[^\n]*No[^\n]*\|",
        data_sources,
    )


def test_source_inventory_includes_variant_normalization_providers() -> None:
    mutalyzer = _source_by_id("mutalyzer")
    variantvalidator = _source_by_id("variantvalidator")

    for source in (mutalyzer, variantvalidator):
        surfaces = source.get("bioMcp_surfaces")
        assert isinstance(surfaces, list)
        assert any("variant normalize" in str(surface) for surface in surfaces)
        assert source.get("bioMcp_auth") == "none"
        assert source.get("integration_mode") == "direct_api"

    assert "mutalyzer.nl" in str(mutalyzer.get("terms_url"))
    assert "variantvalidator" in str(variantvalidator.get("terms_url")).lower()
