from __future__ import annotations

import json
import os
import subprocess
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
WRAPPER_SCRIPT = REPO_ROOT / "tools" / "check-quality-ratchet.sh"
EXCEPTION_REGISTRY = REPO_ROOT / "tools" / "cli-surface-contract-exceptions.json"


def test_quality_ratchet_runs_whole_surface_cli_contract(tmp_path: Path) -> None:
    output_dir = tmp_path / "ratchet-output"
    env = os.environ.copy()
    env["QUALITY_RATCHET_OUTPUT_DIR"] = str(output_dir)

    result = subprocess.run(
        ["bash", str(WRAPPER_SCRIPT)],
        cwd=REPO_ROOT,
        env=env,
        capture_output=True,
        text=True,
        check=False,
    )

    assert result.returncode == 0, result.stderr
    summary_path = output_dir / "quality-ratchet-summary.json"
    assert summary_path.exists(), "quality ratchet did not write summary artifact"
    summary = json.loads(summary_path.read_text(encoding="utf-8"))

    assert "cli_surface_contract" in summary, (
        "quality ratchet must include the whole-surface CLI contract lane so "
        "make lint fails on help/list/docs/spec/JSON drift"
    )
    assert summary["cli_surface_contract"]["status"] == "pass"

    detail_path = output_dir / "quality-ratchet-cli-surface-contract.json"
    assert detail_path.exists(), "whole-surface CLI contract artifact is missing"
    detail = json.loads(detail_path.read_text(encoding="utf-8"))
    assert detail["status"] == "pass"
    assert detail["exception_registry"] == "tools/cli-surface-contract-exceptions.json"
    assert detail["checks"] == [
        "public_flags_and_value_aliases_documented",
        "list_and_reference_docs_cover_public_commands",
        "json_entity_surfaces_include_next_commands_or_exception",
        "copy_paste_examples_are_shell_safe",
    ]


def test_cli_surface_contract_exception_registry_names_initial_exceptions() -> None:
    assert EXCEPTION_REGISTRY.exists(), (
        "whole-surface CLI contract exceptions must be source-controlled, "
        "not hard-coded as inline skips"
    )
    registry = json.loads(EXCEPTION_REGISTRY.read_text(encoding="utf-8"))

    assert registry["schema"] == "biomcp-cli-surface-contract-exceptions-v1"
    entries = registry["entries"]
    by_command = {entry["command"]: entry for entry in entries}

    for command in [
        "biomcp cache path",
        "biomcp --json list",
        "biomcp --json version",
        "biomcp --json search all --counts-only",
    ]:
        entry = by_command[command]
        assert entry["reason"].strip(), command
        assert entry["owner_test"].startswith("tests/test_cli_surface_contract_ratchet.py::"), command

    assert by_command["biomcp cache path"]["exception"] == "plain_text_operator_path"
    assert by_command["biomcp --json list"]["exception"] == "command_reference_payload"
    assert by_command["biomcp --json version"]["exception"] == "release_identity_payload"
    assert (
        by_command["biomcp --json search all --counts-only"]["exception"]
        == "current_counts_only_shape"
    )
