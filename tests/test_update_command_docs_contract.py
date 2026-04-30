"""Docs contract for the fail-closed `biomcp update` checksum behavior.

These assertions pin the docs surface for the self-update verification
requirement and the explicit `--allow-missing-checksum` UNSAFE override.
The runtime change lives in `src/cli/update.rs`; these tests catch
docs/runtime drift where the runtime is fail-closed but a doc surface
still describes the old fail-open behavior.
"""

from __future__ import annotations

import os
import re
import subprocess
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]


def _read(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def _release_bin() -> Path:
    configured = os.environ.get("BIOMCP_BIN")
    return Path(configured) if configured else REPO_ROOT / "target" / "release" / "biomcp"


def _render_list() -> str:
    binary = _release_bin()
    assert binary.exists(), f"missing release binary for rendered list contract: {binary}"
    result = subprocess.run(
        [str(binary), "list"],
        cwd=REPO_ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout


def _update_ops_lines(text: str) -> list[str]:
    return [
        line.strip()
        for line in text.splitlines()
        if line.strip().startswith("- `update ")
    ]


def _assert_update_reference_contract(label: str, text: str) -> str:
    lines = _update_ops_lines(text)
    assert lines, f"{label} must include an update Ops command line"

    for line in lines:
        lower = line.lower()
        has_checksum_concept = "checksum" in lower or re.search(
            r"sha-?256", line, flags=re.IGNORECASE
        )
        if (
            "update" in line
            and "--check" in line
            and "--allow-missing-checksum" in line
            and has_checksum_concept
            and "unsafe" in lower
        ):
            return line

    joined = "\n".join(lines)
    raise AssertionError(
        f"{label} update Ops line must document --check, "
        "--allow-missing-checksum, checksum/SHA256 verification, and unsafe "
        f"override wording; saw:\n{joined}"
    )


def test_update_list_reference_and_rendered_list_describe_checksum_override() -> None:
    source_line = _assert_update_reference_contract(
        "src/cli/list_reference.md",
        _read("src/cli/list_reference.md"),
    )
    rendered_line = _assert_update_reference_contract("biomcp list", _render_list())

    assert "--allow-missing-checksum" in source_line
    assert "--allow-missing-checksum" in rendered_line


def test_update_list_reference_contract_rejects_stale_update_line() -> None:
    stale_ops = "## Ops\n\n- `update [--check]` - self-update from GitHub releases\n"

    with pytest.raises(AssertionError, match="--allow-missing-checksum"):
        _assert_update_reference_contract("synthetic stale list reference", stale_ops)


def test_update_command_docs_describe_verification_requirement() -> None:
    cli_reference = _read("docs/user-guide/cli-reference.md")

    assert "biomcp update [--check] [--allow-missing-checksum]" in cli_reference, (
        "docs/user-guide/cli-reference.md must list the new flag in the "
        "command grammar so the docs match runtime"
    )
    assert re.search(r"SHA-?256", cli_reference, flags=re.IGNORECASE), (
        "cli-reference.md must name SHA256 verification for biomcp update"
    )
    assert "checksum" in cli_reference.lower(), (
        "cli-reference.md must describe checksum verification for biomcp update"
    )
    assert "--allow-missing-checksum" in cli_reference, (
        "cli-reference.md must name the unsafe override flag in prose so "
        "operators can find it from the docs surface"
    )


def test_update_troubleshooting_describes_failclosed_and_unsafe_override() -> None:
    troubleshooting = _read("docs/troubleshooting.md")

    assert "biomcp update" in troubleshooting
    assert "--allow-missing-checksum" in troubleshooting, (
        "troubleshooting.md must point operators at the unsafe override when "
        "a legitimate release ships without a sidecar"
    )
    assert "UNSAFE" in troubleshooting, (
        "troubleshooting.md must mark the override UNSAFE so operators "
        "do not turn it on casually"
    )
    assert re.search(r"fail(?:s|ed|ing)?[- ]closed", troubleshooting, re.IGNORECASE), (
        "troubleshooting.md must describe the new fail-closed behavior so "
        "operators know an update can stop on missing sidecar"
    )


def test_architecture_cli_reference_lists_update_verification_flag() -> None:
    architecture = _read("architecture/ux/cli-reference.md")

    assert "biomcp update [--check] [--allow-missing-checksum]" in architecture, (
        "architecture/ux/cli-reference.md ops grammar must track the new "
        "flag so the durable architecture doc matches runtime"
    )
