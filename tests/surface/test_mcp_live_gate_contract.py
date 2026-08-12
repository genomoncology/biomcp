from __future__ import annotations

import re
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
MAKEFILE = (REPO_ROOT / "Makefile").read_text(encoding="utf-8")
MCP_TESTS = (REPO_ROOT / "tests/rmcp_client_contract.rs").read_text(encoding="utf-8")
LIVE_FULL_CONTRACT_TESTS = (
    "rmcp_child_process_client_verifies_stdio_full_contract",
    "rmcp_streamable_http_client_verifies_full_contract",
)


def _make_target(name: str) -> str:
    match = re.search(
        rf"^{re.escape(name)}:\n(?P<body>(?:\t.*\n)+)",
        MAKEFILE,
        flags=re.MULTILINE,
    )
    assert match is not None, f"missing make target: {name}"
    return match.group("body")


def test_live_mcp_full_contracts_are_excluded_from_routine_test() -> None:
    routine = _make_target("test")
    assert "cargo nextest run --archive-file" in routine
    assert "--run-ignored" not in routine

    for test_name in LIVE_FULL_CONTRACT_TESTS:
        annotation = re.compile(
            r'#\[tokio::test\(flavor = "multi_thread"\)\]\n'
            r'#\[ignore = "live external-service full contract; run through make verify"\]\n'
            rf"async fn {re.escape(test_name)}\(",
        )
        assert annotation.search(MCP_TESTS), f"{test_name} must remain outside make test"


def test_verify_runs_the_ignored_live_mcp_full_contract_target() -> None:
    verify = _make_target("verify")
    assert (
        "$(CARGO_WITH_IDENTITY) nextest run --release --test rmcp_client_contract --run-ignored only"
        in verify
    )
