from __future__ import annotations

import re
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]

OLS4_HEAVY_SPEC_HEADINGS = {
    "spec/entity/disease.md": (
        "Synonym Rescue",
    ),
    "spec/surface/discover.md": (
        "Alias-Like Free Text Still Resolves to Typed Follow-Ups",
        "Disease-Specific Symptom Phrases Stay Clinically Modest",
        "HPO-Backed Symptom Phrases Should Bridge into Phenotype Search",
    ),
}


def _read_repo(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def _rust_function_block(path: str, fn_name: str) -> str:
    lines = _read_repo(path).splitlines()
    signature = f"fn {fn_name}("
    for index, line in enumerate(lines):
        if signature not in line:
            continue

        start = index
        while start > 0 and lines[start - 1].lstrip().startswith("#["):
            start -= 1

        depth = 0
        seen_body = False
        for end in range(index, len(lines)):
            depth += lines[end].count("{")
            seen_body = seen_body or ("{" in lines[end])
            depth -= lines[end].count("}")
            if seen_body and depth == 0:
                return "\n".join(lines[start : end + 1])
        break

    raise AssertionError(f"function {fn_name!r} not found in {path}")


def _make_target_block(name: str) -> str:
    makefile = _read_repo("Makefile")
    match = re.search(
        rf"(?ms)^{re.escape(name)}:\n(.*?)(?=^[A-Za-z0-9_.-]+:|\Z)",
        makefile,
    )
    assert match is not None, f"missing Makefile target {name}"
    return match.group(1)


def _markdown_h2_headings(path: str) -> set[str]:
    return set(re.findall(r"^##\s+(.+?)\s*$", _read_repo(path), flags=re.MULTILINE))


def _markdown_heading_body(path: str, level: int, heading: str) -> str:
    text = _read_repo(path)
    marker = f"{'#' * level} {heading}"
    match = re.search(rf"^{re.escape(marker)}\s*$", text, flags=re.MULTILINE)
    assert match is not None, f"missing heading {marker!r} in {path}"
    end_match = re.search(rf"^#{{1,{level}}}\s+", text[match.end() :], flags=re.MULTILINE)
    end = len(text) if end_match is None else match.end() + end_match.start()
    return text[match.end() : end]


def _non_skipped_bash_blocks(markdown: str) -> list[str]:
    blocks: list[str] = []
    for match in re.finditer(r"^```bash([^`\n]*)\n(.*?)^```", markdown, flags=re.MULTILINE | re.DOTALL):
        fence_tokens = match.group(1).split()
        if "skip" not in fence_tokens:
            blocks.append(match.group(2))
    return blocks


def _bash_blocks(markdown: str) -> list[str]:
    return [
        match.group(2)
        for match in re.finditer(
            r"^```bash([^`\n]*)\n(.*?)^```",
            markdown,
            flags=re.MULTILINE | re.DOTALL,
        )
    ]


def _non_fixture_biomcp_invocations(markdown: str) -> list[str]:
    invocations: list[str] = []
    for block in _bash_blocks(markdown):
        if "setup-article-fulltext-source-fixture.sh" in block:
            continue
        for line in block.splitlines():
            stripped = line.strip()
            if not stripped or stripped.startswith("#"):
                continue
            live_tokens = ("../../tools/biomcp-ci", "BIOMCP_BIN", '"$biomcp_bin"')
            if any(token in stripped for token in live_tokens):
                invocations.append(stripped)
    return invocations


def _has_base_url_probe(text: str) -> bool:
    return bool(
        re.search(r"curl[^\n]*\$(?:\{base_url\}|base_url)", text)
        or re.search(r"wget[^\n]*\$(?:\{base_url\}|base_url)", text)
        or re.search(r"urllib\.request\.[A-Za-z_]+\([^\n]*base_url", text)
        or ("/dev/tcp/" in text and "base_url" in text)
    )


def test_wikipathways_parallel_contract_serializes_shared_mock_env() -> None:
    context = _read_repo("src/cli/search_all/tests/dispatch.rs")
    assert "dispatch_section_pathway_surfaces_sanitized_wikipathways_404_without_timeout" not in context
    assert "MockServer" not in context
    assert "BIOMCP_WIKIPATHWAYS_BASE" not in context


def test_vaers_fixture_contract_waits_for_live_http_readiness() -> None:
    script = _read_repo("spec/fixtures/setup-vaers-spec-fixture.sh")
    before_exports = script.split("printf 'export BIOMCP_VAERS_BASE", 1)[0]
    readiness_tail = before_exports.split('base_url="$(cat "$ready_file")"', 1)[-1]

    assert any(loop_token in readiness_tail for loop_token in ("for _ in", "while ")), (
        "the VAERS fixture setup should retry the readiness probe after base_url is known, not "
        "fire a single best-effort request before exporting BIOMCP_VAERS_BASE"
    )
    assert _has_base_url_probe(readiness_tail), (
        "the VAERS fixture setup must perform a real HTTP readiness probe against $base_url after "
        "choosing the base URL and before exporting BIOMCP_VAERS_BASE, otherwise spec-pr can "
        "still race the background server under xdist load"
    )


def test_trial_alias_retry_contract_uses_private_cache_or_no_cache_mode() -> None:
    context = _read_repo("src/entities/drug/get/tests.rs")
    assert "resolve_trial_aliases_retries_after_transient_lookup_failure" not in context
    assert "MockServer" not in context
    assert "BIOMCP_MYCHEM_BASE" not in context


def test_diagnostic_regulatory_contract_uses_private_openfda_cache() -> None:
    context = _read_repo("src/entities/diagnostic/mod.rs")
    assert "get_regulatory_uses_alias_queries_and_dedupes_pma_supplements" not in context
    assert "MockServer" not in context
    assert "BIOMCP_OPENFDA_BASE" not in context


def test_ticket_372_quarantines_known_routine_gate_blockers() -> None:
    quarantined_sections = (
        (
            "spec/entity/disease.md",
            2,
            "Synonym Rescue",
            ("ticket 371", "fixture-backed", "release/live-smoke"),
        ),
        (
            "spec/surface/discover.md",
            3,
            "MEF2 relational query",
            ("ticket 371", "fixture-backed", "release/live-smoke"),
        ),
        (
            "spec/entity/gene.md",
            2,
            "All-Section Warm Budget",
            ("ticket 371", "benchmark/ratchet", "explicit performance"),
        ),
        (
            "spec/entity/drug.md",
            2,
            "Research-Code Bridge",
            ("ticket 382", "fixture-backed", "release/live-smoke", "drug alias"),
        ),
        (
            "spec/entity/drug.md",
            2,
            "Ambiguous Research-Code Fallback",
            ("ticket 380", "fixture-backed", "release/live-smoke", "drug/alias"),
        ),
    )

    for path, level, heading, required_fragments in quarantined_sections:
        section = _markdown_heading_body(path, level, heading)
        assert not _non_skipped_bash_blocks(section), (
            f"{path}::{heading} must stay out of routine executable specs until it has "
            "deterministic request-contract coverage, a benchmark/ratchet, or an explicit "
            "release/live-smoke/performance lane"
        )
        section_lower = section.lower()
        for fragment in required_fragments:
            assert fragment in section_lower

    timings = _read_repo("spec/README-timings.md").lower()
    assert "spec/entity/gene.md::all-section warm budget" in timings
    assert "quarantined from routine `make spec-pr` by ticket 372" in timings
    assert "benchmark/ratchet" in timings
    assert "performance lane" in timings


def _rust_struct_block(path: str, struct_name: str) -> str:
    lines = _read_repo(path).splitlines()
    signature = f"struct {struct_name}"
    for index, line in enumerate(lines):
        if signature not in line:
            continue

        start = index
        while start > 0 and lines[start - 1].lstrip().startswith("#["):
            start -= 1

        depth = 0
        seen_body = False
        for end in range(index, len(lines)):
            depth += lines[end].count("{")
            seen_body = seen_body or ("{" in lines[end])
            depth -= lines[end].count("}")
            if seen_body and depth == 0:
                return "\n".join(lines[start : end + 1])
        break

    raise AssertionError(f"struct {struct_name!r} not found in {path}")


def _rust_test_blocks(path: str) -> list[str]:
    lines = _read_repo(path).splitlines()
    blocks: list[str] = []
    for index, line in enumerate(lines):
        if "fn " not in line:
            continue

        start = index
        while start > 0 and lines[start - 1].lstrip().startswith("#["):
            start -= 1

        depth = 0
        seen_body = False
        for end in range(index, len(lines)):
            depth += lines[end].count("{")
            seen_body = seen_body or ("{" in lines[end])
            depth -= lines[end].count("}")
            if seen_body and depth == 0:
                block = "\n".join(lines[start : end + 1])
                if "#[test]" in block or "#[tokio::test]" in block:
                    blocks.append(block)
                break
    return blocks


def _assert_contains_all(text: str, fragments: tuple[str, ...], context: str) -> None:
    missing = [fragment for fragment in fragments if fragment not in text]
    assert not missing, f"{context} is missing required request-plan contract fragments: {missing}"


def _contains_all(text: str, fragments: tuple[str, ...]) -> bool:
    return all(fragment in text for fragment in fragments)


def _assert_struct_fields(block: str, fields: tuple[str, ...], context: str) -> None:
    missing = [
        field
        for field in fields
        if not re.search(rf"\b(?:pub(?:\([^)]*\))?\s+)?{re.escape(field)}\s*:", block)
    ]
    assert not missing, f"{context} is missing required request-plan fields: {missing}"


def _assert_request_used_before_marker(
    path: str,
    fn_name: str,
    request_name: str,
    marker: str,
    context: str,
) -> None:
    block = _rust_function_block(path, fn_name)
    assert marker in block, f"{context} must still execute through {marker!r}"
    before_marker = block.split(marker, 1)[0]
    assert request_name in before_marker, (
        f"{context} must build and consume {request_name} before {marker!r}, otherwise the "
        "request-command seam cannot prove user intent before network execution"
    )


def test_ticket_375_request_command_seams_capture_user_intent_before_network_execution() -> None:
    failures: list[str] = []

    def check(label: str, assertion) -> None:
        try:
            assertion()
        except AssertionError as exc:
            failures.append(f"{label}: {exc}")

    check(
        "DiscoverRequest struct fields",
        lambda: _assert_struct_fields(
            _rust_struct_block("src/entities/discover.rs", "DiscoverRequest"),
            ("query", "mode", "ols_query", "medlineplus_enabled", "no_cache"),
            "DiscoverRequest",
        ),
    )
    check(
        "discover request consumed before OLS4 client construction",
        lambda: _assert_request_used_before_marker(
            "src/entities/discover.rs",
            "resolve_query",
            "DiscoverRequest",
            "OlsClient::new()",
            "discover resolve_query",
        ),
    )

    check(
        "DiseaseSearchRequest struct fields",
        lambda: _assert_struct_fields(
            _rust_struct_block("src/entities/disease/search.rs", "DiseaseSearchRequest"),
            (
                "query",
                "source",
                "inheritance",
                "phenotype",
                "onset",
                "limit",
                "offset",
                "fetch_size",
                "resolver_queries",
                "prefer_doid",
            ),
            "DiseaseSearchRequest",
        ),
    )
    check(
        "disease search request consumed before MyDisease client construction",
        lambda: _assert_request_used_before_marker(
            "src/entities/disease/search.rs",
            "search_page",
            "DiseaseSearchRequest",
            "MyDiseaseClient::new()",
            "disease search_page",
        ),
    )

    check(
        "DiseaseFallbackRequest struct fields",
        lambda: _assert_struct_fields(
            _rust_struct_block("src/entities/disease/fallback.rs", "DiseaseFallbackRequest"),
            (
                "query",
                "limit",
                "offset",
                "resolver_queries",
                "skip_reason",
                "discover_mode",
                "prefer_doid",
            ),
            "DiseaseFallbackRequest",
        ),
    )
    check(
        "disease fallback request consumed before discover alias fallback execution",
        lambda: _assert_request_used_before_marker(
            "src/entities/disease/fallback.rs",
            "fallback_search_page",
            "DiseaseFallbackRequest",
            "discover::resolve_query",
            "disease fallback_search_page",
        ),
    )
    check(
        "disease fallback request consumed before MyDisease client construction",
        lambda: _assert_request_used_before_marker(
            "src/entities/disease/fallback.rs",
            "fallback_search_page",
            "DiseaseFallbackRequest",
            "MyDiseaseClient::new()",
            "disease fallback_search_page",
        ),
    )
    check(
        "disease CLI still owns no-fallback gating",
        lambda: _assert_contains_all(
            _rust_function_block("src/cli/disease/dispatch.rs", "handle_search"),
            ("!args.no_fallback", "fallback_search_page"),
            "disease CLI fallback gate",
        ),
    )

    check(
        "ArticleSearchRequest struct fields",
        lambda: _assert_struct_fields(
            _rust_struct_block("src/cli/article/dispatch.rs", "ArticleSearchRequest"),
            (
                "filters",
                "source_filter",
                "limit",
                "offset",
                "sort",
                "ranking",
                "backend_plan",
                "exact_keyword_lookup",
            ),
            "ArticleSearchRequest",
        ),
    )
    check(
        "article request consumed before article search execution",
        lambda: _assert_request_used_before_marker(
            "src/cli/article/dispatch.rs",
            "handle_search",
            "ArticleSearchRequest",
            "entities::article::search_page",
            "article handle_search",
        ),
    )
    check(
        "article request reuses BackendPlan planner",
        lambda: _assert_contains_all(
            _read_repo("src/cli/article/dispatch.rs"),
            ("ArticleSearchRequest", "plan_backends("),
            "article request-command seam",
        ),
    )

    assert not failures, "ticket 375 request-command seam contract failures:\n" + "\n".join(failures)


def test_ticket_374_ols4_search_request_plan_contract_is_source_local() -> None:
    plan_struct = _rust_struct_block("src/sources/ols4.rs", "OlsSearchRequestPlan")
    plan_builder = _rust_function_block("src/sources/ols4.rs", "search_request_plan")
    search_executor = _rust_function_block("src/sources/ols4.rs", "search")

    _assert_struct_fields(
        plan_struct,
        (
            "method",
            "path",
            "query_params",
            "source_label",
            "cache_mode",
            "status_expectation",
            "content_type_expectation",
        ),
        "OlsSearchRequestPlan",
    )
    _assert_contains_all(
        plan_builder,
        (
            "GET",
            "/api/search",
            "q",
            "rows",
            "groupField",
            "ontology",
            "ols4",
            "json",
        ),
        "OlsClient::search_request_plan",
    )
    assert "search_request_plan(" in search_executor.split(".send()", 1)[0], (
        "OlsClient::search must build and consume the request plan before sending the HTTP request, "
        "otherwise tests can still only observe the request after network execution"
    )


def test_ticket_374_mydisease_request_plan_contracts_are_source_local() -> None:
    for struct_name in (
        "MyDiseaseQueryRequestPlan",
        "MyDiseaseXrefLookupRequestPlan",
        "MyDiseaseGetRequestPlan",
    ):
        _assert_struct_fields(
            _rust_struct_block("src/sources/mydisease.rs", struct_name),
            ("method", "path", "query_params", "cache_mode", "status_expectation"),
            struct_name,
        )

    builders = {
        "query_plan": ("RequestPlan::get(\"query\")", "q", "size", "from", "fields", "MYDISEASE_SEARCH_FIELDS"),
        "lookup_disease_by_xref_plan": (
            "RequestPlan::get(\"query\")",
            "mesh",
            "omim",
            "icd10cm",
            "MYDISEASE_SEARCH_FIELDS",
        ),
        "get_plan": ("RequestPlan::get(format!(\"disease/{id}\"))", "fields", "MYDISEASE_GET_FIELDS"),
    }
    for fn_name, fragments in builders.items():
        block = _rust_function_block("src/sources/mydisease.rs", fn_name)
        _assert_contains_all(
            block,
            fragments,
            f"MyDiseaseClient::{fn_name}",
        )

    for executor, builder_name in (
        ("query", "query_plan("),
        ("lookup_disease_by_xref", "lookup_disease_by_xref_plan("),
        ("get", "get_plan("),
    ):
        block = _rust_function_block("src/sources/mydisease.rs", executor)
        assert builder_name in block.split(".send()", 1)[0], (
            f"MyDiseaseClient::{executor} must build and consume {builder_name} before sending "
            "the HTTP request so source contracts do not depend on observing wiremock traffic"
        )


def test_ticket_374_quarantined_disease_discover_holes_have_deterministic_replacements() -> None:
    disease_markers = ("OlsSearchRequestPlan", "lookup_disease_by_xref_plan", "Arnold", "MESH")
    discover_markers = (
        "OlsSearchRequestPlan",
        "genes regulated by MEF2 in the heart",
        "search all --keyword",
    )

    assert any(
        _contains_all(block, disease_markers)
        for block in _rust_test_blocks("src/entities/disease/fallback/tests.rs")
    ), (
        "disease synonym-rescue deterministic replacement must have an executable Rust test block "
        f"with request-plan/fixture markers {disease_markers}"
    )
    assert any(
        _contains_all(block, discover_markers)
        for block in _rust_test_blocks("src/entities/discover.rs")
    ), (
        "discover MEF2 deterministic replacement must have an executable Rust test block with "
        f"request-plan/fixture markers {discover_markers}"
    )

    for path, level, heading in (
        ("spec/entity/disease.md", 2, "Synonym Rescue"),
        ("spec/surface/discover.md", 3, "MEF2 relational query"),
    ):
        section = _markdown_heading_body(path, level, heading)
        section_lower = section.lower()
        assert "quarantined" not in section_lower, (
            f"{path}::{heading} must stop describing the behavior as quarantined once the "
            "ticket-374 deterministic replacement tests exist"
        )
        assert any(fragment in section_lower for fragment in ("fixture", "request-plan", "live-smoke")), (
            f"{path}::{heading} must document whether the restored coverage is fixture/request-plan "
            "backed or deliberately release/live-smoke-only"
        )


def _assert_plan_contract(
    path: str,
    struct_name: str,
    builder_name: str,
    executor_name: str,
    fields: tuple[str, ...],
    builder_fragments: tuple[str, ...],
    consumption_fragments: tuple[str, ...],
    context: str,
) -> None:
    plan_struct = _rust_struct_block(path, struct_name)
    plan_builder = _rust_function_block(path, builder_name)
    executor = _rust_function_block(path, executor_name)

    _assert_struct_fields(plan_struct, fields, struct_name)
    _assert_contains_all(plan_builder, builder_fragments, f"{context} builder")

    send_markers = (".send()", "send_json(", "get_json(")
    marker = next((candidate for candidate in send_markers if candidate in executor), None)
    assert marker is not None, f"{context} executor must still send through the source client"
    before_send = executor.split(marker, 1)[0]
    _assert_contains_all(
        before_send,
        (builder_name, *consumption_fragments),
        f"{context} executor consumption",
    )


def _assert_any_test_block_contains(paths: tuple[str, ...], fragments: tuple[str, ...], context: str) -> None:
    matching_blocks = [
        block
        for path in paths
        for block in _rust_test_blocks(path)
        if _contains_all(block, fragments)
    ]
    assert matching_blocks, (
        f"{context} needs an executable deterministic Rust test block containing fixture/request-plan "
        f"markers {fragments}"
    )


def _assert_ticket_test_blocks_cover(
    paths: tuple[str, ...],
    marker: str,
    fragments: tuple[str, ...],
    context: str,
) -> None:
    matching_blocks = [
        block
        for path in paths
        for block in _rust_test_blocks(path)
        if marker in block
    ]
    assert matching_blocks, (
        f"{context} needs executable deterministic Rust test block(s) named with {marker!r}"
    )
    combined = "\n".join(matching_blocks)
    missing = [fragment for fragment in fragments if fragment not in combined]
    assert not missing, (
        f"{context} ticket-marked Rust test blocks are missing renderer/envelope behavior fragments: "
        f"{missing}"
    )


def test_ticket_376_article_source_request_plans_are_source_local_and_consumed() -> None:
    source_tests = (
        "src/sources/pubmed/tests/construction.rs",
        "src/sources/europepmc/tests/construction.rs",
        "src/sources/pubtator/tests/construction.rs",
        "src/sources/litsense2/tests/construction.rs",
        "src/sources/semantic_scholar/tests/construction.rs",
    )
    for path in source_tests:
        text = _read_repo(path)
        assert "MockServer" not in text
        assert "RequestPlan" in text
        assert "assert_eq!(plan.method" in text or "method" in text
        assert "query_value(" in text or "query_params" in text


def test_ticket_376_article_source_fixture_contracts_replace_routine_live_canaries() -> None:
    article_paths = (
        "src/sources/pubmed/tests/construction.rs",
        "src/sources/europepmc/tests/construction.rs",
        "src/sources/pubtator/tests/construction.rs",
        "src/sources/litsense2/tests/construction.rs",
        "src/sources/semantic_scholar/tests/construction.rs",
    )
    for label, fragments in (
        ("PubMed article source fixture", ("PubMedESearchRequestPlan", "PubMedESummaryRequestPlan", "BRAF")),
        ("Europe PMC article source fixture", ("EuropePmcSearchRequestPlan", "alternative microexon", "pageSize")),
        ("PubTator article source fixture", ("PubTatorSearchRequestPlan", "PubTatorExportRequestPlan", "annotations")),
        ("LitSense2 article source fixture", ("LitSense2SearchRequestPlan", "BRAF")),
        ("LitSense2 PubMed hydration fixture", ("PubMedESummaryRequestPlan", "pubmed_hydration")),
        (
            "Semantic Scholar keyless/auth degradation fixture",
            ("SemanticScholarPaperSearchRequestPlan", "auth_mode", "shared_pool"),
        ),
    ):
        _assert_any_test_block_contains(article_paths, fragments, label)


def test_ticket_376_variant_source_request_plans_are_source_local_and_consumed() -> None:
    for path in (
        "src/sources/myvariant/tests/construction.rs",
        "src/sources/mutalyzer/tests/construction.rs",
        "src/sources/variantvalidator/tests/construction.rs",
    ):
        text = _read_repo(path)
        assert "MockServer" not in text
        assert "RequestPlan" in text or "request_plan" in text
        assert "query_value(" in text or "plan.path" in text


def test_ticket_376_variant_fixture_contracts_replace_routine_live_canaries() -> None:
    variant_paths = (
        "src/sources/myvariant/tests/construction.rs",
        "src/sources/myvariant/tests/parsing.rs",
        "src/sources/mutalyzer/tests/construction.rs",
        "src/sources/mutalyzer/tests/parsing.rs",
        "src/sources/variantvalidator/tests/construction.rs",
        "src/sources/variantvalidator/tests/parsing.rs",
    )
    for label, fragments in (
        ("MyVariant search fixture", ("BRAF", "p.Val600Glu")),
        ("MyVariant get fixture", ("rs113488022", "variant/rs113488022")),
        ("MyVariant not-found fixture", ("NotFound", "rs999")),
        ("Mutalyzer normalization fixture", ("MutalyzerNormalizeRequestPlan", "NM_000248.3:c.135del")),
        ("VariantValidator request fixture", ("VariantValidatorNormalizeRequestPlan", "NM_000248.3:c.135del")),
        ("VariantValidator parsing fixture", ("TranscriptVersionWarning", "NC_000017.11:g.39710409G>T")),
    ):
        _assert_any_test_block_contains(variant_paths, fragments, label)


def test_ticket_376_article_variant_specs_document_deterministic_or_live_smoke_coverage() -> None:
    for path in ("spec/entity/article.md", "spec/entity/variant.md"):
        section = _markdown_heading_body(path, 2, "Deterministic Source Contracts")
        lower = section.lower()
        assert "ticket 376" in lower, f"{path} must document the ticket-376 routine coverage conversion"
        assert "request-plan" in lower or "fixture-backed" in lower, (
            f"{path} must name deterministic request-plan or fixture-backed replacement coverage"
        )
        assert "release/live-smoke" in lower, (
            f"{path} must classify irreducible public-upstream checks as release/live-smoke-only"
        )

    for heading in ("Full-Text HTML Fallback", "PDF Fallback Is Opt-In"):
        section = _markdown_heading_body("spec/entity/article.md", 2, heading)
        assert "setup-article-fulltext-source-fixture.sh" in section, (
            f"spec/entity/article.md::{heading} must preserve the existing fixture-backed fulltext pattern"
        )


def test_ticket_377_renderer_envelope_fixture_contracts_exist() -> None:
    failures: list[str] = []

    def check(label: str, assertion) -> None:
        try:
            assertion()
        except AssertionError as exc:
            failures.append(f"{label}: {exc}")

    contracts = (
        (
            "Disease renderer/envelope fixture contract",
            (
                "src/render/json.rs",
                "src/render/markdown/disease/tests.rs",
                "src/render/provenance.rs",
            ),
            (
                "ticket_377_disease_renderer_envelope_contracts",
                "to_entity_json",
                "disease_next_commands",
                "disease_section_sources",
                "disease_markdown",
                "_meta",
                "next_commands",
                "section_sources",
                "| Gene |",
            ),
        ),
        (
            "Discover renderer/envelope fixture contract",
            (
                "src/render/json.rs",
                "src/render/markdown/discovery/tests.rs",
            ),
            (
                "ticket_377_discover_renderer_envelope_contracts",
                "to_discover_json",
                "render_discover",
                "_meta",
                "next_commands",
                "discovery_sources",
                "section_sources",
                "## Concepts",
                "## Suggested Commands",
            ),
        ),
        (
            "Article renderer/envelope fixture contract",
            (
                "src/cli/article/tests/json.rs",
                "src/render/markdown/article/tests.rs",
            ),
            (
                "ticket_377_article_renderer_envelope_contracts",
                "article_search_json",
                "ArticleSourceStatus",
                "ArticleSourceAvailability::Degraded",
                "_meta",
                "source_status",
                "next_commands",
                "article_search_markdown_with_footer_and_context",
                "Semantic Scholar",
            ),
        ),
        (
            "Variant renderer/envelope fixture contract",
            (
                "src/cli/variant/tests.rs",
                "src/render/markdown/variant/tests.rs",
                "src/entities/variant/normalization.rs",
            ),
            (
                "ticket_377_variant_renderer_envelope_contracts",
                "search_json_with_meta",
                "search_next_commands_variant",
                "_meta",
                "next_commands",
                "variant_search_markdown_with_context",
                "VariantNormalizationResponse",
                "variant_normalization_markdown",
                "VariantNormalizationStatus::InvalidInput",
            ),
        ),
    )

    for label, paths, fragments in contracts:
        check(
            label,
            lambda paths=paths, fragments=fragments, label=label: _assert_ticket_test_blocks_cover(
                paths,
                fragments[0],
                fragments,
                label,
            ),
        )

    assert not failures, (
        "ticket 377 renderer/envelope deterministic replacement failures:\n" + "\n".join(failures)
    )


def test_ticket_377_renderer_envelope_specs_document_deterministic_coverage() -> None:
    contracts = (
        ("spec/entity/disease.md", "ticket_377_disease_renderer_envelope_contracts"),
        ("spec/surface/discover.md", "ticket_377_discover_renderer_envelope_contracts"),
        ("spec/entity/article.md", "ticket_377_article_renderer_envelope_contracts"),
        ("spec/entity/variant.md", "ticket_377_variant_renderer_envelope_contracts"),
    )
    for path, marker in contracts:
        section = _markdown_heading_body(path, 2, "Deterministic Renderer Envelope Contracts")
        lower = section.lower()
        assert "ticket 377" in lower, f"{path} must document the ticket-377 renderer/envelope contract"
        assert "fixture" in lower or "deterministic" in lower, (
            f"{path} must classify renderer/envelope coverage as fixture-backed or deterministic"
        )
        assert "without" in lower and "live" in lower and "calls" in lower, (
            f"{path} must state the contract runs without live source calls"
        )
        if path in {"spec/entity/article.md", "spec/entity/variant.md"}:
            assert marker not in section, f"{path} must not relaunch cargo marker {marker} from routine specs"
            assert "cargo test" not in section, f"{path} must keep renderer/envelope proof in make test"
        else:
            assert marker in section, f"{path} must expose the executable cargo marker {marker}"


ROUTINE_SPEC_PATHS = (
    "spec/entity/article.md",
    "spec/entity/disease-survival-fixture.md",
    "spec/entity/study.md",
    "spec/entity/variant.md",
    "spec/surface/mcp.md",
    "spec/surface/docker-image.md",
    "spec/surface/skills.md",
    "spec/surface/cli-contract-ratchet.md",
    "spec/surface/trial-action-summary.md",
    "spec/surface/ctgov-helper-pivots.md",
)

LIVE_SPEC_PATHS = (
    "spec/entity/diagnostic.md",
    "spec/entity/disease.md",
    "spec/entity/drug.md",
    "spec/entity/gene.md",
    "spec/entity/pathway.md",
    "spec/entity/pgx.md",
    "spec/entity/phenotype.md",
    "spec/entity/protein.md",
    "spec/entity/trial.md",
    "spec/entity/vaers.md",
    "spec/entity/variant-hotspots.md",
    "spec/surface/cli.md",
    "spec/surface/discover.md",
)


def _make_variable_paths(name: str) -> set[str]:
    makefile = _read_repo("Makefile")
    match = re.search(rf"(?ms)^{re.escape(name)} = \\\n(?P<body>.*?)(?=^[A-Z0-9_]+\s*=|^[A-Za-z0-9_.-]+:|\Z)", makefile)
    assert match is not None, f"missing Makefile variable {name}"
    return set(re.findall(r"spec/\S+", match.group("body")))


def _runner_array_paths(name: str) -> list[str]:
    runner = _read_repo("scripts/run-specs.sh")
    match = re.search(rf"(?ms)^{re.escape(name)}=\(\n(?P<body>.*?)^\)", runner)
    assert match is not None, f"missing runner array {name}"
    return re.findall(r"(?:spec|tests)/\S+", match.group("body"))


def test_ticket_442_routine_runner_restores_parallel_isolation_canary() -> None:
    routine = _runner_array_paths("SPEC_ROUTINE_PATHS")
    skills_index = routine.index("spec/surface/skills.md")
    assert routine[skills_index + 1] == "tests/surface/test_parallel_isolation_contract.py"
    assert "spec/surface/cli-contract-ratchet.md" in routine
    assert "spec/entity/disease.md" not in routine
    assert "spec/surface/discover.md" not in routine


def test_ticket_395_routine_and_live_spec_variables_are_disjoint_and_complete() -> None:
    routine = _make_variable_paths("SPEC_ROUTINE_PATHS")
    live = _make_variable_paths("SPEC_LIVE_PATHS")
    spec_files = {str(path.relative_to(REPO_ROOT)) for path in (REPO_ROOT / "spec/entity").glob("*.md")}
    spec_files |= {str(path.relative_to(REPO_ROOT)) for path in (REPO_ROOT / "spec/surface").glob("*.md")}

    assert routine == set(ROUTINE_SPEC_PATHS)
    assert live == set(LIVE_SPEC_PATHS)
    assert not routine & live, "routine and live spec lanes must be disjoint"
    retired = {"spec/surface/request-plan-ratchets.md"}
    assert routine | live == spec_files - retired, "every active entity/surface spec must be explicitly routed"



def test_ticket_395_make_spec_and_spec_pr_run_only_routine_paths() -> None:
    for target_name in ("spec", "spec-pr"):
        block = _make_target_block(target_name)
        assert "scripts/run-specs.sh" in block, f"{target_name} must use the shared binary runner seam"
        assert "$(SPEC_LIVE_PATHS)" not in block, f"{target_name} must not run live upstream paths"
        assert "--deselect" not in block, f"{target_name} must not hide live specs behind deselect/rerun carve-outs"
        assert "--mustmatch-" not in block, f"{target_name} must not invoke the deleted pytest plugin"
        for path in LIVE_SPEC_PATHS:
            assert path not in block, f"{target_name} must not name live spec {path}"



def test_ticket_395_verify_owns_live_specs_and_release_live_smoke_delegates() -> None:
    verify = _make_target_block("verify")
    release_live_smoke = _make_target_block("release-live-smoke")

    assert "--mustmatch-" not in verify, "verify must not invoke the deleted pytest plugin"
    assert "scripts/run-specs.sh" in verify, "verify must run live specs through the shared runner"
    for fragment in (
        "tools/biomcp-ci discover",
        "tools/biomcp-ci search disease",
        "tools/biomcp-ci search article",
        "tools/biomcp-ci variant normalize",
    ):
        assert fragment in verify, "verify must keep the small wrapped live smoke commands"
    assert "$(MAKE) verify" in release_live_smoke, "release-live-smoke should remain a compatibility alias"



def test_ticket_395_mcp_spec_uses_bounded_ready_probe_instead_of_fixed_sleep() -> None:
    mcp = _read_repo("spec/surface/mcp.md")
    for heading in (
        "Probe Routes Stay Lightweight",
        "Remote Workflow Calls Keep BioMCP Text",
        "Read-Only Boundaries and Charted Calls Stay Visible",
    ):
        section = _markdown_heading_body("spec/surface/mcp.md", 2, heading)
        assert "curl -fsS" in section and "/readyz" in section and "/health" in section, (
            f"{heading} must poll readyz with health fallback before connecting"
        )
        assert "for _ in $(seq 1 40)" in section, f"{heading} must use a bounded readiness loop"
    assert "sleep 2" not in mcp, "serve-http specs must not use fixed sleeps before connecting"


def test_ticket_471_mcp_spec_uses_dynamic_ports_for_http_server_blocks() -> None:
    mcp = _read_repo("spec/surface/mcp.md")
    assert "reserve-local-port" in mcp, "MCP serve-http specs must allocate ports dynamically"
    assert not re.search(r"^port=[0-9]+$", mcp, flags=re.MULTILINE), (
        "MCP serve-http specs must not depend on fixed localhost ports"
    )



def test_ticket_378_release_gate_routes_routine_specs_to_standard_gates() -> None:
    makefile = _read_repo("Makefile")
    release_gate_match = re.search(r"^release-gate:\s*(?P<deps>.*)$", makefile, flags=re.MULTILINE)
    assert release_gate_match is not None, "missing Makefile target release-gate"
    release_gate_deps = set(release_gate_match.group("deps").split())

    assert {"lint", "test"}.issubset(release_gate_deps), (
        "release-gate must compose the standard lint and test gates directly"
    )
    assert re.search(
        r"^release-gate: lint test\n"
        r'\t\$\(MAKE\) spec SPEC_PROFILE=release SPEC_BIN="\$\(CURDIR\)/target/release/biomcp"$',
        makefile,
        flags=re.MULTILINE,
    ), "release-gate must run the standard spec gate against the release binary"
    assert "spec-pr" not in release_gate_deps and "verify" not in release_gate_deps, (
        "release-gate must not keep live/cache-backed lanes as routine proof"
    )


def test_ticket_378_docs_describe_routine_and_live_lanes() -> None:
    docs = {
        "spec/README-timings.md": _read_repo("spec/README-timings.md"),
        "architecture/technical/overview.md": _read_repo("architecture/technical/overview.md"),
        "RUN.md": _read_repo("RUN.md"),
        "CONTRIBUTING.md": _read_repo("CONTRIBUTING.md"),
    }

    for path, text in docs.items():
        normalized = re.sub(r"\s+", " ", text.lower())
        assert "make spec" in normalized, f"{path} must name the routine make spec lane"
        assert "make verify" in normalized, f"{path} must name the explicit live verify lane"
        assert "deterministic" in normalized and "offline" in normalized, (
            f"{path} must classify routine validation as offline/deterministic"
        )
        assert "live" in normalized and "opt-in" in normalized, (
            f"{path} must describe public-upstream smoke as an opt-in live lane"
        )
        assert "there is no separate `spec-smoke`" not in normalized
        assert "no separate `spec-smoke` lane" not in normalized


def test_ticket_378_routine_lane_no_longer_depends_on_serialized_live_carveouts() -> None:
    spec_contracts = _make_target_block("spec-contracts")
    runner = _read_repo("scripts/run-specs.sh")
    spec_contracts_surface = spec_contracts + "\n" + runner
    timings = _read_repo("spec/README-timings.md").lower()
    technical_overview = _read_repo("architecture/technical/overview.md").lower()

    assert "spec/surface/mcp.md" in spec_contracts_surface, (
        "spec-contracts should keep local MCP transport proof in routine validation"
    )
    assert "test_parallel_isolation_contract.py" not in spec_contracts, (
        "spec-contracts must not run Python surface contracts; they belong to make test"
    )
    assert "spec/surface/cli.md" not in spec_contracts, (
        "spec-contracts must not keep live CLI/discover/health probes in routine proof"
    )
    assert "pytest spec/entity/ spec/surface/" not in spec_contracts, (
        "spec-contracts must not broad-collect the old live/cache-backed entity and surface corpus"
    )
    assert "--deselect" not in spec_contracts, (
        "spec-contracts must not preserve serialized live carve-outs as routine proof"
    )

    for path in LIVE_SPEC_PATHS:
        assert path not in spec_contracts, f"spec-contracts must not name live spec {path} directly"

    assert "ols4" in timings and "make verify" in timings, (
        "spec/README-timings.md must move public OLS4 confidence to the explicit verify lane"
    )
    assert "ols4" in technical_overview and "make verify" in technical_overview, (
        "architecture/technical/overview.md must move public OLS4 confidence to the explicit verify lane"
    )


def _redundant_live_block_failures(path: str, level: int, headings: tuple[str, ...]) -> list[str]:
    failures: list[str] = []
    for heading in headings:
        section = _markdown_heading_body(path, level, heading)
        invocations = _non_fixture_biomcp_invocations(section)
        if invocations:
            failures.append(f"{path}::{heading}: {invocations}")
    return failures


def _assert_no_redundant_live_block_failures(failures: list[str]) -> None:
    assert not failures, (
        "ticket 379 requires representative sections whose request/source/renderer contracts "
        "already exist to stop executing live public-upstream BioMCP commands. Prune the block, "
        "replace it with deterministic fixture/cargo proof, or classify live confidence in "
        "release-live-smoke instead:\n" + "\n".join(failures)
    )


def test_ticket_379_article_variant_source_specs_prune_redundant_live_blocks() -> None:
    failures = _redundant_live_block_failures(
        "spec/entity/article.md",
        2,
        (
            "Gene Search",
            "Keyword Search",
            "Search Table & Source Ranking",
            "PubTator Annotations",
            "Semantic Scholar Degrades Truthfully Without a Key",
            "Semantic Scholar Source Status Appears in Debug Plans",
            "Authenticated Source Status Is Redacted",
            "Markdown Notes Semantic Scholar Unavailability",
            "Entity Follow-Up",
        ),
    )
    failures.extend(
        _redundant_live_block_failures(
            "spec/entity/variant.md",
            2,
            (
                "Gene-Scoped Variant Search",
                "Search Table Contract",
                "Protein-Filter Narrowing",
                "Residue-Alias Search",
                "Clinical Significance",
                "Population Frequency",
                "Variant Follow-Ups",
                "ID Normalization",
                "Transcript HGVS Normalization Proxies",
                "ERBB2 Transcript HGVS Canary",
            ),
        )
    )
    _assert_no_redundant_live_block_failures(failures)


def test_ticket_379_disease_discover_specs_prune_redundant_live_blocks() -> None:
    failures = _redundant_live_block_failures(
        "spec/entity/disease.md",
        2,
        (
            "Disease Normalization & Search",
            "Genes & Diagnostics",
            "JSON Pivots",
        ),
    )
    failures.extend(
        _redundant_live_block_failures(
            "spec/surface/discover.md",
            2,
            (
                "Alias-Like Free Text Still Resolves to Typed Follow-Ups",
                "Disease-Specific Symptom Phrases Stay Clinically Modest",
                "HPO-Backed Symptom Phrases Should Bridge into Phenotype Search",
                "Relational Queries Redirect Instead of Surfacing Weak Collocation Noise",
                "No-Match Discover Queries Fall Back to Article Search",
            ),
        )
    )
    _assert_no_redundant_live_block_failures(failures)


def _mustmatch_count_prose_lines(section: str, required_terms: tuple[str, ...]) -> list[str]:
    failures: list[str] = []
    for line in section.splitlines():
        stripped = line.strip()
        normalized = stripped.lower()
        if "mustmatch" not in normalized or "showing" not in normalized:
            continue
        if not all(term in normalized for term in required_terms):
            continue
        if any(token in stripped for token in ("[0-9]", "\\d")):
            failures.append(stripped)
    return failures


def test_ticket_379_target_specs_drop_count_prose_trivia() -> None:
    forbidden = (
        (
            "spec/entity/disease.md",
            2,
            "Genes & Diagnostics",
            ("diagnostic",),
        ),
        (
            "spec/entity/disease.md",
            2,
            "NIH Funding Context",
            ("grant",),
        ),
    )
    failures = []
    for path, level, heading, required_terms in forbidden:
        section = _markdown_heading_body(path, level, heading)
        for line in _mustmatch_count_prose_lines(section, required_terms):
            failures.append(f"{path}::{heading} still pins numeric count/prose assertion {line!r}")

    assert not failures, (
        "ticket 379 should relax count/prose pins that fail on upstream total drift or copy edits "
        "rather than BioMCP behavior regressions:\n" + "\n".join(failures)
    )


def test_ticket_379_timing_docs_record_pruned_ownership() -> None:
    timings = re.sub(r"\s+", " ", _read_repo("spec/README-timings.md").lower())

    for fragment in (
        "ticket 379",
        "prun",
        "spec/entity/article.md",
        "spec/entity/variant.md",
        "spec/entity/disease.md",
        "spec/surface/discover.md",
        "deterministic",
        "release-live-smoke",
    ):
        assert fragment in timings, (
            "spec/README-timings.md must record the ticket-379 pruning decision, including the "
            "representative target specs, deterministic replacement ownership, and explicit "
            f"release-live-smoke ownership; missing {fragment!r}"
        )
