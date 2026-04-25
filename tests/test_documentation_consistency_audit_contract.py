from __future__ import annotations

from pathlib import Path
import re

REPO_ROOT = Path(__file__).resolve().parents[1]

HOW_TO_TITLES = {
    "docs/how-to/annotate-variants.md": "# How to: annotate variants",
    "docs/how-to/cross-entity-pivots.md": "# How to: use cross-entity pivots",
    "docs/how-to/find-articles.md": "# How to: find articles",
    "docs/how-to/find-trials.md": "# How to: find trials",
    "docs/how-to/guide-workflows.md": "# How to: follow guide workflows",
    "docs/how-to/predict-effects.md": "# How to: predict variant effects",
    "docs/how-to/reproduce-papers.md": "# How to: reproduce paper workflows",
    "docs/how-to/search-all-workflow.md": "# How to: orient with `search all`",
    "docs/how-to/skill-validation.md": "# How to: validate skill runs",
}

ENTITY_GUIDE_HEADINGS = {
    "docs/user-guide/gene.md": [
        "## Search genes",
        "## Get a gene record",
        "## Request deeper sections",
        "## Helper commands",
        "## JSON mode",
        "## Practical tips",
        "## Related guides",
    ],
    "docs/user-guide/variant.md": [
        "## Search variants",
        "## Get a variant record",
        "## Request variant sections",
        "## Helper commands",
        "## JSON mode",
        "## Practical tips",
        "## Related guides",
    ],
    "docs/user-guide/article.md": [
        "## Search articles",
        "## Get an article",
        "## Request specific sections",
        "## Helper commands",
        "## JSON mode",
        "## Practical tips",
        "## Related guides",
    ],
    "docs/user-guide/drug.md": [
        "## Search drugs",
        "## Get a drug record",
        "## Request drug sections",
        "## Helper commands",
        "## JSON mode",
        "## Practical tips",
        "## Related guides",
    ],
    "docs/user-guide/disease.md": [
        "## Search diseases",
        "## Get disease records",
        "## Disease sections",
        "## Helper commands",
        "## JSON mode",
        "## Practical tips",
        "## Related guides",
    ],
    "docs/user-guide/pathway.md": [
        "## Search pathways",
        "## Get pathway records",
        "## Request pathway sections",
        "## Helper commands",
        "## JSON mode",
        "## Practical tips",
        "## Related guides",
    ],
    "docs/user-guide/protein.md": [
        "## Search proteins",
        "## Get protein records",
        "## Request protein sections",
        "## Helper commands",
        "## JSON mode",
        "## Practical tips",
        "## Related guides",
    ],
    "docs/user-guide/trial.md": [
        "## Search trials (default source)",
        "## Get a trial by NCT ID",
        "## Request trial sections",
        "## Helper commands",
        "## JSON mode",
        "## Practical tips",
        "## Related guides",
    ],
    "docs/user-guide/adverse-event.md": [
        "## Search FAERS reports",
        "## Get a report by ID",
        "## Request report sections",
        "## Helper commands",
        "## JSON mode",
        "## Practical tips",
        "## Related guides",
    ],
    "docs/user-guide/pgx.md": [
        "## Search PGX",
        "## Get PGX records",
        "## Request PGX sections",
        "## Helper commands",
        "## JSON mode",
        "## Practical tips",
        "## Related guides",
    ],
    "docs/user-guide/gwas.md": [
        "## Search GWAS",
        "## Get records",
        "## Request sections",
        "## Helper commands",
        "## JSON mode",
        "## Practical tips",
        "## Related guides",
    ],
    "docs/user-guide/phenotype.md": [
        "## Search phenotypes",
        "## Get records",
        "## Request sections",
        "## Helper commands",
        "## JSON mode",
        "## Practical tips",
        "## Related guides",
    ],
}

PUBLIC_ENV_VARS = (
    "ALPHAGENOME_API_KEY",
    "DISGENET_API_KEY",
    "NCBI_API_KEY",
    "NCI_API_KEY",
    "ONCOKB_TOKEN",
    "OPENFDA_API_KEY",
    "S2_API_KEY",
    "UMLS_API_KEY",
)

