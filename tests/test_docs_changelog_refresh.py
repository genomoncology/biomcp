from __future__ import annotations

import re
from pathlib import Path
import tomllib

REPO_ROOT = Path(__file__).resolve().parents[1]
EXPECTED_RELEASE_TICKETS = {
    "0.8.22": {
        232,
        233,
        235,
        236,
        237,
        238,
        239,
        240,
        242,
        252,
        253,
        *range(264, 285),
        286,
        287,
        288,
        291,
    }
}
EXPECTED_RELEASE_MARKERS = {
    "0.8.22": {
        "fixes": [
            "compact diagnostic rows",
            "capped disease diagnostic pivots",
            "live-valid GTR example",
            "zero-result recovery",
            "entity-aware article follow-ups",
            "same-session",
            "PubMed ESearch",
            "resistance-to-drug mechanism questions",
        ],
        "new_features": [
            "`biomcp suggest <question>`",
            "_meta.workflow",
            "_meta.ladder[]",
            "workflow-ladder sidecars",
        ],
        "docs": [
            "architecture",
            "source-integration",
            "backtick quoting",
            "BioASQ benchmark",
            "canonical `SKILL.md`",
        ],
        "internal": [
            "release/docs contract cleanup",
            "public Python/docs contract lane",
            "make release-gate",
            "SPEC_SMOKE_ARGS",
            "mustmatch pytest item IDs",
            ".march",
            "no-go",
            "no runtime wiring",
        ],
    }
}


