from __future__ import annotations

import os
import re
import tomllib
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_PLANNING_ROOT = REPO_ROOT / "tests" / "fixtures" / "planning" / "biomcp"


def _read_repo(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def _current_release_tag_example() -> str:
    cargo = tomllib.loads(_read_repo("Cargo.toml"))
    return f"v{cargo['package']['version']}"


def _planning_root() -> Path:
    return Path(os.environ.get("BIOMCP_PLANNING_ROOT", DEFAULT_PLANNING_ROOT))


def _read_planning(path: str) -> str:
    return (_planning_root() / path).read_text(encoding="utf-8")


def _normalize_ws(text: str) -> str:
    return re.sub(r"\s+", " ", text).strip()


def _markdown_section(text: str, heading: str, level: int = 2) -> str:
    marker = "#" * level
    match = re.search(
        rf"^{re.escape(marker)} {re.escape(heading)}\n(.*?)(?=^{re.escape(marker)} |\Z)",
        text,
        flags=re.MULTILINE | re.DOTALL,
    )
    assert match is not None, f"missing section {marker} {heading}"
    return match.group(1)


def _markdown_table_rows(text: str) -> list[list[str]]:
    rows: list[list[str]] = []
    for line in text.splitlines():
        stripped = line.strip()
        if not stripped.startswith("|"):
            continue
        cells = [cell.strip() for cell in stripped.strip("|").split("|")]
        if all(re.fullmatch(r":?-+:?", cell) for cell in cells):
            continue
        rows.append(cells)
    return rows


def _workflow_job_block(workflow: str, job_name: str) -> str:
    match = re.search(
        rf"^  {re.escape(job_name)}:\n(.*?)(?=^  [A-Za-z0-9_-]+:\n|\Z)",
        workflow,
        flags=re.MULTILINE | re.DOTALL,
    )
    assert match is not None, f"missing workflow job {job_name}"
    return match.group(1)


def _workflow_run_steps(job_block: str) -> list[str]:
    return re.findall(r"^\s+- run: (.+)$", job_block, flags=re.MULTILINE)


def test_planning_contract_uses_repo_fixture_fallback_by_default(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.delenv("BIOMCP_PLANNING_ROOT", raising=False)

    assert _planning_root() == DEFAULT_PLANNING_ROOT
    assert "# BioMCP Strategy" in _read_planning("strategy.md")


def test_planning_contract_reads_explicit_env_override(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    monkeypatch.setenv("BIOMCP_PLANNING_ROOT", str(tmp_path))
    (tmp_path / "strategy.md").write_text("# override strategy\n", encoding="utf-8")

    assert _planning_root() == tmp_path
    assert _read_planning("strategy.md") == "# override strategy\n"


def test_planning_contract_bad_override_fails_loudly(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    missing_root = tmp_path / "missing-planning-root"
    monkeypatch.setenv("BIOMCP_PLANNING_ROOT", str(missing_root))

    with pytest.raises(FileNotFoundError):
        _read_planning("strategy.md")


def test_strategy_and_frontier_capture_upstream_planning_contract() -> None:
    strategy = _read_planning("strategy.md")
    frontier = _read_planning("frontier.md")

    assert "# BioMCP Strategy" in strategy
    assert "Rust core, Python packaging" in strategy
    assert "Rate limiting is process-local" in strategy
    assert "G002" in strategy
    assert "G003" in strategy
    assert len(strategy.splitlines()) <= 80

    assert "# BioMCP Frontier" in frontier
    assert "## G002" in frontier
    assert "## G003" in frontier
    assert "architecture/functional/overview.md" in frontier
    assert "architecture/technical/overview.md" in frontier
    assert "architecture/ux/cli-reference.md" in frontier
    assert "Harvest Guidance" in frontier


def test_functional_overview_preserves_readme_surface_and_study_family() -> None:
    functional = _read_repo("architecture/functional/overview.md")

    assert "# BioMCP Functional Overview" in functional
    assert "## Entity Surface" in functional
    for entity in (
        "| gene |",
        "| variant |",
        "| article |",
        "| trial |",
        "| drug |",
        "| disease |",
        "| pathway |",
        "| protein |",
        "| adverse-event |",
        "| pgx |",
        "| gwas |",
        "| phenotype |",
    ):
        assert entity in functional

    assert "## Study Command Family" in functional
    assert "`study` is a separate local analytics surface" in functional
    assert (
        "`biomcp study list|download|top-mutated|filter|query|co-occurrence|cohort|survival|compare`"
        in functional
    )
    assert "`gwas` and `phenotype` are search-only" in functional
    assert "BioMCP ships an embedded agent guide plus worked examples" in functional
    assert "`biomcp skill` shows the BioMCP agent guide" in functional
    assert "`biomcp skill render` prints the canonical agent prompt" in functional
    assert "`biomcp skill install <dir>` installs that guide" in functional
    assert "`biomcp skill list` shows embedded worked examples" in functional
    assert "`biomcp skill <name>` opens an embedded worked example" in functional
    assert "`biomcp://skill/<slug>`" in functional
    assert "suggest <question>" in functional
    assert "offline question-to-playbook routing" in functional
    assert "discover <query>" in functional
    assert "free-text concept resolution into typed follow-up commands" in functional
    assert "search all [slot filters]" in functional
    assert "biomcp search all --gene BRAF --disease melanoma" in functional
    assert "biomcp search all BRAF" in functional


def test_functional_overview_repaired_source_rows_match_current_contract() -> None:
    functional = _read_repo("architecture/functional/overview.md")
    functional_ws = _normalize_ws(functional)

    assert "13 remote entity commands" not in functional
    assert "all 13 remote entity commands" not in functional
    assert "public entity surface" in functional_ws

    assert (
        "| article | PubMed, PubTator3, Europe PMC, LitSense2 (keyword-gated), "
        "PMC OA, NCBI ID Converter, Semantic Scholar (optional auth; `S2_API_KEY` "
        "recommended) | `biomcp search article -g BRAF --limit 5` |" in functional
    )
    assert (
        "| gene | MyGene.info, UniProt, Reactome, QuickGO, STRING, GTEx, Human "
        "Protein Atlas, DGIdb, ClinGen, gnomAD, CIViC, NIH Reporter, GTR-backed "
        "diagnostics pivot | `biomcp get gene ERBB2 funding` |" in functional
    )
    assert (
        "| diagnostic | NCBI Genetic Testing Registry local bulk exports, WHO IVD "
        "local CSV, optional OpenFDA device 510(k)/PMA overlay | "
        "`biomcp search diagnostic --gene BRCA1 --limit 5` |" in functional
    )
    assert (
        "| drug | MyChem.info, EMA local batch, WHO Prequalification local CSV, "
        "ChEMBL, OpenTargets, Drugs@FDA, OpenFDA labels/shortages/approvals/"
        "FAERS/MAUDE/recalls, CIViC | `biomcp get drug trastuzumab regulatory "
        "--region who` |" in functional
    )
    assert (
        "| disease | MyDisease.info, Monarch Initiative, MONDO, OpenTargets, "
        "Reactome, CIViC, SEER Explorer, NIH Reporter, MedlinePlus "
        "`clinical_features`, GTR/WHO IVD diagnostics pivot | `biomcp get "
        'disease "chronic myeloid leukemia" funding` |' in functional
    )
    assert (
        "| adverse-event | OpenFDA FAERS/MAUDE/recalls, CDC WONDER VAERS "
        "aggregate vaccine search | `biomcp search adverse-event -d "
        "pembrolizumab` |" in functional
    )
    assert (
        "| pathway | Reactome, KEGG, WikiPathways, g:Profiler, Enrichr-backed "
        "enrichment sections | `biomcp get pathway hsa05200 genes` |" in functional
    )


def test_clinical_features_architecture_doc_is_current_state() -> None:
    clinical = _read_repo("architecture/functional/clinical-features-port.md")
    clinical_ws = _normalize_ws(clinical)
    clinical_lower = clinical_ws.lower()

    assert clinical.startswith("# Disease Clinical Features Architecture")
    for stale_phrase in (
        "target state",
        "target-state",
        "does not yet expose clinical features",
        "current implementation gaps",
        "backlog",
        "build ticket",
        "ticket a:",
        "ticket b:",
        "ticket c:",
        "decomposition",
        "json and markdown contracts expose the same fields",
    ):
        assert stale_phrase not in clinical_lower

    assert "## Current Surface" in clinical
    assert "## Source Selection" in clinical
    assert "## Runtime Flow" in clinical
    assert "## Output Contract" in clinical
    assert "## Failure Behavior" in clinical
    assert "## Verification" in clinical
    assert "`get disease <name_or_id> clinical_features`" in clinical
    assert "MedlinePlus Search" in clinical
    assert "reviewed configured diseases" in clinical_ws
    assert "embedded fallback" in clinical_ws
    assert "JSON exposes the full row contract" in clinical_ws
    assert "Markdown renders stable display columns" in clinical_ws
    assert "explicit opt-in" in clinical_ws
    assert "`all` excludes `clinical_features`" in clinical_ws
    assert "Unsupported diseases" in clinical_ws
    assert "evidence URLs" in clinical_ws
    assert "HPO mapping" in clinical_ws
    assert "`_meta.section_sources`" in clinical
    assert "HPO/Monarch phenotypes remain separate" in clinical_ws


def test_article_fulltext_architecture_doc_is_current_state() -> None:
    article = _read_repo("architecture/functional/article-fulltext.md")
    article_ws = _normalize_ws(article)
    article_lower = article_ws.lower()

    assert article.startswith("# Article Fulltext Architecture")
    for stale_phrase in (
        "target state",
        "target-state",
        "backlog",
        "build ticket",
        "ticket a",
        "ticket b",
        "decomposition",
        "does not yet",
    ):
        assert stale_phrase not in article_lower

    for heading in (
        "## Current Surface",
        "## Identity Bridge and Resolver Order",
        "## Eligibility, Format, and License Gates",
        "## Saved Artifact Contract",
        "## Failure Visibility",
        "## Module Ownership",
        "## Verification",
    ):
        assert heading in article

    for phrase in (
        "`docs/user-guide/article.md`",
        "`docs/reference/source-licensing.md`",
        "`get article <id> fulltext`",
        "`get article <id> fulltext --pdf`",
        "NCBI ID Converter is an identity bridge",
        "Europe PMC PMC XML",
        "NCBI EFetch PMC XML",
        "PMC OA Archive XML",
        "Europe PMC MED XML",
        "PMC HTML",
        "Semantic Scholar PDF",
        "`full_text_path`",
        "`full_text_note`",
        "`full_text_source.kind`",
        "`jats_xml`",
        "`html`",
        "`pdf`",
        "`full_text_source.label`",
        "`full_text_source.source`",
        "`_meta.section_sources`",
        "`Saved to:`",
        "BioMCP does not enforce article-level reuse licenses at runtime",
        "There is no public per-leg trace",
        "XML API errors are recorded internally",
        "HTML and PDF fetch, conversion, or content-type failures are misses",
        "Semantic Scholar enrichment failure is swallowed as a warning",
        "`src/entities/article/detail.rs`",
        "`src/entities/article/fulltext.rs`",
        "`src/sources/europepmc.rs`",
        "`src/sources/ncbi_efetch.rs`",
        "`src/sources/pmc_oa.rs`",
        "`src/sources/ncbi_idconv.rs`",
        "`src/sources/semantic_scholar.rs`",
        "`src/transform/article.rs`",
        "`src/transform/article/jats.rs`",
        "`src/transform/article/html.rs`",
        "`src/transform/article/pdf.rs`",
        "`src/render/markdown/article.rs`",
        "`templates/article.md.j2`",
        "`src/render/provenance.rs`",
        "`src/utils/download.rs`",
        "`spec/18-source-labels.md::Article Fulltext Resolver Order`",
    ):
        assert phrase in article_ws


def test_technical_and_ux_docs_match_current_cli_and_workflow_contracts() -> None:
    technical = _read_repo("architecture/technical/overview.md")
    ux = _read_repo("architecture/ux/cli-reference.md")
    article_guide = _read_repo("docs/user-guide/article.md")
    find_articles = _read_repo("docs/how-to/find-articles.md")
    article_keyword_reference = _read_repo("docs/reference/article-keyword-search.md")
    data_sources = _read_repo("docs/reference/data-sources.md")
    cli_list_reference = _read_repo("src/cli/list_reference.md")
    article_mod = _read_repo("src/entities/article/mod.rs")
    article_planner = _read_repo("src/entities/article/planner.rs")
    article_graph = _read_repo("src/entities/article/graph.rs")
    article_usage = _read_repo("tests/article_usage_stderr.rs")
    release_workflow = _read_repo(".github/workflows/release.yml")
    install_script = _read_repo("install.sh")
    technical_ws = _normalize_ws(technical)
    ux_ws = _normalize_ws(ux)
    example_tag = _current_release_tag_example()
    article_guide_ws = _normalize_ws(article_guide)
    find_articles_ws = _normalize_ws(find_articles)
    article_keyword_reference_ws = _normalize_ws(article_keyword_reference)
    data_sources_ws = _normalize_ws(data_sources)
    cli_list_reference_ws = _normalize_ws(cli_list_reference)
    article_validation_section = _normalize_ws(
        _markdown_section(technical, "Article Federation and Front-Door Validation")
    )
    release_pipeline_section = _normalize_ws(
        _markdown_section(technical, "Release Pipeline")
    )

    assert "## Article Federation and Front-Door Validation" in technical
    assert (
        "`search article --source all` plans PubTator3 plus Europe PMC plus PubMed"
        in article_validation_section
    )
    assert "Keyword-bearing queries also add LitSense2" in article_validation_section
    assert "Semantic Scholar remains an optional compatible search leg" in (
        article_validation_section
    )
    assert (
        "Strict Europe PMC-only filters such as `--open-access` and `--type` "
        "disable the federated planner"
    ) in article_validation_section
    assert (
        "`--source pubtator` with strict Europe PMC-only filters is rejected at the front door"
        in article_validation_section
    )
    assert (
        "`--source` remains `all|pubtator|europepmc|pubmed|litsense2` in v1"
        in article_validation_section
    )
    assert (
        "Article filters remain raw as the shared contract"
        in article_validation_section
    )
    assert "PubMed ESearch cleans bounded question-format filler words" in (
        article_validation_section
    )
    assert (
        "PubTator3, Europe PMC, LitSense2, and Semantic Scholar receive their existing query inputs"
        in (article_validation_section)
    )
    assert (
        "deduplicate across PMID, PMCID, and DOI where possible, then re-rank locally"
        in article_validation_section
    )

    assert (
        "| Article full-text resolution | Europe PMC + NCBI E-utilities + PMC OA + NCBI ID Converter + PMC HTML + opt-in Semantic Scholar PDF metadata |"
        in data_sources
    )
    assert "Optional (`NCBI_API_KEY`, `S2_API_KEY`)" in data_sources
    assert (
        "NCBI ID Converter bridges PMID or DOI to PMCID before PMCID-dependent full-text rungs"
        in data_sources_ws
    )
    assert (
        "Semantic Scholar supplies `openAccessPdf` metadata for the explicit `--pdf` fallback"
        in data_sources_ws
    )
    assert (
        "`search article` rejects missing filters, invalid date values, inverted date ranges, "
        "and unsupported `--type` values before backend calls"
        in article_validation_section
    )
    assert (
        "`get article` accepts PMID, PMCID, and DOI only and rejects unsupported identifiers "
        "such as publisher PIIs with a clean `InvalidArgument`"
        in article_validation_section
    )
    assert (
        "Semantic Scholar helper commands accept PMID, PMCID, DOI, arXiv, and Semantic Scholar paper IDs"
        in article_validation_section
    )
    assert (
        "Keyword-bearing article queries default to `hybrid`, while entity-only article "
        "queries default to `lexical`" in technical_ws
    )
    assert (
        "`semantic` sorts the LitSense2-derived semantic signal descending and falls "
        "back to the lexical comparator" in technical_ws
    )
    assert "with `semantic=0` when LitSense2 did not match" in technical_ws
    assert "0.4*semantic + 0.3*lexical + 0.2*citations + 0.1*position" in technical_ws
    assert (
        "Semantic Scholar article helpers are explicitly limited to 1 request/sec per process and are not part of article search fan-out"
        not in technical
    )
    assert (
        "Article search fans out to PubTator3, Europe PMC, and PubMed by default when the filter set is compatible."
        in article_guide_ws
    )
    assert "MeSH/title/abstract" not in article_guide_ws
    assert (
        "When a non-empty keyword is present, BioMCP also adds LitSense2 to the federated route."
        in article_guide_ws
    )
    assert "LitSense2-derived semantic signal" in article_guide_ws
    assert (
        "Rows without LitSense2 provenance contribute `ranking.semantic_score = 0`"
        in article_guide_ws
    )
    assert "PubMed ESearch cleans bounded filler words" in article_guide_ws
    assert "## Query formulation" in article_guide
    assert (
        "Put a known gene, disease, or drug in `-g/--gene`, `-d/--disease`, or `--drug`."
        in article_guide
    )
    assert (
        'biomcp search article --drug amiodarone -k "photosensitivity mechanism" --limit 5'
        in article_guide
    )
    assert "MeSH/title/abstract" not in find_articles_ws
    assert "LitSense2-derived semantic signal" in find_articles_ws
    assert "semantic=0" in find_articles_ws
    assert (
        "Direct PubMed search and the compatible federated PubMed leg apply the same question-format cleanup"
        in find_articles_ws
    )
    assert (
        "Do not guess `-g`, `-d`, or `--drug` when the question is trying to identify the entity itself."
        in find_articles_ws
    )
    assert (
        'biomcp search article -k "TCGA mutation analysis dataset" --type review --limit 5'
        in find_articles_ws
    )
    assert "MeSH/title/abstract" not in article_keyword_reference_ws
    assert (
        "On the default `--source all` route, adding `-k/--keyword` also brings LitSense2 into compatible federated searches and makes the default relevance mode `hybrid`."
        in article_keyword_reference_ws
    )
    assert "LitSense2-derived semantic signal" in article_keyword_reference_ws
    assert "semantic=0" in article_keyword_reference_ws
    assert "PubMed-specific behavior" in article_keyword_reference_ws
    assert (
        "PubTator3, Europe PMC, LitSense2, and Semantic Scholar keep their existing query behavior"
        in article_keyword_reference_ws
    )
    assert (
        "do not guess a disease or drug name just to fill `-d` or `--drug`"
        in article_keyword_reference_ws
    )
    assert "Turn a literature question into article filters" in cli_list_reference_ws
    assert (
        "known gene/disease/drug anchors go in `-g/-d/--drug`; free-text concepts go in `-k`"
        in cli_list_reference_ws
    )
    assert "PubMed ESearch cleans question-format terms provider-locally" in (
        cli_list_reference_ws
    )
    assert "LitSense2-derived semantic signal" in cli_list_reference_ws
    assert "semantic=0" in cli_list_reference_ws
    assert (
        "PubTator3 + Europe PMC + PubMed + LitSense2 + optional Semantic Scholar"
        in (data_sources_ws)
    )
    assert (
        "PubTator3 + Europe PMC + PubMed for federated search, with LitSense2 added for "
        "keyword-bearing queries and an optional Semantic Scholar leg"
        in data_sources_ws
    )
    assert (
        "fn has_strict_europepmc_filters(filters: &ArticleSearchFilters) -> bool {"
        in article_planner
    )
    assert "fn plan_backends(" in article_planner
    assert "pub(crate) fn semantic_scholar_search_enabled(" in article_planner
    assert "--source pubtator does not support --type." in article_planner
    assert "--source pubtator does not support --open-access." in article_planner
    assert (
        "Unsupported identifier format for Semantic Scholar article helpers:"
        in article_graph
    )
    assert (
        "Unsupported identifier format. BioMCP resolves PMID (digits only"
        in article_mod
    )
    assert (
        "invalid_article_type_is_clean_usage_error_before_pubtator_route"
        in article_usage
    )
    assert "missing_article_filters_is_clean_usage_error" in article_usage

    assert "CI (`.github/workflows/ci.yml`) runs five parallel jobs" in technical
    assert (
        "`check` (`cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`)"
        in technical_ws
    )
    assert "`make check` is the required local ticket gate" in technical
    assert "that means `lint`, `test`, and `check-quality-ratchet`" in technical
    assert "`cargo deny check licenses` plus `cargo deny check advisories`" in technical
    assert "`version-sync` (`bash scripts/check-version-sync.sh`)" in technical
    assert "`climb-hygiene` (`bash scripts/check-no-climb-tracked.sh`)" in technical
    assert (
        "`contracts` (`cargo build --release --locked`, `uv sync --extra dev`, "
        '`uv run pytest tests/ -v --mcp-cmd "./target/release/biomcp serve"`, '
        "`uv run mkdocs build --strict`)" in technical_ws
    )
    assert (
        "`spec-stable` (`cargo build --release --locked`, then `make spec-pr`)"
        in technical_ws
    )
    assert (
        "PR CI runs `make spec-pr` via the `spec-stable` job in `.github/workflows/ci.yml`"
        in technical_ws
    )
    assert (
        "Volatile live-network headings run separately in `.github/workflows/spec-smoke.yml`"
        in technical_ws
    )
    assert (
        "Contract smoke checks run in `.github/workflows/contracts.yml`" in technical_ws
    )
    assert (
        "The semver tag is the canonical release/version authority."
        in release_pipeline_section
    )
    assert (
        "PR CI enforces version parity before release via the `version-sync` job and "
        "`scripts/check-version-sync.sh`" in release_pipeline_section
    )
    assert (
        "The release workflow builds binaries, publishes PyPI wheels, and deploys docs "
        "from the tagged source" in release_pipeline_section
    )
    assert (
        "`install.sh` resolves the latest release with platform assets, not the latest merge to `main`"
        in release_pipeline_section
    )
    assert (
        "The existing `### Post-tag public proof` block is the live verification step for "
        "tag-to-binary and tag-to-docs parity" in release_pipeline_section
    )
    assert (
        f'tag="${{BIOMCP_TAG:?set BIOMCP_TAG to the published release tag, e.g. {example_tag}}}"'
        in technical
    )
    assert 'version="${tag#v}"' in technical
    assert "`workflow_dispatch` can replay a specified tag" in release_pipeline_section
    assert "Release validation runs the Rust checks again" in technical
    assert "workflow_dispatch:" in release_workflow
    assert "inputs:" in release_workflow
    assert "tag:" in release_workflow
    assert "deploy-docs:" in release_workflow
    assert "uv run mkdocs gh-deploy --force" in release_workflow
    assert (
        'DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"'
        in install_script
    )
    assert "Resolved latest release with assets" in install_script
    assert "Streamable HTTP" in technical
    assert "`/mcp`" in technical
    assert "`/health`" in technical
    assert "`/readyz`" in technical
    assert "connection line and `Command:` markers" in technical
    assert "through the remote" in technical
    assert "`biomcp` tool" in technical
    assert "remote `shell`" not in technical
    assert "mirror the CLI command surface" not in technical
    assert "read-only allowlist rather than mirroring the full CLI" in technical_ws
    assert "suggest" in technical_ws
    assert "read-only `skill` lookup/list/render behavior" in technical_ws
    assert (
        "Operator-local or mutating commands such as `cache`, `update`, `serve`, "
        "`serve-http`, and `skill install` stay blocked over MCP." in technical_ws
    )

    assert "`search all` Contract" in ux
    assert "biomcp suggest <question>" in ux
    assert (
        'biomcp suggest "<question>"   → select a worked-example playbook and two starter commands'
        in ux
    )
    assert "biomcp discover <query>" in ux
    assert "## See Also and Next Commands" in ux
    assert "`_meta.next_commands`" in ux
    assert "`discover_try_line()`" in ux
    assert "degrade by omission, not by emitting dead commands" in ux
    assert (
        "runtime-generated per-record `next_commands` remain outside the static list contract"
        in ux_ws
    )
    assert "typed slots first" in ux
    assert "biomcp search all --gene BRAF --disease melanoma" in ux
    assert 'biomcp search all --keyword "checkpoint inhibitor"' in ux
    assert "biomcp search all BRAF" in ux
    assert "positional alias" in ux
    assert "biomcp skill                  → show the embedded BioMCP agent guide" in ux
    assert "biomcp skill render           → print the canonical agent prompt" in ux
    assert "biomcp skill list             → list embedded worked examples" in ux
    assert (
        "biomcp cache path             → print the managed HTTP cache path (plain text; ignores `--json`)"
        in ux
    )
    assert (
        "biomcp cache stats            → show HTTP cache statistics (JSON supported)"
        in ux
    )
    assert (
        "biomcp cache clean            → remove orphan blobs and optionally age- or size-evict the HTTP cache (JSON supported)"
        in ux
    )
    assert (
        "biomcp cache clear [--yes]    → destructively wipe the managed HTTP cache tree (JSON success; TTY or `--yes` required)"
        in ux
    )
    assert "Overview: `biomcp skill`" in ux
    assert "Render: `biomcp skill render`" in ux
    assert "List: `biomcp skill list`" in ux
    assert "Open: `biomcp skill 01` or `biomcp skill article-follow-up`" in ux
    assert "biomcp://skill/<slug>" in ux
    assert (
        "biomcp serve-http            → run the MCP Streamable HTTP server at `/mcp`"
        in ux
    )
    assert (
        "biomcp serve-sse             → removed compatibility command; use `biomcp serve-http`"
        not in ux
    )
    assert (
        "Compatibility note: `biomcp serve-sse` remains available only as a hidden "
        "compatibility command that points users to `biomcp serve-http`." in ux
    )
    assert (
        "JSON is the default script contract for query commands, with a documented "
        "plain-text exception for `biomcp cache path`. `biomcp cache stats`, "
        "`biomcp cache clean`, and `biomcp cache clear` support `--json` on "
        "success, while `cache clear` still refuses non-TTY destructive runs "
        "unless `--yes` is present. The cache family remains CLI-only because "
        "revealing workstation-local filesystem paths over MCP would cross the "
        "runtime security boundary." in ux_ws
    )
    assert "`src/render/markdown/related.rs`" in ux
    assert "`src/render/markdown.rs`" not in ux
    assert "`src/cli/tests/`" in ux
    assert "`next_commands_validity` tests in `src/cli/mod.rs`" not in ux
    assert "13 remote entity commands" not in ux
    assert "all 13 remote entity commands" not in ux
    assert "entity command surface" in ux_ws
    assert 'biomcp get disease "uterine leiomyoma" clinical_features' in ux
    assert (
        "Opt-in sections such as `clinical_features`, `diagnostics`, `disgenet`, "
        "and `funding` still require explicit naming." in ux_ws
    )


def test_technical_overview_repaired_gate_and_health_copy_match_contract() -> None:
    technical = _read_repo("architecture/technical/overview.md")
    technical_ws = _normalize_ws(technical)

    assert (
        "BioMCP is a single Rust binary (`biomcp`) with three operating modes:"
        in technical
    )
    assert (
        "`make check` is the required local ticket gate. In the current `Makefile`, "
        "that means `lint`, `test`, and `check-quality-ratchet`; the `lint` stage "
        "runs `cargo deny check licenses` plus `cargo deny check advisories`, and "
        "still rejects deprecated install strings in `README.md` and `docs/`."
        in technical_ws
    )
    assert (
        "`--apis-only` omits the EMA local-data row, the WHO Prequalification "
        "local-data row, the CDC CVX/MVX local-data row, the GTR local-data "
        "row, the WHO IVD local-data row, the cache-writability row, and the "
        "cache-limits row because none of these are upstream API checks."
        in technical_ws
    )


def test_chart_rendering_architecture_doc_matches_repo_contract() -> None:
    technical = _read_repo("architecture/technical/overview.md")
    chart_section = _normalize_ws(_markdown_section(technical, "Chart Rendering"))

    assert "## Chart Rendering" in technical
    assert "`biomcp chart` serves embedded markdown chart docs" in chart_section
    assert "`src/cli/chart.rs`" in chart_section
    assert "`docs/charts/`" in chart_section
    assert "`RustEmbed`" in chart_section
    assert (
        "`biomcp chart` documents the chart surface, but does not render charts"
        in chart_section
    )
    assert "`ChartArgs`" in chart_section
    assert "`src/cli/types.rs`" in chart_section
    assert "`src/render/chart.rs`" in chart_section
    assert "`study query`" in chart_section
    assert "`study co-occurrence`" in chart_section
    assert "`study compare`" in chart_section
    assert "`study survival`" in chart_section
    assert (
        "`bar`, `stacked-bar`, `pie`, `waterfall`, `heatmap`, `histogram`, `density`, `box`, `violin`, `ridgeline`, `scatter`, and `survival`"
        in chart_section
    )
    assert "terminal" in chart_section
    assert "SVG file" in chart_section
    assert "PNG file behind the `charts-png` feature" in chart_section
    assert "MCP inline SVG" in chart_section
    assert "`--cols` and `--rows` size terminal output" in chart_section
    assert (
        "`--width` and `--height` size SVG, PNG, and MCP inline SVG output"
        in chart_section
    )
    assert "`--scale` is PNG-only" in chart_section
    assert (
        "`--title`, `--theme`, and `--palette` style rendered charts" in chart_section
    )
    assert "Heatmaps reject `--palette`" in chart_section
    assert "`rewrite_mcp_chart_args()`" in chart_section
    assert "text pass plus an SVG pass" in chart_section
    assert "`--terminal` is stripped" in chart_section
    assert "`--output` / `-o` are rejected" in chart_section
    assert (
        "`--cols` / `--rows` and `--scale` are rejected for the SVG pass"
        in chart_section
    )
    assert "`docs/charts/index.md`" in chart_section
    assert "user-facing chart reference and examples" in chart_section


def test_source_integration_architecture_doc_captures_repo_contract() -> None:
    technical = _read_repo("architecture/technical/overview.md")
    source_integration = _read_repo("architecture/technical/source-integration.md")
    drug_guide = _read_repo("docs/user-guide/drug.md")
    bioasq_reference = _read_repo("docs/reference/bioasq-benchmark.md")
    cli_commands = _read_repo("src/cli/commands.rs")
    cli_drug_mod = _read_repo("src/cli/drug/mod.rs")
    cli_list = _read_repo("src/cli/list.rs")
    cli_list_reference = _read_repo("src/cli/list_reference.md")
    cli_reference_guide = _read_repo("docs/user-guide/cli-reference.md")
    drug_get = _read_repo("src/entities/drug/get.rs")
    ema_source = _read_repo("src/sources/ema.rs")
    health = _read_repo("src/cli/health.rs")
    drug_spec = _read_repo("spec/05-drug.md")
    bioasq_reference_ws = _normalize_ws(bioasq_reference)
    cli_reference_guide_ws = _normalize_ws(cli_reference_guide)
    section_first_section = _normalize_ws(
        _markdown_section(source_integration, "Section-First Entity Integration")
    )
    local_runtime_section = _normalize_ws(
        _markdown_section(
            source_integration, "Local Runtime Sources and File-Backed Assets"
        )
    )
    modifier_section = _normalize_ws(
        _markdown_section(source_integration, "Entity-Specific Command Modifiers")
    )
    source_aware_section = _normalize_ws(
        _markdown_section(
            source_integration, "Source-Aware Section Capability Contract"
        )
    )
    source_addition_section = _normalize_ws(
        _markdown_section(source_integration, "Source Addition Checklist")
    )

    assert "source-integration.md" in technical
    assert "# BioMCP Source Integration Architecture" in source_integration
    assert "## New Source vs Existing Source" in source_integration
    assert "`src/sources/<source>.rs`" in source_integration
    assert "`src/sources/mod.rs`" in source_integration
    assert "`shared_client()`" in source_integration
    assert "`streaming_http_client()`" in source_integration
    assert "`env_base(default, ENV_VAR)`" in source_integration
    assert "`read_limited_body()`" in source_integration
    assert "`body_excerpt()`" in source_integration
    assert "`retry_send()`" in source_integration
    assert "## Section-First Entity Integration" in source_integration
    assert "`src/cli/commands.rs`" in section_first_section
    assert "`src/cli/drug/mod.rs`" in section_first_section
    assert "`src/cli/mod.rs`" not in section_first_section
    assert "`src/cli/list.rs`" in section_first_section
    assert "`docs/user-guide/cli-reference.md`" in section_first_section
    assert "default `get` output stays concise" in section_first_section
    assert "## Local Runtime Sources and File-Backed Assets" in source_integration
    assert (
        "EMA, WHO Prequalification, CDC CVX/MVX, GTR, and WHO IVD are local runtime sources."
        in local_runtime_section
    )
    assert (
        "`BIOMCP_EMA_DIR` first, then the platform data directory"
        in local_runtime_section
    )
    assert (
        "`BIOMCP_WHO_DIR` first, then the platform data directory"
        in local_runtime_section
    )
    assert (
        "`BIOMCP_CVX_DIR` first, then the platform data directory"
        in local_runtime_section
    )
    assert (
        "`BIOMCP_GTR_DIR` first, then the platform data directory"
        in local_runtime_section
    )
    assert (
        "`BIOMCP_WHO_IVD_DIR` first, then the platform data directory"
        in local_runtime_section
    )
    assert (
        "`biomcp health` includes the EMA, WHO Prequalification, CDC CVX/MVX, GTR, and WHO IVD local-data readiness rows"
        in local_runtime_section
    )
    assert "`biomcp health --apis-only` excludes those rows" in local_runtime_section
    for expected_row in (
        "| EMA | `BIOMCP_EMA_DIR` | `medicines.json`, `post_authorisation.json`, `referrals.json`, `psusas.json`, `dhpcs.json`, `shortages.json` | 72 hours | `biomcp ema sync` | full health row; omitted from `--apis-only` | `docs/reference/source-licensing.md` / `docs/reference/sources.json` |",
        "| WHO Prequalification | `BIOMCP_WHO_DIR` | `who_pq.csv`, `who_api.csv`, `who_vaccines.csv` | 72 hours | `biomcp who sync` | full health row; omitted from `--apis-only` | `docs/reference/source-licensing.md` / `docs/reference/sources.json` |",
        "| CDC CVX/MVX | `BIOMCP_CVX_DIR` | `cvx.txt`, `TRADENAME.txt`, `mvx.txt` | 30 days | `biomcp cvx sync` | full health row; omitted from `--apis-only` | `docs/reference/source-licensing.md` / `docs/reference/sources.json` |",
        "| GTR | `BIOMCP_GTR_DIR` | `test_version.gz`, `test_condition_gene.txt` | 7 days | `biomcp gtr sync` | full health row; omitted from `--apis-only` | `docs/reference/source-licensing.md` / `docs/reference/sources.json` |",
        "| WHO IVD | `BIOMCP_WHO_IVD_DIR` | `who_ivd.csv` | 72 hours | `biomcp who-ivd sync` | full health row; omitted from `--apis-only` | `docs/reference/source-licensing.md` / `docs/reference/sources.json` |",
    ):
        assert expected_row in source_integration
    assert "first-use auto-download" in local_runtime_section
    assert "canonical provider terms" in local_runtime_section
    assert (
        "`configured`, `configured (stale)`, `available (default path)`, "
        "`available (default path, stale)`, `not configured`, and "
        "`error (missing: ...)`" in local_runtime_section
    )
    assert "`docs/user-guide/drug.md`" in local_runtime_section
    assert "`docs/user-guide/diagnostic.md`" in local_runtime_section
    assert "30-day refresh window" in local_runtime_section
    assert (
        "BioASQ is the canonical file-backed non-runtime asset" in local_runtime_section
    )
    assert (
        "do not join the runtime source inventory, `biomcp health`, or the source-readiness checklist"
        in local_runtime_section
    )
    assert "`docs/reference/bioasq-benchmark.md`" in local_runtime_section
    assert "`benchmarks/bioasq/`" in local_runtime_section
    assert "## EMA local data setup" in drug_guide
    assert "## WHO Prequalification local data setup" in drug_guide
    assert "## CDC CVX/MVX local data setup" in drug_guide
    assert "`configured`:" in drug_guide
    assert drug_guide.count("`configured (stale)`:") >= 3
    assert "`available (default path)`:" in drug_guide
    assert drug_guide.count("`available (default path, stale)`:") >= 3
    assert "`not configured`:" in drug_guide
    assert "`error (missing: ...)`:" in drug_guide
    assert "pub(crate) fn resolve_ema_root() -> PathBuf {" in ema_source
    assert 'std::env::var("BIOMCP_EMA_DIR")' in ema_source
    assert "EMA local data" in health
    assert "CDC CVX/MVX local data" in health
    assert "GTR local data" in health
    assert "WHO IVD local data" in health
    assert "available (default path)" in health
    assert "not configured" in health
    assert "error (missing:" in health
    assert "BioASQ" not in health
    assert "# BioASQ Benchmark" in bioasq_reference
    assert (
        "offline benchmark input, not as a live runtime source" in bioasq_reference_ws
    )
    assert (REPO_ROOT / "benchmarks" / "bioasq").is_dir()
    assert "## Entity-Specific Command Modifiers" in source_integration
    assert (
        "The base grammar remains `get <entity> <id> [section...]`." in modifier_section
    )
    assert "Entity-specific modifiers are named options" in modifier_section
    assert (
        "The canonical example is `get drug <name> ... --region <us|eu|who|all>`."
        in modifier_section
    )
    assert "`src/cli/commands.rs`" in modifier_section
    assert "`src/cli/drug/mod.rs`" in modifier_section
    assert "`src/cli/mod.rs`" not in modifier_section
    assert "`src/cli/list.rs`" in modifier_section
    assert "`src/cli/list_reference.md`" in modifier_section
    assert "`docs/user-guide/cli-reference.md`" in modifier_section
    assert "`docs/user-guide/drug.md`" in modifier_section
    assert "`spec/05-drug.md`" in modifier_section
    assert (
        "Runtime validation belongs in the owning entity or CLI path"
        in modifier_section
    )
    assert (
        "`--region` only changes the data plane for `regulatory`, `safety`, `shortage`, or `all`"
        in modifier_section
    )
    assert (
        "`--region who` is valid for `regulatory` and `all`, but not for `safety` or"
        in modifier_section
    )
    assert "`approvals` remains U.S.-only" in modifier_section
    assert (
        "invalid flag/section combinations fail fast before data fetches"
        in modifier_section
    )
    assert "biomcp get drug trastuzumab regulatory --region who" in cli_commands
    assert "biomcp get drug trastuzumab regulatory --region who" in cli_reference_guide
    assert "biomcp cvx sync" in cli_reference_guide
    assert (
        "Data region for regional sections (regulatory, safety, shortage, or all)"
        in cli_drug_mod
    )
    assert "get drug <name> regulatory [--region <us|eu|who|all>]" in cli_list
    assert "get drug <name> safety [--region <us|eu|all>]" in cli_list
    assert "get drug <name> shortage [--region <us|eu|all>]" in cli_list
    assert "get drug <name> approvals" in cli_list
    assert "get drug <name> regulatory [--region <us|eu|who|all>]" in cli_list_reference
    assert (
        "get drug <name> safety|shortage [--region <us|eu|all>]" in cli_list_reference
    )
    assert (
        "For `get drug`, use `--region` only with `regulatory`, `safety`, `shortage`, or `all`"
        in cli_reference_guide_ws
    )
    assert "get drug <name> regulatory [--region <us|eu|who|all>]" in drug_spec
    assert "--region is not supported with approvals." in drug_get
    assert (
        "--region can only be used with regulatory, safety, shortage, or all."
        in drug_get
    )
    assert "## Provenance and Rendering" in source_integration
    assert "`source_label`" in source_integration
    assert "source-specific notes" in source_integration
    assert "`src/entities/article/mod.rs`" in source_integration
    assert "`src/entities/article.rs`" not in source_integration
    assert "`src/render/markdown/`" in source_integration
    assert "`src/render/markdown.rs`" not in source_integration
    assert "## Auth, Cache, and Secrets" in source_integration
    assert "`BioMcpError::ApiKeyRequired`" in source_integration
    assert "`apply_cache_mode_with_auth(..., true)`" in source_integration
    assert "`docs/getting-started/api-keys.md`" in source_integration
    assert "`docs/reference/data-sources.md`" in source_integration
    assert "Do not log secrets" in source_integration
    assert "## Graceful Degradation and Timeouts" in source_integration
    assert (
        "Optional enrichments must not take down the whole command"
        in source_integration
    )
    assert "truthful about missing or unavailable data" in source_integration
    assert "## Rate Limits and Operational Constraints" in source_integration
    assert "`biomcp serve-http`" in source_integration
    assert "process-local" in source_integration
    assert "## Source Addition Checklist" in source_integration
    assert "`src/cli/commands.rs`" in source_aware_section
    assert "`src/cli/drug/mod.rs`" in source_aware_section
    assert "`src/cli/mod.rs`" not in source_aware_section
    assert "`src/cli/commands.rs`" in source_addition_section
    assert "`src/cli/drug/mod.rs`" in source_addition_section
    assert "`src/cli/mod.rs`" not in source_addition_section
    assert "`docs/reference/source-versioning.md`" in source_addition_section
    assert "`src/cli/health.rs`" in source_addition_section
    assert "`scripts/contract-smoke.sh`" in source_addition_section
    assert "`spec/`" in source_addition_section
    assert "`CHANGELOG.md`" in source_addition_section


def test_pull_request_contract_gate_matches_release_validation() -> None:
    ci = _read_repo(".github/workflows/ci.yml")
    release = _read_repo(".github/workflows/release.yml")
    contracts_smoke = _read_repo(".github/workflows/contracts.yml")
    spec_smoke = _read_repo(".github/workflows/spec-smoke.yml")
    expected_ci_contract_runs = [
        "cargo build --release --locked",
        "uv sync --extra dev",
        'uv run pytest tests/ -v --mcp-cmd "./target/release/biomcp serve"',
        "uv run mkdocs build --strict",
    ]
    expected_release_contract_runs = [
        "uv sync --extra dev",
        'uv run pytest tests/ -v --mcp-cmd "biomcp serve"',
        "uv run mkdocs build --strict",
    ]

    ci_contracts = _workflow_job_block(ci, "contracts")
    ci_spec = _workflow_job_block(ci, "spec-stable")
    ci_version_sync = _workflow_job_block(ci, "version-sync")
    ci_climb_hygiene = _workflow_job_block(ci, "climb-hygiene")
    release_validate = _workflow_job_block(release, "validate")
    smoke_spec = _workflow_job_block(spec_smoke, "spec-volatile-live")

    assert 'python-version: "3.12"' in ci_contracts
    assert 'python-version: "3.12"' in ci_spec
    assert 'python-version: "3.12"' in release_validate
    assert 'python-version: "3.12"' in smoke_spec
    assert _workflow_run_steps(ci_contracts) == expected_ci_contract_runs
    assert "- uses: actions/checkout@v4" in ci_spec
    assert "uses: arduino/setup-protoc@v3" in ci_spec
    assert "uses: dtolnay/rust-toolchain@stable" in ci_spec
    assert "uses: actions/setup-python@v5" in ci_spec
    assert "uses: astral-sh/setup-uv@v4" in ci_spec
    assert _workflow_run_steps(ci_spec) == [
        "cargo build --release --locked",
        "make spec-pr",
    ]
    assert _workflow_run_steps(release_validate)[-3:] == expected_release_contract_runs
    assert "- uses: actions/checkout@v4" in ci_version_sync
    assert _workflow_run_steps(ci_version_sync) == [
        "bash scripts/check-version-sync.sh"
    ]
    for forbidden in (
        "setup-python",
        "setup-uv",
        "setup-protoc",
        "rust-toolchain",
        "cargo ",
        "uv sync",
        "python-version:",
    ):
        assert forbidden not in ci_version_sync
    assert "- uses: actions/checkout@v4" in ci_climb_hygiene
    assert _workflow_run_steps(ci_climb_hygiene) == [
        "bash scripts/check-no-climb-tracked.sh"
    ]
    for forbidden in (
        "setup-python",
        "setup-uv",
        "setup-protoc",
        "rust-toolchain",
        "cargo ",
        "uv sync",
        "python-version:",
    ):
        assert forbidden not in ci_climb_hygiene

    assert "name: Contract Smoke Tests" in contracts_smoke
    assert 'cron: "0 6 * * *"' in contracts_smoke
    assert "workflow_dispatch:" in contracts_smoke
    assert "continue-on-error: true" in contracts_smoke
    assert "- run: bash scripts/contract-smoke.sh" in contracts_smoke

    assert "name: Spec smoke (volatile live-network)" in spec_smoke
    assert 'cron: "0 7 * * *"' in spec_smoke
    assert "workflow_dispatch:" in spec_smoke
    assert "- uses: actions/checkout@v4" in smoke_spec
    assert "uses: arduino/setup-protoc@v3" in smoke_spec
    assert "uses: dtolnay/rust-toolchain@stable" in smoke_spec
    assert "uses: actions/setup-python@v5" in smoke_spec
    assert "uses: astral-sh/setup-uv@v4" in smoke_spec
    assert _workflow_run_steps(smoke_spec) == [
        "cargo build --release --locked",
        "make spec",
    ]


def test_makefile_spec_split_contract_is_documented_and_executable() -> None:
    makefile = _read_repo("Makefile")
    targeted_smoke_section_ids = [
        "spec/06-article.md::Getting Article Details",
        "spec/06-article.md::Article Batch",
        "spec/06-article.md::Article Query Echo Surfaces Explicit Max-Per-Source Overrides",
        "spec/06-article.md::Article Search Gene Keyword Pivot",
        "spec/06-article.md::Article Search Drug Keyword Pivot",
        "spec/06-article.md::Article Search Discover Keyword Pivot",
        "spec/09-search-all.md::Debug Plan",
        "spec/09-search-all.md::Distinct Disease And Keyword Stay Separate",
        "spec/17-cross-entity-pivots.md::Gene to Articles",
        "spec/17-cross-entity-pivots.md::Variant pivots",
    ]

    assert (
        ".PHONY: build test lint check check-quality-ratchet run clean spec spec-pr spec-smoke validate-skills test-contracts install"
        in makefile
    )
    assert "Volatile live-network spec headings." in makefile
    assert (
        "PR gate: repo-local checks plus live-backed headings that have been stable"
        in makefile
    )
    assert (
        "Smoke lane: `search article`, `gene articles`, `variant articles`," in makefile
    )
    assert (
        "To move a heading into the smoke lane, add its stable section ID" in makefile
    )
    assert "SPEC_PR_DESELECT_ARGS stores PR-lane deselects." in makefile
    assert "SPEC_SMOKE_ARGS stores" in makefile
    assert "stable targeted smoke section IDs" in makefile
    assert "resolves to current" in makefile
    assert "mustmatch pytest item IDs at runtime" in makefile
    assert "SPEC_PR_DESELECT_ARGS = \\" in makefile
    for node_id in [
        "spec/02-gene.md::Gene to Articles",
        "spec/03-variant.md::Variant to Articles",
        "spec/06-article.md::Searching by Gene",
        "spec/06-article.md::Searching by Keyword",
        "spec/06-article.md::Sort Behavior",
        "spec/07-disease.md::Disease to Articles",
        *targeted_smoke_section_ids,
    ]:
        assert f'--deselect "{node_id}"' in makefile
    spec_smoke_match = re.search(
        r"^SPEC_SMOKE_ARGS = \\\n(.*?)\n\nSPEC_SERIAL_FILES",
        makefile,
        flags=re.MULTILINE | re.DOTALL,
    )
    assert spec_smoke_match is not None
    assert (
        re.findall(
            r'^\t"([^"]+)"(?: \\)?$', spec_smoke_match.group(1), flags=re.MULTILINE
        )
        == targeted_smoke_section_ids
    )
    assert " (line " not in spec_smoke_match.group(1)
    assert " [bash]" not in spec_smoke_match.group(1)
    assert (
        "SPEC_SERIAL_FILES = spec/05-drug.md spec/13-study.md "
        "spec/21-cross-entity-see-also.md"
    ) in makefile
    assert "SPEC_XDIST_ARGS = -n auto --dist loadfile" in makefile
    assert re.search(r"^test:\n\tcargo nextest run$", makefile, flags=re.MULTILINE)
    assert re.search(
        r"^install:\n"
        r'\tmkdir -p "\$\(HOME\)/\.local/bin"\n'
        r"\tcargo build --release --locked\n"
        r'\tinstall -m 755 target/release/biomcp "\$\(HOME\)/\.local/bin/biomcp"$',
        makefile,
        flags=re.MULTILINE,
    )
    assert re.search(
        r"^spec:\n"
        r"\tXDG_CACHE_HOME=\"\$\(CURDIR\)/\.cache\" PATH=\"\$\(CURDIR\)/target/release:\$\(PATH\)\" BIOMCP_BIN=\"\$\(CURDIR\)/target/release/biomcp\" RUST_LOG=error \\\n"
        r"\t\tuv run --extra dev sh -c 'PATH=\"\$\(CURDIR\)/target/release:\$\$PATH\" BIOMCP_BIN=\"\$\(CURDIR\)/target/release/biomcp\" pytest spec/ --mustmatch-lang bash --mustmatch-timeout 120 -v \$\(SPEC_XDIST_ARGS\) --ignore spec/05-drug\.md --ignore spec/13-study\.md --ignore spec/21-cross-entity-see-also\.md'\n"
        r"\tXDG_CACHE_HOME=\"\$\(CURDIR\)/\.cache\" PATH=\"\$\(CURDIR\)/target/release:\$\(PATH\)\" BIOMCP_BIN=\"\$\(CURDIR\)/target/release/biomcp\" RUST_LOG=error \\\n"
        r"\t\tuv run --extra dev sh -c 'PATH=\"\$\(CURDIR\)/target/release:\$\$PATH\" BIOMCP_BIN=\"\$\(CURDIR\)/target/release/biomcp\" pytest \$\(SPEC_SERIAL_FILES\) --mustmatch-lang bash --mustmatch-timeout 120 -v'$",
        makefile,
        flags=re.MULTILINE,
    )
    assert re.search(
        r"^spec-pr:\n"
        r"\tXDG_CACHE_HOME=\"\$\(CURDIR\)/\.cache\" PATH=\"\$\(CURDIR\)/target/release:\$\(PATH\)\" BIOMCP_BIN=\"\$\(CURDIR\)/target/release/biomcp\" RUST_LOG=error \\\n"
        r"\t\tuv run --extra dev sh -c 'PATH=\"\$\(CURDIR\)/target/release:\$\$PATH\" BIOMCP_BIN=\"\$\(CURDIR\)/target/release/biomcp\" pytest spec/ --mustmatch-lang bash --mustmatch-timeout 60 -v \$\(SPEC_XDIST_ARGS\) \$\(SPEC_PR_DESELECT_ARGS\) --ignore spec/05-drug\.md --ignore spec/13-study\.md --ignore spec/21-cross-entity-see-also\.md'\n"
        r"\tXDG_CACHE_HOME=\"\$\(CURDIR\)/\.cache\" PATH=\"\$\(CURDIR\)/target/release:\$\(PATH\)\" BIOMCP_BIN=\"\$\(CURDIR\)/target/release/biomcp\" RUST_LOG=error \\\n"
        r"\t\tuv run --extra dev sh -c 'PATH=\"\$\(CURDIR\)/target/release:\$\$PATH\" BIOMCP_BIN=\"\$\(CURDIR\)/target/release/biomcp\" pytest \$\(SPEC_SERIAL_FILES\) --mustmatch-lang bash --mustmatch-timeout 60 -v'$",
        makefile,
        flags=re.MULTILINE,
    )
    spec_smoke_target = re.search(
        r"^spec-smoke:\n"
        r"\tXDG_CACHE_HOME=\"\$\(CURDIR\)/\.cache\" PATH=\"\$\(CURDIR\)/target/release:\$\(PATH\)\" BIOMCP_BIN=\"\$\(CURDIR\)/target/release/biomcp\" RUST_LOG=error \\\n"
        r"\t\tuv run --extra dev bash -lc 'set -euo pipefail; export PATH=\"\$\(CURDIR\)/target/release:\$\$PATH\"; export BIOMCP_BIN=\"\$\(CURDIR\)/target/release/biomcp\"; resolved_args=\"\$\$\(python tools/spec_smoke_args\.py --root-dir \"\$\(CURDIR\)\" --makefile \"\$\(CURDIR\)/Makefile\" --makefile-variable SPEC_SMOKE_ARGS\)\"; if \[\[ -z \"\$\$resolved_args\" \]\]; then echo \"SPEC_SMOKE_ARGS resolved to no pytest targets\" >&2; exit 1; fi; mapfile -t spec_smoke_args <<< \"\$\$resolved_args\"; pytest \"\$\${spec_smoke_args\[@\]}\" --mustmatch-lang bash --mustmatch-timeout 120 -v'$",
        makefile,
        flags=re.MULTILINE,
    )
    assert spec_smoke_target is not None
    assert "$(SPEC_XDIST_ARGS)" not in spec_smoke_target.group(0)
    assert "-n auto" not in spec_smoke_target.group(0)
    assert "$(SPEC_SMOKE_ARGS)" not in spec_smoke_target.group(0)
    assert "tools/spec_smoke_args.py" in spec_smoke_target.group(0)
    assert "mapfile -t spec_smoke_args" in spec_smoke_target.group(0)
    assert "eval" not in spec_smoke_target.group(0)
    assert re.search(
        r"^test-contracts:\n"
        r"\tcargo build --release --locked\n"
        r"\tuv sync --extra dev\n"
        r'\tuv run pytest tests/ -v --mcp-cmd "\./target/release/biomcp serve"\n'
        r"\tuv run mkdocs build --strict$",
        makefile,
        flags=re.MULTILINE,
    )


def test_repo_local_parallel_test_contract_is_documented() -> None:
    contributing = _read_repo("CONTRIBUTING.md")
    runbook = _read_repo("RUN.md")
    technical = _read_repo("architecture/technical/overview.md")
    contributing_ws = _normalize_ws(contributing)
    runbook_ws = _normalize_ws(runbook)
    technical_gate_section = _normalize_ws(
        _markdown_section(technical, "1. CI and Repo Gates", level=3)
    )
    technical_spec_section = _normalize_ws(
        _markdown_section(technical, "2. Spec Suite (`spec/`)", level=3)
    )

    assert "cargo install cargo-nextest --locked" in contributing
    assert "`make test` uses `cargo nextest run`." in contributing_ws
    assert "`make spec` and `make spec-pr` use `pytest-xdist`" in contributing_ws
    assert "`make spec-smoke` resolves the ten stable targeted smoke" in contributing_ws
    assert (
        "section IDs in `SPEC_SMOKE_ARGS` to current mustmatch pytest item IDs"
        in contributing_ws
    )
    assert (
        "`spec/05-drug.md`, `spec/13-study.md`, and `spec/21-cross-entity-see-also.md` stay serial"
        in contributing_ws
    )
    assert "beelink" in contributing
    assert "2026-04-13" in contributing
    assert "/usr/bin/time -p" in contributing
    assert "warm-cache steady-state" in contributing_ws
    assert "| Command | Before | After |" in contributing
    assert "".join(("T", "BD")) not in contributing
    for command in (
        "make test",
        "make spec-pr",
        "make check",
    ):
        assert re.search(
            rf"^\| `{re.escape(command)}` \| `\d+\.\d+s` \| `\d+\.\d+s` \|$",
            contributing,
            flags=re.MULTILINE,
        )

    assert "cargo-nextest" in runbook
    assert "`make test` and `make check`" in runbook_ws
    assert "`cargo nextest run`" in runbook_ws
    assert "`make spec-pr`" in runbook_ws
    assert "`make spec-smoke`" in runbook_ws
    assert "`pytest-xdist`" in runbook_ws
    assert "`-n auto --dist loadfile`" in runbook_ws
    assert "ten stable smoke section IDs in `SPEC_SMOKE_ARGS`" in runbook_ws
    assert (
        "resolves those IDs to current mustmatch pytest item IDs at runtime"
        in runbook_ws
    )
    assert (
        "`spec/05-drug.md`, `spec/13-study.md`, and `spec/21-cross-entity-see-also.md`"
        in runbook_ws
    )

    assert "`cargo nextest run`" in technical_gate_section
    assert "`cargo test`" in technical_gate_section
    assert "`pytest-xdist` with `-n auto --dist loadfile`" in technical_spec_section
    assert "`make spec-smoke`" in technical_spec_section
    assert "serial targeted local rerun" in technical_spec_section
    assert "ten stable smoke section IDs in `SPEC_SMOKE_ARGS`" in technical_spec_section
    assert "resolves stable targeted smoke section IDs" in technical_spec_section
    assert (
        "`spec/05-drug.md`, `spec/13-study.md`, and `spec/21-cross-entity-see-also.md`"
        in technical_spec_section
    )


def test_spec_lane_timing_report_is_documented_and_aligned_with_makefile() -> None:
    makefile = _read_repo("Makefile")
    report = _read_repo("spec/README-timings.md")
    runbook = _read_repo("RUN.md")
    technical = _read_repo("architecture/technical/overview.md")

    lane_split_section = _normalize_ws(_markdown_section(report, "Spec Lane Split"))
    audit_method_section = _normalize_ws(_markdown_section(report, "Audit Method"))
    timing_audit_section = _markdown_section(report, "spec-pr Timing Audit")
    smoke_only_section = _markdown_section(
        report, "Smoke-Only Headings (SPEC_PR_DESELECT_ARGS)"
    )
    timing_audit_rows = _markdown_table_rows(timing_audit_section)
    smoke_only_rows = _markdown_table_rows(smoke_only_section)
    runbook_spec_section = _normalize_ws(_markdown_section(runbook, "Spec Suite"))
    technical_spec_section = _normalize_ws(
        _markdown_section(technical, "2. Spec Suite (`spec/`)", level=3)
    )

    for heading in (
        "## Spec Lane Split",
        "## Audit Method",
        "## spec-pr Timing Audit",
        "## Smoke-Only Headings (SPEC_PR_DESELECT_ARGS)",
    ):
        assert heading in report

    for marker in (
        "`make spec-pr`",
        "`make spec`",
        "`make spec-smoke`",
        "`make test-contracts`",
    ):
        assert marker in lane_split_section
    assert "First-pass" in audit_method_section
    assert "Warm-pass" in audit_method_section
    assert (
        "| File | Heading | First-pass Time | First-pass Result | Warm-pass Time | Warm-pass Result | Category | Disposition | Rationale |"
        in report
    )
    assert timing_audit_rows[0] == [
        "File",
        "Heading",
        "First-pass Time",
        "First-pass Result",
        "Warm-pass Time",
        "Warm-pass Result",
        "Category",
        "Disposition",
        "Rationale",
    ]
    assert smoke_only_rows[0] == ["Node ID", "Reason"]

    deselected_node_ids = re.findall(r'--deselect "([^"]+)"', makefile)
    assert deselected_node_ids
    reported_node_ids = [row[0].strip("`") for row in smoke_only_rows[1:]]
    assert reported_node_ids
    assert len(reported_node_ids) == len(set(reported_node_ids))
    assert set(reported_node_ids) == set(deselected_node_ids)

    for row in timing_audit_rows[1:]:
        first_pass_time = row[2].strip("`")
        first_pass_result = row[3]
        warm_pass_time = row[4].strip("`")
        warm_pass_result = row[5]
        category = row[6]

        if first_pass_result == "passed":
            assert re.fullmatch(r"\d+\.\d+s", first_pass_time)
        if warm_pass_result == "passed":
            assert re.fullmatch(r"\d+\.\d+s", warm_pass_time)
        if category == "gated":
            assert first_pass_result == "skipped"
            assert warm_pass_result == "skipped"

    assert "spec/README-timings.md" in runbook_spec_section
    assert "`make spec-smoke`" in runbook_spec_section
    assert (
        "stable targeted smoke section IDs in `SPEC_SMOKE_ARGS`" in runbook_spec_section
    )
    assert "resolve to collectable mustmatch pytest items" in runbook_spec_section
    assert "spec/README-timings.md" in technical_spec_section
    assert "`make spec-smoke`" in technical_spec_section
    assert "stable targeted smoke section IDs" in technical_spec_section
    assert "resolve to collectable mustmatch pytest items" in technical_spec_section
    assert any(
        row[0] == "`spec/18-source-labels.md`"
        and row[1] == "`Article Fulltext Resolver Order`"
        and row[7] == "keep in spec-pr"
        for row in timing_audit_rows[1:]
    )
    assert "`spec/18-source-labels.md::Article Fulltext Resolver Order`" not in (
        row[0] for row in smoke_only_rows[1:]
    )


def test_parallel_test_dependency_contract_is_declared() -> None:
    pyproject = tomllib.loads(_read_repo("pyproject.toml"))
    uv_lock = _read_repo("uv.lock")

    import xdist

    dev_dependencies = pyproject["project"]["optional-dependencies"]["dev"]

    assert "pytest-xdist" in dev_dependencies
    assert '{ name = "pytest-xdist", marker = "extra == \'dev\'" }' in uv_lock
    assert 'name = "pytest-xdist"' in uv_lock
    assert xdist.__file__ is not None


def test_runtime_contract_docs_and_scripts_align_on_release_target() -> None:
    staging_demo = _read_repo("architecture/technical/staging-demo.md")
    runbook = _read_repo("RUN.md")
    technical = _read_repo("architecture/technical/overview.md")
    scripts_readme = _read_repo("scripts/README.md")
    source_contracts = _read_repo("scripts/source-contracts.md")
    contract_smoke = _read_repo("scripts/contract-smoke.sh")
    genegpt_demo = _read_repo("scripts/genegpt-demo.sh")
    geneagent_demo = _read_repo("scripts/geneagent-demo.sh")

    assert "# BioMCP Staging and Demo Contract" in staging_demo
    assert "./target/release/biomcp" in staging_demo
    assert "shared merged-main target" in staging_demo
    assert (
        "BIOMCP_BIN=./target/release/biomcp ./scripts/genegpt-demo.sh" in staging_demo
    )
    assert (
        "BIOMCP_BIN=./target/release/biomcp ./scripts/geneagent-demo.sh" in staging_demo
    )
    assert "./scripts/contract-smoke.sh --fast" in staging_demo
    assert (
        'uv run pytest tests/test_mcp_contract.py -v --mcp-cmd "./target/release/biomcp serve"'
        in staging_demo
    )
    assert "ONCOKB_TOKEN" in staging_demo
    assert (
        "./target/release/biomcp serve-http --host 127.0.0.1 --port 8080"
        in staging_demo
    )
    assert "POST/GET /mcp" in staging_demo
    assert "GET /health" in staging_demo
    assert "GET /readyz" in staging_demo
    assert "GET /" in staging_demo
    assert "tests/test_mcp_http_transport.py" in staging_demo
    assert "S2_API_KEY" in staging_demo
    assert "article citations 22663011 --limit 3" in staging_demo

    assert "# BioMCP Runbook" in runbook
    assert "cargo build --release --locked" in runbook
    assert "./target/release/biomcp serve" in runbook
    assert "./target/release/biomcp serve-http --host 127.0.0.1 --port 8080" in runbook
    assert (
        'uv run pytest tests/test_mcp_contract.py -v --mcp-cmd "./target/release/biomcp serve"'
        in runbook
    )
    assert "curl http://127.0.0.1:8080/health" in runbook
    assert "curl http://127.0.0.1:8080/readyz" in runbook
    assert "curl http://127.0.0.1:8080/" in runbook
    assert "tests/test_mcp_http_surface.py" in runbook
    assert "tests/test_mcp_http_transport.py" in runbook
    assert "make spec" in runbook
    assert "make test-contracts" in runbook
    assert "S2_API_KEY" in runbook
    assert "./target/release/biomcp article citations 22663011 --limit 3" in runbook
    assert (
        "`make test-contracts` runs `cargo build --release --locked`, "
        '`uv sync --extra dev`, `pytest tests/ -v --mcp-cmd "./target/release/biomcp serve"`, '
        "and `mkdocs build --strict` - the same steps that PR CI `contracts` requires."
        in runbook
    )
    assert "docs/user-guide/cli-reference.md" in runbook
    assert "docs/reference/mcp-server.md" in runbook

    assert "architecture/technical/staging-demo.md" in technical
    assert "RUN.md" in technical
    assert "S2_API_KEY" in technical
    assert "Semantic Scholar article enrichment/navigation" in technical
    assert "No `RUN.md` or staging-demo runbook exists" not in technical

    assert "lightweight commands for checking upstream source" in scripts_readme
    assert "source-facing contract probe" in scripts_readme
    assert "091 expansion scope" not in scripts_readme

    assert "# BioMCP Source Contract Probes" in source_contracts
    assert "source-facing API contract probes" in source_contracts
    assert "ONCOKB_TOKEN" in source_contracts
    assert "Semantic Scholar" in source_contracts
    assert "S2_API_KEY" in source_contracts
    assert "091 expansion scope" not in source_contracts

    assert "ONCOKB_TOKEN" in contract_smoke
    assert "ONCOKB_API_TOKEN" in contract_smoke
    assert "S2_API_KEY" in contract_smoke
    assert "Semantic Scholar" in contract_smoke
    assert "set ONCOKB_TOKEN to enable" in contract_smoke

    for demo_script in (genegpt_demo, geneagent_demo):
        assert 'BIN="${BIOMCP_BIN:-' in demo_script
        assert "$ROOT/target/release/biomcp" in demo_script
        assert "target/debug/biomcp" not in demo_script
        assert "command -v biomcp >/dev/null 2>&1" in demo_script


def test_validation_profile_and_hook_contract_docs_are_pinned() -> None:
    runbook = _read_repo("RUN.md")
    contributing = _read_repo("CONTRIBUTING.md")
    technical = _read_repo("architecture/technical/overview.md")
    runbook_prerequisites = _normalize_ws(_markdown_section(runbook, "Prerequisites"))
    runbook_premerge = _normalize_ws(_markdown_section(runbook, "Pre-Merge Checks"))
    contributing_hook = _normalize_ws(
        _markdown_section(contributing, "Local Pre-Commit Hook", level=3)
    )
    ci_gate_section = _normalize_ws(
        _markdown_section(technical, "1. CI and Repo Gates", level=3)
    )
    march_profiles = _normalize_ws(
        _markdown_section(technical, "March Validation Profiles", level=4)
    )

    assert "pre-commit hook" in runbook_premerge
    assert "`scripts/pre-commit-reject-march-artifacts.sh`" in runbook_premerge
    assert "`cargo fmt --check`" in runbook_premerge
    assert "`cargo clippy --lib --tests -- -D warnings`" in runbook_premerge
    assert (
        "`cargo-deny` for the repo-local license and advisory policy checks in "
        "`make check`" in runbook_prerequisites
    )
    assert "does not run" in runbook_premerge
    assert "`cargo nextest run`" in runbook_premerge
    assert "`make check`" in runbook_premerge
    assert "`cargo deny check licenses`" in runbook_premerge
    assert "`cargo deny check advisories`" in runbook_premerge
    assert "`make spec-pr`" in runbook_premerge
    assert "`make test-contracts`" in runbook_premerge
    assert "git commit --no-verify" in runbook_premerge
    for allowed_path in (
        ".march/code-review-log.md",
        ".march/validation-profiles.toml",
    ):
        assert allowed_path in runbook_premerge
        assert allowed_path in contributing_hook

    assert "opt in" in contributing_hook
    assert "does not install it automatically" in contributing_hook
    assert "`$(git rev-parse --git-path hooks/pre-commit)`" in contributing_hook
    assert "`scripts/pre-commit-reject-march-artifacts.sh`" in contributing_hook
    assert "`cargo fmt --check`" in contributing_hook
    assert "`cargo clippy --lib --tests -- -D warnings`" in contributing_hook
    assert "staged deletions" in contributing_hook

    assert ".march/validation-profiles.toml" in ci_gate_section
    assert (
        "`01-design` and `02-design-review` without a validation profile"
        in ci_gate_section
    )
    assert ".march/validation-profiles.toml" in march_profiles
    assert ".march/code-review-log.md" in march_profiles
    assert ".march/verify-log.md" not in march_profiles
    assert ".march/blueprint.md" not in march_profiles
    assert "`.march/` remains ignored by `.gitignore`" in march_profiles
    assert "Python cleanup contract" in march_profiles
    assert "pre-commit helper" in march_profiles
    for row in (
        "| `preflight` | `cargo check --all-targets` | `kickoff` |",
        "| `baseline` | `cargo check --all-targets` | declared, not assigned |",
        "| `focused` | `cargo test --lib && cargo clippy --lib --tests -- -D warnings` | `03-code`, `04-code-review` |",
        "| `full-blocking` | `make check && make spec-pr` | `05-verify` |",
        "| `full-contracts` | `make check && make spec-pr && make test-contracts` | declared, not assigned |",
    ):
        assert row in ci_gate_section
    assert (
        "`full-blocking` deliberately uses `make check && make spec-pr`, not full `make spec`"
        in ci_gate_section
    )
    assert (
        "`full-contracts` is declared for tickets that need the contracts lane, but the shared build flow does not assign it today"
        in ci_gate_section
    )