CHART_REFERENCE_PAGES = [
    "docs/charts/bar.md",
    "docs/charts/box.md",
    "docs/charts/density.md",
    "docs/charts/heatmap.md",
    "docs/charts/histogram.md",
    "docs/charts/pie.md",
    "docs/charts/ridgeline.md",
    "docs/charts/scatter.md",
    "docs/charts/stacked-bar.md",
    "docs/charts/survival.md",
    "docs/charts/violin.md",
    "docs/charts/waterfall.md",
]

EXAMPLE_READMES = [
    "examples/geneagent/README.md",
    "examples/genegpt/README.md",
    "examples/pubmed-beyond/README.md",
    "examples/trialgpt/README.md",
]


def _read(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def _normalize_whitespace(text: str) -> str:
    return re.sub(r"\s+", " ", text).strip()


def _assert_heading_order(text: str, headings: list[str]) -> None:
    positions = [text.index(heading) for heading in headings]
    assert positions == sorted(positions)


def test_landing_copy_and_public_env_inventory_follow_consistency_contract() -> None:
    readme = _read("README.md")
    docs_index = _read("docs/index.md")
    api_keys = _read("docs/getting-started/api-keys.md")
    claude_desktop = _read("docs/getting-started/claude-desktop.md")
    data_sources = _read("docs/reference/data-sources.md")
    error_codes = _read("docs/reference/error-codes.md")
    source_licensing = _read("docs/reference/source-licensing.md")

    article_sentence = (
        "`search article` fans out across PubTator3 and Europe PMC, "
        "deduplicates PMID/PMCID/DOI identifiers, and can add a Semantic "
        "Scholar leg when your filters support it."
    )
    study_sentence = (
        "`study` commands cover local query, cohort, survival, compare, and "
        "co-occurrence workflows with native terminal, SVG, and PNG charts "
        "for downloaded cBioPortal-style datasets."
    )

    assert article_sentence in _normalize_whitespace(readme)
    assert article_sentence in _normalize_whitespace(docs_index)
    assert study_sentence in _normalize_whitespace(readme)
    assert study_sentence in _normalize_whitespace(docs_index)
    assert "suggest <question>" in readme
    assert "suggest <question>" in docs_index
    assert "discover <query>" in readme
    assert "Before Anthropic directory approval" not in readme
    assert "Anthropic Directory" in readme
    assert "generated `.mcpb` bundle directly" not in readme
    assert (
        "For local/manual setups, use the JSON MCP config below."
        in _normalize_whitespace(readme)
    )
    assert (
        "If your Claude Desktop build offers the Anthropic Directory, install "
        "BioMCP there first. Use the JSON config below when you want a "
        "local/manual setup."
    ) in _normalize_whitespace(claude_desktop)

    assert (
        "BioMCP keeps `ONCOKB_TOKEN` because OncoKB itself calls the "
        "credential a token."
    ) in _normalize_whitespace(api_keys)

    for env_var in PUBLIC_ENV_VARS:
        assert env_var in docs_index
        assert env_var in api_keys
        assert env_var in data_sources
        assert env_var in error_codes

    civic_block = source_licensing.split("### CIViC\n", 1)[1].split("\n### ", 1)[0]
    for civic_surface in (
        "get variant <id> civic",
        "get gene <symbol> civic",
        "get drug <name> civic",
        "get disease <id> civic",
        "get disease <id> variants",
    ):
        assert civic_surface in civic_block


def test_how_to_titles_match_consistency_contract() -> None:
    for path, title in HOW_TO_TITLES.items():
        assert _read(path).startswith(f"{title}\n")


def test_entity_guides_keep_canonical_section_flow_and_note_missing_families() -> None:
    for path, headings in ENTITY_GUIDE_HEADINGS.items():
        guide = _read(path)
        _assert_heading_order(guide, headings)

    gene = _read("docs/user-guide/gene.md")
    cli_reference = _read("docs/user-guide/cli-reference.md")
    disease = _read("docs/user-guide/disease.md")
    trial = _read("docs/user-guide/trial.md")
    adverse_event = _read("docs/user-guide/adverse-event.md")
    pgx = _read("docs/user-guide/pgx.md")
    gwas = _read("docs/user-guide/gwas.md")
    phenotype = _read("docs/user-guide/phenotype.md")
    protein = _read("docs/user-guide/protein.md")
    pathway = _read("docs/user-guide/pathway.md")
    variant = _read("docs/user-guide/variant.md")

    assert "biomcp search gene BRAF --limit 5" in gene
    assert "biomcp search gene BRAF --limit 10 --offset 0" in cli_reference
    assert "biomcp get gene BRAF all" in gene
    assert "## Gene helper commands" not in gene
    assert "## Cross-entity helpers" not in _read("docs/user-guide/drug.md")
    assert "\n## Helper command\n" not in protein
    assert "## Practical guidance" not in adverse_event
    assert "### Search filters" not in pgx
    assert "### Search filters" not in gwas
    assert "### Search filters" not in phenotype
    assert "biomcp disease trials melanoma" in disease
    assert "biomcp disease drugs melanoma" in disease
    assert 'biomcp disease articles "Lynch syndrome"' in disease
    assert "no direct `trial <helper>` family" in trial
    assert "`biomcp drug adverse-events <name>`" in adverse_event
    assert "PGX does not expose a separate helper family" in pgx
    assert "GWAS is search-only." in gwas
    assert "Phenotype is search-only." in phenotype
    assert "## Practical tips" in pathway
    assert "## Practical tips" in protein
    assert "## Practical tips" in variant


def test_discover_guide_uses_direct_copy_and_related_guides() -> None:
    discover = _read("docs/user-guide/discover.md")
    discover_ws = _normalize_whitespace(discover)

    assert (
        "Use `biomcp discover` to resolve free-text biomedical phrases into the "
        "right BioMCP follow-up commands."
    ) in discover_ws
    assert "primarily a single-entity resolver" in discover_ws
    assert 'biomcp search all --keyword "<query>"' in discover
    assert "## Related guides" in discover


def test_discover_and_phenotype_guides_cover_hpo_bridge_and_text_resolution() -> None:
    discover = _read("docs/user-guide/discover.md")
    phenotype = _read("docs/user-guide/phenotype.md")

    assert 'biomcp discover "developmental delay"' in discover
    assert 'biomcp search phenotype "HP:..."' in discover
    assert 'biomcp discover "symptoms of Marfan syndrome"' in discover
    assert 'biomcp search all --keyword "<query>"' in discover
    assert 'biomcp search phenotype "seizure, developmental delay"' in phenotype
    assert "space- or comma-separated" in phenotype
    assert "one symptom phrase" in phenotype
    assert "multiple symptom phrases separated by commas" in phenotype
    assert "Free-text symptom phrases are resolved to HPO IDs" in phenotype
    assert 'biomcp discover "<symptom text>"' in phenotype


def test_chart_reference_pages_use_shared_compact_shape() -> None:
    for path in CHART_REFERENCE_PAGES:
        page = _read(path)
        assert "## Supported Commands" in page
        assert "## Examples" in page

    assert "## Terminal Output Example" not in _read("docs/charts/bar.md")


def test_blog_try_it_and_install_copy_are_consistent() -> None:
    blog = _read("docs/blog/we-deleted-35-tools.md")

    assert "We went from 36 MCP tools to one CLI command" in blog
    assert "uv tool install biomcp-cli" in blog
    assert "curl -fsSL https://biomcp.org/install.sh | bash" in blog
    assert blog.index("curl -fsSL https://biomcp.org/install.sh | bash") < blog.index(
        "uv tool install biomcp-cli"
    )
    assert "skills/SKILL.md" in blog
    assert "../getting-started/skills.md" in blog

    blog_files = sorted((REPO_ROOT / "docs/blog").glob("*.md"))
    for path in blog_files:
        blog_text = path.read_text(encoding="utf-8")
        if "## Try" in blog_text:
            assert "## Try it" in blog_text
            assert "## Try It" not in blog_text


def test_examples_and_operator_readmes_use_plain_runtime_copy() -> None:
    examples_index = _read("examples/README.md")
    scripts = _read("scripts/README.md")
    paper = _read("paper/README.md")
    bioasq = _read("benchmarks/bioasq/README.md")

    assert "project 116" not in examples_index

    for path in EXAMPLE_READMES:
        readme = _read(path)
        assert "**Prerequisites:** `uv tool install biomcp-cli`" in readme
        assert "`PI_CMD`" in readme
        assert "default `pi`" in readme
        assert "No environment variables are required for the default prompts." in readme
        assert "anchor facts" not in readme.lower()
        assert " anchors" not in readme.lower()

    assert (
        "lightweight commands for checking upstream source behavior"
        in _normalize_whitespace(scripts)
    )
    assert "source-facing contract probes" not in scripts
    assert "Today, only" not in paper
    assert "committed stubs now" not in paper
    assert "Only `run-traceability-audit.sh` is runnable immediately." in paper
    assert "uv run --quiet --script benchmarks/bioasq/ingest_public.py" in bioasq


def test_temporal_and_term_drift_are_removed_from_touched_docs() -> None:
    search_all = _read("docs/how-to/search-all-workflow.md")
    find_trials = _read("docs/how-to/find-trials.md")
    cross_entity = _read("docs/how-to/cross-entity-pivots.md")
    annotate_variants = _read("docs/how-to/annotate-variants.md")
    data_sources = _read("docs/reference/data-sources.md")
    discover = _read("docs/user-guide/discover.md")
    drug = _read("docs/user-guide/drug.md")
    disease = _read("docs/user-guide/disease.md")
    gene = _read("docs/user-guide/gene.md")
    troubleshooting = _read("docs/troubleshooting.md")

    assert "--date-from" not in search_all
    assert "--since 2024-01-01" in search_all

    for text in (
        find_trials,
        cross_entity,
        annotate_variants,
        data_sources,
        discover,
    ):
        assert "best effort" not in text

    for text in (drug, disease, gene, troubleshooting):
        assert "now auto-downloads" not in text
        assert "now includes" not in text

    assert "fulltext may have different availability" not in data_sources
    assert "metadata, annotations, and full text may have different availability" in data_sources


def test_cache_path_docs_match_resolved_cache_root_contract() -> None:
    troubleshooting = _read("docs/troubleshooting.md")
    data_sources = _read("docs/reference/data-sources.md")
    faq = _read("docs/reference/faq.md")
    blog = _read("docs/blog/biomcp-pubmed-articles.md")
    troubleshooting_ws = _normalize_whitespace(troubleshooting)
    blog_ws = _normalize_whitespace(blog)

    for text in (troubleshooting, data_sources, faq, blog):
        assert "http-cacache" not in text
        assert "/tmp/biomcp" not in text

    assert "biomcp cache clean" in troubleshooting
    assert "biomcp cache clear --yes" in troubleshooting
    assert "biomcp cache path" in troubleshooting
    assert "--dry-run" in troubleshooting
    assert "--max-age 7d" in troubleshooting
    assert "resolved cache root" in troubleshooting_ws
    assert "print the managed HTTP cache path" in troubleshooting_ws
    assert "safe first step" in troubleshooting_ws
    assert "downloads/" in troubleshooting
    assert "manual operator step" not in troubleshooting_ws

    assert "`<cache_root>/http`" in data_sources
    assert "`~/.cache/biomcp/http` on Linux" in data_sources
    assert "`biomcp cache path`" in data_sources

    assert "`http/` for HTTP responses" in faq
    assert "`downloads/` for retrieved text artifacts" in faq
    assert "`biomcp cache path`" in faq

    assert "Saved to: <cache_root>/downloads/" in blog
    assert "follows the resolved cache root on your machine" in blog_ws
