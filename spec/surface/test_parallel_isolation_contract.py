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


def _assert_make_target_serializes_spec_path(target_name: str, block: str, path: str) -> None:
    assert "$(SPEC_XDIST_ARGS)" in block, f"{target_name} should keep its main parallel xdist leg"
    assert f"--deselect {path}" in block, (
        f"Makefile target {target_name} must remove {path} from the main parallel xdist pool "
        "before rerunning it in a serialized or fixture-backed leg"
    )
    spec_commands = re.findall(r"pytest[^\n]*", block)
    assert any(path in command and "$(SPEC_XDIST_ARGS)" not in command for command in spec_commands), (
        f"{target_name} must run {path} outside the main $(SPEC_XDIST_ARGS) pool"
    )


def _has_base_url_probe(text: str) -> bool:
    return bool(
        re.search(r"curl[^\n]*\$(?:\{base_url\}|base_url)", text)
        or re.search(r"wget[^\n]*\$(?:\{base_url\}|base_url)", text)
        or re.search(r"urllib\.request\.[A-Za-z_]+\([^\n]*base_url", text)
        or ("/dev/tcp/" in text and "base_url" in text)
    )


def test_wikipathways_parallel_contract_serializes_shared_mock_env() -> None:
    context = _rust_function_block(
        "src/cli/search_all/tests/dispatch.rs",
        "dispatch_section_pathway_surfaces_sanitized_wikipathways_404_without_timeout",
    )
    preamble = context.split(
        "async fn dispatch_section_pathway_surfaces_sanitized_wikipathways_404_without_timeout(",
        1,
    )[0]

    assert "#[tokio::test]" in preamble, "expected the named flaky function to remain a tokio test"
    assert "#[serial_test::serial]" in preamble, (
        "the WikiPathways search-all flake is an env-mutation test; it must declare an explicit "
        "serial guard on the named test so nextest parallelism cannot swap another test's "
        "BIOMCP_*_BASE values into this warning-path assertion"
    )
    assert any(
        marker in context
        for marker in (
            "with_no_cache(",
            "with_no_http_cache(",
        )
    ), (
        "the WikiPathways search-all warning-path test routes Reactome and KEGG through the shared "
        "HTTP cache/client; it must disable the persistent HTTP cache inside the named test (e.g. "
        "via `crate::sources::with_no_cache(true, ...)`) so cache-disk contention from other "
        "parallel tests cannot push the 12s section timeout and turn the assertion into a "
        "'pathway search timed out' message that no longer mentions wikipathways"
    )


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
    context = _rust_function_block(
        "src/entities/drug/get/tests.rs",
        "resolve_trial_aliases_retries_after_transient_lookup_failure",
    )

    assert any(
        marker in context
        for marker in (
            "with_no_http_cache(",
            'set_env_var("XDG_CACHE_HOME"',
            'set_env_var("BIOMCP_CACHE_DIR"',
            "#[serial_test::serial]",
        )
    ), (
        "the transient trial-alias retry test swaps BIOMCP_MYCHEM_BASE between mock servers; it "
        "must isolate or disable the shared HTTP cache/client state inside the named test so "
        "another test's alias response cannot satisfy this assertion"
    )


def test_diagnostic_regulatory_contract_uses_private_openfda_cache() -> None:
    context = _rust_function_block(
        "src/entities/diagnostic/mod.rs",
        "get_regulatory_uses_alias_queries_and_dedupes_pma_supplements",
    )

    assert any(
        marker in context
        for marker in (
            "with_no_http_cache(",
            'set_env_var("XDG_CACHE_HOME"',
            'set_env_var("BIOMCP_CACHE_DIR"',
            "#[serial_test::serial]",
        )
    ), (
        "the diagnostic regulatory overlay test points OpenFDA at a mock server; it must isolate "
        "or disable the shared HTTP cache/client path inside the named test so nextest "
        "parallelism cannot replay a different PMA/510(k) response into this alias-dedupe "
        "assertion"
    )


def test_protein_complexes_spec_lane_leaves_the_parallel_xdist_pool() -> None:
    spec_target = _make_target_block("spec")
    spec_pr_target = _make_target_block("spec-pr")
    timings = _read_repo("spec/README-timings.md")
    technical_overview = _read_repo("architecture/technical/overview.md")

    for target_name, block in (("spec", spec_target), ("spec-pr", spec_pr_target)):
        _assert_make_target_serializes_spec_path(target_name, block, "spec/entity/protein.md")

    assert re.search(r"protein.*serial", timings, flags=re.IGNORECASE | re.DOTALL), (
        "spec/README-timings.md must document that the protein complexes canary runs in a "
        "serialized spec partition so the lane topology matches reality"
    )
    assert re.search(r"protein.*serial", technical_overview, flags=re.IGNORECASE | re.DOTALL), (
        "architecture/technical/overview.md must describe the serialized protein carve-out so the "
        "repo architecture matches the actual spec lane"
    )


def test_ols4_disease_discover_spec_lane_leaves_the_parallel_xdist_pool() -> None:
    spec_target = _make_target_block("spec")
    spec_pr_target = _make_target_block("spec-pr")
    timings = _read_repo("spec/README-timings.md")
    technical_overview = _read_repo("architecture/technical/overview.md")

    for path, expected_headings in OLS4_HEAVY_SPEC_HEADINGS.items():
        headings = _markdown_h2_headings(path)
        missing = set(expected_headings) - headings
        assert not missing, f"{path} is missing OLS4-heavy heading contract entries: {sorted(missing)}"

    for target_name, block in (("spec", spec_target), ("spec-pr", spec_pr_target)):
        for path in OLS4_HEAVY_SPEC_HEADINGS:
            _assert_make_target_serializes_spec_path(target_name, block, path)

    assert re.search(r"OLS4.*serial", timings, flags=re.IGNORECASE | re.DOTALL), (
        "spec/README-timings.md must document that OLS4-heavy disease/discover canaries run "
        "in the serialized spec partition or are otherwise fixture-backed"
    )
    assert re.search(r"disease.*discover.*serial", technical_overview, flags=re.IGNORECASE | re.DOTALL), (
        "architecture/technical/overview.md must describe the serialized OLS4 disease/discover "
        "carve-out so the repo architecture matches the actual spec lane"
    )
