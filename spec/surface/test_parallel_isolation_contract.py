from __future__ import annotations

import re
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]


def _read_repo(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def _line_window(path: str, needle: str, *, before: int = 6, after: int = 50) -> str:
    lines = _read_repo(path).splitlines()
    for index, line in enumerate(lines):
        if needle in line:
            start = max(0, index - before)
            end = min(len(lines), index + after)
            return "\n".join(lines[start:end])
    raise AssertionError(f"{needle!r} not found in {path}")


def _make_target_block(name: str) -> str:
    makefile = _read_repo("Makefile")
    match = re.search(
        rf"(?ms)^{re.escape(name)}:\n(.*?)(?=^[A-Za-z0-9_.-]+:|\Z)",
        makefile,
    )
    assert match is not None, f"missing Makefile target {name}"
    return match.group(1)


def _assert_isolation_comment(context: str, *, resource_terms: tuple[str, ...]) -> None:
    lowered = context.lower()
    assert "shared" in lowered, (
        "expected an inline comment naming the shared resource being protected so future "
        "contributors do not remove the isolation guard by accident"
    )
    assert any(term.lower() in lowered for term in resource_terms), (
        f"expected the inline comment to name one of {resource_terms} as the shared resource"
    )


def test_wikipathways_parallel_contract_serializes_shared_mock_env() -> None:
    context = _line_window(
        "src/cli/search_all.rs",
        "dispatch_section_pathway_surfaces_sanitized_wikipathways_404_without_timeout",
    )

    assert "#[serial_test::serial]" in context, (
        "the WikiPathways search-all flake is an env-mutation test; it must declare an explicit "
        "serial guard so nextest parallelism cannot swap another test's BIOMCP_*_BASE values "
        "into this warning-path assertion"
    )
    _assert_isolation_comment(
        context,
        resource_terms=("WikiPathways", "mock env", "BIOMCP_WIKIPATHWAYS_BASE"),
    )


def test_vaers_fixture_contract_waits_for_live_http_readiness() -> None:
    script = _read_repo("spec/fixtures/setup-vaers-spec-fixture.sh")
    before_exports = script.split("printf 'export BIOMCP_VAERS_BASE", 1)[0]
    readiness_tail = before_exports.split('base_url="$(cat "$ready_file")"', 1)[-1]

    assert any(
        marker in readiness_tail
        for marker in ("curl ", "wget ", "urllib.request", "/dev/tcp/")
    ), (
        "the VAERS fixture setup must perform a real HTTP readiness probe after choosing the "
        "base URL and before exporting BIOMCP_VAERS_BASE, otherwise spec-pr can still race the "
        "background server under xdist load"
    )
    _assert_isolation_comment(
        readiness_tail,
        resource_terms=("startup race", "fixture server", "parallel spec"),
    )


def test_trial_alias_retry_contract_uses_private_cache_or_no_cache_mode() -> None:
    context = _line_window(
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
        "must isolate or disable the shared HTTP cache/client state so another test's alias "
        "response cannot satisfy this assertion"
    )
    _assert_isolation_comment(
        context,
        resource_terms=("cache", "alias", "BIOMCP_MYCHEM_BASE"),
    )


def test_diagnostic_regulatory_contract_uses_private_openfda_cache() -> None:
    context = _line_window(
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
        "or disable the shared HTTP cache/client path so nextest parallelism cannot replay a "
        "different PMA/510(k) response into this alias-dedupe assertion"
    )
    _assert_isolation_comment(
        context,
        resource_terms=("OpenFDA", "cache", "regulatory"),
    )


def test_protein_complexes_spec_lane_leaves_the_parallel_xdist_pool() -> None:
    spec_target = _make_target_block("spec")
    spec_pr_target = _make_target_block("spec-pr")
    timings = _read_repo("spec/README-timings.md")
    technical_overview = _read_repo("architecture/technical/overview.md")

    for target_name, block in (("spec", spec_target), ("spec-pr", spec_pr_target)):
        assert "spec/entity/protein.md" in block, (
            f"Makefile target {target_name} must carve spec/entity/protein.md out into its own "
            "serialized leg so the live ComplexPortal complexes canary is not left in the main "
            "xdist pool"
        )
        protein_lines = [line for line in block.splitlines() if "spec/entity/protein.md" in line]
        assert protein_lines, f"{target_name} should contain a protein-specific spec command"
        assert any("$(SPEC_XDIST_ARGS)" not in line for line in protein_lines), (
            f"{target_name} must run the protein-specific leg outside the main parallel "
            "$(SPEC_XDIST_ARGS) pool"
        )

    assert re.search(r"protein.*serial", timings, flags=re.IGNORECASE | re.DOTALL), (
        "spec/README-timings.md must document that the protein complexes canary runs in a "
        "serialized spec partition so the lane topology matches reality"
    )
    assert re.search(r"protein.*serial", technical_overview, flags=re.IGNORECASE | re.DOTALL), (
        "architecture/technical/overview.md must describe the serialized protein carve-out so the "
        "repo architecture matches the actual spec lane"
    )
