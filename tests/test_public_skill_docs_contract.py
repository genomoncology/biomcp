from __future__ import annotations

from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]


def _read(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def test_public_skill_docs_match_current_cli_contract() -> None:
    readme = _read("README.md")
    docs_index = _read("docs/index.md")
    skill_file = _read("skills/SKILL.md")
    treatment_use_case = _read("skills/use-cases/01-treatment-lookup.md")
    symptom_use_case = _read("skills/use-cases/02-symptom-phenotype.md")
    orientation_use_case = _read("skills/use-cases/03-gene-disease-orientation.md")
    article_follow_up = _read("skills/use-cases/04-article-follow-up.md")
    skills = _read("docs/getting-started/skills.md")
    reproduce = _read("docs/how-to/reproduce-papers.md")
    cli_reference = _read("docs/user-guide/cli-reference.md")
    article_guide = _read("docs/user-guide/article.md")
    find_articles = _read("docs/how-to/find-articles.md")
    keyword_reference = _read("docs/reference/article-keyword-search.md")
    data_sources = _read("docs/reference/data-sources.md")
    quick_reference = _read("docs/reference/quick-reference.md")
    pivot_guide = _read("docs/how-to/cross-entity-pivots.md")
    blog = _read("docs/blog/biomcp-kuva-charts.md")
    mcp_server = _read("docs/reference/mcp-server.md")
    claude_desktop = _read("docs/getting-started/claude-desktop.md")
    bioasq_benchmark = _read("docs/reference/bioasq-benchmark.md")

    assert "14 guided investigation workflows are built in" not in readme
    assert "biomcp skill install ~/.claude --force" in readme
    assert 'biomcp suggest "<question>"' in readme
    assert "`biomcp skill` to read the embedded BioMCP guide" in readme
    assert "biomcp skill list" not in readme
    assert "biomcp skill show 03" not in readme

    assert "14 guided investigation workflows are built in" not in docs_index
    assert "getting-started/skills.md" in docs_index
    assert "biomcp skill install ~/.claude --force" in docs_index

    assert "# Skills" in skills
    assert "biomcp skill" in skills
    assert "biomcp skill render" in skills
    assert 'biomcp suggest "<question>"' in skills
    assert "biomcp skill list" in skills
    assert "biomcp skill article-follow-up" in skills
    assert "biomcp skill variant-pathogenicity" in skills
    assert "SKILL.md" in skills
    assert "use-cases/" in skills
    assert "jq-examples.md" in skills
    assert "examples/" in skills
    assert "schemas/" in skills
    assert "workflow-ladder.schema.json" in skills
    assert "use-cases/<slug>.ladder.json" in skills
    assert "_meta.workflow" in skills
    assert "_meta.ladder[]" in skills
    assert "Current builds ship examples for treatment lookup, symptom lookup" not in skills
    assert "Current builds ship 15 worked examples" in skills
    for slug in (
        "variant-pathogenicity",
        "drug-regulatory",
        "trial-recruitment",
        "mutation-catalog",
        "negative-evidence",
    ):
        assert slug in skills
    assert "Legacy compatibility note" not in skills
    assert "No skills found" not in skills

    assert "# Skills" in skills
    assert "biomcp skill install ~/.claude" in skills

    assert "biomcp skill list" not in reproduce
    assert "biomcp skill gene-function-lookup" not in reproduce
    assert "biomcp skill 03" not in reproduce
    assert "biomcp get gene BRAF" in reproduce
    assert 'biomcp get variant "BRAF V600E" population' in reproduce
    assert 'biomcp search trial -c melanoma --mutation "BRAF V600E" --status recruiting --limit 5' in reproduce
    assert "biomcp get article 22663011 fulltext" in reproduce

    assert "biomcp skill [list|install|<name>]" not in cli_reference
    assert "biomcp skill install [dir]" in cli_reference
    assert "biomcp skill render" in cli_reference
    assert "biomcp suggest <question>" in cli_reference
    assert "biomcp cache path" in cli_reference
    assert "biomcp cache stats" in cli_reference
    assert "biomcp cache clean" in cli_reference
    assert "biomcp cache clear" in cli_reference
    assert "biomcp skill list                 # list embedded worked examples" in cli_reference
    assert 'biomcp discover "developmental delay"' in cli_reference
    assert 'biomcp search phenotype "seizure, developmental delay" --limit 10' in cli_reference
    assert (
        "`--json` normally returns structured output, but `biomcp cache path` "
        "is a plain-text exception. `biomcp cache stats`, `biomcp cache clean`, "
        "and `biomcp cache clear` respect `--json` on success. `biomcp cache clear` "
        "still refuses non-TTY destructive runs with plain stderr unless you pass `--yes`."
        in cli_reference
    )
    assert "biomcp serve-sse                  # removed compatibility command; use serve-http" not in cli_reference
    assert (
        "`biomcp serve-sse` remains available only as a hidden compatibility "
        "command that points users back to `biomcp serve-http`."
        in cli_reference
    )
    assert "Streamable HTTP" in cli_reference
    assert "/mcp" in cli_reference
    assert "## Workflow ladder metadata" in cli_reference
    assert "_meta.workflow" in cli_reference
    assert "_meta.ladder[]" in cli_reference
    assert "biomcp get drug aspirin --json" in cli_reference
    assert "biomcp drug interactions warfarin" in cli_reference
    assert "single-entity resolver" in cli_reference
    assert 'biomcp search all --keyword "<query>"' in cli_reference
    assert "single-entity concept resolution before typed search/get" in quick_reference
    assert "single-entity concept resolution before entity selection" in docs_index
    assert 'biomcp search all --keyword "<query>"' in docs_index
    assert "search all --keyword" in quick_reference

    assert "one markdown resource per embedded skill use-case" in mcp_server
    assert "biomcp://help" in mcp_server
    assert "biomcp skill render" in mcp_server
    assert "`suggest`" in mcp_server
    assert "biomcp://skill/<slug>" in mcp_server
    assert "Streamable HTTP" in mcp_server
    assert "`biomcp serve-http`" in mcp_server
    assert "`/mcp`" in mcp_server
    assert "`/health`" in mcp_server
    assert "`/readyz`" in mcp_server
    assert "`/`" in mcp_server
    assert "`cache path`" in mcp_server
    assert "`cache stats`" in mcp_server
    assert "`cache clean`" in mcp_server
    assert "`cache clear`" in mcp_server
    assert "reveal workstation-local paths" in mcp_server
    assert "Workflow ladders do not add MCP resources" in mcp_server
    assert "_meta.workflow" in mcp_server
    assert "_meta.ladder[]" in mcp_server

    assert "biomcp skill render" in bioasq_benchmark
    assert "eval runners should call `biomcp skill render`" in bioasq_benchmark
    assert "biomcp skill render > <snapshot-path>" in bioasq_benchmark

    assert "one markdown resource per embedded BioMCP worked example" in claude_desktop
    assert "biomcp://help" in claude_desktop
    assert "biomcp://skill/<slug>" in claude_desktop

    assert "## Routing rules" in skill_file
    assert 'biomcp suggest "<question>"' in skill_file
    assert skill_file.index('biomcp suggest "<question>"') < skill_file.index("## Routing rules")
    assert "## Section reference" in skill_file
    assert "## Cross-entity pivot rules" in skill_file
    assert "## How-to reference" in skill_file
    assert "## Anti-patterns" in skill_file
    assert "## Output and evidence rules" in skill_file
    assert "## Answer commitment" in skill_file
    routing_rules = skill_file[
        skill_file.index("## Routing rules") : skill_file.index("## Section reference")
    ]
    assert "auto-download on first use" in routing_rules
    assert "biomcp ema sync" in routing_rules
    assert "biomcp who sync" in routing_rules
    assert "biomcp cvx sync" in routing_rules
    assert "biomcp ddinter sync" in routing_rules
    assert "biomcp gtr sync" in routing_rules
    assert "biomcp who-ivd sync" in routing_rules
    assert "GTR plus WHO IVD are local diagnostic-test backbones" in routing_rules
    assert "CDC CVX/MVX" in routing_rules
    assert "`biomcp drug interactions <name>`" in routing_rules
    assert 'biomcp search drug --indication "<disease>"' in skill_file
    assert 'biomcp discover "<free text>"' in skill_file
    assert "single-entity resolver" in routing_rules
    assert "single-entity free-text lookup only" in routing_rules
    assert "canonical name, ID, or category of one thing" in routing_rules
    assert "not a relational query tool" in routing_rules
    assert "not a list-question seed" in routing_rules
    assert "biomcp discover BRCA1" in routing_rules
    assert "biomcp discover dabigatran" in routing_rules
    assert 'biomcp search all --keyword "<query>"' in routing_rules
    assert "../docs/" not in skill_file
    assert ".md)" not in skill_file
    section_reference = skill_file[
        skill_file.index("## Section reference") : skill_file.index("## Cross-entity pivot rules")
    ]
    assert "`get diagnostic ... regulatory`" in section_reference
    assert "FDA device 510(k) and PMA" in section_reference
    how_to_table = skill_file[
        skill_file.index("## How-to reference") : skill_file.index("## Anti-patterns")
    ]
    expected_bioasq_rows = [
        (
            "| Specific variant pathogenicity or clinical-evidence question | "
            '`biomcp get variant "<variant>"` | '
            "Use the bounded variant-pathogenicity workflow instead of mixing ad hoc "
            "variant, trial, and article commands |"
        ),
        (
            "| Drug approval, licensing, or regulatory-date question | "
            "`biomcp get drug <name> regulatory` | "
            "Use the structured-first workflow discipline: check `get drug ... "
            "regulatory` before falling back to articles for approval facts |"
        ),
        (
            "| Gene-disease association for a known gene | "
            "`biomcp get gene <symbol> diseases` | "
            "Check `get gene ... diseases` and `search variant --gene ...` for the "
            "full disease spectrum before searching articles |"
        ),
        (
            "| Gene localization or protein-function question | "
            "`biomcp get gene <symbol> protein` and `biomcp get gene <symbol> hpa` | "
            "Pull `get gene ... protein` and `get gene ... hpa` first because "
            "UniProt and HPA usually answer localization or function directly |"
        ),
    ]
    for row in expected_bioasq_rows:
        assert row in how_to_table
    assert (
        "| Diagnostic-test inventory or detail question | "
        "`biomcp list diagnostic` and `biomcp get diagnostic GTR000006692.3` | "
        "Inspect the shipped diagnostic filters and sections first; use gene/disease "
        "diagnostic pivots or `search diagnostic` to get source-native IDs before "
        "`get diagnostic` detail |"
        in how_to_table
    )
    assert "../docs/" not in how_to_table
    assert ".md)" not in how_to_table
    assert (
        "After `search article`, default to `biomcp article batch <id1> <id2> ...` instead of repeated `get article` calls."
        in skill_file
    )
    assert (
        "Use `biomcp batch gene <GENE1,GENE2,...>` when you need the same basic card fields, chromosome, or sectioned output for multiple genes."
        in skill_file
    )
    assert (
        "For diseases with weak ontology-name coverage, run `biomcp discover \"<disease>\"` first, then pass a resolved `MESH:...`, `OMIM:...`, `ICD10CM:...`, `MONDO:...`, or `DOID:...` identifier to `biomcp get disease`."
        in skill_file
    )
    assert (
        "`--type` reduces recall to Europe PMC publication-type filtering today because"
        in skill_file
    )
    assert "Never do more than 3 article searches for one question." in skill_file
    assert "ClinicalTrials.gov usually does not index nicknames" in skill_file
    assert "add `--drug <name>` to `search article`" in skill_file
    assert "DDInter is the local drug-interaction bundle" in skill_file
    assert "_meta.workflow" in skill_file
    assert "_meta.ladder[]" in skill_file
    assert "`biomcp article batch <pmid1> <pmid2> ...` uses spaces between PMIDs." in skill_file
    assert "Only add more commands if a needed claim is still unsupported." in skill_file
    assert "If one command already answers the question, stop searching and answer." in skill_file
    assert "biomcp get drug nivolumab regulatory" in skill_file
    assert "If 1-2 papers you already fetched state the answer" in skill_file
    assert "If 3+ searches keep returning relevant papers" in skill_file
    assert "If you keep reformulating the same search with different keywords" in skill_file
    assert "_meta.next_commands" in skill_file
    assert "Run `biomcp skill list` for worked examples" in skill_file
    anti_patterns = skill_file[
        skill_file.index("## Anti-patterns") : skill_file.index("## Output and evidence rules")
    ]
    assert "### Don't use `discover` for relational or list questions" in anti_patterns
    assert '"drug classes that interact with warfarin"' in anti_patterns
    assert (
        'biomcp search article -k "drug classes that interact with warfarin" --type review --limit 5'
        in anti_patterns
    )
    assert '"genes regulated by MEF2 in the heart"' in anti_patterns
    assert (
        'biomcp search article -k "genes regulated by MEF2 in the heart" --type review --limit 5'
        in anti_patterns
    )
    assert "biomcp get gene <symbol>" in anti_patterns

    assert "Use `article batch` as the default follow-up after `search article`" in article_guide
    assert "`--type` on `--source all` uses Europe PMC + PubMed" in article_guide
    assert "PMC-only note" in article_guide
    assert "LitSense2-derived semantic signal" in article_guide
    assert "Rows without LitSense2 provenance contribute `ranking.semantic_score = 0`" in article_guide
    assert "PubMed ESearch cleans bounded filler words" in article_guide
    assert "MeSH/title/abstract" not in article_guide
    assert (
        "Put a known gene, disease, or drug in `-g/--gene`, `-d/--disease`, or `--drug`."
        in article_guide
    )
    assert (
        'biomcp search article -k \'"cafe-au-lait spots" neurofibromas disease\' --type review --limit 5'
        in article_guide
    )
    assert (
        'biomcp search article --drug amiodarone -k "photosensitivity mechanism" --limit 5'
        in article_guide
    )
    assert (
        "Use `article batch` after search when you already know the candidate PMIDs or"
        in find_articles
    )
    assert "`--type` on the default `--source all` route uses Europe PMC + PubMed" in find_articles
    assert "Europe PMC-only with an explicit note" in find_articles
    assert "LitSense2-derived" in find_articles
    assert "semantic=0" in find_articles
    assert "question-format cleanup before ESearch" in find_articles
    assert "MeSH/title/abstract" not in find_articles
    assert "Do not guess `-g`, `-d`, or `--drug`" in find_articles
    assert 'biomcp search article -k "TCGA mutation analysis dataset" --type review --limit 5' in find_articles
    assert "MeSH/title/abstract" not in keyword_reference
    assert "On the default `--source all` route, adding `-k/--keyword` also brings LitSense2" in keyword_reference
    assert "LitSense2-derived semantic signal" in keyword_reference
    assert "semantic=0" in keyword_reference
    assert "PubMed-specific behavior" in keyword_reference
    assert "Semantic Scholar keep their existing query" in keyword_reference
    assert "do not guess a disease or drug name" in keyword_reference
    assert (
        'biomcp search article --drug amiodarone -k "photosensitivity mechanism" --limit 5'
        in keyword_reference
    )

    assert "# Pattern: Treatment / approved-drug lookup" in treatment_use_case
    assert 'biomcp search drug --indication "myasthenia gravis" --limit 5' in treatment_use_case
    assert "# Pattern: Symptom / phenotype lookup" in symptom_use_case
    assert 'biomcp get disease "Marfan syndrome" phenotypes' in symptom_use_case
    assert 'biomcp discover "developmental delay"' in symptom_use_case
    assert 'biomcp search phenotype "HP:0001263 HP:0001250"' in symptom_use_case
    assert 'biomcp search phenotype "seizure, developmental delay" --limit 5' in symptom_use_case
    assert "# Pattern: Gene-in-disease orientation" in orientation_use_case
    assert "biomcp get gene IPO8" in orientation_use_case
    assert 'biomcp search article -g IPO8 -k "biallelic loss-of-function Loeys-Dietz Shprintzen-Goldberg thoracic aortic aneurysm" --limit 5' in orientation_use_case
    assert "biomcp get gene IPO8 diseases" in orientation_use_case
    assert "biomcp article batch 36905820 34010605" in orientation_use_case
    assert "# Pattern: Article follow-up via citations and recommendations" in article_follow_up
    assert "biomcp article citations 22663011 --limit 5" in article_follow_up

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
