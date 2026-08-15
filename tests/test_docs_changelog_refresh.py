from __future__ import annotations

import hashlib
import re
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
EXPECTED_RELEASE_TICKETS = {
    "0.8.25": {
        450,
        452,
        453,
        459,
        460,
        463,
        465,
        466,
        467,
        468,
        469,
        470,
        472,
        473,
        474,
        476,
        477,
        478,
        479,
    },
    "0.8.24": {
        239,
        240,
        432,
        433,
        *range(434, 445),
    },
    "0.8.23": {
        369,
        *range(371, 387),
        389,
        390,
        391,
        392,
        393,
        395,
        396,
        397,
        398,
        400,
        401,
        403,
        404,
        405,
        406,
        408,
        409,
        410,
        411,
        *range(412, 421),
    },
    "0.8.22": {
        217,
        218,
        220,
        222,
        224,
        226,
        230,
        232,
        233,
        235,
        236,
        237,
        238,
        239,
        240,
        242,
        243,
        244,
        245,
        246,
        247,
        252,
        253,
        254,
        255,
        256,
        258,
        262,
        263,
        *range(264, 286),
        286,
        287,
        288,
        289,
        290,
        291,
        292,
        294,
        297,
        298,
        299,
        300,
        301,
        302,
        303,
        304,
        306,
        307,
        308,
        309,
        310,
        313,
        314,
        315,
        316,
        317,
        318,
        319,
        320,
        321,
        322,
        323,
        324,
        325,
        326,
        327,
        328,
        329,
        330,
        331,
        332,
        333,
        334,
        335,
        336,
        338,
        339,
        340,
        341,
        342,
        343,
        344,
        345,
        346,
        347,
        348,
        350,
        351,
        352,
        353,
        354,
        355,
        357,
        358,
        362,
        363,
        364,
        365,
        366,
        367,
        370,
    }
}
POST_TAG_CHANGELOG_MARKERS = (
    "`variant_normalize_car`",
    "content-addressed handles",
    "`variant_erepo`",
    "`gene_cspec`",
    "JATS- and PMC-HTML-linked article supplements",
    "one-pass pagination",
    "`canonical_equivalence`",
    "ClinGen LDH article identity",
    "`variant articles --input <path|->`",
    "article-search JSON compact by default",
    "PMID/PMCID/DOI/arXiv/Semantic Scholar identifiers",
    "provider-exact `search author`",
    "opt-in PubMed article indexing",
    "consequence-dropping fallback",
    "disease survival `data`/`empty`/`unavailable` outcomes",
    "`inapplicable` lookup state",
    "invisible bidi markers",
    "`BayesDel add-AF`",
    "complex-table omission metadata",
    "command-owned JSON collection paths",
    "standard structured JSON errors",
    "Europe PMC supplementary ZIPs",
    "normal DTD-bearing citation XML",
)
# SHA-256 of the complete 0.8.25 section in the published v0.8.25 CHANGELOG.md.
PUBLISHED_V0_8_25_CHANGELOG_BLOCK_SHA256 = (
    "5d825643e7fb90dcdccd92957a992616f0e186227c660978489d483639435da0"
)