def _read(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def _citation_scalar(field_name: str) -> str:
    citation = _read("CITATION.cff")
    match = re.search(rf"^{re.escape(field_name)}:\s*(.+)$", citation, re.MULTILINE)
    assert match is not None, f"missing {field_name} in CITATION.cff"
    return match.group(1).strip().strip('"')


def _current_release_version() -> str:
    cargo = tomllib.loads(_read("Cargo.toml"))
    return cargo["package"]["version"]


def _current_release_heading() -> str:
    return f"## {_current_release_version()} — {_citation_scalar('date-released')}"


def _current_release_tag_example() -> str:
    return f"v{_current_release_version()}"


def _markdown_section_block(text: str, heading: str) -> str:
    start = text.index(heading)
    remainder = text[start + len(heading) :]
    next_heading = remainder.find("\n## ")
    if next_heading == -1:
        return remainder
    return remainder[:next_heading]


def _markdown_subsection_block(text: str, heading: str) -> str:
    start = text.index(heading)
    remainder = text[start + len(heading) :]
    next_heading = remainder.find("\n### ")
    if next_heading == -1:
        return remainder
    return remainder[:next_heading]


def _ticket_references(text: str) -> set[int]:
    references: set[int] = set()
    for group in re.findall(r"\(([^)]+)\)", text):
        for token in group.split(","):
            token = token.strip()
            if token.isdigit():
                references.add(int(token))
    return references


def test_changelog_has_backfilled_releases_and_release_header() -> None:
    changelog = _read("CHANGELOG.md")
    current_release_version = _current_release_version()
    current_release_heading = _current_release_heading()

    assert current_release_version in EXPECTED_RELEASE_TICKETS
    assert current_release_version in EXPECTED_RELEASE_MARKERS

    latest_release_block = _markdown_section_block(changelog, current_release_heading)
    previous_release_block = _markdown_section_block(
        changelog, "## 0.8.21 — 2026-04-16"
    )
    latest_new_features_block = _markdown_subsection_block(
        latest_release_block, "### New features"
    )
    latest_docs_block = _markdown_subsection_block(latest_release_block, "### Docs")
    latest_fixes_block = _markdown_subsection_block(latest_release_block, "### Fixes")
    assert "### Internal" in latest_release_block
    latest_internal_block = _markdown_subsection_block(
        latest_release_block, "### Internal"
    )
    previous_new_features_block = _markdown_subsection_block(
        previous_release_block, "### New features"
    )

    assert "## [Unreleased]" not in changelog
    assert current_release_heading in changelog
    assert "## 0.8.21 — 2026-04-16" in changelog
    assert changelog.index(current_release_heading) < changelog.index("## 0.8.21 — 2026-04-16")
    assert "## 0.8.20 — 2026-03-30" in changelog
    assert "## 0.8.19 — 2026-03-26" in changelog
    assert "## 0.8.18 — 2026-03-25" in changelog
    assert "## 0.9.0" not in changelog

    assert "article date-range filtering" in previous_release_block
    assert "Expanded trial search with drug alias union" in previous_release_block
    assert "counts-only contract" in previous_release_block
    assert "ClinicalTrials.gov fallback" in previous_release_block
    assert "Refreshed architecture docs" in previous_release_block
    assert "daraxonrasib six-commands workflow" in previous_release_block
    assert "mustmatch" in previous_release_block
    assert "test_support" in previous_release_block
    assert "docs-only validation profile" in previous_release_block
    assert "RustSec" in previous_release_block
    assert "test-contracts" in previous_release_block
    assert "EMA regulatory region" in previous_new_features_block
    assert "--region eu" in previous_new_features_block
    assert "biomcp ema sync" in previous_new_features_block
    assert _ticket_references(previous_release_block) == {182, *range(193, 214), 221}

    assert "WHO Prequalification drug coverage" in latest_release_block
    assert "--product-type api" in latest_release_block
    assert "CDC CVX/MVX vaccine identity bridge" in latest_release_block
    assert "biomcp cvx sync" in latest_new_features_block
    assert "CDC WONDER VAERS" in latest_release_block
    assert "--source <faers|vaers|all>" in latest_release_block
    assert "local GTR-backed `diagnostic` entity" in latest_release_block
    assert "WHO Prequalified IVD" in latest_release_block
    assert "biomcp gtr sync" in latest_new_features_block
    assert "biomcp who-ivd sync" in latest_release_block
    assert "FDA 510(k) and PMA regulatory overlays" in latest_release_block
    assert "get diagnostic <id> regulatory" in latest_release_block
    assert (
        "Added opt-in diagnostic pivots to gene and disease cards"
        in latest_release_block
    )
    assert "`get gene <symbol> diagnostics`" in latest_release_block
    assert "`get disease <name_or_id> diagnostics`" in latest_release_block
    assert "backed by local GTR and WHO IVD diagnostic search." in latest_release_block
    assert "cross-entity keyword hints" in latest_release_block
    assert "_meta.next_commands" in latest_release_block
    assert "See also:" in latest_release_block
    assert "clinical_features" in latest_release_block
    assert "MedlinePlus" in latest_release_block
    assert "BioASQ-oriented how-to routing rows" in latest_docs_block
    assert "skills/SKILL.md" in latest_docs_block
    assert "public landing-copy contract" in latest_docs_block
    assert "workflow landing bullets" in latest_docs_block
    assert "SPEC_SMOKE_ARGS" in latest_release_block
    assert "current mustmatch pytest item IDs" in latest_release_block
    assert "quality ratchet" in latest_release_block
    assert _ticket_references(latest_release_block) == EXPECTED_RELEASE_TICKETS[
        current_release_version
    ]
    for marker in EXPECTED_RELEASE_MARKERS[current_release_version]["fixes"]:
        assert marker in latest_fixes_block
    for marker in EXPECTED_RELEASE_MARKERS[current_release_version]["new_features"]:
        assert marker in latest_new_features_block
    for marker in EXPECTED_RELEASE_MARKERS[current_release_version]["docs"]:
        assert marker in latest_docs_block
    for marker in EXPECTED_RELEASE_MARKERS[current_release_version]["internal"]:
        assert marker in latest_internal_block
    assert "pending separate merge" not in latest_release_block

    expected_releases = [
        ("0.8.20", "2026-03-30"),
        ("0.8.18", "2026-03-25"),
        ("0.8.17", "2026-03-23"),
        ("0.8.16", "2026-03-17"),
        ("0.8.15", "2026-03-11"),
        ("0.8.14", "2026-03-10"),
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


def test_remote_http_docs_are_promoted_for_newcomers() -> None:
    readme = _read("README.md")
    docs_index = _read("docs/index.md")
    mkdocs = _read("mkdocs.yml")
    remote_http = _read("docs/getting-started/remote-http.md")
    demo_readme = _read("examples/streamable-http/README.md")

    assert "### Remote HTTP server" in readme
    assert "biomcp serve-http --host 127.0.0.1 --port 8080" in readme
    assert "http://127.0.0.1:8080/mcp" in readme
    assert "examples/streamable-http/streamable_http_client.py" in readme
    assert "https://biomcp.org/getting-started/remote-http/" in readme
    assert readme.index("### Remote HTTP server") < readme.index(
        "## Multi-worker deployment"
    )

    assert "### Remote HTTP server" in docs_index
    assert "biomcp serve-http --host 127.0.0.1 --port 8080" in docs_index
    assert "http://127.0.0.1:8080/mcp" in docs_index
    assert "`/health`, `/readyz`, and `/`" in docs_index
    assert "getting-started/remote-http.md" in docs_index
    assert "examples/streamable-http/streamable_http_client.py" in docs_index

    assert "Remote HTTP Server: getting-started/remote-http.md" in mkdocs

    assert "# Remote Streamable HTTP Server" in remote_http
    assert "Use `biomcp serve-http` when you need one shared MCP server" in remote_http
    assert "Use `biomcp serve` when a single local client" in remote_http
    assert "biomcp serve-http --host 127.0.0.1 --port 8080" in remote_http
    assert "`/mcp`" in remote_http
    assert "`/health`" in remote_http
    assert "`/readyz`" in remote_http
    assert "streamable_http_client" in remote_http
    assert "terminate_on_close=False" in remote_http
    assert "examples/streamable-http/streamable_http_client.py" in remote_http
    assert "three-step BRAF V600E melanoma" in remote_http
    assert "workflow over the remote MCP `biomcp` tool" in remote_http
    assert "prints `Command: ...` before each BioMCP step" in remote_http
    assert (
        "biomcp search all --gene BRAF --disease melanoma --counts-only" in remote_http
    )
    assert 'biomcp get variant "BRAF V600E" clinvar' in remote_http
    assert (
        'biomcp search trial -c melanoma --mutation "BRAF V600E" --limit 5'
        in remote_http
    )
    assert "examples/streamable-http/README.md" in remote_http
    assert "--scenario braf-melanoma" not in remote_http
    assert "Available tools:" not in remote_http

    assert "# Streamable HTTP Demo" in demo_readme
    assert "what the demo proves" in demo_readme.lower()
    assert "how to start the server" in demo_readme.lower()
    assert "how to run the client" in demo_readme.lower()
    assert "what output to expect" in demo_readme.lower()
    assert (
        "uv run --quiet --script examples/streamable-http/streamable_http_client.py"
        in demo_readme
    )
    assert (
        "./target/release/biomcp serve-http --host 127.0.0.1 --port 8080" in demo_readme
    )
    assert "http://127.0.0.1:8080/mcp" in demo_readme
    assert (
        "Command: biomcp search all --gene BRAF --disease melanoma --counts-only"
        in demo_readme
    )
    assert 'Command: biomcp get variant "BRAF V600E" clinvar' in demo_readme
    assert (
        'Command: biomcp search trial -c melanoma --mutation "BRAF V600E" --limit 5'
        in demo_readme
    )
    assert "--scenario braf-melanoma" not in demo_readme
    assert "Health check passed:" not in demo_readme
    assert "Available tools:" not in demo_readme


def test_diagnostic_docs_and_count_language_are_current() -> None:
    readme = _read("README.md")
    docs_index = _read("docs/index.md")
    cli_reference = _read("docs/user-guide/cli-reference.md")
    disease_guide = _read("docs/user-guide/disease.md")
    functional = _read("architecture/functional/overview.md")
    ux_reference = _read("architecture/ux/cli-reference.md")
    mkdocs = _read("mkdocs.yml")
    diagnostic_guide = _read("docs/user-guide/diagnostic.md")
    diagnostic_arch = _read("architecture/functional/diagnostic.md")
    cross_entity_guide = _read("docs/how-to/cross-entity-pivots.md")

    for text in (readme, docs_index):
        assert "13 remote entity commands" not in text
        assert "all 13 remote entity commands" not in text
        assert "12 remote entity commands" not in text
        assert (
            "| diagnostic | NCBI Genetic Testing Registry local bulk bundle + WHO IVD local CSV + optional OpenFDA device overlay |"
            in text
        )
        assert "GTR-backed diagnostics pivot" in text
        assert "MedlinePlus `clinical_features`" in text
        assert "GTR/WHO IVD diagnostics pivot" in text
        assert (
            "OpenFDA FAERS/MAUDE/recalls plus CDC WONDER VAERS aggregate vaccine search"
            in text
        )
        assert "public entity surface" in text

    assert "biomcp gtr sync" in cli_reference
    assert "biomcp who-ivd sync" in cli_reference
    assert "GTR local data" in cli_reference
    assert "WHO IVD local data" in cli_reference
    assert "matches complete disease words or phrases at boundaries" in cli_reference
    assert "Disease diagnostic cards are capped at" in cli_reference
    assert 'biomcp get disease "uterine leiomyoma" clinical_features' in cli_reference
    assert (
        "`clinical_features`, `diagnostics`, `disgenet`, and `funding` stay opt-in"
        in cli_reference
    )
    assert "13 remote entity commands" not in cli_reference
    assert "all 13 remote entity commands" not in cli_reference
    assert "12 remote entity commands" not in cli_reference

    assert "13 remote entity commands" not in functional
    assert "all 13 remote entity commands" not in functional
    assert "12 entity types" not in functional
    assert "15+ biomedical databases" not in functional
    assert (
        "| diagnostic | NCBI Genetic Testing Registry local bulk exports, WHO IVD local CSV, optional OpenFDA device 510(k)/PMA overlay |"
        in functional
    )
    assert "MedlinePlus `clinical_features`" in functional
    assert "CDC WONDER VAERS aggregate vaccine search" in functional

    assert "biomcp gtr sync" in ux_reference
    assert "biomcp who-ivd sync" in ux_reference
    assert 'biomcp get disease "uterine leiomyoma" clinical_features' in ux_reference
    assert "all 13 remote entity commands" not in ux_reference
    assert "13 remote entity commands" not in ux_reference
    assert "all 12 entity types" not in ux_reference

    assert "Diagnostic: user-guide/diagnostic.md" in mkdocs
    assert "WHO Prequalified IVD: sources/who-ivd.md" in mkdocs
    assert "15 biomedical databases" not in mkdocs

    assert diagnostic_guide.startswith("# Diagnostic")
    assert "biomcp search diagnostic --gene BRCA1 --limit 5" in diagnostic_guide
    assert (
        "biomcp search diagnostic --disease HIV --source who-ivd --limit 5"
        in diagnostic_guide
    )
    assert "`--disease` is a bounded disease phrase filter" in diagnostic_guide
    assert "must contain at least three" in diagnostic_guide
    assert "alphanumeric characters" in diagnostic_guide
    assert "Disease diagnostic cards show at most 10 rows" in diagnostic_guide
    assert (
        "biomcp search diagnostic --disease tuberculosis --source all --limit 50"
        in diagnostic_guide
    )
    assert "`Genes` and `Conditions` cells at five displayed values" in diagnostic_guide
    assert "full deduped symbol arrays" in diagnostic_guide
    assert 'biomcp get diagnostic "ITPW02232- TC40"' in diagnostic_guide
    assert "biomcp gtr sync" in diagnostic_guide
    assert "biomcp who-ivd sync" in diagnostic_guide
    assert diagnostic_arch.startswith("# Diagnostic Functional Note")
    assert "minimum-length" in diagnostic_arch
    assert "word/phrase boundary match" in diagnostic_arch
    assert "## Disease Diagnostic Pivot Contract" in diagnostic_arch
    diagnostic_pivot_contract = _markdown_section_block(
        diagnostic_arch, "## Disease Diagnostic Pivot Contract"
    )
    diagnostic_pivot_contract_text = re.sub(r"\s+", " ", diagnostic_pivot_contract)
    for expected in (
        "`get disease <name_or_id> diagnostics`",
        "opt-in only",
        "excluded from `all`",
        "`disease_query_value()`",
        "`source=All`",
        "`limit=10`",
        "`offset=0`",
        "at least three alphanumeric characters",
        "word/phrase boundary matching",
        "No MONDO/OLS traversal, synonym expansion",
        "The top hit is only the first row",
        "deterministic ordering",
        "normalized diagnostic display name",
        "accession",
        "pagination",
        "10-row cap",
        "40 KB ceiling",
        "`DiagnosticSearchResult`",
        "`Genes` and `Conditions`",
        "five values",
        "JSON keeps the full arrays",
        "`get diagnostic <id>`",
        "true no-match",
        "`diagnostics = Some(Vec::new())`",
        "`diagnostics = None`",
        "local diagnostic data unavailable",
        "source-level dedupe",
        "no cross-source diagnostic-row dedupe",
        "read-only",
        "MCP-safe",
        "shell-quoted",
        "`search diagnostic --disease ... --source all --limit 50`",
        "`spec/07-disease.md`",
        "`spec/17-cross-entity-pivots.md`",
        "`spec/21-cross-entity-see-also.md`",
        "`spec/24-diagnostic.md`",
        "`src/entities/diagnostic/search.rs::disease_phrase_matches_accepts_word_and_phrase_boundaries`",
        "`src/entities/diagnostic/search.rs::disease_phrase_matches_rejects_partial_words_and_keeps_scanning`",
        "`src/entities/diagnostic/search.rs::normalized_filters_reject_short_disease_filter`",
        "`src/entities/diagnostic/mod.rs::search_page_rejects_short_disease_filter`",
        "`src/entities/diagnostic/mod.rs::search_page_disease_filter_requires_word_boundary`",
        "`src/entities/diagnostic/mod.rs::search_page_applies_conjunctive_filters_and_stable_ordering`",
        "`src/entities/diagnostic/mod.rs::search_page_all_source_uses_unknown_total_when_both_sources_match`",
        "`src/entities/diagnostic/mod.rs::get_diagnostic_genes_returns_full_deduped_broad_panel_list`",
        "`src/entities/disease/enrichment/tests.rs::disease_diagnostics_section_populates_from_who_fixture`",
        "`src/entities/disease/enrichment/tests.rs::disease_diagnostics_unavailable_sets_note`",
        "`src/entities/disease/get/tests.rs::disease_parse_sections_all_keeps_diagnostics_opt_in`",
        "`src/entities/disease/get/tests.rs::parse_sections_all_keeps_optional_sections_opt_in`",
        "`src/render/markdown/disease/tests/rendering.rs::disease_markdown_renders_diagnostics_note_then_shell_safe_search_command`",
        "`src/render/markdown/diagnostic/tests.rs::diagnostic_search_rows_caps_genes_and_conditions_with_overflow_marker`",
    ):
        assert expected in diagnostic_pivot_contract_text
    assert "The disease diagnostic card is capped at 10 rows" in disease_guide
    assert (
        "biomcp search diagnostic --disease tuberculosis --source all --limit 50"
        in disease_guide
    )
    assert (
        "It is capped at 10 rows and prints a `See also:` command" in cross_entity_guide
    )


def test_streamable_http_demo_script_is_runnable_repo_artifact() -> None:
    demo_script = _read("examples/streamable-http/streamable_http_client.py")

    assert demo_script.startswith("#!/usr/bin/env -S uv run --script")
    assert '# requires-python = ">=3.11"' in demo_script
    assert '"mcp>=' in demo_script
    assert "biomcp serve-http --host 127.0.0.1 --port 8080" in demo_script
    assert (
        "uv run --script examples/streamable-http/streamable_http_client.py"
        in demo_script
    )
    assert 'DEFAULT_BASE_URL = "http://127.0.0.1:8080"' in demo_script
    assert "mcp_url = f\"{base_url.rstrip('/')}/mcp\"" in demo_script
    assert "def resolve_base_url(argv: list[str]) -> str:" in demo_script
    assert "resolve_base_url(sys.argv)" in demo_script
    assert (
        "Usage: examples/streamable-http/streamable_http_client.py [base_url]"
        in demo_script
    )
    assert "terminate_on_close=False" in demo_script
    assert '"biomcp"' in demo_script
    assert "shell" not in demo_script
    assert 'SCENARIO = "braf-melanoma"' not in demo_script
    assert (
        '"biomcp search all --gene BRAF --disease melanoma --counts-only"'
        in demo_script
    )
    assert 'biomcp get variant "BRAF V600E" clinvar' in demo_script
    assert (
        'biomcp search trial -c melanoma --mutation "BRAF V600E" --limit 5'
        in demo_script
    )
    assert 'print(f"Command: {command}")' in demo_script
    assert "argparse" not in demo_script
    assert "check_health" not in demo_script
    assert "list_tools()" not in demo_script
    assert "--scenario" not in demo_script


def test_release_overview_describes_streamable_http_workflow_demo() -> None:
    overview = _read("architecture/technical/overview.md")

    assert "standalone Streamable HTTP demo client" in overview
    assert "three-step" in overview
    assert "discovery -> evidence -> melanoma trials workflow" in overview
    assert "lists tools" not in overview
    assert "biomcp version" not in overview
    assert "Health check passed:" not in overview
    assert "Command:" in overview
    assert "biomcp search all --gene BRAF --disease melanoma --counts-only" in overview
    assert 'biomcp get variant "BRAF V600E" clinvar' in overview
    assert (
        'biomcp search trial -c melanoma --mutation "BRAF V600E" --limit 5' in overview
    )


def test_latest_changelog_documents_mcp_tool_rename() -> None:
    changelog = _read("CHANGELOG.md")
    v0_8_16_block = _markdown_section_block(changelog, "## 0.8.16 — 2026-03-17")
    v0_8_15_block = _markdown_section_block(changelog, "## 0.8.15 — 2026-03-11")

    assert "MCP execution tool" in v0_8_16_block
    assert "`shell`" in v0_8_16_block
    assert "`biomcp`" in v0_8_16_block
    assert "MCP execution tool" not in v0_8_15_block


def test_changelog_audit_backfills_rust_release_gaps() -> None:
    changelog = _read("CHANGELOG.md")
    v0_8_11_block = _markdown_section_block(changelog, "## 0.8.11 — 2026-03-06")
    v0_8_14_block = _markdown_section_block(changelog, "## 0.8.14 — 2026-03-10")
    v0_8_16_block = _markdown_section_block(changelog, "## 0.8.16 — 2026-03-17")
    v0_8_17_block = _markdown_section_block(changelog, "## 0.8.17 — 2026-03-23")

    assert "Added reusable presentations infrastructure with an intro deck" in (
        v0_8_11_block
    )
    assert "Hardened PyPI release packaging for arm64" in v0_8_11_block

    assert "Reranked disease search results" in v0_8_14_block
    assert "search article` now rejects unsupported identifiers" in v0_8_14_block
    assert "Defaulted article search sorting to relevance" in v0_8_14_block
    assert "Removed stale skill-discovery UX" in v0_8_14_block

    assert "Added Semantic Scholar article enrichment and helpers" in v0_8_16_block
    assert "Trial search now accepts fractional ages" in v0_8_16_block
    assert "Added `CITATION.cff`" in v0_8_16_block
    assert "Expanded release-quality gates" in v0_8_16_block

    assert "Added Human Protein Atlas tissue expression" in v0_8_17_block
    assert "Deepened OpenTargets integration" in v0_8_17_block
    assert "MCP chart responses can now return SVG inline" in v0_8_17_block


def test_release_overview_uses_manifest_reference_for_current_version_and_release_files() -> (
    None
):
    overview = _read("architecture/technical/overview.md")
    example_tag = _current_release_tag_example()

    assert "**Current version:** see `Cargo.toml`" in overview
    assert "`scripts/check-version-sync.sh` keeps" in overview
    assert "Release checklist" in overview
    for required_file in [
        "`Cargo.toml`",
        "`Cargo.lock`",
        "`pyproject.toml`",
        "`manifest.json`",
        "`CITATION.cff`",
        "`CHANGELOG.md`",
    ]:
        assert required_file in overview

    assert (
        f'tag="${{BIOMCP_TAG:?set BIOMCP_TAG to the published release tag, e.g. {example_tag}}}"'
        in overview
    )
    assert 'version="${tag#v}"' in overview
    assert 'BIOMCP_VERSION="$tag" bash install.sh' in overview
    assert "https://biomcp.org/reference/bioasq-benchmark/" in overview
    assert "https://biomcp.org/getting-started/api-keys/" in overview
    assert "https://biomcp.org/user-guide/drug/" in overview


def test_release_overview_post_tag_public_proof_requires_all_markers() -> None:
    overview = _read("architecture/technical/overview.md")
    post_tag_block = _markdown_section_block(overview, "### Post-tag public proof")
    example_tag = _current_release_tag_example()

    assert (
        f'tag="${{BIOMCP_TAG:?set BIOMCP_TAG to the published release tag, e.g. {example_tag}}}"'
        in post_tag_block
    )
    assert 'version="${tag#v}"' in post_tag_block
    assert 'BIOMCP_VERSION="$tag" bash install.sh' in post_tag_block
    assert 'bioasq_page="$(mktemp)"' in post_tag_block
    assert "rg -q 'hf-public-pre2026'" in post_tag_block
    assert "rg -q 'Phase A\\+'" in post_tag_block
    assert "rg -q 'Phase B'" in post_tag_block
    assert 'api_keys_page="$(mktemp)"' in post_tag_block
    assert "rg -q 'shared Semantic Scholar pool at 1 req/2sec'" in post_tag_block
    assert "rg -q 'authenticated quota at 1 req/sec'" in post_tag_block
    assert 'drug_page="$(mktemp)"' in post_tag_block
    assert "rg -q 'trastuzumab regulatory --region who'" in post_tag_block
    assert "rg -q 'WHO Prequalification local data setup'" in post_tag_block
    assert "rg -q 'available \\(default path\\)'" in post_tag_block
    assert "| rg 'hf-public-pre2026|Phase A\\+|Phase B'" not in post_tag_block
    assert (
        "| rg 'shared Semantic Scholar pool at 1 req/2sec|authenticated quota at 1 req/sec'"
        not in post_tag_block
    )
    assert (
        "| rg -- 'trastuzumab regulatory --region who|WHO Prequalification local data setup|available \\(default path\\)'"
        not in post_tag_block
    )


def test_gene_guide_includes_new_sections_and_positional_search() -> None:
    gene_guide = _read("docs/user-guide/gene.md")

    assert "biomcp search gene BRAF --limit 5" in gene_guide
    assert "biomcp get gene BRAF expression" in gene_guide
    assert "biomcp get gene BRAF hpa" in gene_guide
    assert "biomcp get gene BRAF druggability" in gene_guide
    assert "biomcp get gene BRAF clingen" in gene_guide
    assert "biomcp get gene BRAF constraint" in gene_guide


def test_article_guide_documents_federated_search_and_source_flag() -> None:
    article_guide = _read("docs/user-guide/article.md")

    assert "PubTator3, Europe PMC, and PubMed" in article_guide
    assert "deduplicated by PMID" in article_guide
    assert "Semantic Scholar" in article_guide
    assert "S2_API_KEY" in article_guide
    assert "PubMed ESearch cleans bounded filler words" in article_guide
    assert "--source pubtator" in article_guide
    assert "--source europepmc" in article_guide


def test_data_sources_reference_covers_new_gene_and_article_sources() -> None:
    data_sources = _read("docs/reference/data-sources.md")

    assert (
        "UniProt, QuickGO, STRING, GTEx, Human Protein Atlas, DGIdb, OpenTargets, ClinGen, gnomAD GraphQL API"
        in data_sources
    )
    assert "https://gtexportal.org/api/v2" in data_sources
    assert "https://www.proteinatlas.org" in data_sources
    assert "https://dgidb.org/api/graphql" in data_sources
    assert "https://search.clinicalgenome.org" in data_sources
    assert "https://gnomad.broadinstitute.org/api" in data_sources
    assert "gnomAD v4 GRCh38 gene constraint" in data_sources
    assert "HPA protein tissue expression and subcellular localization" in data_sources
    assert (
        "| Article search & metadata | PubTator3 + Europe PMC + PubMed + LitSense2 + optional Semantic Scholar |"
        in data_sources
    )
    assert "| Article enrichment and graph helpers | Semantic Scholar |" in data_sources
    assert "PubTator3 + Europe PMC + PubMed for federated search" in data_sources
    assert "1 request / second" in data_sources


def test_cli_and_quick_reference_cover_search_all_and_gene_sections() -> None:
    cli_reference = _read("docs/user-guide/cli-reference.md")
    quick_reference = _read("docs/reference/quick-reference.md")

    assert "### All (cross-entity)" in cli_reference
    assert "biomcp search all --gene BRAF --disease melanoma" in cli_reference
    assert "biomcp get gene BRAF pathways ontology diseases protein" in cli_reference
    assert (
        "biomcp get gene BRAF go interactions civic expression hpa druggability clingen constraint"
        in cli_reference
    )
    assert "biomcp get gene BRAF all" in cli_reference
    assert "_meta.evidence_urls" in cli_reference
    assert "Ensembl, OMIM, NCBI Gene, and UniProt URLs." in cli_reference

    assert "biomcp search gene BRAF --limit 5" in quick_reference
    assert "biomcp search all --gene BRAF --disease melanoma" in quick_reference
    assert "biomcp search all --keyword resistance --counts-only" in quick_reference


def test_public_docs_surface_local_study_analytics() -> None:
    readme = _read("README.md")
    quick_reference = _read("docs/reference/quick-reference.md")
    cli_reference = _read("docs/user-guide/cli-reference.md")
    study_commands = [
        "biomcp study list",
        "biomcp study download [--list] [<study_id>]",
        "biomcp study filter --study <id> [--mutated <symbol>] [--amplified <symbol>] [--deleted <symbol>] [--expression-above <gene:threshold>] [--expression-below <gene:threshold>] [--cancer-type <type>]",
        "biomcp study query --study <id> --gene <symbol> --type <mutations|cna|expression>",
        "biomcp study cohort --study <id> --gene <symbol>",
        "biomcp study survival --study <id> --gene <symbol> [--endpoint <os|dfs|pfs|dss>]",
        "biomcp study compare --study <id> --gene <symbol> --type <expression|mutations> --target <symbol>",
        "biomcp study co-occurrence --study <id> --genes <g1,g2,...>",
    ]

    assert "plus local study analytics" in readme
    assert "## Local study analytics" in readme
    assert "13 remote entity commands" not in readme
    assert (
        "public entity surface handles API-backed, local-runtime, and hybrid" in readme
    )
    assert "study download" in readme

    assert "## Study commands" in quick_reference
    assert "local downloaded cBioPortal-style datasets" in quick_reference
    assert "BIOMCP_STUDY_DIR" in quick_reference
    for command in study_commands:
        assert command in quick_reference

    assert "## Local study analytics" in cli_reference
    assert "BIOMCP_STUDY_DIR" in cli_reference
    assert "local cBioPortal analytics family for downloaded" in cli_reference
    assert "cBioPortal-style datasets" in cli_reference
    assert "13 remote entity commands" not in cli_reference
    assert (
        "Unlike the public entity surface, `study` operates on files" in cli_reference
    )
    assert "data_mutations.txt" in cli_reference
    assert "data_clinical_patient.txt" in cli_reference
    for command in study_commands:
        assert command in cli_reference
