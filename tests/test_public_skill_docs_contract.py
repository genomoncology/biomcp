from __future__ import annotations

from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]


def _read(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def test_public_skill_docs_match_current_cli_contract() -> None:
    readme = _read("README.md")
    docs_index = _read("docs/index.md")
    skill_file = _read("skills/SKILL.md")
    skills = _read("docs/getting-started/skills.md")
    reproduce = _read("docs/how-to/reproduce-papers.md")
    cli_reference = _read("docs/user-guide/cli-reference.md")
    article_guide = _read("docs/user-guide/article.md")
    data_sources = _read("docs/reference/data-sources.md")
    quick_reference = _read("docs/reference/quick-reference.md")
    pivot_guide = _read("docs/how-to/cross-entity-pivots.md")
    blog = _read("docs/blog/biomcp-kuva-charts.md")
    mcp_server = _read("docs/reference/mcp-server.md")
    claude_desktop = _read("docs/getting-started/claude-desktop.md")

    assert "14 guided investigation workflows are built in" not in readme
    assert "biomcp skill install ~/.claude --force" in readme
    assert "`biomcp skill` to read the embedded BioMCP guide" in readme
    assert "biomcp skill list" not in readme
    assert "biomcp skill show 03" not in readme

    assert "14 guided investigation workflows are built in" not in docs_index
    assert "getting-started/skills.md" in docs_index
    assert "biomcp skill install ~/.claude --force" in docs_index

    assert "# Skills" in skills
    assert "biomcp skill" in skills
    assert "biomcp skill install ~/.claude" in skills
    assert "SKILL.md" in skills
    assert "jq-examples.md" in skills
    assert "examples/" in skills
    assert "schemas/" in skills
    assert "Legacy compatibility note" in skills
    assert "biomcp skill list" in skills
    assert "biomcp skill 03" in skills
    assert "biomcp skill variant-to-treatment" in skills
    assert "Skill topics included" not in skills
    assert "biomcp://skill/<slug>" not in skills

    assert "biomcp skill list" not in reproduce
    assert "biomcp skill gene-function-lookup" not in reproduce
    assert "biomcp skill 03" not in reproduce
    assert "biomcp get gene BRAF" in reproduce
    assert 'biomcp get variant "BRAF V600E" population' in reproduce
    assert 'biomcp search trial -c melanoma --mutation "BRAF V600E" --status recruiting --limit 5' in reproduce
    assert "biomcp get article 22663011 fulltext" in reproduce

    assert "biomcp skill [list|install|<name>]" not in cli_reference
    assert "biomcp skill install [dir]" in cli_reference
    assert "biomcp skill list                 # legacy compatibility alias" in cli_reference
    assert "biomcp serve-sse" in cli_reference
    assert "Streamable HTTP" in cli_reference
    assert "/mcp" in cli_reference

    assert "one markdown resource per registered skill use-case" not in mcp_server
    assert "biomcp://help" in mcp_server
    assert "No `biomcp://skill/<slug>` resources are currently listed" in mcp_server
    assert "Streamable HTTP" in mcp_server
    assert "`biomcp serve-http`" in mcp_server
    assert "`/mcp`" in mcp_server
    assert "`/health`" in mcp_server
    assert "`/readyz`" in mcp_server
    assert "`/`" in mcp_server

    assert "one resource per installed skill" not in claude_desktop
    assert "biomcp://help" in claude_desktop
    assert "do not discover a browsable `biomcp://skill/<slug>` catalog" in claude_desktop

    assert "biomcp article references 22663011 --limit 3" in skill_file
    assert "S2_API_KEY" in skill_file
    assert "publisher elision" in skill_file
    assert "biomcp enrich BRAF,KRAS,NRAS --limit 10" in skill_file
    assert "biomcp batch gene BRAF,TP53 --sections pathways,interactions" in skill_file
    assert "biomcp chart" in skill_file
    assert "--theme dark" in skill_file
    assert "--palette wong" in skill_file
    assert "_meta.next_commands" in skill_file
    assert "FDA label interaction text" in skill_file

    assert "publisher elision" in article_guide
    assert "next_commands" in article_guide

    assert "biomcp enrich` uses **g:Profiler**" in data_sources
    assert "Gene enrichment sections" in data_sources
    assert "Enrichr" in data_sources

    assert "biomcp article references 22663011 --limit 3" in quick_reference
    assert "biomcp article references 22663011 --limit 3" in pivot_guide

    assert "docs/blog/images/tp53-mutation-bar.svg" in blog
    assert "![TP53 mutation classes as a bar chart](images/tp53-mutation-bar.svg)" in blog
    assert "![Terminal screenshot placeholder: mutation-bar-terminal.png](images/mutation-bar-terminal.png)" in blog
    assert "![Terminal screenshot placeholder: ridgeline-terminal.png](images/ridgeline-terminal.png)" in blog