EXPECTED_RELEASE_MARKERS = {
    "0.8.25": {
        "fixes": [
            "JSON usage errors",
            "clap parse failures",
            "variant normalize",
            "protein and phenotype JSON search next commands",
            "zero-result filtered trial searches",
            "CIViC actionability pointer",
            "drug adverse-events --count",
            "disease-survival and CTGov helper output",
            "dynamic reserved ports",
        ],
        "new_features": [
            "Homebrew formula",
            "`biomcp mcp-config` generator",
            "MCP-client setup docs",
            "Docker image publication",
            "ghcr.io/genomoncology/biomcp",
            "official MCP Registry metadata",
            "read-only hints",
            "variant structure annotation",
        ],
        "docs": [
            "Add BioMCP to your MCP client",
            "environment configuration parity",
            "shell-safe HGVS command examples",
            "variant-structure blog walkthrough",
            "skill-authoring guide",
            "v0.8.25",
        ],
        "internal": [
            "whole-surface CLI contract ratchets",
            "helper and parent help discovery",
            "SEER fixture",
            "contract and release smokes",
            "retired validation profiles",
        ],
    },
    "0.8.24": {
        "fixes": [
            "RUSTSEC-2026-0186",
            "memmap2",
            "Python `mcp` test-client dependency",
            "Rust rmcp-client harness",
            "Dependabot alerts",
            "cache disk-space WARN",
            "CTGov trial-helper post-output latency",
            "canonical HGNC genes",
            "JSON error objects on stdout",
        ],
        "new_features": [
            "Claude Code plugin marketplace",
            "serve-http --allowed-hosts",
            "Host guard",
            "MCP responses",
            "typed MCP `search` and `get` tools",
            "CLI rejection errors",
            "transcript HGVS inputs",
            "`get pathway` redirect hints",
        ],
        "docs": [
            "release-prep metadata",
            "v0.8.24",
        ],
        "internal": [
            "light parity/coherence review",
            "Rust rmcp-client contract",
            "no release-blocking findings",
            "parallel-isolation canary",
            "baked version string",
        ],
    },
    "0.8.23": {
        "fixes": [
            "CDC WONDER-compatible",
            "Figshare provider-supplied asset download URLs",
            "private-network",
            "Source unavailable",
            "extreme `Retry-After`",
            "list-only Cargo wrappers",
        ],
        "new_features": [
            "source-first article full-text",
            "asset retrieval improvements",
            "Cancerhotspots recurrence enrichment",
            "AlphaGenome prediction credentials",
        ],
        "docs": [
            "source-versioning",
            "configuration",
            "observability",
            "gettable versus search-only",
            "architecture experiment policy",
            "quality ratchet",
        ],
        "internal": [
            "request-contract tests",
            "mustmatch binary",
            "live upstream specs",
            "0.8.23 hardening pass",
            "Monarch 502",
        ],
    },
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
            "_meta.workflow",
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
            "deferred work",
            "runtime wiring shipped",
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
    return _citation_scalar("version")


def _current_release_heading() -> str:
    return f"## {_current_release_version()} — {_citation_scalar('date-released')}"


def _current_release_tag_example() -> str:
    return "v0.8.24"


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

    assert changelog.startswith("# Changelog\n\n## Unreleased\n")
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
        ("0.8.22", "2026-04-30"),
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


def test_v0_8_25_release_block_matches_the_published_tag_boundary() -> None:
    current_changelog = _read("CHANGELOG.md")
    header = "## 0.8.25 — 2026-07-07"
    published_block = header + _markdown_section_block(current_changelog, header)

    assert (
        hashlib.sha256(published_block.encode("utf-8")).hexdigest()
        == PUBLISHED_V0_8_25_CHANGELOG_BLOCK_SHA256
    )
    unreleased = _markdown_section_block(current_changelog, "## Unreleased")
    published = _markdown_section_block(current_changelog, header)
    for marker in POST_TAG_CHANGELOG_MARKERS:
        assert marker in unreleased, f"post-tag changelog fact was lost: {marker}"
        assert marker not in published, f"post-tag changelog fact remained published: {marker}"


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
    assert "--allowed-hosts biomcp.example.org,localhost:8080" in remote_http
    assert "not authentication" in remote_http

    mcp_reference = _read("docs/reference/mcp-server.md")
    assert "non-loopback bind fails" in mcp_reference
    assert "--unsafe-allow-any-host" in mcp_reference
    assert "adds no authentication" in mcp_reference
    assert "encryption" in mcp_reference
    assert "does not infer trust" in mcp_reference
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
        assert "MedlinePlus `clinical_features`" not in text
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
    assert "biomcp get disease melanoma clinical_features" in cli_reference
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
    assert "MedlinePlus `clinical_features`" not in functional
    assert "CDC WONDER VAERS aggregate vaccine search" in functional

    assert "biomcp gtr sync" in ux_reference
    assert "biomcp who-ivd sync" in ux_reference
    assert "biomcp get disease melanoma clinical_features" in ux_reference
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
        "`spec/entity/diagnostic.md`",
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


def test_release_overview_describes_committed_metadata_and_protected_promotion() -> None:
    overview = _read("architecture/technical/overview.md")

    assert "**Development candidate:** Rust `0.9.0-dev.1`; Python `0.9.0.dev1`" in overview
    assert "validates that mapping and its lock roots" in overview
    assert "both `server.json` version fields" in overview
    assert "`CITATION.cff`" in overview
    assert "v0.8.25 is the latest published release." in overview
    assert "public metadata remains at `0.8.25`" in overview
    assert "Package versions are committed metadata, not values stamped from tags." in overview
    assert "separate `stage` and `promote` modes" in overview
    assert "only then advances mutable" in overview
    assert "Neither implementing the workflow nor staging a candidate publishes" in overview


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
        "| Article search & metadata | PubTator3 + Europe PMC + PubMed + optional Semantic Scholar; Semantic Scholar and LitSense2 by explicit `--source semanticscholar` / `--source litsense2` |"
        in data_sources
    )
    assert "| Article enrichment and graph helpers | Semantic Scholar |" in data_sources
    assert "PubTator3 + Europe PMC + PubMed for default federated search" in data_sources
    assert "1 request / second" in data_sources


def test_variant_article_batch_grammar_stays_aligned_in_user_and_ux_references() -> None:
    references = [
        _read("docs/user-guide/cli-reference.md"),
        _read("architecture/ux/cli-reference.md"),
    ]

    for reference in references:
        assert re.search(r"biomcp\s+--json\s+variant\s+articles[^\n]*--input[^\n]*--debug-plan", reference)
        assert re.search(r"variant\s+articles\s+--input\s+-", reference)
        assert re.search(r"--input\s+<path\|->", reference)
        assert re.search(r"typed[^.\n]*`variant_articles`", reference)


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
        "biomcp study query --study <id> --gene <symbol> --type <mutations|cna|expression|sv>",
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
