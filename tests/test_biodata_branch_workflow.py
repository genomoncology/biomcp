from __future__ import annotations

from pathlib import Path
import re
import os
import subprocess
import tempfile

import pytest


ROOT = Path(__file__).resolve().parents[1]
RETIRED_WORKFLOW = ROOT / ".github/workflows/biodata-1.0.yml"
RUNNER = ROOT / "tools/check-biodata-1.0"
OFFLINE_CHECKER = ROOT / "tools/check-offline-network"
TEST_FILES = (
    "tests/test_biodata_boundary.py",
    "tests/test_source_package_boundary.py",
    "tests/test_ctgov_trial_search_detail_reuse.py",
    "tests/test_nci_filter_transport.py",
    "tests/test_biodata_branch_workflow.py",
)
RUST_TESTS = (
    "entities::trial::get::tests::ctgov_product_core_ignores_every_legacy_core_field",
    "entities::trial::get::tests::ctgov_product_outcomes_ignore_raw_legacy_study_mutation",
    "entities::trial::get::tests::nci_product_conversion_checks_enrollment_and_preserves_source_presence",
    "entities::trial::get::tests::outcome_product_preserves_grouped_values_and_every_section_state",
    "entities::trial::get::tests::product_section_state_preserves_all_four_states",
    "entities::trial::get::tests::shared_core_conversion_preserves_order_and_derives_compatibility_fields",
    "entities::trial::get::tests::ctgov_product_uses_shared_directory_for_ordered_locations_and_states",
    "entities::trial::get::tests::shared_directory_views_authorize_output_and_redact_diagnostics",
    "cli::trial::dispatch::site_directory_tests::location_page_filters_sites_and_site_contacts_but_keeps_central_first",
    "cli::trial::dispatch::site_directory_tests::standalone_pagination_wrapper_serializes_only_the_current_directory_page",
    "sources::clinicaltrials::tests::parsing::legacy_source_aggregates_redact_ignored_contact_sentinels",
    "sources::clinicaltrials::tests::parsing::biodata_detail_wrapper_redacts_field_distinct_site_values_but_shared_getters_retain_them",
    "entities::trial::search::ctgov::tests::raw_page_debug_redacts_nested_legacy_site_values",
    "mcp::shell::typed_get_tests::cli_typed_and_raw_trial_get_preserve_planned_outcome_values_and_states",
    "mcp::shell::typed_get_tests::cli_typed_and_raw_trial_get_return_exact_structured_references",
    "mcp::shell::typed_get_tests::cli_typed_and_raw_nci_trial_get_preserve_assignments_and_outcome_state",
    "mcp::shell::typed_get_tests::typed_and_raw_trial_get_return_exact_age_objects",
    "mcp::shell::typed_get_tests::cli_typed_and_raw_trial_get_share_directory_contacts_locations_and_states",
    "render::markdown::trial::tests::planned_outcome_markdown_preserves_groups_order_text_and_states",
    "render::markdown::trial::tests::response_markdown_explains_selected_section_states",
    "render::markdown::trial::tests::trial_markdown_renders_coordinates_and_sanitizes_unnamed_contacts",
)
EXPECTED_ISOLATED_INVOCATION = (
    '"$ROOT/tools/check-offline-network" true\n'
    'python3 "$ROOT/tools/check-biodata-boundary.py" --root "$ROOT"\n'
    'python3 "$ROOT/tools/check-source-capture-receipts.py" --root "$ROOT/testdata/sources"\n'
    'for test_name in "${RUST_TESTS[@]}"; do\n'
    '  cargo test --locked --offline --no-default-features --lib "$test_name" -- --exact\n'
    "done\n"
    'exec env -u NCI_API_KEY BIOMCP_BIN="$biomcp_bin" \\\n'
    '  uv run --no-sync pytest --basetemp /tmp/pytest "${TEST_FILES[@]}"\n'
)
FORBIDDEN_COMMANDS = (
    "make lint",
    "make test",
    "make spec",
    "make full-feature-check",
    "make release-gate",
    "curl ",
    "wget ",
    "uv publish",
    "cargo publish",
    "git push",
    "deploy",
)


def _runner_test_files(runner: str) -> tuple[str, ...]:
    match = re.search(r"readonly TEST_FILES=\(\n(?P<body>.*?)\n\)", runner, re.DOTALL)
    if match is None:
        return ()
    return tuple(re.findall(r'^\s+"([^"]+)"$', match.group("body"), re.MULTILINE))


def _runner_rust_tests(runner: str) -> tuple[str, ...]:
    match = re.search(r"readonly RUST_TESTS=\(\n(?P<body>.*?)\n\)", runner, re.DOTALL)
    if match is None:
        return ()
    return tuple(re.findall(r'^\s+"([^"]+)"$', match.group("body"), re.MULTILINE))


