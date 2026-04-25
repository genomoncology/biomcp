from __future__ import annotations

from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]


def _read(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def _markdown_section_block(text: str, heading: str, next_heading: str) -> str:
    start = text.index(heading)
    remainder = text[start + len(heading) :]
    end = remainder.find(next_heading)
    if end == -1:
        return remainder
    return remainder[:end]


def test_readme_teaches_search_all_as_unified_entry_point() -> None:
    readme = _read("README.md")
    quick_start = _markdown_section_block(
        readme,
        "## Quick start",
        "\n## Command grammar",
    )
    grammar = _markdown_section_block(
        readme,
        "## Command grammar",
        "\n## Entities and sources",
    )

    assert "biomcp list gene" in quick_start
    assert (
        "biomcp search all --gene BRAF --disease melanoma  "
        "# unified cross-entity discovery"
    ) in quick_start
    assert quick_start.index("biomcp list gene") < quick_start.index(
        "biomcp search all --gene BRAF --disease melanoma"
    )

    assert "batch <entity> <id1,id2,...> → parallel gets" in grammar
    assert "search all [slot filters]    → counts-first cross-entity orientation" in grammar
    assert "across all entities" not in grammar
    assert grammar.index("batch <entity> <id1,id2,...> → parallel gets") < grammar.index(
        "search all [slot filters]    → counts-first cross-entity orientation"
    )


def test_docs_index_teaches_search_all_as_unified_entry_point() -> None:
    docs_index = _read("docs/index.md")
    quick_start = _markdown_section_block(
        docs_index,
        "## Quick start",
        "\n## Command grammar",
    )
    grammar = _markdown_section_block(
        docs_index,
        "## Command grammar",
        "\n## Entities and sources",
    )

    assert "biomcp list gene" in quick_start
    assert (
        "biomcp search all --gene BRAF --disease melanoma  "
        "# unified cross-entity discovery"
    ) in quick_start
    assert quick_start.index("biomcp list gene") < quick_start.index(
        "biomcp search all --gene BRAF --disease melanoma"
    )

    assert "batch <entity> <id1,id2,...> → parallel gets" in grammar
    assert "search all [slot filters]    → counts-first cross-entity orientation" in grammar
    assert "across all entities" not in grammar
    assert grammar.index("batch <entity> <id1,id2,...> → parallel gets") < grammar.index(
        "search all [slot filters]    → counts-first cross-entity orientation"
    )


def test_entities_and_sources_tables_list_current_source_expansion_rows() -> None:
    expectations = {
        "README.md": [
            "| gene | MyGene.info, UniProt, Reactome, QuickGO, STRING, GTEx, Human Protein Atlas, DGIdb, ClinGen, NIH Reporter, DisGeNET, GTR-backed diagnostics pivot | `biomcp get gene BRAF pathways hpa` |",
            "| diagnostic | NCBI Genetic Testing Registry local bulk bundle + WHO IVD local CSV + optional OpenFDA device overlay | `biomcp get diagnostic GTR000006692.3 regulatory` |",
            "| drug | MyChem.info, DDInter local bundle, EMA local batch, WHO Prequalification local exports, ChEMBL, OpenTargets, Drugs@FDA, OpenFDA labels/shortages/approvals/FAERS/MAUDE/recalls, CIViC | `biomcp drug interactions warfarin` |",
            '| disease | MyDisease.info, Monarch Initiative, MONDO, OpenTargets, Reactome, CIViC, SEER Explorer, NIH Reporter, DisGeNET, MedlinePlus `clinical_features`, GTR/WHO IVD diagnostics pivot | `biomcp get disease "Lynch syndrome" genes` |',
            "| pathway | Reactome, KEGG, WikiPathways, g:Profiler, Enrichr-backed enrichment sections | `biomcp get pathway hsa05200 genes` |",
            "| adverse-event | OpenFDA FAERS/MAUDE/recalls plus CDC WONDER VAERS aggregate vaccine search | `biomcp search adverse-event --drug pembrolizumab` |",
        ],
        "docs/index.md": [
            "| gene | MyGene.info, UniProt, Reactome, QuickGO, STRING, GTEx, Human Protein Atlas, DGIdb, ClinGen, NIH Reporter, DisGeNET, GTR-backed diagnostics pivot | `biomcp get gene ERBB2 funding` |",
            "| diagnostic | NCBI Genetic Testing Registry local bulk bundle + WHO IVD local CSV + optional OpenFDA device overlay | `biomcp get diagnostic GTR000006692.3 regulatory` |",
            "| drug | MyChem.info, DDInter local bundle, EMA local batch, WHO Prequalification local exports, ChEMBL, OpenTargets, Drugs@FDA, OpenFDA labels/shortages/approvals/FAERS/MAUDE/recalls, CIViC | `biomcp drug interactions warfarin` |",
            '| disease | MyDisease.info, Monarch Initiative, MONDO, OpenTargets, Reactome, CIViC, SEER Explorer, NIH Reporter, DisGeNET, MedlinePlus `clinical_features`, GTR/WHO IVD diagnostics pivot | `biomcp get disease "chronic myeloid leukemia" funding` |',
            "| pathway | Reactome, KEGG, WikiPathways, g:Profiler, Enrichr-backed enrichment sections | `biomcp get pathway hsa05200 genes` |",
            "| adverse-event | OpenFDA FAERS/MAUDE/recalls plus CDC WONDER VAERS aggregate vaccine search | `biomcp search adverse-event --drug pembrolizumab` |",
        ],
    }

    for path, rows in expectations.items():
        entities = _markdown_section_block(
            _read(path),
            "## Entities and sources",
            "\n## Cross-entity helpers",
        )
        for row in rows:
            assert row in entities


def test_search_all_workflow_guide_has_required_sections_and_examples() -> None:
    guide = _read("docs/how-to/search-all-workflow.md")
    lower = guide.lower()

    assert "# how to:" in lower
    assert "## start with typed slots" in lower
    assert "## use `--counts-only` for a low-noise orientation pass" in lower
    assert "## narrow the next command intentionally" in lower
    assert "## positional compatibility syntax" in lower

    assert "biomcp search all --gene BRAF --disease melanoma" in guide
    assert "biomcp search all --drug pembrolizumab" in guide
    assert 'biomcp search all --keyword "checkpoint inhibitor"' in guide
    assert 'biomcp search all --variant "BRAF V600E"' in guide
    assert "biomcp search all --gene BRAF --counts-only" in guide


def test_search_all_workflow_guide_frames_positional_as_keyword_compatibility() -> None:
    guide = _read("docs/how-to/search-all-workflow.md")
    compat = _markdown_section_block(
        guide,
        "## Positional compatibility syntax",
        "\n## Related",
    )

    assert "biomcp search all BRAF" in compat
    assert "biomcp search all --keyword BRAF" in compat
    assert "--gene BRAF" not in compat


def test_search_all_workflow_guide_teaches_typed_slots_before_compatibility() -> None:
    guide = _read("docs/how-to/search-all-workflow.md")

    assert guide.index("## Start with typed slots") < guide.index(
        "## Positional compatibility syntax"
    )


def test_search_all_workflow_guide_distinguishes_markdown_and_json_counts_only() -> None:
    guide = _read("docs/how-to/search-all-workflow.md")
    counts_only = _markdown_section_block(
        guide,
        "## Use `--counts-only` for a low-noise orientation pass",
        "\n## Use `--debug-plan` to see the executed leg routing",
    )

    assert "In markdown output" in counts_only
    assert "follow-up links" in counts_only
    assert "`--json --counts-only`" in counts_only
    assert "per-section `results` and `links`" in counts_only
    assert "are omitted" in counts_only
    assert "biomcp --json search all --gene BRAF --counts-only" in counts_only


def test_cli_reference_links_search_all_workflow_guide_from_cross_entity_block() -> None:
    cli_reference = _read("docs/user-guide/cli-reference.md")
    all_block = _markdown_section_block(
        cli_reference,
        "### All (cross-entity)",
        "\n### Gene",
    )

    assert (
        "[Search All Workflow](../how-to/search-all-workflow.md)"
        in all_block
    )


def test_docs_index_links_search_all_workflow_guide_from_documentation_section() -> None:
    docs_index = _read("docs/index.md")
    documentation = _markdown_section_block(
        docs_index,
        "## Documentation",
        "\n## Citation",
    )

    assert "[Search All Workflow](how-to/search-all-workflow.md)" in documentation


def test_readme_links_search_all_workflow_guide_from_documentation_section() -> None:
    readme = _read("README.md")
    documentation = _markdown_section_block(
        readme,
        "## Documentation",
        "\n## Citation",
    )

    assert "[Search All Workflow](docs/how-to/search-all-workflow.md)" in documentation


def test_quick_reference_links_search_all_workflow_guide_near_search_all_examples() -> None:
    quick_reference = _read("docs/reference/quick-reference.md")
    common_searches = _markdown_section_block(
        quick_reference,
        "## Common searches",
        "\n## Output modes and discovery commands",
    )

    assert "biomcp search all --gene BRAF --disease melanoma" in common_searches
    assert (
        "[Search All Workflow](../how-to/search-all-workflow.md)"
        in common_searches
    )


def test_mkdocs_nav_contains_search_all_workflow_under_how_to() -> None:
    mkdocs = _read("mkdocs.yml")
    how_to = _markdown_section_block(
        mkdocs,
        "  - How-To:\n",
        "  - Study Charts:\n",
    )

    assert "      - Search All Workflow: how-to/search-all-workflow.md" in how_to
