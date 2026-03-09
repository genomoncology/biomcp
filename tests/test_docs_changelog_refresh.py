from __future__ import annotations

from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]


def _read(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def _markdown_section_block(text: str, heading: str) -> str:
    start = text.index(heading)
    remainder = text[start + len(heading) :]
    next_heading = remainder.find("\n## ")
    if next_heading == -1:
        return remainder
    return remainder[:next_heading]


def test_changelog_has_backfilled_releases_and_release_header() -> None:
    changelog = _read("CHANGELOG.md")

    assert "## [Unreleased]" not in changelog
    assert "## 0.8.13 — 2026-03-09" in changelog
    assert "## 0.9.0" not in changelog

    expected_releases = [
        ("0.8.13", "2026-03-09"),
        ("0.8.12", "2026-03-07"),
        ("0.8.11", "2026-03-06"),
        ("0.8.10", "2026-03-04"),
        ("0.8.9", "2026-03-03"),
        ("0.8.8", "2026-03-02"),
        ("0.8.7", "2026-02-27"),
        ("0.8.6", "2026-02-27"),
        ("0.8.5", "2026-02-26"),
    ]
    for version, date in expected_releases:
        header = f"## {version} — {date}"
        assert header in changelog
        assert "\n- " in _markdown_section_block(changelog, header)


def test_gene_guide_includes_new_sections_and_positional_search() -> None:
    gene_guide = _read("docs/user-guide/gene.md")

    assert "biomcp search gene BRAF --limit 5" in gene_guide
    assert "biomcp get gene BRAF expression" in gene_guide
    assert "biomcp get gene BRAF druggability" in gene_guide
    assert "biomcp get gene BRAF clingen" in gene_guide


def test_article_guide_documents_federated_search_and_source_flag() -> None:
    article_guide = _read("docs/user-guide/article.md")

    assert "PubTator3 and Europe PMC" in article_guide
    assert "deduplicated by PMID" in article_guide
    assert "--source pubtator" in article_guide
    assert "--source europepmc" in article_guide


def test_data_sources_reference_covers_new_gene_and_article_sources() -> None:
    data_sources = _read("docs/reference/data-sources.md")

    assert "UniProt, QuickGO, STRING, GTEx, DGIdb, ClinGen" in data_sources
    assert "https://gtexportal.org/api/v2" in data_sources
    assert "https://dgidb.org/api/graphql" in data_sources
    assert "https://search.clinicalgenome.org" in data_sources
    assert "| Article search & metadata | PubTator3 + Europe PMC |" in data_sources
    assert "PubTator3 + Europe PMC for federated search" in data_sources


def test_cli_and_quick_reference_cover_search_all_and_gene_sections() -> None:
    cli_reference = _read("docs/user-guide/cli-reference.md")
    quick_reference = _read("docs/reference/quick-reference.md")

    assert "### All (cross-entity)" in cli_reference
    assert "biomcp search all --gene BRAF --disease melanoma" in cli_reference
    assert "biomcp get gene BRAF pathways ontology diseases protein" in cli_reference
    assert (
        "biomcp get gene BRAF go interactions civic expression druggability clingen"
        in cli_reference
    )
    assert "biomcp get gene BRAF all" in cli_reference
    assert "_meta.evidence_urls" in cli_reference
    assert "Ensembl, OMIM, NCBI Gene, and UniProt URLs." in cli_reference

    assert "biomcp search gene BRAF --limit 5" in quick_reference
    assert "biomcp search all --gene BRAF --disease melanoma" in quick_reference
    assert "biomcp search all --keyword resistance --counts-only" in quick_reference