def _current_process_has_verified_offline_isolation() -> bool:
    try:
        probe = subprocess.run(
            [str(OFFLINE_CHECKER), "true"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            timeout=5,
        )
    except (OSError, subprocess.TimeoutExpired):
        return False
    return probe.returncode == 0


def _runner_violations(runner: str) -> list[str]:
    violations: list[str] = []
    for value in (
        "unset NCI_API_KEY",
        "env -u NCI_API_KEY",
        "tools/check-biodata-boundary.py",
        "tools/check-offline-network",
        '[[ ! -x "$biomcp_bin" ]]',
    ):
        if value not in runner:
            violations.append(f"missing focused runner control: {value}")
    if runner.count(EXPECTED_ISOLATED_INVOCATION) != 1:
        violations.append("isolated focused runner invocation changed")
    if "tools/run-offline" in runner or "bwrap" in runner:
        violations.append("BioMCP runner must not establish isolation")
    if '[[ "${BIOMCP_OFFLINE_NETWORK:-0}" != 1 ]]' not in runner:
        violations.append("already-isolated mode must require the verified namespace")
    if _runner_test_files(runner) != TEST_FILES:
        violations.append("focused runner test selection changed")
    if _runner_rust_tests(runner) != RUST_TESTS:
        violations.append("focused runner Rust test selection changed")
    if "cargo build" in runner or "cargo run" in runner:
        violations.append("focused runner must require an existing binary")
    for command in FORBIDDEN_COMMANDS:
        if command in runner.lower():
            violations.append(f"forbidden focused runner command: {command}")
    return violations


def test_biomcp_keeps_only_the_local_focused_runner() -> None:
    assert not RETIRED_WORKFLOW.exists()
    assert not any(
        "biodata/biomcp-1.0" in path.read_text(encoding="utf-8")
        for path in (ROOT / ".github/workflows").iterdir()
        if path.suffix in {".yml", ".yaml"}
    )
    runner = RUNNER.read_text(encoding="utf-8")
    assert not _runner_violations(runner)
    assert RUNNER.stat().st_mode & 0o111


@pytest.mark.parametrize(
    ("old", "new"),
    (
        ("unset NCI_API_KEY", "true # retain NCI_API_KEY"),
        ("tools/check-offline-network", "true # trust marker"),
        (TEST_FILES[2], "tests/test_live_provider.py"),
        (RUST_TESTS[0], "live_provider_smoke"),
        ("uv run --no-sync pytest", "make test && uv run --no-sync pytest"),
        ('[[ ! -x "$biomcp_bin" ]]', '[[ -x "$biomcp_bin" ]]'),
        ("--basetemp /tmp/pytest", "--basetemp /var/tmp/pytest"),
        (' "${TEST_FILES[@]}"', ""),
        (' "${TEST_FILES[@]}"', ' "${TEST_FILES[@]}" tests/'),
    ),
)
def test_representative_runner_mutations_are_rejected(old: str, new: str) -> None:
    runner = RUNNER.read_text(encoding="utf-8")
    assert old in runner
    assert _runner_violations(runner.replace(old, new, 1))


def test_forged_offline_marker_fails_in_the_normal_namespace() -> None:
    if _current_process_has_verified_offline_isolation():
        pytest.skip("host-only forged-marker check cannot run inside offline isolation")
    cache = ROOT / ".cache"
    cache.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(dir=cache) as directory:
        binary = Path(directory) / "biomcp"
        binary.write_text("#!/usr/bin/env bash\nexit 0\n", encoding="utf-8")
        binary.chmod(0o755)
        environment = os.environ.copy()
        environment.update(
            BIOMCP_BIN=str(binary),
            BIOMCP_OFFLINE_NETWORK="1",
        )
        result = subprocess.run(
            [str(RUNNER), "--already-isolated"],
            cwd=ROOT,
            env=environment,
            capture_output=True,
            text=True,
        )
    assert result.returncode != 0
    assert "offline privilege isolation failed" in result.stdout + result.stderr


def test_host_only_skip_requires_a_successful_live_isolation_probe(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    returncodes = iter((1, 0))
    calls: list[tuple[object, ...]] = []

    def probe(*args: object, **kwargs: object) -> subprocess.CompletedProcess[bytes]:
        calls.append(args)
        return subprocess.CompletedProcess(args, next(returncodes))

    monkeypatch.setattr(
        subprocess,
        "run",
        probe,
    )
    assert not _current_process_has_verified_offline_isolation()
    assert _current_process_has_verified_offline_isolation()
    assert calls == [
        ([str(OFFLINE_CHECKER), "true"],),
        ([str(OFFLINE_CHECKER), "true"],),
    ]
